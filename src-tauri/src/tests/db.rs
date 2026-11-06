use super::*;
use crate::library;
use crate::test_common::test_db_with_library;
use rusqlite::Connection;
use tempfile::TempDir;

const TEST_LIBRARY_ID: &str = "test_lib_001";

fn test_conn() -> Connection {
    test_db_with_library(TEST_LIBRARY_ID)
}

fn sample_record<'a>(title: &'a str, path: &'a str) -> WorkRecord<'a> {
    WorkRecord {
        library_id: TEST_LIBRARY_ID,
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

    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    assert_eq!(works.len(), 2);
    assert_eq!(works[0].title, "Alpha");
    assert_eq!(works[1].title, "Beta");
}

#[test]
fn path_exists_true_and_false() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/exists.jpg")).unwrap();

    assert!(path_exists(&conn, TEST_LIBRARY_ID, "/exists.jpg").unwrap());
    assert!(!path_exists(&conn, TEST_LIBRARY_ID, "/not_here.jpg").unwrap());
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

    let works = list_works(&conn, TEST_LIBRARY_ID, "created_at", "desc").unwrap();
    assert_eq!(works.len(), 2);
    assert!(works[0].created_at >= works[1].created_at);
}

#[test]
fn list_works_sort_by_title_desc() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("Alpha", "/a.jpg")).unwrap();
    insert_work(&conn, &sample_record("Beta", "/b.jpg")).unwrap();

    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "desc").unwrap();
    assert_eq!(works[0].title, "Beta");
    assert_eq!(works[1].title, "Alpha");
}

#[test]
fn unknown_sort_by_falls_back_to_created_at() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/a.jpg")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "invalid_column", "asc").unwrap();
    assert_eq!(works.len(), 1);
}

#[test]
fn get_thumbnail_returns_data() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/a.jpg")).unwrap();

    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
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

    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
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
        library_id: TEST_LIBRARY_ID,
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

    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
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
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "rust", Some("language")).unwrap();
    assert_eq!(tag.name, "rust");
    assert_eq!(tag.category.as_deref(), Some("language"));
    assert!(tag.id > 0);
}

#[test]
fn create_tag_without_category() {
    let conn = test_conn();
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "favorite", None).unwrap();
    assert_eq!(tag.name, "favorite");
    assert_eq!(tag.category, None);
}

#[test]
fn create_tag_duplicate_returns_error() {
    let conn = test_conn();
    create_tag(&conn, TEST_LIBRARY_ID, "rust", Some("language")).unwrap();
    let result = create_tag(&conn, TEST_LIBRARY_ID, "rust", Some("language"));
    assert!(result.is_err());
}

#[test]
fn create_tag_same_name_different_category() {
    let conn = test_conn();
    let t1 = create_tag(&conn, TEST_LIBRARY_ID, "action", Some("genre")).unwrap();
    let t2 = create_tag(&conn, TEST_LIBRARY_ID, "action", Some("theme")).unwrap();
    assert_ne!(t1.id, t2.id);
}

#[test]
fn update_tag_changes_name_and_category() {
    let conn = test_conn();
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "old", Some("cat")).unwrap();
    update_tag(&conn, tag.id, "new", Some("newcat")).unwrap();

    let tags = search_tags(&conn, TEST_LIBRARY_ID, "new", None).unwrap();
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
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "temp", None).unwrap();
    delete_tag(&conn, tag.id).unwrap();

    let tags = search_tags(&conn, TEST_LIBRARY_ID, "temp", None).unwrap();
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
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "to_delete", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();

    delete_tag(&conn, tag.id).unwrap();
    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert!(tags.is_empty());
}

// --- Tag search ---

#[test]
fn search_tags_partial_match() {
    let conn = test_conn();
    create_tag(&conn, TEST_LIBRARY_ID, "fantasy", Some("genre")).unwrap();
    create_tag(&conn, TEST_LIBRARY_ID, "sci-fi", Some("genre")).unwrap();

    let tags = search_tags(&conn, TEST_LIBRARY_ID, "fan", None).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "fantasy");
}

