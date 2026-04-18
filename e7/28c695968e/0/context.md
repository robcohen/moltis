# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Plan: Fix repeated sandbox package installs + add observability

## Context

GitHub discussion #781: User sees moltis continuously spawning `dpkg` processes
(Docker image builds / package installs) without any moltis logs explaining what's
happening or why.

**Root cause analysis**: There is no single infinite loop, but the architecture
creates repeated provisioning under failure conditions:

1. `ensure_ready()` is called on **every** tool exec (idempotent — r...

### Prompt 2

commit and push, create a PR

