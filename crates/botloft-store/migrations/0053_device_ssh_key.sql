-- The SSH key of a device whose app opens its own tunnel (spec 42.6): the
-- public line, which sshd reads through `botloftd tunnel-keys`. The private
-- key is never kept here.
ALTER TABLE devices ADD COLUMN ssh_key TEXT;
