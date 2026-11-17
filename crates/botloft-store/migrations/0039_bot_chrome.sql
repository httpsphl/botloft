-- The agents the owner gave their own Chrome to (spec 37.2), with when they
-- gave it and when they accepted the risks for it.
CREATE TABLE bot_chrome (
    bot_id            TEXT PRIMARY KEY REFERENCES bots (id),
    given_at          INTEGER NOT NULL,
    accepted_risks_at INTEGER NOT NULL
);
