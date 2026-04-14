# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Fix: Google/Gemini via OpenRouter rejects tool schemas (GitHub #716)

## Context

When a user selects a Google/Gemini model through OpenRouter (e.g.
`google/gemma-4-31b-it:free`), the agent loop crashes with HTTP 400
`INVALID_ARGUMENT` because Google's API rejects our strict-mode-patched
tool schemas.

**Root cause**: `to_openai_tools()` unconditionally applies
`patch_schema_for_strict_mode()` + `strict: true`. The strict patch:

1. Forces ALL properties into ...

### Prompt 2

maybe the compat openai config should include that boolean so you can enable/disable that mode not just for openrouter but others if needed.

### Prompt 3

I think another PR added a schema tool to map on expectation, instead of doing it manually, can you check?

### Prompt 4

i m not talking about the config one, i m talking about one we use already and a crate for llm expected json schema

### Prompt 5

it is e2e_scenarios/tool_arg_serialization.json yes

### Prompt 6

ok so you can add scenario for this bug too, so we know it keeps working

### Prompt 7

commit push create a PR

