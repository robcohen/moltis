CREATE TABLE IF NOT EXISTS task_templates (
    id                  TEXT PRIMARY KEY,
    slug                TEXT,
    title               TEXT NOT NULL,
    description         TEXT,
    priority            TEXT NOT NULL,
    list_key            TEXT,
    goal_id             TEXT REFERENCES goals(id) ON DELETE SET NULL,
    parent_template_id  TEXT REFERENCES task_templates(id) ON DELETE SET NULL,
    owner               TEXT,
    project_id          TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_template_dependencies (
    task_template_id             TEXT NOT NULL REFERENCES task_templates(id) ON DELETE CASCADE,
    depends_on_task_template_id  TEXT NOT NULL REFERENCES task_templates(id) ON DELETE CASCADE,
    PRIMARY KEY (task_template_id, depends_on_task_template_id)
);

CREATE TABLE IF NOT EXISTS recurring_tasks (
    id                      TEXT PRIMARY KEY,
    name                    TEXT NOT NULL,
    template_id             TEXT NOT NULL REFERENCES task_templates(id) ON DELETE CASCADE,
    schedule_json           TEXT NOT NULL,
    enabled                 INTEGER NOT NULL DEFAULT 1,
    list_key                TEXT,
    goal_id                 TEXT REFERENCES goals(id) ON DELETE SET NULL,
    owner                   TEXT,
    project_id              TEXT,
    next_run_at_ms          INTEGER,
    last_materialized_at_ms INTEGER,
    created_at              TEXT NOT NULL,
    updated_at              TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS recurring_task_materializations (
    id                  TEXT PRIMARY KEY,
    recurring_task_id   TEXT NOT NULL REFERENCES recurring_tasks(id) ON DELETE CASCADE,
    template_id         TEXT NOT NULL REFERENCES task_templates(id) ON DELETE CASCADE,
    root_task_id        TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    scheduled_for_ms    INTEGER NOT NULL,
    created_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_task_templates_goal_id ON task_templates(goal_id);
CREATE INDEX IF NOT EXISTS idx_task_templates_project_id ON task_templates(project_id);
CREATE INDEX IF NOT EXISTS idx_task_templates_parent_template_id ON task_templates(parent_template_id);
CREATE INDEX IF NOT EXISTS idx_recurring_tasks_template_id ON recurring_tasks(template_id);
CREATE INDEX IF NOT EXISTS idx_recurring_tasks_project_id ON recurring_tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_recurring_tasks_next_run_at_ms ON recurring_tasks(next_run_at_ms);
CREATE INDEX IF NOT EXISTS idx_recurring_task_materializations_recurring_id
    ON recurring_task_materializations(recurring_task_id, created_at DESC);
