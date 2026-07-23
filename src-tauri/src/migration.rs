use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use crate::library;
use crate::settings;

pub struct MigrationError {
    pub library_id: String,
    pub library_path: String,
    pub message: String,
}

const MIGRATED_FLAG_KEY: &str = "_per_library_db_migrated";

pub(crate) fn migrate_libraries_json(conn: &Connection, app_data_dir: &Path) {
    let json_path = app_data_dir.join("libraries.json");
    if !json_path.exists() {
        return;
    }

    #[derive(serde::Deserialize)]
    struct LibraryStoreData {
        libraries: Vec<JsonLibrary>,
        active_library_id: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct JsonLibrary {
        id: String,
        name: String,
        path: String,
    }

    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let data: LibraryStoreData = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => return,
    };

    for lib in &data.libraries {
        let already_exists: bool = conn
            .prepare_cached("SELECT 1 FROM libraries WHERE id = ?1")
            .and_then(|mut stmt| stmt.exists([&lib.id]))
            .unwrap_or(true);
        if already_exists {
            continue;
        }
        let is_active = data.active_library_id.as_deref() == Some(&lib.id);
        let _ = conn.execute(
            "INSERT INTO libraries (id, name, path, is_active) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![lib.id, lib.name, lib.path, is_active as i32],
        );
    }

    let migrated_path = json_path.with_extension("json.migrated");
    let _ = std::fs::rename(&json_path, &migrated_path);
}

pub fn migrate_per_library_dbs(conn: &Connection) -> Vec<MigrationError> {
    let libraries = match library::list_libraries(conn) {
        Ok(libs) => libs,
        Err(e) => {
            return vec![MigrationError {
                library_id: String::new(),
                library_path: String::new(),
                message: format!("ライブラリ一覧取得に失敗: {e}"),
            }];
        }
    };

    let mut errors = Vec::new();
    for lib in &libraries {
        let lib_path = lib.path.as_deref().unwrap_or("");
        if lib_path.is_empty() {
            continue;
        }
        if let Err(msg) = migrate_single_library(conn, &lib.id, lib_path) {
            errors.push(MigrationError {
                library_id: lib.id.clone(),
                library_path: lib_path.to_string(),
                message: msg,
            });
        }
    }
    errors
}

