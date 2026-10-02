-- What the owner let each bot do without asking again (spec 10.1): a tool
-- and, for a command, site or file, which one (`value`, empty for a tool).
CREATE TABLE allow_rules (
    id         TEXT    PRIMARY KEY,
    bot_id     TEXT    NOT NULL REFERENCES bots (id),
    tool_name  TEXT    NOT NULL,
    kind       TEXT    NOT NULL,
    value      TEXT    NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (bot_id, tool_name, kind, value)
);
