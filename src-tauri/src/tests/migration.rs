use rusqlite::Connection;
use std::path::PathBuf;

use crate::db::open_db_in_memory;
use crate::library;
use crate::settings;

const OLD_SCHEMA_SQL: &str = "
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS works (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT    NOT NULL,
    path       TEXT    NOT NULL UNIQUE,
    type       TEXT    NOT NULL CHECK (type IN ('image', 'pdf', 'archive', 'folder')),
    page_count INTEGER,
    thumbnail  BLOB,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    artist     TEXT,
    year       INTEGER,
    genre      TEXT,
    circle     TEXT,
    origin     TEXT
);

CREATE TABLE IF NOT EXISTS tags (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    name     TEXT NOT NULL,
    category TEXT,
    UNIQUE(name, category)
);

CREATE TABLE IF NOT EXISTS works_tags (
    work_id INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (work_id, tag_id)
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS playlists (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS playlist_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    work_id     INTEGER NOT NULL REFERENCES works(id)     ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    UNIQUE(playlist_id, work_id),
    UNIQUE(playlist_id, position)
);
";

fn new_conn_with_library(lib_id: &str, lib_path: &str) -> Connection {
    let conn = open_db_in_memory().unwrap();
    library::add_library(&conn, "Test Library", lib_path).unwrap();
    conn.execute(
        "UPDATE libraries SET id = ?1 WHERE path = ?2",
        rusqlite::params![lib_id, lib_path],
    )
    .unwrap();
    conn
}

fn create_old_db(dir: &std::path::Path) -> Connection {
    let db_path = dir.join("sharaku.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    conn.execute_batch(OLD_SCHEMA_SQL).unwrap();
    conn
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("sharaku_migration_test_{}_{}", prefix, ts));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn skips_when_no_old_db() {
    let dir = unique_temp_dir("no_old");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let flag = settings::get_setting(&conn, "lib1", super::MIGRATED_FLAG_KEY).unwrap();
    assert_eq!(flag.as_deref(), Some("no_old_db"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrates_empty_old_db() {
    let dir = unique_temp_dir("empty");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());
    let _old = create_old_db(&dir);
    drop(_old);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let flag = settings::get_setting(&conn, "lib1", super::MIGRATED_FLAG_KEY).unwrap();
    assert_eq!(flag.as_deref(), Some("done"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrates_works_with_id_mapping() {
    let dir = unique_temp_dir("works");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    let old = create_old_db(&dir);
    old.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail) VALUES ('W1', '/p1', 'image', 5, X'AABB')",
        [],
    ).unwrap();
    old.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail, artist, year) VALUES ('W2', '/p2', 'folder', 10, X'CCDD', 'Artist', 2024)",
        [],
    ).unwrap();
    drop(old);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let works = crate::db::list_works(&conn, "lib1", "title", "asc").unwrap();
    assert_eq!(works.len(), 2);
    assert_eq!(works[0].title, "W1");
    assert_eq!(works[1].title, "W2");

    let detail = crate::db::get_work(&conn, works[1].id).unwrap();
    assert_eq!(detail.artist.as_deref(), Some("Artist"));
    assert_eq!(detail.year, Some(2024));

    let thumb = crate::db::get_thumbnail(&conn, works[0].id).unwrap();
    assert_eq!(thumb, vec![0xAA, 0xBB]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrates_tags_with_null_category() {
    let dir = unique_temp_dir("tags");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    let old = create_old_db(&dir);
    old.execute("INSERT INTO tags (name, category) VALUES ('tag1', 'genre')", []).unwrap();
    old.execute("INSERT INTO tags (name, category) VALUES ('tag2', NULL)", []).unwrap();
    drop(old);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let tags = crate::db::search_tags(&conn, "lib1", "", None).unwrap();
    assert_eq!(tags.len(), 2);

    let with_cat = tags.iter().find(|t| t.name == "tag1").unwrap();
    assert_eq!(with_cat.category.as_deref(), Some("genre"));

    let without_cat = tags.iter().find(|t| t.name == "tag2").unwrap();
    assert_eq!(without_cat.category, None);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrates_works_tags() {
    let dir = unique_temp_dir("works_tags");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    let old = create_old_db(&dir);
    old.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail) VALUES ('W1', '/p1', 'image', 1, X'AA')",
        [],
    ).unwrap();
    old.execute("INSERT INTO tags (name, category) VALUES ('t1', NULL)", []).unwrap();
    old.execute("INSERT INTO works_tags (work_id, tag_id) VALUES (1, 1)", []).unwrap();
    drop(old);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let works = crate::db::list_works(&conn, "lib1", "title", "asc").unwrap();
    let tags = crate::db::get_tags_for_work(&conn, works[0].id).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "t1");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrates_settings() {
    let dir = unique_temp_dir("settings");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    let old = create_old_db(&dir);
    old.execute("INSERT INTO settings (key, value) VALUES ('directory_template', '{title}')", []).unwrap();
    old.execute("INSERT INTO settings (key, value) VALUES ('type_label_image', 'Img')", []).unwrap();
    drop(old);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let tmpl = settings::get_setting(&conn, "lib1", "directory_template").unwrap();
    assert_eq!(tmpl.as_deref(), Some("{title}"));

    let label = settings::get_setting(&conn, "lib1", "type_label_image").unwrap();
    assert_eq!(label.as_deref(), Some("Img"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrates_playlists_and_items() {
    let dir = unique_temp_dir("playlists");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    let old = create_old_db(&dir);
    old.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail) VALUES ('W1', '/p1', 'image', 1, X'AA')",
        [],
    ).unwrap();
    old.execute(
        "INSERT INTO playlists (name) VALUES ('My Playlist')",
        [],
    ).unwrap();
    old.execute(
        "INSERT INTO playlist_items (playlist_id, work_id, position) VALUES (1, 1, 0)",
        [],
    ).unwrap();
    drop(old);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM playlists WHERE library_id = 'lib1'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);

    let item_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM playlist_items", [], |r| r.get(0))
        .unwrap();
    assert_eq!(item_count, 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn idempotent_no_duplicates_on_second_run() {
    let dir = unique_temp_dir("idempotent");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    let old = create_old_db(&dir);
    old.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail) VALUES ('W1', '/p1', 'image', 1, X'AA')",
        [],
    ).unwrap();
    old.execute("INSERT INTO tags (name, category) VALUES ('t1', NULL)", []).unwrap();
    old.execute("INSERT INTO works_tags (work_id, tag_id) VALUES (1, 1)", []).unwrap();
    drop(old);

    let errors1 = super::migrate_per_library_dbs(&conn);
    assert!(errors1.is_empty());

    let errors2 = super::migrate_per_library_dbs(&conn);
    assert!(errors2.is_empty());

    let works = crate::db::list_works(&conn, "lib1", "title", "asc").unwrap();
    assert_eq!(works.len(), 1);

    let tags = crate::db::search_tags(&conn, "lib1", "", None).unwrap();
    assert_eq!(tags.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn one_library_failure_does_not_block_others() {
    let dir1 = unique_temp_dir("fail1");
    let dir2 = unique_temp_dir("fail2");
    let conn = open_db_in_memory().unwrap();

    library::add_library(&conn, "Lib1", dir1.to_str().unwrap()).unwrap();
    let lib1_id = library::active_library(&conn).unwrap().unwrap().id;

    library::add_library(&conn, "Lib2", dir2.to_str().unwrap()).unwrap();
    let libs = library::list_libraries(&conn).unwrap();
    let lib2_id = libs.iter().find(|l| l.name == "Lib2").unwrap().id.clone();

    // lib1: broken old db (empty file, not valid sqlite)
    std::fs::write(dir1.join("sharaku.db"), b"not a database").unwrap();

    // lib2: valid old db
    let old2 = create_old_db(&dir2);
    old2.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail) VALUES ('W2', '/p2', 'image', 1, X'BB')",
        [],
    ).unwrap();
    drop(old2);

    let errors = super::migrate_per_library_dbs(&conn);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].library_id, lib1_id);

    let works = crate::db::list_works(&conn, &lib2_id, "title", "asc").unwrap();
    assert_eq!(works.len(), 1);

    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn preserves_created_at_updated_at() {
    let dir = unique_temp_dir("timestamps");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    let old = create_old_db(&dir);
    old.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail, created_at, updated_at) \
         VALUES ('W1', '/p1', 'image', 1, X'AA', '2020-01-01T00:00:00.000Z', '2021-06-15T12:30:00.000Z')",
        [],
    ).unwrap();
    drop(old);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let works = crate::db::list_works(&conn, "lib1", "title", "asc").unwrap();
    assert_eq!(works[0].created_at, "2020-01-01T00:00:00.000Z");

    let (updated_at,): (String,) = conn
        .query_row(
            "SELECT updated_at FROM works WHERE id = ?1",
            [works[0].id],
            |r| Ok((r.get(0)?,)),
        )
        .unwrap();
    assert_eq!(updated_at, "2021-06-15T12:30:00.000Z");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn skips_new_schema_db() {
    let dir = unique_temp_dir("new_schema");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    // Create a DB with the new schema (has library_id column)
    let new_schema_path = dir.join("sharaku.db");
    let new_schema_conn = Connection::open(&new_schema_path).unwrap();
    new_schema_conn
        .execute_batch(include_str!("../../migrations/004_unified_local_db.sql"))
        .unwrap();
    drop(new_schema_conn);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    let flag = settings::get_setting(&conn, "lib1", super::MIGRATED_FLAG_KEY).unwrap();
    assert_eq!(flag.as_deref(), Some("new_schema"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn handles_duplicate_works_by_path() {
    let dir = unique_temp_dir("dup_works");
    let conn = new_conn_with_library("lib1", dir.to_str().unwrap());

    // Pre-existing work in new DB
    crate::db::insert_work(
        &conn,
        &crate::db::WorkRecord {
            library_id: "lib1",
            title: "Existing",
            path: "/p1",
            work_type: "image",
            page_count: 1,
            thumbnail: b"new_thumb",
            artist: None,
            year: None,
            genre: None,
            circle: None,
            origin: None,
        },
    )
    .unwrap();

    let old = create_old_db(&dir);
    old.execute(
        "INSERT INTO works (title, path, type, page_count, thumbnail) VALUES ('Old', '/p1', 'image', 1, X'AA')",
        [],
    ).unwrap();
    old.execute("INSERT INTO tags (name, category) VALUES ('t1', NULL)", []).unwrap();
    old.execute("INSERT INTO works_tags (work_id, tag_id) VALUES (1, 1)", []).unwrap();
    drop(old);

    let errors = super::migrate_per_library_dbs(&conn);
    assert!(errors.is_empty());

    // Should still have only 1 work (the existing one)
    let works = crate::db::list_works(&conn, "lib1", "title", "asc").unwrap();
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].title, "Existing");

    // Tag should be linked to existing work via ID mapping
    let tags = crate::db::get_tags_for_work(&conn, works[0].id).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "t1");

    let _ = std::fs::remove_dir_all(&dir);
}
