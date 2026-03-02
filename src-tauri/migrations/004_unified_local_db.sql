-- Migration 004: Unified local database
-- All data is stored in a single DB in app_data_dir, keyed by library_id.

CREATE TABLE IF NOT EXISTS libraries (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    path       TEXT UNIQUE,
    is_active  INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS works (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id TEXT    NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    title      TEXT    NOT NULL,
    path       TEXT    NOT NULL,
    type       TEXT    NOT NULL CHECK (type IN ('image', 'pdf', 'archive', 'folder')),
    page_count INTEGER,
    thumbnail  BLOB,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    artist     TEXT,
    year       INTEGER,
    genre      TEXT,
    circle     TEXT,
    origin     TEXT,
    UNIQUE(library_id, path)
);

CREATE INDEX IF NOT EXISTS idx_works_library_id ON works(library_id);
CREATE INDEX IF NOT EXISTS idx_works_type       ON works(type);
CREATE INDEX IF NOT EXISTS idx_works_title      ON works(title);
CREATE INDEX IF NOT EXISTS idx_works_created_at ON works(created_at);

CREATE TABLE IF NOT EXISTS tags (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    category   TEXT,
    UNIQUE(library_id, name, category)
);

CREATE INDEX IF NOT EXISTS idx_tags_library_id ON tags(library_id);
CREATE INDEX IF NOT EXISTS idx_tags_category   ON tags(category);
CREATE INDEX IF NOT EXISTS idx_tags_name       ON tags(name);

CREATE TABLE IF NOT EXISTS works_tags (
    work_id INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (work_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_works_tags_tag_id ON works_tags(tag_id);

CREATE TABLE IF NOT EXISTS settings (
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    key        TEXT NOT NULL,
    value      TEXT NOT NULL,
    PRIMARY KEY (library_id, key)
);

CREATE TABLE IF NOT EXISTS playlists (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_playlists_library_id ON playlists(library_id);

CREATE TABLE IF NOT EXISTS playlist_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    work_id     INTEGER NOT NULL REFERENCES works(id)     ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    UNIQUE(playlist_id, work_id),
    UNIQUE(playlist_id, position)
);

CREATE INDEX IF NOT EXISTS idx_playlist_items_work_id  ON playlist_items(work_id);
CREATE INDEX IF NOT EXISTS idx_playlist_items_position ON playlist_items(playlist_id, position);
