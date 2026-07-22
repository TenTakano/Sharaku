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

const OUT_OF_ROOT_MESSAGE: &str = "ルート外です";

#[test]
fn returns_path_when_within_library_root() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let lib_root = TempDir::new().unwrap();
    let work_dir = lib_root.path().join("work");
    fs::create_dir_all(&work_dir).unwrap();

    let work_id = insert_work_at(&conn, &work_dir.to_string_lossy());

    let result = validated_work_path(&conn, work_id, lib_root.path(), OUT_OF_ROOT_MESSAGE).unwrap();

    assert_eq!(result, work_dir);
}

#[test]
fn rejects_path_outside_library_root() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let lib_root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let outside_work_dir = outside.path().join("work");
    fs::create_dir_all(&outside_work_dir).unwrap();

    let work_id = insert_work_at(&conn, &outside_work_dir.to_string_lossy());

    let result = validated_work_path(&conn, work_id, lib_root.path(), OUT_OF_ROOT_MESSAGE);

    assert_eq!(result, Err(OUT_OF_ROOT_MESSAGE.to_string()));
}

#[test]
#[cfg(unix)]
fn rejects_symlink_resolving_outside_library_root() {
    use std::os::unix::fs::symlink;

    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let lib_root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let outside_work_dir = outside.path().join("real_work");
    fs::create_dir_all(&outside_work_dir).unwrap();

    let linked_path = lib_root.path().join("linked_work");
    symlink(&outside_work_dir, &linked_path).unwrap();

    let work_id = insert_work_at(&conn, &linked_path.to_string_lossy());

    let result = validated_work_path(&conn, work_id, lib_root.path(), OUT_OF_ROOT_MESSAGE);

    assert_eq!(result, Err(OUT_OF_ROOT_MESSAGE.to_string()));
}

#[test]
fn falls_back_to_raw_paths_when_canonicalize_fails() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let lib_root = PathBuf::from("/nonexistent-sharaku-test-root");
    let work_path = lib_root.join("sub/work");

    let work_id = insert_work_at(&conn, &work_path.to_string_lossy());

    let result = validated_work_path(&conn, work_id, &lib_root, OUT_OF_ROOT_MESSAGE).unwrap();

    assert_eq!(result, work_path);
}

#[test]
fn falls_back_to_raw_paths_and_rejects_when_outside_nonexistent_root() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let lib_root = PathBuf::from("/nonexistent-sharaku-test-root-a");
    let work_path = PathBuf::from("/nonexistent-sharaku-test-root-b/work");

    let work_id = insert_work_at(&conn, &work_path.to_string_lossy());

    let result = validated_work_path(&conn, work_id, &lib_root, OUT_OF_ROOT_MESSAGE);

    assert_eq!(result, Err(OUT_OF_ROOT_MESSAGE.to_string()));
}
