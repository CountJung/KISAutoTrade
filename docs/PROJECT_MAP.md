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
| 수동 주문 화면 | `src/pages/trading/ui/Page.tsx`, `useTradingOrderModel.ts` | `tradingOrderForm.tsx`, `kisPanels.tsx`, `tossPanels.tsx`, `tests/e2e/trading-form-contract.spec.ts`. Page는 조립, 모델은 상태·조회·제출, 폼은 공유 모델 UI를 소유 |
| LTH 편집·종목 선택·preview | `src/pages/strategy/ui/leveragedTrendHoldEditorPanel.tsx`, `useLeveragedTrendHoldEditorModel.ts` | `leveragedTrendHoldTickerPicker.tsx`, `leveragedTrendHoldPreviewPanel.tsx`, `leveragedTrendHoldPreviewChart.tsx`, `strategyResearchPanel.tsx`. 기존 공개 entry·검증 export와 계좌/입력 generation·평가불가 경계를 보존 |
| 전역 UI 상태 | `src/entities/*/model/*Store.ts` | 하위 호환 `src/store/*.ts` re-export |
| 서버 상태 조회/변경 | `src/api/hooks.ts`, `src/api/queryKeys.ts` | `src/api/commands.ts`, `src/shared/api/transport.ts` |
| 공용 IPC 타입 | `src/shared/api/types.ts` | `src/api/types.ts` re-export/legacy surface, Rust serde view |
| DB IPC | `src/api/databaseHooks.ts` | `src/shared/api/databaseCommands.ts`, `commands/database.rs` |
| Tauri command 추가 | `src-tauri/src/commands/<domain>.rs` | `commands.rs`, `lib.rs::generate_handler!`, `docs/ipc-commands.md` |
| Web REST 대응 | `src-tauri/src/server/mod.rs` | `server/{market,profiles,records,toss,trading}.rs`, transport route map |
| 공통 HTTP·TLS 정책 | `src-tauri/src/http.rs`, `http/tests.rs` | `http/testdata/*.der`, `api/{rest,token}.rs`, `broker/toss/http.rs`, `commands/settings.rs`, `market/mod.rs`, `notifications/discord.rs`, `server/profiles.rs`. reqwest 생성14곳의 번들 roots·client별 ring/HTTP1을 공유하고 호출자 정책과 WS0.24/native roots 경계는 유지 |
| KIS 연동 | `src-tauri/src/api/rest.rs`, `api/rest/*` | `api/token.rs`, `api/websocket.rs`, `broker/kis.rs` |
| Toss 연동 | `src-tauri/src/broker/toss/` | `commands/toss.rs` facade → `toss/{accounts,diagnostics,preflight,orders,small_order}.rs`, `commands/toss_market.rs`, `server/toss.rs`; 기존 공개 경로·IPC/REST 계약 유지 |
| broker 공통 계약 | `src-tauri/src/broker/domain.rs`, `adapter.rs` | `kis.rs`, `toss/adapter.rs`, `rate_limit.rs` |
| 캔들 페이지/일반 preview 사전자료 | `src-tauri/src/commands/candles.rs`, `broker/candles.rs` | `api/rest/chart.rs`, `src/shared/api/candlePageTypes.ts`, `pages/strategy/model/candlePageCollector.ts`, `server/market.rs` |
| 자동매매 시작/데몬 | `src-tauri/src/commands/trading.rs` / `commands/trading/daemon.rs` | `daemon/{fence,flow}.rs`, `commands/trading/{positions,balance,risk_restore,history,daily_history}.rs`, `lib.rs` |
| Live 완료 일봉 준비·거래량 | `commands/trading/daily_history.rs`, `daily_history/jobs.rs` | `trading/strategy/{core,manager/live_history}.rs`, `src/shared/ui/LiveDailyHistoryPanel.tsx`, TradingStatus |
| 전략 | `src-tauri/src/trading/strategy/` | `commands/strategy.rs`, `commands/strategy_preview.rs`, Strategy UI |
| 가격조건 소유·손절/재진입 계약 검증 | `trading/order/feedback/tests/price_condition.rs`, `commands/strategy_preview/pc_replay_tests.rs` | `trading/strategy/price_condition.rs`, `order/submission/owned.rs`, `simulation/session.rs`, [고정 설계·생산 재현](research/price-condition-contracts-20261029/report.md), [2회차 기록](tasks/immediate-backlog-20261029.md). 생산 정책 변경 없이 live/replay 합성 계약과 기존 고정 일봉 원장을 재현하며 실제 장중 효능은 미완료 |
| 추세 필터 P2-11 소유·선택 보호 | `src-tauri/src/trading/strategy/mean_trend.rs`, `mean_trend/trend_filter_{params,protection,tests}.rs` | `src/pages/strategy/ui/{strategyPolicyControls,trendFilterControls}.tsx`, `docs/research/trend-filter-policy-20261023/{runner.rs,baseline_runner.rs,report.md}`, `docs/tasks/trend-filter-policy-20261023.md`; 실제 소유/잔량·strict checkpoint·저장/두 preview/UI와 고정81fa00d/v21→v22 생산 실행·독립 대사를 구분 |
| RSI 실제 소유·선택 보호 | `trading/strategy/classic/rsi.rs`, `classic/rsi/params.rs` | `strategy/observation.rs`, `configuration/tests/rsi.rs`, `order/feedback/tests/rsi.rs`, `strategy_preview/rsi_entry_tests.rs`, `rsiControls.tsx`, `docs/research/rsi-policy-20261022/` |
| 변동성 확장 이전 범위·시장일 관측·실제 소유 | `trading/strategy/breakout/volatility_expansion.rs`, `volatility_expansion/{params,context,protection}.rs` | `strategy/{core,manager/live_history}.rs`, `manager/volatility_history_tests.rs`, `configuration/tests/volatility.rs`, `strategy_preview/volatility_entry_tests.rs`, `src/shared/api/strategyResearchTypes.ts`, `volatilityExpansion{Controls,Preparation}.tsx`, `tests/e2e/strategy-volatility-expansion-policy.spec.ts`, `docs/research/volatility-expansion-policy-20261022/` |
| 강한 종가 완료 시장일·실제 소유·선택 보호 | `trading/strategy/breakout/strong_close.rs`, `strong_close/{params,context,protection}.rs` | `strategy/{core,manager/live_history}.rs`, `manager/strong_close_history_tests.rs`, `configuration/tests/strong_close.rs`, `strategy_preview/strong_close_entry_tests.rs`, `strongClose{Controls,Preparation}.tsx`, `tests/e2e/strategy-strong-close-policy.spec.ts`, `docs/research/strong-close-policy-20261022/` |
| 돌파 실패 실제 소유·선택 진입/보호 | `trading/strategy/sequence/failed_breakout.rs`, `failed_breakout/{params,context,protection}.rs` | `strategy/{core,completed_daily,manager/live_history}.rs`, `configuration/tests/failed_breakout.rs`, `strategy_preview/failed_breakout_entry_tests.rs`, `failedBreakout{Controls,Preparation}.tsx`, `tests/e2e/strategy-failed-breakout-policy.spec.ts`, `docs/research/failed-breakout-policy-20261022/` |
| Replay 체결 모델·연구 보호 청산 | `trading/simulation/{session,execution_tests}.rs`, `simulation.rs` | `commands/strategy_preview{,/generic}.rs`, `strategy/leveraged_trend_hold/preview.rs`, `strategyResearchPanel.tsx`, `tests/e2e/strategy-execution-model.spec.ts` |
| Replay 동일 장부 구간 진단 | `trading/simulation/session/phase_diagnostics.rs`, `simulation/phase_diagnostics_tests.rs` | `simulation.rs`, `src/shared/api/strategyResearchTypes.ts`, `src/pages/strategy/ui/strategyPhaseResults.tsx`, `strategyResearchPanel.tsx`, `tests/e2e/strategy-phase-diagnostics.spec.ts`. v23/진단v1 관측·경계·손익 귀속이며 별도 walk-forward와 구분 |
| Replay 손익·현금흐름·비교 | `trading/simulation/{accounting,comparison}.rs`, `simulation/session/accounting.rs` | `simulation/accounting_tests.rs`, `src/shared/api/strategyResearchTypes.ts`, `strategyAccountingPanel.tsx`, `tests/e2e/strategy-accounting.spec.ts`, `docs/research/replay-accounting-20261018/` |
| 연속 replay·분기 리셋 비교 | `commands/strategy_preview/{continuous,driver}.rs` | `continuous_tests.rs`, `simulation/session.rs`, `continuousReplayPanel.tsx`, `candlePageCollector.ts`, `docs/research/continuous-replay-20261018/` |
| 전략 설정 저장·보유 보호 | `src-tauri/src/trading/configuration.rs` | `trading/order/configuration.rs`, `trading/strategy/manager/configuration.rs`, 각 전략의 `reconfigure_tests.rs`, IPC/Web 전략 수정 facade |
| 잔고 조회·tracker 게시·계좌별 캐시 | `src-tauri/src/trading/order/holdings.rs`, `holdings/{fetch,parse}.rs` | `trading/position.rs`, `commands/accounts.rs`, `commands/trading/positions.rs`, `server/{market,trading}.rs`, `src/api/{accountHooks,backendEvents}.ts` |
| 수동·자동 주문 | `src-tauri/src/trading/order/submission.rs` | `order/execution_run.rs`, `submission/{positions,prepare,provider,records}.rs`, `commands/orders.rs` |
| 실행 변경·지연 결과 폐기 | `commands/trading/daemon/fence.rs`, `trading/order/execution_run.rs` | IPC/Web start/stop·전략 수정·프로파일 적용, `submission/run_tests.rs` |
| 주문 전 검증 | `src-tauri/src/trading/preflight.rs`, `trading/guard.rs` | broker별 preflight command, `trading/risk.rs` |
| 리스크 | `src-tauri/src/trading/risk.rs` | `commands/risk.rs`, `storage/risk_store.rs`, restore tests |
| pending/체결 | `trading/order/fills.rs`, `order/fills/reconciliation.rs` | `order/{pending,health}.rs`, `fills/{collector,provider_mode}.rs`, `storage/{pending_order_store,order_store,trade_store}.rs` |
| 응답 불명 주문 대조·복구 | `trading/order/budget/recovery.rs`, `submission/recovery.rs`, `commands/budget.rs` | `src/pages/settings/ui/budgetRecoverySection.tsx`, `src/api/budgetRecoveryHooks.ts`, `src/shared/api/budgetRecovery{Types,Commands}.ts`, `budget/recovery/{history,tests}.rs`, `trading/guard/recovery.rs`, `trading/guard/recovery/tests.rs`, `storage/database/recovery_history.rs`, `storage/{order_store,trade_store}.rs`, `tests/e2e/budget-recovery.spec.ts`, [작업 기록](tasks/immediate-backlog-20261029.md). 데스크톱에서 원래 주문 근거와 보관된 primary 이력의 scope/provider ID 충돌을 대조해 pending과 원래 제출 시각의 멱등 guard 보호를 복원. 공통 제출의 budget_owned→load_budget 경로가 guard 평가 전에 보호를 읽으며 실제 provider 검증·예산 해제와 구분 |
| 전략 소유 수량·체결 피드백 | `trading/order/{ownership,feedback}.rs` | `ownership/runtime.rs`, `submission/owned.rs`, `fills/ownership.rs`, `trading/strategy/{core,manager}.rs`, 각 `live_feedback_tests.rs` |
| 체결 손익·재시도 영수증 | `trading/order/fills/ownership.rs`, `trading/risk/fill_accounting.rs` | `fills/collector/notional.rs`, `commands/trading/risk_restore.rs`, `storage/pending_order_store.rs`, `order/ownership/` |
| Toss 소액 검증·수수료율 표시 | `commands/toss/small_order.rs`, `trading/preflight.rs` | `small_order/tests.rs`, `src/pages/trading/ui/tossPanels.tsx`, `tests/e2e/toss-commission-ratio.spec.ts` |
| 저장 backend·설정 복구 | `src-tauri/src/storage/mod.rs`, `database.rs` | `storage/database/{configuration,connection,tests}.rs`, 각 `*_store.rs` |
| DB TLS·강제 단절·재접속 계약 | `scripts/database-tls-contract.mjs` | `storage/database_contract_tests/tls_restart.rs`, `docs/tasks/database-tls-reconnect-20261018.md`, `docs/HARNESS_MAP.md` |
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

