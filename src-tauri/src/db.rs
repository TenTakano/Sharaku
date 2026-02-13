use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppError;

pub fn open_db(app_data_dir: &Path) -> Result<Connection, AppError> {
    std::fs::create_dir_all(app_data_dir)?;
    let db_path = app_data_dir.join("sharaku.db");
    let conn = Connection::open(db_path)?;
    init_db(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn init_db_for_test(conn: &Connection) -> Result<(), AppError> {
    init_db(conn)
}

fn init_db(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(include_str!("../migrations/001_create_initial_tables.sql"))?;
    apply_migration_002(conn)?;
    Ok(())
}

fn apply_migration_002(conn: &Connection) -> Result<(), AppError> {
    let columns = ["artist", "year", "genre", "circle", "origin"];
    for col in columns {
        let sql = match col {
            "year" => format!("ALTER TABLE works ADD COLUMN {col} INTEGER"),
            _ => format!("ALTER TABLE works ADD COLUMN {col} TEXT"),
        };
        match conn.execute_batch(&sql) {
            Ok(()) => {}
            Err(e) if e.to_string().contains("duplicate column name") => {}
            Err(e) => return Err(AppError::Database(e)),
        }
    }
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    )?;
    Ok(())
}

pub fn path_exists(conn: &Connection, path: &str) -> Result<bool, AppError> {
    let mut stmt = conn.prepare_cached("SELECT 1 FROM works WHERE path = ?1")?;
    Ok(stmt.exists([path])?)
}

pub struct WorkRecord<'a> {
    pub title: &'a str,
    pub path: &'a str,
    pub work_type: &'a str,
    pub page_count: i32,
    pub thumbnail: &'a [u8],
    pub artist: Option<&'a str>,
    pub year: Option<i32>,
    pub genre: Option<&'a str>,
    pub circle: Option<&'a str>,
    pub origin: Option<&'a str>,
}

pub fn insert_work(conn: &Connection, record: &WorkRecord) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail, artist, year, genre, circle, origin) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            record.title,
            record.path,
            record.work_type,
            record.page_count,
            record.thumbnail,
            record.artist,
            record.year,
            record.genre,
            record.circle,
            record.origin,
        ],
    )?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSummary {
    pub id: i64,
    pub title: String,
    pub work_type: String,
    pub page_count: i32,
    pub created_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkDetail {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub work_type: String,
    pub page_count: i32,
    pub created_at: String,
    pub artist: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub circle: Option<String>,
    pub origin: Option<String>,
}

pub fn list_works(
    conn: &Connection,
    sort_by: &str,
    sort_order: &str,
) -> Result<Vec<WorkSummary>, AppError> {
    let column = match sort_by {
        "title" => "title",
        _ => "created_at",
    };
    let order = match sort_order {
        "asc" => "ASC",
        _ => "DESC",
    };
    let sql = format!(
        "SELECT id, title, type, page_count, created_at FROM works ORDER BY {} {}",
        column, order
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(WorkSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            work_type: row.get(2)?,
            page_count: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    let mut works = Vec::new();
    for row in rows {
        works.push(row?);
    }
    Ok(works)
}

pub fn get_thumbnail(conn: &Connection, work_id: i64) -> Result<Vec<u8>, AppError> {
    let mut stmt = conn.prepare_cached("SELECT thumbnail FROM works WHERE id = ?1")?;
    let thumb: Option<Vec<u8>> =
        stmt.query_row([work_id], |row| row.get(0))
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                other => AppError::Database(other),
            })?;
    thumb.ok_or(AppError::NotFound)
}

pub fn get_work(conn: &Connection, work_id: i64) -> Result<WorkDetail, AppError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, title, path, type, page_count, created_at, artist, year, genre, circle, origin FROM works WHERE id = ?1",
    )?;
    stmt.query_row([work_id], |row| {
        Ok(WorkDetail {
            id: row.get(0)?,
            title: row.get(1)?,
            path: row.get(2)?,
            work_type: row.get(3)?,
            page_count: row.get(4)?,
            created_at: row.get(5)?,
            artist: row.get(6)?,
            year: row.get(7)?,
            genre: row.get(8)?,
            circle: row.get(9)?,
            origin: row.get(10)?,
        })
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
        other => AppError::Database(other),
    })
}

#[cfg(test)]
#[path = "tests/db.rs"]
mod tests;
