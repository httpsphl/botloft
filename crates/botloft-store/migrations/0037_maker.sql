-- The Maker (spec 36): the crew Botloft keeps for it, and what it built in
-- one go, so the owner can undo it. No text of the owner's history is kept.
ALTER TABLE crews ADD COLUMN system TEXT;

CREATE TABLE builds (
    id          TEXT PRIMARY KEY,
    bot_id      TEXT NOT NULL,
    approval_id TEXT,
    title       TEXT NOT NULL,
    crew_ids    TEXT NOT NULL,
    applied_at  INTEGER NOT NULL,
    undone_at   INTEGER
);
