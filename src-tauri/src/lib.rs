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

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::{AppHandle, Emitter, Manager};

use db::{Tag, WorkDetail, WorkSummary};
use import_queue::{ImportJob, ImportQueue, ImportQueueEvent};
use importer::{DiscoverProgress, DiscoveredFolder, ParsedMetadata};
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
    /// AppDb を spawn_blocking 上でロックし、クロージャに渡す。
    /// アクティブライブラリの選択有無は問わない。
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

    /// AppDb を可変で spawn_blocking 上でロックし、クロージャに渡す（ライブラリCRUD等の状態更新用）。
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

    /// アクティブライブラリを要求し、conn とアクティブライブラリの両方をクロージャに渡す。
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

    /// アクティブライブラリの選択有無のみをガードとして確認し、conn だけをクロージャに渡す。
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

// --- Library CRUD commands ---

#[tauri::command]
async fn list_libraries(state: tauri::State<'_, AppState>) -> Result<Vec<Library>, String> {
    state
        .with_db(|db| library::list_libraries(&db.conn).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
async fn get_active_library(state: tauri::State<'_, AppState>) -> Result<Option<Library>, String> {
    state
        .with_db(|db| library::active_library(&db.conn).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
async fn create_library(
    state: tauri::State<'_, AppState>,
    name: String,
    path: Option<String>,
    resource_mode: String,
    directory_template: Option<String>,
) -> Result<Library, String> {
    if resource_mode != "full" && resource_mode != "metadata_only" {
        return Err("無効なリソース管理モードです".to_string());
    }
    if resource_mode == "full" && path.is_none() {
        return Err("フルモードではパスの指定が必須です".to_string());
    }
    if resource_mode == "full" {
        let tmpl = directory_template
            .as_deref()
            .unwrap_or(settings::DEFAULT_DIRECTORY_TEMPLATE);
        template::validate_template(tmpl).map_err(|e| e.to_string())?;
    }
    state
        .with_db_mut(move |db| {
            let lib = library::add_library(&db.conn, &name, path.as_deref())
                .map_err(|e| e.to_string())?;
            library::set_active_library(&db.conn, &lib.id).map_err(|e| e.to_string())?;
            settings::set_resource_mode(&db.conn, &lib.id, &resource_mode)
                .map_err(|e| e.to_string())?;
            if resource_mode == "full" {
                let tmpl = directory_template
                    .as_deref()
                    .unwrap_or(settings::DEFAULT_DIRECTORY_TEMPLATE);
                settings::set_directory_template(&db.conn, &lib.id, tmpl)
                    .map_err(|e| e.to_string())?;
            }
            db.active_library = Some(ActiveLibrary {
                id: lib.id.clone(),
                path: lib.path.as_ref().map(PathBuf::from),
            });
            Ok(lib)
        })
        .await
}

#[tauri::command]
async fn switch_library(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .with_db_mut(move |db| {
            let lib = library::find_library_by_id(&db.conn, &id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "ライブラリが見つかりません".to_string())?;
            library::set_active_library(&db.conn, &id).map_err(|e| e.to_string())?;
            db.active_library = Some(ActiveLibrary {
                id: lib.id,
                path: lib.path.map(PathBuf::from),
            });
            Ok(())
        })
        .await
}

#[tauri::command]
async fn remove_library(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .with_db_mut(move |db| {
            library::remove_library(&db.conn, &id).map_err(|e| e.to_string())?;
            let active = library::active_library(&db.conn).map_err(|e| e.to_string())?;
            db.active_library = active.map(|lib| ActiveLibrary {
                id: lib.id,
                path: lib.path.map(PathBuf::from),
            });
            Ok(())
        })
        .await
}

// --- Existing commands (migrated to unified DB) ---

#[tauri::command]
async fn list_works(
    state: tauri::State<'_, AppState>,
    sort_by: String,
    sort_order: String,
) -> Result<Vec<WorkSummary>, String> {
    state
        .with_active_db(move |db, active| {
            db::list_works(&db.conn, &active.id, &sort_by, &sort_order).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn get_thumbnail(state: tauri::State<'_, AppState>, work_id: i64) -> Result<Vec<u8>, String> {
    state
        .with_guarded_db(move |db| db::get_thumbnail(&db.conn, work_id).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
async fn get_work(state: tauri::State<'_, AppState>, work_id: i64) -> Result<WorkDetail, String> {
    state
        .with_guarded_db(move |db| db::get_work(&db.conn, work_id).map_err(|e| e.to_string()))
        .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    resource_mode: String,
    directory_template: Option<String>,
    type_label_image: String,
    type_label_folder: String,
    delete_file_action: String,
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    state
        .with_active_db(move |db, active| {
            let resource_mode =
                settings::get_resource_mode(&db.conn, &active.id).map_err(|e| e.to_string())?;
            let directory_template = settings::get_directory_template(&db.conn, &active.id)
                .map_err(|e| e.to_string())?;
            let type_label_image = settings::get_type_label_image(&db.conn, &active.id)
                .map_err(|e| e.to_string())?;
            let type_label_folder = settings::get_type_label_folder(&db.conn, &active.id)
                .map_err(|e| e.to_string())?;
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
async fn set_resource_mode(state: tauri::State<'_, AppState>, mode: String) -> Result<(), String> {
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
async fn set_delete_file_action(
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
async fn set_directory_template(
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
async fn validate_template(template: String) -> Result<(), String> {
    template::validate_template(&template).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_type_labels(
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
async fn preview_template(
    state: tauri::State<'_, AppState>,
    template: String,
) -> Result<String, String> {
    template::validate_template(&template).map_err(|e| e.to_string())?;
    state
        .with_active_db(move |db, active| {
            let folder_label = settings::get_type_label_folder(&db.conn, &active.id)
                .map_err(|e| e.to_string())?;
            let mut metadata = template::sample_metadata();
            metadata.work_type = Some(folder_label);
            Ok(template::render_template(&template, &metadata))
        })
        .await
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
struct EnqueueResult {
    job_id: String,
}

#[tauri::command]
async fn enqueue_import(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    requests: Vec<importer::ImportRequest>,
) -> Result<EnqueueResult, String> {
    let (library_id, library_root, resource_mode) = {
        let guard = state.db.lock().unwrap();
        let active = guard
            .active_library
            .as_ref()
            .ok_or("ライブラリが選択されていません")?;
        let mode =
            settings::get_resource_mode(&guard.conn, &active.id).map_err(|e| e.to_string())?;
        (active.id.clone(), active.path.clone(), mode)
    };

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
async fn discover_folders(
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
async fn preview_relocation(
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
async fn relocate_works(
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

#[tauri::command]
async fn list_tags(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, String> {
    state
        .with_active_db(|db, active| {
            db::list_tags(&db.conn, &active.id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn search_tags(
    state: tauri::State<'_, AppState>,
    query: String,
    category: Option<String>,
) -> Result<Vec<Tag>, String> {
    state
        .with_active_db(move |db, active| {
            db::search_tags(&db.conn, &active.id, &query, category.as_deref())
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn create_tag(
    state: tauri::State<'_, AppState>,
    name: String,
    category: Option<String>,
) -> Result<Tag, String> {
    state
        .with_active_db(move |db, active| {
            db::create_tag(&db.conn, &active.id, &name, category.as_deref())
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn update_work(
    state: tauri::State<'_, AppState>,
    id: i64,
    title: String,
    artist: Option<String>,
    year: Option<i32>,
    genre: Option<String>,
    circle: Option<String>,
    origin: Option<String>,
) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("タイトルは空にできません".to_string());
    }
    state
        .with_guarded_db(move |db| {
            db::update_work(
                &db.conn,
                id,
                title.trim(),
                artist.as_deref(),
                year,
                genre.as_deref(),
                circle.as_deref(),
                origin.as_deref(),
            )
            .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn update_tag(
    state: tauri::State<'_, AppState>,
    id: i64,
    name: String,
    category: Option<String>,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            db::update_tag(&db.conn, id, &name, category.as_deref()).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn delete_tag(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    state
        .with_guarded_db(move |db| db::delete_tag(&db.conn, id).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
async fn add_tag_to_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            db::add_tag_to_work(&db.conn, work_id, tag_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn remove_tag_from_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    state
        .with_guarded_db(move |db| {
            db::remove_tag_from_work(&db.conn, work_id, tag_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn get_tags_for_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
) -> Result<Vec<Tag>, String> {
    state
        .with_guarded_db(move |db| {
            db::get_tags_for_work(&db.conn, work_id).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn search_works_by_tags(
    state: tauri::State<'_, AppState>,
    tag_ids: Vec<i64>,
    mode: String,
) -> Result<Vec<WorkSummary>, String> {
    state
        .with_active_db(move |db, active| {
            db::search_works_by_tags(&db.conn, &active.id, &tag_ids, &mode)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn delete_work(
    state: tauri::State<'_, AppState>,
    work_id: i64,
    file_action: String,
) -> Result<(), String> {
    state
        .with_active_db(move |db, active| {
            match file_action.as_str() {
                "delete" => {
                    let work = db::get_work(&db.conn, work_id).map_err(|e| e.to_string())?;
                    let path = std::path::Path::new(&work.path);
                    let lib_path = active
                        .path
                        .as_ref()
                        .ok_or("ライブラリルートが設定されていません")?;
                    let canonical_lib = lib_path.canonicalize().unwrap_or_else(|_| lib_path.clone());
                    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                    if !canonical_path.starts_with(&canonical_lib) {
                        return Err("作品パスがライブラリルート外にあるため削除できません".to_string());
                    }
                    if path.exists() {
                        if path.is_dir() {
                            std::fs::remove_dir_all(path).map_err(|e| e.to_string())?;
                        } else {
                            std::fs::remove_file(path).map_err(|e| e.to_string())?;
                        }
                    }
                }
                "trash" => {
                    let work = db::get_work(&db.conn, work_id).map_err(|e| e.to_string())?;
                    let src = std::path::Path::new(&work.path);
                    let lib_path = active
                        .path
                        .as_ref()
                        .ok_or("ライブラリルートが設定されていません")?;
                    let canonical_lib = lib_path.canonicalize().unwrap_or_else(|_| lib_path.clone());
                    let canonical_src = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
                    if !canonical_src.starts_with(&canonical_lib) {
                        return Err("作品パスがライブラリルート外にあるため移動できません".to_string());
                    }
                    if src.exists() {
                        let trash_dir = lib_path.join(".trash");
                        std::fs::create_dir_all(&trash_dir).map_err(|e| e.to_string())?;

                        let file_name = src.file_name().ok_or("ファイル名の取得に失敗しました")?;
                        let mut dest = trash_dir.join(file_name);
                        if dest.exists() {
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let stem = std::path::Path::new(file_name)
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy();
                            let ext = std::path::Path::new(file_name)
                                .extension()
                                .map(|e| format!(".{}", e.to_string_lossy()))
                                .unwrap_or_default();
                            dest = trash_dir.join(format!("{}_{}{}", stem, timestamp, ext));
                        }
                        std::fs::rename(src, &dest).map_err(|e| e.to_string())?;
                    }
                }
                _ => {} // "none" — metadata only
            }

            db::delete_works_by_ids(&db.conn, &active.id, &[work_id]).map_err(|e| e.to_string())?;
            Ok(())
        })
        .await
}

#[tauri::command]
async fn check_integrity(
    state: tauri::State<'_, AppState>,
    on_progress: tauri::ipc::Channel<IntegrityCheckProgress>,
) -> Result<IntegrityReport, String> {
    state
        .with_active_db(move |db, active| {
            integrity::check_integrity(&db.conn, &active.id, active.path.as_deref(), &on_progress)
                .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn delete_orphan_works(
    state: tauri::State<'_, AppState>,
    ids: Vec<i64>,
) -> Result<usize, String> {
    state
        .with_active_db(move |db, active| {
            integrity::delete_orphan_works(&db.conn, &active.id, &ids).map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
async fn get_theme(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state
        .with_db(|db| settings::get_theme_mode(&db.conn).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
async fn set_theme(state: tauri::State<'_, AppState>, mode: String) -> Result<(), String> {
    state
        .with_db(move |db| settings::set_theme_mode(&db.conn, &mode).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
async fn get_banner_auto_close(state: tauri::State<'_, AppState>) -> Result<u32, String> {
    state
        .with_db(|db| settings::get_banner_auto_close(&db.conn).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
async fn set_banner_auto_close(
    state: tauri::State<'_, AppState>,
    seconds: u32,
) -> Result<(), String> {
    state
        .with_db(move |db| {
            settings::set_banner_auto_close(&db.conn, seconds).map_err(|e| e.to_string())
        })
        .await
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
            list_libraries,
            get_active_library,
            create_library,
            switch_library,
            remove_library,
            list_works,
            get_thumbnail,
            get_work,
            update_work,
            get_settings,
            set_resource_mode,
            set_delete_file_action,
            set_directory_template,
            set_type_labels,
            validate_template,
            preview_template,
            has_image_subfolders,
            resolve_drop_path,
            parse_folder_name,
            preview_import_path,
            enqueue_import,
            discover_folders,
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
            delete_work,
            check_integrity,
            delete_orphan_works,
            get_theme,
            set_theme,
            get_banner_auto_close,
            set_banner_auto_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
