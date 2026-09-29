-- The folder a crew works in, when the owner chose one (spec 5). NULL keeps
-- the crew's shared folder, as before.
ALTER TABLE crews ADD COLUMN work_dir TEXT;
