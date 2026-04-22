use {
    crate::{
        Error, Result,
        types::{
            ApprovalStatus, BudgetCheck, BudgetScope, BudgetStatus, CrossProjectDependency,
            ExternalTaskLink, ExternalTaskLinkFilter, Goal, GoalPatch, GoalPlanPreset,
            GoalPlanRequest, GoalPlanResult, GoalStatus, InstantiateTaskTemplate, NewGoal,
            NewRecurringTask, NewTask, NewTaskApproval, NewTaskComment, NewTaskCostAttribution,
            NewTaskRun, NewTaskTemplate, NewWorkBudget, ProjectWorkSummary, RecurringTask,
            RecurringTaskFilter, RecurringTaskMaterialization, RecurringTaskMaterializeResult,
            RecurringTaskPatch, RunStatus, Task, TaskApproval, TaskApprovalPatch, TaskComment,
            TaskCostAttribution, TaskCostFilter, TaskFilter, TaskPatch, TaskPriority, TaskRun,
            TaskRunPatch, TaskStatus, TaskTemplate, TaskTemplateFilter, TaskTemplatePatch,
            TemplateInstantiationResult, TrackerImportRequest, TrackerImportResult, WorkBudget,
            WorkDashboard, WorkPortfolio, WorkTaskCounts,
        },
    },
    std::{
        collections::{BTreeMap, HashMap},
        time::{SystemTime, UNIX_EPOCH},
    },
    time::OffsetDateTime,
};

fn now_iso() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn slugify(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    let mut prev_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "goal".to_string()
    } else {
        slug
    }
}

#[derive(Clone)]
struct GoalPlanTemplateSeed {
    slug_suffix: &'static str,
    title: String,
    description: String,
    priority: TaskPriority,
    blocked_by_suffixes: &'static [&'static str],
}

fn goal_plan_blueprint(goal: &Goal, preset: GoalPlanPreset) -> Vec<GoalPlanTemplateSeed> {
    match preset {
        GoalPlanPreset::Delivery => vec![
            GoalPlanTemplateSeed {
                slug_suffix: "scope",
                title: format!("Define scope for {}", goal.title),
                description:
                    "Capture success criteria, interfaces, risks, and delivery boundaries."
                        .to_string(),
                priority: TaskPriority::High,
                blocked_by_suffixes: &[],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "implement",
                title: format!("Implement {}", goal.title),
                description: "Execute the main delivery work for this goal.".to_string(),
                priority: TaskPriority::High,
                blocked_by_suffixes: &["scope"],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "verify",
                title: format!("Verify {}", goal.title),
                description: "Run the relevant tests, acceptance checks, and regressions."
                    .to_string(),
                priority: TaskPriority::High,
                blocked_by_suffixes: &["implement"],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "rollout",
                title: format!("Roll out {}", goal.title),
                description: "Handle release notes, rollout steps, and post-ship follow-up."
                    .to_string(),
                priority: TaskPriority::Medium,
                blocked_by_suffixes: &["verify"],
            },
        ],
        GoalPlanPreset::Investigation => vec![
            GoalPlanTemplateSeed {
                slug_suffix: "gather",
                title: format!("Gather evidence for {}", goal.title),
                description: "Collect logs, reports, and existing context before changing code."
                    .to_string(),
                priority: TaskPriority::High,
                blocked_by_suffixes: &[],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "analyze",
                title: format!("Analyze failure modes for {}", goal.title),
                description: "Reproduce the issue and narrow the plausible root-cause set."
                    .to_string(),
                priority: TaskPriority::High,
                blocked_by_suffixes: &["gather"],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "synthesize",
                title: format!("Synthesize findings for {}", goal.title),
                description: "Summarize the evidence trail, root cause, and important unknowns."
                    .to_string(),
                priority: TaskPriority::Medium,
                blocked_by_suffixes: &["analyze"],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "next-steps",
                title: format!("Recommend next steps for {}", goal.title),
                description: "Translate the investigation into concrete follow-up work."
                    .to_string(),
                priority: TaskPriority::Medium,
                blocked_by_suffixes: &["synthesize"],
            },
        ],
        GoalPlanPreset::Maintenance => vec![
            GoalPlanTemplateSeed {
                slug_suffix: "audit",
                title: format!("Audit current state for {}", goal.title),
                description: "Inspect the existing surface area and identify required maintenance."
                    .to_string(),
                priority: TaskPriority::Medium,
                blocked_by_suffixes: &[],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "apply",
                title: format!("Apply maintenance changes for {}", goal.title),
                description: "Execute the concrete upkeep work and clean up stale edges."
                    .to_string(),
                priority: TaskPriority::High,
                blocked_by_suffixes: &["audit"],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "validate",
                title: format!("Validate maintenance changes for {}", goal.title),
                description: "Confirm the maintenance work did not introduce regressions."
                    .to_string(),
                priority: TaskPriority::High,
                blocked_by_suffixes: &["apply"],
            },
            GoalPlanTemplateSeed {
                slug_suffix: "follow-up",
                title: format!("Close follow-ups for {}", goal.title),
                description: "Track any leftovers, docs updates, and future cleanup items."
                    .to_string(),
                priority: TaskPriority::Low,
                blocked_by_suffixes: &["validate"],
            },
        ],
    }
}

fn map_remote_status(status: &str) -> TaskStatus {
    let normalized = status.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "in_progress" | "in progress" | "doing" | "active" => TaskStatus::InProgress,
        "blocked" => TaskStatus::Blocked,
        "completed" | "complete" | "done" | "closed" | "resolved" => TaskStatus::Completed,
        "cancelled" | "canceled" => TaskStatus::Cancelled,
        _ => TaskStatus::Pending,
    }
}

