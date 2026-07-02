---
name: kisautotrade-ui
description: "UI conventions bridge for Codex in the repository-owned trading app. Use for MUI v6 layouts, financial UI, chart components, lightweight-charts, colors, typography, empty/loading/error states, dialogs, resizers, and dashboard/trading/settings page presentation."
---

# UI Bridge

Resolve the repository root from the current workspace:

1. Prefer the current working directory when it contains `AGENTS.md`.
2. Otherwise walk upward until `AGENTS.md` and `.github/skills/ui-conventions/SKILL.md` are found.
3. If no such root is found, ask the user for the repository root instead of using stale paths.

Read the canonical repository skill before making UI changes:

`.github/skills/ui-conventions/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `.github/skills/react-best-practices/SKILL.md` when React performance or imports are involved

The repository skill is the source of truth. If a UI convention, chart pattern, color rule, or financial display rule changes, update the repository skill, not this bridge.
