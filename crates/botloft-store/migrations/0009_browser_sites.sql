-- Sites the owner let each bot use in its browser (spec 21.5). A site is a
-- host in lower case without "www."; it also covers its subdomains.
CREATE TABLE browser_sites (
    bot_id     TEXT    NOT NULL REFERENCES bots (id),
    host       TEXT    NOT NULL,
    allowed_at INTEGER NOT NULL,
    PRIMARY KEY (bot_id, host)
);
