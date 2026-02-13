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

#[cfg(test)]
#[path = "tests/settings.rs"]
mod tests;
