use super::*;
use rusqlite::Connection;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
        artist: None,
        year: None,
        genre: None,
        circle: None,
        origin: None,
    }
}

fn sample_folder_record<'a>(title: &'a str, path: &'a str) -> WorkRecord<'a> {
    WorkRecord {
        work_type: "folder",
        ..sample_record(title, path)
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
    assert_eq!(detail.artist, None);
    assert_eq!(detail.year, None);
    assert_eq!(detail.genre, None);
    assert_eq!(detail.circle, None);
    assert_eq!(detail.origin, None);
}

#[test]
fn get_work_not_found() {
    let conn = test_conn();
    let result = get_work(&conn, 9999);
    assert!(matches!(result, Err(AppError::NotFound)));
}

#[test]
fn insert_work_with_metadata() {
    let conn = test_conn();
    let record = WorkRecord {
        title: "My Work",
        path: "/meta.jpg",
        work_type: "image",
        page_count: 1,
        thumbnail: b"thumb",
        artist: Some("Artist A"),
        year: Some(2024),
        genre: Some("Fantasy"),
        circle: Some("Circle X"),
        origin: Some("Original"),
    };
    insert_work(&conn, &record).unwrap();

    let works = list_works(&conn, "title", "asc").unwrap();
    let detail = get_work(&conn, works[0].id).unwrap();
    assert_eq!(detail.artist.as_deref(), Some("Artist A"));
    assert_eq!(detail.year, Some(2024));
    assert_eq!(detail.genre.as_deref(), Some("Fantasy"));
    assert_eq!(detail.circle.as_deref(), Some("Circle X"));
    assert_eq!(detail.origin.as_deref(), Some("Original"));
}

// --- Tag CRUD ---

#[test]
fn create_tag_returns_tag() {
    let conn = test_conn();
    let tag = create_tag(&conn, "rust", Some("language")).unwrap();
    assert_eq!(tag.name, "rust");
    assert_eq!(tag.category.as_deref(), Some("language"));
    assert!(tag.id > 0);
}

#[test]
fn create_tag_without_category() {
    let conn = test_conn();
    let tag = create_tag(&conn, "favorite", None).unwrap();
    assert_eq!(tag.name, "favorite");
    assert_eq!(tag.category, None);
}

#[test]
fn create_tag_duplicate_returns_error() {
    let conn = test_conn();
    create_tag(&conn, "rust", Some("language")).unwrap();
    let result = create_tag(&conn, "rust", Some("language"));
    assert!(result.is_err());
}

#[test]
fn create_tag_same_name_different_category() {
    let conn = test_conn();
    let t1 = create_tag(&conn, "action", Some("genre")).unwrap();
    let t2 = create_tag(&conn, "action", Some("theme")).unwrap();
    assert_ne!(t1.id, t2.id);
}

#[test]
fn update_tag_changes_name_and_category() {
    let conn = test_conn();
    let tag = create_tag(&conn, "old", Some("cat")).unwrap();
    update_tag(&conn, tag.id, "new", Some("newcat")).unwrap();

    let tags = search_tags(&conn, "new", None).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "new");
    assert_eq!(tags[0].category.as_deref(), Some("newcat"));
}

#[test]
fn update_tag_not_found() {
    let conn = test_conn();
    let result = update_tag(&conn, 9999, "x", None);
    assert!(matches!(result, Err(AppError::NotFound)));
}

#[test]
fn delete_tag_removes_it() {
    let conn = test_conn();
    let tag = create_tag(&conn, "temp", None).unwrap();
    delete_tag(&conn, tag.id).unwrap();

    let tags = search_tags(&conn, "temp", None).unwrap();
    assert!(tags.is_empty());
}

#[test]
fn delete_tag_not_found() {
    let conn = test_conn();
    let result = delete_tag(&conn, 9999);
    assert!(matches!(result, Err(AppError::NotFound)));
}

#[test]
fn delete_tag_cascades_works_tags() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W", "/w.jpg")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();
    let tag = create_tag(&conn, "to_delete", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();

    delete_tag(&conn, tag.id).unwrap();
    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert!(tags.is_empty());
}

// --- Tag search ---

#[test]
fn search_tags_partial_match() {
    let conn = test_conn();
    create_tag(&conn, "fantasy", Some("genre")).unwrap();
    create_tag(&conn, "sci-fi", Some("genre")).unwrap();

    let tags = search_tags(&conn, "fan", None).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "fantasy");
}

#[test]
fn search_tags_with_category_filter() {
    let conn = test_conn();
    create_tag(&conn, "action", Some("genre")).unwrap();
    create_tag(&conn, "action_hero", Some("character")).unwrap();

    let tags = search_tags(&conn, "action", Some("genre")).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].category.as_deref(), Some("genre"));
}

#[test]
fn search_tags_empty_query_returns_all() {
    let conn = test_conn();
    create_tag(&conn, "a", None).unwrap();
    create_tag(&conn, "b", None).unwrap();

    let tags = search_tags(&conn, "", None).unwrap();
    assert_eq!(tags.len(), 2);
}

// --- works_tags ---

