-- Searching the chats (spec 8.8). The index reads the text of each chat
-- item through a view, so nothing is stored twice: what the bot was told
-- (the owner, another bot or Botloft), what it replied, and its questions
-- with their answers. Tool calls, approvals and notices are left out.
-- Triggers keep the index in step with chat_items, deletes included.

CREATE VIEW chat_text (id, text) AS
SELECT rowid,
       CASE kind
           WHEN 'inbound' THEN json_extract(data, '$.message.body')
           WHEN 'reply' THEN json_extract(data, '$.text')
           WHEN 'question' THEN json_extract(data, '$.question.text') || char(10) ||
                                coalesce(json_extract(data, '$.question.answer'), '')
       END
FROM chat_items;

CREATE VIRTUAL TABLE chat_search USING fts5 (
    text,
    content = 'chat_text',
    content_rowid = 'id',
    tokenize = 'unicode61 remove_diacritics 2'
);

INSERT INTO chat_search (rowid, text)
SELECT id, text FROM chat_text WHERE text IS NOT NULL;

CREATE TRIGGER chat_search_insert AFTER INSERT ON chat_items
WHEN NEW.kind IN ('inbound', 'reply', 'question')
BEGIN
    INSERT INTO chat_search (rowid, text)
    SELECT id, text FROM chat_text WHERE id = NEW.rowid;
END;

-- The old text has to be given back exactly to take it out.
CREATE TRIGGER chat_search_delete AFTER DELETE ON chat_items
WHEN OLD.kind IN ('inbound', 'reply', 'question')
BEGIN
    INSERT INTO chat_search (chat_search, rowid, text) VALUES ('delete', OLD.rowid,
        CASE OLD.kind
            WHEN 'inbound' THEN json_extract(OLD.data, '$.message.body')
            WHEN 'reply' THEN json_extract(OLD.data, '$.text')
            WHEN 'question' THEN json_extract(OLD.data, '$.question.text') || char(10) ||
                                 coalesce(json_extract(OLD.data, '$.question.answer'), '')
        END);
END;

-- One trigger, so the old text leaves before the new one comes in.
CREATE TRIGGER chat_search_update AFTER UPDATE OF kind, data ON chat_items
BEGIN
    INSERT INTO chat_search (chat_search, rowid, text)
    SELECT 'delete', OLD.rowid,
        CASE OLD.kind
            WHEN 'inbound' THEN json_extract(OLD.data, '$.message.body')
            WHEN 'reply' THEN json_extract(OLD.data, '$.text')
            WHEN 'question' THEN json_extract(OLD.data, '$.question.text') || char(10) ||
                                 coalesce(json_extract(OLD.data, '$.question.answer'), '')
        END
    WHERE OLD.kind IN ('inbound', 'reply', 'question');
    INSERT INTO chat_search (rowid, text)
    SELECT id, text FROM chat_text
    WHERE id = NEW.rowid AND NEW.kind IN ('inbound', 'reply', 'question');
END;
