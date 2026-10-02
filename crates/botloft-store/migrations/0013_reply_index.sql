-- The conversation list asks each bot for its newest reply (spec 15.1).
CREATE INDEX chat_items_reply ON chat_items (bot_id, updated_at) WHERE kind = 'reply';
