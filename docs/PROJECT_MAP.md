# KISAutoTrade 탐색 지도

> 목적: 작업 유형에서 실제 수정 지점으로 빠르게 이동하는 수동 큐레이션 지도다. 전체 파일 트리와 장문의 모듈 책임표는 자동 생성 기준인 [`docs/project-map.md`](project-map.md)를 사용한다.

## 1. 런타임 한눈에 보기

```text
React 18 + TypeScript + MUI
  pages/features/widgets/entities/shared
       ↓ TanStack Query hooks
  src/api/commands.ts
       ↓ src/shared/api/transport.ts (Tauri invoke 또는 Web REST)
Tauri v2 command / Axum route
       ↓ AppState + broker adapter
strategy → guard/preflight/risk → OrderManager → KIS/Toss
       ↓
JSON/DB storage + pending/order/trade/risk + Discord/event
```

상세 계층과 주문 시퀀스는 [`docs/ARCHITECTURE.md`](ARCHITECTURE.md)를 본다.

## 2. 작업별 첫 진입점

| 작업 | 먼저 읽을 곳 | 같이 확인할 곳 |
|---|---|---|
| 앱 라우트/화면 | `src/router/index.ts`, `src/pages/*/ui/Page.tsx` | `src/widgets/`, `src/features/`, `src/shared/ui/` |
| 전역 UI 상태 | `src/entities/*/model/*Store.ts` | 하위 호환 `src/store/*.ts` re-export |
| 서버 상태 조회/변경 | `src/api/hooks.ts`, `src/api/queryKeys.ts` | `src/api/commands.ts`, `src/shared/api/transport.ts` |
| 공용 IPC 타입 | `src/shared/api/types.ts` | `src/api/types.ts` re-export/legacy surface, Rust serde view |
| DB IPC | `src/api/databaseHooks.ts` | `src/shared/api/databaseCommands.ts`, `commands/database.rs` |
| Tauri command 추가 | `src-tauri/src/commands/<domain>.rs` | `commands.rs`, `lib.rs::generate_handler!`, `docs/ipc-commands.md` |
| Web REST 대응 | `src-tauri/src/server/mod.rs` | `server/{market,profiles,records,toss,trading}.rs`, transport route map |
| KIS 연동 | `src-tauri/src/api/rest.rs`, `api/rest/*` | `api/token.rs`, `api/websocket.rs`, `broker/kis.rs` |
| Toss 연동 | `src-tauri/src/broker/toss/` | `commands/toss*.rs`, `server/toss.rs` |
| broker 공통 계약 | `src-tauri/src/broker/domain.rs`, `adapter.rs` | `kis.rs`, `toss/adapter.rs`, `rate_limit.rs` |
| 캔들 페이지/일반 preview 사전자료 | `src-tauri/src/commands/candles.rs`, `broker/candles.rs` | `api/rest/chart.rs`, `src/shared/api/candlePageTypes.ts`, `pages/strategy/model/candlePageCollector.ts`, `server/market.rs` |
| 자동매매 시작/데몬 | `src-tauri/src/commands/trading.rs` / `commands/trading/daemon.rs` | `daemon/{fence,flow}.rs`, `commands/trading/{positions,balance,risk_restore,history,daily_history}.rs`, `lib.rs` |
| Live 52주 완료 일봉 준비 | `commands/trading/daily_history.rs`, `daily_history/jobs.rs` | `trading/strategy/manager/live_history.rs`, `src/shared/ui/LiveDailyHistoryPanel.tsx`, TradingStatus |
| 전략 | `src-tauri/src/trading/strategy/` | `commands/strategy.rs`, `commands/strategy_preview.rs`, Strategy UI |
| 전략 설정 저장·보유 보호 | `src-tauri/src/trading/configuration.rs` | `trading/order/configuration.rs`, `trading/strategy/manager/configuration.rs`, 각 전략의 `reconfigure_tests.rs`, IPC/Web 전략 수정 facade |
| 잔고 조회·tracker 게시·계좌별 캐시 | `src-tauri/src/trading/order/holdings.rs`, `holdings/{fetch,parse}.rs` | `trading/position.rs`, `commands/accounts.rs`, `commands/trading/positions.rs`, `server/{market,trading}.rs`, `src/api/{accountHooks,backendEvents}.ts` |
| 수동·자동 주문 | `src-tauri/src/trading/order/submission.rs` | `order/execution_run.rs`, `submission/{positions,prepare,provider,records}.rs`, `commands/orders.rs` |
| 실행 변경·지연 결과 폐기 | `commands/trading/daemon/fence.rs`, `trading/order/execution_run.rs` | IPC/Web start/stop·전략 수정·프로파일 적용, `submission/run_tests.rs` |
| 주문 전 검증 | `src-tauri/src/trading/preflight.rs`, `trading/guard.rs` | broker별 preflight command, `trading/risk.rs` |
| 리스크 | `src-tauri/src/trading/risk.rs` | `commands/risk.rs`, `storage/risk_store.rs`, restore tests |
| pending/체결 | `trading/order/fills.rs`, `order/fills/reconciliation.rs` | `order/{pending,health}.rs`, `fills/{collector,provider_mode}.rs`, `storage/{pending_order_store,order_store,trade_store}.rs` |
| 전략 소유 수량·체결 피드백 | `trading/order/{ownership,feedback}.rs` | `ownership/runtime.rs`, `submission/owned.rs`, `fills/ownership.rs`, `trading/strategy/{core,manager}.rs`, 각 `live_feedback_tests.rs` |
| 체결 손익·재시도 영수증 | `trading/order/fills/ownership.rs`, `trading/risk/fill_accounting.rs` | `fills/collector/notional.rs`, `commands/trading/risk_restore.rs`, `storage/pending_order_store.rs`, `order/ownership/` |
| Toss 소액 검증·수수료율 표시 | `commands/toss/small_order.rs`, `trading/preflight.rs` | `small_order/tests.rs`, `src/pages/trading/ui/tossPanels.tsx`, `tests/e2e/toss-commission-ratio.spec.ts` |
| 저장 backend | `src-tauri/src/storage/mod.rs`, `database.rs` | 각 `*_store.rs`, DB contract tests |
| 데몬/앱 초기화 | `src-tauri/src/lib.rs` | `commands.rs::AppState`, `server/mod.rs` |
| 릴리스/CI | `package.json`, root `Cargo.toml` | `.github/workflows/*.yml`, `docs/release-security.md` |

