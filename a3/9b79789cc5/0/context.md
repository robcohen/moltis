# Session Context

## User Prompts

### Prompt 1

Look at https://github.com/moltis-org/moltis/issues/758 and plan for a fix

### Prompt 2

backoff needs to resets if there is an error then a successful connection tho

### Prompt 3

proceed

### Prompt 4

commit push create a PR

### Prompt 5

Base directory for this skill: /Users/penso/.claude/skills/greploop

# Greploop

Iteratively fix a PR/MR/CL until Greptile gives a perfect review: 5/5 confidence, zero unresolved comments.

## Inputs

- **PR/MR/CL number** (optional): If not provided, detect the PR/MR for the current branch, or the default pending changelist for p4.

## Instructions

### 0. Detect platform

First check for Perforce, then fall back to git remote detection:

```bash
# Check for Perforce environment
if p4 info >...

