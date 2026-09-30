-- How much each bot thinks before it answers (spec 7.4): the effort level the
-- owner picked, and the level its model uses by itself, as Claude Code last
-- reported it. Existing bots keep their model's own level, as they did.
ALTER TABLE bots ADD COLUMN effort TEXT NOT NULL DEFAULT 'default';
ALTER TABLE bots ADD COLUMN effort_default TEXT;
