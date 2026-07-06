---
name: kisautotrade-rust
description: "Rust/Tauri backend bridge for Claude Code in the repository-owned trading app. Use for src-tauri, IPC commands, AppState, trading strategies, order/risk guards, serde camelCase, axum handlers, storage, async tasks, and cargo validation."
---

# Rust/Tauri Bridge

Read the canonical repository skill before making Rust or Tauri changes:

`.github/skills/rust-skills/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `docs/coding-guide.md` for IPC/AppState/daemon workflows when relevant
- `docs/ipc-commands.md` when adding or changing an IPC command surface

The repository skill is the source of truth. If a Rust, Tauri, serde, IPC, strategy, or risk-control pattern changes, update the repository skill, not this bridge.

Validate with `(cd src-tauri && cargo check)` — aim for zero warnings before reporting work done.
