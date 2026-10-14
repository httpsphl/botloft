-- A crew can keep every bot on automatic mode: the bots it has and the ones
-- that join later (spec 7.4).
ALTER TABLE crews ADD COLUMN auto_mode INTEGER NOT NULL DEFAULT 0;
