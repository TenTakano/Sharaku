use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::import_queue::{ImportJob, ImportQueueEvent};
use crate::importer::{self, DiscoverProgress, DiscoveredFolder, ParsedMetadata};
use crate::relocator::{self, RelocationPreview, RelocationProgress};
use crate::template::{self, WorkMetadata};
use crate::{scanner, settings, AppState};

#[tauri::command]
pub(crate) async fn has_image_subfolders(dir: String) -> Result<bool, String> {
    let path = std::path::Path::new(&dir);
    if !path.is_dir() {
        return Err("有効なディレクトリではありません".to_string());
    }
    Ok(scanner::has_image_subfolders(path))
}

#[tauri::command]
pub(crate) async fn resolve_drop_path(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    let folder = if p.is_dir() {
        p.to_path_buf()
    } else {
        p.parent()
            .ok_or_else(|| "親ディレクトリを取得できません".to_string())?
            .to_path_buf()
    };
    if !folder.is_dir() {
        return Err("有効なディレクトリではありません".to_string());
    }
    folder
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "パスの変換に失敗しました".to_string())
}

#[tauri::command]
pub(crate) async fn parse_folder_name(folder_name: String) -> Result<ParsedMetadata, String> {
    Ok(importer::parse_folder_name(&folder_name))
}

#[tauri::command]
pub(crate) async fn preview_import_path(
    state: tauri::State<'_, AppState>,
    metadata: WorkMetadata,
) -> Result<String, String> {
    state
        .with_active_db(move |db, active| {
            let lib_path = active
                .path
                .as_ref()
                .ok_or("ライブラリルートが設定されていません")?;
            let template_str = settings::get_directory_template(&db.conn, &active.id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "ディレクトリテンプレートが設定されていません".to_string())?;
            Ok(importer::preview_import_path(
                lib_path,
                &template_str,
                &metadata,
            ))
        })
        .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnqueueResult {
    job_id: String,
}

#[tauri::command]
pub(crate) async fn enqueue_import(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    requests: Vec<importer::ImportRequest>,
) -> Result<EnqueueResult, String> {
    let (library_id, library_root, resource_mode) = state
        .with_active_db(|db, active| {
            let mode =
                settings::get_resource_mode(&db.conn, &active.id).map_err(|e| e.to_string())?;
            Ok((active.id.clone(), active.path.clone(), mode))
        })
        .await?;

    if resource_mode == "full" && library_root.is_none() {
        return Err("ライブラリルートが設定されていません".to_string());
    }

    let job_id = uuid::Uuid::new_v4().to_string();
    let total = requests.len();
    let job = ImportJob {
        id: job_id.clone(),
        library_id,
        library_root,
        requests,
    };

    let _ = app.emit(
        "import-queue",
        ImportQueueEvent::Enqueued {
            job_id: job_id.clone(),
            total,
        },
    );

    state.import_queue.enqueue(job)?;

    Ok(EnqueueResult { job_id })
}

#[tauri::command]
pub(crate) async fn discover_folders(
    state: tauri::State<'_, AppState>,
    root_path: String,
    on_progress: tauri::ipc::Channel<DiscoverProgress>,
) -> Result<Vec<DiscoveredFolder>, String> {
    let root = PathBuf::from(root_path);
    state
        .with_active_db(move |db, active| {
            importer::discover_image_folders(&root, &db.conn, &active.id, &on_progress)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn preview_relocation(
    state: tauri::State<'_, AppState>,
    new_template: String,
) -> Result<Vec<RelocationPreview>, String> {
    let trimmed = new_template.trim().to_string();
    template::validate_template(&trimmed).map_err(|e| e.to_string())?;
    state
        .with_active_db(move |db, active| {
            let lib_path = active
                .path
                .as_ref()
                .ok_or("ライブラリルートが設定されていません")?;
            relocator::preview_relocation(&db.conn, &active.id, lib_path, &trimmed)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn relocate_works(
    state: tauri::State<'_, AppState>,
    new_template: String,
    on_progress: tauri::ipc::Channel<RelocationProgress>,
) -> Result<(), String> {
    let trimmed = new_template.trim().to_string();
    template::validate_template(&trimmed).map_err(|e| e.to_string())?;
    state
        .with_active_db(move |db, active| {
            let lib_path = active
                .path
                .as_ref()
                .ok_or("ライブラリルートが設定されていません")?;
            relocator::execute_relocation(&db.conn, &active.id, lib_path, &trimmed, &on_progress)
                .map_err(|e| e.to_string())
        })
        .await
}
