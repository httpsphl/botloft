-- Summaries of the owner's history made for the Maker (spec 36.7): what
-- they covered and what was swapped out, never their text. The text waits
-- in <home>\imports\ until it is sent or forgotten.
CREATE TABLE imports (
    id              TEXT PRIMARY KEY,
    source          TEXT NOT NULL,
    created_at      INTEGER NOT NULL,
    sent_at         INTEGER,
    sent_message_id TEXT,
    scope           TEXT NOT NULL,
    chars           INTEGER NOT NULL,
    redactions      TEXT NOT NULL
);
