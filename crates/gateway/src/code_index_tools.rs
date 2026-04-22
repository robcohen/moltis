//! Project-aware wrapper for code-index tools.
//!
//! Wraps the raw code-index tools (`codebase_search`, `codebase_peek`,
//! `codebase_status`) to check whether code indexing is enabled for the
//! given project before delegating execution.

use std::sync::Arc;

use async_trait::async_trait;
use moltis_agents::tool_registry::AgentTool;
use moltis_projects::ProjectStore;
use serde_json::json;

/// Wraps a code-index tool, returning a "disabled" response when
/// `code_index_enabled` is `false` for the specified project.
pub struct ProjectAwareCodeIndexTool {
    inner: Arc<dyn AgentTool>,
    project_store: Arc<dyn ProjectStore>,
}

impl ProjectAwareCodeIndexTool {
    pub fn new(
        inner: Box<dyn AgentTool>,
        project_store: Arc<dyn ProjectStore>,
    ) -> Self {
        Self {
            inner: Arc::from(inner),
            project_store,
        }
    }

    /// Extract the `project_id` from tool parameters.
    fn project_id(params: &serde_json::Value) -> Option<String> {
        params.get("project_id").and_then(|v| v.as_str()).map(String::from)
    }

    /// Check whether code indexing is enabled for the given project.
    async fn is_enabled(&self, project_id: &str) -> bool {
        match self.project_store.get(project_id).await {
            Ok(Some(project)) => project.code_index_enabled,
            _ => true, // Default to enabled if project not found (e.g. agent passes raw id)
        }
    }
}

#[async_trait]
impl AgentTool for ProjectAwareCodeIndexTool {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.inner.parameters_schema()
    }

    async fn warmup(&self) -> anyhow::Result<()> {
        self.inner.warmup().await
    }

    async fn execute(&self, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        if let Some(ref pid) = Self::project_id(&params) {
            if !self.is_enabled(pid).await {
                return Ok(json!({
                    "disabled": true,
                    "message": format!("Code indexing is disabled for project '{pid}'. Enable it in project settings."),
                }));
            }
        }

        self.inner.execute(params).await
    }
}
