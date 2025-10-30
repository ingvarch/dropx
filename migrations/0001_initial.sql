-- Files table
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    original_filename TEXT NOT NULL,
    size INTEGER NOT NULL,
    mime_type TEXT,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    download_count INTEGER DEFAULT 0,
    uploaded_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_expires_at ON files(expires_at);
CREATE INDEX IF NOT EXISTS idx_uploaded_by ON files(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_created_at ON files(created_at);

-- API Keys table
CREATE TABLE IF NOT EXISTS api_keys (
    key TEXT PRIMARY KEY,
    user_email TEXT,
    created_at INTEGER NOT NULL,
    last_used_at INTEGER,
    is_active INTEGER DEFAULT 1,
    tier TEXT DEFAULT 'standard'
);

CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(is_active);
CREATE INDEX IF NOT EXISTS idx_api_keys_email ON api_keys(user_email);
