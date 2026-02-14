use rusqlite::{Connection, OptionalExtension};

use crate::error::AppError;

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare_cached("SELECT value FROM settings WHERE key = ?1")?;
    let result = stmt
        .query_row([key], |row| row.get(0))
        .optional()
        .map_err(AppError::Database)?;
    Ok(result)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

const KEY_LIBRARY_ROOT: &str = "library_root";
const KEY_DIRECTORY_TEMPLATE: &str = "directory_template";
const KEY_TYPE_LABEL_IMAGE: &str = "type_label_image";
const KEY_TYPE_LABEL_FOLDER: &str = "type_label_folder";

const DEFAULT_TYPE_LABEL_IMAGE: &str = "Image";
const DEFAULT_TYPE_LABEL_FOLDER: &str = "Folder";

pub fn get_library_root(conn: &Connection) -> Result<Option<String>, AppError> {
    get_setting(conn, KEY_LIBRARY_ROOT)
}

pub fn set_library_root(conn: &Connection, path: &str) -> Result<(), AppError> {
    set_setting(conn, KEY_LIBRARY_ROOT, path)
}

pub fn get_directory_template(conn: &Connection) -> Result<Option<String>, AppError> {
    get_setting(conn, KEY_DIRECTORY_TEMPLATE)
}

pub fn set_directory_template(conn: &Connection, template: &str) -> Result<(), AppError> {
    set_setting(conn, KEY_DIRECTORY_TEMPLATE, template)
}

pub fn get_type_label_image(conn: &Connection) -> Result<String, AppError> {
    Ok(get_setting(conn, KEY_TYPE_LABEL_IMAGE)?.unwrap_or_else(|| DEFAULT_TYPE_LABEL_IMAGE.into()))
}

pub fn set_type_label_image(conn: &Connection, label: &str) -> Result<(), AppError> {
    set_setting(conn, KEY_TYPE_LABEL_IMAGE, label)
}

pub fn get_type_label_folder(conn: &Connection) -> Result<String, AppError> {
    Ok(get_setting(conn, KEY_TYPE_LABEL_FOLDER)?.unwrap_or_else(|| DEFAULT_TYPE_LABEL_FOLDER.into()))
}

pub fn set_type_label_folder(conn: &Connection, label: &str) -> Result<(), AppError> {
    set_setting(conn, KEY_TYPE_LABEL_FOLDER, label)
}

pub fn resolve_type_label(conn: &Connection, work_type: &str) -> Result<String, AppError> {
    match work_type {
        "image" => get_type_label_image(conn),
        "folder" => get_type_label_folder(conn),
        _ => Ok(work_type.to_string()),
    }
}

#[cfg(test)]
#[path = "tests/settings.rs"]
mod tests;
