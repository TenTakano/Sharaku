use super::*;
use crate::db;

fn test_conn() -> rusqlite::Connection {
    db::open_db_in_memory().unwrap()
}

#[test]
fn list_empty_returns_empty() {
    let conn = test_conn();
    let libs = list_libraries(&conn).unwrap();
    assert!(libs.is_empty());
}

#[test]
fn add_library_creates_entry() {
    let conn = test_conn();
    let lib = add_library(&conn, "Photos", Some("/path/to/photos")).unwrap();
    assert_eq!(lib.name, "Photos");
    assert_eq!(lib.path, Some("/path/to/photos".to_string()));
    assert!(!lib.id.is_empty());

    let libs = list_libraries(&conn).unwrap();
    assert_eq!(libs.len(), 1);

    let active = active_library(&conn).unwrap();
    assert!(active.is_some());
    assert_eq!(active.unwrap().id, lib.id);
}

#[test]
fn add_duplicate_path_returns_error() {
    let conn = test_conn();
    add_library(&conn, "Photos", Some("/same/path")).unwrap();
    let result = add_library(&conn, "Another", Some("/same/path"));
    assert!(result.is_err());
}

#[test]
fn add_second_library_keeps_first_active() {
    let conn = test_conn();
    let first = add_library(&conn, "First", Some("/first")).unwrap();
    add_library(&conn, "Second", Some("/second")).unwrap();

    let active = active_library(&conn).unwrap().unwrap();
    assert_eq!(active.id, first.id);
}

#[test]
fn add_library_with_no_path() {
    let conn = test_conn();
    let lib = add_library(&conn, "MetadataOnly", None).unwrap();
    assert_eq!(lib.name, "MetadataOnly");
    assert_eq!(lib.path, None);

    let libs = list_libraries(&conn).unwrap();
    assert_eq!(libs.len(), 1);
}

#[test]
fn add_multiple_null_path_libraries() {
    let conn = test_conn();
    let lib1 = add_library(&conn, "Meta1", None).unwrap();
    let lib2 = add_library(&conn, "Meta2", None).unwrap();
    assert_ne!(lib1.id, lib2.id);

    let libs = list_libraries(&conn).unwrap();
    assert_eq!(libs.len(), 2);
}

#[test]
fn remove_library_removes_entry() {
    let conn = test_conn();
    let lib = add_library(&conn, "ToRemove", Some("/remove")).unwrap();
    remove_library(&conn, &lib.id).unwrap();

    let libs = list_libraries(&conn).unwrap();
    assert!(libs.is_empty());
}

#[test]
fn remove_active_library_activates_next() {
    let conn = test_conn();
    let first = add_library(&conn, "First", Some("/first")).unwrap();
    let second = add_library(&conn, "Second", Some("/second")).unwrap();

    remove_library(&conn, &first.id).unwrap();
    let libs = list_libraries(&conn).unwrap();
    assert_eq!(libs.len(), 1);

    let active = active_library(&conn).unwrap().unwrap();
    assert_eq!(active.id, second.id);
}

#[test]
fn remove_nonexistent_returns_error() {
    let conn = test_conn();
    let result = remove_library(&conn, "fake_id");
    assert!(result.is_err());
}

#[test]
fn set_active() {
    let conn = test_conn();
    add_library(&conn, "First", Some("/first")).unwrap();
    let second = add_library(&conn, "Second", Some("/second")).unwrap();

    set_active_library(&conn, &second.id).unwrap();
    let active = active_library(&conn).unwrap().unwrap();
    assert_eq!(active.id, second.id);
}

#[test]
fn set_active_nonexistent_returns_error() {
    let conn = test_conn();
    let result = set_active_library(&conn, "fake_id");
    assert!(result.is_err());
}

#[test]
fn find_by_id_found() {
    let conn = test_conn();
    let lib = add_library(&conn, "Test", Some("/test")).unwrap();
    let found = find_library_by_id(&conn, &lib.id).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test");
}

#[test]
fn find_by_id_not_found() {
    let conn = test_conn();
    let found = find_library_by_id(&conn, "fake_id").unwrap();
    assert!(found.is_none());
}

#[test]
fn active_library_returns_active() {
    let conn = test_conn();
    let lib = add_library(&conn, "Active", Some("/active")).unwrap();
    let active = active_library(&conn).unwrap();
    assert!(active.is_some());
    assert_eq!(active.unwrap().id, lib.id);
}

#[test]
fn active_library_returns_none_when_empty() {
    let conn = test_conn();
    let active = active_library(&conn).unwrap();
    assert!(active.is_none());
}
