//! Viewing history: one `view_sessions` row per viewer open/close of a work.
//!
//! Derived values (last viewed, view count, total viewing time, last page) are
//! computed from the session rows at query time instead of being stored on `works`,
//! so the meaning of a "view" can be refined later without replacing the data.

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use crate::error::AppError;

/// A work together with its aggregated viewing history.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkWithHistory {
    pub id: i64,
    pub title: String,
    pub work_type: String,
    pub page_count: i32,
    pub created_at: String,
    /// Latest activity time over all sessions (`ended_at`, or `started_at` for
    /// sessions that were never closed). `None` when the work has never been viewed.
    pub last_viewed_at: Option<String>,
    pub view_count: i64,
    /// Page recorded in the most recent session (for "continue reading").
    pub last_page: Option<i64>,
    /// Sum of closed sessions' durations. Unclosed sessions count as zero.
    pub total_view_seconds: i64,
}

const HISTORY_SELECT: &str = "\
WITH stats AS (
    SELECT work_id,
           COUNT(*) AS view_count,
           MAX(COALESCE(ended_at, started_at)) AS last_viewed_at,
           MAX(id) AS last_session_id,
           CAST(ROUND(COALESCE(SUM((julianday(ended_at) - julianday(started_at)) * 86400), 0)) AS INTEGER)
               AS total_view_seconds
    FROM view_sessions
    GROUP BY work_id
)
SELECT w.id, w.title, w.type, w.page_count, w.created_at,
       s.last_viewed_at, COALESCE(s.view_count, 0), ls.last_page,
       COALESCE(s.total_view_seconds, 0)
FROM works w
LEFT JOIN stats s ON s.work_id = w.id
LEFT JOIN view_sessions ls ON ls.id = s.last_session_id";

fn map_work_with_history_row(row: &rusqlite::Row) -> rusqlite::Result<WorkWithHistory> {
    Ok(WorkWithHistory {
        id: row.get(0)?,
        title: row.get(1)?,
        work_type: row.get(2)?,
        page_count: row.get(3)?,
        created_at: row.get(4)?,
        last_viewed_at: row.get(5)?,
        view_count: row.get(6)?,
        last_page: row.get(7)?,
        total_view_seconds: row.get(8)?,
    })
}

