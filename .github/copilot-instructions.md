# KISAutoTrade — Copilot / Codex Compatibility

이 프로젝트의 최신 에이전트 지침은 `.github/codex-instructions.md`와 루트 `AGENTS.md`를 기준으로 유지합니다.

GitHub Copilot에서 이 파일을 읽는 경우에도 아래 순서로 작업하세요.

1. `AGENTS.md`를 먼저 읽습니다.
2. `.github/codex-instructions.md`의 규칙을 따릅니다.
3. 도메인별 세부 규칙은 `.github/skills/**/SKILL.md`를 참고합니다.
4. 개선 백로그는 `todo.md`에 기록합니다.
5. 구조·심볼·중복 helper 작업은 `.github/agents/*.agent.md`의 전용 역할과
   `docs/agent-tooling.md`의 라우팅을 따릅니다.

플랫폼별 역할 이름은 Codex
`project_mapper`/`symbol_navigator`/`helper_curator`, Copilot
`project-map-maintainer`/`symbol-navigator`/`helper-curator`입니다.

Codex에서는 루트 `AGENTS.md`를 최상위 지침으로 사용하고, 프로젝트 브리지 스킬 `.codex/skills/kisautotrade-*`가 현재 작업 저장소 루트 기준으로 `.github/skills/**/SKILL.md` 원본을 다시 읽도록 구성되어 있습니다. 프로젝트 서브 에이전트는 `.codex/agents/*.toml`, 동시 실행 한도는 `.codex/config.toml`에 있습니다. Codex 런타임이 계정 스킬만 읽는 경우 `scripts/sync-codex-skills.ps1`로 `~/.codex/skills`에 동기화합니다. 스킬 내용을 수정할 때는 브리지가 아니라 저장소의 원본 `.github/skills/**/SKILL.md`를 갱신하세요.
