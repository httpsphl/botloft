-- How much each bot may do without asking (spec 7.4). Existing bots keep
-- asking before everything, as they did.
ALTER TABLE bots ADD COLUMN permission_mode TEXT NOT NULL DEFAULT 'default';