## 3. Frontend 경계

- `pages`: route 조립과 화면별 UI.
- `widgets`: 앱 셸·사이드바·차트처럼 페이지를 조합하는 큰 블록.
- `features`: 사용자 행동 단위 공개 진입점.
- `entities`: account/settings/trading 도메인 상태.
- `shared`: 재사용 API 계약, transport, 설정, UI, 순수 helper.
- `src/api/*`: 기존 import를 유지하는 애플리케이션 API/hook 계층. 새 중복 구현을 만들지 말고 `shared/api` 공개 surface와 관계를 먼저 확인한다.

현재 route는 dashboard(`/`), trading, strategy, history, log, settings다. import 방향은 `npm run check:fsd`가 검사한다.

## 4. Backend 경계

- `commands/`: Tauri IPC 입력 검증과 facade. 도메인 로직을 command에 복제하지 않는다.
- `server/`: 웹 REST facade. 민감 DB 관리 IPC는 인증 없는 REST에 노출하지 않는다.
- `broker/`와 `api/`: provider 계약·HTTP/WebSocket·rate limit.
- `trading/`: 전략, 주문, 포지션, preflight, guard, risk, simulation의 핵심 도메인.
- `storage/`: JSON/DB 공통 저장 경계와 projection. store에서 backend를 우회한 직접 파일 저장을 추가하지 않는다.
- `lib.rs`: AppState 조립, 6개 백그라운드 작업, command 등록.

## 5. 중요한 동기화 묶음

### IPC 계약 변경

```text
Rust input/view + serde
→ command facade
→ lib.rs generate_handler!
→ TS shared type
→ command wrapper / transport REST map
→ query hook/key/invalidation
→ UI
→ docs/ipc-commands.md
```

### 주문·리스크 변경

```text
BrokerScope + profile/account
→ 실행 epoch + 설정 revision + 고정 profile/client (자동) / 요청 snapshot (수동)
→ strategy/manual request + 지연 조회 후 현재 context 검사
→ provider preflight
→ RiskManager + TradeGuard
→ 같은 OrderManager 잠금에서 실행 context 재검증 + reservation/conflict
→ 고정 provider adapter/client (재시도 포함)
→ pending/order persistence
→ scope/profile/인증 + 저장 주문의 투자 모드 출처 재검증 후 fill reconciliation
→ 정확한 소유 key의 누적 체결·원가·손익 원장 / 체결 이벤트 영수증
→ trade/stats/risk persistence
→ manager → OrderManager 아래 소유 전략 피드백 / 계좌 전체 잔고 재조회
```

