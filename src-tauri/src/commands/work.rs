use std::path::PathBuf;

use crate::db::{self, WorkDetail, WorkSummary};
use crate::AppState;

/// Fetches the registered path for work_id and checks that it still exists on disk.
/// If it is missing, returns missing_message as-is as the error
/// (parameterized so each delete/trash call site can supply its own error wording).
fn validated_work_path(
    conn: &rusqlite::Connection,
    work_id: i64,
    missing_message: &str,
) -> Result<PathBuf, String> {
    let work = db::get_work(conn, work_id).map_err(|e| e.to_string())?;
    let path = PathBuf::from(&work.path);
    if !path.exists() {
        return Err(missing_message.to_string());
    }
    Ok(path)
}

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
    let trimmed_title = title.trim().to_string();
    if trimmed_title.is_empty() {
        return Err("タイトルは空にできません".to_string());
    }
    state
        .with_guarded_db(move |db| {
            db::update_work(
                &db.conn,
                id,
                &trimmed_title,
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
                    let path = validated_work_path(
                        &db.conn,
                        work_id,
                        "作品ファイルが見つからないため削除できません",
                    )?;
                    if path.is_dir() {
                        std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
                    } else {
                        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
                    }
                }
                "trash" => {
                    let lib_path = active
                        .path
                        .as_ref()
                        .ok_or("ライブラリルートが設定されていません")?;
                    let src = validated_work_path(
                        &db.conn,
                        work_id,
                        "作品ファイルが見つからないため移動できません",
                    )?;
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
                    std::fs::rename(&src, &dest).map_err(|e| e.to_string())?;
                }
                _ => {} // "none" — metadata only
            }

            db::delete_works_by_ids(&db.conn, &active.id, &[work_id]).map_err(|e| e.to_string())?;
            Ok(())
        })
        .await
}

#[cfg(test)]
#[path = "tests/work.rs"]
mod tests;
