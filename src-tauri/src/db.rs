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

fn init_db(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(include_str!("../migrations/001_create_initial_tables.sql"))?;
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
}

pub fn insert_work(conn: &Connection, record: &WorkRecord) -> Result<(), AppError> {
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
        "SELECT id, title, path, type, page_count, created_at FROM works WHERE id = ?1",
    )?;
    stmt.query_row([work_id], |row| {
        Ok(WorkDetail {
            id: row.get(0)?,
            title: row.get(1)?,
            path: row.get(2)?,
            work_type: row.get(3)?,
            page_count: row.get(4)?,
            created_at: row.get(5)?,
        })
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
        other => AppError::Database(other),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    fn sample_record<'a>(title: &'a str, path: &'a str) -> WorkRecord<'a> {
        WorkRecord {
            title,
            path,
            work_type: "image",
            page_count: 1,
            thumbnail: b"fake_thumb",
        }
    }

    #[test]
    fn insert_and_list_round_trip() {
        let conn = test_conn();
        insert_work(&conn, &sample_record("Alpha", "/a.jpg")).unwrap();
        insert_work(&conn, &sample_record("Beta", "/b.jpg")).unwrap();

        let works = list_works(&conn, "title", "asc").unwrap();
        assert_eq!(works.len(), 2);
        assert_eq!(works[0].title, "Alpha");
        assert_eq!(works[1].title, "Beta");
    }

    #[test]
    fn path_exists_true_and_false() {
        let conn = test_conn();
        insert_work(&conn, &sample_record("A", "/exists.jpg")).unwrap();

        assert!(path_exists(&conn, "/exists.jpg").unwrap());
        assert!(!path_exists(&conn, "/not_here.jpg").unwrap());
    }

    #[test]
    fn duplicate_path_returns_error() {
        let conn = test_conn();
        insert_work(&conn, &sample_record("A", "/dup.jpg")).unwrap();
        let result = insert_work(&conn, &sample_record("B", "/dup.jpg"));
        assert!(result.is_err());
    }

    #[test]
    fn list_works_sort_by_created_at_desc() {
        let conn = test_conn();
        insert_work(&conn, &sample_record("First", "/1.jpg")).unwrap();
        insert_work(&conn, &sample_record("Second", "/2.jpg")).unwrap();

        let works = list_works(&conn, "created_at", "desc").unwrap();
        assert_eq!(works.len(), 2);
        assert!(works[0].created_at >= works[1].created_at);
    }

    #[test]
    fn list_works_sort_by_title_desc() {
        let conn = test_conn();
        insert_work(&conn, &sample_record("Alpha", "/a.jpg")).unwrap();
        insert_work(&conn, &sample_record("Beta", "/b.jpg")).unwrap();

        let works = list_works(&conn, "title", "desc").unwrap();
        assert_eq!(works[0].title, "Beta");
        assert_eq!(works[1].title, "Alpha");
    }

    #[test]
    fn unknown_sort_by_falls_back_to_created_at() {
        let conn = test_conn();
        insert_work(&conn, &sample_record("A", "/a.jpg")).unwrap();
        let works = list_works(&conn, "invalid_column", "asc").unwrap();
        assert_eq!(works.len(), 1);
    }

    #[test]
    fn get_thumbnail_returns_data() {
        let conn = test_conn();
        insert_work(&conn, &sample_record("A", "/a.jpg")).unwrap();

        let works = list_works(&conn, "title", "asc").unwrap();
        let thumb = get_thumbnail(&conn, works[0].id).unwrap();
        assert_eq!(thumb, b"fake_thumb");
    }

    #[test]
    fn get_thumbnail_not_found() {
        let conn = test_conn();
        let result = get_thumbnail(&conn, 9999);
        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[test]
    fn get_work_returns_detail() {
        let conn = test_conn();
        insert_work(&conn, &sample_record("Title", "/path.jpg")).unwrap();

        let works = list_works(&conn, "title", "asc").unwrap();
        let detail = get_work(&conn, works[0].id).unwrap();
        assert_eq!(detail.title, "Title");
        assert_eq!(detail.path, "/path.jpg");
        assert_eq!(detail.work_type, "image");
    }

    #[test]
    fn get_work_not_found() {
        let conn = test_conn();
        let result = get_work(&conn, 9999);
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
