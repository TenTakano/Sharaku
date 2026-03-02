mod db;
mod error;
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

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::Manager;

use db::{Tag, WorkDetail, WorkSummary};
use importer::{
    BulkImportProgress, BulkImportSummary, DiscoverProgress, DiscoveredFolder, ImportResult,
    ParsedMetadata,
};
use integrity::{IntegrityCheckProgress, IntegrityReport};
use library::Library;
use relocator::{RelocationPreview, RelocationProgress};
use serde::Serialize;
use template::WorkMetadata;

struct ActiveLibrary {
    id: String,
    path: Option<PathBuf>,
}

struct AppDb {
    conn: Connection,
    active_library: Option<ActiveLibrary>,
}

pub struct AppState {
    db: Arc<Mutex<AppDb>>,
}

// --- Library CRUD commands ---

#[tauri::command]
async fn list_libraries(state: tauri::State<'_, AppState>) -> Result<Vec<Library>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        library::list_libraries(&guard.conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_active_library(state: tauri::State<'_, AppState>) -> Result<Option<Library>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        library::active_library(&guard.conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn create_library(
    state: tauri::State<'_, AppState>,
    name: String,
    path: Option<String>,
    resource_mode: String,
) -> Result<Library, String> {
    if resource_mode != "full" && resource_mode != "metadata_only" {
        return Err("無効なリソース管理モードです".to_string());
    }
    if resource_mode == "full" && path.is_none() {
        return Err("フルモードではパスの指定が必須です".to_string());
    }
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let mut guard = db.lock().unwrap();
        let lib =
            library::add_library(&guard.conn, &name, path.as_deref()).map_err(|e| e.to_string())?;
        library::set_active_library(&guard.conn, &lib.id).map_err(|e| e.to_string())?;
        settings::set_resource_mode(&guard.conn, &lib.id, &resource_mode)
            .map_err(|e| e.to_string())?;
        guard.active_library = Some(ActiveLibrary {
            id: lib.id.clone(),
            path: lib.path.as_ref().map(PathBuf::from),
        });
        Ok(lib)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn switch_library(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let mut guard = db.lock().unwrap();
        let lib = library::find_library_by_id(&guard.conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "ライブラリが見つかりません".to_string())?;
        library::set_active_library(&guard.conn, &id).map_err(|e| e.to_string())?;
        guard.active_library = Some(ActiveLibrary {
            id: lib.id,
            path: lib.path.map(PathBuf::from),
        });
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn remove_library(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let mut guard = db.lock().unwrap();
        library::remove_library(&guard.conn, &id).map_err(|e| e.to_string())?;
        let active = library::active_library(&guard.conn).map_err(|e| e.to_string())?;
        guard.active_library = active.map(|lib| ActiveLibrary {
            id: lib.id,
            path: lib.path.map(PathBuf::from),
        });
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- Existing commands (migrated to unified DB) ---

#[tauri::command]
async fn list_works(
    state: tauri::State<'_, AppState>,
    sort_by: String,
    sort_order: String,
) -> Result<Vec<WorkSummary>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::list_works(&guard.conn, &active.id, &sort_by, &sort_order).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_thumbnail(state: tauri::State<'_, AppState>, work_id: i64) -> Result<Vec<u8>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::get_thumbnail(&guard.conn, work_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_work(state: tauri::State<'_, AppState>, work_id: i64) -> Result<WorkDetail, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::get_work(&guard.conn, work_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    resource_mode: String,
    directory_template: Option<String>,
    type_label_image: String,
    type_label_folder: String,
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        let resource_mode =
            settings::get_resource_mode(&guard.conn, &active.id).map_err(|e| e.to_string())?;
        let directory_template =
            settings::get_directory_template(&guard.conn, &active.id).map_err(|e| e.to_string())?;
        let type_label_image =
            settings::get_type_label_image(&guard.conn, &active.id).map_err(|e| e.to_string())?;
        let type_label_folder =
            settings::get_type_label_folder(&guard.conn, &active.id).map_err(|e| e.to_string())?;
        Ok(AppSettings {
            resource_mode,
            directory_template,
            type_label_image,
            type_label_folder,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_resource_mode(state: tauri::State<'_, AppState>, mode: String) -> Result<(), String> {
    if mode != "full" && mode != "metadata_only" {
        return Err("無効なリソース管理モードです".to_string());
    }
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        settings::set_resource_mode(&guard.conn, &active.id, &mode).map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        settings::set_directory_template(&guard.conn, &active.id, &trimmed)
            .map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        settings::set_type_label_image(&guard.conn, &active.id, &image_label)
            .map_err(|e| e.to_string())?;
        settings::set_type_label_folder(&guard.conn, &active.id, &folder_label)
            .map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        let folder_label =
            settings::get_type_label_folder(&guard.conn, &active.id).map_err(|e| e.to_string())?;
        let mut metadata = template::sample_metadata();
        metadata.work_type = Some(folder_label);
        Ok(template::render_template(&template, &metadata))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn has_image_subfolders(dir: String) -> Result<bool, String> {
    let path = std::path::Path::new(&dir);
    if !path.is_dir() {
        return Err("有効なディレクトリではありません".to_string());
    }
    Ok(scanner::has_image_subfolders(path))
}

#[tauri::command]
async fn resolve_drop_path(path: String) -> Result<String, String> {
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
async fn parse_folder_name(folder_name: String) -> Result<ParsedMetadata, String> {
    Ok(importer::parse_folder_name(&folder_name))
}

#[tauri::command]
async fn preview_import_path(
    state: tauri::State<'_, AppState>,
    metadata: WorkMetadata,
) -> Result<String, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        let lib_path = active
            .path
            .as_ref()
            .ok_or("ライブラリルートが設定されていません")?;
        let template_str = settings::get_directory_template(&guard.conn, &active.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "ディレクトリテンプレートが設定されていません".to_string())?;
        Ok(importer::preview_import_path(
            lib_path,
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        let lib_path = active
            .path
            .as_ref()
            .ok_or("ライブラリルートが設定されていません")?;
        importer::import_work(&request, &guard.conn, &active.id, lib_path)
            .map_err(|e| e.to_string())
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
    let db = state.db.clone();
    let root = PathBuf::from(root_path);
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        importer::discover_image_folders(&root, &guard.conn, &active.id, &on_progress)
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        let lib_path = active
            .path
            .as_ref()
            .ok_or("ライブラリルートが設定されていません")?;
        importer::bulk_import(&requests, &guard.conn, &active.id, lib_path, &on_progress)
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        let lib_path = active
            .path
            .as_ref()
            .ok_or("ライブラリルートが設定されていません")?;
        relocator::preview_relocation(&guard.conn, &active.id, lib_path, &trimmed)
            .map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        let lib_path = active
            .path
            .as_ref()
            .ok_or("ライブラリルートが設定されていません")?;
        relocator::execute_relocation(&guard.conn, &active.id, lib_path, &trimmed, &on_progress)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn list_tags(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::list_tags(&guard.conn, &active.id).map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::search_tags(&guard.conn, &active.id, &query, category.as_deref())
            .map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::create_tag(&guard.conn, &active.id, &name, category.as_deref())
            .map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::update_tag(&guard.conn, id, &name, category.as_deref()).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn delete_tag(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::delete_tag(&guard.conn, id).map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::add_tag_to_work(&guard.conn, work_id, tag_id).map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::remove_tag_from_work(&guard.conn, work_id, tag_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_tags_for_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
) -> Result<Vec<Tag>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::get_tags_for_work(&guard.conn, work_id).map_err(|e| e.to_string())
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
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        db::search_works_by_tags(&guard.conn, &active.id, &tag_ids, &mode)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn check_integrity(
    state: tauri::State<'_, AppState>,
    on_progress: tauri::ipc::Channel<IntegrityCheckProgress>,
) -> Result<IntegrityReport, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        integrity::check_integrity(
            &guard.conn,
            &active.id,
            active.path.as_deref(),
            &on_progress,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn delete_orphan_works(
    state: tauri::State<'_, AppState>,
    ids: Vec<i64>,
) -> Result<usize, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        integrity::delete_orphan_works(&guard.conn, &active.id, &ids).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn migrate_libraries_json(conn: &Connection, app_data_dir: &std::path::Path) {
    let json_path = app_data_dir.join("libraries.json");
    if !json_path.exists() {
        return;
    }

    #[derive(serde::Deserialize)]
    struct LibraryStoreData {
        libraries: Vec<JsonLibrary>,
        active_library_id: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct JsonLibrary {
        id: String,
        name: String,
        path: String,
    }

    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let data: LibraryStoreData = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => return,
    };

    for lib in &data.libraries {
        let already_exists: bool = conn
            .prepare_cached("SELECT 1 FROM libraries WHERE id = ?1")
            .and_then(|mut stmt| stmt.exists([&lib.id]))
            .unwrap_or(true);
        if already_exists {
            continue;
        }
        let is_active = data.active_library_id.as_deref() == Some(&lib.id);
        let _ = conn.execute(
            "INSERT INTO libraries (id, name, path, is_active) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![lib.id, lib.name, lib.path, is_active as i32],
        );
    }

    let migrated_path = json_path.with_extension("json.migrated");
    let _ = std::fs::rename(&json_path, &migrated_path);
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
            migrate_libraries_json(&conn, &app_data_dir);

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

            app.manage(AppState {
                db: Arc::new(Mutex::new(AppDb {
                    conn,
                    active_library,
                })),
            });

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
            list_libraries,
            get_active_library,
            create_library,
            switch_library,
            remove_library,
            list_works,
            get_thumbnail,
            get_work,
            get_settings,
            set_resource_mode,
            set_directory_template,
            set_type_labels,
            validate_template,
            preview_template,
            has_image_subfolders,
            resolve_drop_path,
            parse_folder_name,
            preview_import_path,
            import_work,
            discover_folders,
            bulk_import,
            preview_relocation,
            relocate_works,
            list_tags,
            search_tags,
            create_tag,
            update_tag,
            delete_tag,
            add_tag_to_work,
            remove_tag_from_work,
            get_tags_for_work,
            search_works_by_tags,
            check_integrity,
            delete_orphan_works,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
