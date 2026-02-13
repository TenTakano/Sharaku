mod db;
mod error;
mod importer;
mod relocator;
mod scanner;
mod settings;
mod template;
mod thumbnail;
mod viewer;

use std::path::PathBuf;

use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};

use db::{WorkDetail, WorkSummary};
use importer::{BulkImportProgress, BulkImportSummary, DiscoverProgress, DiscoveredFolder,
    ImportResult, ParsedMetadata};
use relocator::{RelocationPreview, RelocationProgress};
use serde::Serialize;
use template::WorkMetadata;

#[tauri::command]
async fn list_works(
    app: tauri::AppHandle,
    sort_by: String,
    sort_order: String,
) -> Result<Vec<WorkSummary>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&app_data_dir).map_err(|e| e.to_string())?;
        db::list_works(&conn, &sort_by, &sort_order).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_thumbnail(app: tauri::AppHandle, work_id: i64) -> Result<Vec<u8>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&app_data_dir).map_err(|e| e.to_string())?;
        db::get_thumbnail(&conn, work_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_work(app: tauri::AppHandle, work_id: i64) -> Result<WorkDetail, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&app_data_dir).map_err(|e| e.to_string())?;
        db::get_work(&conn, work_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    library_root: Option<String>,
    directory_template: Option<String>,
}

#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&app_data_dir).map_err(|e| e.to_string())?;
        let library_root = settings::get_library_root(&conn).map_err(|e| e.to_string())?;
        let directory_template =
            settings::get_directory_template(&conn).map_err(|e| e.to_string())?;
        Ok(AppSettings {
            library_root,
            directory_template,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_library_root(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&app_data_dir).map_err(|e| e.to_string())?;
        settings::set_library_root(&conn, &path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_directory_template(app: tauri::AppHandle, template: String) -> Result<(), String> {
    let trimmed = template.trim().to_string();
    if !trimmed.is_empty() {
        template::validate_template(&trimmed).map_err(|e| e.to_string())?;
    }
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&app_data_dir).map_err(|e| e.to_string())?;
        settings::set_directory_template(&conn, &trimmed).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn validate_template(template: String) -> Result<(), String> {
    template::validate_template(&template).map_err(|e| e.to_string())
}

#[tauri::command]
async fn preview_template(template: String) -> Result<String, String> {
    template::validate_template(&template).map_err(|e| e.to_string())?;
    let metadata = template::sample_metadata();
    Ok(template::render_template(&template, &metadata))
}

#[tauri::command]
async fn parse_folder_name(folder_name: String) -> Result<ParsedMetadata, String> {
    Ok(importer::parse_folder_name(&folder_name))
}

#[tauri::command]
async fn preview_import_path(
    app: tauri::AppHandle,
    metadata: WorkMetadata,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&app_data_dir).map_err(|e| e.to_string())?;
        let library_root = settings::get_library_root(&conn)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "ライブラリルートが設定されていません".to_string())?;
        let template_str = settings::get_directory_template(&conn)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "ディレクトリテンプレートが設定されていません".to_string())?;
        Ok(importer::preview_import_path(
            std::path::Path::new(&library_root),
            &template_str,
            &metadata,
        ))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_work(
    app: tauri::AppHandle,
    request: importer::ImportRequest,
) -> Result<ImportResult, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        importer::import_work(&request, &app_data_dir).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn discover_folders(
    app: tauri::AppHandle,
    root_path: String,
    on_progress: tauri::ipc::Channel<DiscoverProgress>,
) -> Result<Vec<DiscoveredFolder>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let root = PathBuf::from(root_path);
    tokio::task::spawn_blocking(move || {
        importer::discover_image_folders(&root, &app_data_dir, &on_progress)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn bulk_import(
    app: tauri::AppHandle,
    requests: Vec<importer::ImportRequest>,
    on_progress: tauri::ipc::Channel<BulkImportProgress>,
) -> Result<BulkImportSummary, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        importer::bulk_import(&requests, &app_data_dir, &on_progress).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn preview_relocation(
    app: tauri::AppHandle,
    new_template: String,
) -> Result<Vec<RelocationPreview>, String> {
    let trimmed = new_template.trim().to_string();
    template::validate_template(&trimmed).map_err(|e| e.to_string())?;
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&app_data_dir).map_err(|e| e.to_string())?;
        let library_root = settings::get_library_root(&conn)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "ライブラリルートが設定されていません".to_string())?;
        relocator::preview_relocation(&conn, std::path::Path::new(&library_root), &trimmed)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn relocate_works(
    app: tauri::AppHandle,
    new_template: String,
    on_progress: tauri::ipc::Channel<RelocationProgress>,
) -> Result<(), String> {
    let trimmed = new_template.trim().to_string();
    template::validate_template(&trimmed).map_err(|e| e.to_string())?;
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        relocator::execute_relocation(&app_data_dir, &trimmed, &on_progress)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/001_create_initial_tables.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add_metadata_and_settings",
            sql: include_str!("../migrations/002_add_metadata_and_settings.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "allow_folder_work_type",
            sql: include_str!("../migrations/003_allow_folder_work_type.sql"),
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:sharaku.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("sharaku", |ctx, request| {
            let uri = request.uri().to_string();
            match viewer::parse_view_uri(&uri) {
                Some((work_id, page_index)) => match ctx.app_handle().path().app_data_dir() {
                    Ok(app_data_dir) => {
                        viewer::handle_view_request(&app_data_dir, work_id, page_index)
                    }
                    Err(_) => tauri::http::Response::builder()
                        .status(500)
                        .body(Vec::new())
                        .unwrap(),
                },
                None => tauri::http::Response::builder()
                    .status(400)
                    .body(Vec::new())
                    .unwrap(),
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_works,
            get_thumbnail,
            get_work,
            get_settings,
            set_library_root,
            set_directory_template,
            validate_template,
            preview_template,
            parse_folder_name,
            preview_import_path,
            import_work,
            discover_folders,
            bulk_import,
            preview_relocation,
            relocate_works,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
