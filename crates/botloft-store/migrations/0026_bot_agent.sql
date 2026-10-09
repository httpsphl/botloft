-- Which agent runs a bot (spec 30): `claude`, `codex` or `agy`.
ALTER TABLE bots ADD COLUMN agent TEXT NOT NULL DEFAULT 'claude';
