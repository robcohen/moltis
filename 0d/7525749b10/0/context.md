# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Fix: OpenAI-compatible provider timeout when no model specified (#502)

## Context

When adding an OpenAI-compatible provider via the UI without specifying a model ID, the user gets "Connection timed out" instead of the expected behavior: discovering models from `/v1/models` and presenting them for selection.

**Root cause**: In `validate_key`, custom OpenAI-compatible providers without a model always go through the full probe flow: `build_registry` → fire dis...

### Prompt 2

commit and push, create a PR

### Prompt 3

Fix comments from https://github.com/moltis-org/moltis/pull/507 and resolve conversations

