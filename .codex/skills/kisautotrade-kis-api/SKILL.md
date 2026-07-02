---
name: kisautotrade-kis-api
description: "Korea Investment Securities KIS Open API bridge for Codex in the repository-owned trading app. Use for authentication, TR-ID, REST/WebSocket endpoints, order, balance, execution, overseas stock, paper trading, API errors, and official sample verification."
---

# KIS API Bridge

Resolve the repository root from the current workspace:

1. Prefer the current working directory when it contains `AGENTS.md`.
2. Otherwise walk upward until `AGENTS.md` and `.github/skills/kis-api/SKILL.md` are found.
3. If no such root is found, ask the user for the repository root instead of using stale paths.

Read the canonical repository skill before making API decisions:

`.github/skills/kis-api/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`

The repository skill is the source of truth. If a KIS API behavior, TR-ID, endpoint, response field, paper-trading limitation, or error code is newly verified, update the repository skill, not this bridge.
