-- Which of the owner's claude.ai connectors each agent may use (spec 34.4),
-- by the connector's plain name ("Google Calendar").
CREATE TABLE bot_connectors (
    bot_id TEXT NOT NULL REFERENCES bots (id),
    name   TEXT NOT NULL,
    PRIMARY KEY (bot_id, name)
);