### 돌파 실패의 가격관측·완료 일봉과 소유 보호

`strategy/sequence.rs`는 공개 facade이고 `sequence/failed_breakout.rs`가
기본 이전 N가격관측 최대값과 선택 이전 N완료 일봉 고가를 구분한다.
`failed_breakout/params.rs`는 선택 기준·ATR/거래량·수익 보호 쌍·보유 관측
한도를 검증하고, `context.rs`는 이전 완료 일봉 최대512개와 준비/후보 진단을
소유한다. ATR은 이전15봉의 TR14, 거래량은 직전 완료량/그 이전20봉 평균이며
현재 봉 고가·저가·거래량이나 live 누적량을 자신의 진입 확인에 쓰지 않는다.

`core.rs`의 `CompletedDailyBar`는 OHLC와 `Option<u64>` 거래량을 함께
전달한다. live 수집→jobs→manager의 ticket/freshness 검증→전략 apply와
replay history 초기화→종가 판단/피드백→완료봉 게시가 같은 훅을 사용한다.
기존 전략의 기본 훅은 OHLC 경로로 한 번 위임하고 거래량 미확인을0으로 만들지
않는다. 공통 `completed_daily.rs`는0을 허용하는 TR14와 기존 양수 ATR14
계약을 나누며 준비 정책은 전략 context에 남긴다.