/// Runs `HISTORY_SELECT` for one library. `filter` and `order` are fixed SQL
/// fragments chosen by the callers in this module, never user input.
fn query_library_history(
    conn: &Connection,
    library_id: &str,
    filter: &str,
    order: &str,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, AppError> {
    let sql =
        format!("{HISTORY_SELECT} WHERE w.library_id = ?1 AND {filter} ORDER BY {order} LIMIT ?2");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params![library_id, limit],
        map_work_with_history_row,
    )?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Opens a session for a work in `library_id` and returns its id.
pub fn start_view_session(
    conn: &Connection,
    library_id: &str,
    work_id: i64,
    page: i64,
) -> Result<i64, AppError> {
    let rows = conn.execute(
        "INSERT INTO view_sessions (work_id, last_page) \
         SELECT id, ?3 FROM works WHERE id = ?1 AND library_id = ?2",
        rusqlite::params![work_id, library_id, page],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(conn.last_insert_rowid())
}

/// Records the page currently shown, so the position survives an app exit
/// without the viewer being closed.
pub fn update_view_session_page(
    conn: &Connection,
    session_id: i64,
    page: i64,
) -> Result<(), AppError> {
    let rows = conn.execute(
        "UPDATE view_sessions SET last_page = ?1 WHERE id = ?2",
        rusqlite::params![page, session_id],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Closes a session, recording the end time and the final page.
pub fn end_view_session(conn: &Connection, session_id: i64, page: i64) -> Result<(), AppError> {
    let rows = conn.execute(
        "UPDATE view_sessions \
         SET ended_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), last_page = ?1 \
         WHERE id = ?2",
        rusqlite::params![page, session_id],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Closes every session still open, ending it now. Called when the app exits
/// with the viewer still open, where the frontend gets no chance to close it.
pub fn end_open_view_sessions(conn: &Connection) -> Result<usize, AppError> {
    Ok(conn.execute(
        "UPDATE view_sessions \
         SET ended_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE ended_at IS NULL",
        [],
    )?)
}

/// Closes sessions left open by a previous run that did not exit cleanly (crash,
/// killed process). Their real end time is unknown, so they are closed with zero
/// duration (`ended_at = started_at`); `last_page` keeps the last recorded page.
/// Called when the database is opened, before any session of this run starts.
pub fn close_orphaned_view_sessions(conn: &Connection) -> Result<usize, AppError> {
    Ok(conn.execute(
        "UPDATE view_sessions SET ended_at = started_at WHERE ended_at IS NULL",
        [],
    )?)
}

/// Viewed works, most recently viewed first.
pub fn list_recently_viewed_works(
    conn: &Connection,
    library_id: &str,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, AppError> {
    query_library_history(
        conn,
        library_id,
        "s.work_id IS NOT NULL",
        "s.last_viewed_at DESC, w.id DESC",
        limit,
    )
}

/// Works that have never been viewed, newest registrations first.
pub fn list_unviewed_works(
    conn: &Connection,
    library_id: &str,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, AppError> {
    query_library_history(
        conn,
        library_id,
        "s.work_id IS NULL",
        "w.created_at DESC, w.id DESC",
        limit,
    )
}

/// Viewed works ranked by session count, ties broken by recency.
pub fn list_frequently_viewed_works(
    conn: &Connection,
    library_id: &str,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, AppError> {
    query_library_history(
        conn,
        library_id,
        "s.work_id IS NOT NULL",
        "s.view_count DESC, s.last_viewed_at DESC, w.id DESC",
        limit,
    )
}

/// Works in registration order, newest first, whether viewed or not.
pub fn list_recently_added_works(
    conn: &Connection,
    library_id: &str,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, AppError> {
    query_library_history(
        conn,
        library_id,
        "1 = 1",
        "w.created_at DESC, w.id DESC",
        limit,
    )
}

/// Works viewed at least `PICKUP_MIN_VIEWS` times but not in the last
/// `PICKUP_IDLE_DAYS` days ("よく見ていたのに、しばらく開いていない"), the longest
/// untouched first, ties broken by view count.
///
/// A deliberately simple rule for the Home pickup shelf; a real rediscovery score
/// replaces it later (#355).
pub fn list_pickup_works(
    conn: &Connection,
    library_id: &str,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, AppError> {
    let filter = format!(
        "s.view_count >= {PICKUP_MIN_VIEWS} \
         AND s.last_viewed_at < strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-{PICKUP_IDLE_DAYS} days')"
    );
    query_library_history(
        conn,
        library_id,
        &filter,
        "s.last_viewed_at ASC, s.view_count DESC, w.id DESC",
        limit,
    )
}

const PICKUP_MIN_VIEWS: u32 = 2;
const PICKUP_IDLE_DAYS: u32 = 14;

/// A random sample of the library's works (a new sample on every call).
pub fn list_random_works(
    conn: &Connection,
    library_id: &str,
    limit: u32,
) -> Result<Vec<WorkWithHistory>, AppError> {
    query_library_history(conn, library_id, "1 = 1", "RANDOM()", limit)
}

/// Viewing history of a single work (zero/None values if never viewed).
pub fn get_work_history(conn: &Connection, work_id: i64) -> Result<WorkWithHistory, AppError> {
    let sql = format!("{HISTORY_SELECT} WHERE w.id = ?1");
    conn.query_row(&sql, [work_id], map_work_with_history_row)
        .optional()?
        .ok_or(AppError::NotFound)
}

#[cfg(test)]
#[path = "tests/history.rs"]
mod tests;