#[derive(sqlx::FromRow)]
struct GoalRow {
    id: String,
    slug: Option<String>,
    title: String,
    description: Option<String>,
    status: String,
    project_id: Option<String>,
    parent_goal_id: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<GoalRow> for Goal {
    type Error = Error;

    fn try_from(value: GoalRow) -> Result<Self> {
        Ok(Self {
            id: value.id,
            slug: value.slug,
            title: value.title,
            description: value.description,
            status: value.status.parse::<GoalStatus>()?,
            project_id: value.project_id,
            parent_goal_id: value.parent_goal_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: String,
    list_key: Option<String>,
    title: String,
    description: Option<String>,
    status: String,
    priority: String,
    goal_id: Option<String>,
    parent_task_id: Option<String>,
    owner: Option<String>,
    project_id: Option<String>,
    session_key: Option<String>,
    claimed_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct TaskDependencyRow {
    depends_on_task_id: String,
}

#[derive(sqlx::FromRow)]
struct TaskTemplateRow {
    id: String,
    slug: Option<String>,
    title: String,
    description: Option<String>,
    priority: String,
    list_key: Option<String>,
    goal_id: Option<String>,
    parent_template_id: Option<String>,
    owner: Option<String>,
    project_id: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct TaskTemplateDependencyRow {
    depends_on_task_template_id: String,
}

impl TaskTemplateRow {
    async fn into_template(self, pool: &sqlx::SqlitePool) -> Result<TaskTemplate> {
        let blocked_by = sqlx::query_as::<_, TaskTemplateDependencyRow>(
            "SELECT depends_on_task_template_id
             FROM task_template_dependencies
             WHERE task_template_id = ?
             ORDER BY depends_on_task_template_id",
        )
        .bind(&self.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.depends_on_task_template_id)
        .collect();

        Ok(TaskTemplate {
            id: self.id,
            slug: self.slug,
            title: self.title,
            description: self.description,
            priority: self.priority.parse::<TaskPriority>()?,
            list_key: self.list_key,
            goal_id: self.goal_id,
            parent_template_id: self.parent_template_id,
            owner: self.owner,
            project_id: self.project_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            blocked_by,
        })
    }
}

impl TaskRow {
    async fn into_task(self, pool: &sqlx::SqlitePool) -> Result<Task> {
        let blocked_by = sqlx::query_as::<_, TaskDependencyRow>(
            "SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ? ORDER BY depends_on_task_id",
        )
        .bind(&self.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.depends_on_task_id)
        .collect();

        Ok(Task {
            id: self.id,
            list_key: self.list_key,
            title: self.title,
            description: self.description,
            status: self.status.parse::<TaskStatus>()?,
            priority: self.priority.parse::<TaskPriority>()?,
            goal_id: self.goal_id,
            parent_task_id: self.parent_task_id,
            owner: self.owner,
            project_id: self.project_id,
            session_key: self.session_key,
            claimed_at: self.claimed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
            blocked_by,
        })
    }
}

#[derive(sqlx::FromRow)]
struct TaskCommentRow {
    id: String,
    task_id: String,
    author: Option<String>,
    body: String,
    created_at: String,
}

impl From<TaskCommentRow> for TaskComment {
    fn from(value: TaskCommentRow) -> Self {
        Self {
            id: value.id,
            task_id: value.task_id,
            author: value.author,
            body: value.body,
            created_at: value.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ExternalTaskLinkRow {
    id: String,
    task_id: String,
    provider: String,
    remote_id: String,
    remote_title: String,
    remote_status: String,
    remote_url: Option<String>,
    remote_updated_at: Option<String>,
    synced_at: String,
    created_at: String,
    updated_at: String,
}

impl From<ExternalTaskLinkRow> for ExternalTaskLink {
    fn from(value: ExternalTaskLinkRow) -> Self {
        Self {
            id: value.id,
            task_id: value.task_id,
            provider: value.provider,
            remote_id: value.remote_id,
            remote_title: value.remote_title,
            remote_status: value.remote_status,
            remote_url: value.remote_url,
            remote_updated_at: value.remote_updated_at,
            synced_at: value.synced_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TaskRunRow {
    id: String,
    task_id: String,
    runtime_run_id: Option<String>,
    session_key: Option<String>,
    agent_id: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    status: String,
    summary: Option<String>,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    request_input_tokens: Option<i64>,
    request_output_tokens: Option<i64>,
    duration_ms: Option<i64>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<TaskRunRow> for TaskRun {
    type Error = Error;

    fn try_from(value: TaskRunRow) -> Result<Self> {
        Ok(Self {
            id: value.id,
            task_id: value.task_id,
            runtime_run_id: value.runtime_run_id,
            session_key: value.session_key,
            agent_id: value.agent_id,
            provider: value.provider,
            model: value.model,
            status: value.status.parse::<RunStatus>()?,
            summary: value.summary,
            input_tokens: value.input_tokens.map(|value| value as u32),
            output_tokens: value.output_tokens.map(|value| value as u32),
            request_input_tokens: value.request_input_tokens.map(|value| value as u32),
            request_output_tokens: value.request_output_tokens.map(|value| value as u32),
            duration_ms: value.duration_ms.map(|value| value as u64),
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct TaskApprovalRow {
    id: String,
    task_id: Option<String>,
    kind: String,
    status: String,
    reason: Option<String>,
    created_at: String,
    updated_at: String,
    resolved_at: Option<String>,
}

impl TryFrom<TaskApprovalRow> for TaskApproval {
    type Error = Error;

    fn try_from(value: TaskApprovalRow) -> Result<Self> {
        Ok(Self {
            id: value.id,
            task_id: value.task_id,
            kind: value.kind,
            status: value.status.parse::<ApprovalStatus>()?,
            reason: value.reason,
            created_at: value.created_at,
            updated_at: value.updated_at,
            resolved_at: value.resolved_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct TaskCostAttributionRow {
    id: String,
    task_id: String,
    task_run_id: Option<String>,
    goal_id: Option<String>,
    project_id: Option<String>,
    session_key: Option<String>,
    runtime_run_id: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    input_tokens: i64,
    output_tokens: i64,
    request_input_tokens: Option<i64>,
    request_output_tokens: Option<i64>,
    total_tokens: i64,
    estimated_cost_usd: Option<f64>,
    created_at: String,
}

impl From<TaskCostAttributionRow> for TaskCostAttribution {
    fn from(value: TaskCostAttributionRow) -> Self {
        Self {
            id: value.id,
            task_id: value.task_id,
            task_run_id: value.task_run_id,
            goal_id: value.goal_id,
            project_id: value.project_id,
            session_key: value.session_key,
            runtime_run_id: value.runtime_run_id,
            provider: value.provider,
            model: value.model,
            input_tokens: value.input_tokens as u64,
            output_tokens: value.output_tokens as u64,
            request_input_tokens: value.request_input_tokens.map(|value| value as u64),
            request_output_tokens: value.request_output_tokens.map(|value| value as u64),
            total_tokens: value.total_tokens as u64,
            estimated_cost_usd: value.estimated_cost_usd,
            created_at: value.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct WorkBudgetRow {
    id: String,
    scope: String,
    scope_id: String,
    soft_limit_tokens: Option<i64>,
    hard_limit_tokens: Option<i64>,
    soft_limit_usd: Option<f64>,
    hard_limit_usd: Option<f64>,
    note: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<WorkBudgetRow> for WorkBudget {
    type Error = Error;

    fn try_from(value: WorkBudgetRow) -> Result<Self> {
        Ok(Self {
            id: value.id,
            scope: value.scope.parse::<BudgetScope>()?,
            scope_id: value.scope_id,
            soft_limit_tokens: value.soft_limit_tokens.map(|value| value as u64),
            hard_limit_tokens: value.hard_limit_tokens.map(|value| value as u64),
            soft_limit_usd: value.soft_limit_usd,
            hard_limit_usd: value.hard_limit_usd,
            note: value.note,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RecurringTaskRow {
    id: String,
    name: String,
    template_id: String,
    schedule_json: String,
    enabled: i64,
    list_key: Option<String>,
    goal_id: Option<String>,
    owner: Option<String>,
    project_id: Option<String>,
    next_run_at_ms: Option<i64>,
    last_materialized_at_ms: Option<i64>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<RecurringTaskRow> for RecurringTask {
    type Error = Error;

    fn try_from(value: RecurringTaskRow) -> Result<Self> {
        Ok(Self {
            id: value.id,
            name: value.name,
            template_id: value.template_id,
            schedule: serde_json::from_str(&value.schedule_json)?,
            enabled: value.enabled != 0,
            list_key: value.list_key,
            goal_id: value.goal_id,
            owner: value.owner,
            project_id: value.project_id,
            next_run_at_ms: value.next_run_at_ms.map(|value| value as u64),
            last_materialized_at_ms: value.last_materialized_at_ms.map(|value| value as u64),
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RecurringTaskMaterializationRow {
    id: String,
    recurring_task_id: String,
    template_id: String,
    root_task_id: String,
    scheduled_for_ms: i64,
    created_at: String,
}

impl From<RecurringTaskMaterializationRow> for RecurringTaskMaterialization {
    fn from(value: RecurringTaskMaterializationRow) -> Self {
        Self {
            id: value.id,
            recurring_task_id: value.recurring_task_id,
            template_id: value.template_id,
            root_task_id: value.root_task_id,
            scheduled_for_ms: value.scheduled_for_ms as u64,
            created_at: value.created_at,
        }
    }
}

pub struct SqliteWorkStore {
    pool: sqlx::SqlitePool,
}

impl SqliteWorkStore {
    #[must_use]
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    async fn replace_dependencies(&self, task_id: &str, blocked_by: &[String]) -> Result<()> {
        sqlx::query("DELETE FROM task_dependencies WHERE task_id = ?")
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        for depends_on in blocked_by {
            sqlx::query(
                "INSERT INTO task_dependencies (task_id, depends_on_task_id) VALUES (?, ?)",
            )
            .bind(task_id)
            .bind(depends_on)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn replace_template_dependencies(
        &self,
        template_id: &str,
        blocked_by: &[String],
    ) -> Result<()> {
        sqlx::query("DELETE FROM task_template_dependencies WHERE task_template_id = ?")
            .bind(template_id)
            .execute(&self.pool)
            .await?;
        for depends_on in blocked_by {
            sqlx::query(
                "INSERT INTO task_template_dependencies (task_template_id, depends_on_task_template_id) VALUES (?, ?)",
            )
            .bind(template_id)
            .bind(depends_on)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn task_required(&self, task_id: &str) -> Result<Task> {
        self.get_task(task_id)
            .await?
            .ok_or_else(|| Error::message(format!("task {task_id} not found")))
    }

    async fn template_required(&self, template_id: &str) -> Result<TaskTemplate> {
        self.get_task_template(template_id)
            .await?
            .ok_or_else(|| Error::message(format!("task template {template_id} not found")))
    }

    async fn collect_template_subtree(&self, root_template_id: &str) -> Result<Vec<TaskTemplate>> {
        let mut ordered = Vec::new();
        let mut stack = vec![root_template_id.to_string()];

        while let Some(template_id) = stack.pop() {
            let template = self.template_required(&template_id).await?;
            let child_ids = sqlx::query_scalar::<_, String>(
                "SELECT id
                 FROM task_templates
                 WHERE parent_template_id = ?
                 ORDER BY created_at DESC, id DESC",
            )
            .bind(&template_id)
            .fetch_all(&self.pool)
            .await?;
            ordered.push(template);
            stack.extend(child_ids);
        }

        Ok(ordered)
    }

    async fn instantiate_template_internal(
        &self,
        params: InstantiateTaskTemplate,
    ) -> Result<TemplateInstantiationResult> {
        let templates = self.collect_template_subtree(&params.template_id).await?;
        if templates.is_empty() {
            return Err(Error::message("template subtree was empty"));
        }

        let now = now_iso();
        let mut tx = self.pool.begin().await?;
        let mut template_to_task_id = HashMap::new();
        let mut created_task_ids = Vec::with_capacity(templates.len());

        for template in &templates {
            let task_id = new_id();
            let parent_task_id = match template.parent_template_id.as_deref() {
                Some(parent_template_id) => Some(
                    template_to_task_id
                        .get(parent_template_id)
                        .cloned()
                        .ok_or_else(|| {
                            Error::message(format!(
                                "template {} references missing parent template {}",
                                template.id, parent_template_id
                            ))
                        })?,
                ),
                None => params.parent_task_id.clone(),
            };

            sqlx::query(
                "INSERT INTO tasks
                 (id, list_key, title, description, status, priority, goal_id, parent_task_id, owner, project_id, session_key, claimed_at, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, ?, ?)",
            )
            .bind(&task_id)
            .bind(params.list_key.as_ref().or(template.list_key.as_ref()))
            .bind(&template.title)
            .bind(&template.description)
            .bind(TaskStatus::Pending.as_str())
            .bind(template.priority.as_str())
            .bind(params.goal_id.as_ref().or(template.goal_id.as_ref()))
            .bind(&parent_task_id)
            .bind(params.owner.as_ref().or(template.owner.as_ref()))
            .bind(params.project_id.as_ref().or(template.project_id.as_ref()))
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;

            template_to_task_id.insert(template.id.clone(), task_id.clone());
            created_task_ids.push(task_id);
        }

        for template in &templates {
            let task_id = template_to_task_id
                .get(&template.id)
                .cloned()
                .ok_or_else(|| Error::message("missing instantiated task mapping"))?;
            for dependency_template_id in &template.blocked_by {
                let depends_on_task_id = template_to_task_id
                    .get(dependency_template_id)
                    .cloned()
                    .ok_or_else(|| {
                    Error::message(format!(
                        "template {} depends on template {} outside the instantiated tree",
                        template.id, dependency_template_id
                    ))
                })?;
                sqlx::query(
                    "INSERT INTO task_dependencies (task_id, depends_on_task_id) VALUES (?, ?)",
                )
                .bind(&task_id)
                .bind(&depends_on_task_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        let root_task_id = template_to_task_id
            .get(&params.template_id)
            .cloned()
            .ok_or_else(|| Error::message("missing root task mapping"))?;

        Ok(TemplateInstantiationResult {
            template_id: params.template_id,
            root_task_id,
            count: created_task_ids.len(),
            task_ids: created_task_ids,
        })
    }

    async fn pending_budget_override_id(&self, task_id: &str) -> Result<Option<String>> {
        sqlx::query_scalar::<_, String>(
            "SELECT id
             FROM task_approvals
             WHERE task_id = ?
               AND kind = 'budget_override'
               AND status = 'pending'
             ORDER BY updated_at DESC, created_at DESC
             LIMIT 1",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn has_approved_budget_override(&self, task_id: &str) -> Result<bool> {
        let approved = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM task_approvals
             WHERE task_id = ?
               AND kind = 'budget_override'
               AND status = 'approved'",
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(approved > 0)
    }

    async fn ensure_pending_budget_override(
        &self,
        task_id: &str,
        reason: Option<String>,
    ) -> Result<String> {
        if let Some(existing_id) = self.pending_budget_override_id(task_id).await? {
            return Ok(existing_id);
        }

        Ok(self
            .create_approval(NewTaskApproval {
                task_id: Some(task_id.to_string()),
                kind: "budget_override".to_string(),
                reason,
                status: ApprovalStatus::Pending,
            })
            .await?
            .id)
    }

    async fn budget_status_for(&self, budget: WorkBudget) -> Result<BudgetStatus> {
        let row = match budget.scope {
            BudgetScope::Task => {
                let total_tokens = sqlx::query_scalar::<_, i64>(
                    "SELECT COALESCE(SUM(total_tokens), 0)
                     FROM task_cost_attribution
                     WHERE task_id = ?",
                )
                .bind(&budget.scope_id)
                .fetch_one(&self.pool)
                .await?;
                let estimated_cost_usd = sqlx::query_scalar::<_, Option<f64>>(
                    "SELECT SUM(estimated_cost_usd)
                     FROM task_cost_attribution
                     WHERE task_id = ?",
                )
                .bind(&budget.scope_id)
                .fetch_one(&self.pool)
                .await?
                .unwrap_or_default();
                (total_tokens as u64, estimated_cost_usd)
            },
            BudgetScope::Goal => {
                let total_tokens = sqlx::query_scalar::<_, i64>(
                    "SELECT COALESCE(SUM(total_tokens), 0)
                     FROM task_cost_attribution
                     WHERE goal_id = ?",
                )
                .bind(&budget.scope_id)
                .fetch_one(&self.pool)
                .await?;
                let estimated_cost_usd = sqlx::query_scalar::<_, Option<f64>>(
                    "SELECT SUM(estimated_cost_usd)
                     FROM task_cost_attribution
                     WHERE goal_id = ?",
                )
                .bind(&budget.scope_id)
                .fetch_one(&self.pool)
                .await?
                .unwrap_or_default();
                (total_tokens as u64, estimated_cost_usd)
            },
            BudgetScope::Project => {
                let total_tokens = sqlx::query_scalar::<_, i64>(
                    "SELECT COALESCE(SUM(total_tokens), 0)
                     FROM task_cost_attribution
                     WHERE project_id = ?",
                )
                .bind(&budget.scope_id)
                .fetch_one(&self.pool)
                .await?;
                let estimated_cost_usd = sqlx::query_scalar::<_, Option<f64>>(
                    "SELECT SUM(estimated_cost_usd)
                     FROM task_cost_attribution
                     WHERE project_id = ?",
                )
                .bind(&budget.scope_id)
                .fetch_one(&self.pool)
                .await?
                .unwrap_or_default();
                (total_tokens as u64, estimated_cost_usd)
            },
        };

        let soft_exceeded = budget.soft_limit_tokens.is_some_and(|limit| row.0 >= limit)
            || budget.soft_limit_usd.is_some_and(|limit| row.1 >= limit);
        let hard_exceeded = budget.hard_limit_tokens.is_some_and(|limit| row.0 >= limit)
            || budget.hard_limit_usd.is_some_and(|limit| row.1 >= limit);

        Ok(BudgetStatus {
            budget,
            total_tokens: row.0,
            estimated_cost_usd: row.1,
            soft_exceeded,
            hard_exceeded,
        })
    }

    pub async fn list_goals(&self) -> Result<Vec<Goal>> {
        let rows = sqlx::query_as::<_, GoalRow>(
            "SELECT * FROM goals ORDER BY updated_at DESC, created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn get_goal(&self, id: &str) -> Result<Option<Goal>> {
        let row = sqlx::query_as::<_, GoalRow>("SELECT * FROM goals WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(TryInto::try_into).transpose()
    }

    pub async fn create_goal(&self, goal: NewGoal) -> Result<Goal> {
        let id = new_id();
        let now = now_iso();
        sqlx::query(
            "INSERT INTO goals (id, slug, title, description, status, project_id, parent_goal_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&goal.slug)
        .bind(&goal.title)
        .bind(&goal.description)
        .bind(goal.status.as_str())
        .bind(&goal.project_id)
        .bind(&goal.parent_goal_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.get_goal(&id)
            .await?
            .ok_or_else(|| Error::message("goal was not created"))
    }

    pub async fn update_goal(&self, id: &str, patch: GoalPatch) -> Result<Option<Goal>> {
        let Some(existing) = self.get_goal(id).await? else {
            return Ok(None);
        };
        let now = now_iso();
        sqlx::query(
            "UPDATE goals
             SET slug = ?, title = ?, description = ?, status = ?, project_id = ?, parent_goal_id = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(patch.slug.or(existing.slug))
        .bind(patch.title.unwrap_or(existing.title))
        .bind(patch.description.or(existing.description))
        .bind(patch.status.unwrap_or(existing.status).as_str())
        .bind(patch.project_id.or(existing.project_id))
        .bind(patch.parent_goal_id.or(existing.parent_goal_id))
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.get_goal(id).await
    }

    pub async fn delete_goal(&self, id: &str) -> Result<bool> {
        let rows = sqlx::query("DELETE FROM goals WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(rows > 0)
    }

    pub async fn plan_goal(&self, request: GoalPlanRequest) -> Result<GoalPlanResult> {
        let goal = self
            .get_goal(&request.goal_id)
            .await?
            .ok_or_else(|| Error::message(format!("goal {} not found", request.goal_id)))?;
        let project_id = request
            .project_id
            .clone()
            .or_else(|| goal.project_id.clone());
        let base_slug = goal.slug.clone().unwrap_or_else(|| slugify(&goal.title));
        let root_template = self
            .create_task_template(NewTaskTemplate {
                slug: Some(format!("{base_slug}-plan")),
                title: goal.title.clone(),
                description: goal.description.clone(),
                priority: TaskPriority::High,
                list_key: request.list_key.clone(),
                goal_id: Some(goal.id.clone()),
                parent_template_id: None,
                owner: request.owner.clone(),
                project_id: project_id.clone(),
                blocked_by: Vec::new(),
            })
            .await?;

        let mut templates = vec![root_template.clone()];
        let mut templates_by_suffix = HashMap::new();

        for seed in goal_plan_blueprint(&goal, request.preset) {
            let blocked_by = seed
                .blocked_by_suffixes
                .iter()
                .filter_map(|suffix| templates_by_suffix.get(*suffix).cloned())
                .collect();
            let template = self
                .create_task_template(NewTaskTemplate {
                    slug: Some(format!("{base_slug}-{}", seed.slug_suffix)),
                    title: seed.title,
                    description: Some(seed.description),
                    priority: seed.priority,
                    list_key: request.list_key.clone(),
                    goal_id: Some(goal.id.clone()),
                    parent_template_id: Some(root_template.id.clone()),
                    owner: request.owner.clone(),
                    project_id: project_id.clone(),
                    blocked_by,
                })
                .await?;
            templates_by_suffix.insert(seed.slug_suffix.to_string(), template.id.clone());
            templates.push(template);
        }

        let instantiated = if request.instantiate {
            Some(
                self.instantiate_task_template(InstantiateTaskTemplate {
                    template_id: root_template.id.clone(),
                    list_key: request.list_key.clone(),
                    goal_id: Some(goal.id.clone()),
                    parent_task_id: None,
                    owner: request.owner.clone(),
                    project_id,
                })
                .await?,
            )
        } else {
            None
        };

        Ok(GoalPlanResult {
            goal_id: goal.id,
            preset: request.preset,
            root_template,
            templates,
            instantiated,
        })
    }

    pub(crate) async fn import_goal_record(
        &self,
        id: &str,
        slug: Option<String>,
        title: String,
        description: Option<String>,
        status: GoalStatus,
        project_id: Option<String>,
        parent_goal_id: Option<String>,
    ) -> Result<Goal> {
        let now = now_iso();
        let created_at = self
            .get_goal(id)
            .await?
            .map(|goal| goal.created_at)
            .unwrap_or_else(|| now.clone());
        sqlx::query(
            "INSERT INTO goals
             (id, slug, title, description, status, project_id, parent_goal_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                slug = excluded.slug,
                title = excluded.title,
                description = excluded.description,
                status = excluded.status,
                project_id = excluded.project_id,
                parent_goal_id = excluded.parent_goal_id,
                updated_at = excluded.updated_at",
        )
        .bind(id)
        .bind(&slug)
        .bind(&title)
        .bind(&description)
        .bind(status.as_str())
        .bind(&project_id)
        .bind(&parent_goal_id)
        .bind(&created_at)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.get_goal(id)
            .await?
            .ok_or_else(|| Error::message("goal import did not persist"))
    }

    pub async fn list_task_templates(
        &self,
        filter: &TaskTemplateFilter,
    ) -> Result<Vec<TaskTemplate>> {
        let rows = sqlx::query_as::<_, TaskTemplateRow>(
            "SELECT * FROM task_templates
             WHERE (?1 IS NULL OR goal_id = ?1)
               AND (?2 IS NULL OR project_id = ?2)
               AND (?3 IS NULL OR parent_template_id = ?3)
             ORDER BY updated_at DESC, created_at DESC",
        )
        .bind(&filter.goal_id)
        .bind(&filter.project_id)
        .bind(&filter.parent_template_id)
        .fetch_all(&self.pool)
        .await?;
        let mut templates = Vec::with_capacity(rows.len());
        for row in rows {
            templates.push(row.into_template(&self.pool).await?);
        }
        Ok(templates)
    }

    pub async fn get_task_template(&self, id: &str) -> Result<Option<TaskTemplate>> {
        let row = sqlx::query_as::<_, TaskTemplateRow>("SELECT * FROM task_templates WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(row) => Ok(Some(row.into_template(&self.pool).await?)),
            None => Ok(None),
        }
    }

    pub async fn create_task_template(&self, template: NewTaskTemplate) -> Result<TaskTemplate> {
        let id = new_id();
        let now = now_iso();
        sqlx::query(
            "INSERT INTO task_templates
             (id, slug, title, description, priority, list_key, goal_id, parent_template_id, owner, project_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&template.slug)
        .bind(&template.title)
        .bind(&template.description)
        .bind(template.priority.as_str())
        .bind(&template.list_key)
        .bind(&template.goal_id)
        .bind(&template.parent_template_id)
        .bind(&template.owner)
        .bind(&template.project_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.replace_template_dependencies(&id, &template.blocked_by)
            .await?;
        self.get_task_template(&id)
            .await?
            .ok_or_else(|| Error::message("task template was not created"))
    }

    pub async fn update_task_template(
        &self,
        id: &str,
        patch: TaskTemplatePatch,
    ) -> Result<Option<TaskTemplate>> {
        let Some(existing) = self.get_task_template(id).await? else {
            return Ok(None);
        };
        let now = now_iso();
        sqlx::query(
            "UPDATE task_templates
             SET slug = ?, title = ?, description = ?, priority = ?, list_key = ?, goal_id = ?, parent_template_id = ?, owner = ?, project_id = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(patch.slug.or(existing.slug))
        .bind(patch.title.unwrap_or(existing.title))
        .bind(patch.description.or(existing.description))
        .bind(patch.priority.unwrap_or(existing.priority).as_str())
        .bind(patch.list_key.or(existing.list_key))
        .bind(patch.goal_id.or(existing.goal_id))
        .bind(patch.parent_template_id.or(existing.parent_template_id))
        .bind(patch.owner.or(existing.owner))
        .bind(patch.project_id.or(existing.project_id))
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        if let Some(blocked_by) = patch.blocked_by {
            self.replace_template_dependencies(id, &blocked_by).await?;
        }
        self.get_task_template(id).await
    }

    pub async fn instantiate_task_template(
        &self,
        params: InstantiateTaskTemplate,
    ) -> Result<TemplateInstantiationResult> {
        self.instantiate_template_internal(params).await
    }

    pub async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<Task>> {
        let rows = sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks
             WHERE (?1 IS NULL OR list_key = ?1)
               AND (?2 IS NULL OR goal_id = ?2)
               AND (?3 IS NULL OR project_id = ?3)
               AND (?4 IS NULL OR session_key = ?4)
               AND (?5 IS NULL OR owner = ?5)
               AND (?6 IS NULL OR status = ?6)
             ORDER BY updated_at DESC, created_at DESC",
        )
        .bind(&filter.list_key)
        .bind(&filter.goal_id)
        .bind(&filter.project_id)
        .bind(&filter.session_key)
        .bind(&filter.owner)
        .bind(filter.status.map(|status| status.as_str()))
        .fetch_all(&self.pool)
        .await?;
        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            tasks.push(row.into_task(&self.pool).await?);
        }
        Ok(tasks)
    }

    pub async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        let row = sqlx::query_as::<_, TaskRow>("SELECT * FROM tasks WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(row) => Ok(Some(row.into_task(&self.pool).await?)),
            None => Ok(None),
        }
    }

    pub async fn create_task(&self, task: NewTask) -> Result<Task> {
        let id = new_id();
        let now = now_iso();
        let claimed_at = task.session_key.as_ref().map(|_| now.clone());
        sqlx::query(
            "INSERT INTO tasks
             (id, list_key, title, description, status, priority, goal_id, parent_task_id, owner, project_id, session_key, claimed_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&task.list_key)
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status.as_str())
        .bind(task.priority.as_str())
        .bind(&task.goal_id)
        .bind(&task.parent_task_id)
        .bind(&task.owner)
        .bind(&task.project_id)
        .bind(&task.session_key)
        .bind(&claimed_at)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.replace_dependencies(&id, &task.blocked_by).await?;
        self.get_task(&id)
            .await?
            .ok_or_else(|| Error::message("task was not created"))
    }

    pub async fn update_task(&self, id: &str, patch: TaskPatch) -> Result<Option<Task>> {
        let Some(existing) = self.get_task(id).await? else {
            return Ok(None);
        };
        let now = now_iso();
        let next_status = patch.status.unwrap_or(existing.status);
        let next_session_key = patch.session_key.or(existing.session_key);
        let claimed_at = if next_status == TaskStatus::InProgress && next_session_key.is_some() {
            existing.claimed_at.or_else(|| Some(now.clone()))
        } else if matches!(next_status, TaskStatus::Completed | TaskStatus::Cancelled) {
            None
        } else {
            existing.claimed_at
        };
        sqlx::query(
            "UPDATE tasks
             SET list_key = ?, title = ?, description = ?, status = ?, priority = ?, goal_id = ?, parent_task_id = ?, owner = ?, project_id = ?, session_key = ?, claimed_at = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(patch.list_key.or(existing.list_key))
        .bind(patch.title.unwrap_or(existing.title))
        .bind(patch.description.or(existing.description))
        .bind(next_status.as_str())
        .bind(patch.priority.unwrap_or(existing.priority).as_str())
        .bind(patch.goal_id.or(existing.goal_id))
        .bind(patch.parent_task_id.or(existing.parent_task_id))
        .bind(patch.owner.or(existing.owner))
        .bind(patch.project_id.or(existing.project_id))
        .bind(&next_session_key)
        .bind(&claimed_at)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        if let Some(blocked_by) = patch.blocked_by {
            self.replace_dependencies(id, &blocked_by).await?;
        }
        self.get_task(id).await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn import_task_record(
        &self,
        id: &str,
        list_key: Option<String>,
        title: String,
        description: Option<String>,
        status: TaskStatus,
        priority: TaskPriority,
        goal_id: Option<String>,
        parent_task_id: Option<String>,
        owner: Option<String>,
        project_id: Option<String>,
        blocked_by: Vec<String>,
    ) -> Result<Task> {
        let now = now_iso();
        let existing = self.get_task(id).await?;
        let created_at = existing
            .as_ref()
            .map(|task| task.created_at.clone())
            .unwrap_or_else(|| now.clone());
        let session_key = existing.as_ref().and_then(|task| task.session_key.clone());
        let claimed_at = existing.as_ref().and_then(|task| task.claimed_at.clone());
        sqlx::query(
            "INSERT INTO tasks
             (id, list_key, title, description, status, priority, goal_id, parent_task_id, owner, project_id, session_key, claimed_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                list_key = excluded.list_key,
                title = excluded.title,
                description = excluded.description,
                status = excluded.status,
                priority = excluded.priority,
                goal_id = excluded.goal_id,
                parent_task_id = excluded.parent_task_id,
                owner = excluded.owner,
                project_id = excluded.project_id,
                updated_at = excluded.updated_at",
        )
        .bind(id)
        .bind(&list_key)
        .bind(&title)
        .bind(&description)
        .bind(status.as_str())
        .bind(priority.as_str())
        .bind(&goal_id)
        .bind(&parent_task_id)
        .bind(&owner)
        .bind(&project_id)
        .bind(&session_key)
        .bind(&claimed_at)
        .bind(&created_at)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.replace_dependencies(id, &blocked_by).await?;
        self.get_task(id)
            .await?
            .ok_or_else(|| Error::message("task import did not persist"))
    }

    pub async fn claim_task(
        &self,
        id: &str,
        owner: Option<String>,
        session_key: Option<String>,
    ) -> Result<Option<Task>> {
        let Some(existing) = self.get_task(id).await? else {
            return Ok(None);
        };
        if existing.status != TaskStatus::Pending && existing.status != TaskStatus::Blocked {
            return Err(Error::message(format!(
                "task {id} cannot be claimed from status {}",
                existing.status.as_str()
            )));
        }
        let blocked = self.list_incomplete_dependencies(id).await?;
        if !blocked.is_empty() {
            return Err(Error::message(format!(
                "task {id} is blocked by incomplete tasks: {}",
                blocked.join(", ")
            )));
        }
        self.update_task(id, TaskPatch {
            owner,
            session_key,
            status: Some(TaskStatus::InProgress),
            ..TaskPatch::default()
        })
        .await
    }

    pub async fn list_incomplete_dependencies(&self, task_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT d.depends_on_task_id
             FROM task_dependencies d
             JOIN tasks t ON t.id = d.depends_on_task_id
             WHERE d.task_id = ?
               AND t.status NOT IN ('completed', 'cancelled')
             ORDER BY d.depends_on_task_id",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_comments(&self, task_id: &str) -> Result<Vec<TaskComment>> {
        let rows = sqlx::query_as::<_, TaskCommentRow>(
            "SELECT * FROM task_comments WHERE task_id = ? ORDER BY created_at ASC",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn add_comment(&self, comment: NewTaskComment) -> Result<TaskComment> {
        let id = new_id();
        let now = now_iso();
        sqlx::query(
            "INSERT INTO task_comments (id, task_id, author, body, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&comment.task_id)
        .bind(&comment.author)
        .bind(&comment.body)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let row = sqlx::query_as::<_, TaskCommentRow>("SELECT * FROM task_comments WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.into())
    }

    pub async fn list_external_links(
        &self,
        filter: &ExternalTaskLinkFilter,
    ) -> Result<Vec<ExternalTaskLink>> {
        let rows = sqlx::query_as::<_, ExternalTaskLinkRow>(
            "SELECT *
             FROM task_external_links
             WHERE (?1 IS NULL OR task_id = ?1)
               AND (?2 IS NULL OR provider = ?2)
             ORDER BY updated_at DESC
             LIMIT ?3",
        )
        .bind(&filter.task_id)
        .bind(&filter.provider)
        .bind(i64::from(filter.limit.unwrap_or(50)))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn import_tracker_task(
        &self,
        request: TrackerImportRequest,
    ) -> Result<TrackerImportResult> {
        let existing_link = sqlx::query_as::<_, ExternalTaskLinkRow>(
            "SELECT *
             FROM task_external_links
             WHERE provider = ? AND remote_id = ?",
        )
        .bind(&request.provider)
        .bind(&request.remote_id)
        .fetch_optional(&self.pool)
        .await?;

        let existing_task_id = existing_link.as_ref().map(|link| link.task_id.clone());
        let resolved_task_id = request.task_id.clone().or(existing_task_id);
        let (task, created_task) = if let Some(task_id) = resolved_task_id {
            let task = self
                .update_task(&task_id, TaskPatch {
                    list_key: request.list_key.clone(),
                    title: Some(request.remote_title.clone()),
                    description: request.remote_body.clone(),
                    status: Some(map_remote_status(&request.remote_status)),
                    goal_id: request.goal_id.clone(),
                    owner: request.owner.clone(),
                    project_id: request.project_id.clone(),
                    ..TaskPatch::default()
                })
                .await?
                .ok_or_else(|| Error::message(format!("task {task_id} not found")))?;
            (task, false)
        } else {
            let task = self
                .create_task(NewTask {
                    list_key: request.list_key.clone(),
                    title: request.remote_title.clone(),
                    description: request.remote_body.clone(),
                    status: map_remote_status(&request.remote_status),
                    priority: TaskPriority::Medium,
                    goal_id: request.goal_id.clone(),
                    parent_task_id: None,
                    owner: request.owner.clone(),
                    project_id: request.project_id.clone(),
                    session_key: None,
                    blocked_by: Vec::new(),
                })
                .await?;
            (task, true)
        };

        let now = now_iso();
        let created_at = existing_link
            .as_ref()
            .map(|link| link.created_at.clone())
            .unwrap_or_else(|| now.clone());
        let link_id = existing_link
            .as_ref()
            .map(|link| link.id.clone())
            .unwrap_or_else(new_id);
        sqlx::query(
            "INSERT INTO task_external_links
             (id, task_id, provider, remote_id, remote_title, remote_status, remote_url, remote_updated_at, synced_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(provider, remote_id) DO UPDATE SET
                task_id = excluded.task_id,
                remote_title = excluded.remote_title,
                remote_status = excluded.remote_status,
                remote_url = excluded.remote_url,
                remote_updated_at = excluded.remote_updated_at,
                synced_at = excluded.synced_at,
                updated_at = excluded.updated_at",
        )
        .bind(&link_id)
        .bind(&task.id)
        .bind(&request.provider)
        .bind(&request.remote_id)
        .bind(&request.remote_title)
        .bind(&request.remote_status)
        .bind(&request.remote_url)
        .bind(&request.remote_updated_at)
        .bind(&now)
        .bind(&created_at)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let link = sqlx::query_as::<_, ExternalTaskLinkRow>(
            "SELECT *
             FROM task_external_links
             WHERE provider = ? AND remote_id = ?",
        )
        .bind(&request.provider)
        .bind(&request.remote_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TrackerImportResult {
            task,
            link: link.into(),
            created_task,
        })
    }

    pub async fn list_runs(&self, task_id: Option<&str>) -> Result<Vec<TaskRun>> {
        let rows = sqlx::query_as::<_, TaskRunRow>(
            "SELECT * FROM task_runs WHERE (?1 IS NULL OR task_id = ?1) ORDER BY updated_at DESC, created_at DESC",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn create_run(&self, run: NewTaskRun) -> Result<TaskRun> {
        let id = new_id();
        let now = now_iso();
        sqlx::query(
            "INSERT INTO task_runs
             (id, task_id, runtime_run_id, session_key, agent_id, provider, model, status, summary, input_tokens, output_tokens, request_input_tokens, request_output_tokens, duration_ms, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, NULL, NULL, ?, ?)",
        )
        .bind(&id)
        .bind(&run.task_id)
        .bind(&run.runtime_run_id)
        .bind(&run.session_key)
        .bind(&run.agent_id)
        .bind(&run.provider)
        .bind(&run.model)
        .bind(run.status.as_str())
        .bind(&run.summary)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let row = sqlx::query_as::<_, TaskRunRow>("SELECT * FROM task_runs WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        row.try_into()
    }

    pub async fn update_run(&self, id: &str, patch: TaskRunPatch) -> Result<Option<TaskRun>> {
        let row = sqlx::query_as::<_, TaskRunRow>("SELECT * FROM task_runs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        let Some(existing) = row else {
            return Ok(None);
        };
        let status = patch
            .status
            .unwrap_or(existing.status.parse::<RunStatus>()?);
        let summary = patch.summary.or(existing.summary);
        let now = now_iso();
        sqlx::query(
            "UPDATE task_runs
             SET status = ?, summary = ?, input_tokens = ?, output_tokens = ?, request_input_tokens = ?, request_output_tokens = ?, duration_ms = ?, updated_at = ?
             WHERE id = ?",
        )
            .bind(status.as_str())
            .bind(summary)
            .bind(patch.input_tokens.or(existing.input_tokens.map(|value| value as u32)))
            .bind(patch.output_tokens.or(existing.output_tokens.map(|value| value as u32)))
            .bind(
                patch
                    .request_input_tokens
                    .or(existing.request_input_tokens.map(|value| value as u32)),
            )
            .bind(
                patch
                    .request_output_tokens
                    .or(existing.request_output_tokens.map(|value| value as u32)),
            )
            .bind(
                patch
                    .duration_ms
                    .or(existing.duration_ms.map(|value| value as u64))
                    .map(|value| value as i64),
            )
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        let row = sqlx::query_as::<_, TaskRunRow>("SELECT * FROM task_runs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(TryInto::try_into).transpose()
    }

    pub async fn list_approvals(&self, task_id: Option<&str>) -> Result<Vec<TaskApproval>> {
        let rows = sqlx::query_as::<_, TaskApprovalRow>(
            "SELECT * FROM task_approvals WHERE (?1 IS NULL OR task_id = ?1) ORDER BY updated_at DESC, created_at DESC",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn create_approval(&self, approval: NewTaskApproval) -> Result<TaskApproval> {
        let id = new_id();
        let now = now_iso();
        sqlx::query(
            "INSERT INTO task_approvals (id, task_id, kind, status, reason, created_at, updated_at, resolved_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, NULL)",
        )
        .bind(&id)
        .bind(&approval.task_id)
        .bind(&approval.kind)
        .bind(approval.status.as_str())
        .bind(&approval.reason)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let row = sqlx::query_as::<_, TaskApprovalRow>("SELECT * FROM task_approvals WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        row.try_into()
    }

    pub async fn update_approval(
        &self,
        id: &str,
        patch: TaskApprovalPatch,
    ) -> Result<Option<TaskApproval>> {
        let now = now_iso();
        let resolved_at = match patch.status {
            ApprovalStatus::Pending => None,
            ApprovalStatus::Approved | ApprovalStatus::Rejected => Some(now.clone()),
        };
        let rows = sqlx::query(
            "UPDATE task_approvals SET status = ?, reason = ?, updated_at = ?, resolved_at = ? WHERE id = ?",
        )
        .bind(patch.status.as_str())
        .bind(&patch.reason)
        .bind(&now)
        .bind(&resolved_at)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();
        if rows == 0 {
            return Ok(None);
        }
        let row = sqlx::query_as::<_, TaskApprovalRow>("SELECT * FROM task_approvals WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(TryInto::try_into).transpose()
    }

    pub async fn list_costs(&self, filter: &TaskCostFilter) -> Result<Vec<TaskCostAttribution>> {
        let limit = i64::from(filter.limit.unwrap_or(20));
        let rows = sqlx::query_as::<_, TaskCostAttributionRow>(
            "SELECT *
             FROM task_cost_attribution
             WHERE (?1 IS NULL OR task_id = ?1)
               AND (?2 IS NULL OR goal_id = ?2)
               AND (?3 IS NULL OR project_id = ?3)
               AND (?4 IS NULL OR session_key = ?4)
             ORDER BY created_at DESC
             LIMIT ?5",
        )
        .bind(&filter.task_id)
        .bind(&filter.goal_id)
        .bind(&filter.project_id)
        .bind(&filter.session_key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn record_cost(&self, entry: NewTaskCostAttribution) -> Result<TaskCostAttribution> {
        let task = self.task_required(&entry.task_id).await?;
        let id = new_id();
        let now = now_iso();
        let total_tokens = entry.input_tokens.saturating_add(entry.output_tokens);
        sqlx::query(
            "INSERT INTO task_cost_attribution
             (id, task_id, task_run_id, goal_id, project_id, session_key, runtime_run_id, provider, model, input_tokens, output_tokens, request_input_tokens, request_output_tokens, total_tokens, estimated_cost_usd, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&entry.task_id)
        .bind(&entry.task_run_id)
        .bind(&task.goal_id)
        .bind(&task.project_id)
        .bind(entry.session_key.as_ref().or(task.session_key.as_ref()))
        .bind(&entry.runtime_run_id)
        .bind(&entry.provider)
        .bind(&entry.model)
        .bind(entry.input_tokens as i64)
        .bind(entry.output_tokens as i64)
        .bind(entry.request_input_tokens.map(|value| value as i64))
        .bind(entry.request_output_tokens.map(|value| value as i64))
        .bind(total_tokens as i64)
        .bind(entry.estimated_cost_usd)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let row = sqlx::query_as::<_, TaskCostAttributionRow>(
            "SELECT * FROM task_cost_attribution WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn list_budgets(
        &self,
        scope: Option<BudgetScope>,
        scope_id: Option<&str>,
    ) -> Result<Vec<WorkBudget>> {
        let rows = sqlx::query_as::<_, WorkBudgetRow>(
            "SELECT *
             FROM work_budgets
             WHERE (?1 IS NULL OR scope = ?1)
               AND (?2 IS NULL OR scope_id = ?2)
             ORDER BY scope ASC, scope_id ASC",
        )
        .bind(scope.map(BudgetScope::as_str))
        .bind(scope_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn save_budget(&self, budget: NewWorkBudget) -> Result<WorkBudget> {
        let now = now_iso();
        let existing = sqlx::query_as::<_, WorkBudgetRow>(
            "SELECT * FROM work_budgets WHERE scope = ? AND scope_id = ?",
        )
        .bind(budget.scope.as_str())
        .bind(&budget.scope_id)
        .fetch_optional(&self.pool)
        .await?;

        let id = existing
            .as_ref()
            .map(|row| row.id.clone())
            .unwrap_or_else(new_id);
        sqlx::query(
            "INSERT INTO work_budgets
             (id, scope, scope_id, soft_limit_tokens, hard_limit_tokens, soft_limit_usd, hard_limit_usd, note, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(scope, scope_id) DO UPDATE SET
                soft_limit_tokens = excluded.soft_limit_tokens,
                hard_limit_tokens = excluded.hard_limit_tokens,
                soft_limit_usd = excluded.soft_limit_usd,
                hard_limit_usd = excluded.hard_limit_usd,
                note = excluded.note,
                updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(budget.scope.as_str())
        .bind(&budget.scope_id)
        .bind(budget.soft_limit_tokens.map(|value| value as i64))
        .bind(budget.hard_limit_tokens.map(|value| value as i64))
        .bind(budget.soft_limit_usd)
        .bind(budget.hard_limit_usd)
        .bind(&budget.note)
        .bind(existing.as_ref().map_or(&now, |row| &row.created_at))
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let row = sqlx::query_as::<_, WorkBudgetRow>("SELECT * FROM work_budgets WHERE id = ?")
            .bind(&id)
            .fetch_one(&self.pool)
            .await?;
        row.try_into()
    }

    pub async fn budget_statuses_for_task(&self, task_id: &str) -> Result<Vec<BudgetStatus>> {
        let task = self.task_required(task_id).await?;
        let mut budgets = self
            .list_budgets(Some(BudgetScope::Task), Some(task_id))
            .await?;
        if let Some(goal_id) = task.goal_id.as_deref() {
            budgets.extend(
                self.list_budgets(Some(BudgetScope::Goal), Some(goal_id))
                    .await?,
            );
        }
        if let Some(project_id) = task.project_id.as_deref() {
            budgets.extend(
                self.list_budgets(Some(BudgetScope::Project), Some(project_id))
                    .await?,
            );
        }

        let mut statuses = Vec::with_capacity(budgets.len());
        for budget in budgets {
            statuses.push(self.budget_status_for(budget).await?);
        }
        Ok(statuses)
    }

    pub async fn enforce_task_budget(&self, task_id: &str) -> Result<BudgetCheck> {
        let scopes = self.budget_statuses_for_task(task_id).await?;
        let hard_exceeded = scopes.iter().any(|status| status.hard_exceeded);
        if !hard_exceeded || self.has_approved_budget_override(task_id).await? {
            return Ok(BudgetCheck {
                allowed: true,
                task_id: task_id.to_string(),
                requires_approval: false,
                pending_approval_id: None,
                scopes,
            });
        }

        let pending_approval_id = self
            .ensure_pending_budget_override(
                task_id,
                Some("Task budget exceeded, operator approval required to continue".to_string()),
            )
            .await?;
        Ok(BudgetCheck {
            allowed: false,
            task_id: task_id.to_string(),
            requires_approval: true,
            pending_approval_id: Some(pending_approval_id),
            scopes,
        })
    }

    pub async fn list_recurring_tasks(
        &self,
        filter: &RecurringTaskFilter,
    ) -> Result<Vec<RecurringTask>> {
        let rows = sqlx::query_as::<_, RecurringTaskRow>(
            "SELECT *
             FROM recurring_tasks
             WHERE (?1 IS NULL OR template_id = ?1)
               AND (?2 IS NULL OR goal_id = ?2)
               AND (?3 IS NULL OR project_id = ?3)
               AND (?4 IS NULL OR enabled = ?4)
             ORDER BY updated_at DESC, created_at DESC",
        )
        .bind(&filter.template_id)
        .bind(&filter.goal_id)
        .bind(&filter.project_id)
        .bind(filter.enabled.map(|enabled| {
            if enabled {
                1_i64
            } else {
                0_i64
            }
        }))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn get_recurring_task(&self, id: &str) -> Result<Option<RecurringTask>> {
        let row =
            sqlx::query_as::<_, RecurringTaskRow>("SELECT * FROM recurring_tasks WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        row.map(TryInto::try_into).transpose()
    }

    pub async fn create_recurring_task(&self, task: NewRecurringTask) -> Result<RecurringTask> {
        self.template_required(&task.template_id).await?;
        let id = new_id();
        let now = now_iso();
        let next_run_at_ms = if task.enabled {
            moltis_cron::schedule::compute_next_run(&task.schedule, now_ms())
                .map_err(|error| Error::message(error.to_string()))?
                .map(|value| value as i64)
        } else {
            None
        };
        sqlx::query(
            "INSERT INTO recurring_tasks
             (id, name, template_id, schedule_json, enabled, list_key, goal_id, owner, project_id, next_run_at_ms, last_materialized_at_ms, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)",
        )
        .bind(&id)
        .bind(&task.name)
        .bind(&task.template_id)
        .bind(serde_json::to_string(&task.schedule)?)
        .bind(if task.enabled { 1_i64 } else { 0_i64 })
        .bind(&task.list_key)
        .bind(&task.goal_id)
        .bind(&task.owner)
        .bind(&task.project_id)
        .bind(next_run_at_ms)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.get_recurring_task(&id)
            .await?
            .ok_or_else(|| Error::message("recurring task was not created"))
    }

    pub async fn update_recurring_task(
        &self,
        id: &str,
        patch: RecurringTaskPatch,
    ) -> Result<Option<RecurringTask>> {
        let Some(existing) = self.get_recurring_task(id).await? else {
            return Ok(None);
        };

        let template_id = patch.template_id.unwrap_or(existing.template_id);
        self.template_required(&template_id).await?;

        let recompute_next_run = patch.enabled.is_some() || patch.schedule.is_some();
        let schedule = patch.schedule.unwrap_or(existing.schedule);
        let enabled = patch.enabled.unwrap_or(existing.enabled);
        let next_run_at_ms = if enabled && recompute_next_run {
            moltis_cron::schedule::compute_next_run(&schedule, now_ms())
                .map_err(|error| Error::message(error.to_string()))?
                .map(|value| value as i64)
        } else if enabled {
            existing.next_run_at_ms.map(|value| value as i64)
        } else {
            None
        };

        let now = now_iso();
        sqlx::query(
            "UPDATE recurring_tasks
             SET name = ?, template_id = ?, schedule_json = ?, enabled = ?, list_key = ?, goal_id = ?, owner = ?, project_id = ?, next_run_at_ms = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(patch.name.unwrap_or(existing.name))
        .bind(&template_id)
        .bind(serde_json::to_string(&schedule)?)
        .bind(if enabled { 1_i64 } else { 0_i64 })
        .bind(patch.list_key.or(existing.list_key))
        .bind(patch.goal_id.or(existing.goal_id))
        .bind(patch.owner.or(existing.owner))
        .bind(patch.project_id.or(existing.project_id))
        .bind(next_run_at_ms)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.get_recurring_task(id).await
    }

    pub async fn list_recurring_materializations(
        &self,
        recurring_task_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<RecurringTaskMaterialization>> {
        let rows = sqlx::query_as::<_, RecurringTaskMaterializationRow>(
            "SELECT *
             FROM recurring_task_materializations
             WHERE (?1 IS NULL OR recurring_task_id = ?1)
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .bind(recurring_task_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn materialize_due_recurring_tasks(
        &self,
        recurring_task_id: Option<&str>,
        limit: Option<u32>,
        force: bool,
        now_ms_override: Option<u64>,
    ) -> Result<RecurringTaskMaterializeResult> {
        let evaluation_ms = now_ms_override.unwrap_or_else(now_ms);
        let limit = i64::from(limit.unwrap_or(20));
        let rows = sqlx::query_as::<_, RecurringTaskRow>(
            "SELECT *
             FROM recurring_tasks
             WHERE (?1 IS NULL OR id = ?1)
               AND enabled = 1
               AND (?2 = 1 OR (next_run_at_ms IS NOT NULL AND next_run_at_ms <= ?3))
             ORDER BY next_run_at_ms ASC, created_at ASC
             LIMIT ?4",
        )
        .bind(recurring_task_id)
        .bind(if force {
            1_i64
        } else {
            0_i64
        })
        .bind(evaluation_ms as i64)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut materializations = Vec::with_capacity(rows.len());
        for row in rows {
            let recurring: RecurringTask = row.try_into()?;
            let scheduled_for_ms = recurring.next_run_at_ms.unwrap_or(evaluation_ms);
            let created = self
                .instantiate_template_internal(InstantiateTaskTemplate {
                    template_id: recurring.template_id.clone(),
                    list_key: recurring.list_key.clone(),
                    goal_id: recurring.goal_id.clone(),
                    parent_task_id: None,
                    owner: recurring.owner.clone(),
                    project_id: recurring.project_id.clone(),
                })
                .await?;

            let materialization_id = new_id();
            let created_at = now_iso();
            sqlx::query(
                "INSERT INTO recurring_task_materializations
                 (id, recurring_task_id, template_id, root_task_id, scheduled_for_ms, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&materialization_id)
            .bind(&recurring.id)
            .bind(&recurring.template_id)
            .bind(&created.root_task_id)
            .bind(scheduled_for_ms as i64)
            .bind(&created_at)
            .execute(&self.pool)
            .await?;

            let next_run_at_ms =
                moltis_cron::schedule::compute_next_run(&recurring.schedule, evaluation_ms)
                    .map_err(|error| Error::message(error.to_string()))?
                    .map(|value| value as i64);
            sqlx::query(
                "UPDATE recurring_tasks
                 SET next_run_at_ms = ?, last_materialized_at_ms = ?, updated_at = ?
                 WHERE id = ?",
            )
            .bind(next_run_at_ms)
            .bind(evaluation_ms as i64)
            .bind(now_iso())
            .bind(&recurring.id)
            .execute(&self.pool)
            .await?;

            materializations.push(RecurringTaskMaterialization {
                id: materialization_id,
                recurring_task_id: recurring.id,
                template_id: recurring.template_id,
                root_task_id: created.root_task_id,
                scheduled_for_ms,
                created_at,
            });
        }

        Ok(RecurringTaskMaterializeResult {
            count: materializations.len(),
            materializations,
        })
    }

    pub async fn portfolio(&self) -> Result<WorkPortfolio> {
        let goals = self.list_goals().await?;
        let tasks = self.list_tasks(&TaskFilter::default()).await?;
        let approvals = self.list_approvals(None).await?;
        let runs = self.list_runs(None).await?;
        let budgets = self.list_budgets(None, None).await?;
        let task_map: HashMap<String, Task> = tasks
            .iter()
            .cloned()
            .map(|task| (task.id.clone(), task))
            .collect();

        let mut projects: BTreeMap<Option<String>, ProjectWorkSummary> = BTreeMap::new();
        for goal in goals {
            let entry =
                projects
                    .entry(goal.project_id.clone())
                    .or_insert_with(|| ProjectWorkSummary {
                        project_id: goal.project_id.clone(),
                        ..ProjectWorkSummary::default()
                    });
            entry.goal_count += 1;
        }

        for task in &tasks {
            let entry =
                projects
                    .entry(task.project_id.clone())
                    .or_insert_with(|| ProjectWorkSummary {
                        project_id: task.project_id.clone(),
                        ..ProjectWorkSummary::default()
                    });
            match task.status {
                TaskStatus::Pending => entry.task_counts.pending += 1,
                TaskStatus::InProgress => entry.task_counts.in_progress += 1,
                TaskStatus::Blocked => entry.task_counts.blocked += 1,
                TaskStatus::Completed => entry.task_counts.completed += 1,
                TaskStatus::Cancelled => entry.task_counts.cancelled += 1,
            }
        }

        for approval in approvals {
            if approval.status != ApprovalStatus::Pending {
                continue;
            }
            let project_id = approval
                .task_id
                .as_ref()
                .and_then(|task_id| task_map.get(task_id))
                .and_then(|task| task.project_id.clone());
            let entry = projects
                .entry(project_id.clone())
                .or_insert_with(|| ProjectWorkSummary {
                    project_id,
                    ..ProjectWorkSummary::default()
                });
            entry.pending_approvals += 1;
        }

        for run in runs {
            if !matches!(run.status, RunStatus::Pending | RunStatus::Running) {
                continue;
            }
            let Some(task) = task_map.get(&run.task_id) else {
                continue;
            };
            let entry =
                projects
                    .entry(task.project_id.clone())
                    .or_insert_with(|| ProjectWorkSummary {
                        project_id: task.project_id.clone(),
                        ..ProjectWorkSummary::default()
                    });
            entry.active_runs += 1;
        }

        for budget in budgets {
            let status = self.budget_status_for(budget).await?;
            if !(status.soft_exceeded || status.hard_exceeded) {
                continue;
            }
            let project_id = match status.budget.scope {
                BudgetScope::Project => Some(status.budget.scope_id.clone()),
                BudgetScope::Task => task_map
                    .get(&status.budget.scope_id)
                    .and_then(|task| task.project_id.clone()),
                BudgetScope::Goal => None,
            };
            let entry = projects
                .entry(project_id.clone())
                .or_insert_with(|| ProjectWorkSummary {
                    project_id,
                    ..ProjectWorkSummary::default()
                });
            entry.budget_alerts += 1;
        }

        let mut cross_project_dependencies = Vec::new();
        for task in &tasks {
            for depends_on_task_id in &task.blocked_by {
                let Some(depends_on) = task_map.get(depends_on_task_id) else {
                    continue;
                };
                if task.project_id == depends_on.project_id {
                    continue;
                }
                cross_project_dependencies.push(CrossProjectDependency {
                    task_id: task.id.clone(),
                    task_title: task.title.clone(),
                    task_project_id: task.project_id.clone(),
                    depends_on_task_id: depends_on.id.clone(),
                    depends_on_task_title: depends_on.title.clone(),
                    depends_on_project_id: depends_on.project_id.clone(),
                });
            }
        }

        let mut project_summaries = projects.into_values().collect::<Vec<_>>();
        project_summaries.sort_by(|left, right| left.project_id.cmp(&right.project_id));
        cross_project_dependencies.sort_by(|left, right| {
            left.task_title
                .cmp(&right.task_title)
                .then(left.depends_on_task_title.cmp(&right.depends_on_task_title))
        });

        Ok(WorkPortfolio {
            projects: project_summaries,
            cross_project_dependencies,
        })
    }

    pub async fn dashboard(&self, project_id: Option<&str>) -> Result<WorkDashboard> {
        let tasks = self
            .list_tasks(&TaskFilter {
                project_id: project_id.map(ToOwned::to_owned),
                ..TaskFilter::default()
            })
            .await?;
        let task_ids: std::collections::HashSet<String> =
            tasks.iter().map(|task| task.id.clone()).collect();

        let counts = tasks
            .iter()
            .fold(WorkTaskCounts::default(), |mut counts, task| {
                match task.status {
                    TaskStatus::Pending => counts.pending += 1,
                    TaskStatus::InProgress => counts.in_progress += 1,
                    TaskStatus::Blocked => counts.blocked += 1,
                    TaskStatus::Completed => counts.completed += 1,
                    TaskStatus::Cancelled => counts.cancelled += 1,
                }
                counts
            });

        let approvals = self.list_approvals(None).await?;
        let runs = self.list_runs(None).await?;
        let recent_costs = self
            .list_costs(&TaskCostFilter {
                project_id: project_id.map(ToOwned::to_owned),
                limit: Some(12),
                ..TaskCostFilter::default()
            })
            .await?;

        let budget_rows = if let Some(project_id) = project_id {
            self.list_budgets(None, Some(project_id)).await?
        } else {
            self.list_budgets(None, None).await?
        };
        let mut budget_statuses = Vec::with_capacity(budget_rows.len());
        for budget in budget_rows {
            budget_statuses.push(self.budget_status_for(budget).await?);
        }

        Ok(WorkDashboard {
            counts,
            pending_tasks: tasks
                .iter()
                .filter(|task| task.status == TaskStatus::Pending)
                .cloned()
                .collect(),
            in_progress_tasks: tasks
                .iter()
                .filter(|task| task.status == TaskStatus::InProgress)
                .cloned()
                .collect(),
            blocked_tasks: tasks
                .iter()
                .filter(|task| task.status == TaskStatus::Blocked)
                .cloned()
                .collect(),
            completed_tasks: tasks
                .iter()
                .filter(|task| task.status == TaskStatus::Completed)
                .cloned()
                .collect(),
            cancelled_tasks: tasks
                .iter()
                .filter(|task| task.status == TaskStatus::Cancelled)
                .cloned()
                .collect(),
            pending_approvals: approvals
                .into_iter()
                .filter(|approval| {
                    approval.status == ApprovalStatus::Pending
                        && approval
                            .task_id
                            .as_ref()
                            .is_none_or(|task_id| task_ids.contains(task_id))
                })
                .collect(),
            active_runs: runs
                .into_iter()
                .filter(|run| {
                    task_ids.contains(&run.task_id)
                        && matches!(run.status, RunStatus::Pending | RunStatus::Running)
                })
                .collect(),
            recent_costs,
            budget_statuses,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult<T> = Result<T>;

    async fn store() -> TestResult<SqliteWorkStore> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
        crate::run_migrations(&pool).await?;
        Ok(SqliteWorkStore::new(pool))
    }

    #[tokio::test]
    async fn create_and_claim_task() -> TestResult<()> {
        let store = store().await?;
        let task = store
            .create_task(NewTask {
                list_key: Some("default".to_string()),
                title: "Ship it".to_string(),
                description: None,
                status: TaskStatus::Pending,
                priority: TaskPriority::High,
                goal_id: None,
                parent_task_id: None,
                owner: None,
                project_id: None,
                session_key: None,
                blocked_by: Vec::new(),
            })
            .await?;
        let claimed = store
            .claim_task(
                &task.id,
                Some("worker".to_string()),
                Some("main".to_string()),
            )
            .await?
            .ok_or_else(|| Error::message("missing claimed task"))?;
        assert_eq!(claimed.status, TaskStatus::InProgress);
        assert_eq!(claimed.session_key.as_deref(), Some("main"));
        Ok(())
    }

    #[tokio::test]
    async fn blocked_task_cannot_be_claimed() -> TestResult<()> {
        let store = store().await?;
        let dependency = store
            .create_task(NewTask {
                list_key: None,
                title: "dep".to_string(),
                description: None,
                status: TaskStatus::Pending,
                priority: TaskPriority::Medium,
                goal_id: None,
                parent_task_id: None,
                owner: None,
                project_id: None,
                session_key: None,
                blocked_by: Vec::new(),
            })
            .await?;
        let blocked = store
            .create_task(NewTask {
                list_key: None,
                title: "blocked".to_string(),
                description: None,
                status: TaskStatus::Pending,
                priority: TaskPriority::Medium,
                goal_id: None,
                parent_task_id: None,
                owner: None,
                project_id: None,
                session_key: None,
                blocked_by: vec![dependency.id.clone()],
            })
            .await?;
        let err = match store
            .claim_task(&blocked.id, None, Some("main".to_string()))
            .await
        {
            Ok(_) => return Err(Error::message("claim should fail")),
            Err(err) => err,
        };
        assert!(err.to_string().contains("blocked by incomplete tasks"));
        Ok(())
    }

    #[tokio::test]
    async fn budget_enforcement_requires_override_when_hard_limit_exceeded() -> TestResult<()> {
        let store = store().await?;
        let task = store
            .create_task(NewTask {
                list_key: Some("default".to_string()),
                title: "Watch the burn".to_string(),
                description: None,
                status: TaskStatus::Pending,
                priority: TaskPriority::High,
                goal_id: None,
                parent_task_id: None,
                owner: None,
                project_id: Some("proj-1".to_string()),
                session_key: None,
                blocked_by: Vec::new(),
            })
            .await?;
        let budget = store
            .save_budget(NewWorkBudget {
                scope: BudgetScope::Task,
                scope_id: task.id.clone(),
                soft_limit_tokens: Some(50),
                hard_limit_tokens: Some(100),
                soft_limit_usd: None,
                hard_limit_usd: None,
                note: Some("keep it tight".to_string()),
            })
            .await?;
        assert_eq!(budget.scope, BudgetScope::Task);

        let run = store
            .create_run(NewTaskRun {
                task_id: task.id.clone(),
                runtime_run_id: Some("runtime-1".to_string()),
                session_key: Some("main".to_string()),
                agent_id: Some("main".to_string()),
                provider: Some("openai".to_string()),
                model: Some("gpt-5.4".to_string()),
                status: RunStatus::Running,
                summary: Some("hello".to_string()),
            })
            .await?;
        let _ = store
            .record_cost(NewTaskCostAttribution {
                task_id: task.id.clone(),
                task_run_id: Some(run.id.clone()),
                session_key: Some("main".to_string()),
                runtime_run_id: Some("runtime-1".to_string()),
                provider: Some("openai".to_string()),
                model: Some("gpt-5.4".to_string()),
                input_tokens: 80,
                output_tokens: 30,
                request_input_tokens: Some(80),
                request_output_tokens: Some(30),
                estimated_cost_usd: None,
            })
            .await?;

        let check = store.enforce_task_budget(&task.id).await?;
        assert!(!check.allowed);
        assert!(check.requires_approval);
        assert!(check.pending_approval_id.is_some());
        assert!(check.scopes.iter().any(|scope| scope.hard_exceeded));
        Ok(())
    }

    #[tokio::test]
    async fn dashboard_includes_running_work_and_recent_costs() -> TestResult<()> {
        let store = store().await?;
        let task = store
            .create_task(NewTask {
                list_key: Some("default".to_string()),
                title: "Ship dashboard".to_string(),
                description: None,
                status: TaskStatus::InProgress,
                priority: TaskPriority::High,
                goal_id: None,
                parent_task_id: None,
                owner: Some("worker".to_string()),
                project_id: Some("proj-2".to_string()),
                session_key: Some("main".to_string()),
                blocked_by: Vec::new(),
            })
            .await?;
        let run = store
            .create_run(NewTaskRun {
                task_id: task.id.clone(),
                runtime_run_id: Some("runtime-2".to_string()),
                session_key: Some("main".to_string()),
                agent_id: Some("main".to_string()),
                provider: Some("anthropic".to_string()),
                model: Some("claude-opus".to_string()),
                status: RunStatus::Running,
                summary: Some("doing the thing".to_string()),
            })
            .await?;
        let _ = store
            .record_cost(NewTaskCostAttribution {
                task_id: task.id.clone(),
                task_run_id: Some(run.id.clone()),
                session_key: Some("main".to_string()),
                runtime_run_id: Some("runtime-2".to_string()),
                provider: Some("anthropic".to_string()),
                model: Some("claude-opus".to_string()),
                input_tokens: 12,
                output_tokens: 6,
                request_input_tokens: Some(12),
                request_output_tokens: Some(6),
                estimated_cost_usd: None,
            })
            .await?;

        let dashboard = store.dashboard(Some("proj-2")).await?;
        assert_eq!(dashboard.counts.in_progress, 1);
        assert_eq!(dashboard.active_runs.len(), 1);
        assert_eq!(dashboard.recent_costs.len(), 1);
        assert_eq!(
            dashboard.active_runs[0].runtime_run_id.as_deref(),
            Some("runtime-2")
        );
        Ok(())
    }

    #[tokio::test]
    async fn instantiate_template_tree_creates_tasks_and_dependencies() -> TestResult<()> {
        let store = store().await?;
        let root = store
            .create_task_template(NewTaskTemplate {
                slug: Some("release-checklist".to_string()),
                title: "Release checklist".to_string(),
                description: Some("Root planning task".to_string()),
                priority: TaskPriority::High,
                list_key: Some("ops".to_string()),
                goal_id: None,
                parent_template_id: None,
                owner: Some("ops".to_string()),
                project_id: Some("proj-template".to_string()),
                blocked_by: Vec::new(),
            })
            .await?;
        let child_a = store
            .create_task_template(NewTaskTemplate {
                slug: None,
                title: "Cut release branch".to_string(),
                description: None,
                priority: TaskPriority::High,
                list_key: None,
                goal_id: None,
                parent_template_id: Some(root.id.clone()),
                owner: None,
                project_id: None,
                blocked_by: Vec::new(),
            })
            .await?;
        let child_b = store
            .create_task_template(NewTaskTemplate {
                slug: None,
                title: "Publish release notes".to_string(),
                description: None,
                priority: TaskPriority::Medium,
                list_key: None,
                goal_id: None,
                parent_template_id: Some(root.id.clone()),
                owner: None,
                project_id: None,
                blocked_by: vec![child_a.id.clone()],
            })
            .await?;

        let result = store
            .instantiate_task_template(InstantiateTaskTemplate {
                template_id: root.id.clone(),
                list_key: Some("shipping".to_string()),
                goal_id: None,
                parent_task_id: None,
                owner: Some("release-bot".to_string()),
                project_id: Some("proj-instantiated".to_string()),
            })
            .await?;

        assert_eq!(result.template_id, root.id);
        assert_eq!(result.count, 3);
        assert_eq!(result.task_ids.len(), 3);

        let tasks = store
            .list_tasks(&TaskFilter {
                project_id: Some("proj-instantiated".to_string()),
                ..TaskFilter::default()
            })
            .await?;
        assert_eq!(tasks.len(), 3);

        let instantiated_root = store
            .get_task(&result.root_task_id)
            .await?
            .ok_or_else(|| Error::message("missing root task"))?;
        assert_eq!(instantiated_root.title, "Release checklist");
        assert_eq!(instantiated_root.owner.as_deref(), Some("release-bot"));
        assert_eq!(instantiated_root.list_key.as_deref(), Some("shipping"));

        let release_notes = tasks
            .iter()
            .find(|task| task.title == "Publish release notes")
            .ok_or_else(|| Error::message("missing release notes task"))?;
        let release_branch = tasks
            .iter()
            .find(|task| task.title == "Cut release branch")
            .ok_or_else(|| Error::message("missing branch task"))?;
        assert_eq!(release_notes.blocked_by, vec![release_branch.id.clone()]);

        let branch_task = store
            .get_task(&release_branch.id)
            .await?
            .ok_or_else(|| Error::message("missing branch task by id"))?;
        assert_eq!(
            branch_task.parent_task_id.as_deref(),
            Some(instantiated_root.id.as_str())
        );

        let _ = child_b;
        Ok(())
    }

    #[tokio::test]
    async fn recurring_tasks_materialize_due_templates() -> TestResult<()> {
        let store = store().await?;
        let template = store
            .create_task_template(NewTaskTemplate {
                slug: Some("daily-triage".to_string()),
                title: "Daily triage".to_string(),
                description: Some("Review inbox and open work".to_string()),
                priority: TaskPriority::Medium,
                list_key: Some("daily".to_string()),
                goal_id: None,
                parent_template_id: None,
                owner: Some("operator".to_string()),
                project_id: Some("proj-recurring".to_string()),
                blocked_by: Vec::new(),
            })
            .await?;
        let recurring = store
            .create_recurring_task(NewRecurringTask {
                name: "Daily triage automation".to_string(),
                template_id: template.id.clone(),
                schedule: moltis_cron::types::CronSchedule::Every {
                    every_ms: 60_000,
                    anchor_ms: Some(0),
                },
                enabled: true,
                list_key: Some("daily".to_string()),
                goal_id: None,
                owner: Some("operator".to_string()),
                project_id: Some("proj-recurring".to_string()),
            })
            .await?;

        let result = store
            .materialize_due_recurring_tasks(Some(&recurring.id), Some(10), true, Some(120_000))
            .await?;
        assert_eq!(result.count, 1);
        assert_eq!(result.materializations.len(), 1);

        let created_task = store
            .get_task(&result.materializations[0].root_task_id)
            .await?
            .ok_or_else(|| Error::message("missing recurring materialized task"))?;
        assert_eq!(created_task.title, "Daily triage");
        assert_eq!(created_task.project_id.as_deref(), Some("proj-recurring"));

        let refreshed = store
            .get_recurring_task(&recurring.id)
            .await?
            .ok_or_else(|| Error::message("missing refreshed recurring task"))?;
        assert_eq!(refreshed.last_materialized_at_ms, Some(120_000));
        assert!(refreshed.next_run_at_ms.is_some());

        let materializations = store
            .list_recurring_materializations(Some(&recurring.id), 10)
            .await?;
        assert_eq!(materializations.len(), 1);
        assert_eq!(materializations[0].root_task_id, created_task.id);
        Ok(())
    }

    #[tokio::test]
    async fn goal_plan_creates_template_tree_and_optional_instantiation() -> TestResult<()> {
        let store = store().await?;
        let goal = store
            .create_goal(NewGoal {
                slug: Some("ship-notifications".to_string()),
                title: "Ship notifications".to_string(),
                description: Some("Deliver the notifications feature.".to_string()),
                status: GoalStatus::Active,
                project_id: Some("proj-goal".to_string()),
                parent_goal_id: None,
            })
            .await?;

        let result = store
            .plan_goal(GoalPlanRequest {
                goal_id: goal.id.clone(),
                preset: GoalPlanPreset::Delivery,
                list_key: Some("delivery".to_string()),
                owner: Some("planner".to_string()),
                project_id: None,
                instantiate: true,
            })
            .await?;

        assert_eq!(result.goal_id, goal.id);
        assert_eq!(result.preset, GoalPlanPreset::Delivery);
        assert_eq!(result.templates.len(), 5);
        assert_eq!(
            result.root_template.goal_id.as_deref(),
            Some(goal.id.as_str())
        );
        assert!(result.instantiated.is_some());

        let tasks = store
            .list_tasks(&TaskFilter {
                goal_id: Some(goal.id),
                ..TaskFilter::default()
            })
            .await?;
        assert_eq!(tasks.len(), 5);
        assert!(
            tasks
                .iter()
                .any(|task| task.title == "Verify Ship notifications")
        );
        Ok(())
    }

    #[tokio::test]
    async fn portfolio_surfaces_cross_project_dependencies() -> TestResult<()> {
        let store = store().await?;
        let upstream = store
            .create_task(NewTask {
                list_key: Some("portfolio".to_string()),
                title: "Upstream blocker".to_string(),
                description: None,
                status: TaskStatus::InProgress,
                priority: TaskPriority::High,
                goal_id: None,
                parent_task_id: None,
                owner: Some("team-a".to_string()),
                project_id: Some("proj-a".to_string()),
                session_key: None,
                blocked_by: Vec::new(),
            })
            .await?;
        let downstream = store
            .create_task(NewTask {
                list_key: Some("portfolio".to_string()),
                title: "Downstream work".to_string(),
                description: None,
                status: TaskStatus::Blocked,
                priority: TaskPriority::High,
                goal_id: None,
                parent_task_id: None,
                owner: Some("team-b".to_string()),
                project_id: Some("proj-b".to_string()),
                session_key: None,
                blocked_by: vec![upstream.id.clone()],
            })
            .await?;
        let _approval = store
            .create_approval(NewTaskApproval {
                task_id: Some(downstream.id.clone()),
                kind: "plan".to_string(),
                reason: Some("Need sign-off".to_string()),
                status: ApprovalStatus::Pending,
            })
            .await?;
        let run = store
            .create_run(NewTaskRun {
                task_id: upstream.id.clone(),
                runtime_run_id: Some("run-1".to_string()),
                session_key: Some("main".to_string()),
                agent_id: Some("agent-a".to_string()),
                provider: None,
                model: None,
                status: RunStatus::Running,
                summary: None,
            })
            .await?;
        let _ = run;

        let portfolio = store.portfolio().await?;
        assert_eq!(portfolio.projects.len(), 2);
        assert_eq!(portfolio.cross_project_dependencies.len(), 1);
        assert_eq!(
            portfolio.cross_project_dependencies[0].task_title,
            "Downstream work"
        );
        assert_eq!(
            portfolio.cross_project_dependencies[0].depends_on_task_title,
            "Upstream blocker"
        );

        let proj_b = portfolio
            .projects
            .iter()
            .find(|project| project.project_id.as_deref() == Some("proj-b"))
            .ok_or_else(|| Error::message("missing proj-b summary"))?;
        assert_eq!(proj_b.task_counts.blocked, 1);
        assert_eq!(proj_b.pending_approvals, 1);
        Ok(())
    }

    #[tokio::test]
    async fn tracker_import_creates_and_updates_linked_task() -> TestResult<()> {
        let store = store().await?;
        let imported = store
            .import_tracker_task(TrackerImportRequest {
                provider: "github".to_string(),
                remote_id: "moltis-org/moltis#42".to_string(),
                remote_title: "Fix work dashboard".to_string(),
                remote_body: Some("Imported issue body".to_string()),
                remote_status: "open".to_string(),
                remote_url: Some("https://github.com/moltis-org/moltis/issues/42".to_string()),
                remote_updated_at: Some("2026-04-02T10:00:00Z".to_string()),
                task_id: None,
                list_key: Some("imports".to_string()),
                goal_id: None,
                owner: Some("sync-bot".to_string()),
                project_id: Some("proj-imports".to_string()),
            })
            .await?;
        assert!(imported.created_task);
        assert_eq!(imported.task.title, "Fix work dashboard");
        assert_eq!(imported.task.status, TaskStatus::Pending);

        let links = store
            .list_external_links(&ExternalTaskLinkFilter {
                provider: Some("github".to_string()),
                ..ExternalTaskLinkFilter::default()
            })
            .await?;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].task_id, imported.task.id);

        let refreshed = store
            .import_tracker_task(TrackerImportRequest {
                provider: "github".to_string(),
                remote_id: "moltis-org/moltis#42".to_string(),
                remote_title: "Fix work dashboard now".to_string(),
                remote_body: Some("Updated body".to_string()),
                remote_status: "in_progress".to_string(),
                remote_url: Some("https://github.com/moltis-org/moltis/issues/42".to_string()),
                remote_updated_at: Some("2026-04-02T11:00:00Z".to_string()),
                task_id: None,
                list_key: Some("imports".to_string()),
                goal_id: None,
                owner: Some("sync-bot".to_string()),
                project_id: Some("proj-imports".to_string()),
            })
            .await?;
        assert!(!refreshed.created_task);
        assert_eq!(refreshed.task.id, imported.task.id);
        assert_eq!(refreshed.task.title, "Fix work dashboard now");
        assert_eq!(refreshed.task.status, TaskStatus::InProgress);
        Ok(())
    }
}
