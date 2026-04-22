use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    Result, SqliteWorkStore,
    types::{
        BudgetScope, GoalStatus, NewWorkBudget, TaskPriority, TaskStatus, WorkPackageExportResult,
        WorkPackageImportResult,
    },
};

const PACKAGE_VERSION: u32 = 1;
const MANIFEST_FILE: &str = "MOLTIS.work.yaml";

#[derive(Debug, Serialize, Deserialize)]
struct WorkPackageManifest {
    version: u32,
    project_id: Option<String>,
    #[serde(default)]
    budgets: Vec<BudgetManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BudgetManifestEntry {
    scope: BudgetScope,
    scope_id: String,
    soft_limit_tokens: Option<u64>,
    hard_limit_tokens: Option<u64>,
    soft_limit_usd: Option<f64>,
    hard_limit_usd: Option<f64>,
    note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoalFrontmatter {
    id: String,
    slug: Option<String>,
    title: String,
    status: GoalStatus,
    project_id: Option<String>,
    parent_goal_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskFrontmatter {
    id: String,
    list_key: Option<String>,
    title: String,
    status: TaskStatus,
    priority: TaskPriority,
    goal_id: Option<String>,
    parent_task_id: Option<String>,
    owner: Option<String>,
    project_id: Option<String>,
    #[serde(default)]
    blocked_by: Vec<String>,
}

fn slugify(value: &str) -> String {
    let slug = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "item".to_string()
    } else {
        slug
    }
}

fn split_frontmatter(content: &str) -> Result<(String, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---\n") {
        return Err(crate::Error::message(
            "markdown file must start with YAML frontmatter delimited by ---",
        ));
    }
    let after_open = &trimmed[4..];
    let close_pos = after_open
        .find("\n---\n")
        .ok_or_else(|| crate::Error::message("markdown file is missing closing ---"))?;
    let frontmatter = after_open[..close_pos].trim().to_string();
    let body = after_open[close_pos + 5..].trim().to_string();
    Ok((frontmatter, body))
}

fn markdown_with_frontmatter<T: Serialize>(frontmatter: &T, body: Option<&str>) -> Result<String> {
    let frontmatter = serde_yaml::to_string(frontmatter)?;
    let body = body.unwrap_or("").trim();
    if body.is_empty() {
        Ok(format!("---\n{frontmatter}---\n"))
    } else {
        Ok(format!("---\n{frontmatter}---\n\n{body}\n"))
    }
}

async fn read_markdown_files(dir: &Path) -> Result<Vec<(PathBuf, String)>> {
    let mut files = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return Ok(files);
    };

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        files.push((path.clone(), tokio::fs::read_to_string(path).await?));
    }

    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

