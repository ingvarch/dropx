-- Delete tokens, API keys and IP hashes were stored as plain or unkeyed values.
-- SQLite cannot compute SHA-256 or HMAC, so the old values are replaced with random ones:
-- old delete tokens and API keys stop working, and no secret stays readable in the database.
-- Values that are already 64-character SHA-256 hex were written by the new version and stay.
UPDATE files SET delete_token = lower(hex(randomblob(32))) WHERE length(delete_token) <> 64;
UPDATE api_keys SET key = lower(hex(randomblob(32))), is_active = 0 WHERE length(key) <> 64;
-- Old anonymous uploader ids (unkeyed IP hashes) have the same length as the new HMACs,
-- so all uploader ids except 64-character key hashes are replaced.
UPDATE files SET uploaded_by = lower(hex(randomblob(16))) WHERE length(uploaded_by) <> 64;
