-- What the owner let each bot see and do on their desktop (spec 24.2,
-- 24.11): one app, by its executable, or the whole desktop; to see, or to
-- see and act; with or without the real mouse and keyboard, and while the
-- owner is away.
CREATE TABLE desktop_grants (
    id                TEXT    PRIMARY KEY,
    bot_id            TEXT    NOT NULL REFERENCES bots (id),
    scope             TEXT    NOT NULL CHECK (scope IN ('app', 'desktop')),
    app_path          TEXT,
    app_name          TEXT,
    level             TEXT    NOT NULL CHECK (level IN ('see', 'act')),
    real_input        INTEGER NOT NULL DEFAULT 0,
    unattended        INTEGER NOT NULL DEFAULT 0,
    accepted_risks_at INTEGER,
    created_at        INTEGER NOT NULL
);

-- One grant per bot and reach: an app by its path, case aside, or the
-- whole desktop.
CREATE UNIQUE INDEX desktop_grants_reach
    ON desktop_grants (bot_id, scope, lower(coalesce(app_path, '')));
