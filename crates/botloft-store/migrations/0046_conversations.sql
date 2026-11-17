-- Several conversations per bot (spec 40, ADR 0004). Every bot has a main
-- one, whose id is `cnv_` with the bot's own ULID; it takes the session the
-- bot resumes, and every chat item and delivery so far.

CREATE TABLE conversations (
    id          TEXT PRIMARY KEY,
    bot_id      TEXT NOT NULL REFERENCES bots (id),
    main        INTEGER NOT NULL DEFAULT 0,
    title       TEXT,
    -- Who named it: 'owner' or 'ai'; NULL while it has no title.
    title_by    TEXT,
    -- The conversation Claude Code has on disk (spec 7.3).
    session_id  TEXT,
    created_at  INTEGER NOT NULL,
    archived_at INTEGER
);

CREATE UNIQUE INDEX conversations_main ON conversations (bot_id) WHERE main = 1;
CREATE INDEX conversations_bot ON conversations (bot_id);

INSERT INTO conversations (id, bot_id, main, session_id, created_at)
SELECT 'cnv_' || substr(id, 5), id, 1, session_id, created_at FROM bots;

ALTER TABLE chat_items ADD COLUMN conversation_id TEXT;
UPDATE chat_items SET conversation_id = 'cnv_' || substr(bot_id, 5);
CREATE INDEX chat_items_conversation ON chat_items (conversation_id);

ALTER TABLE deliveries ADD COLUMN conversation_id TEXT;
UPDATE deliveries SET conversation_id = 'cnv_' || substr(bot_id, 5);
