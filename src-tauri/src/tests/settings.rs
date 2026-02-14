use rusqlite::Connection;

use crate::db;

use super::*;

fn test_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db_for_test(&conn).unwrap();
    conn
}

#[test]
fn get_setting_returns_none_when_not_set() {
    let conn = test_conn();
    assert_eq!(get_setting(&conn, "nonexistent").unwrap(), None);
}

#[test]
fn set_and_get_setting() {
    let conn = test_conn();
    set_setting(&conn, "key1", "value1").unwrap();
    assert_eq!(get_setting(&conn, "key1").unwrap(), Some("value1".into()));
}

#[test]
fn set_setting_overwrites_existing() {
    let conn = test_conn();
    set_setting(&conn, "key1", "old").unwrap();
    set_setting(&conn, "key1", "new").unwrap();
    assert_eq!(get_setting(&conn, "key1").unwrap(), Some("new".into()));
}

#[test]
fn library_root_helpers() {
    let conn = test_conn();
    assert_eq!(get_library_root(&conn).unwrap(), None);
    set_library_root(&conn, "/my/library").unwrap();
    assert_eq!(get_library_root(&conn).unwrap(), Some("/my/library".into()));
}

#[test]
fn directory_template_helpers() {
    let conn = test_conn();
    assert_eq!(get_directory_template(&conn).unwrap(), None);
    set_directory_template(&conn, "{artist}/{title}").unwrap();
    assert_eq!(
        get_directory_template(&conn).unwrap(),
        Some("{artist}/{title}".into())
    );
}

#[test]
fn type_label_defaults() {
    let conn = test_conn();
    assert_eq!(get_type_label_image(&conn).unwrap(), "Image");
    assert_eq!(get_type_label_folder(&conn).unwrap(), "Folder");
}

#[test]
fn type_label_set_and_get() {
    let conn = test_conn();
    set_type_label_image(&conn, "イラスト").unwrap();
    set_type_label_folder(&conn, "漫画").unwrap();
    assert_eq!(get_type_label_image(&conn).unwrap(), "イラスト");
    assert_eq!(get_type_label_folder(&conn).unwrap(), "漫画");
}

#[test]
fn resolve_type_label_uses_settings() {
    let conn = test_conn();
    set_type_label_folder(&conn, "漫画").unwrap();
    assert_eq!(resolve_type_label(&conn, "folder").unwrap(), "漫画");
    assert_eq!(resolve_type_label(&conn, "image").unwrap(), "Image");
    assert_eq!(resolve_type_label(&conn, "other").unwrap(), "other");
}
