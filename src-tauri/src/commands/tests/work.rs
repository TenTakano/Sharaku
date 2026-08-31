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
    let tmp_dir = TempDir::new().unwrap();
    let work_dir = tmp_dir.path().join("work");
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

#[test]
fn copy_recursive_copies_nested_directory_contents() {
    let src_dir = TempDir::new().unwrap();
    let dest_dir = TempDir::new().unwrap();
    let src = src_dir.path().join("work");
    let dest = dest_dir.path().join("work");

    fs::create_dir_all(src.join("nested")).unwrap();
    fs::write(src.join("top.txt"), b"top").unwrap();
    fs::write(src.join("nested").join("inner.txt"), b"inner").unwrap();

    copy_recursive(&src, &dest).unwrap();

    assert_eq!(fs::read(dest.join("top.txt")).unwrap(), b"top");
    assert_eq!(
        fs::read(dest.join("nested").join("inner.txt")).unwrap(),
        b"inner"
    );
    assert!(src.join("top.txt").exists(), "source must be left intact");
}

#[test]
fn copy_recursive_copies_single_file() {
    let src_dir = TempDir::new().unwrap();
    let dest_dir = TempDir::new().unwrap();
    let src = src_dir.path().join("work.txt");
    let dest = dest_dir.path().join("work.txt");
    fs::write(&src, b"content").unwrap();

    copy_recursive(&src, &dest).unwrap();

    assert_eq!(fs::read(dest).unwrap(), b"content");
}

#[test]
fn move_path_moves_directory_within_same_filesystem() {
    let root = TempDir::new().unwrap();
    let src = root.path().join("src");
    let dest = root.path().join("dest");
    fs::create_dir_all(src.join("nested")).unwrap();
    fs::write(src.join("nested").join("file.txt"), b"data").unwrap();

    move_path(&src, &dest).unwrap();

    assert!(!src.exists());
    assert_eq!(
        fs::read(dest.join("nested").join("file.txt")).unwrap(),
        b"data"
    );
}

#[test]
fn move_path_moves_file_within_same_filesystem() {
    let root = TempDir::new().unwrap();
    let src = root.path().join("src.txt");
    let dest = root.path().join("dest.txt");
    fs::write(&src, b"data").unwrap();

    move_path(&src, &dest).unwrap();

    assert!(!src.exists());
    assert_eq!(fs::read(dest).unwrap(), b"data");
}

// A partial `remove_dir_all(src)` failure (e.g. one sub-entry is
// unremovable) must not roll back `dest`, because `dest` is the only
// remaining copy of the entries that were already removed from `src`
// before the failure. Deleting `dest` in that situation would destroy
// data that no longer exists anywhere else (regression test for F1).
#[cfg(unix)]
#[test]
fn copy_then_remove_src_keeps_dest_when_src_removal_partially_fails() {
    use std::os::unix::fs::PermissionsExt;

    let src_dir = TempDir::new().unwrap();
    let dest_dir = TempDir::new().unwrap();
    let src = src_dir.path().join("work");
    let dest = dest_dir.path().join("work");

    fs::create_dir_all(src.join("locked")).unwrap();
    fs::write(src.join("removable.txt"), b"removable").unwrap();
    fs::write(src.join("locked").join("stuck.txt"), b"stuck").unwrap();

    let locked_dir = src.join("locked");
    fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o555)).unwrap();

    let result = copy_then_remove_src(&src, &dest);

    fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o755)).unwrap();

    assert!(result.is_err(), "src removal must fail for this test to be meaningful");
    assert!(
        !src.join("removable.txt").exists(),
        "src's removable entry is expected to be gone before the failure"
    );
    assert_eq!(
        fs::read(dest.join("removable.txt")).unwrap(),
        b"removable",
        "dest must retain the copy of the entry already removed from src"
    );
    assert_eq!(
        fs::read(dest.join("locked").join("stuck.txt")).unwrap(),
        b"stuck",
        "dest must not be rolled back on partial src removal failure"
    );
}
