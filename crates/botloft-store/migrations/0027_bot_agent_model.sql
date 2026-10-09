-- The model of a bot whose agent is not Claude Code (spec 30): an id the
-- agent lists, NULL for the agent's own default.
ALTER TABLE bots ADD COLUMN agent_model TEXT;
