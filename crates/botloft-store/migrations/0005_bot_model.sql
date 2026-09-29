-- Which Claude model each bot runs on (spec 7.4), and the model Claude Code
-- last reported. Existing bots keep the account's default, as they did.
ALTER TABLE bots ADD COLUMN model TEXT NOT NULL DEFAULT 'default';
ALTER TABLE bots ADD COLUMN model_in_use TEXT;
