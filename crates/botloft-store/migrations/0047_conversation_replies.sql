-- Replies go back to the conversation that asked (spec 40.2): the result
-- of a task to the conversation that gave it, a routine or reminder the
-- agent made to the conversation that made it. NULL is the main one.
-- Questions, cards, key requests and approvals know theirs by their chat
-- item.

ALTER TABLE tasks ADD COLUMN requester_conversation_id TEXT;
ALTER TABLE routines ADD COLUMN conversation_id TEXT;
