-- Tokens per bot over a period (spec 8.7) read only the `turn` items, by
-- the time they ended.
CREATE INDEX chat_items_turns ON chat_items (created_at) WHERE kind = 'turn';
