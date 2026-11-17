-- Which of the owner's skills each agent may use (spec 38.3), by the name
-- of the skill's folder.
CREATE TABLE bot_skills (
    bot_id TEXT NOT NULL REFERENCES bots (id),
    name   TEXT NOT NULL,
    PRIMARY KEY (bot_id, name)
);
