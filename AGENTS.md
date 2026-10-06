# KISAutoTrade — Codex 에이전트 가이드

> 작업 전 이 파일을 읽고 시작한다.  
> 상세 정보는 아래 링크의 문서를 참조한다.

---

## 빠른 참조

| 항목 | 경로 |
|------|------|
| 디렉토리 맵 + 아키텍처 | `docs/project-map.md` |
| 에이전트 코드 탐색 도구 운영 | `docs/agent-tooling.md` |
| 개선 백로그 | `todo.md` |
| IPC 커맨드 목록 (35개+) | `docs/ipc-commands.md` |
| 코딩 가이드 (AppState·IPC·데몬) | `docs/coding-guide.md` |
| KIS API 스킬 | `.github/skills/kis-api/SKILL.md` |
| Toss API 스킬 | `.github/skills/toss-api/SKILL.md` |
| Rust 코딩 스킬 | `.github/skills/rust-skills/SKILL.md` |
| React/Tauri 성능 스킬 | `.github/skills/react-best-practices/SKILL.md` |
| Frontend FSD 스킬 | `.github/skills/frontend-fsd/SKILL.md` |
| UI 컨벤션 스킬 | `.github/skills/ui-conventions/SKILL.md` |
| Codex 상세 지침 | `.github/codex-instructions.md` |
| Copilot 호환 지침 | `.github/copilot-instructions.md` |
| Claude Code 지침 (AGENTS.md import) | `CLAUDE.md` |
| Codex 프로젝트 브리지 스킬 | `.codex/skills/kisautotrade-*` |
| Codex 프로젝트 서브 에이전트 | `.codex/agents/*.toml` |
| Copilot 호환 서브 에이전트 | `.github/agents/*.agent.md` |
| Claude Code 프로젝트 브리지 스킬 | `.claude/skills/kisautotrade-*` |

---

## 핵심 파일 경로

| 역할 | 파일 |
|------|------|
| Tauri IPC 커맨드 + AppState | `src-tauri/src/commands.rs` |
| 백그라운드 데몬 + Builder | `src-tauri/src/lib.rs` |
| KIS REST Client | `src-tauri/src/api/rest.rs` |
| 전략 엔진 | `src-tauri/src/trading/strategy.rs` |
| React 훅 (TanStack Query) | `src/api/hooks.ts` |
| TypeScript 타입 미러 | `src/api/types.ts` |
| invoke() 래퍼 | `src/api/commands.ts` |

---

## 빌드 / 검증

```powershell
cd src-tauri; cargo check            # Rust 빠른 검증
cd ..; npx tsc --noEmit              # TypeScript 타입 체크
npm run check:graphify               # 코드 그래프 freshness 검사
npm run check:project-map            # 파일/모듈 구조 문서 drift 검사
# UI 시각/상호작용 위험 변경: npm run test:e2e 또는 focused Playwright spec 실행
```

**경고 0개** 달성 후 완료 보고.

---

## Codex 작업 원칙

