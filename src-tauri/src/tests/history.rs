use super::*;
use crate::db::{self, WorkRecord};
use crate::library;
use crate::test_common::test_db_with_library;

const TEST_LIBRARY_ID: &str = "test_lib_history";

fn insert_work_in(conn: &Connection, library_id: &str, title: &str, path: &str) -> i64 {
    db::insert_work(
        conn,
        &WorkRecord {
            library_id,
            title,
            path,
            work_type: "folder",
            page_count: 10,
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

fn insert_work(conn: &Connection, title: &str) -> i64 {
    insert_work_in(conn, TEST_LIBRARY_ID, title, &format!("/test/{title}"))
}

/// Inserts a session with explicit timestamps; `now` defaults would tie within a test.
fn insert_session(
    conn: &Connection,
    work_id: i64,
    started_at: &str,
    ended_at: Option<&str>,
    last_page: i64,
) -> i64 {
    conn.execute(
        "INSERT INTO view_sessions (work_id, started_at, ended_at, last_page) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![work_id, started_at, ended_at, last_page],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn ids(works: &[WorkWithHistory]) -> Vec<i64> {
    works.iter().map(|w| w.id).collect()
}

#[test]
fn start_and_end_session_records_row() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let work_id = insert_work(&conn, "A");

    let session_id = start_view_session(&conn, TEST_LIBRARY_ID, work_id, 2).unwrap();
    let (ended_at, last_page): (Option<String>, i64) = conn
        .query_row(
            "SELECT ended_at, last_page FROM view_sessions WHERE id = ?1",
            [session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(ended_at, None);
    assert_eq!(last_page, 2);

    update_view_session_page(&conn, session_id, 4).unwrap();
    end_view_session(&conn, session_id, 5).unwrap();
    let (started_at, ended_at, last_page): (String, Option<String>, i64) = conn
        .query_row(
            "SELECT started_at, ended_at, last_page FROM view_sessions WHERE id = ?1",
            [session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    let ended_at = ended_at.expect("ended_at should be set");
    assert!(ended_at >= started_at);
    assert_eq!(last_page, 5);
}

#[test]
fn start_session_rejects_work_outside_library() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let other_lib = library::add_library(&conn, "Other", Some("/other")).unwrap();
    let other_work = insert_work_in(&conn, &other_lib.id, "X", "/other/X");

    assert!(matches!(
        start_view_session(&conn, TEST_LIBRARY_ID, other_work, 0),
        Err(AppError::NotFound)
    ));
    assert!(matches!(
        start_view_session(&conn, TEST_LIBRARY_ID, 9999, 0),
        Err(AppError::NotFound)
    ));
}

#[test]
fn update_and_end_unknown_session_return_not_found() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    assert!(matches!(
        update_view_session_page(&conn, 42, 1),
        Err(AppError::NotFound)
    ));
    assert!(matches!(
        end_view_session(&conn, 42, 1),
        Err(AppError::NotFound)
    ));
}

#[test]
fn recently_viewed_orders_by_latest_activity() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let a = insert_work(&conn, "A");
    let b = insert_work(&conn, "B");
    let c = insert_work(&conn, "C");
    let _unviewed = insert_work(&conn, "D");

    insert_session(
        &conn,
        a,
        "2026-01-01T10:00:00.000Z",
        Some("2026-01-01T10:05:00.000Z"),
        0,
    );
    insert_session(
        &conn,
        b,
        "2026-01-03T10:00:00.000Z",
        Some("2026-01-03T10:01:00.000Z"),
        0,
    );
    insert_session(
        &conn,
        a,
        "2026-01-02T10:00:00.000Z",
        Some("2026-01-02T10:01:00.000Z"),
        0,
    );
    // Never closed: its start time counts as the activity time.
    insert_session(&conn, c, "2026-01-04T10:00:00.000Z", None, 0);

    let works = list_recently_viewed_works(&conn, TEST_LIBRARY_ID, 10).unwrap();
    assert_eq!(ids(&works), vec![c, b, a]);
    assert_eq!(
        works[2].last_viewed_at.as_deref(),
        Some("2026-01-02T10:01:00.000Z")
    );

    let limited = list_recently_viewed_works(&conn, TEST_LIBRARY_ID, 2).unwrap();
    assert_eq!(ids(&limited), vec![c, b]);
}

#[test]
fn unviewed_lists_only_works_without_sessions() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let a = insert_work(&conn, "A");
    let b = insert_work(&conn, "B");
    let c = insert_work(&conn, "C");
    insert_session(&conn, b, "2026-01-01T10:00:00.000Z", None, 0);

    let works = list_unviewed_works(&conn, TEST_LIBRARY_ID, 10).unwrap();
    let mut got = ids(&works);
    got.sort_unstable();
    assert_eq!(got, vec![a, c]);
    assert!(works
        .iter()
        .all(|w| w.view_count == 0 && w.last_viewed_at.is_none() && w.last_page.is_none()));
}

#[test]
fn frequently_viewed_ranks_by_session_count_then_recency() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let a = insert_work(&conn, "A");
    let b = insert_work(&conn, "B");
    let c = insert_work(&conn, "C");

    for day in 1..=3 {
        insert_session(&conn, b, &format!("2026-01-0{day}T10:00:00.000Z"), None, 0);
    }
    insert_session(&conn, a, "2026-01-01T10:00:00.000Z", None, 0);
    insert_session(&conn, c, "2026-01-05T10:00:00.000Z", None, 0);

    let works = list_frequently_viewed_works(&conn, TEST_LIBRARY_ID, 10).unwrap();
    assert_eq!(ids(&works), vec![b, c, a]);
    assert_eq!(works[0].view_count, 3);
}

#[test]
fn history_queries_are_scoped_to_library() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let other_lib = library::add_library(&conn, "Other", Some("/other")).unwrap();
    let mine = insert_work(&conn, "Mine");
    let theirs = insert_work_in(&conn, &other_lib.id, "Theirs", "/other/Theirs");
    let _theirs_unviewed = insert_work_in(&conn, &other_lib.id, "T2", "/other/T2");
    insert_session(&conn, mine, "2026-01-01T10:00:00.000Z", None, 0);
    insert_session(&conn, theirs, "2026-01-02T10:00:00.000Z", None, 0);

    assert_eq!(
        ids(&list_recently_viewed_works(&conn, TEST_LIBRARY_ID, 10).unwrap()),
        vec![mine]
    );
    assert_eq!(
        ids(&list_frequently_viewed_works(&conn, TEST_LIBRARY_ID, 10).unwrap()),
        vec![mine]
    );
    assert!(list_unviewed_works(&conn, TEST_LIBRARY_ID, 10)
        .unwrap()
        .is_empty());
}

#[test]
fn work_history_aggregates_sessions() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let a = insert_work(&conn, "A");

    let never = get_work_history(&conn, a).unwrap();
    assert_eq!(never.view_count, 0);
    assert_eq!(never.last_viewed_at, None);
    assert_eq!(never.last_page, None);
    assert_eq!(never.total_view_seconds, 0);

    insert_session(
        &conn,
        a,
        "2026-01-01T10:00:00.000Z",
        Some("2026-01-01T10:01:30.000Z"),
        7,
    );
    insert_session(
        &conn,
        a,
        "2026-01-02T10:00:00.000Z",
        Some("2026-01-02T10:00:30.000Z"),
        3,
    );
    // Unclosed session: counted as a view, contributes no duration, and is the latest.
    insert_session(&conn, a, "2026-01-03T10:00:00.000Z", None, 5);

    let stats = get_work_history(&conn, a).unwrap();
    assert_eq!(stats.view_count, 3);
    assert_eq!(
        stats.last_viewed_at.as_deref(),
        Some("2026-01-03T10:00:00.000Z")
    );
    assert_eq!(stats.last_page, Some(5));
    assert_eq!(stats.total_view_seconds, 120);
}

