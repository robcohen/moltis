# Session Context

## User Prompts

### Prompt 1

Look at https://github.com/moltis-org/moltis/issues/655 and plan a fix

### Prompt 2

Just proceed implementing

### Prompt 3

commit push and create a PR

### Prompt 4

## Context

- Current git status: On branch precious-tibia
Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   crates/chat/src/lib.rs

no changes added to commit (use "git add" and/or "git commit -a")
- Current git diff (staged and unstaged changes): diff --git a/crates/chat/src/lib.rs b/crates/chat/src/lib.rs
index 54e33246..a1c130e3 100644
--- a/crates/chat/src/lib.rs...

### Prompt 5

Fix and solve PR comments

### Prompt 6

Base directory for this skill: /Users/penso/.claude/skills/check-pr

# Check PR

Analyze a pull request (GitHub), merge request (GitLab), or shelved changelist (Perforce) for review comments, status checks, and description completeness, then help address any issues found.

## Inputs

- **PR/MR/CL number** (optional): If not provided, detect the PR/MR for the current branch, or the default pending changelist for p4.

## Instructions

### 0. Detect platform

First check if the user is working i...

