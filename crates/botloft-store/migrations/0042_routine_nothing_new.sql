-- A run the bot ended with `nothing_new`: nothing to report (spec 20.5).
ALTER TABLE routine_runs ADD COLUMN nothing_new INTEGER NOT NULL DEFAULT 0;