#[test]
fn search_tags_with_category_filter() {
    let conn = test_conn();
    create_tag(&conn, TEST_LIBRARY_ID, "action", Some("genre")).unwrap();
    create_tag(&conn, TEST_LIBRARY_ID, "action_hero", Some("character")).unwrap();

    let tags = search_tags(&conn, TEST_LIBRARY_ID, "action", Some("genre")).unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].category.as_deref(), Some("genre"));
}

#[test]
fn search_tags_empty_query_returns_all() {
    let conn = test_conn();
    create_tag(&conn, TEST_LIBRARY_ID, "a", None).unwrap();
    create_tag(&conn, TEST_LIBRARY_ID, "b", None).unwrap();

    let tags = search_tags(&conn, TEST_LIBRARY_ID, "", None).unwrap();
    assert_eq!(tags.len(), 2);
}

// --- list_tags ---

#[test]
fn list_tags_returns_all_sorted_by_category_and_name() {
    let conn = test_conn();
    create_tag(&conn, TEST_LIBRARY_ID, "sci-fi", Some("genre")).unwrap();
    create_tag(&conn, TEST_LIBRARY_ID, "monet", Some("artist")).unwrap();
    create_tag(&conn, TEST_LIBRARY_ID, "fantasy", Some("genre")).unwrap();
    create_tag(&conn, TEST_LIBRARY_ID, "uncategorized", None).unwrap();

    let tags = list_tags(&conn, TEST_LIBRARY_ID).unwrap();
    assert_eq!(tags.len(), 4);
    // NULL category sorts first in SQLite, then "artist", then "genre"
    assert_eq!(tags[0].name, "uncategorized");
    assert_eq!(tags[1].name, "monet");
    assert_eq!(tags[2].name, "fantasy");
    assert_eq!(tags[3].name, "sci-fi");
}

#[test]
fn list_tags_empty_library() {
    let conn = test_conn();
    let tags = list_tags(&conn, TEST_LIBRARY_ID).unwrap();
    assert!(tags.is_empty());
}

#[test]
fn list_tags_no_limit() {
    let conn = test_conn();
    for i in 0..60 {
        create_tag(&conn, TEST_LIBRARY_ID, &format!("tag_{:03}", i), None).unwrap();
    }
    let tags = list_tags(&conn, TEST_LIBRARY_ID).unwrap();
    assert_eq!(tags.len(), 60);
}

// --- works_tags ---

#[test]
fn add_and_get_tags_for_work() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let t1 = create_tag(&conn, TEST_LIBRARY_ID, "tag_b", None).unwrap();
    let t2 = create_tag(&conn, TEST_LIBRARY_ID, "tag_a", None).unwrap();
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
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "dup", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();

    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert_eq!(tags.len(), 1);
}

#[test]
fn remove_tag_from_work_removes_association() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "rm", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();
    remove_tag_from_work(&conn, works[0].id, tag.id).unwrap();

    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert!(tags.is_empty());
}

#[test]
fn remove_tag_from_work_idempotent() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "rm2", None).unwrap();
    remove_tag_from_work(&conn, works[0].id, tag.id).unwrap();
}

