-- One row per e-mail address; the account is born when the first sign-in link
-- is confirmed (spec 27.3).
CREATE TABLE accounts (
    id         INTEGER PRIMARY KEY,
    email      TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL
);

-- A computer or phone that signed in. Only the SHA-256 of its token is kept.
CREATE TABLE devices (
    id           TEXT PRIMARY KEY,
    account_id   INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    token_hash   TEXT NOT NULL UNIQUE,
    created_at   INTEGER NOT NULL,
    last_used_at INTEGER NOT NULL
);
CREATE INDEX devices_by_account ON devices(account_id);

-- A sign-in waiting for the link. `account_id` is set when the link is
-- confirmed; the app's next poll turns it into a device and the row goes away.
CREATE TABLE logins (
    request_hash TEXT PRIMARY KEY,
    code_hash    TEXT NOT NULL UNIQUE,
    email        TEXT NOT NULL,
    device_name  TEXT NOT NULL,
    locale       TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER NOT NULL,
    account_id   INTEGER REFERENCES accounts(id) ON DELETE CASCADE
);

-- What the rate limits count (27.3): `kind` and a hashed `key`.
CREATE TABLE attempts (
    kind TEXT NOT NULL,
    key  TEXT NOT NULL,
    at   INTEGER NOT NULL
);
CREATE INDEX attempts_by_key ON attempts(kind, key, at);
