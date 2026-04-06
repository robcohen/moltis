# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# External CLI Agent Bridge for Moltis

## Context

Moltis currently runs its own agent loop (LLM provider + tool execution) for all
chat sessions. The Polyphony project at `~/code/polyphony` demonstrates a
pluggable architecture where CLI coding agents (Claude Code, opencode, Codex CLI,
Pi agent) are dispatched as external backends via PTY/tmux sessions and JSON-RPC
protocols, with context snapshots passed between orchestrator and agent.

**Goal**: Let Moltis c...

### Prompt 2

commit push create a PR

### Prompt 3

Fix PR comments

