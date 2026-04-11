# Session Context

## User Prompts

### Prompt 1

Look at https://github.com/moltis-org/moltis/issues/658 and plan a fix

### Prompt 2

proceed and implement

### Prompt 3

proceed

### Prompt 4

commit push and create a PR

### Prompt 5

Fix and resolve PR comments

### Prompt 6

Base directory for this skill: /Users/penso/.claude/skills/check-pr

# Check PR

Analyze a pull request (GitHub), merge request (GitLab), or shelved changelist (Perforce) for review comments, status checks, and description completeness, then help address any issues found.

## Inputs

- **PR/MR/CL number** (optional): If not provided, detect the PR/MR for the current branch, or the default pending changelist for p4.

## Instructions

### 0. Detect platform

First check if the user is working i...

