use std::collections::HashSet;
use std::path::Path;

use rusqlite::params_from_iter;
use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppError;

pub fn open_db(app_data_dir: &Path) -> Result<Connection, AppError> {
    std::fs::create_dir_all(app_data_dir)?;
    let db_path = app_data_dir.join("sharaku.db");
    let conn = Connection::open(db_path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    init_db(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_db_in_memory() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    init_db(&conn)?;
    Ok(conn)
}

fn init_db(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(include_str!("../migrations/004_unified_local_db.sql"))?;
    migrate_libraries_path_nullable(conn)?;
    Ok(())
}

fn migrate_libraries_path_nullable(conn: &Connection) -> Result<(), AppError> {
    let notnull: bool = conn
        .prepare("PRAGMA table_info(libraries)")?
        .query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, bool>(3)?))
        })?
        .filter_map(|r| r.ok())
        .any(|(name, notnull)| name == "path" && notnull);

    if !notnull {
        return Ok(());
    }

    conn.execute_batch(
        "PRAGMA foreign_keys=OFF;
         BEGIN;
         CREATE TABLE libraries_new (
             id         TEXT PRIMARY KEY,
             name       TEXT NOT NULL,
             path       TEXT UNIQUE,
             is_active  INTEGER NOT NULL DEFAULT 0,
             created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         );
         INSERT INTO libraries_new SELECT * FROM libraries;
         DROP TABLE libraries;
         ALTER TABLE libraries_new RENAME TO libraries;
         COMMIT;
         PRAGMA foreign_keys=ON;",
    )?;
    Ok(())
}

pub fn path_exists(conn: &Connection, library_id: &str, path: &str) -> Result<bool, AppError> {
    let mut stmt =
        conn.prepare_cached("SELECT 1 FROM works WHERE library_id = ?1 AND path = ?2")?;
    Ok(stmt.exists(rusqlite::params![library_id, path])?)
}

pub struct WorkRecord<'a> {
    pub library_id: &'a str,
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
        "INSERT INTO works (library_id, title, path, type, page_count, thumbnail, artist, year, genre, circle, origin) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            record.library_id,
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

