# Session Context

## User Prompts

### Prompt 1

When using openrouter with Jamba large LLM, I see <|eom|> at the end of the messages. This is the full context, can you do something generic to remove that? I feel like LLM response include the tags so we can figure it out.

LLM output:
[
  {
    "cacheReadTokens": 0,
    "cacheWriteTokens": 0,
    "content": "Why don’t scientists trust atoms?\nBecause they make up everything!<|eom|>",
    "created_at": 1776248413913,
    "durationMs": 2816,
    "inputTokens": 20344,
    "llmApiResponse": [
 ...

### Prompt 2

I feel like the LLM response gives that as a separator token, could you leverage that instead of hardcoding the string?

### Prompt 3

again, I feel like you're doing it manually. What happens if another model has <|eom|> but its legitimage.

### Prompt 4

commit push create a PR

