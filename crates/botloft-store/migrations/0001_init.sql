-- Crews, bots and daemon settings (spec 12). Times are Unix milliseconds.
-- Archiving is logical: archived_at is set and rows are kept.

CREATE TABLE crews (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    paused      INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    archived_at INTEGER
);

CREATE TABLE bots (
    id           TEXT PRIMARY KEY,
    crew_id      TEXT NOT NULL REFERENCES crews (id),
    name         TEXT NOT NULL,
    handle       TEXT NOT NULL,
    slug         TEXT NOT NULL,
    role         TEXT NOT NULL,
    instructions TEXT NOT NULL,
    color        TEXT NOT NULL,
    paused       INTEGER NOT NULL DEFAULT 0,
    -- SHA-256 of the current generation's bot token; set by the runtime (M2).
    token_hash   TEXT,
    created_at   INTEGER NOT NULL,
    archived_at  INTEGER,
    UNIQUE (crew_id, slug)
);

CREATE INDEX bots_crew ON bots (crew_id);

-- Handles address bots inside a crew, so they are unique among active bots.
CREATE UNIQUE INDEX bots_active_handle ON bots (crew_id, handle) WHERE archived_at IS NULL;

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
