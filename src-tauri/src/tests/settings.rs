use super::*;

use crate::db;
use crate::library;

const TEST_LIBRARY_ID: &str = "test_lib_settings";

fn test_conn() -> rusqlite::Connection {
    let conn = db::open_db_in_memory().unwrap();
    library::add_library(&conn, "Test", "/test").ok();
    conn.execute(
        "UPDATE libraries SET id = ?1 WHERE id = (SELECT id FROM libraries LIMIT 1)",
        [TEST_LIBRARY_ID],
    )
    .unwrap();
    conn
}

#[test]
fn get_setting_returns_none_when_not_set() {
    let conn = test_conn();
    assert_eq!(
        get_setting(&conn, TEST_LIBRARY_ID, "nonexistent").unwrap(),
        None
    );
}

#[test]
fn set_and_get_setting() {
    let conn = test_conn();
    set_setting(&conn, TEST_LIBRARY_ID, "key1", "value1").unwrap();
    assert_eq!(
        get_setting(&conn, TEST_LIBRARY_ID, "key1").unwrap(),
        Some("value1".into())
    );
}

#[test]
fn set_setting_overwrites_existing() {
    let conn = test_conn();
    set_setting(&conn, TEST_LIBRARY_ID, "key1", "old").unwrap();
    set_setting(&conn, TEST_LIBRARY_ID, "key1", "new").unwrap();
    assert_eq!(
        get_setting(&conn, TEST_LIBRARY_ID, "key1").unwrap(),
        Some("new".into())
    );
}

#[test]
fn directory_template_helpers() {
    let conn = test_conn();
    assert_eq!(
        get_directory_template(&conn, TEST_LIBRARY_ID).unwrap(),
        None
    );
    set_directory_template(&conn, TEST_LIBRARY_ID, "{artist}/{title}").unwrap();
    assert_eq!(
        get_directory_template(&conn, TEST_LIBRARY_ID).unwrap(),
        Some("{artist}/{title}".into())
    );
}

#[test]
fn type_label_defaults() {
    let conn = test_conn();
    assert_eq!(
        get_type_label_image(&conn, TEST_LIBRARY_ID).unwrap(),
        "Image"
    );
    assert_eq!(
        get_type_label_folder(&conn, TEST_LIBRARY_ID).unwrap(),
        "Folder"
    );
}

#[test]
fn type_label_set_and_get() {
    let conn = test_conn();
    set_type_label_image(&conn, TEST_LIBRARY_ID, "イラスト").unwrap();
    set_type_label_folder(&conn, TEST_LIBRARY_ID, "漫画").unwrap();
    assert_eq!(
        get_type_label_image(&conn, TEST_LIBRARY_ID).unwrap(),
        "イラスト"
    );
    assert_eq!(
        get_type_label_folder(&conn, TEST_LIBRARY_ID).unwrap(),
        "漫画"
    );
}

#[test]
fn resolve_type_label_uses_settings() {
    let conn = test_conn();
    set_type_label_folder(&conn, TEST_LIBRARY_ID, "漫画").unwrap();
    assert_eq!(
        resolve_type_label(&conn, TEST_LIBRARY_ID, "folder").unwrap(),
        "漫画"
    );
    assert_eq!(
        resolve_type_label(&conn, TEST_LIBRARY_ID, "image").unwrap(),
        "Image"
    );
    assert_eq!(
        resolve_type_label(&conn, TEST_LIBRARY_ID, "other").unwrap(),
        "other"
    );
}
