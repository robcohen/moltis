use {
    moltis_cron::types::CronSchedule,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    #[default]
    Pending,
    Active,
    Blocked,
    Completed,
    Cancelled,
}

impl GoalStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for GoalStatus {
    type Err = crate::Error;

    fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "active" => Ok(Self::Active),
            "blocked" => Ok(Self::Blocked),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(crate::Error::message(format!(
                "unknown goal status: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalPlanPreset {
    #[default]
    Delivery,
    Investigation,
    Maintenance,
}

impl GoalPlanPreset {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delivery => "delivery",
            Self::Investigation => "investigation",
            Self::Maintenance => "maintenance",
        }
    }
}

impl std::str::FromStr for GoalPlanPreset {
    type Err = crate::Error;

    fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "delivery" => Ok(Self::Delivery),
            "investigation" => Ok(Self::Investigation),
            "maintenance" => Ok(Self::Maintenance),
            _ => Err(crate::Error::message(format!(
                "unknown goal plan preset: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
}

impl TaskStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = crate::Error;

    fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "in_progress" => Ok(Self::InProgress),
            "blocked" => Ok(Self::Blocked),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(crate::Error::message(format!(
                "unknown task status: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl TaskPriority {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

impl std::str::FromStr for TaskPriority {
    type Err = crate::Error;

    fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err(crate::Error::message(format!(
                "unknown task priority: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
}

impl ApprovalStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

impl std::str::FromStr for ApprovalStatus {
    type Err = crate::Error;

    fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            _ => Err(crate::Error::message(format!(
                "unknown approval status: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl RunStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetScope {
    #[default]
    Task,
    Goal,
    Project,
}

impl BudgetScope {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Goal => "goal",
            Self::Project => "project",
        }
    }
}

impl std::str::FromStr for BudgetScope {
    type Err = crate::Error;

    fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "task" => Ok(Self::Task),
            "goal" => Ok(Self::Goal),
            "project" => Ok(Self::Project),
            _ => Err(crate::Error::message(format!(
                "unknown budget scope: {value}"
            ))),
        }
    }
}

impl std::str::FromStr for RunStatus {
    type Err = crate::Error;

    fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(crate::Error::message(format!(
                "unknown run status: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub slug: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: GoalStatus,
    pub project_id: Option<String>,
    pub parent_goal_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub list_key: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub goal_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
    pub session_key: Option<String>,
    pub claimed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComment {
    pub id: String,
    pub task_id: String,
    pub author: Option<String>,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRun {
    pub id: String,
    pub task_id: String,
    pub runtime_run_id: Option<String>,
    pub session_key: Option<String>,
    pub agent_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub status: RunStatus,
    pub summary: Option<String>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub request_input_tokens: Option<u32>,
    pub request_output_tokens: Option<u32>,
    pub duration_ms: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskApproval {
    pub id: String,
    pub task_id: Option<String>,
    pub kind: String,
    pub status: ApprovalStatus,
    pub reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCostAttribution {
    pub id: String,
    pub task_id: String,
    pub task_run_id: Option<String>,
    pub goal_id: Option<String>,
    pub project_id: Option<String>,
    pub session_key: Option<String>,
    pub runtime_run_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub request_input_tokens: Option<u64>,
    pub request_output_tokens: Option<u64>,
    pub total_tokens: u64,
    pub estimated_cost_usd: Option<f64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkBudget {
    pub id: String,
    pub scope: BudgetScope,
    pub scope_id: String,
    pub soft_limit_tokens: Option<u64>,
    pub hard_limit_tokens: Option<u64>,
    pub soft_limit_usd: Option<f64>,
    pub hard_limit_usd: Option<f64>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub budget: WorkBudget,
    pub total_tokens: u64,
    pub estimated_cost_usd: f64,
    pub soft_exceeded: bool,
    pub hard_exceeded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCheck {
    pub allowed: bool,
    pub task_id: String,
    pub requires_approval: bool,
    pub pending_approval_id: Option<String>,
    #[serde(default)]
    pub scopes: Vec<BudgetStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDashboard {
    pub counts: WorkTaskCounts,
    #[serde(default)]
    pub pending_tasks: Vec<Task>,
    #[serde(default)]
    pub in_progress_tasks: Vec<Task>,
    #[serde(default)]
    pub blocked_tasks: Vec<Task>,
    #[serde(default)]
    pub completed_tasks: Vec<Task>,
    #[serde(default)]
    pub cancelled_tasks: Vec<Task>,
    #[serde(default)]
    pub pending_approvals: Vec<TaskApproval>,
    #[serde(default)]
    pub active_runs: Vec<TaskRun>,
    #[serde(default)]
    pub recent_costs: Vec<TaskCostAttribution>,
    #[serde(default)]
    pub budget_statuses: Vec<BudgetStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalPlanResult {
    pub goal_id: String,
    pub preset: GoalPlanPreset,
    pub root_template: TaskTemplate,
    #[serde(default)]
    pub templates: Vec<TaskTemplate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instantiated: Option<TemplateInstantiationResult>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkPortfolio {
    #[serde(default)]
    pub projects: Vec<ProjectWorkSummary>,
    #[serde(default)]
    pub cross_project_dependencies: Vec<CrossProjectDependency>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectWorkSummary {
    pub project_id: Option<String>,
    pub goal_count: usize,
    pub task_counts: WorkTaskCounts,
    pub pending_approvals: usize,
    pub active_runs: usize,
    pub budget_alerts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossProjectDependency {
    pub task_id: String,
    pub task_title: String,
    pub task_project_id: Option<String>,
    pub depends_on_task_id: String,
    pub depends_on_task_title: String,
    pub depends_on_project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalTaskLink {
    pub id: String,
    pub task_id: String,
    pub provider: String,
    pub remote_id: String,
    pub remote_title: String,
    pub remote_status: String,
    pub remote_url: Option<String>,
    pub remote_updated_at: Option<String>,
    pub synced_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerImportResult {
    pub task: Task,
    pub link: ExternalTaskLink,
    pub created_task: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkTaskCounts {
    pub pending: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub completed: usize,
    pub cancelled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackageExportResult {
    pub root_path: String,
    pub goal_count: usize,
    pub task_count: usize,
    pub budget_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackageImportResult {
    pub root_path: String,
    pub goals_imported: usize,
    pub tasks_imported: usize,
    pub budgets_imported: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub id: String,
    pub slug: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub list_key: Option<String>,
    pub goal_id: Option<String>,
    pub parent_template_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInstantiationResult {
    pub template_id: String,
    pub root_task_id: String,
    #[serde(default)]
    pub task_ids: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTask {
    pub id: String,
    pub name: String,
    pub template_id: String,
    pub schedule: CronSchedule,
    pub enabled: bool,
    pub list_key: Option<String>,
    pub goal_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
    pub next_run_at_ms: Option<u64>,
    pub last_materialized_at_ms: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTaskMaterialization {
    pub id: String,
    pub recurring_task_id: String,
    pub template_id: String,
    pub root_task_id: String,
    pub scheduled_for_ms: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTaskMaterializeResult {
    #[serde(default)]
    pub materializations: Vec<RecurringTaskMaterialization>,
    pub count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskFilter {
    pub list_key: Option<String>,
    pub goal_id: Option<String>,
    pub project_id: Option<String>,
    pub session_key: Option<String>,
    pub owner: Option<String>,
    pub status: Option<TaskStatus>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskTemplateFilter {
    pub goal_id: Option<String>,
    pub project_id: Option<String>,
    pub parent_template_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalTaskLinkFilter {
    pub task_id: Option<String>,
    pub provider: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGoal {
    pub slug: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: GoalStatus,
    pub project_id: Option<String>,
    pub parent_goal_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalPlanRequest {
    pub goal_id: String,
    #[serde(default)]
    pub preset: GoalPlanPreset,
    #[serde(default)]
    pub list_key: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub instantiate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerImportRequest {
    pub provider: String,
    pub remote_id: String,
    pub remote_title: String,
    #[serde(default)]
    pub remote_body: Option<String>,
    pub remote_status: String,
    #[serde(default)]
    pub remote_url: Option<String>,
    #[serde(default)]
    pub remote_updated_at: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub list_key: Option<String>,
    #[serde(default)]
    pub goal_id: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalPatch {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<GoalStatus>,
    pub project_id: Option<String>,
    pub parent_goal_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTask {
    pub list_key: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub goal_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
    pub session_key: Option<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskPatch {
    pub list_key: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub goal_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
    pub session_key: Option<String>,
    pub blocked_by: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaskTemplate {
    pub slug: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub list_key: Option<String>,
    pub goal_id: Option<String>,
    pub parent_template_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskTemplatePatch {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub list_key: Option<String>,
    pub goal_id: Option<String>,
    pub parent_template_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
    pub blocked_by: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiateTaskTemplate {
    pub template_id: String,
    pub list_key: Option<String>,
    pub goal_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaskComment {
    pub task_id: String,
    pub author: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaskRun {
    pub task_id: String,
    pub runtime_run_id: Option<String>,
    pub session_key: Option<String>,
    pub agent_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub status: RunStatus,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskRunPatch {
    pub status: Option<RunStatus>,
    pub summary: Option<String>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub request_input_tokens: Option<u32>,
    pub request_output_tokens: Option<u32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaskApproval {
    pub task_id: Option<String>,
    pub kind: String,
    pub reason: Option<String>,
    pub status: ApprovalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskApprovalPatch {
    pub status: ApprovalStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskCostFilter {
    pub task_id: Option<String>,
    pub goal_id: Option<String>,
    pub project_id: Option<String>,
    pub session_key: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaskCostAttribution {
    pub task_id: String,
    pub task_run_id: Option<String>,
    pub session_key: Option<String>,
    pub runtime_run_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub request_input_tokens: Option<u64>,
    pub request_output_tokens: Option<u64>,
    pub estimated_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWorkBudget {
    pub scope: BudgetScope,
    pub scope_id: String,
    pub soft_limit_tokens: Option<u64>,
    pub hard_limit_tokens: Option<u64>,
    pub soft_limit_usd: Option<f64>,
    pub hard_limit_usd: Option<f64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRecurringTask {
    pub name: String,
    pub template_id: String,
    pub schedule: CronSchedule,
    pub enabled: bool,
    pub list_key: Option<String>,
    pub goal_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecurringTaskPatch {
    pub name: Option<String>,
    pub template_id: Option<String>,
    pub schedule: Option<CronSchedule>,
    pub enabled: Option<bool>,
    pub list_key: Option<String>,
    pub goal_id: Option<String>,
    pub owner: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecurringTaskFilter {
    pub template_id: Option<String>,
    pub goal_id: Option<String>,
    pub project_id: Option<String>,
    pub enabled: Option<bool>,
}
