-- Add delete_token field to files table
ALTER TABLE files ADD COLUMN delete_token TEXT;

-- Create index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_delete_token ON files(delete_token);
