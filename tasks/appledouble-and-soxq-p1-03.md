# 작업 기록: AppleDouble 점검 및 SOXQ-P1-03

## 1. 메타데이터

- 작업 ID: SOXQ-P1-03 / 내부 드라이브 이전 후 부산물 점검
- 담당자: Codex 및 저장소 지정 영향 추적·리뷰·문서 에이전트
- 상태: Done
- 작성/갱신일: 2026-10-03
- 대상 브랜치: main (커밋/PR 생성 없음)
- 관련 문서: [검증 보고서](../docs/research/soxq-3m-20261003/p1-03-validation.md), [TODO](../todo.md)
- 위험 등급: 보통 — replay 공용 훅, 실제 주문 실행 없음

## 2. 목표와 비목표

목표: 등록 프로젝트의 `._*` AppleDouble 부산물을 점검하고, 날짜/완료봉 이벤트로 강한 종가·변동성 확장 일봉 replay를 수정한다.

비목표: 일반 `_` 접두 파일 삭제, 전체 드라이브 정리, live provider 완료봉 공급, next-open/장중 체결, LTH 시각 모델, 전략 파라미터 최적화.

사용자 완료 조건:

- [x] 안전한 AppleDouble 후보 점검과 결과 기록.
- [x] P1-03 날짜별 조건과 보유 상태·실행 피드백 구현/검증.
- [x] 같은 SOXQ 입력 재평가와 한계·준비 상태 기록.
- [x] 독립 문서 pass와 최종 map/graph 게이트.

## 3. 시작 상태

```text
branch: main
git status: 기존 P1-02/P1-01/캐시 정리/연구 문서/UI/포맷/그래프 변경이 미커밋 상태.
보존 대상: 기존 untracked .cargo 설정, 연구/fixture 파일, task 기록 및 수정 파일.
```

- [x] 기존 작업을 보존하고 P1-03 변경을 이어서 적용했다.
- [x] .env, secure_config.json, profiles.json을 읽지 않았다.

## 4. 조사 근거

| 파일/심볼 | 확인 사실 | 적용 |
|---|---|---|
| 등록 프로젝트 목록 | MacWorking, CodexAuto 두 로컬 작업 루트 | symlink 비추적 read-only 점검 |
| StrongCloseStrategy | 이전 warmup 마지막 날 조건만 준비 | 매 완료일 다음 평가일 조건 준비 |
| VolatilityExpansionStrategy | 날짜 전환 없이 범위 누적 | 당일 OHLC와 이전 N일 range ring 분리 |
| Strategy/preview/manager | live은 현재가 tick, generic만 과거 일봉 공급 | 공용 기본 훅 추가, live tick 경계 유지 |
| generic 실행 피드백 | raw 신호 내부 상태와 실제 체결 상태가 다름 | sync_position 후 완료봉 적재 |

AppleDouble 조사 루트:

- `/Users/jsjmac/Workspace/MacWorking`
- `/Users/jsjmac/Workspace/CodexAuto`

ignored cache를 포함해 두 루트를 재귀 점검했다. symlink 디렉터리는 따라가지 않으며, 일반 `._*` 후보만 AppleDouble magic/version 검증 대상으로 정했다. **후보 0건, 제거 0건, 오류 0건**이다. 일반 `_` 접두 소스는 제거하지 않았다. 점검 목록은 ignored `.cache/appledouble-cleanup/inventory.json`에 기록했다. `.gitignore`의 `._*`/`.DS_Store` 규칙은 이미 있다.

공식 KIS/Toss 근거: N/A — 새로운 provider API 동작 변경 없음. 기존 동결 자료를 오프라인 실행했다.

가정: 일봉 시가/종가 공개 시점과 종가 체결 근사를 사용한다. 실제 장중 경로/익일 시가를 복원하지 않는다.

## 5. 영향 범위와 계약

| 경로 | 변경 |
|---|---|
| src-tauri/src/trading/strategy/core.rs | 기본 날짜 시작/종가/완료 훅, 0범위 보존 |
| src-tauri/src/trading/strategy/breakout.rs | 전일 조건 및 이전 완료 range ring |
| src-tauri/src/commands/strategy_preview/generic.rs | 일봉 이벤트 순서, 비일봉 입력 거부 |
| src-tauri/src/trading/simulation.rs | 엔진 v5 |
| 새 daily_event_tests.rs, breakout/day_event_tests.rs | 13개 날짜/피드백/준비 회귀 |
| strategyPreviewPanel.tsx, strategyResearchTypes.ts, E2E | 두 전략 일봉 제한과 모델 안내/cadence 타입 |
| 연구 결과·TODO·스킬·IPC·architecture·project-map | 결과/경계/구조 문서 |

계약 체크:

- [x] Frontend public API/FSD: 기존 shared 공개 API 경유, 검사 통과.
- [x] Rust serde ↔ TypeScript: cadence 문자열 값 갱신, 기존 입력/preparation 필드 유지.
- [x] Tauri registration: N/A — 새 command 없음.
- [x] Web REST/transport: 기존 preview 경로, 새 endpoint 없음.
- [x] cache invalidation: 기존 입력 변경 무효화 유지, 일봉 전환 UI 검사.
- [x] persisted schema: N/A — 영속 전략/주문 schema 변경 없음.
- [x] IPC/architecture 문서 갱신; project-map 최종 pass 완료.

