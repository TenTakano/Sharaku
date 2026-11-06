-- Migration 005: Viewing sessions
-- One row per viewer open/close of a work. Derived values (last viewed, view count,
-- total viewing time) are computed from these rows rather than stored on works.
--
-- ended_at stays NULL while the session is open. Sessions still open at app exit are
-- ended at exit time; ones left open by a crash are closed with ended_at = started_at
-- on the next startup (see history.rs). last_page is updated during the session, so it
-- is the page the user was on when the session was last recorded ("continue reading").

CREATE TABLE IF NOT EXISTS view_sessions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id    INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    started_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ended_at   TEXT,
    last_page  INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_view_sessions_work_id    ON view_sessions(work_id);
CREATE INDEX IF NOT EXISTS idx_view_sessions_started_at ON view_sessions(started_at);