실제 양수 체결 피드백이 pending 진입 기준을 확정한다. 실패→수익 보호→보유
관측 한도 순으로 실제 잔량 Sell 후보를 내고, 미제출/거절·부분 체결·같은 scope
설정 변경은 확정 기준과 알려진 보호 상태를 보존한다. `protection.rs`는 v2
나이/관측 최고가/무장을 원자적으로 복원하고 v1의 미확인 값을 추정하지 않는다.
두 preview는 현재 v20 driver를 공유하며 완료 고가/ATR/거래량 활성 비일봉을 거부하고,
거래량 필터 활성 hash는 미확인과 검증된0을 구분한다. UI controls는 raw 초안과
수익 보호 쌍을 검증하고 preparation은 현재/A-B에서 준비 미확인·거래량0·기준
평균0·후보 통과를 구분한다. 통과는 실제 체결이 아니다.

### 변동성 확장의 이전 완료 범위·시장일 관측과 실제 소유

`strategy/breakout.rs`는 공개 facade이며 구현은 `breakout/volatility_expansion.rs`와
`volatility_expansion/{params,context,protection}.rs`로 나뉜다. params는 기존10/2/3과
선택 수익 보호 쌍/보유 관측3개의 strict 검증을 소유하고, context는 최대512개 이전
완료 고저 범위·0 포함 산술평균과 당일 관측 범위를 분리한다. 원래 진입은 당일 범위가
이전 N완료 범위 평균×배수보다 엄격히 크고 현재가도 당일 시가보다 엄격히 커야 한다.
배수0도 양의 범위가 필요하며 손절0은 평균가 이하 청산이다.

