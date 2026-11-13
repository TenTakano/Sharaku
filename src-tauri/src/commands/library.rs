use std::path::PathBuf;

use crate::library::{self, Library};
use crate::{ActiveLibrary, AppState};

#[tauri::command]
pub(crate) async fn list_libraries(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Library>, String> {
    state
        .with_db(|db| library::list_libraries(&db.conn).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn get_active_library(
    state: tauri::State<'_, AppState>,
) -> Result<Option<Library>, String> {
    state
        .with_db(|db| library::active_library(&db.conn).map_err(|e| e.to_string()))
        .await
}

#[tauri::command]
pub(crate) async fn create_library(
    state: tauri::State<'_, AppState>,
    name: String,
    path: Option<String>,
) -> Result<Library, String> {
    state
        .with_db_mut(move |db| {
            let lib = library::add_library(&db.conn, &name, path.as_deref())
                .map_err(|e| e.to_string())?;
            library::set_active_library(&db.conn, &lib.id).map_err(|e| e.to_string())?;
            db.active_library = Some(ActiveLibrary {
                id: lib.id.clone(),
                path: lib.path.as_ref().map(PathBuf::from),
            });
            Ok(lib)
        })
        .await
}

#[tauri::command]
pub(crate) async fn switch_library(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
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
pub(crate) async fn remove_library(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
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
