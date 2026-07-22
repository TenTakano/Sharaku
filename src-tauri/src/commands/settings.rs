use serde::Serialize;

use crate::settings;
use crate::template;
use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    resource_mode: String,
    directory_template: Option<String>,
    type_label_image: String,
    type_label_folder: String,
    delete_file_action: String,
}

#[tauri::command]
pub(crate) async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    state
        .with_active_db(move |db, active| {
            let resource_mode =
                settings::get_resource_mode(&db.conn, &active.id).map_err(|e| e.to_string())?;
            let directory_template = settings::get_directory_template(&db.conn, &active.id)
                .map_err(|e| e.to_string())?;
            let type_label_image =
                settings::get_type_label_image(&db.conn, &active.id).map_err(|e| e.to_string())?;
            let type_label_folder =
                settings::get_type_label_folder(&db.conn, &active.id).map_err(|e| e.to_string())?;
            let delete_file_action = settings::get_delete_file_action(&db.conn, &active.id)
                .map_err(|e| e.to_string())?;
            Ok(AppSettings {
                resource_mode,
                directory_template,
                type_label_image,
                type_label_folder,
                delete_file_action,
            })
        })
        .await
}

#[tauri::command]
pub(crate) async fn set_resource_mode(
    state: tauri::State<'_, AppState>,
    mode: String,
) -> Result<(), String> {
    if mode != "full" && mode != "metadata_only" {
        return Err("無効なリソース管理モードです".to_string());
    }
    state
        .with_active_db(move |db, active| {
            settings::set_resource_mode(&db.conn, &active.id, &mode).map_err(|e| e.to_string())
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
pub(crate) async fn set_directory_template(
    state: tauri::State<'_, AppState>,
    template: String,
) -> Result<(), String> {
    if !template.trim().is_empty() {
        template::validate_template(template.trim()).map_err(|e| e.to_string())?;
    }
    state
        .with_active_db(move |db, active| {
            settings::set_directory_template(&db.conn, &active.id, template.trim())
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn validate_template(template: String) -> Result<(), String> {
    template::validate_template(&template).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn set_type_labels(
    state: tauri::State<'_, AppState>,
    image_label: String,
    folder_label: String,
) -> Result<(), String> {
    if image_label.trim().is_empty() || folder_label.trim().is_empty() {
        return Err("ラベルは空にできません".to_string());
    }
    state
        .with_active_db(move |db, active| {
            settings::set_type_label_image(&db.conn, &active.id, image_label.trim())
                .map_err(|e| e.to_string())?;
            settings::set_type_label_folder(&db.conn, &active.id, folder_label.trim())
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn preview_template(
    state: tauri::State<'_, AppState>,
    template: String,
) -> Result<String, String> {
    template::validate_template(&template).map_err(|e| e.to_string())?;
    state
        .with_active_db(move |db, active| {
            let folder_label =
                settings::get_type_label_folder(&db.conn, &active.id).map_err(|e| e.to_string())?;
            let mut metadata = template::sample_metadata();
            metadata.work_type = Some(folder_label);
            Ok(template::render_template(&template, &metadata))
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
