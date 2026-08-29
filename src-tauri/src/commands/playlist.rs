use crate::db::{self, Playlist, PlaylistItem};
use crate::AppState;

fn validate_playlist_name(name: &str) -> Result<String, String> {
    let trimmed_name = name.trim().to_string();
    if trimmed_name.is_empty() {
        return Err("プレイリスト名は空にできません".to_string());
    }
    Ok(trimmed_name)
}

#[tauri::command]
pub(crate) async fn list_playlists(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Playlist>, String> {
    state
        .with_active_db(|db, active| {
            db::list_playlists(&db.conn, &active.id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn create_playlist(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<Playlist, String> {
    let trimmed_name = validate_playlist_name(&name)?;
    state
        .with_active_db(move |db, active| {
            db::create_playlist(&db.conn, &active.id, &trimmed_name).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn rename_playlist(
    state: tauri::State<'_, AppState>,
    id: i64,
    name: String,
) -> Result<(), String> {
    let trimmed_name = validate_playlist_name(&name)?;
    state
        .with_guarded_db(move |db| {
            db::rename_playlist(&db.conn, id, &trimmed_name).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn delete_playlist(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| db::delete_playlist(&db.conn, id).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn get_playlist_items(
    state: tauri::State<'_, AppState>,
    playlist_id: i64,
) -> Result<Vec<PlaylistItem>, String> {
    state
        .with_guarded_db(move |db| {
            db::get_playlist_items(&db.conn, playlist_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn add_item_to_playlist(
    state: tauri::State<'_, AppState>,
    playlist_id: i64,
    work_id: i64,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            db::add_item_to_playlist(&db.conn, playlist_id, work_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn remove_item_from_playlist(
    state: tauri::State<'_, AppState>,
    playlist_id: i64,
    work_id: i64,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            db::remove_item_from_playlist(&db.conn, playlist_id, work_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn reorder_playlist_items(
    state: tauri::State<'_, AppState>,
    playlist_id: i64,
    work_ids: Vec<i64>,
) -> Result<(), String> {
    state
        .with_guarded_db_mut(move |db| {
            db::reorder_playlist_items(&mut db.conn, playlist_id, &work_ids)
                .map_err(|e| e.to_string())
        })
        .await
}
