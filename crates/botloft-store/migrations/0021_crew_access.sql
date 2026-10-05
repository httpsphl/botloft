-- What the owner let a bot reach in another crew for good (spec 10.4): the
-- whole crew, or one of its bots. Access for one turn only stays in memory.
CREATE TABLE crew_access (
    id            TEXT    PRIMARY KEY,
    bot_id        TEXT    NOT NULL REFERENCES bots (id),
    crew_id       TEXT    NOT NULL REFERENCES crews (id),
    target_bot_id TEXT    REFERENCES bots (id),
    talk          INTEGER NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL
);

-- One row per bot and reach: the crew, or one bot of it.
CREATE UNIQUE INDEX crew_access_reach
    ON crew_access (bot_id, crew_id, coalesce(target_bot_id, ''));
