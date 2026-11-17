-- How a wait ended (spec 20.14): 'happened' or 'timed_out'; NULL for every
-- other run.
ALTER TABLE routine_runs ADD COLUMN waited TEXT;
