use {async_trait::async_trait, serde::Deserialize, serde_json::Value, std::path::PathBuf};

use moltis_work::{
    SqliteWorkStore,
    types::{
        ApprovalStatus, BudgetScope, GoalPatch, GoalPlanRequest, InstantiateTaskTemplate, NewGoal,
        NewRecurringTask, NewTask, NewTaskApproval, NewTaskComment, NewTaskCostAttribution,
        NewTaskRun, NewTaskTemplate, NewWorkBudget, RecurringTaskFilter, RecurringTaskPatch,
        TaskApprovalPatch, TaskCostFilter, TaskFilter, TaskPatch, TaskRunPatch, TaskStatus,
        TaskTemplateFilter, TaskTemplatePatch, TrackerImportRequest,
    },
};

use crate::services::{ServiceError, ServiceResult, WorkService};

pub struct LiveWorkService {
    store: SqliteWorkStore,
}

impl LiveWorkService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self {
            store: SqliteWorkStore::new(pool),
        }
    }
}

#[derive(Debug, Deserialize)]
struct IdParams {
    id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskListParams {
    #[serde(default, alias = "list_id")]
    list_key: Option<String>,
    #[serde(default)]
    goal_id: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    session_key: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    status: Option<TaskStatus>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplateListParams {
    #[serde(default)]
    goal_id: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    parent_template_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskClaimParams {
    id: String,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default, alias = "_session_key")]
    session_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskIdParams {
    task_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunsListParams {
    #[serde(default)]
    task_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CostsListParams {
    #[serde(default)]
    task_id: Option<String>,
    #[serde(default)]
    goal_id: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    session_key: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BudgetsListParams {
    #[serde(default)]
    scope: Option<BudgetScope>,
    #[serde(default)]
    scope_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BudgetEnforceParams {
    task_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DashboardParams {
    #[serde(default)]
    project_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecurringListParams {
    #[serde(default)]
    template_id: Option<String>,
    #[serde(default)]
    goal_id: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecurringMaterializeParams {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    force: Option<bool>,
    #[serde(default)]
    now_ms: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecurringMaterializationsParams {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrackerLinksParams {
    #[serde(default)]
    task_id: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageParams {
    root_path: PathBuf,
    #[serde(default)]
    project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoalUpdateParams {
    id: String,
    #[serde(flatten)]
    patch: GoalPatch,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplateUpdateParams {
    id: String,
    #[serde(flatten)]
    patch: TaskTemplatePatch,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskUpdateParams {
    id: String,
    #[serde(flatten)]
    patch: TaskPatch,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunUpdateParams {
    id: String,
    #[serde(flatten)]
    patch: TaskRunPatch,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApprovalUpdateParams {
    id: String,
    status: ApprovalStatus,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecurringUpdateParams {
    id: String,
    #[serde(flatten)]
    patch: RecurringTaskPatch,
}

#[async_trait]
impl WorkService for LiveWorkService {
    async fn goals_list(&self) -> ServiceResult {
        let goals = self
            .store
            .list_goals()
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(goals)?)
    }

    async fn goals_get(&self, params: Value) -> ServiceResult {
        let params: IdParams = serde_json::from_value(params)?;
        let goal = self
            .store
            .get_goal(&params.id)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(goal)?)
    }

    async fn goals_create(&self, params: Value) -> ServiceResult {
        let payload: NewGoal = serde_json::from_value(params)?;
        let goal = self
            .store
            .create_goal(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(goal)?)
    }

    async fn goals_update(&self, params: Value) -> ServiceResult {
        let params: GoalUpdateParams = serde_json::from_value(params)?;
        let goal = self
            .store
            .update_goal(&params.id, params.patch)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(goal)?)
    }

    async fn goals_delete(&self, params: Value) -> ServiceResult {
        let params: IdParams = serde_json::from_value(params)?;
        let deleted = self
            .store
            .delete_goal(&params.id)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({ "deleted": deleted, "id": params.id }))
    }

    async fn goals_plan(&self, params: Value) -> ServiceResult {
        let payload: GoalPlanRequest = serde_json::from_value(params)?;
        let result = self
            .store
            .plan_goal(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(result)?)
    }

    async fn templates_list(&self, params: Value) -> ServiceResult {
        let params: TemplateListParams = serde_json::from_value(params)?;
        let templates = self
            .store
            .list_task_templates(&TaskTemplateFilter {
                goal_id: params.goal_id,
                project_id: params.project_id,
                parent_template_id: params.parent_template_id,
            })
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "templates": templates,
            "count": templates.len(),
        }))
    }

    async fn templates_get(&self, params: Value) -> ServiceResult {
        let params: IdParams = serde_json::from_value(params)?;
        let template = self
            .store
            .get_task_template(&params.id)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(template)?)
    }

    async fn templates_create(&self, params: Value) -> ServiceResult {
        let payload: NewTaskTemplate = serde_json::from_value(params)?;
        let template = self
            .store
            .create_task_template(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(template)?)
    }

    async fn templates_update(&self, params: Value) -> ServiceResult {
        let params: TemplateUpdateParams = serde_json::from_value(params)?;
        let template = self
            .store
            .update_task_template(&params.id, params.patch)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(template)?)
    }

    async fn templates_instantiate(&self, params: Value) -> ServiceResult {
        let payload: InstantiateTaskTemplate = serde_json::from_value(params)?;
        let result = self
            .store
            .instantiate_task_template(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(result)?)
    }

    async fn tasks_list(&self, params: Value) -> ServiceResult {
        let params: TaskListParams = serde_json::from_value(params)?;
        let tasks = self
            .store
            .list_tasks(&TaskFilter {
                list_key: params.list_key,
                goal_id: params.goal_id,
                project_id: params.project_id,
                session_key: params.session_key,
                owner: params.owner,
                status: params.status,
            })
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "tasks": tasks,
            "count": tasks.len(),
        }))
    }

    async fn tasks_get(&self, params: Value) -> ServiceResult {
        let params: IdParams = serde_json::from_value(params)?;
        let task = self
            .store
            .get_task(&params.id)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(task)?)
    }

    async fn tasks_create(&self, params: Value) -> ServiceResult {
        let payload: NewTask = serde_json::from_value(params)?;
        let task = self
            .store
            .create_task(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(task)?)
    }

    async fn tasks_update(&self, params: Value) -> ServiceResult {
        let params: TaskUpdateParams = serde_json::from_value(params)?;
        let task = self
            .store
            .update_task(&params.id, params.patch)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(task)?)
    }

    async fn tasks_claim(&self, params: Value) -> ServiceResult {
        let params: TaskClaimParams = serde_json::from_value(params)?;
        let task = self
            .store
            .claim_task(&params.id, params.owner, params.session_key)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(task)?)
    }

    async fn comments_list(&self, params: Value) -> ServiceResult {
        let params: TaskIdParams = serde_json::from_value(params)?;
        let comments = self
            .store
            .list_comments(&params.task_id)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "comments": comments,
            "count": comments.len(),
        }))
    }

