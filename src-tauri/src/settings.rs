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

pub const DEFAULT_DIRECTORY_TEMPLATE: &str = "{artist}/{title}";

const KEY_DIRECTORY_TEMPLATE: &str = "directory_template";
const KEY_TYPE_LABEL_IMAGE: &str = "type_label_image";
const KEY_TYPE_LABEL_FOLDER: &str = "type_label_folder";
const KEY_RESOURCE_MODE: &str = "resource_mode";
const KEY_DELETE_FILE_ACTION: &str = "delete_file_action";

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

pub fn set_resource_mode(conn: &Connection, library_id: &str, mode: &str) -> Result<(), AppError> {
    set_setting(conn, library_id, KEY_RESOURCE_MODE, mode)
}

pub fn get_delete_file_action(conn: &Connection, library_id: &str) -> Result<String, AppError> {
    Ok(get_setting(conn, library_id, KEY_DELETE_FILE_ACTION)?.unwrap_or_else(|| "ask".into()))
}

pub fn set_delete_file_action(
    conn: &Connection,
    library_id: &str,
    action: &str,
) -> Result<(), AppError> {
    match action {
        "delete" | "trash" | "ask" => set_setting(conn, library_id, KEY_DELETE_FILE_ACTION, action),
        _ => Err(AppError::InvalidSetting(
            "無効な削除時のファイル処理設定です".into(),
        )),
    }
}

pub fn get_app_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare_cached("SELECT value FROM app_settings WHERE key = ?1")?;
    let result = stmt
        .query_row(rusqlite::params![key], |row| row.get(0))
        .optional()
        .map_err(AppError::Database)?;
    Ok(result)
}

pub fn set_app_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

const KEY_THEME_MODE: &str = "theme_mode";
const KEY_BANNER_AUTO_CLOSE: &str = "banner_auto_close";

pub fn get_theme_mode(conn: &Connection) -> Result<String, AppError> {
    Ok(get_app_setting(conn, KEY_THEME_MODE)?.unwrap_or_else(|| "system".into()))
}

pub fn set_theme_mode(conn: &Connection, mode: &str) -> Result<(), AppError> {
    match mode {
        "light" | "dark" | "system" => set_app_setting(conn, KEY_THEME_MODE, mode),
        _ => Err(AppError::InvalidSetting("無効なテーマモードです".into())),
    }
}

pub fn get_banner_auto_close(conn: &Connection) -> Result<u32, AppError> {
    let value = get_app_setting(conn, KEY_BANNER_AUTO_CLOSE)?;
    Ok(value.and_then(|v| v.parse().ok()).unwrap_or(3))
}

pub fn set_banner_auto_close(conn: &Connection, seconds: u32) -> Result<(), AppError> {
    match seconds {
        0 | 1 | 3 | 5 => set_app_setting(conn, KEY_BANNER_AUTO_CLOSE, &seconds.to_string()),
        _ => Err(AppError::InvalidSetting(
            "無効なバナー自動クローズ設定です".into(),
        )),
    }
}

#[cfg(test)]
#[path = "tests/settings.rs"]
mod tests;