live daily coordinator의 scope/profile/config/generation/시장일·OHLC/freshness 검증 뒤
N완료봉을 dated 훅에 전달한다. 검증된 새 시장일의 충분한 자료만 당일 관측을
초기화하며 같은 날 재조회·실패·늦은 legacy warmup은 이미 관측한 범위와 보유 보호를
지우지 않는다. live 시가는 검증된 날 첫 관측 가격의 대용값이며 고저도 관측 가격
범위다. 일봉 replay의 D 완성 OHLC/D종가 신호→다음 관측 시가(nextOpen)와 구분한다.
현재 완료 범위는 판단·체결 피드백 뒤에 게시한다.

실제 q/native 평균가와 후보/pending은 별개이며 원래 손절→보유 관측 최고가의 수익
보호→보유 관측 cap 순으로 실제 잔량 Sell 후보를 낸다. strict v1
kind=volatilityExpansion metadata는 미확인 나이/고점/무장을 추정하지 않는다.
거절·부분 체결·같은 scope 설정은 알려진 보호를 보존하며 미확인 나이와 활성 cap은
다음 보유 관측에서 보수적 청산 후보를 낸다.

현재 v23 공통 driver의 두 preview와 저장 경계는 같은6필드를 strict 검증하고 기존 일봉
전용을 유지한다. `core.rs`의 전용12필드 `VolatilityExpansionPreparation`은 평가=보유+
미보유 후보, 후보=미준비+범위 거부+방향 거부+통과로 분류한다. 선택 응답 필드→TS
미러→preview/research panel→현재/A-B로 보존하며 controls/preparation은 raw/stored
초안·합계/형식 검증·누락/손상 미확인을 표시한다. 수량 산정 ATR·후보 통과·실제
체결을 구분한다. 고정47fc10b/v20 baseline과 v21 후보를 포함한36연속/432별도분기 기록468원장,
v21 전용390진단·249412검사의 독립 감사는 같은 연구 경로의 gzip2개·summary/aggregate/
validation·independent-audit.json으로 연결한다. 분할36개는 생산 assertion과 저장 원장
hash 연결만 확인했으며 미저장 대체 원장 독립 재정산이나 새 표본으로 세지 않는다.
[연구](research/volatility-expansion-policy-20261022/report.md)와
[작업](tasks/volatility-expansion-policy-20261022.md)에서 자동 구현·일봉 연구의 검증 범위를
확인한다. 실제 장중 최초 발동/체결 자료 게이트는 미완료다.