#[test]
fn work_history_of_unknown_work_is_not_found() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    assert!(matches!(
        get_work_history(&conn, 9999),
        Err(AppError::NotFound)
    ));
}

#[test]
fn deleting_work_removes_its_sessions() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let a = insert_work(&conn, "A");
    insert_session(&conn, a, "2026-01-01T10:00:00.000Z", None, 0);

    db::delete_works_by_ids(&conn, TEST_LIBRARY_ID, &[a]).unwrap();
    let remaining: i64 = conn
        .query_row("SELECT COUNT(*) FROM view_sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(remaining, 0);
}

#[test]
fn sessions_persist_across_reopen() {
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let work_id = {
        let conn = db::open_db(tmp_dir.path()).unwrap();
        let lib = library::add_library(&conn, "Lib", Some("/lib")).unwrap();
        let work_id = insert_work_in(&conn, &lib.id, "A", "/lib/A");
        let session_id = start_view_session(&conn, &lib.id, work_id, 0).unwrap();
        end_view_session(&conn, session_id, 3).unwrap();
        work_id
    };

    let conn = db::open_db(tmp_dir.path()).unwrap();
    let stats = get_work_history(&conn, work_id).unwrap();
    assert_eq!(stats.view_count, 1);
    assert_eq!(stats.last_page, Some(3));
}

