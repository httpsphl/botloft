-- Headless bots with a chat (ADR 0001; spec 8, 9.1, 9.5, 10.1 and 12).
-- Times are Unix milliseconds; rowid keeps the insertion order.

-- The Claude Code conversation a bot resumes (spec 7.3).
ALTER TABLE bots ADD COLUMN session_id TEXT;

-- Which process a delivery was written to, the uuid it went with, and when
-- the bot began the turn for it (spec 9.1).
ALTER TABLE deliveries ADD COLUMN sent_generation INTEGER;
ALTER TABLE deliveries ADD COLUMN turn_uuid TEXT;
ALTER TABLE deliveries ADD COLUMN read_at INTEGER;

CREATE INDEX deliveries_turn_uuid ON deliveries (turn_uuid) WHERE turn_uuid IS NOT NULL;
CREATE INDEX deliveries_unread ON deliveries (bot_id, sent_generation)
    WHERE state = 'sent' AND read_at IS NULL;

CREATE TABLE attachments (
    id         TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages (id),
    name       TEXT NOT NULL,
    media_type TEXT NOT NULL,
    size       INTEGER NOT NULL,
    -- Relative to the bot's workspace, with forward slashes.
    path       TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX attachments_message ON attachments (message_id);

CREATE TABLE chat_items (
    id         TEXT PRIMARY KEY,
    bot_id     TEXT NOT NULL REFERENCES bots (id),
    kind       TEXT NOT NULL,
    -- The item's body (protocol ChatBody) as JSON.
    data       TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX chat_items_bot ON chat_items (bot_id);

CREATE TABLE approvals (
    id           TEXT PRIMARY KEY,
    bot_id       TEXT NOT NULL REFERENCES bots (id),
    chat_item_id TEXT NOT NULL REFERENCES chat_items (id),
    tool_use_id  TEXT NOT NULL,
    tool_name    TEXT NOT NULL,
    summary      TEXT NOT NULL,
    input        TEXT NOT NULL,
    status       TEXT NOT NULL,
    note         TEXT,
    created_at   INTEGER NOT NULL,
    answered_at  INTEGER
);

CREATE INDEX approvals_pending ON approvals (bot_id) WHERE status = 'pending';
