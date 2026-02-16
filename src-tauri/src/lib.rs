mod db;
mod error;
mod importer;
mod library;
mod relocator;
mod scanner;
mod settings;
mod template;
mod thumbnail;
mod viewer;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::Manager;

use db::{Tag, WorkDetail, WorkSummary};
use importer::{
    BulkImportProgress, BulkImportSummary, DiscoverProgress, DiscoveredFolder, ImportResult,
    ParsedMetadata,
};
use library::Library;
use relocator::{RelocationPreview, RelocationProgress};
use serde::Serialize;
use template::WorkMetadata;

struct ActiveLibrary {
    path: PathBuf,
    conn: Connection,
}

pub struct AppState {
    app_data_dir: PathBuf,
    active_library: Arc<Mutex<Option<ActiveLibrary>>>,
}

// --- Library CRUD commands ---

#[tauri::command]
async fn list_libraries(state: tauri::State<'_, AppState>) -> Result<Vec<Library>, String> {
    let app_data_dir = state.app_data_dir.clone();
    tokio::task::spawn_blocking(move || {
        let store = library::LibraryStore::new(&app_data_dir);
        let (libraries, _) = store.load().map_err(|e| e.to_string())?;
        Ok(libraries)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_active_library(state: tauri::State<'_, AppState>) -> Result<Option<Library>, String> {
    let app_data_dir = state.app_data_dir.clone();
    tokio::task::spawn_blocking(move || {
        let store = library::LibraryStore::new(&app_data_dir);
        store.active_library().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn create_library(
    state: tauri::State<'_, AppState>,
    name: String,
    path: String,
) -> Result<Library, String> {
    let app_data_dir = state.app_data_dir.clone();
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let store = library::LibraryStore::new(&app_data_dir);
        let lib = store.add(&name, &path).map_err(|e| e.to_string())?;
        let library_root = PathBuf::from(&lib.path);
        match db::open_db(&library_root) {
            Ok(conn) => {
                store.set_active(&lib.id).map_err(|e| e.to_string())?;
                *db.lock().unwrap() = Some(ActiveLibrary {
                    path: library_root,
                    conn,
                });
                Ok(lib)
            }
            Err(e) => {
                let _ = store.remove(&lib.id);
                Err(e.to_string())
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn switch_library(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let app_data_dir = state.app_data_dir.clone();
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let store = library::LibraryStore::new(&app_data_dir);
        let lib = store
            .find_by_id(&id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "ライブラリが見つかりません".to_string())?;
        let library_root = PathBuf::from(&lib.path);
        let conn = db::open_db(&library_root).map_err(|e| e.to_string())?;
        store.set_active(&id).map_err(|e| e.to_string())?;
        *db.lock().unwrap() = Some(ActiveLibrary {
            path: library_root,
            conn,
        });
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn remove_library(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let app_data_dir = state.app_data_dir.clone();
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let store = library::LibraryStore::new(&app_data_dir);
        store.remove(&id).map_err(|e| e.to_string())?;
        let active = store.active_library().map_err(|e| e.to_string())?;
        let new_active = match active {
            Some(lib) => {
                let library_root = PathBuf::from(&lib.path);
                db::open_db(&library_root)
                    .ok()
                    .map(|conn| ActiveLibrary {
                        path: library_root,
                        conn,
                    })
            }
            None => None,
        };
        *db.lock().unwrap() = new_active;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- Existing commands (migrated to AppState) ---

#[tauri::command]
async fn list_works(
    state: tauri::State<'_, AppState>,
    sort_by: String,
    sort_order: String,
) -> Result<Vec<WorkSummary>, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::list_works(&lib.conn, &sort_by, &sort_order).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_thumbnail(state: tauri::State<'_, AppState>, work_id: i64) -> Result<Vec<u8>, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::get_thumbnail(&lib.conn, work_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_work(state: tauri::State<'_, AppState>, work_id: i64) -> Result<WorkDetail, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::get_work(&lib.conn, work_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    directory_template: Option<String>,
    type_label_image: String,
    type_label_folder: String,
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        let directory_template =
            settings::get_directory_template(&lib.conn).map_err(|e| e.to_string())?;
        let type_label_image =
            settings::get_type_label_image(&lib.conn).map_err(|e| e.to_string())?;
        let type_label_folder =
            settings::get_type_label_folder(&lib.conn).map_err(|e| e.to_string())?;
        Ok(AppSettings {
            directory_template,
            type_label_image,
            type_label_folder,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_directory_template(
    state: tauri::State<'_, AppState>,
    template: String,
) -> Result<(), String> {
    let trimmed = template.trim().to_string();
    if !trimmed.is_empty() {
        template::validate_template(&trimmed).map_err(|e| e.to_string())?;
    }
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        settings::set_directory_template(&lib.conn, &trimmed).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn validate_template(template: String) -> Result<(), String> {
    template::validate_template(&template).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_type_labels(
    state: tauri::State<'_, AppState>,
    image_label: String,
    folder_label: String,
) -> Result<(), String> {
    let image_label = image_label.trim().to_string();
    let folder_label = folder_label.trim().to_string();
    if image_label.is_empty() || folder_label.is_empty() {
        return Err("ラベルは空にできません".to_string());
    }
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        settings::set_type_label_image(&lib.conn, &image_label).map_err(|e| e.to_string())?;
        settings::set_type_label_folder(&lib.conn, &folder_label).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn preview_template(
    state: tauri::State<'_, AppState>,
    template: String,
) -> Result<String, String> {
    template::validate_template(&template).map_err(|e| e.to_string())?;
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        let folder_label =
            settings::get_type_label_folder(&lib.conn).map_err(|e| e.to_string())?;
        let mut metadata = template::sample_metadata();
        metadata.work_type = Some(folder_label);
        Ok(template::render_template(&template, &metadata))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn parse_folder_name(folder_name: String) -> Result<ParsedMetadata, String> {
    Ok(importer::parse_folder_name(&folder_name))
}

#[tauri::command]
async fn preview_import_path(
    state: tauri::State<'_, AppState>,
    metadata: WorkMetadata,
) -> Result<String, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        let template_str = settings::get_directory_template(&lib.conn)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "ディレクトリテンプレートが設定されていません".to_string())?;
        Ok(importer::preview_import_path(
            &lib.path,
            &template_str,
            &metadata,
        ))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_work(
    state: tauri::State<'_, AppState>,
    request: importer::ImportRequest,
) -> Result<ImportResult, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        importer::import_work(&request, &lib.conn, &lib.path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn discover_folders(
    state: tauri::State<'_, AppState>,
    root_path: String,
    on_progress: tauri::ipc::Channel<DiscoverProgress>,
) -> Result<Vec<DiscoveredFolder>, String> {
    let db = state.active_library.clone();
    let root = PathBuf::from(root_path);
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        importer::discover_image_folders(&root, &lib.conn, &on_progress)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn bulk_import(
    state: tauri::State<'_, AppState>,
    requests: Vec<importer::ImportRequest>,
    on_progress: tauri::ipc::Channel<BulkImportProgress>,
) -> Result<BulkImportSummary, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        importer::bulk_import(&requests, &lib.conn, &lib.path, &on_progress)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn preview_relocation(
    state: tauri::State<'_, AppState>,
    new_template: String,
) -> Result<Vec<RelocationPreview>, String> {
    let trimmed = new_template.trim().to_string();
    template::validate_template(&trimmed).map_err(|e| e.to_string())?;
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        relocator::preview_relocation(&lib.conn, &lib.path, &trimmed).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn relocate_works(
    state: tauri::State<'_, AppState>,
    new_template: String,
    on_progress: tauri::ipc::Channel<RelocationProgress>,
) -> Result<(), String> {
    let trimmed = new_template.trim().to_string();
    template::validate_template(&trimmed).map_err(|e| e.to_string())?;
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        relocator::execute_relocation(&lib.conn, &lib.path, &trimmed, &on_progress)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn search_tags(
    state: tauri::State<'_, AppState>,
    query: String,
    category: Option<String>,
) -> Result<Vec<Tag>, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::search_tags(&lib.conn, &query, category.as_deref()).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn create_tag(
    state: tauri::State<'_, AppState>,
    name: String,
    category: Option<String>,
) -> Result<Tag, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::create_tag(&lib.conn, &name, category.as_deref()).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn update_tag(
    state: tauri::State<'_, AppState>,
    id: i64,
    name: String,
    category: Option<String>,
) -> Result<(), String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::update_tag(&lib.conn, id, &name, category.as_deref()).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn delete_tag(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::delete_tag(&lib.conn, id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn add_tag_to_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::add_tag_to_work(&lib.conn, work_id, tag_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn remove_tag_from_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::remove_tag_from_work(&lib.conn, work_id, tag_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_tags_for_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
) -> Result<Vec<Tag>, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::get_tags_for_work(&lib.conn, work_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn search_works_by_tags(
    state: tauri::State<'_, AppState>,
    tag_ids: Vec<i64>,
    mode: String,
) -> Result<Vec<WorkSummary>, String> {
    let db = state.active_library.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let lib = guard.as_ref().ok_or("ライブラリが選択されていません")?;
        db::search_works_by_tags(&lib.conn, &tag_ids, &mode).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;

            let store = library::LibraryStore::new(&app_data_dir);
            let active_library = store
                .active_library()
                .ok()
                .flatten()
                .and_then(|lib| {
                    let path = PathBuf::from(&lib.path);
                    db::open_db(&path).ok().map(|conn| ActiveLibrary { path, conn })
                });

            app.manage(AppState {
                app_data_dir,
                active_library: Arc::new(Mutex::new(active_library)),
            });

            Ok(())
        })
        .register_uri_scheme_protocol("sharaku", |ctx, request| {
            let uri = request.uri().to_string();
            match viewer::parse_view_uri(&uri) {
                Some((work_id, page_index)) => {
                    let state = ctx.app_handle().state::<AppState>();
                    let guard = state.active_library.lock().unwrap();
                    match guard.as_ref() {
                        Some(lib) => {
                            viewer::handle_view_request(&lib.conn, work_id, page_index)
                        }
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
            list_libraries,
            get_active_library,
            create_library,
            switch_library,
            remove_library,
            list_works,
            get_thumbnail,
            get_work,
            get_settings,
            set_directory_template,
            set_type_labels,
            validate_template,
            preview_template,
            parse_folder_name,
            preview_import_path,
            import_work,
            discover_folders,
            bulk_import,
            preview_relocation,
            relocate_works,
            search_tags,
            create_tag,
            update_tag,
            delete_tag,
            add_tag_to_work,
            remove_tag_from_work,
            get_tags_for_work,
            search_works_by_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
