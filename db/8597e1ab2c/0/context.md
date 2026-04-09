# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Fix: 404 selecting Ollama model (issue #615)

## Context

Users on Linux selecting an Ollama model (e.g., `gemma4:e2b`) in the web UI get a 404 error.
Model discovery works (models are listed via `/api/tags`), but the probe request to
`/v1/chat/completions` fails. Two root causes identified:

1. **Hostname divergence**: Discovery uses `localhost:11434`, but the probe uses
   `127.0.0.1:11434`. On Linux with IPv6, these can resolve differently.
2. **No fallback...

### Prompt 2

commit push create a PR

### Prompt 3

Fix and resolve PR comments

### Prompt 4

Fix and resolve PR comments

