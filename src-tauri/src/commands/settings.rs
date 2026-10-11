use serde::Serialize;

use crate::settings;
use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    delete_file_action: String,
}

#[tauri::command]
pub(crate) async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    state
        .with_active_db(move |db, active| {
            let delete_file_action = settings::get_delete_file_action(&db.conn, &active.id)
                .map_err(|e| e.to_string())?;
            Ok(AppSettings { delete_file_action })
        })
        .await
}

#[tauri::command]
pub(crate) async fn set_delete_file_action(
    state: tauri::State<'_, AppState>,
    action: String,
) -> Result<(), String> {
    if action != "delete" && action != "trash" && action != "ask" {
        return Err("無効な削除時のファイル処理設定です".to_string());
    }
    state
        .with_active_db(move |db, active| {
            settings::set_delete_file_action(&db.conn, &active.id, &action)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn get_theme(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state
        .with_db(|db| settings::get_theme_mode(&db.conn).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn set_theme(
    state: tauri::State<'_, AppState>,
    mode: String,
) -> Result<(), String> {
    state
        .with_db(move |db| settings::set_theme_mode(&db.conn, &mode).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn get_banner_auto_close(
    state: tauri::State<'_, AppState>,
) -> Result<u32, String> {
    state
        .with_db(|db| settings::get_banner_auto_close(&db.conn).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn set_banner_auto_close(
    state: tauri::State<'_, AppState>,
    seconds: u32,
) -> Result<(), String> {
    state
        .with_db(move |db| {
            settings::set_banner_auto_close(&db.conn, seconds).map_err(|e| e.to_string())
        })
        .await
}
