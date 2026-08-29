use super::*;
use crate::library;
use crate::test_common::test_db_with_library;
use rusqlite::Connection;

const TEST_LIBRARY_ID: &str = "test_lib_playlist";

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

fn insert_sample_work(conn: &Connection, title: &str, path: &str) -> i64 {
    insert_work(conn, &sample_record(title, path)).unwrap();
    conn.query_row("SELECT id FROM works WHERE path = ?1", [path], |row| {
        row.get(0)
    })
    .unwrap()
}

#[test]
fn create_and_list_playlists() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "My Playlist").unwrap();
    assert_eq!(playlist.name, "My Playlist");

    let playlists = list_playlists(&conn, TEST_LIBRARY_ID).unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].id, playlist.id);
    assert_eq!(playlists[0].name, "My Playlist");
}

#[test]
fn list_playlists_scoped_to_library() {
    let conn = test_conn();
    let other_lib = library::add_library(&conn, "Other", Some("/other")).unwrap();
    create_playlist(&conn, TEST_LIBRARY_ID, "A").unwrap();
    create_playlist(&conn, &other_lib.id, "B").unwrap();

    let playlists = list_playlists(&conn, TEST_LIBRARY_ID).unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].name, "A");
}

#[test]
fn rename_playlist_updates_name() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "Old").unwrap();
    rename_playlist(&conn, playlist.id, "New").unwrap();

    let playlists = list_playlists(&conn, TEST_LIBRARY_ID).unwrap();
    assert_eq!(playlists[0].name, "New");
}

#[test]
fn rename_playlist_not_found() {
    let conn = test_conn();
    let result = rename_playlist(&conn, 999, "New");
    assert!(matches!(result, Err(AppError::NotFound)));
}

#[test]
fn delete_playlist_removes_it() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "ToDelete").unwrap();
    delete_playlist(&conn, playlist.id).unwrap();

    let playlists = list_playlists(&conn, TEST_LIBRARY_ID).unwrap();
    assert!(playlists.is_empty());
}

#[test]
fn delete_playlist_not_found() {
    let conn = test_conn();
    let result = delete_playlist(&conn, 999);
    assert!(matches!(result, Err(AppError::NotFound)));
}

#[test]
fn delete_playlist_cascades_items() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_id = insert_sample_work(&conn, "A", "/a.jpg");
    add_item_to_playlist(&conn, playlist.id, work_id).unwrap();

    delete_playlist(&conn, playlist.id).unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM playlist_items", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn delete_work_cascades_playlist_items() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_id = insert_sample_work(&conn, "A", "/a.jpg");
    add_item_to_playlist(&conn, playlist.id, work_id).unwrap();

    let deleted = delete_works_by_ids(&conn, TEST_LIBRARY_ID, &[work_id]).unwrap();
    assert_eq!(deleted, 1);

    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert!(items.is_empty());
}

#[test]
fn add_item_assigns_sequential_positions() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");
    let work_b = insert_sample_work(&conn, "B", "/b.jpg");

    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_b).unwrap();

    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].work_id, work_a);
    assert_eq!(items[1].work_id, work_b);
}

#[test]
fn add_item_duplicate_is_ignored() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");

    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();

    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn remove_item_deletes_association() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");
    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();

    remove_item_from_playlist(&conn, playlist.id, work_a).unwrap();

    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert!(items.is_empty());
}

#[test]
fn remove_item_idempotent() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");

    remove_item_from_playlist(&conn, playlist.id, work_a).unwrap();
    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert!(items.is_empty());
}

#[test]
fn add_item_after_removal_reuses_gap_free_max_position() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");
    let work_b = insert_sample_work(&conn, "B", "/b.jpg");
    let work_c = insert_sample_work(&conn, "C", "/c.jpg");

    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_b).unwrap();
    remove_item_from_playlist(&conn, playlist.id, work_b).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_c).unwrap();

    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].work_id, work_a);
    assert_eq!(items[1].work_id, work_c);
}

#[test]
fn reorder_items_applies_new_order() {
    let mut conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");
    let work_b = insert_sample_work(&conn, "B", "/b.jpg");
    let work_c = insert_sample_work(&conn, "C", "/c.jpg");
    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_b).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_c).unwrap();

    reorder_playlist_items(&mut conn, playlist.id, &[work_c, work_a, work_b]).unwrap();

    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert_eq!(
        items.iter().map(|i| i.work_id).collect::<Vec<_>>(),
        vec![work_c, work_a, work_b]
    );
}

#[test]
fn reorder_items_rejects_missing_id() {
    let mut conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");
    let work_b = insert_sample_work(&conn, "B", "/b.jpg");
    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_b).unwrap();

    let result = reorder_playlist_items(&mut conn, playlist.id, &[work_a]);
    assert!(matches!(result, Err(AppError::PlaylistError(_))));

    // The failed call must not have mutated any positions.
    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert_eq!(
        items.iter().map(|i| i.work_id).collect::<Vec<_>>(),
        vec![work_a, work_b]
    );
}

#[test]
fn reorder_items_rejects_invalid_ids() {
    let mut conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");
    let work_b = insert_sample_work(&conn, "B", "/b.jpg");
    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_b).unwrap();

    let cases: [(&str, &[i64]); 2] = [
        ("unknown id", &[work_a, 999999]),
        ("duplicate id", &[work_a, work_a]),
    ];
    for (case, work_ids) in cases {
        let result = reorder_playlist_items(&mut conn, playlist.id, work_ids);
        assert!(
            matches!(result, Err(AppError::PlaylistError(_))),
            "case `{case}` should be rejected"
        );
    }
}

#[test]
fn reorder_then_add_uses_correct_max_position() {
    let mut conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let work_a = insert_sample_work(&conn, "A", "/a.jpg");
    let work_b = insert_sample_work(&conn, "B", "/b.jpg");
    let work_c = insert_sample_work(&conn, "C", "/c.jpg");
    add_item_to_playlist(&conn, playlist.id, work_a).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_b).unwrap();

    reorder_playlist_items(&mut conn, playlist.id, &[work_b, work_a]).unwrap();
    add_item_to_playlist(&conn, playlist.id, work_c).unwrap();

    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert_eq!(
        items.iter().map(|i| i.work_id).collect::<Vec<_>>(),
        vec![work_b, work_a, work_c]
    );
}

#[test]
fn get_playlist_items_empty() {
    let conn = test_conn();
    let playlist = create_playlist(&conn, TEST_LIBRARY_ID, "P").unwrap();
    let items = get_playlist_items(&conn, playlist.id).unwrap();
    assert!(items.is_empty());
}
