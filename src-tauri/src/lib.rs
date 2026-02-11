mod db;
mod error;
mod scanner;
mod thumbnail;

use std::path::PathBuf;

use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};

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
        .invoke_handler(tauri::generate_handler![scan_library])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
