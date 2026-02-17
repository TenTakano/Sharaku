use std::path::Path;

use rusqlite::params_from_iter;
use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppError;

pub fn open_db(library_root: &Path) -> Result<Connection, AppError> {
    std::fs::create_dir_all(library_root)?;
    let db_path = library_root.join("sharaku.db");
    let conn = Connection::open(db_path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    init_db(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn init_db_for_test(conn: &Connection) -> Result<(), AppError> {
    init_db(conn)
}

fn init_db(conn: &Connection) -> Result<(), AppError> {
    // CIFS/SMB上ではWALモードの.db-shm(mmap)が動作しないためDELETEモードを採用。
    // DB接続はArc<Mutex>で一元管理しておりWALの同時読み書き性能は不要。
    conn.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(include_str!("../migrations/001_create_initial_tables.sql"))?;
    apply_migration_002(conn)?;
    apply_migration_003(conn)?;
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

fn apply_migration_003(conn: &Connection) -> Result<(), AppError> {
    let needs_migration = match conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name='works'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(sql) => !sql.contains("folder"),
        Err(e) => return Err(AppError::Database(e)),
    };

    if needs_migration {
        conn.execute_batch(include_str!("../migrations/003_allow_folder_work_type.sql"))?;
    }
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub category: Option<String>,
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

pub fn list_folder_works(conn: &Connection) -> Result<Vec<WorkDetail>, AppError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, title, path, type, page_count, created_at, artist, year, genre, circle, origin FROM works WHERE type = 'folder'",
    )?;
    let rows = stmt.query_map([], |row| {
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
    })?;
    let mut works = Vec::new();
    for row in rows {
        works.push(row?);
    }
    Ok(works)
}

pub fn update_work_path(conn: &Connection, work_id: i64, new_path: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE works SET path = ?1 WHERE id = ?2",
        rusqlite::params![new_path, work_id],
    )?;
    Ok(())
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

pub fn create_tag(conn: &Connection, name: &str, category: Option<&str>) -> Result<Tag, AppError> {
    conn.execute(
        "INSERT INTO tags (name, category) VALUES (?1, ?2)",
        rusqlite::params![name, category],
    )?;
    Ok(Tag {
        id: conn.last_insert_rowid(),
        name: name.to_string(),
        category: category.map(|s| s.to_string()),
    })
}

pub fn update_tag(
    conn: &Connection,
    id: i64,
    name: &str,
    category: Option<&str>,
) -> Result<(), AppError> {
    let rows = conn.execute(
        "UPDATE tags SET name = ?1, category = ?2 WHERE id = ?3",
        rusqlite::params![name, category, id],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete_tag(conn: &Connection, id: i64) -> Result<(), AppError> {
    let rows = conn.execute("DELETE FROM tags WHERE id = ?1", [id])?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn search_tags(
    conn: &Connection,
    query: &str,
    category: Option<&str>,
) -> Result<Vec<Tag>, AppError> {
    let pattern = format!("%{query}%");
    let mut stmt = conn.prepare(
        "SELECT id, name, category FROM tags WHERE name LIKE ?1 AND (?2 IS NULL OR category = ?2) ORDER BY name LIMIT 50",
    )?;
    let rows = stmt.query_map(rusqlite::params![pattern, category], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
        })
    })?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn add_tag_to_work(conn: &Connection, work_id: i64, tag_id: i64) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR IGNORE INTO works_tags (work_id, tag_id) VALUES (?1, ?2)",
        rusqlite::params![work_id, tag_id],
    )?;
    Ok(())
}

pub fn remove_tag_from_work(conn: &Connection, work_id: i64, tag_id: i64) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM works_tags WHERE work_id = ?1 AND tag_id = ?2",
        rusqlite::params![work_id, tag_id],
    )?;
    Ok(())
}

pub fn get_tags_for_work(conn: &Connection, work_id: i64) -> Result<Vec<Tag>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.category FROM tags t INNER JOIN works_tags wt ON t.id = wt.tag_id WHERE wt.work_id = ?1 ORDER BY t.name",
    )?;
    let rows = stmt.query_map([work_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
        })
    })?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn search_works_by_tags(
    conn: &Connection,
    tag_ids: &[i64],
    mode: &str,
) -> Result<Vec<WorkSummary>, AppError> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut unique_ids: Vec<i64> = tag_ids.to_vec();
    unique_ids.sort_unstable();
    unique_ids.dedup();

    let placeholders: Vec<String> = (1..=unique_ids.len()).map(|i| format!("?{i}")).collect();
    let in_clause = placeholders.join(", ");

    let sql = if mode == "and" {
        format!(
            "SELECT w.id, w.title, w.type, w.page_count, w.created_at \
             FROM works w \
             INNER JOIN works_tags wt ON w.id = wt.work_id \
             WHERE wt.tag_id IN ({in_clause}) \
             GROUP BY w.id \
             HAVING COUNT(DISTINCT wt.tag_id) = ?{} \
             ORDER BY w.created_at DESC",
            unique_ids.len() + 1
        )
    } else {
        format!(
            "SELECT DISTINCT w.id, w.title, w.type, w.page_count, w.created_at \
             FROM works w \
             INNER JOIN works_tags wt ON w.id = wt.work_id \
             WHERE wt.tag_id IN ({in_clause}) \
             ORDER BY w.created_at DESC"
        )
    };

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
        unique_ids.iter().map(|id| Box::new(*id) as _).collect();
    if mode == "and" {
        params.push(Box::new(unique_ids.len() as i64));
    }

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params.iter()), |row| {
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

#[cfg(test)]
#[path = "tests/db.rs"]
mod tests;
