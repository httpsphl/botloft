-- Messages, their deliveries and tasks between bots (spec 9 and 12).
-- Times are Unix milliseconds. Rows are never deleted; rowid keeps the
-- insertion order, which the courier follows for each bot.

CREATE TABLE tasks (
    id               TEXT PRIMARY KEY,
    crew_id          TEXT NOT NULL REFERENCES crews (id),
    requester_bot_id TEXT NOT NULL REFERENCES bots (id),
    assignee_bot_id  TEXT NOT NULL REFERENCES bots (id),
    status           TEXT NOT NULL,
    deadline_at      INTEGER NOT NULL,
    hops             INTEGER NOT NULL,
    origin_task_id   TEXT REFERENCES tasks (id),
    result           TEXT,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL
);

CREATE INDEX tasks_assignee_status ON tasks (assignee_bot_id, status);
CREATE INDEX tasks_open_deadline ON tasks (deadline_at) WHERE status = 'open';

CREATE TABLE messages (
    id          TEXT PRIMARY KEY,
    crew_id     TEXT NOT NULL REFERENCES crews (id),
    from_kind   TEXT NOT NULL,
    from_bot_id TEXT REFERENCES bots (id),
    to_bot_id   TEXT NOT NULL REFERENCES bots (id),
    kind        TEXT NOT NULL,
    body        TEXT NOT NULL,
    task_id     TEXT REFERENCES tasks (id),
    created_at  INTEGER NOT NULL
);

CREATE INDEX messages_crew_created ON messages (crew_id, created_at);
CREATE INDEX messages_to_bot ON messages (to_bot_id);
CREATE INDEX messages_from_bot ON messages (from_bot_id);

CREATE TABLE deliveries (
    id              TEXT PRIMARY KEY,
    message_id      TEXT NOT NULL REFERENCES messages (id),
    bot_id          TEXT NOT NULL REFERENCES bots (id),
    state           TEXT NOT NULL,
    attempts        INTEGER NOT NULL DEFAULT 0,
    next_attempt_at INTEGER NOT NULL,
    -- Set while sending; a lease that ran out means the daemon died mid-send.
    lease_until     INTEGER,
    last_error      TEXT,
    updated_at      INTEGER NOT NULL
);

CREATE INDEX deliveries_state_next ON deliveries (state, next_attempt_at);
CREATE INDEX deliveries_bot_state ON deliveries (bot_id, state);
