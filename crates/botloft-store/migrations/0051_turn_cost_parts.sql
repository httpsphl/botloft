-- What each turn cost at API list prices (spec 8.7), now per model and
-- with the routine whose run the turn carried, so the owner sees the cost
-- by model and by routine. Rows from before have neither.
ALTER TABLE turn_costs ADD COLUMN model TEXT;
ALTER TABLE turn_costs ADD COLUMN routine_id TEXT;

-- The plan's share per bot is no longer learned from the plan's rise.
DROP TABLE plan_readings;
