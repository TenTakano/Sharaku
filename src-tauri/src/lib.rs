mod db;
mod error;
mod scanner;
mod thumbnail;
mod viewer;

use std::path::PathBuf;

use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};

use db::{WorkDetail, WorkSummary};
use scanner::ScanProgress;

#[tauri::command]
async fn scan_library(
    app: tauri::AppHandle,
    root_path: String,
    on_progress: tauri::ipc::Channel<ScanProgress>,
) -> Result<(), String> {
    let app_data_dir: PathBuf = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let root = PathBuf::from(root_path);

    tokio::task::spawn_blocking(move || {
        let result = scanner::scan_directory(&root, &app_data_dir, &on_progress);
        if let Err(ref e) = result {
            let _ = on_progress.send(ScanProgress::Error {
                message: e.to_string(),
            });
        }
        result
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![Migration {
        version: 1,
        description: "create_initial_tables",
        sql: include_str!("../migrations/001_create_initial_tables.sql"),
        kind: MigrationKind::Up,
    }];

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
                }
                None => tauri::http::Response::builder()
                    .status(400)
                    .body(Vec::new())
                    .unwrap(),
            }
        })
        .invoke_handler(tauri::generate_handler![
            scan_library,
            list_works,
            get_thumbnail,
            get_work,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
