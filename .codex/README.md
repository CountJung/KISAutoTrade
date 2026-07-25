# KISAutoTrade Codex Settings

This directory contains project-owned Codex configuration, custom agents, and
bridge skills. Keep these files in Git so the repository, not a single user
account, owns the workflow.

- `config.toml` enables up to three concurrent project sub-agents.
- `agents/` defines project-map, Serena symbol-navigation, and Graphify helper
  curation roles.
- `skills/` routes Codex to the canonical repository instructions.

The canonical domain instructions remain in:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `.github/skills/**/SKILL.md`

The bridge skills under `.codex/skills/kisautotrade-*` only route Codex to those repository files. If a Codex runtime does not auto-discover project-local skills, run `scripts/sync-codex-skills.ps1` to mirror them into `$CODEX_HOME/skills` or `~/.codex/skills`.

The Serena and Graphify MCP servers are machine tools registered in
`~/.codex/config.toml`. See `docs/agent-tooling.md` for versions, verification,
and restart requirements.
