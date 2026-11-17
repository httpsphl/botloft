-- Which of the owner's Claude Code plugins each agent may use (spec 41.3),
-- by the plugin's name, and the ones every agent on Claude Code uses, the
-- ones made later included. Giving all of them is a setting of its own
-- (`all_plugins_everyone`).
CREATE TABLE bot_plugins (
    bot_id TEXT NOT NULL REFERENCES bots (id),
    name   TEXT NOT NULL,
    PRIMARY KEY (bot_id, name)
);

CREATE TABLE plugins_everyone (
    name TEXT PRIMARY KEY
);
