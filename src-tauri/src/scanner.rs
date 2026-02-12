use std::path::Path;

use serde::Serialize;
use tauri::ipc::Channel;
use walkdir::WalkDir;

use crate::db::{self, WorkRecord};
use crate::error::AppError;
use crate::thumbnail;

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

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

fn is_image_file(path: &Path) -> bool {
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
            },
        )?;

        registered += 1;
    }

    let _ = on_progress.send(ScanProgress::Completed { registered, failed });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn recognizes_supported_extensions() {
        for ext in &["jpg", "jpeg", "png", "gif", "webp", "bmp"] {
            let name = format!("photo.{}", ext);
            assert!(is_image_file(Path::new(&name)), "should accept .{}", ext);
        }
    }

    #[test]
    fn recognizes_uppercase_and_mixed_case() {
        assert!(is_image_file(Path::new("photo.JPG")));
        assert!(is_image_file(Path::new("photo.Png")));
        assert!(is_image_file(Path::new("photo.GIF")));
        assert!(is_image_file(Path::new("photo.WeBp")));
    }

    #[test]
    fn rejects_non_image_extensions() {
        for ext in &["pdf", "zip", "txt", "mp4", "doc", "rs"] {
            let name = format!("file.{}", ext);
            assert!(!is_image_file(Path::new(&name)), "should reject .{}", ext);
        }
    }

    #[test]
    fn rejects_no_extension() {
        assert!(!is_image_file(Path::new("README")));
        assert!(!is_image_file(Path::new("Makefile")));
    }

    #[test]
    fn rejects_hidden_files_without_image_ext() {
        assert!(!is_image_file(Path::new(".gitignore")));
        assert!(!is_image_file(Path::new(".hidden")));
    }

    #[test]
    fn accepts_hidden_files_with_image_ext() {
        assert!(is_image_file(Path::new(".photo.jpg")));
    }
}
