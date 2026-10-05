-- Connected tools (spec 25): the MCP servers the owner registered, and which
-- bots use them. The values of headers and environment variables are not
-- here: they live in secrets/mcp, and `config` holds only their names.
CREATE TABLE mcp_servers (
    id          TEXT    PRIMARY KEY,
    name        TEXT    NOT NULL,
    slug        TEXT    NOT NULL UNIQUE,
    kind        TEXT    NOT NULL CHECK (kind IN ('http', 'stdio')),
    config      TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL
);

CREATE TABLE bot_mcp_servers (
    bot_id    TEXT NOT NULL REFERENCES bots (id),
    server_id TEXT NOT NULL REFERENCES mcp_servers (id),
    PRIMARY KEY (bot_id, server_id)
);

CREATE INDEX bot_mcp_servers_server ON bot_mcp_servers (server_id);
