CREATE TABLE IF NOT EXISTS task_external_links (
    id                  TEXT PRIMARY KEY,
    task_id             TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    provider            TEXT NOT NULL,
    remote_id           TEXT NOT NULL,
    remote_title        TEXT NOT NULL,
    remote_status       TEXT NOT NULL,
    remote_url          TEXT,
    remote_updated_at   TEXT,
    synced_at           TEXT NOT NULL,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    UNIQUE(provider, remote_id)
);

CREATE INDEX IF NOT EXISTS idx_task_external_links_task_id
    ON task_external_links(task_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_task_external_links_provider
    ON task_external_links(provider, remote_id);