- `.env`, `secure_config.json`, `profiles.json`은 읽지 않는다.
- 코드 변경 전 현재 구현을 먼저 검색하고, 기존 패턴을 우선한다.
- 작업별 검증과 리뷰·문서 게이트가 완료되면 변경을 커밋하고 현재 브랜치를 origin에 푸시한다. 캐시·민감 파일은 제외하며 강제 푸시는 하지 않는다. (사용자 지시: 2026-10-03)
- 교차 모듈 심볼 변경은 `symbol_navigator`에게 Serena 참조 추적을 위임한다.
- 파일 추가·이동·삭제 또는 구조 변경은 `project_mapper`에게 프로젝트 맵 pass를 위임한다.
- 중복 helper 후보는 `helper_curator`에게 Graphify 후보 탐색과 Serena 참조 검증을 위임하고, 실제 동등성이 확인된 경우에만 공용 유틸로 승격한다.
- KIS API 동작·TR-ID·제한사항은 추측하지 말고 공식 포털 또는 `koreainvestment/open-trading-api` 샘플로 확인한다.
- 새 IPC 커맨드는 Rust command, `lib.rs` 등록, TypeScript 타입/래퍼/훅, 문서를 함께 갱신한다.
- 반복 매매·손실 방지 관련 변경은 `todo.md`와 관련 스킬 문서에 남긴다.
- Copilot 호환 지침과 `.github/skills/**`의 원본 스킬은 프로젝트 브리지 스킬(`.codex/skills/kisautotrade-*`)을 통해 재사용한다. Codex 런타임이 계정 스킬만 읽는 경우 `scripts/sync-codex-skills.ps1`로 동기화한다. 원본은 저장소의 `.github/skills/**/SKILL.md`로 유지한다.
- 코드 리뷰 또는 문서 업데이트 조건이 충족되면 `.github/codex-instructions.md`의 위임 게이트를 따른다. subagent 도구가 있으면 review/documentation pass를 위임하고, 도구가 없으면 같은 체크리스트를 직접 수행한 뒤 다음 턴용 위임 프롬프트를 최종 보고에 남긴다.

---

## 최근 변경 요약

> 전체 이력은 `git log --oneline`. 여기는 최근 5건만 유지.

| 날짜 | 한줄 요약 |
|------|----------|
| 2026-10-06 | Rust 1.99 Clippy 7건을 async-trait 0.1.92와 웹 주문 검증 오류 boxing으로 해소; 응답 보존·오프라인 회귀 검증, 그래프 안전성 보강 |
| 2026-10-06 | Mac 저장·권한 검증 및 Tauri 2.12/Vite 6.4 보안 갱신; XML 예외 2개·RustSec 경고 12개 해소, RSA 예외 1개·Linux 경고 2개 잔여 |
| 2026-10-06 | Windows JSON 저장을 write-through 교체와 쓰기 가능한 백업 flush로 수정; Unix fsync·읽기 전용 교체 유지 및 저장 실패/재시도 회귀 보강 |
| 2026-10-06 | rustls 0.23.45/webpki 0.103.15 및 seroval 1.6.8/nanoid 3.3.20/source-map-js 1.2.2 보안 패치; 기존 예외·Windows 저장 문제와 Mac 검증은 별도 인계 |
| 2026-10-05 | P1-05 LTH 일봉 context·분봉 지표 분리, 일봉 성과 평가 차단과 분봉 표본/시간 미검증 표시; 실제 3개월 분봉 검증은 잔여 |
## 운영 문서 스택

작업 전 변경 성격에 맞춰 다음 루트 문서를 함께 확인한다.

- [`MASTER_PLAN.html`](MASTER_PLAN.html): 사람용 단계 계획과 금융 안전 게이트
- [`PROJECT_MAP.md`](PROJECT_MAP.md): 작업별 시작 파일과 핵심 경로
- [`ARCHITECTURE.md`](ARCHITECTURE.md): React/Tauri/Axum/주문·리스크·저장 경계
- [`HARNESS_MAP.md`](HARNESS_MAP.md): 변경 유형별 검증 명령
- [`tasks/TASK_TEMPLATE.md`](tasks/TASK_TEMPLATE.md): 범위·위험·완료 조건·검증 기록 템플릿

기존 상세 자료인 `docs/project-map.md`, `docs/agent-tooling.md`, `docs/ipc-commands.md`, `docs/coding-guide.md`는 계속 유효하다. Serena는 공용 타입, IPC, 주문, 리스크의 실제 참조 범위를 확인해야 할 때만 사용하고, Graphify는 호출 계층이나 공용 helper를 바꾸는 구조 리팩터링 때만 사용한다. 금융 안전 규칙과 provider 경계는 도구 사용 여부와 관계없이 항상 우선한다.
