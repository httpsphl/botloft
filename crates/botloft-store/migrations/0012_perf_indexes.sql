-- Indexes for lookups that scanned whole tables. Rows are never deleted, so
-- each of those scans grew with every message, delivery and tool call.

-- Foreign keys: deleting a parent row looks for its children (spec 7.6).
CREATE INDEX messages_task ON messages (task_id);
CREATE INDEX messages_routine ON messages (routine_id);
CREATE INDEX deliveries_message ON deliveries (message_id);
CREATE INDEX tasks_requester_status ON tasks (requester_bot_id, status);
CREATE INDEX tasks_crew ON tasks (crew_id);
CREATE INDEX tasks_origin ON tasks (origin_task_id);
CREATE INDEX approvals_bot ON approvals (bot_id);
CREATE INDEX approvals_chat_item ON approvals (chat_item_id);

-- A tool result finds its call, and the files panel the files a bot wrote.
-- The expressions must match the queries in chat.rs word for word.
CREATE INDEX chat_items_tool_use ON chat_items (bot_id, json_extract(data, '$.toolUseId'))
    WHERE kind = 'tool';
CREATE INDEX chat_items_tool_file ON chat_items (bot_id, json_extract(data, '$.file'))
    WHERE kind = 'tool' AND json_extract(data, '$.file') IS NOT NULL;

-- A routine's runs are read newest first by rowid; `id` in the key forced a
-- sort. Open runs get their own index.
DROP INDEX routine_runs_routine;
CREATE INDEX routine_runs_routine ON routine_runs (routine_id);
CREATE INDEX routine_runs_queued ON routine_runs (routine_id) WHERE status = 'queued';

-- Only enabled, active routines ever come due.
DROP INDEX routines_due;
CREATE INDEX routines_due ON routines (next_run_at) WHERE enabled AND archived_at IS NULL;
