-- The names of an agent's keys (spec 32.2). Their values live in
-- secrets\keys\<bot_id>.json, never here.
CREATE TABLE agent_keys (
    bot_id      TEXT NOT NULL REFERENCES bots (id),
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    set_at      INTEGER NOT NULL,
    PRIMARY KEY (bot_id, name)
);
