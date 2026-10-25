-- A bot can show its plan before a long task and start by itself after a few
-- seconds (spec 10.5).
ALTER TABLE bots ADD COLUMN plan_first INTEGER NOT NULL DEFAULT 0;
