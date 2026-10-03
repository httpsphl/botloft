-- Questions a bot asks the owner without waiting (spec 23.7). The answer
-- goes to the bot as a message that points back to its question.

CREATE TABLE questions (
    id           TEXT PRIMARY KEY,
    crew_id      TEXT NOT NULL REFERENCES crews (id),
    bot_id       TEXT NOT NULL REFERENCES bots (id),
    chat_item_id TEXT NOT NULL REFERENCES chat_items (id),
    text         TEXT NOT NULL,
    -- JSON array of strings; empty for a free answer only.
    options      TEXT NOT NULL,
    status       TEXT NOT NULL,
    answer       TEXT,
    created_at   INTEGER NOT NULL,
    answered_at  INTEGER
);

CREATE INDEX questions_status ON questions (status, created_at);
CREATE INDEX questions_bot ON questions (bot_id);
CREATE INDEX questions_crew ON questions (crew_id);
CREATE INDEX questions_chat_item ON questions (chat_item_id);

ALTER TABLE messages ADD COLUMN question_id TEXT REFERENCES questions (id);
CREATE INDEX messages_question ON messages (question_id);
