CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT DEFAULT 'operator',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER DEFAULT 22,
    ssh_user TEXT DEFAULT 'root',
    ssh_key_path TEXT,
    labels TEXT,
    group_name TEXT,
    status TEXT DEFAULT 'unknown',
    last_health_check DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    recipe_name TEXT NOT NULL,
    server_ids TEXT NOT NULL,
    params TEXT,
    status TEXT DEFAULT 'pending',
    output TEXT,
    started_at DATETIME,
    finished_at DATETIME,
    created_by TEXT REFERENCES users(id),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Default admin user is created at startup by the application with a proper bcrypt hash.
-- Credentials: admin/admin