### 강한 종가의 완료 시장일·실제 소유와 선택 보호

`strategy/breakout.rs`는 공개 facade이며 실제 StrongClose는
`breakout/strong_close.rs`와 `strong_close/{params,context,protection}.rs`다.
기존 기준3/손절3·(고가−종가)/고가 조건과 평탄봉 통과를 유지한다. 손절0은
평균가 이하 청산이고 새 추세/수익 보호 쌍/보유 관측4옵션만 누락/0 비활성이다.
추세는 현재가가 이전 N완료 일봉 종가 평균보다 엄격히 높은지 검사하며 최대512개
context에 현재봉을 미리 게시하지 않는다.

StrongClose는 항상 max(1,추세N) 완료 일봉을 요구한다. live jobs의 자료가 기존
scope/profile/config/generation/시장 날짜·OHLC/freshness를 통과하면 manager가
검증된 ticket 시장일을 `core.rs`의 dated apply 훅에 전달한다. 기본 훅은 기존
apply로 한 번 위임하며 StrongClose와 VolatilityExpansion은 각각 override한다.
StrongClose는 새 시장일의 전일 조건을 seed하고 소비를 관리하며 VolatilityExpansion은
새 시장일 당일 관측 open/high/low를 초기화하고 같은 날 범위를 보존한다. StrongClose의
같은 날 재조회는 context만 교체하며 소비한 조건을 재무장하지 않는다.
실패/무효화/늦은 legacy 초기화에도 날짜 소비와 보유 보호를 유지하고 신규 Buy만
보류한다. OHLC 동일성·호스트 날짜로 시장일을 추측하거나 flat 재시작 quota를
추가하지 않는다.

