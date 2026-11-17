-- Files a phone sends to its computer (spec 28.14). The phone seals each one
-- with a key of its own, which only the computer gets, inside the relay; the
-- server keeps the sealed bytes until the computer takes them, two hours at
-- most. A row goes with the phone, the computer or the account, and whoever
-- removes it removes the bytes too.
CREATE TABLE phone_files (
    id         TEXT PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    phone      TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    computer   TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    size       INTEGER NOT NULL,
    sha256     TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);
CREATE INDEX phone_files_by_phone ON phone_files(phone);
CREATE INDEX phone_files_by_computer ON phone_files(computer);
CREATE INDEX phone_files_by_expiry ON phone_files(expires_at);
