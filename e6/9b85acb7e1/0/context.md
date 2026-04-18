# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Plan: Add `server.external_url` config field

## Context

GitHub Discussion #782: Users running moltis behind a reverse proxy with SSL termination
(e.g. `https://moltis.example.com`) can't tell moltis its public URL. Moltis thinks it's
on `http://localhost:13131`, which breaks WebAuthn passkey auth and OIDC callbacks.

Currently the only workaround is setting env vars (`MOLTIS_WEBAUTHN_RP_ID` +
`MOLTIS_WEBAUTHN_ORIGIN`), which isn't discoverable or documented ...

### Prompt 2

commit push create a PR

### Prompt 3

Fix and resolve PR comments

