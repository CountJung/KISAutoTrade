---
name: kisautotrade-toss-api
description: "Toss Securities Open API bridge for Codex in the repository-owned trading app. Use for Toss OpenAPI JSON, OAuth2 Client Credentials, X-Tossinvest-Account, REST endpoints, rate-limit/error envelope, and Toss broker adapter work."
---

# Toss API Bridge

Resolve the repository root from the current workspace:

1. Prefer the current working directory when it contains `AGENTS.md`.
2. Otherwise walk upward until `AGENTS.md` and `.github/skills/toss-api/SKILL.md` are found.
3. If no such root is found, ask the user for the repository root instead of using stale paths.

Read the canonical repository skill before making Toss API decisions:

`.github/skills/toss-api/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `docs/toss-openapi.md`

The repository skill is the source of truth. If a Toss API behavior, endpoint, response field, rate-limit header, or error code is newly verified, update `.github/skills/toss-api/SKILL.md` and `docs/toss-openapi.md`, not this bridge.