#[test]
fn get_tags_for_work_empty() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W1", "/w1.jpg")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();

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
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();

    let t1 = create_tag(&conn, TEST_LIBRARY_ID, "tag1", None).unwrap();
    let t2 = create_tag(&conn, TEST_LIBRARY_ID, "tag2", None).unwrap();
    add_tag_to_work(&conn, works[0].id, t1.id).unwrap();
    add_tag_to_work(&conn, works[1].id, t2.id).unwrap();

    let result = search_works_by_tags(&conn, TEST_LIBRARY_ID, &[t1.id, t2.id], "or").unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn search_works_by_tags_and_mode() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("W1", "/w1")).unwrap();
    insert_work(&conn, &sample_folder_record("W2", "/w2")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();

    let t1 = create_tag(&conn, TEST_LIBRARY_ID, "tag1", None).unwrap();
    let t2 = create_tag(&conn, TEST_LIBRARY_ID, "tag2", None).unwrap();
    add_tag_to_work(&conn, works[0].id, t1.id).unwrap();
    add_tag_to_work(&conn, works[0].id, t2.id).unwrap();
    add_tag_to_work(&conn, works[1].id, t1.id).unwrap();

    let result = search_works_by_tags(&conn, TEST_LIBRARY_ID, &[t1.id, t2.id], "and").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].title, "W1");
}

#[test]
fn search_works_by_tags_empty_ids() {
    let conn = test_conn();
    let result = search_works_by_tags(&conn, TEST_LIBRARY_ID, &[], "or").unwrap();
    assert!(result.is_empty());
}

#[test]
fn search_works_by_tags_no_match() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("W1", "/w1")).unwrap();
    let t1 = create_tag(&conn, TEST_LIBRARY_ID, "unused", None).unwrap();

    let result = search_works_by_tags(&conn, TEST_LIBRARY_ID, &[t1.id], "or").unwrap();
    assert!(result.is_empty());
}

#[test]
fn search_works_by_tags_ordered_by_created_at_desc() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("First", "/f1")).unwrap();
    insert_work(&conn, &sample_folder_record("Second", "/f2")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();

    let tag = create_tag(&conn, TEST_LIBRARY_ID, "shared", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();
    add_tag_to_work(&conn, works[1].id, tag.id).unwrap();

    let result = search_works_by_tags(&conn, TEST_LIBRARY_ID, &[tag.id], "or").unwrap();
    assert_eq!(result.len(), 2);
    assert!(result[0].created_at >= result[1].created_at);
}

#[test]
fn search_works_by_tags_and_mode_deduplicates_ids() {
    let conn = test_conn();
    insert_work(&conn, &sample_folder_record("W1", "/w1")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();

    let tag = create_tag(&conn, TEST_LIBRARY_ID, "only", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();

    let result = search_works_by_tags(&conn, TEST_LIBRARY_ID, &[tag.id, tag.id], "and").unwrap();
    assert_eq!(result.len(), 1);
}

// --- delete_works_by_ids ---

#[test]
fn delete_works_by_ids_removes_works() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/a.jpg")).unwrap();
    insert_work(&conn, &sample_record("B", "/b.jpg")).unwrap();
    insert_work(&conn, &sample_record("C", "/c.jpg")).unwrap();

    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let ids: Vec<i64> = works.iter().take(2).map(|w| w.id).collect();

    let deleted = delete_works_by_ids(&conn, TEST_LIBRARY_ID, &ids).unwrap();
    assert_eq!(deleted, 2);

    let remaining = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].title, "C");
}

#[test]
fn delete_works_by_ids_empty_ids() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("A", "/a.jpg")).unwrap();

    let deleted = delete_works_by_ids(&conn, TEST_LIBRARY_ID, &[]).unwrap();
    assert_eq!(deleted, 0);

    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    assert_eq!(works.len(), 1);
}

#[test]
fn delete_works_by_ids_nonexistent_ids() {
    let conn = test_conn();
    let deleted = delete_works_by_ids(&conn, TEST_LIBRARY_ID, &[9999, 8888]).unwrap();
    assert_eq!(deleted, 0);
}

#[test]
fn delete_works_by_ids_cascades_tags() {
    let conn = test_conn();
    insert_work(&conn, &sample_record("W", "/w.jpg")).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let tag = create_tag(&conn, TEST_LIBRARY_ID, "test_tag", None).unwrap();
    add_tag_to_work(&conn, works[0].id, tag.id).unwrap();

    let deleted = delete_works_by_ids(&conn, TEST_LIBRARY_ID, &[works[0].id]).unwrap();
    assert_eq!(deleted, 1);

    // works_tags entry should be cascade-deleted
    let tags = get_tags_for_work(&conn, works[0].id).unwrap();
    assert!(tags.is_empty());
}