fn migrate_single_library(
    conn: &Connection,
    library_id: &str,
    library_path: &str,
) -> Result<(), String> {
    let already_migrated = settings::get_setting(conn, library_id, MIGRATED_FLAG_KEY)
        .map_err(|e| format!("フラグ確認失敗: {e}"))?;
    if already_migrated.is_some() {
        return Ok(());
    }

    let old_db_path = Path::new(library_path).join("sharaku.db");
    if !old_db_path.exists() {
        settings::set_setting(conn, library_id, MIGRATED_FLAG_KEY, "no_old_db")
            .map_err(|e| format!("フラグ設定失敗: {e}"))?;
        return Ok(());
    }

    let old_conn = Connection::open_with_flags(&old_db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("旧DB開けず: {e}"))?;

    if !is_old_schema(&old_conn).map_err(|e| format!("スキーマ検証失敗: {e}"))? {
        settings::set_setting(conn, library_id, MIGRATED_FLAG_KEY, "new_schema")
            .map_err(|e| format!("フラグ設定失敗: {e}"))?;
        return Ok(());
    }

    conn.execute_batch("BEGIN")
        .map_err(|e| format!("トランザクション開始失敗: {e}"))?;

    let result = migrate_all_tables(conn, &old_conn, library_id).and_then(|()| {
        settings::set_setting(conn, library_id, MIGRATED_FLAG_KEY, "done")
            .map_err(|e| format!("フラグ設定失敗: {e}"))
    });

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")
                .map_err(|e| format!("コミット失敗: {e}"))?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

fn is_old_schema(old_conn: &Connection) -> Result<bool, rusqlite::Error> {
    let has_works: bool = old_conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='works'")?
        .exists([])?;
    if !has_works {
        return Ok(false);
    }

    let has_library_id: bool = old_conn
        .prepare("SELECT 1 FROM pragma_table_info('works') WHERE name='library_id'")?
        .exists([])?;
    Ok(!has_library_id)
}

fn migrate_all_tables(
    conn: &Connection,
    old_conn: &Connection,
    library_id: &str,
) -> Result<(), String> {
    let work_id_map = migrate_works(conn, old_conn, library_id)?;
    let tag_id_map = migrate_tags(conn, old_conn, library_id)?;
    migrate_works_tags(conn, old_conn, &work_id_map, &tag_id_map)?;
    migrate_settings(conn, old_conn, library_id)?;
    let playlist_id_map = migrate_playlists(conn, old_conn, library_id)?;
    migrate_playlist_items(conn, old_conn, &playlist_id_map, &work_id_map)?;
    Ok(())
}

fn migrate_works(
    conn: &Connection,
    old_conn: &Connection,
    library_id: &str,
) -> Result<HashMap<i64, i64>, String> {
    let mut id_map = HashMap::new();

    let mut stmt = old_conn
        .prepare(
            "SELECT id, title, path, type, page_count, thumbnail, created_at, updated_at, \
             artist, year, genre, circle, origin FROM works",
        )
        .map_err(|e| format!("旧works読み取り失敗: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i32>>(4)?,
                row.get::<_, Option<Vec<u8>>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<i32>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<String>>(12)?,
            ))
        })
        .map_err(|e| format!("旧works読み取り失敗: {e}"))?;

    for row in rows {
        let (
            old_id,
            title,
            path,
            work_type,
            page_count,
            thumbnail,
            created_at,
            updated_at,
            artist,
            year,
            genre,
            circle,
            origin,
        ) = row.map_err(|e| format!("旧worksレコード読み取り失敗: {e}"))?;

        let existing_id: Option<i64> = conn
            .prepare_cached("SELECT id FROM works WHERE library_id = ?1 AND path = ?2")
            .and_then(|mut s| s.query_row(rusqlite::params![library_id, path], |r| r.get(0)))
            .ok();

        if let Some(new_id) = existing_id {
            id_map.insert(old_id, new_id);
        } else {
            conn.execute(
                "INSERT INTO works (library_id, title, path, type, page_count, thumbnail, created_at, updated_at, artist, year, genre, circle, origin) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    library_id, title, path, work_type, page_count, thumbnail, created_at,
                    updated_at, artist, year, genre, circle, origin
                ],
            )
            .map_err(|e| format!("works挿入失敗: {e}"))?;
            id_map.insert(old_id, conn.last_insert_rowid());
        }
    }

    Ok(id_map)
}

