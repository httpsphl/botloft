-- A bot's note to the owner (spec 23.10) lives with its questions: it is
-- shown, kept and closed the same way, but asks nothing.

ALTER TABLE questions ADD COLUMN kind TEXT NOT NULL DEFAULT 'question';
