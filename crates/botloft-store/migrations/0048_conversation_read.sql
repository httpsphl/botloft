-- What the owner read of each conversation that is not the main one
-- (spec 40.7); the main one's stays in bots.read_through, which the phone
-- shares (28.12).

ALTER TABLE conversations ADD COLUMN read_through INTEGER;
