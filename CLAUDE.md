@AGENTS.md

## Claude Code 전용 참고

이 저장소는 `AGENTS.md`를 Codex/Copilot/Claude Code 공용 정본으로 사용합니다. 위 `@AGENTS.md` import로 전체 내용이 이 세션에 자동 로드됩니다 — 별도로 다시 읽을 필요는 없지만, 최신 상태를 확인하려면 파일을 직접 열어보세요.

- 도메인 스킬 원본은 `.github/skills/**/SKILL.md`에 있습니다. Claude Code는 프로젝트 로컬 `.claude/skills/`만 자동 탐색하므로, `.claude/skills/kisautotrade-*`에 있는 브리지 스킬이 이 원본을 가리킵니다 (Codex용 `.codex/skills/kisautotrade-*`와 동일한 구조 — 하나를 바꾸면 다른 하나도 맞춰 갱신).
- 스킬 내용을 수정할 때는 브리지 파일(`.claude/skills/**`, `.codex/skills/**`)이 아니라 저장소 원본(`.github/skills/**/SKILL.md`, `AGENTS.md`)을 갱신하세요.
- `AGENTS.md`의 빌드/검증 명령은 PowerShell 예시입니다. 이 머신(macOS/zsh)에서는 아래로 대체합니다:
  ```bash
  (cd src-tauri && cargo check)   # Rust 빠른 검증
  npx tsc --noEmit                # TypeScript 타입 체크
  ```
- 두 개의 독립된 증권사 프로파일(KIS/한국투자증권, Toss/토스증권)이 `active_broker_id`로 전환됩니다. 시세 조회·주문·잔고 등 브로커 연동 코드를 작성/수정할 때는 반드시 활성 프로파일(`useAppConfig().data?.active_broker_id` 또는 백엔드의 `profile.broker_id`)을 분기해서 처리하세요 — KIS 전용 커맨드(`getOverseasPrice` 등)를 Toss 활성 상태에서도 무조건 호출하는 실수가 이 저장소에서 반복된 버그 패턴입니다.
