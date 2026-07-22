use crate::db::{self, WorkDetail, WorkSummary};
use crate::integrity::{self, IntegrityCheckProgress, IntegrityReport};
use crate::AppState;

#[tauri::command]
pub(crate) async fn list_works(
    state: tauri::State<'_, AppState>,
    sort_by: String,
    sort_order: String,
) -> Result<Vec<WorkSummary>, String> {
    state
        .with_active_db(move |db, active| {
            db::list_works(&db.conn, &active.id, &sort_by, &sort_order).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn get_thumbnail(
    state: tauri::State<'_, AppState>,
    work_id: i64,
) -> Result<Vec<u8>, String> {
    state
        .with_guarded_db(move |db| db::get_thumbnail(&db.conn, work_id).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn get_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
) -> Result<WorkDetail, String> {
    state
        .with_guarded_db(move |db| db::get_work(&db.conn, work_id).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn update_work(
    state: tauri::State<'_, AppState>,
    id: i64,
    title: String,
    artist: Option<String>,
    year: Option<i32>,
    genre: Option<String>,
    circle: Option<String>,
    origin: Option<String>,
) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("タイトルは空にできません".to_string());
    }
    state
        .with_guarded_db(move |db| {
            db::update_work(
                &db.conn,
                id,
                title.trim(),
                artist.as_deref(),
                year,
                genre.as_deref(),
                circle.as_deref(),
                origin.as_deref(),
            )
            .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn delete_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    file_action: String,
) -> Result<(), String> {
    state
        .with_active_db(move |db, active| {
            match file_action.as_str() {
                "delete" => {
                    let work = db::get_work(&db.conn, work_id).map_err(|e| e.to_string())?;
                    let path = std::path::Path::new(&work.path);
                    let lib_path = active
                        .path
                        .as_ref()
                        .ok_or("ライブラリルートが設定されていません")?;
                    let canonical_lib =
                        lib_path.canonicalize().unwrap_or_else(|_| lib_path.clone());
                    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                    if !canonical_path.starts_with(&canonical_lib) {
                        return Err(
                            "作品パスがライブラリルート外にあるため削除できません".to_string()
                        );
                    }
                    if path.exists() {
                        if path.is_dir() {
                            std::fs::remove_dir_all(path).map_err(|e| e.to_string())?;
                        } else {
                            std::fs::remove_file(path).map_err(|e| e.to_string())?;
                        }
                    }
                }
                "trash" => {
                    let work = db::get_work(&db.conn, work_id).map_err(|e| e.to_string())?;
                    let src = std::path::Path::new(&work.path);
                    let lib_path = active
                        .path
                        .as_ref()
                        .ok_or("ライブラリルートが設定されていません")?;
                    let canonical_lib =
                        lib_path.canonicalize().unwrap_or_else(|_| lib_path.clone());
                    let canonical_src = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
                    if !canonical_src.starts_with(&canonical_lib) {
                        return Err(
                            "作品パスがライブラリルート外にあるため移動できません".to_string()
                        );
                    }
                    if src.exists() {
                        let trash_dir = lib_path.join(".trash");
                        std::fs::create_dir_all(&trash_dir).map_err(|e| e.to_string())?;

                        let file_name = src.file_name().ok_or("ファイル名の取得に失敗しました")?;
                        let mut dest = trash_dir.join(file_name);
                        if dest.exists() {
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let stem = std::path::Path::new(file_name)
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy();
                            let ext = std::path::Path::new(file_name)
                                .extension()
                                .map(|e| format!(".{}", e.to_string_lossy()))
                                .unwrap_or_default();
                            dest = trash_dir.join(format!("{}_{}{}", stem, timestamp, ext));
                        }
                        std::fs::rename(src, &dest).map_err(|e| e.to_string())?;
                    }
                }
                _ => {} // "none" — metadata only
            }

            db::delete_works_by_ids(&db.conn, &active.id, &[work_id]).map_err(|e| e.to_string())?;
            Ok(())
        })
        .await
}

#[tauri::command]
pub(crate) async fn check_integrity(
    state: tauri::State<'_, AppState>,
    on_progress: tauri::ipc::Channel<IntegrityCheckProgress>,
) -> Result<IntegrityReport, String> {
    state
        .with_active_db(move |db, active| {
            integrity::check_integrity(&db.conn, &active.id, active.path.as_deref(), &on_progress)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn delete_orphan_works(
    state: tauri::State<'_, AppState>,
    ids: Vec<i64>,
) -> Result<usize, String> {
    state
        .with_active_db(move |db, active| {
            integrity::delete_orphan_works(&db.conn, &active.id, &ids).map_err(|e| e.to_string())
        })
        .await
}
