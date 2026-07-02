---
name: kisautotrade-react
description: "React/Tauri frontend bridge for Codex in the repository-owned trading app. Use for React, TypeScript, TanStack Query, Tauri invoke wrappers, frontend performance, MUI icon imports, polling intervals, Zustand stores, and npx tsc validation."
---

# React Frontend Bridge

Resolve the repository root from the current workspace:

1. Prefer the current working directory when it contains `AGENTS.md`.
2. Otherwise walk upward until `AGENTS.md` and `.github/skills/react-best-practices/SKILL.md` are found.
3. If no such root is found, ask the user for the repository root instead of using stale paths.

Read the canonical repository skill before making React or TypeScript changes:

`.github/skills/react-best-practices/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `.github/skills/ui-conventions/SKILL.md` when UI layout or display behavior changes
- `.github/skills/frontend-fsd/SKILL.md` when moving frontend modules

The repository skill is the source of truth. If a React performance, TanStack Query, polling, or import pattern changes, update the repository skill, not this bridge.
