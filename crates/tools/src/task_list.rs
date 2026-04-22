//! Shared task list tool for inter-agent task coordination.

use std::sync::Arc;

use {
    async_trait::async_trait,
    moltis_agents::tool_registry::AgentTool,
    moltis_sessions::metadata::SqliteSessionMetadata,
    moltis_work::{
        SqliteWorkStore,
        types::{
            GoalPlanPreset, InstantiateTaskTemplate, NewRecurringTask, NewTask, NewTaskComment,
            NewTaskTemplate, RecurringTask, RecurringTaskPatch, Task, TaskFilter, TaskPatch,
            TaskPriority, TaskStatus, TaskTemplate, TaskTemplatePatch,
        },
    },
};

use crate::{
    Error,
    params::{require_str, str_param, str_param_any},
};

fn parse_task_status(value: Option<&str>) -> crate::Result<Option<TaskStatus>> {
    value
        .map(str::parse::<TaskStatus>)
        .transpose()
        .map_err(|error| Error::message(error.to_string()))
}

fn parse_task_priority(value: Option<&str>) -> crate::Result<Option<TaskPriority>> {
    value
        .map(str::parse::<TaskPriority>)
        .transpose()
        .map_err(|error| Error::message(error.to_string()))
}

fn parse_goal_plan_preset(value: Option<&str>) -> crate::Result<GoalPlanPreset> {
    value
        .unwrap_or("delivery")
        .parse::<GoalPlanPreset>()
        .map_err(|error| Error::message(error.to_string()))
}

fn blocked_by_from_params(params: &serde_json::Value) -> Option<Vec<String>> {
    params
        .get("blocked_by")
        .or_else(|| params.get("blockedBy"))
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
}

fn parse_schedule(params: &serde_json::Value) -> crate::Result<moltis_cron::types::CronSchedule> {
    let schedule_value = params
        .get("schedule")
        .cloned()
        .ok_or_else(|| Error::message("missing required parameter: schedule"))?;
    serde_json::from_value(schedule_value).map_err(|error| Error::message(error.to_string()))
}

fn task_to_json(task: &Task) -> serde_json::Value {
    serde_json::json!({
        "id": task.id.clone(),
        "list_id": task.list_key.clone(),
        "listId": task.list_key.clone(),
        "list_key": task.list_key.clone(),
        "subject": task.title.clone(),
        "title": task.title.clone(),
        "description": task.description.clone().unwrap_or_default(),
        "status": task.status,
        "priority": task.priority,
        "owner": task.owner.clone(),
        "project_id": task.project_id.clone(),
        "projectId": task.project_id.clone(),
        "goal_id": task.goal_id.clone(),
        "goalId": task.goal_id.clone(),
        "session_key": task.session_key.clone(),
        "sessionKey": task.session_key.clone(),
        "claimed_at": task.claimed_at.clone(),
        "created_at": task.created_at.clone(),
        "updated_at": task.updated_at.clone(),
        "blocked_by": task.blocked_by.clone(),
        "blockedBy": task.blocked_by.clone(),
    })
}

fn template_to_json(template: &TaskTemplate) -> serde_json::Value {
    serde_json::json!({
        "id": template.id.clone(),
        "slug": template.slug.clone(),
        "title": template.title.clone(),
        "description": template.description.clone().unwrap_or_default(),
        "priority": template.priority,
        "list_id": template.list_key.clone(),
        "listId": template.list_key.clone(),
        "goal_id": template.goal_id.clone(),
        "goalId": template.goal_id.clone(),
        "parent_template_id": template.parent_template_id.clone(),
        "parentTemplateId": template.parent_template_id.clone(),
        "owner": template.owner.clone(),
        "project_id": template.project_id.clone(),
        "projectId": template.project_id.clone(),
        "created_at": template.created_at.clone(),
        "updated_at": template.updated_at.clone(),
        "blocked_by": template.blocked_by.clone(),
        "blockedBy": template.blocked_by.clone(),
    })
}