impl SqliteWorkStore {
    pub async fn export_package(
        &self,
        root: &Path,
        project_id: Option<&str>,
    ) -> Result<WorkPackageExportResult> {
        let goals_dir = root.join("goals");
        let tasks_dir = root.join("tasks");
        tokio::fs::create_dir_all(&goals_dir).await?;
        tokio::fs::create_dir_all(&tasks_dir).await?;

        let tasks = self
            .list_tasks(&crate::types::TaskFilter {
                project_id: project_id.map(ToOwned::to_owned),
                ..crate::types::TaskFilter::default()
            })
            .await?;
        let mut goals = self.list_goals().await?;
        let task_goal_ids: std::collections::HashSet<String> = tasks
            .iter()
            .filter_map(|task| task.goal_id.clone())
            .collect();
        goals.retain(|goal| {
            project_id.is_some_and(|project_id| goal.project_id.as_deref() == Some(project_id))
                || task_goal_ids.contains(&goal.id)
        });

        for goal in &goals {
            let name = format!(
                "{}-{}.md",
                slugify(goal.slug.as_deref().unwrap_or(&goal.title)),
                &goal.id[..goal.id.len().min(8)]
            );
            let content = markdown_with_frontmatter(
                &GoalFrontmatter {
                    id: goal.id.clone(),
                    slug: goal.slug.clone(),
                    title: goal.title.clone(),
                    status: goal.status,
                    project_id: goal.project_id.clone(),
                    parent_goal_id: goal.parent_goal_id.clone(),
                },
                goal.description.as_deref(),
            )?;
            tokio::fs::write(goals_dir.join(name), content).await?;
        }

        for task in &tasks {
            let name = format!(
                "{}-{}.md",
                slugify(&task.title),
                &task.id[..task.id.len().min(8)]
            );
            let content = markdown_with_frontmatter(
                &TaskFrontmatter {
                    id: task.id.clone(),
                    list_key: task.list_key.clone(),
                    title: task.title.clone(),
                    status: task.status,
                    priority: task.priority,
                    goal_id: task.goal_id.clone(),
                    parent_task_id: task.parent_task_id.clone(),
                    owner: task.owner.clone(),
                    project_id: task.project_id.clone(),
                    blocked_by: task.blocked_by.clone(),
                },
                task.description.as_deref(),
            )?;
            tokio::fs::write(tasks_dir.join(name), content).await?;
        }

        let goal_ids: std::collections::HashSet<String> =
            goals.iter().map(|goal| goal.id.clone()).collect();
        let task_ids: std::collections::HashSet<String> =
            tasks.iter().map(|task| task.id.clone()).collect();
        let all_budgets = self.list_budgets(None, None).await?;
        let budgets = all_budgets
            .into_iter()
            .filter(|budget| match budget.scope {
                BudgetScope::Task => task_ids.contains(&budget.scope_id),
                BudgetScope::Goal => goal_ids.contains(&budget.scope_id),
                BudgetScope::Project => {
                    project_id.is_none_or(|project_id| budget.scope_id == project_id)
                },
            })
            .map(|budget| BudgetManifestEntry {
                scope: budget.scope,
                scope_id: budget.scope_id,
                soft_limit_tokens: budget.soft_limit_tokens,
                hard_limit_tokens: budget.hard_limit_tokens,
                soft_limit_usd: budget.soft_limit_usd,
                hard_limit_usd: budget.hard_limit_usd,
                note: budget.note,
            })
            .collect::<Vec<_>>();

        let manifest = serde_yaml::to_string(&WorkPackageManifest {
            version: PACKAGE_VERSION,
            project_id: project_id.map(ToOwned::to_owned),
            budgets: budgets.clone(),
        })?;
        tokio::fs::write(root.join(MANIFEST_FILE), manifest).await?;

        Ok(WorkPackageExportResult {
            root_path: root.display().to_string(),
            goal_count: goals.len(),
            task_count: tasks.len(),
            budget_count: budgets.len(),
        })
    }

