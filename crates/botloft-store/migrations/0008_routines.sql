-- Routines (spec 20.7): a bot working on its own at set times, and each
-- time a routine came due. A routine's messages point back to it.

CREATE TABLE routines (
    id          TEXT PRIMARY KEY,
    bot_id      TEXT NOT NULL REFERENCES bots (id),
    name        TEXT NOT NULL,
    prompt      TEXT NOT NULL,
    -- JSON: {"kind": "weekly" | "interval" | "cron", ...}
    schedule    TEXT NOT NULL,
    timezone    TEXT NOT NULL,
    overlap     TEXT NOT NULL,
    missed      TEXT NOT NULL,
    enabled     INTEGER NOT NULL,
    next_run_at INTEGER,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    archived_at INTEGER
);

CREATE INDEX routines_due ON routines (enabled, next_run_at);
CREATE INDEX routines_bot ON routines (bot_id);

CREATE TABLE routine_runs (
    id            TEXT PRIMARY KEY,
    routine_id    TEXT NOT NULL REFERENCES routines (id),
    scheduled_for INTEGER NOT NULL,
    status        TEXT NOT NULL,
    reason        TEXT,
    skipped_count INTEGER NOT NULL DEFAULT 0,
    message_id    TEXT REFERENCES messages (id),
    created_at    INTEGER NOT NULL,
    finished_at   INTEGER
);

CREATE INDEX routine_runs_routine ON routine_runs (routine_id, id);
CREATE INDEX routine_runs_message ON routine_runs (message_id);

ALTER TABLE messages ADD COLUMN routine_id TEXT REFERENCES routines (id);
