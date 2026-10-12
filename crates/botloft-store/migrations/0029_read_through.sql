-- Until when the owner read each bot's replies (Unix ms): the unread mark
-- of the conversation list survives the app losing its own storage.
ALTER TABLE bots ADD COLUMN read_through INTEGER;
