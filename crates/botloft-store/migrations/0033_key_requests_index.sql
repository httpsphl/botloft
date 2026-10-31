-- The home and the island list the bots' open requests for a key (spec 32.5)
-- each time something waits; they are chat items, among every other one.
-- The expressions must match `open_key_request_items` in agent_keys.rs word
-- for word.
CREATE INDEX chat_items_key_requests ON chat_items (created_at)
    WHERE kind = 'key_request' AND json_extract(data, '$.status') = 'open';
