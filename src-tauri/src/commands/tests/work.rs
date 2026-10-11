use std::fs;

use tempfile::TempDir;

use super::*;
use crate::db::{self, WorkRecord};
use crate::test_common::test_db_with_library;

const TEST_LIBRARY_ID: &str = "test_lib_work";

fn insert_work_at(conn: &rusqlite::Connection, path: &str) -> i64 {
    db::insert_work(
        conn,
        &WorkRecord {
            library_id: TEST_LIBRARY_ID,
            title: "Work",
            path,
            work_type: "folder",
            page_count: 1,
            thumbnail: b"thumb",
            artist: None,
            year: None,
            genre: None,
            circle: None,
            origin: None,
        },
    )
    .unwrap();
    conn.last_insert_rowid()
}

const MISSING_MESSAGE: &str = "見つかりません";

#[test]
fn returns_path_when_it_exists_on_disk() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let lib_root = TempDir::new().unwrap();
    let work_dir = lib_root.path().join("work");
    fs::create_dir_all(&work_dir).unwrap();

    let work_id = insert_work_at(&conn, &work_dir.to_string_lossy());

    let result = validated_work_path(&conn, work_id, MISSING_MESSAGE).unwrap();

    assert_eq!(result, work_dir);
}

#[test]
fn accepts_path_outside_any_library_root_when_it_exists() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let outside = TempDir::new().unwrap();
    let outside_work_dir = outside.path().join("work");
    fs::create_dir_all(&outside_work_dir).unwrap();

    let work_id = insert_work_at(&conn, &outside_work_dir.to_string_lossy());

    let result = validated_work_path(&conn, work_id, MISSING_MESSAGE).unwrap();

    assert_eq!(result, outside_work_dir);
}

#[test]
fn rejects_path_missing_from_disk() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let missing_path = PathBuf::from("/nonexistent-sharaku-test-path/work");

    let work_id = insert_work_at(&conn, &missing_path.to_string_lossy());

    let result = validated_work_path(&conn, work_id, MISSING_MESSAGE);

    assert_eq!(result, Err(MISSING_MESSAGE.to_string()));
}
