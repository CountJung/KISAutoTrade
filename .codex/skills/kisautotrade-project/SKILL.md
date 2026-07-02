---
name: kisautotrade-project
description: "Repository-owned trading app instruction bridge for Codex. Use when working in a Rust Tauri React auto-trading repository with AGENTS.md, .github/codex-instructions.md, .github/skills, Korea Investment Securities/KIS API, Copilot migration, project instructions, or repository-wide agent behavior."
---

# Repository Project Bridge

This is a Codex bridge for repository-owned instructions that are kept with the project for GitHub Copilot and Codex compatibility.

Resolve the repository root from the current workspace:

1. Prefer the current working directory when it contains `AGENTS.md`.
2. Otherwise walk upward from the current working directory until `AGENTS.md` and `.github/codex-instructions.md` are found.
3. If no such root is found, do not use stale hardcoded paths; ask the user for the repository root.

Before acting on this repository, read these canonical files from that resolved root:

1. `AGENTS.md`
2. `.github/codex-instructions.md`
3. `todo.md` when backlog, priorities, or status matter

Do not treat this bridge file as the source of truth. The repository files above are canonical and should be updated when project rules change.

For domain work, also load the matching bridge skill:

- KIS Open API, TR-ID, REST/WebSocket, paper-trading quirks: `kisautotrade-kis-api`
- Rust/Tauri backend, IPC, serde, strategy/trading modules: `kisautotrade-rust`
- React, TanStack Query, performance, MUI imports: `kisautotrade-react`
- Frontend Feature-Sliced Design migration: `kisautotrade-frontend-fsd`
- UI conventions, MUI layout, charts, finance display: `kisautotrade-ui`
