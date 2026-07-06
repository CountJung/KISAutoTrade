---
name: kisautotrade-frontend-fsd
description: "Frontend Feature-Sliced Design bridge for Claude Code in the repository-owned trading app. Use when changing src/ structure, moving React modules, adding shared/entities/features/widgets/pages layers, splitting pages/components, or reviewing FSD import boundaries."
---

# Frontend FSD Bridge

Read the canonical repository skill before changing frontend structure:

`.github/skills/frontend-fsd/SKILL.md`

Also read:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `docs/project-map.md` when module paths change

The repository skill is the source of truth. If FSD layer rules or migration sequencing changes, update the repository skill, not this bridge.

Run `node scripts/check-fsd-imports.mjs` when moving modules across layers, if applicable.
