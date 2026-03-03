use std::collections::HashSet;
use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;
use tauri::ipc::Channel;
use walkdir::WalkDir;

use crate::db;
use crate::error::AppError;
use crate::scanner;
use crate::settings;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrphanWork {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub work_type: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnregisteredEntry {
    pub path: String,
    pub folder_name: String,
    pub image_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityReport {
    pub total_works: usize,
    pub orphan_works: Vec<OrphanWork>,
    pub unregistered_entries: Vec<UnregisteredEntry>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum IntegrityCheckProgress {
    #[serde(rename_all = "camelCase")]
    CheckingWorks {
        checked: usize,
        total: usize,
    },
    #[serde(rename_all = "camelCase")]
    ScanningDirectory {
        scanned_dirs: usize,
    },
    Completed,
}

pub fn check_integrity(
    conn: &Connection,
    library_id: &str,
    library_root: Option<&Path>,
    on_progress: &Channel<IntegrityCheckProgress>,
) -> Result<IntegrityReport, AppError> {
    check_integrity_core(conn, library_id, library_root, |p| {
        let _ = on_progress.send(p);
    })
}

fn check_integrity_core(
    conn: &Connection,
    library_id: &str,
    library_root: Option<&Path>,
    on_progress: impl Fn(IntegrityCheckProgress),
) -> Result<IntegrityReport, AppError> {
    let work_entries = db::list_work_paths(conn, library_id)?;
    let total_works = work_entries.len();

    // Phase 1: orphan detection
    let mut orphan_works = Vec::new();
    let mut registered_paths = HashSet::new();
    let mut registered_dirs = HashSet::new();
    for (i, entry) in work_entries.iter().enumerate() {
        if (i + 1).is_multiple_of(50) || i + 1 == total_works {
            on_progress(IntegrityCheckProgress::CheckingWorks {
                checked: i + 1,
                total: total_works,
            });
        }

        registered_paths.insert(entry.path.clone());
        let path = Path::new(&entry.path);
        if entry.work_type == "folder" {
            registered_dirs.insert(entry.path.clone());
        } else if let Some(parent) = path.parent() {
            registered_dirs.insert(parent.to_string_lossy().to_string());
        }

        let path = Path::new(&entry.path);
        let exists = match entry.work_type.as_str() {
            "folder" => path.is_dir(),
            _ => path.is_file(),
        };
        if !exists {
            orphan_works.push(OrphanWork {
                id: entry.id,
                title: entry.title.clone(),
                path: entry.path.clone(),
                work_type: entry.work_type.clone(),
            });
        }
    }

    // Phase 2: unregistered entry detection (skipped in metadata_only mode or when no library root)
    let resource_mode = settings::get_resource_mode(conn, library_id)?;
    let mut unregistered_entries = Vec::new();
    if let (Some(root), true) = (library_root, resource_mode != "metadata_only") {
        let mut scanned_dirs = 0usize;
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_dir() {
                continue;
            }

            scanned_dirs += 1;
            if scanned_dirs.is_multiple_of(50) {
                on_progress(IntegrityCheckProgress::ScanningDirectory { scanned_dirs });
            }

            let dir_path = entry.path();
            let path_str = dir_path.to_string_lossy().to_string();
            if registered_dirs.contains(&path_str) {
                continue;
            }

            let image_count = scanner::count_direct_images(dir_path);
            if image_count == 0 {
                continue;
            }

            let folder_name = dir_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            unregistered_entries.push(UnregisteredEntry {
                path: path_str,
                folder_name,
                image_count,
            });
        }
    }

    on_progress(IntegrityCheckProgress::Completed);

    Ok(IntegrityReport {
        total_works,
        orphan_works,
        unregistered_entries,
    })
}

pub fn delete_orphan_works(
    conn: &Connection,
    library_id: &str,
    ids: &[i64],
) -> Result<usize, AppError> {
    db::delete_works_by_ids(conn, library_id, ids)
}

#[cfg(test)]
#[path = "tests/integrity.rs"]
mod tests;