fn recurring_to_json(item: &RecurringTask) -> serde_json::Value {
    serde_json::json!({
        "id": item.id.clone(),
        "name": item.name.clone(),
        "template_id": item.template_id.clone(),
        "templateId": item.template_id.clone(),
        "schedule": item.schedule.clone(),
        "enabled": item.enabled,
        "list_id": item.list_key.clone(),
        "listId": item.list_key.clone(),
        "goal_id": item.goal_id.clone(),
        "goalId": item.goal_id.clone(),
        "owner": item.owner.clone(),
        "project_id": item.project_id.clone(),
        "projectId": item.project_id.clone(),
        "next_run_at_ms": item.next_run_at_ms,
        "nextRunAtMs": item.next_run_at_ms,
        "last_materialized_at_ms": item.last_materialized_at_ms,
        "lastMaterializedAtMs": item.last_materialized_at_ms,
        "created_at": item.created_at.clone(),
        "updated_at": item.updated_at.clone(),
    })
}

/// Tool wrapper around the durable work store.
pub struct TaskListTool {
    store: Arc<SqliteWorkStore>,
    session_metadata: Option<Arc<SqliteSessionMetadata>>,
}

impl TaskListTool {
    pub fn new(
        pool: sqlx::SqlitePool,
        session_metadata: Option<Arc<SqliteSessionMetadata>>,
    ) -> Self {
        Self {
            store: Arc::new(SqliteWorkStore::new(pool)),
            session_metadata,
        }
    }
}

#[async_trait]
impl AgentTool for TaskListTool {
    fn name(&self) -> &str {
        "task_list"
    }

