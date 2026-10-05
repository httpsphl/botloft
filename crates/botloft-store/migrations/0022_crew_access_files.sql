-- Reading and editing another crew's files, besides talking (spec 10.4).
ALTER TABLE crew_access ADD COLUMN read INTEGER NOT NULL DEFAULT 0;
ALTER TABLE crew_access ADD COLUMN edit INTEGER NOT NULL DEFAULT 0;
