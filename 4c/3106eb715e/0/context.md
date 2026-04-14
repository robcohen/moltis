# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Plan: Pass sender name to LLM for all channel types

## Context

GitHub issue #713: When Moltis receives messages from Telegram (or Discord, WhatsApp, etc.),
the LLM cannot identify who sent the message. All channel handlers already extract `sender_name`
and `username` into `ChannelMessageMeta`, and this metadata is persisted in the session JSON.
However, `ChatMessage::User` has no `name` field, so the identity is silently dropped when
converting persisted mes...