// --- update_work ---

#[test]
fn update_work_changes_metadata() {
    let conn = test_conn();
    let record = WorkRecord {
        library_id: TEST_LIBRARY_ID,
        title: "Original",
        path: "/orig.jpg",
        work_type: "image",
        page_count: 1,
        thumbnail: b"thumb",
        artist: Some("Old Artist"),
        year: Some(2020),
        genre: Some("Old Genre"),
        circle: Some("Old Circle"),
        origin: Some("Old Origin"),
    };
    insert_work(&conn, &record).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let id = works[0].id;

    update_work(
        &conn,
        id,
        "Updated Title",
        Some("New Artist"),
        Some(2025),
        Some("New Genre"),
        Some("New Circle"),
        Some("New Origin"),
    )
    .unwrap();

    let detail = get_work(&conn, id).unwrap();
    assert_eq!(detail.title, "Updated Title");
    assert_eq!(detail.artist.as_deref(), Some("New Artist"));
    assert_eq!(detail.year, Some(2025));
    assert_eq!(detail.genre.as_deref(), Some("New Genre"));
    assert_eq!(detail.circle.as_deref(), Some("New Circle"));
    assert_eq!(detail.origin.as_deref(), Some("New Origin"));
}

#[test]
fn update_work_not_found() {
    let conn = test_conn();
    let result = update_work(&conn, 9999, "Title", None, None, None, None, None);
    assert!(matches!(result, Err(AppError::NotFound)));
}

#[test]
fn update_work_clears_optional_fields() {
    let conn = test_conn();
    let record = WorkRecord {
        library_id: TEST_LIBRARY_ID,
        title: "Work",
        path: "/w.jpg",
        work_type: "image",
        page_count: 1,
        thumbnail: b"thumb",
        artist: Some("Artist"),
        year: Some(2024),
        genre: Some("Genre"),
        circle: Some("Circle"),
        origin: Some("Origin"),
    };
    insert_work(&conn, &record).unwrap();
    let works = list_works(&conn, TEST_LIBRARY_ID, "title", "asc").unwrap();
    let id = works[0].id;

    update_work(&conn, id, "Work", None, None, None, None, None).unwrap();

    let detail = get_work(&conn, id).unwrap();
    assert_eq!(detail.title, "Work");
    assert_eq!(detail.artist, None);
    assert_eq!(detail.year, None);
    assert_eq!(detail.genre, None);
    assert_eq!(detail.circle, None);
    assert_eq!(detail.origin, None);
}

// --- database is locked reproduction ---

#[test]
fn open_db_succeeds_despite_brief_external_lock() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().to_path_buf();

    let db_path = dir.join("sharaku.db");

    let blocker = Connection::open(&db_path).unwrap();
    blocker.execute_batch("BEGIN EXCLUSIVE").unwrap();

    let dir_clone = dir.clone();
    let handle = std::thread::spawn(move || open_db(&dir_clone));

    std::thread::sleep(std::time::Duration::from_millis(200));
    blocker.execute_batch("ROLLBACK").unwrap();
    drop(blocker);

    let result = handle.join().unwrap();
    assert!(
        result.is_ok(),
        "open_db should succeed after lock is released, but got: {:?}",
        result.err()
    );

    let conn = result.unwrap();
    library::add_library(&conn, "Test", Some("/test")).unwrap();
    let lib_id = library::active_library(&conn).unwrap().unwrap().id;
    let works = list_works(&conn, &lib_id, "title", "asc").unwrap();
    assert_eq!(works.len(), 0);

    drop(conn);
}