시작·정지와 daemon의 전략 변경은 running → StrategyManager → OrderManager 순서로
직렬화한다. 전략 설정은 공통 `trading/configuration.rs`가 저장 작업을 소유하며,
보유 대상·처리 중 신호·예약 주문을 검증하고 설정 편집 예약을 잡은 뒤 잠금 밖에서
저장한다. 저장 성공 후 manager → OrderManager에서 계좌 identity를 재검증하고
같은 ID/scope 전략의 보유 상태를 유지하며 새 revision을 게시한다. 저장 실패는
기존 인스턴스와 revision을 유지하고, 첫 저장 오류도 교체 후 오류일 수 있어
저장 후 identity 검증 실패와 함께 원본 저장값을 복구한다. 프로파일 적용은 같은 잠금 순서로 scope·client·tracker를 교체하고
전략 인스턴스를 새로 만들어 이전 프로파일의 내부 보유 상태를 넘기지 않는다. provider
조회는 잠금 밖에서 수행한다. 정지 전 예약한 주문·pending·예산은 원래 scope로
계속 추적하며, 이전 실행의 quote·신호·전략 상태 복원은 새 실행에 반영하지 않는다.
일반 조회·백그라운드 갱신·시작 복원·수동 주문의 holdings 게시는 공통
`order/holdings.rs`에서 계좌·profile·client·epoch와 설정 편집 예약을 재검증한다.
`position.rs`의 조회/변경 UUID로 최신 조회와 체결 뒤 상태를 보호하고, KR/US
동시 게시는 두 시장 검증 후 함께 교체한다. 읽기 전용 IPC/Web holdings 응답도
identity를 검사한다. 프론트 잔고 캐시는 profile/broker/account/모드로 나누고,
잔고 이벤트는 현재 계좌의 재조회를 유도한다. 체결 조회 결과 적용도 계좌·profile·client 변경을 재검증한다.
KIS pending의 저장 TR-ID가 현재 실전/모의 모드와 다르거나 출처를 확인할 수
없으면 조회·직접 체결·체결 조회의 종료 상태와 예산 반영을 보류하고 체결 동기화
health에 남긴다.

계좌 전체 tracker는 증권사 잔고의 관측값이며 체결 delta를 다시 더하지 않는다.
`ownership.rs`는 profile·투자 모드·broker/account·strategy·market·symbol별로
확인된 소유 수량과 원가를 기록한다. 시작 복원과 daemon 피드백은 정확한 전략에
소유 원장의 수량만 전달하고 다른 전략·수동 보유를 모든 전략에 broadcast하지
않는다. LTH 최고가·보유 관측 수는 별도 metadata로 보존한다. 원장 세대·누적
영수증·저장 검증 실패는 주문 차단 경계로 전달한다.

`holdings/parse.rs`는 일반 갱신·시작·수동 주문에서 같은 잔고 검증을 적용한다.
Toss 응답의 broker/account·시장/통화·중복 종목과 수치 범위를 확인하고 소수 잔량을
정수 주문 수량으로 올리지 않는다. 리스크 복원은 계좌와 투자 모드를 검증한 후보를
저장한 뒤 공개한다. 구현·검증 범위는 [P1-FILL 작업 기록](tasks/strategy-fill-ownership-20261018.md)을 따른다.

`fills/collector/notional.rs`는 provider 누적 체결금액을 원·cents의 정수로 변환한다.
Toss의 명시 금액과 누락을 구분하고 실제 평균가 대체 계산도 총액 변환 뒤 한 번만
반올림한다. 소액 검증의 즉시 체결은 원 주문 식별자·종목·매수 방향·통화·1주와
명시 체결대금이 일치해야 하며, 미확정 자료를 체결로 만들어내지 않는다.

공용 타입·IPC·주문·리스크의 다중 참조 추적에는 필요할 때 Serena를 선택한다. Graphify는 위 구조 자체를 이동·통합하는 리팩터링 때만 사용한다.

## 6. 데이터와 민감 영역

| 데이터 | 코드상 위치/경계 |
|---|---|
| 프로파일 | Tauri app data의 `profiles.json`; `lib.rs`/`config` 관리 |
| 거래 데이터 | 실행 CWD의 `data/`; `storage::*Store`와 JSON/DB document 경계 |
| 로그 | 실행 CWD의 `logs/`; `logging/` |
| DB 설정 | app data의 `database_config.json`; password는 IPC view에서 반환하지 않음 |
| DB export | app data의 `database_exports/`; manifest 포함 |

실제 `.env`, `secure_config.json`, `profiles.json`은 탐색 대상이 아니다. 위치·형식의 상세 기준은 `docs/project-map.md`와 `docs/coding-guide.md`를 따른다.

## 7. 지도 유지 원칙

- 이 문서는 작업 라우팅이 달라질 때만 갱신한다.
- 전체 파일 추가·이동·삭제는 `npm run project-map:update`로 `docs/project-map.md` 생성 블록을 갱신하고 `npm run check:project-map`으로 확인한다.
- IPC 이름은 이 문서에 복제하지 않고 `docs/ipc-commands.md`에만 유지한다.
- 검증 명령은 이 문서에 복제하지 않고 [`docs/HARNESS_MAP.md`](HARNESS_MAP.md)에 유지한다.
