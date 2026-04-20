# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Fix Gemini thought_signature round-tripping (#375) and schema validation (#793)

## Context

**#375**: Gemini 3.x models return `thought_signature` on each tool call in their OpenAI-compat
API responses. When Moltis replays tool calls in subsequent turns, this field must be present or
Gemini rejects with HTTP 400. Currently `ToolCall` only carries `id`, `name`, `arguments` -- the
signature is silently dropped at parse time. This affects both direct Gemini API ...

### Prompt 2

commit push create a PR

### Prompt 3

merge main to this branch, commit and push

### Prompt 4

~/.s/w/m/adorable-protocol adorable-protocol ❯ ./scripts/local-validate.sh
Detected macOS without nvcc; using Darwin-native validation commands (metal for provider/gateway, no Linux CUDA path).
CI still covers the Linux/CUDA all-features path. Override with LOCAL_VALIDATE_* if you need a different split.
Validating PR #795 (8a3f983dab3335293d55ffc889349e54ea6d9d38) in moltis-org/moltis
Publishing commit statuses to: moltis-org/moltis
Current CI workflow: https://github.com/moltis-org/moltis/a...

