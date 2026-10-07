-- The sealed copies an account keeps (spec 27.4). The bytes live in the
-- CopyStore under `<account_id>/<id>`; this is only what the server knows.
CREATE TABLE copies (
    id         TEXT PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    size       INTEGER NOT NULL,
    sha256     TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX copies_by_account ON copies(account_id, created_at);

-- A request to delete an account, waiting for the owner to press the button
-- in the e-mail (spec 27.3).
CREATE TABLE deletions (
    code_hash  TEXT PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    locale     TEXT NOT NULL,
    expires_at INTEGER NOT NULL
);
