---
name: kisautotrade-project
description: "Repository-owned trading app instruction bridge for Claude Code. Use when working in this Rust/Tauri/React auto-trading repository with AGENTS.md, CLAUDE.md, .github/skills, KIS/Toss broker profiles, or repository-wide agent behavior and conventions."
---

# Repository Project Bridge

This is a Claude Code bridge for repository-owned instructions kept alongside the project so Codex, Copilot, and Claude Code all follow the same rules without duplication.

Before acting on this repository, read these canonical files from the repo root (the directory containing `AGENTS.md`):

1. `AGENTS.md` — top-level agent guide (also imported automatically by root `CLAUDE.md`)
2. `.github/codex-instructions.md` — detailed working conventions (build/verify commands, IPC/AppState/daemon patterns, safety rules)
3. `todo.md` — improvement backlog, priorities, status
4. `docs/project-map.md` — full directory map and module responsibilities

Do not treat this bridge file as the source of truth. The files above are canonical; update them (not this bridge) when project rules change.

For domain work, also load the matching bridge skill:

- KIS Open API, TR-ID, REST/WebSocket, paper-trading quirks: `kisautotrade-kis-api`
- Toss Securities OpenAPI, OAuth2, account scoping, order preflight: `kisautotrade-toss-api`
- Rust/Tauri backend, IPC, serde, strategy/trading/broker modules: `kisautotrade-rust`
- React, TanStack Query, performance, MUI imports: `kisautotrade-react`
- Frontend Feature-Sliced Design layout and import boundaries: `kisautotrade-frontend-fsd`
- UI conventions, MUI layout, charts, finance display: `kisautotrade-ui`

## Two-broker awareness

This app supports two independently-configured broker profiles — Korea Investment Securities (KIS/한국투자증권) and Toss Securities (토스증권) — selected via `active_broker_id` in `AppConfigView` (`'kis' | 'toss'`). Any frontend code that resolves ticker/price/name data, submits orders, or reads holdings must branch on the *active* broker profile rather than assuming KIS. A common bug pattern in this repo is UI code calling a KIS-only command (e.g. `getOverseasPrice`) unconditionally even when the Toss profile is active — check `useAppConfig().data?.active_broker_id` (frontend) or the profile's `broker_id` (backend) before choosing which broker's API to call.
