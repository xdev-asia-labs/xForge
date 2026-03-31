-- Key Store for SSH keys and credentials
CREATE TABLE IF NOT EXISTS key_store (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    key_type TEXT NOT NULL DEFAULT 'ssh_key',  -- ssh_key | login_password | token
    key_data TEXT NOT NULL,
    description TEXT,
    created_by TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Job Schedules (cron-based)
CREATE TABLE IF NOT EXISTS schedules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    recipe_name TEXT NOT NULL,
    server_ids TEXT NOT NULL,  -- JSON array
    params TEXT,               -- JSON object
    cron_expression TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at DATETIME,
    next_run_at DATETIME,
    created_by TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Notification Channels
CREATE TABLE IF NOT EXISTS notification_channels (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    channel_type TEXT NOT NULL DEFAULT 'webhook',  -- webhook
    config TEXT NOT NULL,        -- JSON: { url, headers, template }
    events TEXT NOT NULL DEFAULT '["job.success","job.failed"]',  -- JSON array
    enabled INTEGER NOT NULL DEFAULT 1,
    created_by TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Add new columns to users
ALTER TABLE users ADD COLUMN email TEXT;
ALTER TABLE users ADD COLUMN display_name TEXT;

-- Add key_id to servers for key store integration
ALTER TABLE servers ADD COLUMN key_id TEXT;
