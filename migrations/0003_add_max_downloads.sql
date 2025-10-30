-- Add max_downloads field to files table for download limit feature
ALTER TABLE files ADD COLUMN max_downloads INTEGER;

-- Index for efficient queries when checking download limits
CREATE INDEX idx_max_downloads ON files(max_downloads) WHERE max_downloads IS NOT NULL;
