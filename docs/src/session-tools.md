# Session Tools

Session tools enable persistent, asynchronous coordination between agent sessions.

## Available Tools

### `sessions_list`

List sessions visible to the current policy.

Input:

```json
{
  "filter": "optional text",
  "limit": 20
}
```

### `sessions_history`

Read message history from a target session.

Input:

```json
{
  "key": "agent:research:main",
  "limit": 20,
  "offset": 0
}
```

### `sessions_search`

Search prior session history for relevant snippets. By default the current
session is excluded when `_session_key` is available in tool context.

```json
{
  "query": "checkpoint rollback",
  "limit": 5,
  "exclude_current": true
}
```

### `sessions_send`

Send a message to another session, optionally waiting for reply.

```json
{
  "key": "agent:coder:main",
  "message": "Please implement JWT middleware",
  "wait_for_reply": true,
  "context": "coordinator"
}
```

### `task_list`

Manage durable shared tasks backed by the work graph instead of ad hoc JSON files.
Claiming a task with `_session_key` now binds that session to the task so prompt
and context APIs can explain what the session is working on.

```json
{
  "action": "create",
  "list_id": "default",
  "subject": "Implement gateway auth audit trail",
  "priority": "high",
  "blocked_by": []
}
```

Supported actions:

- `create`
- `list`
- `get`
- `update`
- `claim`
- `comment`
- `comments`
- `goal_plan`
- `tracker_import`
- `tracker_links`
- `template_create`
- `template_list`
- `template_get`
- `template_update`
- `template_instantiate`
- `recurring_create`
- `recurring_list`
- `recurring_get`
- `recurring_update`
- `recurring_materialize`

### Work RPC

The gateway also exposes a first-class work API for goals, tasks, runs, comments,
approvals, usage attribution, budgets, and operator dashboards:

- `work.goals.{list,get,create,update,delete}`
- `work.goals.plan`
- `work.templates.{list,get,create,update,instantiate}`
- `work.tasks.{list,get,create,update,claim}`
- `work.comments.{list,add}`
- `work.runs.{list,create,update}`
- `work.approvals.{list,create,update}`
- `work.costs.{list,record}`
- `work.budgets.{list,save,enforce}`
- `work.recurring.{list,get,create,update,materialize}`
- `work.recurring.materializations.list`
- `work.trackers.links.list`
- `work.trackers.import`
- `work.dashboard`
- `work.portfolio`
- `work.package.{export,import}`

Cron jobs and heartbeat settings can now bind scheduled agent turns to a
durable work task with `taskId` / `task_id`, so autonomous scheduled work uses
the same budgets, approvals, and run tracking as interactive sessions.

### Templates And Recurring Work

`work.templates.*` stores reusable task blueprints, including parent/child
template trees and dependency edges. `work.templates.instantiate` turns a root
template into real durable tasks in one transaction.

`work.goals.plan` auto-generates a starter template tree from an existing goal
using `delivery`, `investigation`, or `maintenance` presets, and can
optionally instantiate the generated template immediately.

`work.recurring.*` stores scheduled materializers that point at a root
template. `work.recurring.materialize` creates due task instances and records
each emission in `work.recurring.materializations.list`.

The Projects page exposes these controls directly, including template creation,
template instantiation, selected-template detail, recurring-job creation, and
one-click materialization with recent emission history for operator-driven
workflows.

`work.portfolio` aggregates work across every project and highlights
cross-project task dependencies so shared blockers stop hiding in per-project
boards.

`work.trackers.import` reconciles an external tracker snapshot into a durable
task and stores a stable external-link record. That gives future GitHub, Linear,
or Jira adapters a common sync target instead of bespoke task-import logic.

### Work Package Format

Work packages export canonical work items as markdown files plus a small YAML
sidecar for runtime-only metadata:

- `goals/*.md` for durable goal definitions
- `tasks/*.md` for durable task definitions
- `MOLTIS.work.yaml` for budgets and package metadata

Use `work.package.export` with a `rootPath` and optional `projectId` to write a
package, then `work.package.import` with the same `rootPath` to round-trip it
back into the work graph.

## Session Access Policy

Configure policy in a preset to control what sessions a sub-agent can access:

```toml
[agents.presets.coordinator]
tools.allow = ["sessions_list", "sessions_history", "sessions_search", "sessions_send", "task_list", "spawn_agent"]
sessions.can_send = true

[agents.presets.observer]
tools.allow = ["sessions_list", "sessions_history", "sessions_search"]
sessions.key_prefix = "agent:research:"
sessions.can_send = false
```

Policy fields:

- `key_prefix`: restrict visibility by session-key prefix
- `allowed_keys`: extra explicit session keys
- `can_send`: controls `sessions_send` (default: `true`)
- `cross_agent`: allow access to sessions owned by other agents (default: `false`)

When no policy is configured, all sessions are visible and sendable.

## Coordination Patterns

Use `spawn_agent` when work is short-lived and synchronous.

Use session tools when you need:

- long-lived specialist sessions
- handoffs with durable history
- asynchronous team-style orchestration

Common coordinator flow:

1. `sessions_list` to discover workers
2. `sessions_search` to find prior related work
3. `sessions_history` to inspect progress
4. `sessions_send` to dispatch next tasks
5. `task_list` to track and claim durable cross-session work items
