# KISAutoTrade — Todo

> 완료 이력은 `git log`와 릴리스 노트에서 관리한다. 이 문서에는 아직 끝나지 않은, 검증 가능한 작업만 둔다.
> 우선순위는 `P1 정확성/신뢰성 → P2 전략 연구 UX → P3 유지보수` 순이다.

## *마지막 비판적 점검: 2026-07-11*

## P1 — PostgreSQL/MariaDB 운영 완성도

- [ ] 실제 PostgreSQL/MariaDB 지원 버전을 컨테이너 contract test로 검증한다.
  - [x] PostgreSQL: `storage/database_contract_tests.rs` — 미존재 DB 자동 생성, create/status/import/export/clear/drop, JSON↔DB backend 왕복(문서 read/write 포함), 잘못된 자격증명 거부, 확인 문구 gate를 실서버(`KISAT_PG_*` env)로 검증 (2026-07-12).
  - [ ] 잔여: MariaDB 동일 fixture 검증 (검증 서버 준비 후 — 사용자 결정으로 보류).
  - [ ] 잔여: TLS require(verify-full) 인증서 시나리오, 연결 단절·transaction rollback·재접속.
  - [ ] 잔여: CI service container에서 migration/schema version 호환성을 release gate로 둔다.

- [x] DB 자격증명을 OS keychain/credential vault로 이전한다 (2026-07-12).
  - password는 keyring(macOS Keychain/Windows Credential Manager)에 저장하고 `database_config.json`에서 제거, 기존 파일 password는 시작 시 1회 migration. keychain 불가 환경은 기존 파일(0o600) 저장 fallback.
  - 회귀 테스트: 저장 후 파일에 평문 없음, 재시작 시 keychain 복원, 레거시 파일 → keychain 이전 (mock keychain). IPC view는 기존처럼 `password_configured`만 노출, 설정 파일은 logical export 대상(data_dir) 밖에 위치.

- [ ] DB backend 전용 보관·복구 schema를 추가한다.
  - v1 JSON document 호환 계층 위에 broker/account scoped order, fill, position, risk runtime 정규화 테이블을 schema version migration으로 추가한다.
  - DB backend에서도 trade retention을 transaction으로 실행하고 Settings 보관 통계와 실제 row가 일치하게 한다.
  - 앱 재시작 시 미체결/부분체결과 리스크 runtime 상태를 DB에서 복원하는 contract test를 추가한다.

## P1 — 주문·리스크 정확성과 운영 가시성

- [ ] provider-neutral 주문 상태 머신을 완성한다.
  - `Pending`, `PartiallyFilled`, `Filled`, `Cancelled`, `Rejected`, `Expired`, `Failed`를 구분한다.
  - 취소·거부·만료·0체결 terminal 상태도 provider 조회로 확정하고 pending 차단을 해제한다.
  - 부분체결 누적수량/평균가와 정정·취소 경쟁 조건을 테스트한다.

- [ ] 리스크 설정과 일별 runtime 상태를 재시작 후 복원한다.
  - [x] 설정값(`risk/config.json`)과 일별 runtime(`risk/runtime.json`)을 분리 저장하고 시작 시 복원한다 (2026-07-12).
  - [x] 주문 접수·체결 반영·비상정지 변경 시점마다 runtime 스냅샷을 영속화하고, 저장 실패는 fail-closed로 신규 주문을 차단한다 (2026-07-12).
  - [x] 앱 재시작으로 손실 한도·주문 횟수·연속 손실 차단·비상정지를 우회할 수 없다 — 스냅샷 날짜가 오늘일 때만 카운터를 복원하고 비상정지는 날짜와 무관하게 유지 (unit test 4종).
  - [x] 손익 ledger 재구축을 실행 scope 체결로 격리한다 — `TradeRecord.matches_scope` 필터 (2026-07-12).
  - [ ] 잔여: 일일 손익 외의 주문 횟수·연속 손실 카운터도 broker 확인 주문/체결 ledger에서 재구축해 스냅샷과 교차 검증한다.
  - [ ] 잔여: 재시작 복원 시나리오(제출→강제 종료→재시작→차단 유지) contract test를 추가한다.

- [ ] 포지션과 주문/체결 기록을 `BrokerScope` 기준으로 일관되게 격리한다.
  - [x] 트래커는 단일 활성 scope 스냅샷만 보유: 프로파일 전환 시 `clear()` 후 다음 holdings `replace()`/수동 주문 직전 refresh로 재동기화, 실행 중 전환은 `execution_scope`로 분리 (2026-07-12).
  - [x] holdings snapshot은 replace/reconcile 방식으로 실제 계좌 상태를 반영한다 (기존 `replace()` 확인).
  - [x] `OrderRecord`/`TradeRecord`에 broker/account를 저장하고(`with_broker_scope`), 리스크 복원(`restore_risk_from_today_trades`)을 실행 scope 체결로 격리한다 (2026-07-12, `matches_scope` 테스트 4종).
  - [ ] 잔여: 여러 scope 포지션을 동시에 보관하려면 broker/account/market/symbol key로 전환한다 (현재는 단일 scope 불변식으로 충분).

- [ ] broker별 rate limit과 timeout을 process-wide 정책으로 통합한다.
  - Toss limiter/backoff를 짧게 생성되는 client마다 만들지 말고 credential/account scope별로 공유한다.
  - KIS/Toss 모든 요청에 connect/request/body timeout과 응답 크기 상한을 적용한다.
  - 429 횟수, 현재 pause, 마지막 성공 요청, 연속 실패를 운영 상태로 노출한다.

