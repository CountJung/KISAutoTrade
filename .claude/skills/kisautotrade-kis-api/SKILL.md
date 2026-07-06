---
name: kisautotrade-kis-api
description: "Korea Investment Securities KIS Open API bridge for Claude Code in the repository-owned trading app. Use for authentication, TR-ID, REST/WebSocket endpoints, order, balance, execution, overseas stock, paper trading, API errors, and official sample verification."
---

# KIS API Bridge

Read the canonical repository skill before making API decisions:

`.github/skills/kis-api/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`

The repository skill is the source of truth. If a KIS API behavior, TR-ID, endpoint, response field, paper-trading limitation, or error code is newly verified, update the repository skill, not this bridge.

Never guess KIS TR-ID or endpoint behavior — verify against the official portal or the `koreainvestment/open-trading-api` sample repo, per `AGENTS.md`.
