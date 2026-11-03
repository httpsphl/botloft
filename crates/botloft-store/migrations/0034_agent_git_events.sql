-- What an agent did in git (spec 33.5), from Claude Code's gitOperation:
-- commits, pushes, branches merged or rebased, pull requests. The newest
-- 50 of each agent are kept.
CREATE TABLE agent_git_events (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    bot_id  TEXT NOT NULL REFERENCES bots (id),
    kind    TEXT NOT NULL,
    sha     TEXT,
    branch  TEXT,
    action  TEXT,
    number  INTEGER,
    url     TEXT,
    at      INTEGER NOT NULL
);

CREATE INDEX agent_git_events_bot ON agent_git_events (bot_id, id);
