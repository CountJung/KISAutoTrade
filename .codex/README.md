# KISAutoTrade Codex Settings

This directory contains project-owned Codex configuration, custom agents, and
bridge skills. Keep these files in Git so the repository, not a single user
account, owns the workflow.

- `config.toml` enables up to three concurrent project sub-agents.
- `agents/` defines project-map, Serena symbol-navigation, Graphify helper
  curation, equity-research and UI/UX review roles.
- `skills/` routes Codex to the canonical repository instructions.

The canonical domain instructions remain in:

- `AGENTS.md`
- `.github/codex-instructions.md`
- `.github/skills/**/SKILL.md`

The bridge skills under `.codex/skills/kisautotrade-*` only route Codex to those repository files. If a Codex runtime does not auto-discover project-local skills, run `scripts/sync-codex-skills.ps1` to mirror them into `$CODEX_HOME/skills` or `~/.codex/skills`.

The Serena and Graphify MCP servers are machine tools registered in
`~/.codex/config.toml`. See `docs/agent-tooling.md` for versions, verification,
and restart requirements.

UI/UX reviews use `agents/ui_ux_expert.toml`. Invoke `ui_ux_expert` with a
read-only screen/workflow scope and synthetic evidence; reconcile proposals in
`docs/todo.md` before separately authorizing implementation. See
[role usage and session-loading limits](../docs/agent-tooling.md#uiux-전문가-사용).
