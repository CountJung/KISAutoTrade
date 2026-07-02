---
name: kisautotrade-rust
description: "Rust/Tauri backend bridge for Codex in the repository-owned trading app. Use for src-tauri, IPC commands, AppState, trading strategies, order/risk guards, serde camelCase, axum handlers, storage, async tasks, and cargo validation."
---

# Rust/Tauri Bridge

Resolve the repository root from the current workspace:

1. Prefer the current working directory when it contains `AGENTS.md`.
2. Otherwise walk upward until `AGENTS.md` and `.github/skills/rust-skills/SKILL.md` are found.
3. If no such root is found, ask the user for the repository root instead of using stale paths.

Read the canonical repository skill before making Rust or Tauri changes:

`.github/skills/rust-skills/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `docs/coding-guide.md` for IPC/AppState/daemon workflows when relevant

The repository skill is the source of truth. If a Rust, Tauri, serde, IPC, strategy, or risk-control pattern changes, update the repository skill, not this bridge.
