-- Shell commands the owner lets a bot run when its agent cannot ask (spec 30):
-- a JSON array of command prefixes.
ALTER TABLE bots ADD COLUMN allowed_commands TEXT NOT NULL DEFAULT '[]';
