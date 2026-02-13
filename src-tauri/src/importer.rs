use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::db::{self, WorkRecord};
use crate::error::AppError;
use crate::scanner;
use crate::settings;
use crate::template::{self, WorkMetadata};
use crate::thumbnail;

#[derive(Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
    images.sort();
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

pub fn import_work(request: &ImportRequest, app_data_dir: &Path) -> Result<ImportResult, AppError> {
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

    let conn = db::open_db(app_data_dir)?;

    let library_root = settings::get_library_root(&conn)?
        .ok_or_else(|| AppError::ImportError("ライブラリルートが設定されていません".to_string()))?;
    let template_str = settings::get_directory_template(&conn)?.ok_or_else(|| {
        AppError::ImportError("ディレクトリテンプレートが設定されていません".to_string())
    })?;

    let metadata = WorkMetadata {
        title: request.title.clone(),
        artist: request.artist.clone(),
        year: request.year,
        genre: request.genre.clone(),
        circle: request.circle.clone(),
        origin: request.origin.clone(),
    };

    let dest =
        template::resolve_unique_work_path(Path::new(&library_root), &template_str, &metadata);

    if paths_overlap(source, &dest) {
        return Err(AppError::ImportError(
            "取り込み元と取り込み先が重複しています".to_string(),
        ));
    }

    let thumb = thumbnail::generate_thumbnail(&images[0])?;

    std::fs::create_dir_all(&dest)?;

    let rollback = |dest: &Path| {
        let _ = std::fs::remove_dir_all(dest);
    };

    // Always copy first (even in Move mode) to avoid data loss on failure
    if let Err(e) = copy_images_to_dest(&images, &dest) {
        rollback(&dest);
        return Err(e);
    }

    let dest_str = dest.to_string_lossy().to_string();
    let page_count = images.len();

    if let Err(e) = db::insert_work(
        &conn,
        &WorkRecord {
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

    // Delete source files only after successful DB registration
    if request.mode == ImportMode::Move {
        for image in &images {
            let _ = std::fs::remove_file(image);
        }
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

#[cfg(test)]
#[path = "tests/importer.rs"]
mod tests;
