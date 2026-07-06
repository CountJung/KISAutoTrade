# KISAutoTrade Claude Code Settings

This directory contains project-owned Claude Code bridge skills. Keep these files in Git so the repository, not a single user account, owns the Claude Code setup.

The canonical domain instructions remain in:

- `AGENTS.md` (auto-loaded via the root `CLAUDE.md` `@AGENTS.md` import)
- `.github/codex-instructions.md`
- `.github/skills/**/SKILL.md`

The bridge skills under `.claude/skills/kisautotrade-*` only route Claude Code to those repository files, mirroring the equivalent `.codex/skills/kisautotrade-*` bridges used for Codex. Claude Code auto-discovers `.claude/skills/` in the repo root with no separate sync step. If you add or rename a domain skill under `.github/skills/`, add/update the matching bridge here and under `.codex/skills/` together.