fn migrate_tags(
    conn: &Connection,
    old_conn: &Connection,
    library_id: &str,
) -> Result<HashMap<i64, i64>, String> {
    let mut id_map = HashMap::new();

    let mut stmt = old_conn
        .prepare("SELECT id, name, category FROM tags")
        .map_err(|e| format!("旧tags読み取り失敗: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(|e| format!("旧tags読み取り失敗: {e}"))?;

    for row in rows {
        let (old_id, name, category) =
            row.map_err(|e| format!("旧tagsレコード読み取り失敗: {e}"))?;

        let existing_id: Option<i64> = conn
            .prepare_cached(
                "SELECT id FROM tags WHERE library_id = ?1 AND name = ?2 AND category IS ?3",
            )
            .and_then(|mut s| {
                s.query_row(rusqlite::params![library_id, name, category], |r| r.get(0))
            })
            .ok();

        if let Some(new_id) = existing_id {
            id_map.insert(old_id, new_id);
        } else {
            conn.execute(
                "INSERT INTO tags (library_id, name, category) VALUES (?1, ?2, ?3)",
                rusqlite::params![library_id, name, category],
            )
            .map_err(|e| format!("tags挿入失敗: {e}"))?;
            id_map.insert(old_id, conn.last_insert_rowid());
        }
    }

    Ok(id_map)
}

fn migrate_works_tags(
    conn: &Connection,
    old_conn: &Connection,
    work_id_map: &HashMap<i64, i64>,
    tag_id_map: &HashMap<i64, i64>,
) -> Result<(), String> {
    let mut stmt = old_conn
        .prepare("SELECT work_id, tag_id FROM works_tags")
        .map_err(|e| format!("旧works_tags読み取り失敗: {e}"))?;

    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| format!("旧works_tags読み取り失敗: {e}"))?;

    for row in rows {
        let (old_work_id, old_tag_id) =
            row.map_err(|e| format!("旧works_tagsレコード読み取り失敗: {e}"))?;

        let new_work_id = match work_id_map.get(&old_work_id) {
            Some(id) => *id,
            None => continue,
        };
        let new_tag_id = match tag_id_map.get(&old_tag_id) {
            Some(id) => *id,
            None => continue,
        };

        conn.execute(
            "INSERT OR IGNORE INTO works_tags (work_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![new_work_id, new_tag_id],
        )
        .map_err(|e| format!("works_tags挿入失敗: {e}"))?;
    }

    Ok(())
}

fn migrate_settings(
    conn: &Connection,
    old_conn: &Connection,
    library_id: &str,
) -> Result<(), String> {
    let has_settings: bool = old_conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='settings'")
        .and_then(|mut s| s.exists([]))
        .map_err(|e| format!("旧settingsテーブル確認失敗: {e}"))?;
    if !has_settings {
        return Ok(());
    }

    let mut stmt = old_conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| format!("旧settings読み取り失敗: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("旧settings読み取り失敗: {e}"))?;

    for row in rows {
        let (key, value) = row.map_err(|e| format!("旧settingsレコード読み取り失敗: {e}"))?;

        conn.execute(
            "INSERT OR IGNORE INTO settings (library_id, key, value) VALUES (?1, ?2, ?3)",
            rusqlite::params![library_id, key, value],
        )
        .map_err(|e| format!("settings挿入失敗: {e}"))?;
    }

    Ok(())
}

fn migrate_playlists(
    conn: &Connection,
    old_conn: &Connection,
    library_id: &str,
) -> Result<HashMap<i64, i64>, String> {
    let mut id_map = HashMap::new();

    let has_playlists: bool = old_conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='playlists'")
        .and_then(|mut s| s.exists([]))
        .map_err(|e| format!("旧playlistsテーブル確認失敗: {e}"))?;
    if !has_playlists {
        return Ok(id_map);
    }

    let mut stmt = old_conn
        .prepare("SELECT id, name, created_at, updated_at FROM playlists")
        .map_err(|e| format!("旧playlists読み取り失敗: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("旧playlists読み取り失敗: {e}"))?;

    for row in rows {
        let (old_id, name, created_at, updated_at) =
            row.map_err(|e| format!("旧playlistsレコード読み取り失敗: {e}"))?;

        conn.execute(
            "INSERT INTO playlists (library_id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![library_id, name, created_at, updated_at],
        )
        .map_err(|e| format!("playlists挿入失敗: {e}"))?;
        id_map.insert(old_id, conn.last_insert_rowid());
    }

    Ok(id_map)
}

fn migrate_playlist_items(
    conn: &Connection,
    old_conn: &Connection,
    playlist_id_map: &HashMap<i64, i64>,
    work_id_map: &HashMap<i64, i64>,
) -> Result<(), String> {
    let has_items: bool = old_conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='playlist_items'")
        .and_then(|mut s| s.exists([]))
        .map_err(|e| format!("旧playlist_itemsテーブル確認失敗: {e}"))?;
    if !has_items {
        return Ok(());
    }

    let mut stmt = old_conn
        .prepare("SELECT playlist_id, work_id, position FROM playlist_items")
        .map_err(|e| format!("旧playlist_items読み取り失敗: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| format!("旧playlist_items読み取り失敗: {e}"))?;

    for row in rows {
        let (old_playlist_id, old_work_id, position) =
            row.map_err(|e| format!("旧playlist_itemsレコード読み取り失敗: {e}"))?;

        let new_playlist_id = match playlist_id_map.get(&old_playlist_id) {
            Some(id) => *id,
            None => continue,
        };
        let new_work_id = match work_id_map.get(&old_work_id) {
            Some(id) => *id,
            None => continue,
        };

        conn.execute(
            "INSERT OR IGNORE INTO playlist_items (playlist_id, work_id, position) VALUES (?1, ?2, ?3)",
            rusqlite::params![new_playlist_id, new_work_id, position],
        )
        .map_err(|e| format!("playlist_items挿入失敗: {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/migration.rs"]
mod tests;