fn map_work_summary_row(row: &rusqlite::Row) -> rusqlite::Result<WorkSummary> {
    Ok(WorkSummary {
        id: row.get(0)?,
        title: row.get(1)?,
        work_type: row.get(2)?,
        page_count: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn map_work_detail_row(row: &rusqlite::Row) -> rusqlite::Result<WorkDetail> {
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
}

fn map_tag_row(row: &rusqlite::Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(2)?,
    })
}

pub fn list_works(
    conn: &Connection,
    library_id: &str,
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
        "SELECT id, title, type, page_count, created_at FROM works WHERE library_id = ?1 ORDER BY {} {}",
        column, order
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([library_id], map_work_summary_row)?;
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

#[allow(clippy::too_many_arguments)]
pub fn update_work(
    conn: &Connection,
    id: i64,
    title: &str,
    artist: Option<&str>,
    year: Option<i32>,
    genre: Option<&str>,
    circle: Option<&str>,
    origin: Option<&str>,
) -> Result<(), AppError> {
    let rows = conn.execute(
        "UPDATE works SET title = ?1, artist = ?2, year = ?3, genre = ?4, circle = ?5, origin = ?6, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?7",
        rusqlite::params![title, artist, year, genre, circle, origin, id],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn get_work(conn: &Connection, work_id: i64) -> Result<WorkDetail, AppError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, title, path, type, page_count, created_at, artist, year, genre, circle, origin FROM works WHERE id = ?1",
    )?;
    stmt.query_row([work_id], map_work_detail_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Database(other),
        })
}

pub fn create_tag(
    conn: &Connection,
    library_id: &str,
    name: &str,
    category: Option<&str>,
) -> Result<Tag, AppError> {
    conn.execute(
        "INSERT INTO tags (library_id, name, category) VALUES (?1, ?2, ?3)",
        rusqlite::params![library_id, name, category],
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
    library_id: &str,
    query: &str,
    category: Option<&str>,
) -> Result<Vec<Tag>, AppError> {
    let pattern = format!("%{query}%");
    let mut stmt = conn.prepare(
        "SELECT id, name, category FROM tags WHERE library_id = ?1 AND name LIKE ?2 AND (?3 IS NULL OR category = ?3) ORDER BY name LIMIT 50",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![library_id, pattern, category],
        map_tag_row,
    )?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn list_tags(conn: &Connection, library_id: &str) -> Result<Vec<Tag>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, category FROM tags WHERE library_id = ?1 ORDER BY category, name",
    )?;
    let rows = stmt.query_map([library_id], map_tag_row)?;
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
    let rows = stmt.query_map([work_id], map_tag_row)?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn search_works_by_tags(
    conn: &Connection,
    library_id: &str,
    tag_ids: &[i64],
    mode: &str,
) -> Result<Vec<WorkSummary>, AppError> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut unique_ids: Vec<i64> = tag_ids.to_vec();
    unique_ids.sort_unstable();
    unique_ids.dedup();

    // Parameter index: ?1 = library_id, then tag IDs start at ?2
    let placeholders: Vec<String> = (2..=unique_ids.len() + 1)
        .map(|i| format!("?{i}"))
        .collect();
    let in_clause = placeholders.join(", ");

    let sql = if mode == "and" {
        format!(
            "SELECT w.id, w.title, w.type, w.page_count, w.created_at \
             FROM works w \
             INNER JOIN works_tags wt ON w.id = wt.work_id \
             WHERE w.library_id = ?1 AND wt.tag_id IN ({in_clause}) \
             GROUP BY w.id \
             HAVING COUNT(DISTINCT wt.tag_id) = ?{} \
             ORDER BY w.created_at DESC",
            unique_ids.len() + 2
        )
    } else {
        format!(
            "SELECT DISTINCT w.id, w.title, w.type, w.page_count, w.created_at \
             FROM works w \
             INNER JOIN works_tags wt ON w.id = wt.work_id \
             WHERE w.library_id = ?1 AND wt.tag_id IN ({in_clause}) \
             ORDER BY w.created_at DESC"
        )
    };

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
        vec![Box::new(library_id.to_string()) as _];
    for id in &unique_ids {
        params.push(Box::new(*id) as _);
    }
    if mode == "and" {
        params.push(Box::new(unique_ids.len() as i64));
    }

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params.iter()), map_work_summary_row)?;
    let mut works = Vec::new();
    for row in rows {
        works.push(row?);
    }
    Ok(works)
}

pub fn delete_works_by_ids(
    conn: &Connection,
    library_id: &str,
    ids: &[i64],
) -> Result<usize, AppError> {
    if ids.is_empty() {
        return Ok(0);
    }
    let mut total_deleted = 0usize;
    for chunk in ids.chunks(500) {
        let placeholders: Vec<String> = (2..=chunk.len() + 1).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "DELETE FROM works WHERE library_id = ?1 AND id IN ({})",
            placeholders.join(", ")
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(library_id.to_string()) as _];
        for id in chunk {
            params.push(Box::new(*id) as _);
        }
        let deleted = conn.execute(&sql, params_from_iter(params.iter()))?;
        total_deleted += deleted;
    }
    Ok(total_deleted)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub work_id: i64,
    pub title: String,
    pub work_type: String,
    pub page_count: i32,
    pub created_at: String,
}

fn map_playlist_row(row: &rusqlite::Row) -> rusqlite::Result<Playlist> {
    Ok(Playlist {
        id: row.get(0)?,
        name: row.get(1)?,
    })
}

fn map_playlist_item_row(row: &rusqlite::Row) -> rusqlite::Result<PlaylistItem> {
    Ok(PlaylistItem {
        work_id: row.get(0)?,
        title: row.get(1)?,
        work_type: row.get(2)?,
        page_count: row.get(3)?,
        created_at: row.get(4)?,
    })
}

