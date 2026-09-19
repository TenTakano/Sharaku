use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::import_queue::{ImportJob, ImportQueueEvent};
use crate::importer::{self, DiscoverProgress, DiscoverResult, ImportKind, ParsedMetadata};
use crate::template::WorkMetadata;
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
pub(crate) async fn classify_drop_path(path: String) -> Result<scanner::DropKind, String> {
    let p = std::path::Path::new(&path);
    Ok(scanner::classify_path(p))
}

#[tauri::command]
pub(crate) async fn parse_folder_name(folder_name: String) -> Result<ParsedMetadata, String> {
    Ok(importer::parse_folder_name(&folder_name))
}

#[tauri::command]
pub(crate) async fn preview_import_path(
    state: tauri::State<'_, AppState>,
    metadata: WorkMetadata,
    kind: Option<ImportKind>,
) -> Result<String, String> {
    let kind = kind.unwrap_or_default();
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
                kind.work_kind(),
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
    let library_id = state
        .with_active_db(|_db, active| Ok(active.id.clone()))
        .await?;

    let job_id = uuid::Uuid::new_v4().to_string();
    let total = requests.len();
    let job = ImportJob {
        id: job_id.clone(),
        library_id,
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
) -> Result<DiscoverResult, String> {
    let root = PathBuf::from(root_path);
    state
        .with_active_db(move |db, active| {
            importer::discover_image_folders(&root, &db.conn, &active.id, &on_progress)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub(crate) async fn discover_dropped_paths(
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
    on_progress: tauri::ipc::Channel<DiscoverProgress>,
) -> Result<DiscoverResult, String> {
    let roots: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    state
        .with_active_db(move |db, active| {
            importer::discover_from_paths(&roots, &db.conn, &active.id, &on_progress)
                .map_err(|e| e.to_string())
        })
        .await
}
