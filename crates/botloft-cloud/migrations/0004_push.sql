-- Where a phone is told something waits for it (spec 28.8): the address its
-- browser's push service gave it. One per phone; it goes with the phone.
CREATE TABLE push_subscriptions (
    device     TEXT PRIMARY KEY REFERENCES devices(id) ON DELETE CASCADE,
    endpoint   TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