    async fn comments_add(&self, params: Value) -> ServiceResult {
        let payload: NewTaskComment = serde_json::from_value(params)?;
        let comment = self
            .store
            .add_comment(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(comment)?)
    }

    async fn runs_list(&self, params: Value) -> ServiceResult {
        let params: RunsListParams = serde_json::from_value(params)?;
        let runs = self
            .store
            .list_runs(params.task_id.as_deref())
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "runs": runs,
            "count": runs.len(),
        }))
    }

    async fn runs_create(&self, params: Value) -> ServiceResult {
        let payload: NewTaskRun = serde_json::from_value(params)?;
        let run = self
            .store
            .create_run(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(run)?)
    }

    async fn runs_update(&self, params: Value) -> ServiceResult {
        let params: RunUpdateParams = serde_json::from_value(params)?;
        let run = self
            .store
            .update_run(&params.id, params.patch)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(run)?)
    }

    async fn approvals_list(&self, params: Value) -> ServiceResult {
        let params: RunsListParams = serde_json::from_value(params)?;
        let approvals = self
            .store
            .list_approvals(params.task_id.as_deref())
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "approvals": approvals,
            "count": approvals.len(),
        }))
    }

    async fn approvals_create(&self, params: Value) -> ServiceResult {
        let payload: NewTaskApproval = serde_json::from_value(params)?;
        let approval = self
            .store
            .create_approval(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(approval)?)
    }

    async fn approvals_update(&self, params: Value) -> ServiceResult {
        let params: ApprovalUpdateParams = serde_json::from_value(params)?;
        let approval = self
            .store
            .update_approval(&params.id, TaskApprovalPatch {
                status: params.status,
                reason: params.reason,
            })
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(approval)?)
    }

    async fn costs_list(&self, params: Value) -> ServiceResult {
        let params: CostsListParams = serde_json::from_value(params)?;
        let costs = self
            .store
            .list_costs(&TaskCostFilter {
                task_id: params.task_id,
                goal_id: params.goal_id,
                project_id: params.project_id,
                session_key: params.session_key,
                limit: params.limit,
            })
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "costs": costs,
            "count": costs.len(),
        }))
    }

    async fn costs_record(&self, params: Value) -> ServiceResult {
        let payload: NewTaskCostAttribution = serde_json::from_value(params)?;
        let cost = self
            .store
            .record_cost(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(cost)?)
    }

    async fn budgets_list(&self, params: Value) -> ServiceResult {
        let params: BudgetsListParams = serde_json::from_value(params)?;
        let budgets = self
            .store
            .list_budgets(params.scope, params.scope_id.as_deref())
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "budgets": budgets,
            "count": budgets.len(),
        }))
    }

    async fn budgets_save(&self, params: Value) -> ServiceResult {
        let payload: NewWorkBudget = serde_json::from_value(params)?;
        let budget = self
            .store
            .save_budget(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(budget)?)
    }

    async fn budgets_enforce(&self, params: Value) -> ServiceResult {
        let params: BudgetEnforceParams = serde_json::from_value(params)?;
        let status = self
            .store
            .enforce_task_budget(&params.task_id)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(status)?)
    }

    async fn recurring_list(&self, params: Value) -> ServiceResult {
        let params: RecurringListParams = serde_json::from_value(params)?;
        let items = self
            .store
            .list_recurring_tasks(&RecurringTaskFilter {
                template_id: params.template_id,
                goal_id: params.goal_id,
                project_id: params.project_id,
                enabled: params.enabled,
            })
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "items": items,
            "count": items.len(),
        }))
    }

    async fn recurring_get(&self, params: Value) -> ServiceResult {
        let params: IdParams = serde_json::from_value(params)?;
        let item = self
            .store
            .get_recurring_task(&params.id)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(item)?)
    }

    async fn recurring_create(&self, params: Value) -> ServiceResult {
        let payload: NewRecurringTask = serde_json::from_value(params)?;
        let item = self
            .store
            .create_recurring_task(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(item)?)
    }

    async fn recurring_update(&self, params: Value) -> ServiceResult {
        let params: RecurringUpdateParams = serde_json::from_value(params)?;
        let item = self
            .store
            .update_recurring_task(&params.id, params.patch)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(item)?)
    }

    async fn recurring_materialize(&self, params: Value) -> ServiceResult {
        let params: RecurringMaterializeParams = serde_json::from_value(params)?;
        let result = self
            .store
            .materialize_due_recurring_tasks(
                params.id.as_deref(),
                params.limit,
                params.force.unwrap_or(false),
                params.now_ms,
            )
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(result)?)
    }

    async fn recurring_materializations_list(&self, params: Value) -> ServiceResult {
        let params: RecurringMaterializationsParams = serde_json::from_value(params)?;
        let materializations = self
            .store
            .list_recurring_materializations(params.id.as_deref(), params.limit.unwrap_or(20))
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "materializations": materializations,
            "count": materializations.len(),
        }))
    }

    async fn trackers_links_list(&self, params: Value) -> ServiceResult {
        let params: TrackerLinksParams = serde_json::from_value(params)?;
        let links = self
            .store
            .list_external_links(&moltis_work::types::ExternalTaskLinkFilter {
                task_id: params.task_id,
                provider: params.provider,
                limit: params.limit,
            })
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::json!({
            "links": links,
            "count": links.len(),
        }))
    }

    async fn trackers_import(&self, params: Value) -> ServiceResult {
        let payload: TrackerImportRequest = serde_json::from_value(params)?;
        let result = self
            .store
            .import_tracker_task(payload)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(result)?)
    }

    async fn dashboard(&self, params: Value) -> ServiceResult {
        let params: DashboardParams = serde_json::from_value(params)?;
        let dashboard = self
            .store
            .dashboard(params.project_id.as_deref())
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(dashboard)?)
    }

    async fn portfolio(&self) -> ServiceResult {
        let portfolio = self
            .store
            .portfolio()
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(portfolio)?)
    }

    async fn package_export(&self, params: Value) -> ServiceResult {
        let params: PackageParams = serde_json::from_value(params)?;
        let result = self
            .store
            .export_package(&params.root_path, params.project_id.as_deref())
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(result)?)
    }

    async fn package_import(&self, params: Value) -> ServiceResult {
        let params: PackageParams = serde_json::from_value(params)?;
        let result = self
            .store
            .import_package(&params.root_path)
            .await
            .map_err(ServiceError::message)?;
        Ok(serde_json::to_value(result)?)
    }
}
