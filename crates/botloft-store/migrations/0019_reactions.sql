-- The owner's reactions to what a bot wrote (spec 8.9): one per reply,
-- with the reply's text as quoted to the bot, and the owner's message that
-- took it there (null while it waits for the next one).

CREATE TABLE reactions (
    item_id    TEXT PRIMARY KEY REFERENCES chat_items (id),
    bot_id     TEXT NOT NULL REFERENCES bots (id),
    emoji      TEXT NOT NULL,
    quote      TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    sent_in    TEXT REFERENCES messages (id)
);

CREATE INDEX reactions_bot ON reactions (bot_id, sent_in);
CREATE INDEX reactions_sent_in ON reactions (sent_in) WHERE sent_in IS NOT NULL;