- [ ] 자동매매 건강 상태 패널을 추가한다.
  - 마지막 정상 holdings sync, 마지막 체결 reconciliation, 가장 오래된 pending, persistence 실패, daemon 연속 실패를 표시한다.
  - 신규 매수 중단과 전체 비상정지를 구분하고 사용자가 복구 조건을 확인할 수 있게 한다.
  - 오류는 로그에만 남기지 말고 Strategy/Dashboard와 Discord에 actionable message로 노출한다.

- [ ] Strategy 편집 상태를 활성 profile/broker scope와 함께 관리한다.
  - profile/account 전환 시 이전 scope의 `editMap`, 가격조건/레버리지 편집값, 세션 편집값을 폐기하거나 scope별로 보관한다.
  - 전략 조회·저장·토글 실패와 빈 상태를 카드 영역에 명확히 표시한다.
  - scope 전환 직후 이전 계좌의 미저장 값이 새 계좌 전략에 저장되지 않는 E2E 테스트를 둔다.

## P2 — 전략 시뮬레이션과 연구 워크플로

- [ ] 현재의 “신호 미리보기”와 실제 daemon 실행 모델의 차이를 제거한다.
  - 일봉 1개를 `on_tick()` 1회로 replay하는 전략과 실제 10초 tick/1분봉 전략의 시간축을 명시적으로 분리한다.
  - live와 preview가 같은 입력 정규화, warmup, session/장마감, 주문 수량·리스크 규칙을 재사용하게 한다.
  - look-ahead bias 없는 deterministic replay fixture로 live/preview signal parity를 검증한다.

- [ ] 카드 전체 너비를 활용하는 백테스트 결과 요약을 추가한다.
  - 기간·봉 주기·초기자본·수수료·세금·슬리피지·환율을 입력받는다.
  - 누적 수익률, MDD, 승률, 손익비, turnover, exposure, 거래 목록과 equity curve를 표시한다.
  - 단순 신호 개수와 실제 주문 가능/체결 가정 결과를 구분해 보여준다.

- [ ] 전략 세팅 비교와 재현 가능한 실험 저장을 지원한다.
  - 현재 편집값을 A/B preset으로 복제해 같은 데이터 구간에서 비교한다.
  - 전략 버전, 파라미터, 데이터 범위/source, 비용 가정, 생성 시각을 결과와 함께 저장한다.
  - in-sample/out-of-sample 또는 walk-forward 구간을 제공해 과최적화를 경고한다.

- [ ] 시뮬레이션 UX의 반응형·접근성 회귀를 계속 검증한다.
  - 모든 전략 카드가 데스크톱에서도 1열 전체 너비를 유지하는 Playwright 테스트를 유지한다.
  - 좁은 화면에서 파라미터 입력, 티커 Select, 미리보기 버튼, 차트가 겹치거나 잘리지 않는지 검증한다.
  - 편집값·티커·broker가 바뀌면 이전 미리보기 결과를 즉시 무효화한다.

## P3 — 유지보수·품질 게이트

- [ ] 1,000라인 초과 파일을 책임 단위로 분리한다.
  - `src-tauri/src/trading/strategy/leveraged_trend_hold.rs`: 계산/상태/preview/tests 분리.
  - `src-tauri/src/commands/trading.rs`: lifecycle/reconciliation/price-source/daemon cycle 분리.
  - `src/pages/trading/ui/Page.tsx`: broker별 orchestration과 공통 order form 분리.
  - `src/api/hooks.ts`: account/market/order/strategy/settings query 모듈 분리 후 public API 유지.
  - `src-tauri/src/commands/toss.rs`: diagnostic/preflight/orders surface 분리.
  - `src-tauri/src/trading/order.rs`: facade를 낮추고 남은 helper/state 책임 분리.
  - 검증: 변경 파일과 신규 파일은 1,000라인 아래, FSD/API public surface와 IPC 이름은 유지한다.

- [ ] 핵심 거래 흐름의 자동화 테스트를 release gate로 승격한다.
  - broker mock 기반 제출→부분체결→완전체결→취소/거부→재시작 복구 테스트를 추가한다.
  - balance fail-closed, scope 전환, midnight rollover, manual/auto parity, 인증 없는 REST 거부를 포함한다.
  - `cargo check`, `cargo test`, `npx tsc --noEmit`, `npm run check:fsd`, `npm run test:e2e`, OpenAPI 검증을 CI에서 실행한다.

- [ ] 의존성·릴리스 보안 점검을 자동화한다.
  - `cargo audit`와 npm audit 정책, Dependabot/Renovate, lockfile 검증을 CI에 추가한다.
  - release artifact 서명/해시와 updater 경로를 검증하고, 실패 시 사용자가 확인할 수 있게 한다.

- [ ] 사용자 가이드를 broker별 실제 지원 범위와 일치시킨다.
  - KIS 전용 소개를 KIS/Toss 공통 기능, broker별 주문·시세·자동매매 제한 표로 교체한다.
  - 실제 UI의 13개 전략 카드, 프로파일 설정, 세션/캔들 source, 소액 실거래 gate를 기준으로 오래된 절차를 정리한다.

## 완료 기준

- 각 항목은 코드, 실패/복구 테스트, 사용자-visible 상태, 관련 문서가 함께 반영되어야 닫는다.
- 실계좌 검증은 모의/fixture → read-only 계좌 조회 → 명시 승인된 소액 주문 순서를 지킨다.
- 경고를 포함해 `cargo check`와 TypeScript 검증이 깨끗해야 하며, 주요 UI 배치 변경은 Playwright로 확인한다.