## 6. 금융·자동매매 안전 검토

실제 주문 가능성: 없음 — 오프라인 replay와 mock UI 검증.

- [x] 실전·모의 주문과 자동매매 시작을 실행하지 않았다.
- [x] BrokerScope: N/A — live 요청/주문 경로 변경 없음.
- [x] provider 전 가격/수량/동의/리스크: N/A — provider 주문 호출 없음.
- [x] 중복/반대 방향/동시 submission: 기존 guard 보존, 차단 결과 되먹임 검증.
- [x] timeout/rate limit/불명 응답: N/A — provider transport 변경 없음.
- [x] 영속화 실패 시 신규 주문 차단: N/A — live 영속화 변경 없음.
- [x] 비상정지/일일 손실 재시작 우회: N/A — 해당 경계 변경 없음.
- [x] credential/계좌 원문 기록 없음.
- [x] UI/검증 문서에 종가 근사와 잔여 체결 모델 한계 표시.

## 7. 구현과 롤백

시가-only 날짜 시작 → 완성 OHLC 종가 조건 → 실제 실행 포지션 피드백 → 완료봉 적재. 날짜 전환에 reset하지 않고 이전 N일 평균에 당일 범위를 미리 섞지 않는다. 미래 봉 변경·보유 손절·차단 매수·flat range를 테스트했다.

롤백: 이 작업의 event hooks/호출/일봉 guard/UI 설명만 대응 제거하고 엔진 버전을 되돌린다. 기존 P1-01/P1-02 사용자 변경이나 연구 기록을 일괄 reset하지 않는다. 실제 운영 변경은 수행하지 않았다.

## 8. 도구 선택

- Serena 사용: 지정 symbol_navigator가 Strategy 공용 훅과 generic/live manager caller를 추적했다. 공용 타입 변경이 live tick을 바꾸지 않는지 확인했다.
- Graphify 사용: 공용 event hook 호출 경계와 새 회귀 파일 추가에 따른 구조 갱신 및 freshness 검사.
- 독립 review: p103_review가 변경 코드/fixture를 검토해 미수정 결함 없음 보고.
- 독립 documentation/project-map: p103_docs_map이 저장소 위임 게이트에 따라 완료했다. UI 스킬 갱신 시각과 평균 범위 설명의 모호함을 수정했다.

## 9. 검증 결과

| 상태 | 명령/검사 | 결과 |
|---|---|---|
| PASS | AppleDouble 두 등록 작업 루트 | 후보/제거/오류 모두 0 |
| PASS | cargo fmt --all --check | 포맷 일치 |
| PASS | cargo check --workspace --all-targets --locked | 경고 0 |
| PASS | cargo clippy --workspace --all-targets --all-features --locked | 경고 0 |
| PASS | cargo test --workspace --lib --locked | 180 tests |
| PASS | npx tsc --noEmit / npm run check:fsd | 타입/FSD 통과 |
| PASS | npm run check:lockfiles / npm run build:web | lockfile/production build 통과 |
| PASS | npx playwright test tests/e2e/strategy-scrollbar.spec.ts | 22 tests |
| PASS | 같은 입력 2회 및 각20bp 재평가 | 지원 24실행 일치, 다른 전략 20실행 v4 금융/준비 결과 유지 |
| PASS | npm run check:project-map | 새 인벤토리/책임/데이터 흐름 반영, 경고 0 |
| PASS | npm run graphify:refresh / check:graphify | graph source snapshot 최신 |
| PASS | git diff --check | 최종 문서 변경 포함 공백 오류 없음 |

코드 경고는 0이다. Playwright Node 환경의 `NO_COLOR`/`FORCE_COLOR` 충돌 진단 2건은 도구 환경에서 발생했다. Graphify 갱신에서는 설정 JSON 4개(extensions/launch/tasks/secure_config.example)에 심볼이 없다는 추출 진단 1줄이 나왔다. Graphify freshness 검사는 통과했다. 이 파일들의 민감 설정값을 읽은 작업은 아니다.

실전 주문/모의 주문: NOT RUN. LTH deterministic/장중 성능은 미검증 범위(P1-04/05)로 유지한다.

## 10. 완료 보고

이전 사용자 변경을 모두 보존했다. AppleDouble 삭제 대상은 없었으며 daily replay의 날짜/완료/체결 피드백을 수정했다. SOXQ 강한 종가 +1.7179%(완료 2회 모두 손실), 변동성 +0.7807%(완료 0회)는 미청산 10주를 포함한 엔진 진단이다.

변경 파일은 위 범위 표의 저장소 루트 `/Users/jsjmac/Workspace/MacWorking/KISAutoTrade` 하위다. next-open(P2-14), true-range ATR(P2-16), provider paging(P2-17), LTH(P1-04/05)는 남아 있다.

운영자 확인:

- [x] N/A — 운영 배포/자동매매 설정 변경을 수행하지 않았다.
- [x] 실전 전 모의·소액 검증은 별도 승인 범위로 유지한다.
- [x] 롤백 경계와 로그 기록은 위에 명시했다.
