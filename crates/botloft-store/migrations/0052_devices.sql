-- Devices that reach the daemon with a token of their own, such as the app
-- on another computer through a tunnel (spec 42.6). The token is never
-- kept: only its SHA-256, in hex.
CREATE TABLE devices (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    token_hash   TEXT NOT NULL UNIQUE,
    paired_at    INTEGER NOT NULL,
    last_seen_at INTEGER
);
