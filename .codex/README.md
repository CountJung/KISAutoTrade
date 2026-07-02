# KISAutoTrade Codex Settings

This directory contains project-owned Codex bridge skills. Keep these files in Git so the repository, not a single user account, owns the Codex setup.

The canonical domain instructions remain in:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `.github/skills/**/SKILL.md`

The bridge skills under `.codex/skills/kisautotrade-*` only route Codex to those repository files. If a Codex runtime does not auto-discover project-local skills, run `scripts/sync-codex-skills.ps1` to mirror them into `$CODEX_HOME/skills` or `~/.codex/skills`.
