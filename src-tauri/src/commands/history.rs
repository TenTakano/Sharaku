use crate::history::{self, WorkWithHistory};
use crate::AppState;

#[tauri::command]
pub(crate) async fn start_view_session(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    page: u32,
) -> Result<i64, String> {
    state
        .with_active_db(move |db, active| {
            history::start_view_session(&db.conn, &active.id, work_id, page.into())
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn update_view_session_page(
    state: tauri::State<'_, AppState>,
    session_id: i64,
    page: u32,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            history::update_view_session_page(&db.conn, session_id, page.into())
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn end_view_session(
    state: tauri::State<'_, AppState>,
    session_id: i64,
    page: u32,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            history::end_view_session(&db.conn, session_id, page.into()).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn list_recently_viewed_works(
    state: tauri::State<'_, AppState>,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, String> {
    state
        .with_active_db(move |db, active| {
            history::list_recently_viewed_works(&db.conn, &active.id, limit)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn list_unviewed_works(
    state: tauri::State<'_, AppState>,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, String> {
    state
        .with_active_db(move |db, active| {
            history::list_unviewed_works(&db.conn, &active.id, limit).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn list_frequently_viewed_works(
    state: tauri::State<'_, AppState>,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, String> {
    state
        .with_active_db(move |db, active| {
            history::list_frequently_viewed_works(&db.conn, &active.id, limit)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn list_recently_added_works(
    state: tauri::State<'_, AppState>,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, String> {
    state
        .with_active_db(move |db, active| {
            history::list_recently_added_works(&db.conn, &active.id, limit)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn list_pickup_works(
    state: tauri::State<'_, AppState>,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, String> {
    state
        .with_active_db(move |db, active| {
            history::list_pickup_works(&db.conn, &active.id, limit).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn list_random_works(
    state: tauri::State<'_, AppState>,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, String> {
    state
        .with_active_db(move |db, active| {
            history::list_random_works(&db.conn, &active.id, limit).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn get_work_history(
    state: tauri::State<'_, AppState>,
    work_id: i64,
) -> Result<WorkWithHistory, String> {
    state
        .with_guarded_db(move |db| {
            history::get_work_history(&db.conn, work_id).map_err(|e| e.to_string())
        })
        .await
}
