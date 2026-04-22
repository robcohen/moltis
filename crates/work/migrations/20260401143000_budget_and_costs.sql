ALTER TABLE task_runs ADD COLUMN runtime_run_id TEXT;
ALTER TABLE task_runs ADD COLUMN provider TEXT;
ALTER TABLE task_runs ADD COLUMN model TEXT;
ALTER TABLE task_runs ADD COLUMN input_tokens INTEGER;
ALTER TABLE task_runs ADD COLUMN output_tokens INTEGER;
ALTER TABLE task_runs ADD COLUMN request_input_tokens INTEGER;
ALTER TABLE task_runs ADD COLUMN request_output_tokens INTEGER;
ALTER TABLE task_runs ADD COLUMN duration_ms INTEGER;

CREATE TABLE IF NOT EXISTS task_cost_attribution (
    id                      TEXT PRIMARY KEY,
    task_id                 TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    task_run_id             TEXT REFERENCES task_runs(id) ON DELETE SET NULL,
    goal_id                 TEXT REFERENCES goals(id) ON DELETE SET NULL,
    project_id              TEXT,
    session_key             TEXT,
    runtime_run_id          TEXT,
    provider                TEXT,
    model                   TEXT,
    input_tokens            INTEGER NOT NULL,
    output_tokens           INTEGER NOT NULL,
    request_input_tokens    INTEGER,
    request_output_tokens   INTEGER,
    total_tokens            INTEGER NOT NULL,
    estimated_cost_usd      REAL,
    created_at              TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS work_budgets (
    id                  TEXT PRIMARY KEY,
    scope               TEXT NOT NULL,
    scope_id            TEXT NOT NULL,
    soft_limit_tokens   INTEGER,
    hard_limit_tokens   INTEGER,
    soft_limit_usd      REAL,
    hard_limit_usd      REAL,
    note                TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    UNIQUE(scope, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_task_runs_runtime_run_id ON task_runs(runtime_run_id);
CREATE INDEX IF NOT EXISTS idx_task_cost_attribution_task_id ON task_cost_attribution(task_id);
CREATE INDEX IF NOT EXISTS idx_task_cost_attribution_goal_id ON task_cost_attribution(goal_id);
CREATE INDEX IF NOT EXISTS idx_task_cost_attribution_project_id ON task_cost_attribution(project_id);
CREATE INDEX IF NOT EXISTS idx_task_cost_attribution_session_key ON task_cost_attribution(session_key);
CREATE INDEX IF NOT EXISTS idx_task_cost_attribution_runtime_run_id ON task_cost_attribution(runtime_run_id);
CREATE INDEX IF NOT EXISTS idx_task_cost_attribution_created_at ON task_cost_attribution(created_at);
CREATE INDEX IF NOT EXISTS idx_work_budgets_scope ON work_budgets(scope, scope_id);