    pub async fn import_package(&self, root: &Path) -> Result<WorkPackageImportResult> {
        let manifest_path = root.join(MANIFEST_FILE);
        let manifest = if manifest_path.is_file() {
            let content = tokio::fs::read_to_string(manifest_path).await?;
            Some(serde_yaml::from_str::<WorkPackageManifest>(&content)?)
        } else {
            None
        };

        let goal_files = read_markdown_files(&root.join("goals")).await?;
        let task_files = read_markdown_files(&root.join("tasks")).await?;

        for (_, content) in &goal_files {
            let (frontmatter, body) = split_frontmatter(content)?;
            let goal: GoalFrontmatter = serde_yaml::from_str(&frontmatter)?;
            self.import_goal_record(
                &goal.id,
                goal.slug,
                goal.title,
                (!body.is_empty()).then_some(body),
                goal.status,
                goal.project_id,
                goal.parent_goal_id,
            )
            .await?;
        }

        for (_, content) in &task_files {
            let (frontmatter, body) = split_frontmatter(content)?;
            let task: TaskFrontmatter = serde_yaml::from_str(&frontmatter)?;
            self.import_task_record(
                &task.id,
                task.list_key,
                task.title,
                (!body.is_empty()).then_some(body),
                task.status,
                task.priority,
                task.goal_id,
                task.parent_task_id,
                task.owner,
                task.project_id,
                task.blocked_by,
            )
            .await?;
        }

        let budgets_imported = if let Some(manifest) = manifest {
            for budget in &manifest.budgets {
                self.save_budget(NewWorkBudget {
                    scope: budget.scope,
                    scope_id: budget.scope_id.clone(),
                    soft_limit_tokens: budget.soft_limit_tokens,
                    hard_limit_tokens: budget.hard_limit_tokens,
                    soft_limit_usd: budget.soft_limit_usd,
                    hard_limit_usd: budget.hard_limit_usd,
                    note: budget.note.clone(),
                })
                .await?;
            }
            manifest.budgets.len()
        } else {
            0
        };

        Ok(WorkPackageImportResult {
            root_path: root.display().to_string(),
            goals_imported: goal_files.len(),
            tasks_imported: task_files.len(),
            budgets_imported,
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use {
        super::*,
        crate::types::{GoalStatus, NewTask, TaskFilter, TaskPriority, TaskStatus},
    };

    async fn test_store() -> Result<SqliteWorkStore> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
        crate::run_migrations(&pool).await?;
        Ok(SqliteWorkStore::new(pool))
    }

    #[tokio::test]
    async fn exports_and_imports_markdown_package() -> Result<()> {
        let store = test_store().await?;
        let goal = store
            .create_goal(crate::types::NewGoal {
                slug: Some("ship-work".to_string()),
                title: "Ship work".to_string(),
                description: Some("Move durable work into markdown.".to_string()),
                status: GoalStatus::Active,
                project_id: Some("proj-portable".to_string()),
                parent_goal_id: None,
            })
            .await?;
        let task = store
            .create_task(NewTask {
                list_key: Some("default".to_string()),
                title: "Export package".to_string(),
                description: Some("Write markdown files.".to_string()),
                status: TaskStatus::Pending,
                priority: TaskPriority::High,
                goal_id: Some(goal.id.clone()),
                parent_task_id: None,
                owner: Some("worker".to_string()),
                project_id: Some("proj-portable".to_string()),
                session_key: None,
                blocked_by: Vec::new(),
            })
            .await?;
        let _ = store
            .save_budget(NewWorkBudget {
                scope: BudgetScope::Project,
                scope_id: "proj-portable".to_string(),
                soft_limit_tokens: Some(100),
                hard_limit_tokens: Some(200),
                soft_limit_usd: None,
                hard_limit_usd: None,
                note: Some("portable".to_string()),
            })
            .await?;

        let dir = tempdir()?;
        let export = store
            .export_package(dir.path(), Some("proj-portable"))
            .await?;
        assert_eq!(export.goal_count, 1);
        assert_eq!(export.task_count, 1);
        assert_eq!(export.budget_count, 1);
        assert!(dir.path().join("goals").is_dir());
        assert!(dir.path().join("tasks").is_dir());
        assert!(dir.path().join(MANIFEST_FILE).is_file());

        let imported_store = test_store().await?;
        let import = imported_store.import_package(dir.path()).await?;
        assert_eq!(import.goals_imported, 1);
        assert_eq!(import.tasks_imported, 1);
        assert_eq!(import.budgets_imported, 1);

        let imported_tasks = imported_store
            .list_tasks(&TaskFilter {
                project_id: Some("proj-portable".to_string()),
                ..TaskFilter::default()
            })
            .await?;
        assert_eq!(imported_tasks.len(), 1);
        assert_eq!(imported_tasks[0].id, task.id);
        assert_eq!(imported_tasks[0].goal_id.as_deref(), Some(goal.id.as_str()));

        let budgets = imported_store
            .list_budgets(Some(BudgetScope::Project), Some("proj-portable"))
            .await?;
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].soft_limit_tokens, Some(100));
        Ok(())
    }
}
