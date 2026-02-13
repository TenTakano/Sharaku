use std::path::Path;

use serde::Serialize;
use tauri::ipc::Channel;
use walkdir::WalkDir;

use crate::db::{self, WorkRecord};
use crate::error::AppError;
use crate::thumbnail;

pub(crate) const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ScanProgress {
    Started {
        total: usize,
    },
    Processing {
        current: usize,
        total: usize,
        file_name: String,
    },
    Completed {
        registered: usize,
        failed: usize,
    },
    Error {
        message: String,
    },
}

pub(crate) fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn scan_directory(
    root: &Path,
    app_data_dir: &Path,
    on_progress: &Channel<ScanProgress>,
) -> Result<(), AppError> {
    let mut walk_errors = 0usize;
    let image_files: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(_) => {
                walk_errors += 1;
                None
            }
        })
        .filter(|e| e.file_type().is_file() && is_image_file(e.path()))
        .collect();

    let total = image_files.len();
    let _ = on_progress.send(ScanProgress::Started { total });

    let conn = db::open_db(app_data_dir)?;
    let mut registered = 0usize;
    let mut failed = walk_errors;

    for (i, entry) in image_files.iter().enumerate() {
        let path = entry.path();
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let _ = on_progress.send(ScanProgress::Processing {
            current: i + 1,
            total,
            file_name,
        });

        let path_str = path.to_string_lossy();
        if db::path_exists(&conn, &path_str)? {
            continue;
        }

        let thumb = match thumbnail::generate_thumbnail(path) {
            Ok(data) => data,
            Err(_) => {
                failed += 1;
                continue;
            }
        };

        let title = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        db::insert_work(
            &conn,
            &WorkRecord {
                title: &title,
                path: &path_str,
                work_type: "image",
                page_count: 1,
                thumbnail: &thumb,
                artist: None,
                year: None,
                genre: None,
                circle: None,
                origin: None,
            },
        )?;

        registered += 1;
    }

    let _ = on_progress.send(ScanProgress::Completed { registered, failed });
    Ok(())
}

#[cfg(test)]
#[path = "tests/scanner.rs"]
mod tests;
