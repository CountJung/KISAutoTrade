# KISAutoTrade — Codex 에이전트 가이드

> 작업 전 이 파일을 읽고 시작한다.  
> 상세 정보는 아래 링크의 문서를 참조한다.

---

## 빠른 참조

| 항목 | 경로 |
|------|------|
| 디렉토리 맵 + 아키텍처 | `docs/project-map.md` |
| 에이전트 코드 탐색 도구 운영 | `docs/agent-tooling.md` |
| 개선 백로그 | `docs/todo.md` |
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

- 루트 문서는 `README.md`, `AGENTS.md`, `CLAUDE.md`를 유지하고 일반 운영·설계·할일 문서와 라이선스(`docs/LICENSE`)는 `docs/`, 작업 기록은 `docs/tasks/`에 둔다. 도구가 요구하는 `.github/`, `.codex/`, `.claude/`, `.serena/` 진입 설정·스킬 경로는 유지한다.
- `.env`, `secure_config.json`, `profiles.json`은 읽지 않는다.
- 코드 변경 전 현재 구현을 먼저 검색하고, 기존 패턴을 우선한다.
- 작업별 검증과 리뷰·문서 게이트가 완료되면 변경을 커밋하고 현재 브랜치를 origin에 푸시한다. 캐시·민감 파일은 제외하며 강제 푸시는 하지 않는다. (사용자 지시: 2026-10-03)
- 교차 모듈 심볼 변경은 `symbol_navigator`에게 Serena 참조 추적을 위임한다.
- 파일 추가·이동·삭제 또는 구조 변경은 `project_mapper`에게 프로젝트 맵 pass를 위임한다.
- 중복 helper 후보는 `helper_curator`에게 Graphify 후보 탐색과 Serena 참조 검증을 위임하고, 실제 동등성이 확인된 경우에만 공용 유틸로 승격한다.
- 주식 전략 연구는 `stock_analyst`에게 생산 replay 결과의 비용·손익·표본 해석을 감사하도록 위임한다. 실제 자료/fixture와 평가 불가를 구분하고 기존 TODO에 근거·완료 조건을 기록한다. 역할은 주문·자동매매 시작·민감 계정 접근을 맡지 않는다.
- KIS API 동작·TR-ID·제한사항은 추측하지 말고 공식 포털 또는 `koreainvestment/open-trading-api` 샘플로 확인한다.
- 새 IPC 커맨드는 Rust command, `lib.rs` 등록, TypeScript 타입/래퍼/훅, 문서를 함께 갱신한다.
- 반복 매매·손실 방지 관련 변경은 `docs/todo.md`와 관련 스킬 문서에 남긴다.
- Copilot 호환 지침과 `.github/skills/**`의 원본 스킬은 프로젝트 브리지 스킬(`.codex/skills/kisautotrade-*`)을 통해 재사용한다. Codex 런타임이 계정 스킬만 읽는 경우 `scripts/sync-codex-skills.ps1`로 동기화한다. 원본은 저장소의 `.github/skills/**/SKILL.md`로 유지한다.
- 코드 리뷰 또는 문서 업데이트 조건이 충족되면 `.github/codex-instructions.md`의 위임 게이트를 따른다. subagent 도구가 있으면 review/documentation pass를 위임하고, 도구가 없으면 같은 체크리스트를 직접 수행한 뒤 다음 턴용 위임 프롬프트를 최종 보고에 남긴다.

---

## 최근 변경 요약

> 전체 이력은 `git log --oneline`. 여기는 최근 5건만 유지.

| 날짜 | 한줄 요약 |
|------|----------|
| 2026-10-20 | 이격도 실제 소유·관측 보호와 선택 진입/청산 UI, 36연속/432분기·독립 정산 감사 및 저장 중 새 초안 보존 |
| 2026-10-20 | 루트 문서를 README·에이전트 진입으로 한정, LICENSE를 docs로 이동하고 링크·지도 점검 |
| 2026-10-20 | Momentum 실제 소유/미체결 교차·UI 검증, 비모멘텀1122회 호환·24연속/288분기와 독립 원장 감사로 부적합 신호 제거·옵션 기각 |
| 2026-10-18 | Consecutive 선택 진입 필터·완료 일봉 맥락 분리, 기본1,230회 호환과 단독24/분기288 비교·독립 회계 감사 완료 |
| 2026-10-18 | replay v12 연속청크/상태보존·bounded자료수집·모드/A-B, 3년72·분할72·분기초기화864회와 독립 원장/노출 대사 검증 |

## 운영 문서 스택

작업 전 변경 성격에 맞춰 다음 운영 문서를 함께 확인한다.

- [`docs/MASTER_PLAN.html`](docs/MASTER_PLAN.html): 사람용 단계 계획과 금융 안전 게이트
- [`docs/PROJECT_MAP.md`](docs/PROJECT_MAP.md): 작업별 시작 파일과 핵심 경로
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md): React/Tauri/Axum/주문·리스크·저장 경계
- [`docs/HARNESS_MAP.md`](docs/HARNESS_MAP.md): 변경 유형별 검증 명령
- [`docs/tasks/TASK_TEMPLATE.md`](docs/tasks/TASK_TEMPLATE.md): 범위·위험·완료 조건·검증 기록 템플릿

기존 상세 자료인 `docs/project-map.md`, `docs/agent-tooling.md`, `docs/ipc-commands.md`, `docs/coding-guide.md`는 계속 유효하다. Serena는 공용 타입, IPC, 주문, 리스크의 실제 참조 범위를 확인해야 할 때만 사용하고, Graphify는 호출 계층이나 공용 helper를 바꾸는 구조 리팩터링 때만 사용한다. 금융 안전 규칙과 provider 경계는 도구 사용 여부와 관계없이 항상 우선한다.
