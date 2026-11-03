-- The sites the owner signed in to once for the agents (spec 35), and which
-- agents have each login. No cookie is kept here: they stay in the browser
-- profiles.
CREATE TABLE site_logins (
    site      TEXT PRIMARY KEY,
    signed_at INTEGER NOT NULL
);

CREATE TABLE bot_site_logins (
    bot_id   TEXT NOT NULL REFERENCES bots (id),
    site     TEXT NOT NULL REFERENCES site_logins (site),
    given_at INTEGER NOT NULL,
    PRIMARY KEY (bot_id, site)
);