#[test]
fn add_and_get_tags_for_work() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();
    let t1 = create_tag(&conn, "tag_b", None).unwrap();
    let t2 = create_tag(&conn, "tag_a", None).unwrap();
    add_tag_to_work(&conn, works[0].id, t1.id).unwrap();
    add_tag_to_work(&conn, works[0].id, t2.id).unwrap();

    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].name, "tag_a");
    assert_eq!(tags[1].name, "tag_b");
}

#[test]
fn add_tag_to_work_idempotent() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();
    let tag = create_tag(&conn, "dup", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();

    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert_eq!(tags.len(), 1);
}

#[test]
fn remove_tag_from_work_removes_association() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();
    let tag = create_tag(&conn, "rm", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();
    remove_tag_from_work(&conn, works[0].id, tag.id).unwrap();

    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert!(tags.is_empty());
}

#[test]
fn remove_tag_from_work_idempotent() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();
    let tag = create_tag(&conn, "rm2", None).unwrap();
    remove_tag_from_work(&conn, works[0].id, tag.id).unwrap();
}

#[test]
fn get_tags_for_work_empty() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();

    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert!(tags.is_empty());
}

// --- search_works_by_tags ---

#[test]
fn search_works_by_tags_or_mode() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("W1", "/w1")).unwrap();
    insert_work(&conn, &sample_folder_record("W2", "/w2")).unwrap();
    insert_work(&conn, &sample_folder_record("W3", "/w3")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();

    let t1 = create_tag(&conn, "tag1", None).unwrap();
    let t2 = create_tag(&conn, "tag2", None).unwrap();
    add_tag_to_work(&conn, works[0].id, t1.id).unwrap();
    add_tag_to_work(&conn, works[1].id, t2.id).unwrap();

    let result = search_works_by_tags(&conn, &[t1.id, t2.id], "or").unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn search_works_by_tags_and_mode() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("W1", "/w1")).unwrap();
    insert_work(&conn, &sample_folder_record("W2", "/w2")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();

    let t1 = create_tag(&conn, "tag1", None).unwrap();
    let t2 = create_tag(&conn, "tag2", None).unwrap();
    add_tag_to_work(&conn, works[0].id, t1.id).unwrap();
    add_tag_to_work(&conn, works[0].id, t2.id).unwrap();
    add_tag_to_work(&conn, works[1].id, t1.id).unwrap();

    let result = search_works_by_tags(&conn, &[t1.id, t2.id], "and").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].title, "W1");
}

#[test]
fn search_works_by_tags_empty_ids() {
    let conn = test_conn();
    let result = search_works_by_tags(&conn, &[], "or").unwrap();
    assert!(result.is_empty());
}

#[test]
fn search_works_by_tags_no_match() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("W1", "/w1")).unwrap();
    let t1 = create_tag(&conn, "unused", None).unwrap();

    let result = search_works_by_tags(&conn, &[t1.id], "or").unwrap();
    assert!(result.is_empty());
}

#[test]
fn search_works_by_tags_ordered_by_created_at_desc() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("First", "/f1")).unwrap();
    insert_work(&conn, &sample_folder_record("Second", "/f2")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();

    let tag = create_tag(&conn, "shared", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();
    add_tag_to_work(&conn, works[1].id, tag.id).unwrap();

    let result = search_works_by_tags(&conn, &[tag.id], "or").unwrap();
    assert_eq!(result.len(), 2);
    assert!(result[0].created_at >= result[1].created_at);
}

#[test]
fn search_works_by_tags_and_mode_deduplicates_ids() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("W1", "/w1")).unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();

    let tag = create_tag(&conn, "only", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();

    let result = search_works_by_tags(&conn, &[tag.id, tag.id], "and").unwrap();
    assert_eq!(result.len(), 1);
}

// --- database is locked reproduction ---

fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sharaku_test_{}_{}", prefix, ts))
}

/// Reproduce "database is locked" error:
/// An external process (e.g. GNOME tracker-miner-fs) briefly holds an exclusive
/// lock on the newly created database file while open_db tries to run migrations.
#[test]
fn open_db_succeeds_despite_brief_external_lock() {
    let dir = unique_temp_dir("locked");
    std::fs::create_dir_all(&dir).unwrap();

    let db_path = dir.join("sharaku.db");

    // Simulate an external process holding an exclusive lock on the db file
    let blocker = Connection::open(&db_path).unwrap();
    blocker.execute_batch("BEGIN EXCLUSIVE").unwrap();

    let dir_clone = dir.clone();
    let handle = std::thread::spawn(move || open_db(&dir_clone));

    // Release the lock after 200ms (well within the busy_timeout window)
    std::thread::sleep(Duration::from_millis(200));
    blocker.execute_batch("ROLLBACK").unwrap();
    drop(blocker);

    let result = handle.join().unwrap();
    assert!(
        result.is_ok(),
        "open_db should succeed after lock is released, but got: {:?}",
        result.err()
    );

    // Verify the database is usable
    let conn = result.unwrap();
    let works = list_works(&conn, "title", "asc").unwrap();
    assert_eq!(works.len(), 0);

    drop(conn);
    let _ = std::fs::remove_dir_all(&dir);
}

