# Core
- Canonical operating rules: AGENTS.md, then .github/codex-instructions.md.
- Architecture and directory ownership: `mem:backend/core`, `mem:frontend/core`.
- Toolchain and package boundaries: `mem:tech_stack`.
- Commands and completion gates: `mem:suggested_commands`, `mem:task_completion`.
- Project map, Serena, Graphify, and duplicate-helper routing: docs/agent-tooling.md.
- Never read .env, secure_config.json, or profiles.json. Preserve broker/account scope and fail-closed trading safeguards.
