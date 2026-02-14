use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::ipc::Channel;

use crate::db::{self, WorkDetail};
use crate::error::AppError;
use crate::importer;
use crate::settings;
use crate::template::{self, WorkMetadata};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RelocationProgress {
    #[serde(rename_all = "camelCase")]
    Started { total: usize },
    #[serde(rename_all = "camelCase")]
    Moving {
        current: usize,
        total: usize,
        title: String,
    },
    #[serde(rename_all = "camelCase")]
    Completed {
        relocated: usize,
        skipped: usize,
        failed: usize,
    },
    #[serde(rename_all = "camelCase")]
    Error { message: String },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelocationPreview {
    pub work_id: i64,
    pub title: String,
    pub old_path: String,
    pub new_path: String,
}

fn work_detail_to_metadata(work: &WorkDetail, type_label: &str) -> WorkMetadata {
    WorkMetadata {
        title: work.title.clone(),
        artist: work.artist.clone(),
        year: work.year,
        genre: work.genre.clone(),
        circle: work.circle.clone(),
        origin: work.origin.clone(),
        work_type: Some(type_label.to_string()),
    }
}

fn compute_relocation_plan(
    works: &[WorkDetail],
    library_root: &Path,
    new_template: &str,
    type_label: &str,
) -> Vec<RelocationPreview> {
    let mut used_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut previews = Vec::new();

    for work in works {
        let metadata = work_detail_to_metadata(work, type_label);
        let base_path = template::resolve_work_path(library_root, new_template, &metadata);
        let base_str = base_path.to_string_lossy().to_string();

        let new_path =
            if used_paths.contains(&base_str) || (base_path.exists() && base_str != work.path) {
                make_unique_path(&base_path, &used_paths)
            } else {
                base_path
            };

        let new_path_str = new_path.to_string_lossy().to_string();
        if new_path_str != work.path {
            used_paths.insert(new_path_str.clone());
            previews.push(RelocationPreview {
                work_id: work.id,
                title: work.title.clone(),
                old_path: work.path.clone(),
                new_path: new_path_str,
            });
        }
    }

    previews
}

fn make_unique_path(base: &Path, used_paths: &std::collections::HashSet<String>) -> PathBuf {
    let base_name = base.file_name().unwrap().to_string_lossy().to_string();
    for i in 1u32.. {
        let dir_name = format!("{}_{:04x}", base_name, i);
        let candidate = base.with_file_name(&dir_name);
        let candidate_str = candidate.to_string_lossy().to_string();
        if !used_paths.contains(&candidate_str) && !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
}

pub fn preview_relocation(
    conn: &rusqlite::Connection,
    library_root: &Path,
    new_template: &str,
) -> Result<Vec<RelocationPreview>, AppError> {
    let works = db::list_folder_works(conn)?;
    let type_label = settings::get_type_label_folder(conn)?;
    Ok(compute_relocation_plan(
        &works,
        library_root,
        new_template,
        &type_label,
    ))
}

pub fn execute_relocation(
    app_data_dir: &Path,
    new_template: &str,
    on_progress: &Channel<RelocationProgress>,
) -> Result<(), AppError> {
    let conn = db::open_db(app_data_dir)?;
    let library_root = settings::get_library_root(&conn)?
        .ok_or_else(|| AppError::RelocationError("ライブラリルートが設定されていません".into()))?;
    let library_root = PathBuf::from(&library_root);

    let works = db::list_folder_works(&conn)?;
    let type_label = settings::get_type_label_folder(&conn)?;
    let plan = compute_relocation_plan(&works, &library_root, new_template, &type_label);

    let total = plan.len();
    let _ = on_progress.send(RelocationProgress::Started { total });

    let mut relocated = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for (i, item) in plan.iter().enumerate() {
        let _ = on_progress.send(RelocationProgress::Moving {
            current: i + 1,
            total,
            title: item.title.clone(),
        });

        let old_path = Path::new(&item.old_path);
        let new_path = Path::new(&item.new_path);

        if !old_path.exists() {
            skipped += 1;
            continue;
        }

        match copy_work_files(old_path, new_path) {
            Ok(()) => {
                if let Err(e) = db::update_work_path(&conn, item.work_id, &item.new_path) {
                    let _ = on_progress.send(RelocationProgress::Error {
                        message: format!("DB更新失敗 ({}): {}", item.title, e),
                    });
                    let _ = std::fs::remove_dir_all(new_path);
                    failed += 1;
                    continue;
                }
                remove_work_files(old_path);
                cleanup_empty_ancestors(old_path, &library_root);
                relocated += 1;
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(new_path);
                let _ = on_progress.send(RelocationProgress::Error {
                    message: format!("移動失敗 ({}): {}", item.title, e),
                });
                failed += 1;
            }
        }
    }

    settings::set_directory_template(&conn, new_template)?;

    let _ = on_progress.send(RelocationProgress::Completed {
        relocated,
        skipped,
        failed,
    });

    Ok(())
}

fn copy_work_files(old_path: &Path, new_path: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(new_path)?;

    let images = importer::list_images_in_folder(old_path)?;
    for image in &images {
        let file_name = image
            .file_name()
            .ok_or_else(|| AppError::RelocationError("無効なファイル名".into()))?;
        let dest = new_path.join(file_name);
        std::fs::copy(image, &dest)?;
    }

    Ok(())
}

fn remove_work_files(path: &Path) {
    if let Ok(images) = importer::list_images_in_folder(path) {
        for image in &images {
            let _ = std::fs::remove_file(image);
        }
    }
    let _ = std::fs::remove_dir(path);
}

fn cleanup_empty_ancestors(path: &Path, stop_at: &Path) {
    let mut current = path.to_path_buf();
    while let Some(parent) = current.parent() {
        if parent == stop_at || !parent.starts_with(stop_at) {
            break;
        }
        if std::fs::read_dir(parent).map_or(true, |mut d| d.next().is_some()) {
            break;
        }
        let _ = std::fs::remove_dir(parent);
        current = parent.to_path_buf();
    }
}

#[cfg(test)]
#[path = "tests/relocator.rs"]
mod tests;
