-- Recipe Sources: external GitHub repos as marketplace
CREATE TABLE IF NOT EXISTS recipe_sources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT UNIQUE NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'pending',  -- pending | syncing | synced | error
    sync_error TEXT,
    last_synced_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Recipes discovered from a source (auto-detected or from recipe.yaml)
CREATE TABLE IF NOT EXISTS source_recipes (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES recipe_sources(id) ON DELETE CASCADE,
    slug TEXT NOT NULL,           -- used as local recipe name when installed
    name TEXT NOT NULL,
    description TEXT,
    playbook TEXT NOT NULL,       -- relative path inside the cloned repo
    version TEXT DEFAULT '1.0.0',
    tags TEXT DEFAULT '[]',       -- JSON array
    installed INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_id, slug)
);

-- Audit Log
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    action TEXT NOT NULL,        -- e.g. "source.add", "job.create", "server.delete"
    resource_type TEXT,
    resource_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