pub fn list_playlists(conn: &Connection, library_id: &str) -> Result<Vec<Playlist>, AppError> {
    let mut stmt =
        conn.prepare("SELECT id, name FROM playlists WHERE library_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([library_id], map_playlist_row)?;
    let mut playlists = Vec::new();
    for row in rows {
        playlists.push(row?);
    }
    Ok(playlists)
}

pub fn create_playlist(
    conn: &Connection,
    library_id: &str,
    name: &str,
) -> Result<Playlist, AppError> {
    conn.execute(
        "INSERT INTO playlists (library_id, name) VALUES (?1, ?2)",
        rusqlite::params![library_id, name],
    )?;
    Ok(Playlist {
        id: conn.last_insert_rowid(),
        name: name.to_string(),
    })
}

pub fn rename_playlist(conn: &Connection, id: i64, name: &str) -> Result<(), AppError> {
    let rows = conn.execute(
        "UPDATE playlists SET name = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        rusqlite::params![name, id],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete_playlist(conn: &Connection, id: i64) -> Result<(), AppError> {
    let rows = conn.execute("DELETE FROM playlists WHERE id = ?1", [id])?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn get_playlist_items(
    conn: &Connection,
    playlist_id: i64,
) -> Result<Vec<PlaylistItem>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT w.id, w.title, w.type, w.page_count, w.created_at \
         FROM playlist_items pi \
         INNER JOIN works w ON w.id = pi.work_id \
         WHERE pi.playlist_id = ?1 \
         ORDER BY pi.position",
    )?;
    let rows = stmt.query_map([playlist_id], map_playlist_item_row)?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

pub fn add_item_to_playlist(
    conn: &Connection,
    playlist_id: i64,
    work_id: i64,
) -> Result<(), AppError> {
    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_items WHERE playlist_id = ?1",
        [playlist_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO playlist_items (playlist_id, work_id, position) VALUES (?1, ?2, ?3)",
        rusqlite::params![playlist_id, work_id, next_position],
    )?;
    Ok(())
}

pub fn remove_item_from_playlist(
    conn: &Connection,
    playlist_id: i64,
    work_id: i64,
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM playlist_items WHERE playlist_id = ?1 AND work_id = ?2",
        rusqlite::params![playlist_id, work_id],
    )?;
    Ok(())
}

/// Reassigns positions for every item of `playlist_id` per the order of `work_ids`.
/// `work_ids` must be exactly the current item set (no missing/extra/duplicate ids),
/// otherwise the call is rejected before any write happens.
///
/// `UNIQUE(playlist_id, position)` is not DEFERRABLE in this schema, so a direct
/// bulk UPDATE to the final positions can collide mid-transaction. Existing rows are
/// first staged to negative offsets (guaranteed disjoint from any valid position),
/// then reassigned to their final 0..n-1 positions.
pub fn reorder_playlist_items(
    conn: &mut Connection,
    playlist_id: i64,
    work_ids: &[i64],
) -> Result<(), AppError> {
    let given: HashSet<i64> = work_ids.iter().copied().collect();
    if given.len() != work_ids.len() {
        return Err(AppError::PlaylistError(
            "reorder work_ids contains duplicates".to_string(),
        ));
    }

    let mut existing_stmt =
        conn.prepare("SELECT work_id FROM playlist_items WHERE playlist_id = ?1")?;
    let existing: HashSet<i64> = existing_stmt
        .query_map([playlist_id], |row| row.get(0))?
        .collect::<rusqlite::Result<HashSet<i64>>>()?;
    drop(existing_stmt);

    if existing != given {
        return Err(AppError::PlaylistError(
            "reorder work_ids does not match the playlist's current items".to_string(),
        ));
    }

    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE playlist_items SET position = -(position + 1) WHERE playlist_id = ?1",
        [playlist_id],
    )?;
    for (index, work_id) in work_ids.iter().enumerate() {
        tx.execute(
            "UPDATE playlist_items SET position = ?1 WHERE playlist_id = ?2 AND work_id = ?3",
            rusqlite::params![index as i64, playlist_id, work_id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/db.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/playlist.rs"]
mod playlist_tests;
