-- Migration 003: Allow 'folder' work type
-- SQLite does not support ALTER TABLE to modify CHECK constraints,
-- so we recreate the table with the updated constraint.

PRAGMA foreign_keys = OFF;

CREATE TABLE works_new (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT    NOT NULL,
    path       TEXT    NOT NULL UNIQUE,
    type       TEXT    NOT NULL CHECK (type IN ('image', 'pdf', 'archive', 'folder')),
    page_count INTEGER,
    thumbnail  BLOB,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    artist     TEXT,
    year       INTEGER,
    genre      TEXT,
    circle     TEXT,
    origin     TEXT
);

INSERT INTO works_new (id, title, path, type, page_count, thumbnail, created_at, updated_at, artist, year, genre, circle, origin)
    SELECT id, title, path, type, page_count, thumbnail, created_at, updated_at, artist, year, genre, circle, origin
    FROM works;

DROP TABLE works;
ALTER TABLE works_new RENAME TO works;

CREATE INDEX idx_works_type       ON works(type);
CREATE INDEX idx_works_title      ON works(title);
CREATE INDEX idx_works_created_at ON works(created_at);

PRAGMA foreign_keys = ON;
