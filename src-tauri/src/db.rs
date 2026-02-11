use std::path::Path;

use rusqlite::Connection;

use crate::error::ScanError;

pub fn open_db(app_data_dir: &Path) -> Result<Connection, ScanError> {
    std::fs::create_dir_all(app_data_dir)?;
    let db_path = app_data_dir.join("sharaku.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(include_str!("../migrations/001_create_initial_tables.sql"))?;
    Ok(conn)
}

pub fn path_exists(conn: &Connection, path: &str) -> Result<bool, ScanError> {
    let mut stmt = conn.prepare_cached("SELECT 1 FROM works WHERE path = ?1")?;
    Ok(stmt.exists([path])?)
}

pub struct WorkRecord<'a> {
    pub title: &'a str,
    pub path: &'a str,
    pub work_type: &'a str,
    pub page_count: i32,
    pub thumbnail: &'a [u8],
}

pub fn insert_work(conn: &Connection, record: &WorkRecord) -> Result<(), ScanError> {
    conn.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            record.title,
            record.path,
            record.work_type,
            record.page_count,
            record.thumbnail,
        ],
    )?;
    Ok(())
}
