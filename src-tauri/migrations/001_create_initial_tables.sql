PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS works (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT    NOT NULL,
    path       TEXT    NOT NULL UNIQUE,
    type       TEXT    NOT NULL CHECK (type IN ('image', 'pdf', 'archive')),
    page_count INTEGER,
    thumbnail  BLOB,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_works_type       ON works(type);
CREATE INDEX IF NOT EXISTS idx_works_title      ON works(title);
CREATE INDEX IF NOT EXISTS idx_works_created_at ON works(created_at);

CREATE TABLE IF NOT EXISTS tags (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    name     TEXT NOT NULL,
    category TEXT,
    UNIQUE(name, category)
);

CREATE INDEX IF NOT EXISTS idx_tags_category ON tags(category);
CREATE INDEX IF NOT EXISTS idx_tags_name     ON tags(name);

CREATE TABLE IF NOT EXISTS works_tags (
    work_id INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (work_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_works_tags_tag_id ON works_tags(tag_id);

CREATE TABLE IF NOT EXISTS playlists (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

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
