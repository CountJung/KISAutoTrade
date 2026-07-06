---
name: kisautotrade-toss-api
description: "Toss Securities Open API bridge for Claude Code in the repository-owned trading app. Use for Toss OpenAPI JSON, OAuth2 Client Credentials, X-Tossinvest-Account, REST endpoints, rate-limit/error envelope, and Toss broker adapter work."
---

# Toss API Bridge

Read the canonical repository skill before making Toss API decisions:

`.github/skills/toss-api/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `docs/toss-openapi.md`
- `docs/toss-readonly-small-order-checklist.md` when touching real-money order paths

The repository skill is the source of truth. If a Toss API behavior, endpoint, response field, rate-limit header, or error code is newly verified, update `.github/skills/toss-api/SKILL.md` and `docs/toss-openapi.md`, not this bridge.

Frontend code must gate Toss-only commands (`getTossMarketSnapshot`, `getTossStockSafety`, `checkTossOrderPreflight`, etc.) behind `active_broker_id === 'toss'`, and must not fall back to KIS-only commands (`getOverseasPrice`, `usePrice`, etc.) when the Toss profile is active — see `kisautotrade-project` for this repo's recurring two-broker bug pattern.
