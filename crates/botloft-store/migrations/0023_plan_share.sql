-- How much of the owner's Claude plan each bot uses (spec 8.7): what each
-- turn cost at API list prices, as Claude Code reports it, and the plan's
-- usage as it changes. The share is learned from the two.
CREATE TABLE turn_costs (
    bot_id TEXT    NOT NULL REFERENCES bots (id),
    at     INTEGER NOT NULL,
    cost   REAL    NOT NULL
);
CREATE INDEX turn_costs_at ON turn_costs (at);
CREATE INDEX turn_costs_bot ON turn_costs (bot_id);

-- One row each time a usage window of the plan reads a new value.
CREATE TABLE plan_readings (
    name        TEXT    NOT NULL,
    at          INTEGER NOT NULL,
    utilization REAL    NOT NULL,
    resets_at   INTEGER
);
CREATE INDEX plan_readings_window_at ON plan_readings (name, at);
