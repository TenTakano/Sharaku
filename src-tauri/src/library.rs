use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
}

fn map_library_row(row: &rusqlite::Row) -> rusqlite::Result<Library> {
    Ok(Library {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get::<_, Option<String>>(2)?,
    })
}

pub fn list_libraries(conn: &Connection) -> Result<Vec<Library>, AppError> {
    let mut stmt = conn.prepare_cached("SELECT id, name, path FROM libraries ORDER BY rowid")?;
    let rows = stmt.query_map([], map_library_row)?;
    let mut libs = Vec::new();
    for row in rows {
        libs.push(row?);
    }
    Ok(libs)
}

pub fn add_library(conn: &Connection, name: &str, path: Option<&str>) -> Result<Library, AppError> {
    if let Some(p) = path {
        let existing: bool = conn
            .prepare_cached("SELECT 1 FROM libraries WHERE path = ?1")?
            .exists([p])?;
        if existing {
            return Err(AppError::LibraryError(
                "同じパスのライブラリが既に存在します".to_string(),
            ));
        }
    }

    let id = generate_id();
    conn.execute(
        "INSERT INTO libraries (id, name, path) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, name, path],
    )?;

    let has_active: bool = conn
        .prepare_cached("SELECT 1 FROM libraries WHERE is_active = 1")?
        .exists([])?;
    if !has_active {
        conn.execute("UPDATE libraries SET is_active = 1 WHERE id = ?1", [&id])?;
    }

    Ok(Library {
        id,
        name: name.to_string(),
        path: path.map(|s| s.to_string()),
    })
}

pub fn remove_library(conn: &Connection, id: &str) -> Result<(), AppError> {
    let was_active: bool = conn
        .prepare_cached("SELECT is_active FROM libraries WHERE id = ?1")?
        .query_row([id], |row| row.get(0))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::LibraryError("指定されたライブラリが見つかりません".to_string())
            }
            other => AppError::Database(other),
        })?;

    conn.execute("DELETE FROM libraries WHERE id = ?1", [id])?;

    if was_active {
        conn.execute_batch(
            "UPDATE libraries SET is_active = 1 WHERE rowid = (SELECT MIN(rowid) FROM libraries)",
        )?;
    }

    Ok(())
}

pub fn find_library_by_id(conn: &Connection, id: &str) -> Result<Option<Library>, AppError> {
    let mut stmt = conn.prepare_cached("SELECT id, name, path FROM libraries WHERE id = ?1")?;
    let result = stmt.query_row([id], map_library_row).optional();
    match result {
        Ok(lib) => Ok(lib),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn active_library(conn: &Connection) -> Result<Option<Library>, AppError> {
    let mut stmt =
        conn.prepare_cached("SELECT id, name, path FROM libraries WHERE is_active = 1 LIMIT 1")?;
    let result = stmt.query_row([], map_library_row).optional();
    match result {
        Ok(lib) => Ok(lib),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn set_active_library(conn: &Connection, id: &str) -> Result<(), AppError> {
    let exists: bool = conn
        .prepare_cached("SELECT 1 FROM libraries WHERE id = ?1")?
        .exists([id])?;
    if !exists {
        return Err(AppError::LibraryError(
            "指定されたライブラリが見つかりません".to_string(),
        ));
    }
    conn.execute("UPDATE libraries SET is_active = 0 WHERE is_active = 1", [])?;
    conn.execute("UPDATE libraries SET is_active = 1 WHERE id = ?1", [id])?;
    Ok(())
}

fn generate_id() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = duration.as_nanos();
    format!("{:016x}", nanos & 0xFFFFFFFFFFFFFFFF)
}

#[cfg(test)]
#[path = "tests/library.rs"]
mod tests;
