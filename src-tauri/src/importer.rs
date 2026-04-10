use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use walkdir::WalkDir;

use crate::db::{self, WorkRecord};
use crate::error::AppError;
use crate::scanner;
use crate::settings;
use crate::template::{self, WorkMetadata};
use crate::thumbnail;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub source_path: String,
    pub title: String,
    pub artist: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub circle: Option<String>,
    pub origin: Option<String>,
    pub mode: ImportMode,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub destination_path: String,
    pub page_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedMetadata {
    pub title: String,
    pub artist: Option<String>,
}

#[derive(Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ImportMode {
    Copy,
    Move,
}

pub fn parse_folder_name(folder_name: &str) -> ParsedMetadata {
    // Pattern: [artist] title
    if let Some(rest) = folder_name.strip_prefix('[') {
        if let Some(close) = rest.find(']') {
            let artist = rest[..close].trim();
            let title = rest[close + 1..].trim();
            if !artist.is_empty() && !title.is_empty() {
                return ParsedMetadata {
                    title: title.to_string(),
                    artist: Some(artist.to_string()),
                };
            }
        }
    }

    // Pattern: artist - title
    if let Some(sep_pos) = folder_name.find(" - ") {
        let artist = folder_name[..sep_pos].trim();
        let title = folder_name[sep_pos + 3..].trim();
        if !artist.is_empty() && !title.is_empty() {
            return ParsedMetadata {
                title: title.to_string(),
                artist: Some(artist.to_string()),
            };
        }
    }

    ParsedMetadata {
        title: folder_name.to_string(),
        artist: None,
    }
}

pub fn list_images_in_folder(folder_path: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut images: Vec<PathBuf> = std::fs::read_dir(folder_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && scanner::is_image_file(path))
        .collect();
    images.sort_by(|a, b| {
        let a_name = a.file_name().unwrap_or_default().to_string_lossy();
        let b_name = b.file_name().unwrap_or_default().to_string_lossy();
        natord::compare(&a_name, &b_name)
    });
    Ok(images)
}

pub fn preview_import_path(
    library_root: &Path,
    template_str: &str,
    metadata: &WorkMetadata,
) -> String {
    let path = template::resolve_work_path(library_root, template_str, metadata);
    path.to_string_lossy().to_string()
}

pub fn import_work(
    request: &ImportRequest,
    conn: &rusqlite::Connection,
    library_id: &str,
    library_root: &Path,
) -> Result<ImportResult, AppError> {
    let source = Path::new(&request.source_path);
    if !source.is_dir() {
        return Err(AppError::ImportError(
            "ソースパスがディレクトリではありません".to_string(),
        ));
    }

    let images = list_images_in_folder(source)?;
    if images.is_empty() {
        return Err(AppError::ImportError(
            "フォルダ内に画像ファイルがありません".to_string(),
        ));
    }

    let resource_mode = settings::get_resource_mode(conn, library_id)?;
    let thumb = thumbnail::generate_thumbnail(&images[0])?;
    let page_count = images.len();

    if resource_mode == "metadata_only" {
        let source_str = source.to_string_lossy().to_string();

        db::insert_work(
            conn,
            &WorkRecord {
                library_id,
                title: &request.title,
                path: &source_str,
                work_type: "folder",
                page_count: page_count as i32,
                thumbnail: &thumb,
                artist: request.artist.as_deref(),
                year: request.year,
                genre: request.genre.as_deref(),
                circle: request.circle.as_deref(),
                origin: request.origin.as_deref(),
            },
        )?;

        return Ok(ImportResult {
            destination_path: source_str,
            page_count,
        });
    }

    let template_str = settings::get_directory_template(conn, library_id)?.ok_or_else(|| {
        AppError::ImportError("ディレクトリテンプレートが設定されていません".to_string())
    })?;

    let type_label = settings::resolve_type_label(conn, library_id, "folder")?;
    let metadata = WorkMetadata {
        title: request.title.clone(),
        artist: request.artist.clone(),
        year: request.year,
        genre: request.genre.clone(),
        circle: request.circle.clone(),
        origin: request.origin.clone(),
        work_type: Some(type_label),
    };

    let dest = template::resolve_unique_work_path(library_root, &template_str, &metadata);

    if paths_overlap(source, &dest) {
        return Err(AppError::ImportError(
            "取り込み元と取り込み先が重複しています".to_string(),
        ));
    }

    std::fs::create_dir_all(&dest)?;

    let rollback = |dest: &Path| {
        let _ = std::fs::remove_dir_all(dest);
    };

    if let Err(e) = copy_images_to_dest(&images, &dest) {
        rollback(&dest);
        return Err(e);
    }

    let dest_str = dest.to_string_lossy().to_string();

    if let Err(e) = db::insert_work(
        conn,
        &WorkRecord {
            library_id,
            title: &request.title,
            path: &dest_str,
            work_type: "folder",
            page_count: page_count as i32,
            thumbnail: &thumb,
            artist: request.artist.as_deref(),
            year: request.year,
            genre: request.genre.as_deref(),
            circle: request.circle.as_deref(),
            origin: request.origin.as_deref(),
        },
    ) {
        rollback(&dest);
        return Err(e);
    }

    if request.mode == ImportMode::Move {
        for image in &images {
            let _ = std::fs::remove_file(image);
        }
        let _ = std::fs::remove_dir(source);
    }

    Ok(ImportResult {
        destination_path: dest_str,
        page_count,
    })
}

