# 작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단

## 1. 메타데이터

- 작업 ID: SOXQ-P1-04
- 담당자: Codex, 지정 영향 추적·독립 리뷰·프로젝트 맵 에이전트
- 상태: Done
- 날짜: 2026-10-03
- 브랜치: main
- 위험 등급: 보통 — 과거 실행 계약 변경, 실제 주문 없음
- 관련 문서: [검증](../docs/research/soxq-3m-20261003/p1-04-validation.md), [TODO](../todo.md)

## 2. 목표와 비목표

목표: LTH 과거 실행의 시스템 시각 의존을 제거한다. 완료 조건의 대안 중 generic 거부를 선택하고 기존 timed 전용 경로를 검증한다.

비목표: live clock/주문 변경, 제공봉을 provider 조회로 자동 전환, 실제 분봉 확보, 시장 세션/DST·지표 버퍼 모델(P1-05), next-open 체결.

사용자 완료 조건:

- [x] 검증 완료 후 커밋·푸시하는 저장소 규칙을 AGENTS.md에 기록.
- [x] 이전 P1-01~03/연구/캐시 작업을 65e465e로 커밋하고 origin/main 푸시 완료.
- [x] generic LTH clock 경로 차단과 명시 오류, production timed 회귀.
- [x] 서로 다른 시스템 분/시간대의 재현과 동일 입력 영향 비교.
- [x] 최종 코드/문서/map/graph 게이트 완료. 검증된 변경을 커밋·푸시하며 실제 해시는 최종 응답에 기록한다.

## 3. 시작 상태

```text
branch: main
baseline: 65e465e (origin/main에도 푸시됨)
기존 P1-01~03 변경과 동결 연구·캐시 정책을 baseline에 보존.
```

- [x] 기존 작업을 보존했다.
- [x] .env / secure_config.json / profiles.json을 읽지 않았다.

## 4. 조사 근거

| 심볼/파일 | 확인 사실 | 적용 |
|---|---|---|
| LTH live clock helpers | 현재 분 OHLC·세션·blackout에 Local::now 사용 | generic factory 전에 거부 |
| build_strategy | starts_with leveraged_trend_hold 라우팅 | 접미 ID도 같은 차단 기준 |
| preview_signals_with_execution | timed.time 기준 세션/blackout/청산 | 기존 전용 경로 유지 |
| cfg(test) clock helpers | live 분/세션의 테스트 대역 존재 | production-library integration으로 검증 |
| Page/REST preview | UI는 LTH 전용 분기, REST는 같은 generic 함수 | 화면·transport 계약 유지 |

공식 provider 근거: N/A — provider API 동작 변경/실제 조회 없음. 합성 일봉 및 고정 시간 모델의 시장 정확성은 P1-05로 유지한다.

## 5. 변경과 계약

| 경로 | 변경 |
|---|---|
| commands/strategy_preview/generic.rs | UNSUPPORTED_GENERIC_REPLAY, 지원 결과 deterministic=true |
| strategy_preview/preparation_tests.rs | 12 지원 전략, 42 기본/접미·간격/history 거부 및 malformed fixture |
| src-tauri/tests/lth_replay_clock.rs | production library 입력 시각/실행/future 회귀 3개 |
| trading/simulation.rs | engine v6 |
| 연구 runner/results/validation | 24 정상 + 2 구조화 거부, unexpected error fail |
| E2E fixture·TODO·IPC·architecture·Rust skill·map | 버전/지원 경계/검증 동기화 |

- [x] Frontend public API/FSD: import 변경 없음, 타입/FSD 검증.
- [x] Rust serde/TS: 성공 응답 schema 유지, CmdError 기존 code/message 사용.
- [x] Tauri command registration: N/A — 새 커맨드 없음.
- [x] Web REST: 같은 generic 함수에서 거부, 기존 오류 transport 유지.
- [x] cache/query invalidation: 기존 generation/input 처리 보존, engine/기본 version/hash v6.
- [x] persisted JSON/DB schema: N/A — 영속화 변경 없음.
- [x] IPC/스킬/프로젝트 맵 독립 문서 pass 완료, 남은 계약 drift 없음.

## 6. 금융·자동매매 안전

