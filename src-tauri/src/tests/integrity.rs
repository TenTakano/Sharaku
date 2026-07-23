use std::fs;

use rusqlite::Connection;
use tempfile::TempDir;

use super::*;
use crate::db::{self, WorkRecord};
use crate::settings;
use crate::test_common::test_db_with_library;

const TEST_LIBRARY_ID: &str = "test_lib_integrity";

fn test_conn() -> Connection {
    test_db_with_library(TEST_LIBRARY_ID)
}

fn insert_work(conn: &Connection, title: &str, path: &str, work_type: &str) {
    db::insert_work(
        conn,
        &WorkRecord {
            library_id: TEST_LIBRARY_ID,
            title,
            path,
            work_type,
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
}

fn noop_progress(_: IntegrityCheckProgress) {}

// --- check_integrity_core tests ---

#[test]
fn no_issues_when_all_files_exist() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("image.jpg");
    fs::write(&file_path, b"fake").unwrap();

    let conn = test_conn();
    insert_work(&conn, "Image1", &file_path.to_string_lossy(), "image");

    let report =
        check_integrity_core(&conn, TEST_LIBRARY_ID, Some(tmp.path()), noop_progress).unwrap();

    assert_eq!(report.total_works, 1);
    assert!(report.orphan_works.is_empty());
    assert!(report.unregistered_entries.is_empty());
}

#[test]
fn detects_orphan_when_file_missing() {
    let conn = test_conn();
    insert_work(&conn, "Missing", "/nonexistent/image.jpg", "image");

    let report = check_integrity_core(&conn, TEST_LIBRARY_ID, None, noop_progress).unwrap();

    assert_eq!(report.total_works, 1);
    assert_eq!(report.orphan_works.len(), 1);
    assert_eq!(report.orphan_works[0].title, "Missing");
    assert_eq!(report.orphan_works[0].work_type, "image");
}

#[test]
fn detects_orphan_folder_when_dir_missing() {
    let conn = test_conn();
    insert_work(&conn, "MissingFolder", "/nonexistent/folder", "folder");

    let report = check_integrity_core(&conn, TEST_LIBRARY_ID, None, noop_progress).unwrap();

    assert_eq!(report.orphan_works.len(), 1);
    assert_eq!(report.orphan_works[0].title, "MissingFolder");
    assert_eq!(report.orphan_works[0].work_type, "folder");
}

#[test]
fn detects_unregistered_directory() {
    let tmp = TempDir::new().unwrap();

    let registered_dir = tmp.path().join("registered");
    fs::create_dir_all(&registered_dir).unwrap();
    fs::write(registered_dir.join("01.jpg"), b"fake").unwrap();

    let unregistered_dir = tmp.path().join("unregistered");
    fs::create_dir_all(&unregistered_dir).unwrap();
    fs::write(unregistered_dir.join("01.png"), b"fake").unwrap();
    fs::write(unregistered_dir.join("02.jpg"), b"fake").unwrap();

    let conn = test_conn();
    insert_work(
        &conn,
        "Registered",
        &registered_dir.to_string_lossy(),
        "folder",
    );

    let report =
        check_integrity_core(&conn, TEST_LIBRARY_ID, Some(tmp.path()), noop_progress).unwrap();

    assert_eq!(report.unregistered_entries.len(), 1);
    assert_eq!(report.unregistered_entries[0].folder_name, "unregistered");
    assert_eq!(report.unregistered_entries[0].image_count, 2);
}

#[test]
fn skips_phase2_when_metadata_only() {
    let tmp = TempDir::new().unwrap();
    let unregistered_dir = tmp.path().join("extra");
    fs::create_dir_all(&unregistered_dir).unwrap();
    fs::write(unregistered_dir.join("01.jpg"), b"fake").unwrap();

    let conn = test_conn();
    settings::set_resource_mode(&conn, TEST_LIBRARY_ID, "metadata_only").unwrap();

    let report =
        check_integrity_core(&conn, TEST_LIBRARY_ID, Some(tmp.path()), noop_progress).unwrap();

    assert!(report.unregistered_entries.is_empty());
}

#[test]
fn skips_phase2_when_library_root_is_none() {
    let conn = test_conn();

    let report = check_integrity_core(&conn, TEST_LIBRARY_ID, None, noop_progress).unwrap();

    assert!(report.unregistered_entries.is_empty());
}

// --- delete_orphan_works tests ---

#[test]
fn deletes_specified_orphan_works() {
    let conn = test_conn();
    insert_work(&conn, "Work1", "/path/a", "image");
    insert_work(&conn, "Work2", "/path/b", "image");
    insert_work(&conn, "Work3", "/path/c", "image");

    let entries = db::list_work_paths(&conn, TEST_LIBRARY_ID).unwrap();
    assert_eq!(entries.len(), 3);

    let ids_to_delete: Vec<i64> = entries.iter().take(2).map(|e| e.id).collect();
    let deleted = delete_orphan_works(&conn, TEST_LIBRARY_ID, &ids_to_delete).unwrap();
    assert_eq!(deleted, 2);

    let remaining = db::list_work_paths(&conn, TEST_LIBRARY_ID).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].title, "Work3");
}

#[test]
fn handles_nonexistent_ids_gracefully() {
    let conn = test_conn();
    insert_work(&conn, "Keep", "/path/keep", "image");

    let deleted = delete_orphan_works(&conn, TEST_LIBRARY_ID, &[9999, 8888]).unwrap();
    assert_eq!(deleted, 0);

    let remaining = db::list_work_paths(&conn, TEST_LIBRARY_ID).unwrap();
    assert_eq!(remaining.len(), 1);
}