    fn description(&self) -> &str {
        "Manage durable shared tasks, reusable task templates, and recurring \
         task materialization for coordinated multi-agent execution. Actions: \
         create, list, get, update, claim, comment, comments, goal_plan, \
         template_create, template_list, template_get, template_update, \
         template_instantiate, tracker_import, tracker_links, \
         recurring_create, recurring_list, recurring_get, recurring_update, \
         recurring_materialize."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "create", "list", "get", "update", "claim", "comment", "comments",
                        "goal_plan",
                        "tracker_import", "tracker_links",
                        "template_create", "template_list", "template_get", "template_update", "template_instantiate",
                        "recurring_create", "recurring_list", "recurring_get", "recurring_update", "recurring_materialize"
                    ],
                    "description": "Task list action to perform."
                },
                "list_id": {
                    "type": "string",
                    "description": "Task list identifier (default: default)."
                },
                "id": {
                    "type": "string",
                    "description": "Task ID for get/update/claim."
                },
                "subject": {
                    "type": "string",
                    "description": "Task subject for create/update."
                },
                "description": {
                    "type": "string",
                    "description": "Task description for create/update."
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "blocked", "completed", "cancelled"],
                    "description": "Task status for list/update."
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "critical"],
                    "description": "Task priority for create/update."
                },
                "preset": {
                    "type": "string",
                    "enum": ["delivery", "investigation", "maintenance"],
                    "description": "Goal planning preset for goal_plan."
                },
                "owner": {
                    "type": "string",
                    "description": "Task owner for update/claim."
                },
                "blocked_by": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of task IDs that block this task."
                },
                "body": {
                    "type": "string",
                    "description": "Task comment body for comment."
                },
                "provider": {
                    "type": "string",
                    "description": "External tracker provider for tracker_import/tracker_links."
                },
                "remote_id": {
                    "type": "string",
                    "description": "External tracker record identifier for tracker_import."
                },
                "remote_title": {
                    "type": "string",
                    "description": "External tracker record title for tracker_import."
                },
                "remote_status": {
                    "type": "string",
                    "description": "External tracker record status for tracker_import."
                },
                "remote_url": {
                    "type": "string",
                    "description": "External tracker record URL for tracker_import."
                },
                "schedule": {
                    "type": "object",
                    "description": "Cron-style schedule object for recurring_create/recurring_update."
                },
                "instantiate": {
                    "type": "boolean",
                    "description": "Whether goal_plan should also instantiate real tasks."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let action = require_str(&params, "action")?;
        let list_key = str_param_any(&params, &["list_id", "listId"]).unwrap_or("default");

        match action {
            "create" => {
                let title = str_param_any(&params, &["subject", "title"])
                    .ok_or_else(|| Error::message("missing required parameter: subject"))?
                    .to_string();
                let task = self
                    .store
                    .create_task(NewTask {
                        list_key: Some(list_key.to_string()),
                        title,
                        description: str_param(&params, "description").map(ToString::to_string),
                        status: parse_task_status(str_param(&params, "status"))?
                            .unwrap_or(TaskStatus::Pending),
                        priority: parse_task_priority(str_param(&params, "priority"))?
                            .unwrap_or(TaskPriority::Medium),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        parent_task_id: str_param_any(&params, &["parent_task_id", "parentTaskId"])
                            .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                        session_key: str_param_any(&params, &["session_key", "sessionKey"])
                            .map(ToString::to_string),
                        blocked_by: blocked_by_from_params(&params).unwrap_or_default(),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "task": task_to_json(&task),
                }))
            },
            "list" => {
                let tasks = self
                    .store
                    .list_tasks(&TaskFilter {
                        list_key: Some(list_key.to_string()),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                        session_key: str_param_any(&params, &["session_key", "sessionKey"])
                            .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        status: parse_task_status(str_param(&params, "status"))?,
                    })
                    .await?;
                let items = tasks.iter().map(task_to_json).collect::<Vec<_>>();
                Ok(serde_json::json!({
                    "ok": true,
                    "tasks": items,
                    "count": tasks.len(),
                }))
            },
            "get" => {
                let id = require_str(&params, "id")?;
                let task = self.store.get_task(id).await?;
                Ok(serde_json::json!({
                    "ok": task.is_some(),
                    "task": task.as_ref().map(task_to_json),
                }))
            },
            "update" => {
                let id = require_str(&params, "id")?;
                let task = self
                    .store
                    .update_task(id, TaskPatch {
                        list_key: str_param_any(&params, &["list_id", "listId"])
                            .map(ToString::to_string),
                        title: str_param_any(&params, &["subject", "title"])
                            .map(ToString::to_string),
                        description: str_param(&params, "description").map(ToString::to_string),
                        status: parse_task_status(str_param(&params, "status"))?,
                        priority: parse_task_priority(str_param(&params, "priority"))?,
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        parent_task_id: str_param_any(&params, &["parent_task_id", "parentTaskId"])
                            .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                        session_key: str_param_any(&params, &["session_key", "sessionKey"])
                            .map(ToString::to_string),
                        blocked_by: blocked_by_from_params(&params),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": task.is_some(),
                    "task": task.as_ref().map(task_to_json),
                }))
            },
            "claim" => {
                let id = require_str(&params, "id")?;
                let session_key =
                    str_param_any(&params, &["_session_key", "session_key", "sessionKey"])
                        .map(ToString::to_string);
                let owner = str_param_any(&params, &["owner", "_session_key"])
                    .map(ToString::to_string)
                    .or_else(|| session_key.clone())
                    .or_else(|| Some("agent".to_string()));
                let task = self
                    .store
                    .claim_task(id, owner, session_key.clone())
                    .await?;
                if let Some(ref metadata) = self.session_metadata
                    && let Some(ref key) = session_key
                    && let Some(ref claimed_task) = task
                {
                    metadata
                        .set_task_id(key, Some(claimed_task.id.clone()))
                        .await;
                }
                Ok(serde_json::json!({
                    "ok": task.is_some(),
                    "task": task.as_ref().map(task_to_json),
                }))
            },
            "comment" => {
                let task_id = require_str(&params, "id")?.to_string();
                let body = require_str(&params, "body")?.to_string();
                let author =
                    str_param_any(&params, &["author", "_session_key"]).map(ToString::to_string);
                let comment = self
                    .store
                    .add_comment(NewTaskComment {
                        task_id,
                        author,
                        body,
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "comment": comment,
                }))
            },
            "comments" => {
                let task_id = require_str(&params, "id")?;
                let comments = self.store.list_comments(task_id).await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "comments": comments,
                    "count": comments.len(),
                }))
            },
            "goal_plan" => {
                let goal_id = str_param_any(&params, &["id", "goal_id", "goalId"])
                    .ok_or_else(|| Error::message("missing required parameter: goal_id"))?
                    .to_string();
                let result = self
                    .store
                    .plan_goal(moltis_work::types::GoalPlanRequest {
                        goal_id,
                        preset: parse_goal_plan_preset(str_param(&params, "preset"))?,
                        list_key: str_param_any(&params, &["list_id", "listId"])
                            .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                        instantiate: params
                            .get("instantiate")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "result": result,
                }))
            },
            "tracker_import" => {
                let provider = require_str(&params, "provider")?.to_string();
                let remote_id = str_param_any(&params, &["remote_id", "remoteId"])
                    .ok_or_else(|| Error::message("missing required parameter: remote_id"))?
                    .to_string();
                let remote_title = str_param_any(&params, &["remote_title", "remoteTitle"])
                    .ok_or_else(|| Error::message("missing required parameter: remote_title"))?
                    .to_string();
                let remote_status = str_param_any(&params, &["remote_status", "remoteStatus"])
                    .ok_or_else(|| Error::message("missing required parameter: remote_status"))?
                    .to_string();
                let result = self
                    .store
                    .import_tracker_task(moltis_work::types::TrackerImportRequest {
                        provider,
                        remote_id,
                        remote_title,
                        remote_body: str_param_any(&params, &["remote_body", "remoteBody"])
                            .map(ToString::to_string),
                        remote_status,
                        remote_url: str_param_any(&params, &["remote_url", "remoteUrl"])
                            .map(ToString::to_string),
                        remote_updated_at: str_param_any(&params, &[
                            "remote_updated_at",
                            "remoteUpdatedAt",
                        ])
                        .map(ToString::to_string),
                        task_id: str_param_any(&params, &["id", "task_id", "taskId"])
                            .map(ToString::to_string),
                        list_key: str_param_any(&params, &["list_id", "listId"])
                            .map(ToString::to_string),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "result": result,
                }))
            },
            "tracker_links" => {
                let links = self
                    .store
                    .list_external_links(&moltis_work::types::ExternalTaskLinkFilter {
                        task_id: str_param_any(&params, &["id", "task_id", "taskId"])
                            .map(ToString::to_string),
                        provider: str_param(&params, "provider").map(ToString::to_string),
                        limit: params
                            .get("limit")
                            .and_then(serde_json::Value::as_u64)
                            .map(|value| value as u32),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "links": links,
                    "count": links.len(),
                }))
            },
            "template_create" => {
                let title = str_param_any(&params, &["subject", "title"])
                    .ok_or_else(|| Error::message("missing required parameter: subject"))?
                    .to_string();
                let template = self
                    .store
                    .create_task_template(NewTaskTemplate {
                        slug: str_param(&params, "slug").map(ToString::to_string),
                        title,
                        description: str_param(&params, "description").map(ToString::to_string),
                        priority: parse_task_priority(str_param(&params, "priority"))?
                            .unwrap_or(TaskPriority::Medium),
                        list_key: str_param_any(&params, &["list_id", "listId"])
                            .map(ToString::to_string),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        parent_template_id: str_param_any(&params, &[
                            "parent_template_id",
                            "parentTemplateId",
                        ])
                        .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                        blocked_by: blocked_by_from_params(&params).unwrap_or_default(),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "template": template_to_json(&template),
                }))
            },
            "template_list" => {
                let templates = self
                    .store
                    .list_task_templates(&moltis_work::types::TaskTemplateFilter {
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                        parent_template_id: str_param_any(&params, &[
                            "parent_template_id",
                            "parentTemplateId",
                        ])
                        .map(ToString::to_string),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "templates": templates.iter().map(template_to_json).collect::<Vec<_>>(),
                    "count": templates.len(),
                }))
            },
            "template_get" => {
                let id = require_str(&params, "id")?;
                let template = self.store.get_task_template(id).await?;
                Ok(serde_json::json!({
                    "ok": template.is_some(),
                    "template": template.as_ref().map(template_to_json),
                }))
            },
            "template_update" => {
                let id = require_str(&params, "id")?;
                let template = self
                    .store
                    .update_task_template(id, TaskTemplatePatch {
                        slug: str_param(&params, "slug").map(ToString::to_string),
                        title: str_param_any(&params, &["subject", "title"])
                            .map(ToString::to_string),
                        description: str_param(&params, "description").map(ToString::to_string),
                        priority: parse_task_priority(str_param(&params, "priority"))?,
                        list_key: str_param_any(&params, &["list_id", "listId"])
                            .map(ToString::to_string),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        parent_template_id: str_param_any(&params, &[
                            "parent_template_id",
                            "parentTemplateId",
                        ])
                        .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                        blocked_by: blocked_by_from_params(&params),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": template.is_some(),
                    "template": template.as_ref().map(template_to_json),
                }))
            },
            "template_instantiate" => {
                let template_id = str_param_any(&params, &["id", "template_id", "templateId"])
                    .ok_or_else(|| Error::message("missing required parameter: template_id"))?
                    .to_string();
                let result = self
                    .store
                    .instantiate_task_template(InstantiateTaskTemplate {
                        template_id,
                        list_key: str_param_any(&params, &["list_id", "listId"])
                            .map(ToString::to_string),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        parent_task_id: str_param_any(&params, &["parent_task_id", "parentTaskId"])
                            .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "result": result,
                }))
            },
            "recurring_create" => {
                let name = str_param(&params, "name")
                    .ok_or_else(|| Error::message("missing required parameter: name"))?
                    .to_string();
                let template_id = str_param_any(&params, &["template_id", "templateId"])
                    .ok_or_else(|| Error::message("missing required parameter: template_id"))?
                    .to_string();
                let item = self
                    .store
                    .create_recurring_task(NewRecurringTask {
                        name,
                        template_id,
                        schedule: parse_schedule(&params)?,
                        enabled: params
                            .get("enabled")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(true),
                        list_key: str_param_any(&params, &["list_id", "listId"])
                            .map(ToString::to_string),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "recurring": recurring_to_json(&item),
                }))
            },
            "recurring_list" => {
                let items = self
                    .store
                    .list_recurring_tasks(&moltis_work::types::RecurringTaskFilter {
                        template_id: str_param_any(&params, &["template_id", "templateId"])
                            .map(ToString::to_string),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                        enabled: params.get("enabled").and_then(serde_json::Value::as_bool),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "recurring": items.iter().map(recurring_to_json).collect::<Vec<_>>(),
                    "count": items.len(),
                }))
            },
            "recurring_get" => {
                let id = require_str(&params, "id")?;
                let item = self.store.get_recurring_task(id).await?;
                Ok(serde_json::json!({
                    "ok": item.is_some(),
                    "recurring": item.as_ref().map(recurring_to_json),
                }))
            },
            "recurring_update" => {
                let id = require_str(&params, "id")?;
                let item = self
                    .store
                    .update_recurring_task(id, RecurringTaskPatch {
                        name: str_param(&params, "name").map(ToString::to_string),
                        template_id: str_param_any(&params, &["template_id", "templateId"])
                            .map(ToString::to_string),
                        schedule: params
                            .get("schedule")
                            .cloned()
                            .map(serde_json::from_value)
                            .transpose()
                            .map_err(|error| Error::message(error.to_string()))?,
                        enabled: params.get("enabled").and_then(serde_json::Value::as_bool),
                        list_key: str_param_any(&params, &["list_id", "listId"])
                            .map(ToString::to_string),
                        goal_id: str_param_any(&params, &["goal_id", "goalId"])
                            .map(ToString::to_string),
                        owner: str_param(&params, "owner").map(ToString::to_string),
                        project_id: str_param_any(&params, &["project_id", "projectId"])
                            .map(ToString::to_string),
                    })
                    .await?;
                Ok(serde_json::json!({
                    "ok": item.is_some(),
                    "recurring": item.as_ref().map(recurring_to_json),
                }))
            },
            "recurring_materialize" => {
                let recurring_id = str_param_any(&params, &["id", "recurring_id", "recurringId"]);
                let result = self
                    .store
                    .materialize_due_recurring_tasks(
                        recurring_id,
                        params
                            .get("limit")
                            .and_then(serde_json::Value::as_u64)
                            .map(|value| value as u32),
                        params
                            .get("force")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false),
                        params
                            .get("now_ms")
                            .or_else(|| params.get("nowMs"))
                            .and_then(serde_json::Value::as_u64),
                    )
                    .await?;
                Ok(serde_json::json!({
                    "ok": true,
                    "result": result,
                }))
            },
            _ => Err(Error::message(format!("unknown task_list action: {action}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

    async fn tool() -> TestResult<TaskListTool> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
        moltis_work::run_migrations(&pool).await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS projects (id TEXT PRIMARY KEY)")
            .execute(&pool)
            .await?;
        SqliteSessionMetadata::init(&pool).await?;
        let metadata = Arc::new(SqliteSessionMetadata::new(pool.clone()));
        metadata.upsert("main", None).await?;
        Ok(TaskListTool::new(pool, Some(metadata)))
    }

    #[tokio::test]
    async fn create_and_list_tasks() -> TestResult<()> {
        let task_tool = tool().await?;
        task_tool
            .execute(serde_json::json!({
                "action": "create",
                "subject": "first",
                "description": "desc"
            }))
            .await?;

        let result = task_tool
            .execute(serde_json::json!({
                "action": "list"
            }))
            .await?;
        assert_eq!(result["count"], 1);
        assert_eq!(result["tasks"][0]["subject"], "first");
        assert_eq!(result["tasks"][0]["status"], "pending");
        Ok(())
    }

    #[tokio::test]
    async fn claim_moves_task_to_in_progress_and_binds_session() -> TestResult<()> {
        let task_tool = tool().await?;
        let created = task_tool
            .execute(serde_json::json!({
                "action": "create",
                "subject": "work"
            }))
            .await?;
        let id = created["task"]["id"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("missing task id"))?;

        let claimed = task_tool
            .execute(serde_json::json!({
                "action": "claim",
                "id": id,
                "_session_key": "main",
                "owner": "worker-a"
            }))
            .await?;
        assert_eq!(claimed["task"]["status"], "in_progress");
        assert_eq!(claimed["task"]["owner"], "worker-a");
        assert_eq!(claimed["task"]["sessionKey"], "main");
        Ok(())
    }

    #[tokio::test]
    async fn claim_rejects_when_blocked_dependencies_incomplete() -> TestResult<()> {
        let task_tool = tool().await?;
        let dep = task_tool
            .execute(serde_json::json!({
                "action": "create",
                "subject": "dep"
            }))
            .await?;
        let dep_id = dep["task"]["id"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("missing dep id"))?;

        let main = task_tool
            .execute(serde_json::json!({
                "action": "create",
                "subject": "main"
            }))
            .await?;
        let main_id = main["task"]["id"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("missing main id"))?;

        task_tool
            .execute(serde_json::json!({
                "action": "update",
                "id": main_id,
                "blocked_by": [dep_id]
            }))
            .await?;

        let result = task_tool
            .execute(serde_json::json!({
                "action": "claim",
                "id": main_id
            }))
            .await;
        let err = result
            .err()
            .ok_or_else(|| std::io::Error::other("expected blocked claim failure"))?;
        assert!(err.to_string().contains("blocked by incomplete tasks"));
        Ok(())
    }

    #[tokio::test]
    async fn goal_plan_action_builds_templates_and_tasks() -> TestResult<()> {
        let task_tool = tool().await?;
        let goal = task_tool
            .store
            .create_goal(moltis_work::types::NewGoal {
                slug: Some("stabilize-api".to_string()),
                title: "Stabilize API".to_string(),
                description: Some("Reduce churn and verify compatibility.".to_string()),
                status: moltis_work::types::GoalStatus::Active,
                project_id: Some("proj-api".to_string()),
                parent_goal_id: None,
            })
            .await?;

        let planned = task_tool
            .execute(serde_json::json!({
                "action": "goal_plan",
                "id": goal.id,
                "preset": "maintenance",
                "list_id": "stability",
                "instantiate": true,
                "owner": "planner-a",
            }))
            .await?;
        assert_eq!(planned["result"]["preset"], "maintenance");
        assert_eq!(
            planned["result"]["templates"].as_array().map(Vec::len),
            Some(5)
        );
        assert!(planned["result"]["instantiated"].is_object());

        let tasks = task_tool
            .execute(serde_json::json!({
                "action": "list",
                "list_id": "stability",
            }))
            .await?;
        assert_eq!(tasks["count"], 5);
        Ok(())
    }

    #[tokio::test]
    async fn tracker_import_and_links_actions_round_trip() -> TestResult<()> {
        let task_tool = tool().await?;
        let imported = task_tool
            .execute(serde_json::json!({
                "action": "tracker_import",
                "provider": "github",
                "remote_id": "moltis-org/moltis#77",
                "remote_title": "Sync imported issue",
                "remote_status": "in_progress",
                "remote_url": "https://github.com/moltis-org/moltis/issues/77",
                "list_id": "imports",
            }))
            .await?;
        assert_eq!(imported["result"]["task"]["title"], "Sync imported issue");
        assert_eq!(imported["result"]["task"]["status"], "in_progress");

        let links = task_tool
            .execute(serde_json::json!({
                "action": "tracker_links",
                "provider": "github",
            }))
            .await?;
        assert_eq!(links["count"], 1);
        assert_eq!(
            links["links"][0]["remote_url"],
            "https://github.com/moltis-org/moltis/issues/77"
        );
        Ok(())
    }

    #[tokio::test]
    async fn template_actions_round_trip_and_instantiate() -> TestResult<()> {
        let task_tool = tool().await?;
        let root = task_tool
            .execute(serde_json::json!({
                "action": "template_create",
                "subject": "Ship release",
                "description": "Root release workflow",
                "slug": "ship-release",
                "priority": "high",
            }))
            .await?;
        let root_id = root["template"]["id"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("missing root template id"))?;

        let child = task_tool
            .execute(serde_json::json!({
                "action": "template_create",
                "subject": "Cut tag",
                "parent_template_id": root_id,
                "blocked_by": [root_id],
            }))
            .await?;
        let child_id = child["template"]["id"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("missing child template id"))?;

        let listed = task_tool
            .execute(serde_json::json!({
                "action": "template_list",
            }))
            .await?;
        assert_eq!(listed["count"], 2);

        let fetched = task_tool
            .execute(serde_json::json!({
                "action": "template_get",
                "id": child_id,
            }))
            .await?;
        assert_eq!(fetched["template"]["parentTemplateId"], root_id);

        let updated = task_tool
            .execute(serde_json::json!({
                "action": "template_update",
                "id": child_id,
                "subject": "Cut and publish tag",
                "priority": "critical",
            }))
            .await?;
        assert_eq!(updated["template"]["title"], "Cut and publish tag");
        assert_eq!(updated["template"]["priority"], "critical");

        let instantiated = task_tool
            .execute(serde_json::json!({
                "action": "template_instantiate",
                "id": root_id,
                "list_id": "release",
                "owner": "worker-a",
            }))
            .await?;
        let root_task_id = instantiated["result"]["root_task_id"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("missing root task id"))?;

        let tasks = task_tool
            .execute(serde_json::json!({
                "action": "list",
                "list_id": "release",
            }))
            .await?;
        assert_eq!(tasks["count"], 2);
        assert!(
            tasks["tasks"]
                .as_array()
                .ok_or_else(|| std::io::Error::other("missing tasks array"))?
                .iter()
                .any(|task| task["id"] == root_task_id)
        );
        Ok(())
    }

    #[tokio::test]
    async fn recurring_actions_create_list_update_and_get() -> TestResult<()> {
        let task_tool = tool().await?;
        let template = task_tool
            .execute(serde_json::json!({
                "action": "template_create",
                "subject": "Weekly review",
            }))
            .await?;
        let template_id = template["template"]["id"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("missing template id"))?;

        let recurring = task_tool
            .execute(serde_json::json!({
                "action": "recurring_create",
                "name": "Weekly review cadence",
                "template_id": template_id,
                "schedule": {
                    "kind": "every",
                    "every_ms": 60000,
                    "anchor_ms": 0
                }
            }))
            .await?;
        let recurring_id = recurring["recurring"]["id"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("missing recurring id"))?;

        let listed = task_tool
            .execute(serde_json::json!({
                "action": "recurring_list",
            }))
            .await?;
        assert_eq!(listed["count"], 1);

        let updated = task_tool
            .execute(serde_json::json!({
                "action": "recurring_update",
                "id": recurring_id,
                "name": "Weekly review cadence v2",
            }))
            .await?;
        assert_eq!(updated["recurring"]["name"], "Weekly review cadence v2");

        let fetched = task_tool
            .execute(serde_json::json!({
                "id": recurring_id,
                "action": "recurring_get",
            }))
            .await?;
        assert_eq!(fetched["recurring"]["name"], "Weekly review cadence v2");
        Ok(())
    }

    #[tokio::test]
    async fn recurring_materialize_action_creates_tasks() -> TestResult<()> {
        let task_tool = tool().await?;
        let template = task_tool
            .store
            .create_task_template(NewTaskTemplate {
                slug: Some("weekly-review".to_string()),
                title: "Weekly review".to_string(),
                description: Some("Review active work".to_string()),
                priority: TaskPriority::Medium,
                list_key: Some("default".to_string()),
                goal_id: None,
                parent_template_id: None,
                owner: Some("operator".to_string()),
                project_id: None,
                blocked_by: Vec::new(),
            })
            .await?;
        let recurring = task_tool
            .store
            .create_recurring_task(NewRecurringTask {
                name: "Weekly review cadence".to_string(),
                template_id: template.id.clone(),
                schedule: moltis_cron::types::CronSchedule::Every {
                    every_ms: 60_000,
                    anchor_ms: Some(0),
                },
                enabled: true,
                list_key: Some("default".to_string()),
                goal_id: None,
                owner: Some("operator".to_string()),
                project_id: None,
            })
            .await?;

        let materialized = task_tool
            .execute(serde_json::json!({
                "action": "recurring_materialize",
                "id": recurring.id,
                "force": true,
                "now_ms": 120000,
            }))
            .await?;
        assert_eq!(materialized["result"]["count"], 1);

        let tasks = task_tool
            .execute(serde_json::json!({
                "action": "list",
            }))
            .await?;
        assert_eq!(tasks["count"], 1);

        let fetched = task_tool
            .execute(serde_json::json!({
                "action": "recurring_get",
                "id": recurring.id,
            }))
            .await?;
        assert_eq!(fetched["recurring"]["lastMaterializedAtMs"], 120000);
        Ok(())
    }
}
