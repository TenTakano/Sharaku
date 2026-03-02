use rusqlite::{Connection, OptionalExtension};

use crate::error::AppError;

pub fn get_setting(
    conn: &Connection,
    library_id: &str,
    key: &str,
) -> Result<Option<String>, AppError> {
    let mut stmt =
        conn.prepare_cached("SELECT value FROM settings WHERE library_id = ?1 AND key = ?2")?;
    let result = stmt
        .query_row(rusqlite::params![library_id, key], |row| row.get(0))
        .optional()
        .map_err(AppError::Database)?;
    Ok(result)
}

pub fn set_setting(
    conn: &Connection,
    library_id: &str,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (library_id, key, value) VALUES (?1, ?2, ?3) ON CONFLICT(library_id, key) DO UPDATE SET value = excluded.value",
        rusqlite::params![library_id, key, value],
    )?;
    Ok(())
}

const KEY_DIRECTORY_TEMPLATE: &str = "directory_template";
const KEY_TYPE_LABEL_IMAGE: &str = "type_label_image";
const KEY_TYPE_LABEL_FOLDER: &str = "type_label_folder";
const KEY_RESOURCE_MODE: &str = "resource_mode";

const DEFAULT_TYPE_LABEL_IMAGE: &str = "Image";
const DEFAULT_TYPE_LABEL_FOLDER: &str = "Folder";

pub fn get_directory_template(
    conn: &Connection,
    library_id: &str,
) -> Result<Option<String>, AppError> {
    get_setting(conn, library_id, KEY_DIRECTORY_TEMPLATE)
}

pub fn set_directory_template(
    conn: &Connection,
    library_id: &str,
    template: &str,
) -> Result<(), AppError> {
    set_setting(conn, library_id, KEY_DIRECTORY_TEMPLATE, template)
}

pub fn get_type_label_image(conn: &Connection, library_id: &str) -> Result<String, AppError> {
    Ok(get_setting(conn, library_id, KEY_TYPE_LABEL_IMAGE)?
        .unwrap_or_else(|| DEFAULT_TYPE_LABEL_IMAGE.into()))
}

pub fn set_type_label_image(
    conn: &Connection,
    library_id: &str,
    label: &str,
) -> Result<(), AppError> {
    set_setting(conn, library_id, KEY_TYPE_LABEL_IMAGE, label)
}

pub fn get_type_label_folder(conn: &Connection, library_id: &str) -> Result<String, AppError> {
    Ok(get_setting(conn, library_id, KEY_TYPE_LABEL_FOLDER)?
        .unwrap_or_else(|| DEFAULT_TYPE_LABEL_FOLDER.into()))
}

pub fn set_type_label_folder(
    conn: &Connection,
    library_id: &str,
    label: &str,
) -> Result<(), AppError> {
    set_setting(conn, library_id, KEY_TYPE_LABEL_FOLDER, label)
}

pub fn get_resource_mode(conn: &Connection, library_id: &str) -> Result<String, AppError> {
    Ok(get_setting(conn, library_id, KEY_RESOURCE_MODE)?.unwrap_or_else(|| "full".into()))
}

pub fn set_resource_mode(
    conn: &Connection,
    library_id: &str,
    mode: &str,
) -> Result<(), AppError> {
    set_setting(conn, library_id, KEY_RESOURCE_MODE, mode)
}

pub fn resolve_type_label(
    conn: &Connection,
    library_id: &str,
    work_type: &str,
) -> Result<String, AppError> {
    match work_type {
        "image" => get_type_label_image(conn, library_id),
        "folder" => get_type_label_folder(conn, library_id),
        _ => Ok(work_type.to_string()),
    }
}

#[cfg(test)]
#[path = "tests/settings.rs"]
mod tests;
