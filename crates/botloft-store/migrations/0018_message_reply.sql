-- Replying to something in a bot's chat (spec 9.3): the message keeps the
-- chat item it replies to (no foreign key: chat items leave before
-- messages when a bot is deleted, spec 7.6) and its text as quoted then.

ALTER TABLE messages ADD COLUMN reply_item_id TEXT;
ALTER TABLE messages ADD COLUMN reply_text TEXT;