실제 소유 q/native 평균가와 pending 후보를 분리하며 원래 손절→보유 관측
최고가의 수익 보호→보유 관측 cap 순으로 실제 잔량 Sell 후보를 낸다. 거절·부분
체결·같은 scope 설정은 알려진 나이/고점/무장을 보존한다. strict v1
kind=strongClose metadata는 미확인 나이/고점/무장을 만들어내지 않으며 나이
미확인·cap 활성은 다음 보유 관측에 보수적 청산 후보를 낸다.

현재 v23 공통 driver는 선택 params를 strict 검증하고 기존 일봉 전용·실제 체결 피드백
뒤 완료봉 게시 순서를 유지한다. D-1조건→D종가 신호→D+1관측 시가(nextOpen)는
live 첫 적격 관측과 다르다. controls/preparation은 raw/stored 편집·수익 보호 쌍과
현재/A-B 준비/후보 진단·legacy 미확인을 표시한다. 생산 실행기·기록 정산·작동
집계와 `independent-audit.json`의 기록390원장 독립 정산은
`docs/research/strong-close-policy-20261022/`에 둔다. 분할30개는 생산 assertion·입력
hash 연결이며 미저장 대체 원장 독립 정산과 구분하고 live 날짜 fixture도 과거
연구 성과와 분리한다. [작업](tasks/strong-close-policy-20261022.md)과
[연구](research/strong-close-policy-20261022/report.md)에서 최종 검증 범위를 확인한다.

### RSI 소유 피드백·관측과 교차

`strategy/classic.rs`는 공개 경로를 유지하는 facade이며 실제 RSI 구현은
`classic/rsi.rs`와 `rsi/params.rs`다. 기존14/30/70 단순평균·매 관측 교차 기준
갱신과 별도 실제 소유 q/native 평균가·기록 나이/origin을 구분한다. 선택 추세는
현재 제외 이전 입력가격으로, live 첫 추세 seed만 정리하고 기존 RSI warmup을
지우지 않는다. 원래 교차는 거절 뒤 재시도하지 않고 손절/관측 cap만 조건 지속
중 잔량 청산 후보를 다시 낸다. 모든 held Sell은 실제 잔량 전량을 요청한다.

