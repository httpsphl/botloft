-- The history of each bot's memory and instructions (spec 8.10): every
-- version as it was, who made it, and the version an Undo went back to.
CREATE TABLE bot_versions (
    id            TEXT PRIMARY KEY,
    bot_id        TEXT NOT NULL REFERENCES bots(id),
    kind          TEXT NOT NULL,
    author        TEXT NOT NULL,
    text          TEXT NOT NULL,
    restored_from TEXT,
    created_at    INTEGER NOT NULL
);

CREATE INDEX bot_versions_bot ON bot_versions (bot_id, kind);
