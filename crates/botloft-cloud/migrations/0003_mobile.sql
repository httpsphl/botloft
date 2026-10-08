-- The phone (spec 28). A device is a `computer` (as before) or a `phone`; a
-- phone belongs to the computer it was connected with (`peer`), and goes with
-- it when that computer is removed.
ALTER TABLE devices ADD COLUMN kind TEXT NOT NULL DEFAULT 'computer'
    CHECK (kind IN ('computer', 'phone'));
ALTER TABLE devices ADD COLUMN peer TEXT REFERENCES devices(id) ON DELETE CASCADE;
CREATE INDEX devices_by_peer ON devices(peer);

-- A phone being connected (spec 28.3). The computer opens the row; the phone
-- joins with its public key and a proof; the computer accepts with its own;
-- the phone then collects its token once and the row goes away. The server
-- never sees the QR's secret, so it can only carry these values, not forge them.
CREATE TABLE pairings (
    id         TEXT PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    computer   TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    phone_pub  TEXT,
    phone_name TEXT,
    proof      TEXT,
    poll_hash  TEXT,
    daemon_pub TEXT,
    proof2     TEXT
);
CREATE INDEX pairings_by_computer ON pairings(computer);

-- Sealed messages from a phone that the computer has not acknowledged (spec
-- 28.4). The server never reads `body`. Nothing is queued the other way.
CREATE TABLE relay_queue (
    from_device TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    to_device   TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    seq         INTEGER NOT NULL,
    body        TEXT NOT NULL,
    expires_at  INTEGER NOT NULL,
    PRIMARY KEY (from_device, seq)
);
CREATE INDEX relay_queue_by_to ON relay_queue(to_device, from_device, seq);
