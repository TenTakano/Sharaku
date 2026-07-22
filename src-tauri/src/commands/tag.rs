use crate::db::{self, Tag, WorkSummary};
use crate::AppState;

#[tauri::command]
pub(crate) async fn list_tags(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, String> {
    state
        .with_active_db(|db, active| db::list_tags(&db.conn, &active.id).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn search_tags(
    state: tauri::State<'_, AppState>,
    query: String,
    category: Option<String>,
) -> Result<Vec<Tag>, String> {
    state
        .with_active_db(move |db, active| {
            db::search_tags(&db.conn, &active.id, &query, category.as_deref())
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn create_tag(
    state: tauri::State<'_, AppState>,
    name: String,
    category: Option<String>,
) -> Result<Tag, String> {
    state
        .with_active_db(move |db, active| {
            db::create_tag(&db.conn, &active.id, &name, category.as_deref())
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn update_tag(
    state: tauri::State<'_, AppState>,
    id: i64,
    name: String,
    category: Option<String>,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            db::update_tag(&db.conn, id, &name, category.as_deref()).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn delete_tag(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    state
        .with_guarded_db(move |db| db::delete_tag(&db.conn, id).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn add_tag_to_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            db::add_tag_to_work(&db.conn, work_id, tag_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn remove_tag_from_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            db::remove_tag_from_work(&db.conn, work_id, tag_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn get_tags_for_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
) -> Result<Vec<Tag>, String> {
    state
        .with_guarded_db(move |db| {
            db::get_tags_for_work(&db.conn, work_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn search_works_by_tags(
    state: tauri::State<'_, AppState>,
    tag_ids: Vec<i64>,
    mode: String,
) -> Result<Vec<WorkSummary>, String> {
    state
        .with_active_db(move |db, active| {
            db::search_works_by_tags(&db.conn, &active.id, &tag_ids, &mode)
                .map_err(|e| e.to_string())
        })
        .await
}
