-- Security Audits
CREATE TABLE IF NOT EXISTS security_audits (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'running',  -- running | completed | failed
    score INTEGER,                            -- 0-100 overall score
    results TEXT,                             -- JSON: array of check results
    started_at DATETIME,
    finished_at DATETIME,
    created_by TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers(id)
);
