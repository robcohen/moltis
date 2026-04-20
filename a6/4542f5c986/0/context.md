# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Plan: Centralized command registry + wire all commands through all channels

## Context

Issue [#788](https://github.com/moltis-org/moltis/issues/788) — user can't `/stop` or `/peek`
from Telegram. The gateway has 15 working commands but each channel maintains its own hardcoded
list of command names (for interception, help text, and platform registration). Lists have drifted:
Telegram is missing `stop`/`peek`, Slack is missing 8 commands, Nostr has no command ...

### Prompt 2

commit push create a PR

### Prompt 3

Fix and resolve PR comments