주문 가능성: 없음 — 오프라인/합성 fixture 및 mock UI 검사.

- [x] 실제·모의 주문/자동매매 시작 실행 없음.
- [x] scope/수량·가격·동의/provider preflight: N/A — live/provider 호출 변경 없음.
- [x] 동시 submission/중복 guard: N/A — 실행 경계 변경 없음.
- [x] timeout/rate limit/부분 응답 fail-closed: N/A — transport 변경 없음.
- [x] pending·risk 영속화 실패 차단: N/A — storage 변경 없음.
- [x] 비상정지·일일 손실·재시작: N/A — live state 변경 없음.
- [x] credential/계좌 원문 기록 없음.
- [x] 미지원 generic LTH를 정상 무신호/0% 성과로 제시하지 않는다.
- [x] 전용 일봉 합성/세션 정확성 한계를 유지한다.

## 7. 구현·롤백

Generic 진입부의 ID 차단 → 지원 전략 기존 평가 → engine v6. LTH는 timed 전용 경로를 명시 안내하며 provider 조회로 자동 우회하지 않는다.

롤백: 이 작업의 guard/version/test/doc만 대응 변경한다. 현재시각 generic LTH가 되살아나므로 단순 롤백으로 정상 재현성을 주장하지 않는다. 이전 P1-01~03 baseline을 일괄 reset하지 않는다.

## 8. 도구·위임

- Serena: symbol_navigator가 factory/preview/live clock 및 UI/REST 참조를 추적.
- 독립 p103_review: 기능 결함 없음. IPC/architecture/TODO와 E2E mock 버전 drift를 수정했다.
- Graphify: 호출 경계와 integration 구조의 source freshness 갱신/검사.
- Project mapper: 새 integration·연구·task 인벤토리 및 지원 경계 문서 pass.

## 9. 검증

| 상태 | 검사 | 결과 |
|---|---|---|
| PASS | cargo test --workspace --lib --locked | 181 tests |
| PASS | cargo test --locked --test lth_replay_clock | production library 3 tests |
| PASS | cargo test --workspace --locked | lib 181 + integration 3 + doc 3 = 187 PASS |
| PASS | cargo check --workspace --all-targets --locked | 경고 0 |
| PASS | all-targets all-features Clippy / rustfmt | 최종 코드 경고 0, 포맷 일치 |
| PASS | npx tsc --noEmit | 타입 오류 없음 |
| PASS | FSD/lockfile/web build | 구조/lockfile/production 빌드 통과 |
| PASS | focused Playwright | 22 tests, 전용 LTH UI 유지 |
| PASS | SOXQ 전체 JSON 반복 | generatedAt 제외 전부 일치, UTC 분과 시간대 다름 |
| PASS | v5 대비 지원 24실행 | candles·신호·준비·backtest 동일 |
| PASS | project-map / graphify / diff check | 최신 source snapshot/인벤토리/공백 검사 |

실전·모의 주문: NOT RUN. 한계를 확대 해석하지 않는다. 테스트 개발 중 미사용 import/JSON 배열 slice 컴파일 오류 및 err-expect Clippy 경고를 수정했다. 최종 Rust check/Clippy/build 경고는 0이다. Playwright는 기존 Node NO_COLOR/FORCE_COLOR 환경 충돌 진단 2건, Graphify는 설정 JSON 4개의 no-symbol 추출 진단을 출력했다. 각각 테스트/freshness 게이트는 통과했으며 코드 경고와 구분한다.

## 10. 완료 보고

변경 파일은 `/Users/jsjmac/Workspace/MacWorking/KISAutoTrade` 하위의 위 표 경로다. 이전 작업은 65e465e에 보존했다. LTH 일반 과거 실행을 명시 거부하고 입력 시각 전용 실행의 비어 있지 않은 신호·체결 회귀와 시장 자료 진단을 구분했다. P1-05, next-open, true range/provider paging은 남아 있다.

운영자 확인:

- [x] N/A — 운영 배포/자동매매 설정 변경 없음.
- [x] 실전/소액 검증은 별도 명시 승인 범위 유지.
- [x] 롤백/기록 경계는 위에 명시했다.