fn session_end(conn: &Connection, session_id: i64) -> (String, Option<String>, i64) {
    conn.query_row(
        "SELECT started_at, ended_at, last_page FROM view_sessions WHERE id = ?1",
        [session_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .unwrap()
}

#[test]
fn end_open_sessions_closes_only_unclosed_sessions() {
    let conn = test_db_with_library(TEST_LIBRARY_ID);
    let a = insert_work(&conn, "A");
    let closed = insert_session(
        &conn,
        a,
        "2026-01-01T10:00:00.000Z",
        Some("2026-01-01T10:01:00.000Z"),
        1,
    );
    let open = start_view_session(&conn, TEST_LIBRARY_ID, a, 0).unwrap();
    update_view_session_page(&conn, open, 4).unwrap();

    assert_eq!(end_open_view_sessions(&conn).unwrap(), 1);

    let (_, closed_end, _) = session_end(&conn, closed);
    assert_eq!(closed_end.as_deref(), Some("2026-01-01T10:01:00.000Z"));
    let (started_at, open_end, last_page) = session_end(&conn, open);
    assert!(open_end.expect("open session should be ended") >= started_at);
    assert_eq!(last_page, 4);
}

#[test]
fn reopening_db_closes_orphaned_sessions_with_zero_duration() {
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let (lib_id, work_id, orphan) = {
        let conn = db::open_db(tmp_dir.path()).unwrap();
        let lib = library::add_library(&conn, "Lib", Some("/lib")).unwrap();
        let work_id = insert_work_in(&conn, &lib.id, "A", "/lib/A");
        let orphan = insert_session(&conn, work_id, "2026-01-01T10:00:00.000Z", None, 6);
        (lib.id, work_id, orphan)
    };

    let conn = db::open_db(tmp_dir.path()).unwrap();
    let (_, ended_at, last_page) = session_end(&conn, orphan);
    assert_eq!(ended_at.as_deref(), Some("2026-01-01T10:00:00.000Z"));
    assert_eq!(last_page, 6);

    // Sessions started after the database is opened are left open.
    let current = start_view_session(&conn, &lib_id, work_id, 0).unwrap();
    assert_eq!(session_end(&conn, current).1, None);

    let stats = get_work_history(&conn, work_id).unwrap();
    assert_eq!(stats.view_count, 2);
    assert_eq!(stats.total_view_seconds, 0);
}