params 변경은 공통 configuration strict 저장/게시와 두 preview driver의 검증,
`rsiControls.tsx`의 raw/타입·단위·기본0 및 current/A-B 요약을 함께 확인한다.
보유 나이/metadata 기계 동작만 기존 `strategy/observation.rs`의8함수를 재사용하고
RSI 계산·교차·손절/추세 의미는 전략에 남긴다. 고정 생산30연속/30분할/
360별도분기 비교에서 실제 소유 기본의78개 경제 경로는 legacy와 같았다.
부적합 신호 감소를 수익 개선으로 해석하지 않고 선택 추세/손절 기각과
보유63 미발동·효능 미평가를 구분한다. 실제 실행·감사 범위와 해석은
[작업](tasks/rsi-policy-20261022.md), [연구](research/rsi-policy-20261022/report.md),
[IPC 계약](ipc-commands.md#rsi-v18-실제-소유선택-보호)을 따른다.

### Replay 회계·체결 피드백

`trading/simulation/session.rs`의 ReplaySession이 batch와 일반/LTH preview의
포지션·비용·리스크 회계를 공유한다. 각 관측 봉은 begin_bar의 이전 종가 신호 시가
체결·연구 보호 청산 → 전략에 체결 포지션 피드백 → 종가 신호 → process_close의
종가 체결 또는 nextOpen 보류·mark/equity → 체결 포지션 피드백 순서로 처리한다.
다음 시가 체결은 이전 완료봉 ATR을 사용하고, 평가 마지막 신호의 미체결은 차단과
구분한다. 연구 보호 주문은 원시 전략 신호와 별도이며 OHLC 내부 순서는 미검증으로
남기거나 선택한 손절 우선 근사로 처리한다.

연구 UI는 체결/보호 가정·신호 공개와 체결 단계·미체결·동시 도달·A/B의 엔진과
가정 차이를 표시한다. 이전 결과의 누락 metadata는 미확인으로 남긴다. 동결 자료를
생산 엔진으로 비교하는 진입점은 `docs/research/replay-execution-20261018/runner.rs`다.
이 경로는 사용자 운용 설정·실제 주문을 변경하지 않는다. [연구 비교](research/replay-execution-20261018/report.md)와 [작업 기록](tasks/replay-execution-model-20261018.md)에서 검증 범위를 확인한다.

### 동일 장부의 앞/뒤 구간 관측

`session/phase_diagnostics.rs`는 v23 ReplaySession에서 두 구간의 시작/끝
snapshot과 관측·종가 보유봉을 기록한다. 시작 snapshot은 해당 봉의 FX·배당·
시가 체결 전이며, 실현손익·잔여 평가손익 변화·순배당을 자산 증감과 연결한다.
장부·보유·대기 주문·guard/risk는 경계를 넘어 유지하고 기존 구간값은 보존한다.
선택 `diagnostics`의 v1/sameLedgerObservationSplit은 TS 미러를 거쳐
`strategyPhaseResults.tsx`의 flat/LTH/continuous·현재/A-B에 전달한다. 누락·
손상·미지원 버전은 미확인으로 표시하고 저장 축약 자료로 다시 계산하지 않는다.
보합과 손실을 구분하며 길이·노출·부분매도 포함 완료 표본이 다른 누적수익을
과최적화나 독립 OOS로 판정하지 않는다. 별도 생산 walk-forward 설계는
`docs/research/phase-walk-forward-20261029/`에서 관리하며 이 관측 진단과 구분한다.

### Replay 손익·현금흐름과 비교

`simulation/accounting.rs`가 방향별·최소 수수료와 수동 출처, 평가 봉에 일치하는
bounded FX·배당 가정을 검증한다. `ReplaySession`은 해당 봉의 환율·직전 보유
수량의 배당을 적용한 뒤 대기 시가 주문을 처리하며 native 평균 체결가와 역사적
KRW 원가를 분리한다. 해외 장부의 유휴 현금은 KRW이며 USD 현금 재평가가 아니다.
`session/accounting.rs`는 실현·잔여·가격/환율·배당·비용 대사와 거래 수를 바꾸지
않는 종료 청산 projection, 관측 봉말 투자 비중을 생성한다. 최소수수료 모드의
일반 이익 매도는 실제 청산 수량의 순익을 검사하고 손절·native 손실 청산은 보존한다.

`comparison.rs`는 같은 입력 수량의 보유 지시를 같은 세션으로 실행하고 실제 수량과
노출 차이를 공개한다. 실제 체결 경로의 기록 KRW 비용 환급은 독립 무비용 전략
재실행과 구분한다. `strategyAccountingPanel.tsx`가 일반/LTH 공통 입력·결과를
표시하며 잘못된 초안은 실행을 막고 과거 누락 회계를 미확인으로 남긴다.
생산 실행·기준·결과는 `docs/research/replay-accounting-20261018/`에 둔다.
[연구 보고서](research/replay-accounting-20261018/report.md)의 직전/현재 harness
manifest와 `validate.py`의 독립 기록 fill 대사로 재현하며, 구현·검증 범위는
[작업 기록](tasks/replay-accounting-20261018.md)을 따른다.

### 연속 replay 경계

`commands/strategy_preview/generic.rs`의 기존 flat500 입력과 새 `continuous.rs`의
최대16청크/각500봉·평가4096봉·이전history512봉 입력은 `driver.rs`를 공유한다.
모든 시각/겹침/가정을 검증한 뒤 생산 전략과 warmup·ReplaySession을 한 번만
생성하고 순차 청크 사이의 지표·보유·guard/risk·pending·ATR을 보존한다.
단일 요청이 끝날 때 한 번 finish하며 서버의 세션/append 상태는 만들지 않는다.

UI는 별도 일봉 날짜 범위와 최대48페이지·보관4608봉 정책으로 엄격히 이전252봉을
수집한다. 준비/요청 범위/관측 간격 경계를 확인하지 못하면 계산을 보류하고
coverageVerified=false 한계를 유지한다. mutation/공통 transport는 desktop의
`preview_strategy_continuous`와 Web POST `/api/strategy/preview/continuous`를
같은 command에 연결한다. 응답 continuity는 공급 경계의 수량·대기 신호를
표시하며 분기 리셋·독립 실험이 아니다. 입력 hash는 모드를 구분하고 청크 분할
형태는 제외한다. 사전 고정된 연속/분기 리셋 비교는
`docs/research/continuous-replay-20261018/`의12파일에 보관한다.
[연구 보고서](research/continuous-replay-20261018/report.md)의 baseline/현재
harness manifest로 재현하고 `validate.py`로 기록 fill과 실제 보유구간을 독립
대사한다. 전체 JSONL은 두 .gz로 손실 없이 압축해 보관하며 생성 평문 .cache는
프로젝트 인벤토리에 넣지 않는다. 구현·검증 범위는
[작업 기록](tasks/continuous-replay-20261018.md)을 따른다.

### DB 설정·pool 변경

설정 파일·OS keychain 저장과 메모리 게시 경계는 `database/configuration.rs`,
provider TLS·관리자 DB 생성·pool 경계는 `database/connection.rs`가 소유한다.
설정 저장 전 recovery 표시와 runtime latch를 유지하고, 파일/비밀 복원 불확실·취소는
연결과 후속 저장을 차단한다. cold pool 생성은 직렬화하고 연결 완료 후 설정
fingerprint를 재검사한다. 상태는 같은 설정/pool snapshot을 사용한다.

DB 단절은 store에 오류로 전달하며, 서버 재기동 후 기존 pool의 새 연결 취득으로
복구한다. 애플리케이션의 쓰기 자동 재실행은 추가하지 않는다. 계약 실행기의
임시 CA·개인키·결과는 Git 제외 `.cache/database-tls-contract/<UUID>/`에 두고
고유 소유 라벨의 컨테이너만 제어·정리한다. 실행 명령과 한계는 Harness 지도를 본다.

## 6. 데이터와 민감 영역

| 데이터 | 코드상 위치/경계 |
|---|---|
| 프로파일 | Tauri app data의 `profiles.json`; `lib.rs`/`config` 관리 |
| 거래 데이터 | 실행 CWD의 `data/`; `storage::*Store`와 JSON/DB document 경계 |
| 로그 | 실행 CWD의 `logs/`; `logging/` |
| DB 설정 | app data의 `database_config.json`; password는 OS keychain 우선, 사용 불가 시 권한 제한 파일에 유지하며 IPC view에서는 미반환 |
| DB 설정 복구 표시 | app data의 `database_config.json.recovery`; 비밀 없는 저장/게시 미완료 표시이며 runtime latch와 함께 사용 |
| DB export | app data의 `database_exports/`; manifest 포함 |

실제 `.env`, `secure_config.json`, `profiles.json`은 탐색 대상이 아니다. 위치·형식의 상세 기준은 `docs/project-map.md`와 `docs/coding-guide.md`를 따른다.

## 7. 지도 유지 원칙

- 이 문서는 작업 라우팅이 달라질 때만 갱신한다.
- 전체 파일 추가·이동·삭제는 `npm run project-map:update`로 `docs/project-map.md` 생성 블록을 갱신하고 `npm run check:project-map`으로 확인한다.
- IPC 이름은 이 문서에 복제하지 않고 `docs/ipc-commands.md`에만 유지한다.
- 검증 명령은 이 문서에 복제하지 않고 [`docs/HARNESS_MAP.md`](HARNESS_MAP.md)에 유지한다.