fn paths_overlap(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

fn copy_images_to_dest(images: &[PathBuf], dest: &Path) -> Result<(), AppError> {
    for image in images {
        let file_name = image
            .file_name()
            .ok_or_else(|| AppError::ImportError("無効なファイル名".to_string()))?;
        let dest_file = dest.join(file_name);
        std::fs::copy(image, &dest_file)?;
    }
    Ok(())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredFolder {
    pub path: String,
    pub folder_name: String,
    pub image_count: usize,
    pub parsed_metadata: ParsedMetadata,
    pub already_registered: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkippedFolder {
    pub path: String,
    pub folder_name: String,
    pub image_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverResult {
    pub folders: Vec<DiscoveredFolder>,
    pub skipped_folders: Vec<SkippedFolder>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum DiscoverProgress {
    Scanning { scanned_dirs: usize },
    Completed { found: usize },
}

pub fn discover_image_folders(
    root: &Path,
    conn: &rusqlite::Connection,
    library_id: &str,
    on_progress: &Channel<DiscoverProgress>,
) -> Result<DiscoverResult, AppError> {
    let mut candidates: Vec<(PathBuf, usize)> = Vec::new();
    let mut scanned_dirs = 0usize;

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() {
            continue;
        }

        scanned_dirs += 1;
        if scanned_dirs.is_multiple_of(50) {
            let _ = on_progress.send(DiscoverProgress::Scanning { scanned_dirs });
        }

        let dir_path = entry.path();
        let image_count = scanner::count_direct_images(dir_path);
        if image_count > 0 {
            candidates.push((dir_path.to_path_buf(), image_count));
        }
    }

    let result = partition_leaf_folders(&candidates, conn, library_id, &[root])?;

    let _ = on_progress.send(DiscoverProgress::Completed {
        found: result.folders.len(),
    });
    Ok(result)
}

pub fn discover_from_paths(
    roots: &[PathBuf],
    conn: &rusqlite::Connection,
    library_id: &str,
    on_progress: &Channel<DiscoverProgress>,
) -> Result<DiscoverResult, AppError> {
    let mut candidates: Vec<(PathBuf, usize)> = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut scanned_dirs = 0usize;

    for root in roots {
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_dir() {
                continue;
            }

            scanned_dirs += 1;
            if scanned_dirs.is_multiple_of(50) {
                let _ = on_progress.send(DiscoverProgress::Scanning { scanned_dirs });
            }

            let dir_path = entry.path();
            let path_str = dir_path.to_string_lossy().to_string();

            if !seen_paths.insert(path_str) {
                continue;
            }

            let image_count = scanner::count_direct_images(dir_path);
            if image_count > 0 {
                candidates.push((dir_path.to_path_buf(), image_count));
            }
        }
    }

    let root_refs: Vec<&Path> = roots.iter().map(|r| r.as_path()).collect();
    let result = partition_leaf_folders(&candidates, conn, library_id, &root_refs)?;

    let _ = on_progress.send(DiscoverProgress::Completed {
        found: result.folders.len(),
    });
    Ok(result)
}

fn find_leaf_indices(candidates: &[(PathBuf, usize)], roots: &[&Path]) -> HashSet<usize> {
    let root_set: HashSet<&Path> = roots.iter().copied().collect();
    let mut parent_of_image_dir = HashSet::new();
    for (path, _) in candidates {
        let mut ancestor = path.parent();
        while let Some(a) = ancestor {
            if !parent_of_image_dir.insert(a.to_path_buf()) {
                break;
            }
            if root_set.contains(a) {
                break;
            }
            ancestor = a.parent();
        }
    }

    candidates
        .iter()
        .enumerate()
        .filter(|(_, (path, _))| !parent_of_image_dir.contains(path))
        .map(|(i, _)| i)
        .collect()
}

fn partition_leaf_folders(
    candidates: &[(PathBuf, usize)],
    conn: &rusqlite::Connection,
    library_id: &str,
    roots: &[&Path],
) -> Result<DiscoverResult, AppError> {
    let leaf_indices = find_leaf_indices(candidates, roots);

    let mut folders = Vec::new();
    let mut skipped_folders = Vec::new();

    for (i, (path, image_count)) in candidates.iter().enumerate() {
        let folder_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if leaf_indices.contains(&i) {
            let path_str = path.to_string_lossy().to_string();
            let already_registered = db::path_exists(conn, library_id, &path_str)?;
            let parsed_metadata = parse_folder_name(&folder_name);

            folders.push(DiscoveredFolder {
                path: path_str,
                folder_name,
                image_count: *image_count,
                parsed_metadata,
                already_registered,
            });
        } else {
            skipped_folders.push(SkippedFolder {
                path: path.to_string_lossy().to_string(),
                folder_name,
                image_count: *image_count,
            });
        }
    }

    Ok(DiscoverResult {
        folders,
        skipped_folders,
    })
}

#[cfg(test)]
#[path = "tests/importer.rs"]
mod tests;
