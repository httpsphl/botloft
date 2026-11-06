-- What a build of the Maker changed in crews that already existed, and how
-- to put each thing back (spec 36.4). A JSON list; never message text.
ALTER TABLE builds ADD COLUMN changes TEXT NOT NULL DEFAULT '[]';
