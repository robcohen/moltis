# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Prompt Caching for Moltis

## Context

Moltis users are requesting prompt caching. Moltis already **tracks** cache tokens in `Usage` (`cache_read_tokens`, `cache_write_tokens`) and parses them from API responses, but never **requests** caching — no cache markers are sent in requests.

After researching OpenClaw's implementation and all providers Moltis supports, here's the landscape:

### Provider Cache Support Matrix

| Provider | Caching? | Client action nee...

### Prompt 2

Anything you can improve for caching on other providers/llms?

### Prompt 3

And look deeply into openclaw to make sure it does not offer caching we don't offer

### Prompt 4

Add all missing features compared to openclaw

### Prompt 5

Please add a nice git commit, push, create a PR.

