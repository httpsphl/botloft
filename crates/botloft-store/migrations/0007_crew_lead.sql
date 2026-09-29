-- The bot that leads each crew (spec 10.2). No foreign key: the chief of a
-- new crew is saved in the same transaction as its bot, and an archived
-- chief simply no longer counts.
ALTER TABLE crews ADD COLUMN lead_bot_id TEXT;
