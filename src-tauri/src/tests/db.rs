use super::*;
use rusqlite::Connection;

fn test_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    conn
}

fn sample_record<'a>(title: &'a str, path: &'a str) -> WorkRecord<'a> {
    WorkRecord {
        title,
        path,
        work_type: "image",
        page_count: 1,
        thumbnail: b"fake_thumb",
    }
}

#[test]
fn insert_and_list_round_trip() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("Alpha", "/a.jpg")).unwrap();
    insert_work(&conn, &sample_record("Beta", "/b.jpg")).unwrap();

    let works = list_works(&conn, "title", "asc").unwrap();
    assert_eq!(works.len(), 2);
    assert_eq!(works[0].title, "Alpha");
    assert_eq!(works[1].title, "Beta");
}

#[test]
fn path_exists_true_and_false() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/exists.jpg")).unwrap();

    assert!(path_exists(&conn, "/exists.jpg").unwrap());
    assert!(!path_exists(&conn, "/not_here.jpg").unwrap());
}

#[test]
fn duplicate_path_returns_error() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/dup.jpg")).unwrap();
    let result = insert_work(&conn, &sample_record("B", "/dup.jpg"));
    assert!(result.is_err());
}

#[test]
fn list_works_sort_by_created_at_desc() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("First", "/1.jpg")).unwrap();
    insert_work(&conn, &sample_record("Second", "/2.jpg")).unwrap();

    let works = list_works(&conn, "created_at", "desc").unwrap();
    assert_eq!(works.len(), 2);
    assert!(works[0].created_at >= works[1].created_at);
}

#[test]
fn list_works_sort_by_title_desc() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("Alpha", "/a.jpg")).unwrap();
    insert_work(&conn, &sample_record("Beta", "/b.jpg")).unwrap();

    let works = list_works(&conn, "title", "desc").unwrap();
    assert_eq!(works[0].title, "Beta");
    assert_eq!(works[1].title, "Alpha");
}

#[test]
fn unknown_sort_by_falls_back_to_created_at() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/a.jpg")).unwrap();
    let works = list_works(&conn, "invalid_column", "asc").unwrap();
    assert_eq!(works.len(), 1);
}

#[test]
fn get_thumbnail_returns_data() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/a.jpg")).unwrap();

    let works = list_works(&conn, "title", "asc").unwrap();
    let thumb = get_thumbnail(&conn, works[0].id).unwrap();
    assert_eq!(thumb, b"fake_thumb");
}

#[test]
fn get_thumbnail_not_found() {
    let conn = test_conn();
    let result = get_thumbnail(&conn, 9999);
    assert!(matches!(result, Err(AppError::NotFound)));
}

#[test]
fn get_work_returns_detail() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("Title", "/path.jpg")).unwrap();

    let works = list_works(&conn, "title", "asc").unwrap();
    let detail = get_work(&conn, works[0].id).unwrap();
    assert_eq!(detail.title, "Title");
    assert_eq!(detail.path, "/path.jpg");
    assert_eq!(detail.work_type, "image");
}

#[test]
fn get_work_not_found() {
    let conn = test_conn();
    let result = get_work(&conn, 9999);
    assert!(matches!(result, Err(AppError::NotFound)));
}
