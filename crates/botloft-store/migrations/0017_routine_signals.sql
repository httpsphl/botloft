-- Signals between bots (spec 20.13): a run a signal caused keeps which
-- signal, who sent it (no foreign key: a deleted sender only leaves its
-- id behind) and the note that came with it.

ALTER TABLE routine_runs ADD COLUMN signal_name TEXT;
ALTER TABLE routine_runs ADD COLUMN signal_from TEXT;
ALTER TABLE routine_runs ADD COLUMN signal_note TEXT;

CREATE INDEX routine_runs_signal ON routine_runs (routine_id, created_at)
    WHERE signal_name IS NOT NULL;
