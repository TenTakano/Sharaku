mod commands;
mod db;
mod error;
mod import_queue;
mod importer;
mod integrity;
mod library;
mod migration;
mod relocator;
mod scanner;
mod settings;
mod template;
mod thumbnail;
mod viewer;

#[cfg(test)]
#[path = "tests/common.rs"]
mod test_common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::Manager;

use import_queue::ImportQueue;

struct ActiveLibrary {
    id: String,
    path: Option<PathBuf>,
}

struct AppDb {
    conn: Connection,
    active_library: Option<ActiveLibrary>,
}

impl AppDb {
    fn active_library(&self) -> Result<&ActiveLibrary, String> {
        self.active_library
            .as_ref()
            .ok_or_else(|| "ライブラリが選択されていません".to_string())
    }
}

pub struct AppState {
    db: Arc<Mutex<AppDb>>,
    import_queue: ImportQueue,
}

impl AppState {
    /// Locks the AppDb on spawn_blocking and passes it to the closure.
    /// Does not require an active library to be selected.
    async fn with_db<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&AppDb) -> Result<T, String> + Send + 'static,
        T: Send + 'static,
    {
        let app_db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let guard = app_db.lock().unwrap();
            f(&guard)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Locks the AppDb mutably on spawn_blocking and passes it to the closure (for state updates like library CRUD).
    async fn with_db_mut<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut AppDb) -> Result<T, String> + Send + 'static,
        T: Send + 'static,
    {
        let app_db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = app_db.lock().unwrap();
            f(&mut guard)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Requires an active library, passing both conn and the active library to the closure.
    async fn with_active_db<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&AppDb, &ActiveLibrary) -> Result<T, String> + Send + 'static,
        T: Send + 'static,
    {
        self.with_db(move |db| {
            let active = db.active_library()?;
            f(db, active)
        })
        .await
    }

    /// Only checks whether an active library is selected as a guard, passing just conn to the closure.
    async fn with_guarded_db<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&AppDb) -> Result<T, String> + Send + 'static,
        T: Send + 'static,
    {
        self.with_db(move |db| {
            db.active_library()?;
            f(db)
        })
        .await
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = {
                let base = app.path().app_data_dir()?;
                if cfg!(debug_assertions) {
                    base.with_file_name("com.sharaku.viewer.dev")
                } else {
                    base
                }
            };

            let conn = db::open_db(&app_data_dir).expect("Failed to open database");
            migration::migrate_libraries_json(&conn, &app_data_dir);

            let migration_errors = migration::migrate_per_library_dbs(&conn);
            for err in &migration_errors {
                eprintln!(
                    "Warning: データ移行に失敗 (library: {}, path: {}): {}",
                    err.library_id, err.library_path, err.message
                );
            }

            let active_library =
                library::active_library(&conn)
                    .ok()
                    .flatten()
                    .map(|lib| ActiveLibrary {
                        id: lib.id,
                        path: lib.path.map(PathBuf::from),
                    });

            let db = Arc::new(Mutex::new(AppDb {
                conn,
                active_library,
            }));
            let import_queue = ImportQueue::new(app.handle().clone(), db.clone());

            app.manage(AppState { db, import_queue });

            Ok(())
        })
        .register_uri_scheme_protocol("sharaku", |ctx, request| {
            let uri = request.uri().to_string();
            match viewer::parse_view_uri(&uri) {
                Some((work_id, page_index)) => {
                    let state = ctx.app_handle().state::<AppState>();
                    let guard = state.db.lock().unwrap();
                    match guard.active_library.as_ref() {
                        Some(_) => viewer::handle_view_request(&guard.conn, work_id, page_index),
                        None => tauri::http::Response::builder()
                            .status(500)
                            .body(Vec::new())
                            .unwrap(),
                    }
                }
                None => tauri::http::Response::builder()
                    .status(400)
                    .body(Vec::new())
                    .unwrap(),
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::list_libraries,
            commands::library::get_active_library,
            commands::library::create_library,
            commands::library::switch_library,
            commands::library::remove_library,
            commands::work::list_works,
            commands::work::get_thumbnail,
            commands::work::get_work,
            commands::work::update_work,
            commands::settings::get_settings,
            commands::settings::set_resource_mode,
            commands::settings::set_delete_file_action,
            commands::settings::set_directory_template,
            commands::settings::set_type_labels,
            commands::settings::validate_template,
            commands::settings::preview_template,
            commands::import::has_image_subfolders,
            commands::import::resolve_drop_path,
            commands::import::parse_folder_name,
            commands::import::preview_import_path,
            commands::import::enqueue_import,
            commands::import::discover_folders,
            commands::import::preview_relocation,
            commands::import::relocate_works,
            commands::tag::list_tags,
            commands::tag::search_tags,
            commands::tag::create_tag,
            commands::tag::update_tag,
            commands::tag::delete_tag,
            commands::tag::add_tag_to_work,
            commands::tag::remove_tag_from_work,
            commands::tag::get_tags_for_work,
            commands::tag::search_works_by_tags,
            commands::work::delete_work,
            commands::work::check_integrity,
            commands::work::delete_orphan_works,
            commands::settings::get_theme,
            commands::settings::set_theme,
            commands::settings::get_banner_auto_close,
            commands::settings::set_banner_auto_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
