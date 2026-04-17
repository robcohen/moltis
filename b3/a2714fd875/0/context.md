# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Plan: NIP-59 Gift Wrap support for Nostr DMs

**Issue**: [#759](https://github.com/moltis-org/moltis/issues/759)

## Context

The `moltis-nostr` crate uses NIP-04 (kind:4) for all DMs. NIP-04 is deprecated because it leaks social graph metadata — both sender and receiver pubkeys are visible in plain text. NIP-17/NIP-59 (Gift Wraps, kind:1059) wraps messages with ephemeral keys, fully hiding metadata. The nostr-sdk 0.44 already has complete NIP-59 support behin...

### Prompt 2

commit push create a PR

### Prompt 3

Fix and resolve PR comments

