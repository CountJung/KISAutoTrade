# Graph Report - KISAutoTrade  (2026-11-08)

## Corpus Check
- 618 files · ~448,282 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 8267 nodes · 15453 edges · 1602 communities (357 shown, 1245 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 1043 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `67f8d9ca`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- OwnershipLedger
- shared/api/commands.ts
- rest/types.rs
- RiskManager
- simulation/minute_timing_tests.rs
- TradeRecord
- FiftyTwoWeekHighStrategy
- StrongCloseStrategy
- TrendFilterStrategy
- leveragedTrendHoldEditorPanel.tsx
- OrderManager
- ui/AppShell.tsx
- toss/types.rs
- toss_market.rs
- DatabaseManager
- market_hours.rs
- database_io.rs
- BrokerAccountId
- TossOpenApiClient
- observation.rs
- accounts.rs
- history/ui/Page.tsx
- page_view
- toss_market.rs
- ServerState
- BrokerQuantity
- StrategyManager
- RunFence
- tauri.conf.json
- prepare
- strategy/ui/Page.tsx
- accounts.rs
- RsiStrategy
- settings.rs
- order/ownership/tests.rs
- .submit_order_shared
- strategy_preview.rs
- server/records.rs
- databaseManagementSection.tsx
- DatabaseManager
- AppState
- DatabaseConfig
- MomentumStrategy
- databaseManagementSection.tsx
- dashboard/ui/Page.tsx
- commands/records.rs
- PriceConditionStrategy
- DiscordNotifier
- autoTradingBudgetSection.tsx
- FailedBreakoutStrategy
- PositionTracker
- ReplaySession
- leveraged_trend_hold.rs
- AppState
- security.rs
- VolatilityExpansionStrategy
- simulation.rs
- BrokerSymbol
- order/budget.rs
- OrderManager
- OrderRecord
- settings/ui/Page.tsx
- run_backtest
- DatabasePool
- TradeRecord
- server/trading.rs
- MovingAverageCrossStrategy
- scripts
- build_strategy
- commands/toss.rs
- strategyPolicyControls.tsx
- commands/risk.rs
- .new
- strategyPreviewPanel.tsx
- KisWebSocketClient
- OhlcCandle
- read_json_or_default
- database_projection.rs
- ConsecutiveMoveStrategy
- project-map.mjs
- start_trading
- BrokerQuantity
- MovingAverageCrossStrategy
- accountHooks.ts
- shared/api/types.ts
- strategyResearchPanel.tsx
- collect_kis_pending_fills
- support.rs
- strategy_with_params
- HoldingsQuery
- simulation/accounting.rs
- TradeStore
- strategyResearchTypes.ts
- @mui/icons-material
- balance_store.rs
- LeveragedTrendHoldParams
- check
- compilerOptions
- holdings/tests.rs
- MeanReversionStrategy
- verify-toss-openapi.mjs
- OverseasExecutedOrder
- BrokerAccountId
- TossAccessToken
- dependencies
- model/tradingStore.ts
- TossApiError
- LeveragedTrendHoldStrategy
- model/accountStore.ts
- leveragedTrendHoldPreviewChart.tsx
- NotificationService
- exchange.rs
- toss_currency_from_view
- strategy/tests.rs
- day_event_tests.rs
- setup-local.sh
- Signal
- main.rs
- volatility_expansion/tests.rs
- generic_tests.rs
- preview_strategy_continuous_from_candles
- failed_breakout/tests.rs
- discord-notification-config/index.ts
- manager
- strategy-configure/index.ts
- generic.rs
- Result
- PendingOrderStore
- conflicts.rs
- MarketDayCalendar
- DailyHistoryTicket
- validate_pending_kis_mode
- BrokerScope
- .search
- run_backtest_with_history
- AppConfigView
- 작업 기록: <짧은 제목>
- StatsStore
- .process_buy
- CurrencyLedger
- verify-lockfiles.mjs
- modify_toss_order_for_profile
- deviation/tests.rs
- HARNESS_MAP.md
- 2. 목표와 비목표
- react-grid-layout
- api/mod.rs
- rsi/tests.rs
- Arc
- AtomicBool
- ChartCandle
- restore_risk_from_today_trades
- Fixture
- ExecutedOrder
- HeaderMap
- .new
- strategy-continuous-replay.spec.ts
- PriceResponse
- breakout.rs
- Strategy
- collect_pages
- .checkpoint_strategy_positions_shared
- .capture
- .collect
- collect_daily_history
- toss/orders.rs
- Result
- Self
- String
- T
- Value
- Vec
- Arc
- Client
- lth_replay_clock.rs
- Mutex
- Option
- Result
- Self
- String
- integer
- AppHandle
- preview_strategy_from_candles
- .submit_order_shared
- AccountProfile
- FixedOffset
- RwLock
- Self
- Sender
- String
- checked_budget_scope
- BrokerAdapterResult
- failed_breakout_entry_tests.rs
- EntryContext
- commands/risk.rs
- ConfigurationEditReservation
- SectorPortfolioPanel
- DeviationStrategy
- toss/orders.rs
- Self
- consecutive/tests.rs
- trend_filter_protection.rs
- Arc
- BalanceItem
- ui/OverseasStockChart.tsx
- config
- input
- .prepare_confirmed_ownership
- TossTokenState
- .preview_signals_with_execution
- strong_close/tests.rs
- uiux-history-details.spec.ts
- .new
- HashMap
- HeaderMap
- broker/candles.rs
- StockStore
- 작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단
- budget/recovery/tests.rs
- toss/tests.rs
- PendingOrderStore
- BrokerHoldingView
- BrokerAdapterResult
- BrokerCurrency
- BrokerId
- mean_trend/trend_filter_tests.rs
- submission.rs
- strong_close_history_tests.rs
- lth_replay_clock.rs
- Self
- initialize_strategy_warmup
- Vec
- Arc
- BrokerCurrency
- Client
- Into
- Option
- Result
- String
- T
- Vec
- CandleDataSummary.tsx
- MockSource
- StatusCode
- String
- strategy/tests.rs
- Client
- Response
- Result
- main
- Option
- Result
- Self
- String
- Vec
- BrokerCurrency
- BrokerMarket
- Option
- Result
- String
- config
- failedBreakoutControls.tsx
- From
- bounded_optional_integer
- holding
- config
- classic/live_feedback_tests.rs
- T
- feedback.rs
- fill_accounting.rs
- vite
- DailyHistoryJobs
- strategyAccountingPanel.tsx
- prepare
- strategy_preview/tests.rs
- collect_daily_history
- CmdError
- RunFence
- ChartCandle
- Option
- Result
- Self
- String
- deviation_entry_tests.rs
- CmdResult
- mean_reversion_entry_tests.rs
- capture_cycle
- get_toss_market_snapshot_for_profile
- database_archive.rs
- scaled_total
- guard/recovery/tests.rs
- StrongCloseState
- database/tests.rs
- trendFilterControls.tsx
- KisBrokerAdapter
- CmdResult
- Error
- RwLock
- autoTradingBudgetSection.tsx
- String
- CmdError
- CmdResult
- DatabaseConfigView
- formatters.ts
- DatabaseTransferResult
- attach_comparison
- FailedBreakoutParams
- lib.rs
- execution_block_reason
- CandleDataSummary.tsx
- continuous.rs
- ChartCandle
- ChartCandle
- DatabaseManager
- strategy_preview/trend_filter_tests.rs
- ContinuousStrategyPreviewInput
- managed_paths
- input
- mean_reversion_entry_tests.rs
- .record_fill_pnl_once
- CmdResult
- Option
- OrderResponse
- OrderSide
- OrderType
- Self
- State
- String
- CmdResult
- Option
- State
- String
- strategy-trend-filter-policy.spec.ts
- mean_reversion/tests.rs
- strategy-volatility-expansion-policy.spec.ts
- FailedBreakoutState
- FailedBreakoutParams
- PreparedOrderSubmission
- config
- config
- balance-events.spec.ts
- momentum_entry_tests.rs
- .capture
- historyFixture
- strategy-failed-breakout-policy.spec.ts
- strategy-consecutive-entry-filters.spec.ts
- StrategyConfig
- attach_comparison
- current_position_snapshot
- ReplayAssessmentView
- place_prepared_order
- finish_closed_batch
- .submit_order_shared
- .restore_recovered_submission_at
- .new
- mockApi
- strategy-strong-close-policy.spec.ts
- TradeStore
- managed_paths
- candles/tests.rs
- .validate_recovery_history
- report_json
- database/tests.rs
- initialize_strategy_warmup_with_daily_bars
- TossAccessToken
- failed_breakout_state
- config
- install_database_manager
- integer
- strategy/tests.rs
- LeveragedTrendHoldStrategy
- preview_strategy
- exits.rs
- indicators.rs
- live.rs
- preview.rs
- BrokerId
- CmdResult
- From
- Option
- Self
- String
- volatility_entry_tests.rs
- leveragedTrendHoldPreviewChart.tsx
- AtrWindow
- VolatilityExpansionState
- S
- rsi_state
- strongCloseControls.tsx
- TossTokenState
- notional/tests.rs
- database_archive.rs
- config
- Arc
- BalanceItem
- BrokerAdapterResult
- Arc
- BrokerCurrency
- BrokerId
- BrokerMarket
- BrokerMoneyView
- Mutex
- Option
- Duration
- FnOnce
- String
- Arc
- Mutex
- Option
- OrderManager
- RwLock
- Vec
- AppHandle
- Arc
- BrokerId
- PathBuf
- Result
- Self
- StatusCode
- String
- Value
- Arc
- BalanceItem
- BalanceSummary
- BrokerCurrency
- BrokerId
- BrokerMarket
- CmdError
- CmdResult
- From
- KisRestClient
- Option
- Self
- Send
- Sync
- Into
- Option
- Self
- String
- AutoTradingBudgetView
- CmdError
- Json
- NaiveDate
- Option
- String
- Utc
- Vec
- CandlePageInput
- OrderType
- Path
- Query
- Response
- Result
- State
- String
- Value
- Self
- String
- Arc
- BrokerId
- CmdResult
- Default
- LogConfig
- Option
- Path
- RwLock
- StockSearchItem
- TradeArchiveConfig
- Value
- Vec
- NaiveDate
- Option
- Result
- String
- Vec
- StrategyPreviewView
- String
- Value
- Vec
- ChartCandle
- ContinuousStrategyPreviewInput
- StrategyPreviewInput
- StrategyPreviewView
- Value
- Vec
- ChartCandle
- StrategyPreviewInput
- Value
- Vec
- ContinuousStrategyPreviewView
- String
- Value
- CmdResult
- PreviewRow
- StrategyPreviewInput
- StrategyPreviewView
- Query
- Result
- State
- String
- Value
- BrokerCurrency
- BrokerId
- Json
- Arc
- Mutex
- NaiveDate
- Path
- PathBuf
- Result
- T
- SimulationAssumptions
- String
- Value
- Vec
- VolatilityExpansionPreparation
- ChartCandle
- StrategyPreviewInput
- StrategyPreviewView
- ContinuousStrategyPreviewView
- StrategyPreviewInput
- String
- Value
- ContinuousStrategyPreviewView
- String
- Value
- PathBuf
- Result
- Self
- Mutex
- NaiveDate
- PathBuf
- Result
- Self
- String
- Vec
- Arc
- HashMap
- I
- ContinuousStrategyPreviewView
- String
- Value
- ContinuousStrategyPreviewView
- StrategyPreviewInput
- String
- Value
- ContinuousStrategyPreviewView
- StrategyPreviewInput
- String
- Value
- ContinuousStrategyPreviewView
- StrategyPreviewInput
- String
- Value
- BrokerId
- CmdResult
- From
- Option
- Self
- String
- Value
- Vec
- CmdResult
- String
- Vec
- BrokerId
- CmdResult
- Into
- Option
- String
- Vec
- Arc
- BrokerId
- CmdResult
- Mutex
- Option
- OrderManager
- String
- Vec
- BrokerCurrency
- BrokerId
- BrokerMarket
- BrokerMoneyView
- CmdResult
- Option
- String
- Vec
- Arc
- BrokerCurrency
- BrokerId
- BrokerMarket
- BrokerMoneyView
- CmdResult
- Mutex
- NaiveDate
- Option
- OrderManager
- String
- Arc
- KisRestClient
- String
- Arc
- BrokerId
- HashMap
- Option
- String
- Value
- Vec
- Client
- Result
- String
- String
- high_tests.rs
- Arc
- package.json
- AtomicUsize
- KisRestClient
- Mutex
- OrderManager
- Self
- StrategyManager
- FnOnce
- Option
- StrategyManager
- String
- Vec
- Arc
- Fixture
- Mutex
- StrategyManager
- Arc
- KisRestClient
- Mutex
- Option
- OrderManager
- RiskManager
- RwLock
- StrategyManager
- String
- Value
- Arc
- BTreeMap
- DateTime
- Drop
- Duration
- F
- Instant
- JoinHandle
- Mutex
- Option
- OrderManager
- StrategyManager
- String
- Utc
- Vec
- Arc
- AtomicUsize
- DateTime
- KisRestClient
- Mutex
- OrderManager
- StrategyManager
- Utc
- Vec
- Arc
- ChartCandle
- F
- KisRestClient
- Option
- String
- Vec
- CandlePageInput
- ChartCandle
- Option
- String
- Vec
- Arc
- KisRestClient
- Mutex
- Option
- OrderManager
- RiskManager
- RwLock
- StrategyManager
- String
- Vec
- CmdResult
- Arc
- CmdResult
- Mutex
- RiskManager
- TradeRecord
- AppHandle
- Arc
- BrokerId
- CmdResult
- Option
- RwLock
- String
- TradingHealthStatus
- Vec
- ClientBuilder
- Arc
- AtomicUsize
- ClientBuilder
- Drop
- Error
- HashMap
- HeaderMap
- JoinHandle
- Json
- Mutex
- Query
- Self
- String
- SupportedProtocolVersion
- Uri
- Value
- Vec
- Option
- Path
- PathBuf
- StockSearchItem
- String
- Vec
- Self
- String
- Box
- CandlePageInput
- Json
- Option
- OrderSide
- OrderType
- Path
- Query
- Response
- String
- Value
- BrokerId
- Json
- Option
- Path
- String
- Value
- Arc
- Body
- HeaderMap
- Next
- Option
- Response
- Router
- Self
- StatusCode
- BrokerId
- ContinuousStrategyPreviewInput
- exits.rs
- indicators.rs
- live.rs
- StatusCode
- StrategyPreviewInput
- String
- Value
- Vec
- DatabaseConfigView
- FnOnce
- Option
- Path
- PathBuf
- kis-auto-trade
- SaveDatabaseConfigInput
- Self
- String
- Duration
- Error
- F
- Option
- Path
- PathBuf
- SaveDatabaseConfigInput
- String
- DatabaseManager
- Error
- Option
- PathBuf
- PgPool
- SaveDatabaseConfigInput
- Self
- String
- Drop
- FnOnce
- Path
- PathBuf
- Self
- MutexGuard
- Option
- String
- Option
- String
- Value
- Vec
- Path
- PathBuf
- T
- Vec
- Arc
- AtomicBool
- DatabaseStatusView
- DatabaseTransferResult
- JsonStorageInventoryView
- Mutex
- Option
- Path
- PathBuf
- PgPool
- RwLock
- StorageBackend
- String
- Vec
- Path
- Self
- PathBuf
- SaveDatabaseConfigInput
- BrokerId
- Into
- Mutex
- NaiveDate
- Option
- PathBuf
- Self
- String
- Vec
- Mutex
- PathBuf
- PendingOrder
- Self
- Vec
- BrokerId
- Mutex
- NaiveDate
- Option
- PathBuf
- Self
- String
- Vec
- Arc
- HashMap
- HashSet
- Into
- JoinHandle
- Mutex
- Option
- OrderManager
- P
- Self
- StrategyManager
- String
- Value
- Vec
- Fixture
- Fixture
- Value
- Fixture
- DateTime
- FixedOffset
- String
- DateTime
- FixedOffset
- TradeGuard
- DateTime
- Default
- FixedOffset
- HashMap
- NaiveDate
- Option
- Self
- String
- Tz
- BrokerId
- Option
- PendingOrder
- BrokerId
- BrokerMarket
- Option
- OrderSide
- PendingOrder
- String
- Vec
- ApproveBudgetRecoveryInput
- Fixture
- OrderManager
- Arc
- HashMap
- KisRestClient
- String
- Vec
- Weak
- Arc
- Fixture
- PendingOrder
- BrokerId
- Item
- Iterator
- Option
- OrderSide
- PendingOrder
- String
- Arc
- KisRestClient
- Option
- String
- Arc
- Mutex
- Self
- StrategyManager
- Arc
- AtomicBool
- Drop
- Fixture
- JoinHandle
- Json
- Mutex
- Self
- StrategyManager
- Value
- Vec
- Arc
- BrokerMarket
- Fixture
- Mutex
- StrategyManager
- Value
- Vec
- BrokerCurrency
- NaiveDate
- Option
- Option
- Arc
- BrokerId
- ExecutedOrder
- KisRestClient
- NaiveDate
- Option
- OverseasExecutedOrder
- String
- Vec
- Option
- PendingOrder
- Option
- PendingOrder
- Arc
- BrokerId
- KisRestClient
- Mutex
- Option
- Self
- String
- Vec
- Arc
- AtomicBool
- AtomicUsize
- Fixture
- HashMap
- HeaderMap
- JoinHandle
- Json
- Notify
- Query
- String
- Value
- NaiveDate
- Option
- String
- Option
- String
- TradingHealthStatus
- Arc
- Mutex
- Option
- P
- Self
- Self
- BrokerMarket
- Arc
- BrokerId
- KisRestClient
- Option
- String
- Weak
- Arc
- AtomicUsize
- Drop
- Fixture
- JoinHandle
- Notify
- Self
- String
- Value
- Option
- OrderResponse
- OrderSide
- String
- BrokerMarket
- BTreeMap
- D
- Debug
- Default
- Error
- Formatter
- Item
- Iterator
- Option
- OrderSide
- S
- Self
- String
- Uuid
- Value
- Vec
- KisRestClient
- Option
- Self
- String
- Fixture
- OrderSide
- String
- Option
- OrderSide
- PendingOrder
- String
- Vec
- Arc
- HashMap
- HashSet
- KisRestClient
- Mutex
- Option
- OrderSide
- RiskManager
- RwLock
- Self
- String
- TradeGuard
- Value
- Weak
- Option
- OrderManager
- Arc
- Mutex
- Option
- OrderManager
- Option
- OrderSide
- RestOrderSide
- String
- Arc
- KisRestClient
- OrderResponse
- Arc
- Option
- OrderResponse
- PendingOrder
- String
- PendingOrder
- Arc
- Box
- From
- Into
- KisRestClient
- Mutex
- Option
- OrderResponse
- OrderSide
- OrderType
- RestOrderSide
- RiskManager
- RwLock
- Self
- Send
- String
- Value
- Arc
- Drop
- KisRestClient
- Mutex
- OrderManager
- PathBuf
- Self
- String
- Default
- HashMap
- I
- Option
- Self
- String
- Uuid
- Vec
- BrokerCurrency
- Option
- String
- Vec
- NaiveDate
- RiskManager
- BTreeMap
- Default
- HashMap
- I
- NaiveDate
- Option
- Self
- String
- Vec
- HashSet
- Item
- Iterator
- String
- Vec
- Option
- SimulationAssumptions
- Option
- Self
- String
- Vec
- Option
- String
- VecDeque
- String
- Vec
- Option
- SimulationAssumptions
- DateTime
- Option
- Utc
- DateTime
- Utc
- Option
- SimulationAssumptions
- Option
- String
- Option
- RiskManager
- Self
- SimulationAssumptions
- String
- TradeGuard
- Vec
- Option
- SimulationAssumptions
- Vec
- Value
- Value
- Value
- Value
- Default
- HashMap
- Option
- Self
- String
- Value
- VecDeque
- EntryFilterPreparation
- VecDeque
- Default
- Option
- Self
- String
- Value
- Option
- String
- Value
- EntryContext
- EntryFilterPreparation
- HashMap
- Option
- Self
- String
- Value
- Value
- Default
- Self
- VolatilityExpansionPreparation
- Default
- Option
- Self
- String
- Value
- Option
- String
- Value
- Default
- HashMap
- Option
- Self
- String
- Value
- VecDeque
- VolatilityExpansionPreparation
- Value
- Default
- Self
- String
- Value
- HashMap
- Option
- Self
- String
- Value
- Option
- VecDeque
- Value
- Default
- HashMap
- Option
- Self
- String
- Value
- VecDeque
- Value
- Value
- Value
- Default
- Self
- String
- Value
- HashMap
- Option
- Self
- String
- Value
- VecDeque
- Value
- Option
- VecDeque
- BrokerId
- BrokerMarket
- Into
- Option
- Self
- Send
- String
- Sync
- Value
- Vec
- Default
- Option
- Self
- String
- Vec
- Vec
- Option
- String
- Option
- Vec
- VecDeque
- Option
- String
- Value
- Option
- String
- Vec
- Counters
- EntryFilterPreparation
- Option
- VecDeque
- Value
- EntryFilterPreparation
- Option
- Value
- Vec
- Default
- Option
- Self
- String
- Value
- EntryContext
- EntryFilterPreparation
- HashMap
- Option
- Self
- String
- VecDeque
- Weak
- BrokerId
- DateTime
- NaiveDate
- Option
- String
- Utc
- Vec
- Arc
- AtomicUsize
- DateTime
- StrategyManager
- String
- Utc
- Vec
- Box
- BrokerId
- BrokerMarket
- BTreeMap
- Default
- F
- HashMap
- HashSet
- Option
- Self
- String
- Value
- Vec
- Weak
- StrategyManager
- StrategyManager
- Value
- Value
- Default
- Self
- String
- Value
- HashMap
- Option
- Self
- String
- Value
- VecDeque
- Option
- VecDeque
- Value
- Value
- HashMap
- Item
- Iterator
- Option
- Self
- String
- Value
- Vec
- VecDeque
- Default
- Option
- Self
- String
- Value
- Option
- String
- Value
- Value
- Option
- String
- Value
- VecDeque
- Value
- HashMap
- Option
- Self
- String
- Value
- Vec
- Counters
- EntryFilterPreparation
- Option
- String
- Value
- VecDeque
- Default
- EntryContext
- EntryFilterPreparation
- HashMap
- Option
- Self
- String
- Value
- VecDeque
- EntryFilterPreparation
- Value
- EntryFilterPreparation
- Option
- VecDeque
- Default
- Option
- Self
- String
- Value
- Option
- String
- Value
- EntryContext
- EntryFilterPreparation
- HashMap
- Option
- Self
- String
- Value
- VecDeque
- EntryFilterPreparation
- Option
- Value
- Value
- Value
- Value
- Into
- Option
- Self
- String
- Value
- Option
- Self
- String
- Vec
- BrokerCurrency
- BrokerMarket
- Option
- String
- BrokerAdapterResult
- DateTime
- From
- Option
- Self
- String
- T
- Utc
- Value
- Vec
- CmdResult
- DatabaseManager
- Default
- NaiveDate
- Option
- Path
- PathBuf
- Self
- String
- Vec
- CmdError
- CmdResult
- DatabaseConfigView
- DatabaseStatusView
- DatabaseTransferResult
- Error
- JsonStorageInventoryView
- SaveDatabaseConfigInput
- StorageBackend
- String
- ChartCandle
- CmdResult
- Option
- OrderResponse
- PriceResponse
- StockSearchItem
- String
- Vec
- CmdResult
- DailyStats
- ExecutedOrder
- Option
- OverseasExecutedOrder
- String
- TradeRecord
- Vec
- BrokerId
- CmdResult
- Option
- PendingOrder
- RiskManager
- String
- Vec
- Arc
- BrokerId
- BrokerMarket
- BrokerMoneyView
- ChartCandle
- CmdError
- CmdResult
- From
- Option
- RwLock
- Self
- String
- Vec
- Arc
- BrokerId
- Option
- Path
- Self
- String
- Vec
- Default
- Option
- Path
- PathBuf
- Self
- String
- Vec
- DateTime
- FixedOffset
- Option
- Self
- String
- Send
- Sync
- Into
- Option
- Self
- String
- DailyStats
- HashMap
- Json
- Option
- Query
- String
- TradeRecord
- Value
- PathBuf
- Self
- String
- Vec
- NaiveDate
- Option
- R
- String
- Vec
- Default
- From
- Option
- Self
- String
- Vec
- Arc
- Mutex
- NaiveDate
- Path
- PathBuf
- T
- Option
- PathBuf
- Self
- Mutex
- NaiveDate
- PathBuf
- Self
- String
- Vec
- Arc
- HashMap
- I
- Option
- Path
- PathBuf
- RwLock
- Self
- StockSearchItem
- String
- Vec
- Mutex
- Path
- PathBuf
- Self
- Vec
- BrokerId
- HashMap
- Option
- String
- Value
- Vec
- Option
- String

## God Nodes (most connected - your core abstractions)
1. `invoke()` - 109 edges
2. `BrokerSymbol` - 95 edges
3. `ServerState` - 63 edges
4. `AppState` - 55 edges
5. `mockApi()` - 55 edges
6. `TossOpenApiClient` - 47 edges
7. `BrokerCandle` - 41 edges
8. `preview_strategy_from_candles()` - 41 edges
9. `KEYS` - 40 edges
10. `BrokerAccountId` - 39 edges

## Surprising Connections (you probably didn't know these)
- `useWriteFrontendLog()` --indirect_call--> `input()`  [INFERRED]
  src/api/logHooks.ts → tests/e2e/uiux-keyboard-selection.spec.ts
- `StrategyAccountPanel()` --indirect_call--> `entries()`  [INFERRED]
  src/pages/strategy/ui/Page.tsx → tests/e2e/uiux-log-status.spec.ts
- `useLeveragedTrendHoldEditorModel()` --indirect_call--> `quantity()`  [INFERRED]
  src/pages/strategy/ui/useLeveragedTrendHoldEditorModel.ts → tests/e2e/trading-form-contract.spec.ts
- `useTradingOrderModel()` --indirect_call--> `qty()`  [INFERRED]
  src/pages/trading/ui/useTradingOrderModel.ts → tests/e2e/uiux-keyboard-selection.spec.ts
- `mockNative()` --indirect_call--> `command()`  [INFERRED]
  tests/e2e/budget-recovery.spec.ts → scripts/database-tls-contract.mjs

## Import Cycles
- None detected.

## Communities (1602 total, 1245 thin omitted)

### Community 0 - "OwnershipLedger"
Cohesion: 0.22
Nodes (29): bars(), completed_252_keeps_final_high_and_uses_high_instead_of_close(), completed_volume_history_is_atomic_scoped_and_does_not_change_owned_protection(), config_aba_and_profile_scope_switch_cannot_revive_old_tickets(), configured_512_window_and_invalid_ohlc_are_not_mislabeled_ready(), consecutive_manager(), consecutive_pending_failure_midnight_and_config_refresh_preserve_held_exit_streak(), consecutive_scoped_daily_apply_preserves_live_streak_and_requires_verified_source() (+21 more)

### Community 1 - "shared/api/commands.ts"
Cohesion: 0.04
Nodes (95): handleLookupTossAccounts(), handleSubmit(), handleSubmit(), activateEmergencyStop(), addProfile(), cancelLthResearch(), checkConfig(), checkForUpdate() (+87 more)

### Community 2 - "rest/types.rs"
Cohesion: 0.07
Nodes (7): FailedBreakoutState, FailedBreakoutStrategy, encode(), restore(), V1, V2, prep()

### Community 3 - "RiskManager"
Cohesion: 0.04
Nodes (76): canUseTauriEvents(), useBackendEvents(), useAutoTradingBudget(), useUpdateAutoTradingBudget(), useKisExecutedByRange(), useOverseasExecutedByRange(), frontendLogger, useWriteFrontendLog() (+68 more)

### Community 4 - "simulation/minute_timing_tests.rs"
Cohesion: 0.38
Nodes (4): DeviationEntryMode, DeviationParams, parse(), validate_deviation_entry_params()

### Community 5 - "TradeRecord"
Cohesion: 0.10
Nodes (28): buy(), checkpoint_rejects_old_run_and_configuration_generation_without_writing(), deviation_feedback_pending_fill_partial_and_restart_preserve_exact_owner_age(), feedback_assigns_only_exact_owner_amid_manual_and_other_account_holdings(), feedback_cold_lth_restores_metadata_and_partial_change_preserves_live_high(), feedback_never_transfers_same_account_quantity_across_profile_or_mode(), key(), manager() (+20 more)

### Community 6 - "FiftyTwoWeekHighStrategy"
Cohesion: 0.10
Nodes (18): BollingerControls(), handlePickerSelect(), handlePickerUsSearch(), LeveragedTrendHoldPreviewPanel(), count(), date(), LthCollectionSummaryPanel(), object() (+10 more)

### Community 7 - "StrongCloseStrategy"
Cohesion: 0.14
Nodes (6): legacy_params(), MomentumEntryMode, MomentumParams, MomentumState, MomentumStrategy, validate_momentum_entry_params()

### Community 8 - "TrendFilterStrategy"
Cohesion: 0.11
Nodes (6): config(), prep(), initializer_restores_previous_averages_for_first_evaluation_cross(), insufficient_history_has_no_previous_long_average(), MaCrossState, MovingAverageCrossStrategy

### Community 9 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.15
Nodes (21): SectorDataValidation, ValidateSectorDataInput, SectorPortfolioInput, symbols, syntheticSectorPortfolioInput(), blockUnsafe(), openPreparation(), requestFixture() (+13 more)

### Community 10 - "OrderManager"
Cohesion: 0.07
Nodes (29): scripts, build, build:app, build:app:debug, build:research, build:web, build:web:research, check:fsd (+21 more)

### Community 11 - "ui/AppShell.tsx"
Cohesion: 0.05
Nodes (42): useRecentLogs(), useUpdateCheck(), SettingsState, useSettingsStore, LEVEL_COLORS, Log(), LogLevel, Strategy() (+34 more)

### Community 12 - "toss/types.rs"
Cohesion: 0.15
Nodes (18): Handshake, RootCertStore, SocketAddr, builder_with_roots(), client(), client_builder(), error_chain(), http_auth_and_order_posts_are_not_resubmitted_after_response_disconnect() (+10 more)

### Community 13 - "toss_market.rs"
Cohesion: 0.08
Nodes (53): SectorDaily, SectorPortfolioResult, SectorRank, SectorTrade, plan_sector_portfolio(), rank_sectors(), return_n(), DefensiveState (+45 more)

### Community 14 - "DatabaseManager"
Cohesion: 0.22
Nodes (4): MySqlPool, DatabaseManager, DatabasePool, safe_export_path()

### Community 15 - "market_hours.rs"
Cohesion: 0.10
Nodes (41): diagnose_minute_timing(), ExecutionClockView, ExecutionDayView, ExecutionExclusionCount, ExecutionExclusionExample, local(), LthMinuteTimingView, MinuteLabelConvention (+33 more)

### Community 16 - "database_io.rs"
Cohesion: 0.08
Nodes (40): useStrategies(), useUpdateStrategy(), hasInvalidLthEntries(), addSymbol(), brokerLabel(), EditState, fmtTossSessionWindow(), getEdit() (+32 more)

### Community 17 - "BrokerAccountId"
Cohesion: 0.15
Nodes (19): TossApiResponse, TossCost, TossCurrencyAmount, TossDailyProfitLoss, TossHoldingsItem, TossHoldingsOverview, TossKrIntegratedHour, TossKrMarketDay (+11 more)

### Community 18 - "TossOpenApiClient"
Cohesion: 0.08
Nodes (6): ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, validate_entry_params(), validate_consecutive_entry_params(), bounded_window_with_extra()

### Community 19 - "observation.rs"
Cohesion: 0.13
Nodes (28): options, useBrokerChartScope(), useCollectBrokerChartData(), usePreviewBrokerChartPortfolio(), collectBrokerChartData(), getBrokerChartScope(), previewBrokerChartPortfolio(), BrokerChartCollection (+20 more)

### Community 20 - "accounts.rs"
Cohesion: 0.18
Nodes (16): buildTree(), compareNames(), currentDocument, fail(), generateInventory(), listRepositoryFiles(), mode, nextDocument (+8 more)

### Community 21 - "history/ui/Page.tsx"
Cohesion: 0.12
Nodes (17): order_side_label(), OrderManager, OrderSubmission, OwnedStrategySignal, PendingOrder, current_position_snapshot(), refresh_manual_positions(), ProviderOrderRequest (+9 more)

### Community 22 - "page_view"
Cohesion: 0.06
Nodes (49): useRefreshStockList(), useStockSearch(), usePreviewLeveragedTrendHold(), ProfileFormState, LeveragedTrendHoldEditorPanel(), boolParam(), EXCHANGE_SEARCH_ORDER, LeveragedTrendHoldEditorPanelProps (+41 more)

### Community 23 - "toss_market.rs"
Cohesion: 0.15
Nodes (15): IntoIterator, StdMutex, applies_retry_after_to_group_pause(), epoch_ms_now(), rate_limit_reset_delay(), RateLimitGroupState, RateLimitGroupStatus, RateLimitScheduler (+7 more)

### Community 24 - "ServerState"
Cohesion: 0.05
Nodes (40): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, Korean, app, security (+32 more)

### Community 26 - "StrategyManager"
Cohesion: 0.08
Nodes (7): MeanReversionStrategy, MeanReversionExitMode, MeanReversionParams, parse(), validate_mean_reversion_params(), MeanReversionState, bounded_window()

### Community 27 - "RunFence"
Cohesion: 0.16
Nodes (20): get_active_toss_calendar_override(), get_toss_chart_data(), get_toss_chart_data_for_profile(), get_toss_market_calendar(), get_toss_market_calendar_for_profile(), get_toss_stock_safety(), get_toss_stock_safety_for_profile(), normalize_toss_chart_time() (+12 more)

### Community 28 - "tauri.conf.json"
Cohesion: 0.16
Nodes (30): addProfileTrigger(), attemptMainClose(), captureProfiles(), clickProfileBackdrop(), currentError(), expectedAddBody(), expectedEditBody(), fakeProfiles (+22 more)

### Community 29 - "prepare"
Cohesion: 0.09
Nodes (20): BalanceItem, BalanceResponse, BalanceSummary, ChartCandle, ExecutedOrder, first_positive_u64(), KisOutput1, KisOutput1<T> (+12 more)

### Community 30 - "strategy/ui/Page.tsx"
Cohesion: 0.10
Nodes (4): OverseasPosition, OverseasPositionTracker, PositionSnapshotVersion, PositionTracker

### Community 31 - "accounts.rs"
Cohesion: 0.09
Nodes (22): useStatsByRange(), useTradesByRange(), usePendingOrders(), FilledOrdersPanel(), fmt(), PendingOrdersPanel(), SortDir, todayStr() (+14 more)

### Community 32 - "RsiStrategy"
Cohesion: 0.14
Nodes (8): BrokerCurrency, Option, Vec, TossOpenApiClient, validate_iso_date(), validate_toss_order_id(), TossCandlePageResponse, TossCredentials

### Community 33 - "settings.rs"
Cohesion: 0.09
Nodes (29): UpdateInfo, AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_trading_type(), DetectTradingTypeResult (+21 more)

### Community 34 - "order/ownership/tests.rs"
Cohesion: 0.11
Nodes (19): KisBrokerAdapter, maps_kis_balance_item_to_broker_holding(), BrokerClientOrderId, BrokerCurrency, BrokerHolding, BrokerId, BrokerMarket, BrokerMoney (+11 more)

### Community 35 - ".submit_order_shared"
Cohesion: 0.07
Nodes (23): config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope(), daily_buy_order_limit_is_disabled_even_after_submission_count_increases(), daily_order_limit_isolated_by_broker_account_scope(), daily_sell_order_limit_blocks_after_submission_count_reaches_limit(), DailyOrderCountEntry, DailyOrderSide (+15 more)

### Community 36 - "strategy_preview.rs"
Cohesion: 0.10
Nodes (3): live_high_excludes_unconfirmed_latest_bar_without_shortening_252_requirement(), owned_target_matches(), StrategyManager

### Community 37 - "server/records.rs"
Cohesion: 0.50
Nodes (3): TossApiError, TossErrorResponse, TossHttpError

### Community 38 - "databaseManagementSection.tsx"
Cohesion: 0.12
Nodes (26): automatic_missing_context_and_stale_reservation_leave_budget_and_pending_unchanged(), buy(), delayed_old_account_insufficient_balance_does_not_suspend_new_account(), legacy_configured_account_without_profile_allows_manual_but_fences_client_replacement(), local_holdings(), local_order(), local_single_order(), LocalProvider (+18 more)

### Community 39 - "DatabaseManager"
Cohesion: 0.12
Nodes (18): account_sync_error(), chart_handler(), ChartQuery, invalid_order_input(), order_handler(), OrderBody, overseas_chart_handler(), overseas_order_handler() (+10 more)

### Community 40 - "AppState"
Cohesion: 0.19
Nodes (8): AtomicU64, configuration_save_rollback_aba_invalidates_quote_and_position_feedback(), delayed_name_and_late_rollback_do_not_mutate_the_new_run(), delayed_quote_cannot_tick_a_restarted_same_scope_run(), Fixture, Probe, stale_balance_cannot_clear_or_add_new_run_health_failures(), unpublished_revision_or_stopped_flag_fail_closed_but_current_tick_runs()

### Community 41 - "DatabaseConfig"
Cohesion: 0.24
Nodes (12): MySqlSslMode, PgSslMode, cached_connection_snapshot_reports_current_matching_settings(), concurrent_first_connections_create_one_pool(), connect_pool(), connecting_old_target_is_closed_and_not_cached_after_config_save(), create_missing_mariadb_database(), create_missing_postgres_database() (+4 more)

### Community 42 - "MomentumStrategy"
Cohesion: 0.11
Nodes (74): run_backtest(), run_backtest_with_history(), assumptions(), bar(), buy(), directional_fees_and_native_minimum_are_converted_with_the_current_fx(), dividend_adjusted_prices_reject_known_double_counting(), dividend_entitlement_precedes_pending_open_and_respects_partial_exit_quantity() (+66 more)

### Community 43 - "databaseManagementSection.tsx"
Cohesion: 0.10
Nodes (29): useClearDatabaseTables(), useCreateDatabaseTables(), useDatabaseConfig(), useDropDatabaseTables(), useExportDatabaseToJson(), useImportJsonToDatabase(), useJsonStorageInventory(), useSaveDatabaseConfig() (+21 more)

### Community 45 - "commands/records.rs"
Cohesion: 0.16
Nodes (18): calendar_override_represents_holiday_without_fallback(), calendar_session_window_checks_time_range(), is_krx_open(), is_krx_open_at(), is_market_open_for(), is_market_open_for_with_calendar(), is_market_open_for_with_calendar_policy(), is_us_open() (+10 more)

### Community 46 - "PriceConditionStrategy"
Cohesion: 0.19
Nodes (21): assert_failed_save_preserves_pair(), cancelled_save_before_memory_publication_blocks_further_connections(), clear_recovery_marker(), committed_file_without_memory_publication_stays_fail_closed(), create_recovery_marker(), DatabaseManager, ensure_recovered(), failed_save_keeps_live_and_restarted_endpoint_credential_pair() (+13 more)

### Community 47 - "DiscordNotifier"
Cohesion: 0.14
Nodes (14): account_less_scope_matches_any_account_of_same_broker(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), scope(), scoped_record_matches_only_same_broker_account(), TradeMarket (+6 more)

### Community 48 - "autoTradingBudgetSection.tsx"
Cohesion: 0.07
Nodes (49): useCheckConfig(), useTodayStats(), useExchangeRate(), useExchangeRateStatus(), useSubmitTossSmallBuyVerification(), useRefreshInterval(), useActivateEmergencyStop(), useClearBuySuspension() (+41 more)

### Community 49 - "FailedBreakoutStrategy"
Cohesion: 0.22
Nodes (28): age_keeps_accumulating_when_option_off_and_survives_warmup_parameter_edits(), age_starts_at_actual_fill_and_counts_only_owned_evaluations(), buy(), config(), crossing_at_boundary_requires_previous_strictly_above(), disabled_unknown_and_other_symbol_observations_cannot_change_held_age(), downward_crossing_retries_zero_feedback_and_cancels_above_threshold(), filled() (+20 more)

### Community 50 - "PositionTracker"
Cohesion: 0.15
Nodes (11): Snapshot, current_equity(), krw_to_display(), phase_for_index(), round_metric(), round_money(), units_to_display(), units_to_krw() (+3 more)

### Community 51 - "ReplaySession"
Cohesion: 0.13
Nodes (41): align_accounting_scenarios(), broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc() (+33 more)

### Community 52 - "leveraged_trend_hold.rs"
Cohesion: 0.11
Nodes (6): config(), FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy, FiftyTwoWeekState, strong_close_ready_is_distinct_from_condition_and_preserves_positions(), volatility_requires_full_range_window_and_preserves_position()

### Community 53 - "AppState"
Cohesion: 0.22
Nodes (24): bar(), buy(), buy_blocked(), config(), daily(), sell(), strategy(), volatility_expansion_actual_fill_only_and_remaining_sell_reoffers() (+16 more)

### Community 54 - "security.rs"
Cohesion: 0.35
Nodes (17): FetchFuture, Fn, at_most_two_jobs_run_and_other_instruments_are_not_starved(), completed(), failures_use_backoff_and_do_not_retry_on_every_quote_tick(), history(), insufficient_history_reports_real_count_reason_and_retry(), maintain_captures_the_new_scope_atomically_after_a_waiting_start() (+9 more)

### Community 55 - "VolatilityExpansionStrategy"
Cohesion: 0.13
Nodes (6): AccountProfile, DiscordConfig, load_secure_config(), non_empty(), ProfilesConfig, SecureConfig

### Community 56 - "simulation.rs"
Cohesion: 0.12
Nodes (46): conflicting_overlap_keeps_only_validated_cohort_and_last_admission(), real_200_regular_80_afterhours_exact_statistics_and_bounds(), adapter_reported_cursor_stall_cannot_be_accepted_as_complete(), context(), cursor(), daily(), daily_context_is_strictly_earlier_and_can_use_a_later_page(), exhaustion_empty_no_progress_partial_error_and_repeated_cursor_are_explicit() (+38 more)

### Community 57 - "BrokerSymbol"
Cohesion: 0.08
Nodes (23): Display, BrokerAdapterResult, BrokerId, Debug, Default, Formatter, Into, Option (+15 more)

### Community 58 - "order/budget.rs"
Cohesion: 0.26
Nodes (13): BudgetCurrencyView, AutoTradingBudgetView, budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent(), cost(), legacy_funded_ledger_restart_preserves_ownership_and_sell_reservation(), manager(), OrderManager, pending() (+5 more)

### Community 59 - "OrderManager"
Cohesion: 0.15
Nodes (11): Barrier, daily_order_limit_key(), estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), OrderManager, order_exchange_code(), OrderManager (+3 more)

### Community 60 - "OrderRecord"
Cohesion: 0.10
Nodes (6): live_feedback_price_condition_changes_position_only_after_confirmed_fill(), strategy(), PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick()

### Community 61 - "settings/ui/Page.tsx"
Cohesion: 0.07
Nodes (40): useSetTradeArchiveConfig(), useTradeArchiveConfig(), useTradeArchiveStats(), useLogConfig(), useSetLogConfig(), useSetStockUpdateInterval(), useStockListStats(), useBrokerRateLimitStatus() (+32 more)

### Community 62 - "run_backtest"
Cohesion: 0.11
Nodes (31): lth_default_adx_period(), lth_default_breakeven_buffer(), lth_default_buy_adx(), lth_default_buy_rsi(), lth_default_ema_long(), lth_default_ema_short(), lth_default_entry_end(), lth_default_entry_failure_observations() (+23 more)

### Community 63 - "DatabasePool"
Cohesion: 0.12
Nodes (13): OverseasPriceResponse, KisRestClient, AtomicBool, ExecutedOrder, HeaderMap, OrderResponse, OverseasExecutedOrder, PriceResponse (+5 more)

### Community 64 - "TradeRecord"
Cohesion: 0.14
Nodes (37): request(), ConfigurationEditReservation, ConfigurationUpdateError, ConfigurationUpdateRequest, ConfigurationUpdateState, effective_target_markets(), run_update(), spawn_update() (+29 more)

### Community 65 - "server/trading.rs"
Cohesion: 0.17
Nodes (31): BrokerAdapter, Send, Sync, Admission, Budget, collect(), collect_with_summary(), Context (+23 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.12
Nodes (25): assert_auth(), broker_chart_daily_response_keeps_read_and_size_errors_fixed(), candle_values(), chart_response_handles_empty_output_and_malformed_multibyte_body(), chart_response_preserves_http_and_provider_errors(), chart_response_propagates_announced_and_observed_body_caps(), domestic_latest_first_fixture_reverses_and_maps_ohlcv(), domestic_request_keeps_date_bounds_period_adjustment_and_query_order() (+17 more)

### Community 67 - "scripts"
Cohesion: 0.12
Nodes (44): activate_emergency_handler(), app_config_handler(), check_config_handler(), check_update_handler(), clear_buy_suspension_handler(), clear_emergency_handler(), exchange_rate_handler(), exchange_rate_status_handler() (+36 more)

### Community 68 - "build_strategy"
Cohesion: 0.08
Nodes (40): candle_market_date(), BrokerMarket, ToString, CandlePageInput, CandlePageView, completed_before(), decode_cursor(), cursor_default_count_and_unhashed_credentials_keep_the_fixed_identity() (+32 more)

### Community 69 - "commands/toss.rs"
Cohesion: 0.27
Nodes (12): Exclusion, kst_minute(), label_clock(), regular_clock(), RegularClock, DateTime, NaiveDate, Option (+4 more)

### Community 70 - "strategyPolicyControls.tsx"
Cohesion: 0.06
Nodes (67): DeviationControls(), DeviationNumericField(), deviationPolicyError(), DeviationPolicySummary(), fieldError(), fields, isDeviationPolicyParam(), modeOf() (+59 more)

### Community 71 - "commands/risk.rs"
Cohesion: 0.22
Nodes (7): PendingCandidate, pending_order_provider(), OrderManager, validate_pending_kis_mode(), OrderManager, ReconciliationSnapshot, same_profile()

### Community 72 - ".new"
Cohesion: 0.14
Nodes (25): AppConfigView, OverseasBalanceItem, OverseasBalanceSummary, ProfileView, add_profile(), AddProfileInput, apply_active_profile(), BalanceResult (+17 more)

### Community 73 - "strategyPreviewPanel.tsx"
Cohesion: 0.11
Nodes (32): usePreviewStrategy(), usePreviewStrategyContinuous(), assertPageScope(), calendarLabel(), CandleCollectionEnd, candleTimestamp(), checkCancelled(), collectContinuousCandles() (+24 more)

### Community 74 - "KisWebSocketClient"
Cohesion: 0.08
Nodes (20): FormatTime, FrontendLogInput, get_recent_logs(), get_stats_by_range(), get_trades_by_range(), GetStatsByRangeInput, GetTradesByRangeInput, save_trade() (+12 more)

### Community 75 - "OhlcCandle"
Cohesion: 0.30
Nodes (21): age_starts_after_positive_fill_and_survives_same_partial_and_warmup_feedback(), buy(), config(), current_inclusive_population_bands_and_strict_boundaries_are_preserved(), disabled_unknown_symbol_and_reset_preserve_observation_contract(), exact_metadata_roundtrips_pending_and_confirmed_age_without_inventing_ownership(), filled(), hold() (+13 more)

### Community 76 - "read_json_or_default"
Cohesion: 0.14
Nodes (23): Make, CancelAck, End, error(), Handle, Lease, LthResearchRuntime, OwnedWork<'_, F> (+15 more)

### Community 77 - "database_projection.rs"
Cohesion: 0.11
Nodes (34): MySql, Postgres, Transaction, drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract() (+26 more)

### Community 78 - "ConsecutiveMoveStrategy"
Cohesion: 0.16
Nodes (13): classify_token_response(), detect_trading_type(), DetectedTradingType, DetectTokenReq, DetectTradingTypeError, is_paper_key_rejected_by_real_domain(), is_real_key_rejected_by_paper_domain(), mentions_app_key() (+5 more)

### Community 79 - "project-map.mjs"
Cohesion: 0.20
Nodes (15): ChartRequest, ChartRequest<'a>, KisRestClient, parse_broker_chart_daily_body(), parse_broker_chart_daily_response(), parse_domestic_response(), parse_overseas_response(), parse_sector_daily_body() (+7 more)

### Community 80 - "start_trading"
Cohesion: 0.30
Nodes (19): preview_strategy_continuous_from_candles(), actual_indicator_warmup_and_state_survive_more_than_500_bars_and_arbitrary_partitions(), actual_strategy_single_and_chunk_supply_match_and_pending_orders_cross_both_boundaries(), bounded_request_rejects_empty_or_oversized_chunks_history_and_total(), candle(), changing_future_chunk_prices_preserves_all_earlier_strategy_fills_and_equity(), common_phase_diagnostics_and_warning_match_flat_single_and_partitioned_replays(), consecutive_realized_losses_preserve_the_strategy_symbol_block_across_chunks() (+11 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.18
Nodes (16): collect_kis_pending_fills(), collect_toss_pending_fills(), domestic_order_is_terminal(), domestic_order_terminal_status(), domestic_provider_status(), kis_pending_from(), normalized_execution_u64(), execution_quantity() (+8 more)

### Community 82 - "MovingAverageCrossStrategy"
Cohesion: 0.09
Nodes (10): parse(), RsiParams, validate_rsi_params(), RsiState, RsiStrategy, integer(), parse(), percent() (+2 more)

### Community 83 - "accountHooks.ts"
Cohesion: 0.07
Nodes (50): react, AccountConfigRefreshError, accountScope(), holdingsPrefixes, profileMutationKey, profileMutationScope, reconcileRejectedProfileMutation(), refreshAccountConfig() (+42 more)

### Community 84 - "shared/api/types.ts"
Cohesion: 0.20
Nodes (19): checkpoint_validation_is_atomic_and_kind_separated(), cold_owned_age_stays_unknown_and_active_cap_offers_conservative_exit(), config(), damaged_optional_runtime_fails_buy_closed_and_retains_custom_core_and_valid_exits(), disabled_cap_still_counts_known_age_and_reconfigure_does_not_reset_it(), entry(), flat_sell_and_held_buy_are_removed_without_mutating_actual_position(), known_age_starts_after_actual_fill_and_preserves_partial_same_quantity_retries() (+11 more)

### Community 85 - "strategyResearchPanel.tsx"
Cohesion: 0.09
Nodes (34): clearExperimentSlots(), compactSnapshot(), ExperimentScope, loadExperimentSlots(), normalizePart(), readIndex(), saveExperimentSlot(), storageKey() (+26 more)

### Community 86 - "collect_kis_pending_fills"
Cohesion: 0.05
Nodes (13): DeviationStrategy, capacity(), DeviationState, advance_owned_age(), age_preserves_pending_partial_same_quantity_and_unknown_origins(), begin_live_trend(), both_kinds_preserve_exact_checkpoint_shape_and_pending_fill_origin(), checkpoints_reject_cross_kind_unknown_fields_and_contradictions() (+5 more)

### Community 87 - "support.rs"
Cohesion: 0.22
Nodes (16): checked_quantity_cost_and_signed_profit_overflows_are_atomic(), cumulative_buys_and_sells_track_native_weighted_cost_and_proportional_profit(), cumulative_progress_replaces_only_the_latest_durable_application_result(), duplicate_and_regressing_cumulative_reports_are_noops_but_contradictions_fail(), exact_account_profile_mode_owner_market_and_symbol_are_separate_and_orders_cannot_rebind(), fill(), invalid_oversized_unowned_and_duplicate_metadata_fail_atomically_and_on_restore(), key() (+8 more)

### Community 88 - "strategy_with_params"
Cohesion: 0.10
Nodes (40): LeveragedTrendHoldEntry, LeveragedTrendHoldStrategy, average_fill_price_drives_preview_stop_even_when_observation_price_rises(), daily_context_cannot_prepare_minute_indicators_or_rebound(), daily_reinitialization_keeps_intraday_bollinger_and_actual_position(), failed_buy_feedback_does_not_create_phantom_position(), failed_sell_feedback_keeps_actual_position_for_next_risk_exit(), intraday_and_daily_context_buffers_have_independent_bounds() (+32 more)

### Community 89 - "HoldingsQuery"
Cohesion: 0.24
Nodes (7): DomesticHoldings, OverseasHoldings, ExpectedSource, HoldingsMarkets, HoldingsQuery, HoldingsUpdate, OrderManager

### Community 91 - "TradeStore"
Cohesion: 0.09
Nodes (15): Deserialize, encode(), Protection, restore(), VolatilityExpansionState, DateRangeQuery, frontend_log_handler(), FrontendLogBody (+7 more)

### Community 92 - "strategyResearchTypes.ts"
Cohesion: 0.11
Nodes (16): BacktestAtrPreparation, BacktestReport, ReplayAccounting, ReplayComparison, SimulationAssumptions, accounting, card(), choose() (+8 more)

### Community 93 - "@mui/icons-material"
Cohesion: 0.44
Nodes (10): order_side_label(), pending_conflict_blocks_opposite_side_in_same_scope(), pending_conflict_blocks_same_side_in_same_scope(), pending_conflict_reason_for_scope(), pending_conflict_scan_ignores_different_scope(), pending_order(), pending_order_conflict_reason(), pending_order_provider_uses_provider_trace() (+2 more)

### Community 94 - "balance_store.rs"
Cohesion: 0.07
Nodes (43): Ok, fixture(), fixture_with_base(), ownership_fill_empty_sell_snapshot_uses_exact_owner_cost_across_partial_and_repeat(), ownership_fill_exact_native_notional_preserves_301_total_and_partial_remainder(), ownership_fill_larger_terminal_cumulative_waits_for_earlier_intent_then_applies_every_share(), ownership_fill_legacy_applying_intent_requires_durable_date_fx_and_notional(), ownership_fill_legacy_sell_does_not_adopt_other_owners_account_average() (+35 more)

### Community 95 - "LeveragedTrendHoldParams"
Cohesion: 0.27
Nodes (17): bar(), canonical(), input(), near(), partial_bars(), partition(), pc_configured_sell_request_is_preserved_when_actual_fill_is_smaller(), pc_native_and_overlay_reports_signals_and_hash_are_invariant_under_continuous_partitions() (+9 more)

### Community 96 - "check"
Cohesion: 0.08
Nodes (23): DOM, DOM.Iterable, ES2020, src, compilerOptions, allowImportingTsExtensions, isolatedModules, jsx (+15 more)

### Community 97 - "compilerOptions"
Cohesion: 0.15
Nodes (12): KrxItem, KrxProxyItem, KrxProxySearchResponse, KrxResponse, NaverAcItem, NaverAcResponse, save_cache(), StockList (+4 more)

### Community 98 - "holdings/tests.rs"
Cohesion: 0.29
Nodes (17): HoldingsUpdate, a_change_to_either_market_rejects_both_without_partial_publication(), configuration_lease_rejects_capture_and_inflight_response_even_after_release(), domestic(), empty_and_zero_holdings_clear_only_the_queried_market(), fixture(), held(), identity_changes_reject_without_mutating_either_tracker() (+9 more)

### Community 99 - "MeanReversionStrategy"
Cohesion: 0.19
Nodes (15): add_profile_handler(), AddProfileBody, apply_profile_change(), delete_profile_handler(), DeleteProfileBody, detect_profile_handler(), detect_trading_type_handler(), DetectTradingTypeBody (+7 more)

### Community 100 - "verify-toss-openapi.mjs"
Cohesion: 0.10
Nodes (19): devDependencies, @playwright/test, @tauri-apps/cli, @types/node, @types/react, @types/react-dom, @types/react-grid-layout, typescript (+11 more)

### Community 101 - "OverseasExecutedOrder"
Cohesion: 0.13
Nodes (20): buildChartOptions(), CandlePoint, CHART_PRESETS, ChartPreset, ChartSource, ChartType, CrosshairData, formatChartTime() (+12 more)

### Community 102 - "BrokerAccountId"
Cohesion: 0.15
Nodes (17): args, assertGitClean(), assertTagAvailable(), branchName, changedFiles, currentBranchName(), escapeRegExp(), fail() (+9 more)

### Community 103 - "TossAccessToken"
Cohesion: 0.16
Nodes (11): assertDialogScope(), currentScope(), ExpectedMutation, focusedOpener(), ScopeSnapshot, setup(), snapshot(), start() (+3 more)

### Community 104 - "dependencies"
Cohesion: 0.10
Nodes (21): @emotion/react, @emotion/styled, lightweight-charts, @mui/icons-material, @mui/material, dependencies, @emotion/react, @emotion/styled (+13 more)

### Community 105 - "model/tradingStore.ts"
Cohesion: 0.19
Nodes (10): LeveragedRapidReboundSnapshot, LeveragedReboundSnapshot, lth_default_qty(), LeveragedTrendHoldStrategy, PreviewPosition, F, LeveragedTrendHoldPreviewSignal, Option (+2 more)

### Community 106 - "TossApiError"
Cohesion: 0.19
Nodes (11): AccessToken, kis_token_client_never_forwards_post_body_to_redirect_destination(), Arc, DateTime, Mutex, Option, Self, String (+3 more)

### Community 107 - "LeveragedTrendHoldStrategy"
Cohesion: 0.10
Nodes (10): config(), daily(), encode(), Protection, restore(), TrendFilterState, TrendFilterStrategy, warmup_mean_reversion_uses_close_bands_and_first_signal() (+2 more)

### Community 108 - "model/accountStore.ts"
Cohesion: 0.17
Nodes (24): SectorPrice, DefensiveState, BTreeMap, Option, SectorCorporateAction, SectorDay, String, Vec (+16 more)

### Community 109 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.10
Nodes (15): validates_order_create_request_shape(), TossOrderCreateRequest, market_from_currency(), new_toss_client_order_id(), toss_currency(), toss_market(), validate_client_order_id(), validate_optional_decimal() (+7 more)

### Community 110 - "NotificationService"
Cohesion: 0.29
Nodes (8): StrategyView, get_positions(), get_strategies(), PositionView, update_strategy(), UpdateStrategyInput, build_strategy_view(), StrategyView

### Community 111 - "exchange.rs"
Cohesion: 0.20
Nodes (16): BacktestPhaseView, BacktestReportView, BacktestSummaryView, BacktestTradeView, EquityPointView, overfit_warning(), percent(), phase_summaries() (+8 more)

### Community 112 - "toss_currency_from_view"
Cohesion: 0.17
Nodes (30): LthResearchHandle, active_profile(), leveraged_trend_hold_preview_handler(), lth_research_cancel_handler(), lth_research_reserve_handler(), lth_research_run_handler(), research_response(), CmdError (+22 more)

### Community 113 - "strategy/tests.rs"
Cohesion: 0.13
Nodes (38): assert_private(), body_limit_preserves_header_stream_and_exact_boundary(), broker_chart_toss_raw_provider_reuses_token_and_sends_quote_only_contract(), candle_source_known_bytes_through_adapter(), candle_source_raw_identity_chunking_precision_and_legacy_projection(), candle_sourced_failures_never_return_metadata_or_empty_success(), candle_sourced_refresh_fingerprints_final_200_not_401_body(), candles() (+30 more)

### Community 114 - "day_event_tests.rs"
Cohesion: 0.14
Nodes (21): CollectSectorDataInput, check_scope(), collect_sector_data(), error(), get_sector_quote_scope(), NativeQuoteSource, QuoteSnapshot, Arc (+13 more)

### Community 115 - "setup-local.sh"
Cohesion: 0.21
Nodes (5): OrderRecord, OrderSide, OrderStatus, OrderStore, parallel_appends_do_not_lose_order_records()

### Community 116 - "Signal"
Cohesion: 0.10
Nodes (27): RecentSideHistory, ScopedSymbol, blocks_reentry_after_stop_loss_sell_on_same_day(), blocks_repeated_same_side_signal_inside_cooldown(), blocks_tiny_profit_after_costs(), blocks_whipsaw_after_repeated_opposite_signals(), buy_signal(), cooldown_is_isolated_by_broker_account_scope() (+19 more)

### Community 117 - "main.rs"
Cohesion: 0.27
Nodes (17): bar(), fill(), seed(), strategy(), strong_close_actual_quantity_average_partial_stop_zero_and_retry(), strong_close_age_unknown_disabled_partial_and_protection_priority(), strong_close_dated_same_day_failure_late_legacy_and_new_day(), strong_close_invalid_optional_diagnostics_count_unready_without_trend() (+9 more)

### Community 118 - "volatility_expansion/tests.rs"
Cohesion: 0.18
Nodes (5): atr14(), mean_true_range14(), ConsecutiveMoveParams, Counters, EntryContext

### Community 119 - "generic_tests.rs"
Cohesion: 0.12
Nodes (21): Fake, BrokerCandlePage, BrokerCandlePageWithSource, CandleCursor, CandlePageRequest, kis_cursor_date(), next_toss_cursor(), previous_kis_cursor() (+13 more)

### Community 120 - "preview_strategy_continuous_from_candles"
Cohesion: 0.24
Nodes (15): recovered_native_stop_guard_survives_full_fill_flat_restart_and_budget_load(), recovery_binding_restart_partial_reverse_terminal_and_reapproval_are_idempotent(), recovery_history_corrupt_primary_cannot_fall_back_to_empty_backup(), recovery_intent_survives_pending_write_failure_and_does_not_clear_block(), recovery_intent_write_failure_never_publishes_pending(), recovery_list_hides_normal_pending_and_prioritizes_unresolved_over_completed_receipts(), recovery_missing_history_primary_with_backup_or_quarantine_fails_closed(), recovery_never_invents_legacy_or_partially_applied_pending() (+7 more)

### Community 121 - "failed_breakout/tests.rs"
Cohesion: 0.29
Nodes (16): atr_includes_gaps_but_excludes_candidate_ohlc_and_completion_is_once(), bar(), combined_threshold_rejections_are_independent_and_require_all_filters(), completed_daily_context_remains_bounded_at_512_and_has_no_current_price_duplicates(), config(), entry_atr_retains_fractional_true_range_average_and_inclusive_threshold(), huge_legacy_move_counts_do_not_overflow_or_grow_unbounded_buffers(), invalid_optional_fields_fail_closed_without_replacing_custom_held_exit_parameters() (+8 more)

### Community 122 - "discord-notification-config/index.ts"
Cohesion: 0.23
Nodes (11): DatabaseConfig, DatabaseConfigView, DatabaseProvider, DatabaseStatusView, DatabaseTableStatus, DatabaseTlsMode, DatabaseTransferResult, JsonStorageCategoryView (+3 more)

### Community 123 - "manager"
Cohesion: 0.19
Nodes (6): TossOrder, TossOrderExecution, TossOrderListQuery, TossOrderListStatus, TossOrderModifyRequest, TossPaginatedOrderResponse

### Community 124 - "strategy-configure/index.ts"
Cohesion: 0.15
Nodes (3): DiscordNotifier, NotificationEvent, NotificationLevel

### Community 125 - "generic.rs"
Cohesion: 0.16
Nodes (5): calculate_domestic_fee(), calculate_overseas_fee_cents(), cumulative_fill_delta(), OrderManager, PendingFill

### Community 126 - "Result"
Cohesion: 0.31
Nodes (17): bounded_history_and_future_suffix_cannot_change_past_signals_or_state(), buy(), config(), crossing(), crossing_cancellation_requires_full_hysteresis_reset_before_reentry(), crossing_inclusive_entry_retries_after_equal_zero_sync_and_consumes_only_real_fill(), invalid_optional_fields_preserve_custom_legacy_exit_and_fail_closed_entries(), level_hysteresis_is_inert_but_range_is_validated_for_every_mode() (+9 more)

### Community 127 - "PendingOrderStore"
Cohesion: 0.27
Nodes (12): collect_trade_archive_stats(), collect_trade_archive_stats_active(), collect_trade_day_dirs(), dir_size_bytes(), get_trade_archive_config(), get_trade_archive_stats(), purge_old_trade_files(), purge_trade_archive_active() (+4 more)

### Community 128 - "conflicts.rs"
Cohesion: 0.20
Nodes (12): parse_amount_f64(), fetch_account_risk_balance_krw(), fetch_toss_risk_balance_krw(), clear_buy_suspension(), get_trading_status(), sync_kis_strategy_positions(), sync_strategy_positions_from_active_broker(), sync_toss_strategy_positions() (+4 more)

### Community 129 - "MarketDayCalendar"
Cohesion: 0.73
Nodes (5): check_toss_profile_connection(), run_toss_connection_diagnostic(), toss_diag_step(), TossConnectionDiagnostic, TossConnectionStep

### Community 130 - "DailyHistoryTicket"
Cohesion: 0.16
Nodes (26): LeveragedTrendHoldPreviewView, cancel_lth_research(), cancel_lth_research_owned(), LthResearchCancelView, LthResearchHandle, LthResearchReserveInput, rejected(), reserve_lth_research() (+18 more)

### Community 131 - "validate_pending_kis_mode"
Cohesion: 0.07
Nodes (64): chart_volume_to_u64(), blocked_strong_close_buy_does_not_discard_next_days_completed_condition(), candle(), daily_event_strategies_reject_other_cadences(), flat_completed_days_fill_history_without_compressing_the_window(), input(), strong_close_uses_previous_completed_day_and_keeps_position_across_day_changes(), volatility_uses_previous_days_rolling_ranges_and_independent_today_ohlc() (+56 more)

### Community 132 - "BrokerScope"
Cohesion: 0.29
Nodes (13): fetch_overseas_tick(), fetch_toss_tick(), finish_closed_batch(), market_readiness(), MarketReadiness, is_market_closed_error(), poll_symbols_tick(), PriceSource (+5 more)

### Community 133 - ".search"
Cohesion: 0.18
Nodes (13): build_router(), Router, app(), bearer_token(), constant_time_eq(), cross_origin_mutation_is_rejected_even_with_token(), has_same_origin(), mutation_without_token_is_rejected() (+5 more)

### Community 134 - "run_backtest_with_history"
Cohesion: 0.22
Nodes (3): mean_close(), Counters, EntryContext

### Community 135 - "AppConfigView"
Cohesion: 0.16
Nodes (14): LeveragedTrendHoldEntry, LeveragedTrendHoldMarketState, LeveragedTrendHoldParams, LeveragedTrendHoldPosition, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldStrategy, Default, HashMap (+6 more)

### Community 136 - "작업 기록: <짧은 제목>"
Cohesion: 0.36
Nodes (13): bar(), config(), context_refresh_failure_and_readiness_queries_never_reseed_cross_observations(), cross_state(), current_completed_daily_arrives_after_decision_and_duplicates_are_ignored(), disabled_filters_keep_golden_death_cross_quantity_and_metadata(), malformed_optional_does_not_replace_custom_legacy_periods_or_sell(), minute_seed_and_live_quotes_never_manufacture_completed_daily_context() (+5 more)

### Community 137 - "StatsStore"
Cohesion: 0.09
Nodes (22): fixture, realm(), DailyStats, mockSettingsApi(), apply(), Control, dates(), expectZero() (+14 more)

### Community 138 - ".process_buy"
Cohesion: 0.17
Nodes (15): lth_research_production_routes_reserve_scope_cancel_and_reuse_without_provider(), lth_research_unknown_cancel_is_an_error_without_provider_dispatch(), production_kis_routes_preserve_capture_query_headers_and_response_mapping(), provider_http_failure_keeps_the_legacy_200_error_envelope(), toss_and_control_routes_keep_safe_errors_without_provider_or_engine_start(), json_body(), Value, application_error_status_and_envelopes_remain_distinct() (+7 more)

### Community 139 - "CurrencyLedger"
Cohesion: 0.15
Nodes (22): actual_pipeline_serializes_success_summary_camel_case_without_certification(), collection_failure_preserves_error_and_stops_io_without_success_view(), Fake, fetch_leveraged_trend_hold_candles(), fetch_outcome(), instant(), legacy_pipeline_matches_before_fixed_hash_and_backtest(), legacy_pipeline_tuple_projection_order_errors_and_replay_hash_are_preserved() (+14 more)

### Community 140 - "verify-lockfiles.mjs"
Cohesion: 0.19
Nodes (19): Ready, actual_preview_entry_prepare_guards_and_two_shared_arc_calls_are_io_zero(), cancel_before_run_terminal_and_scope_aba_do_not_recreate_old_jobs(), factory_panic_and_running_work_panic_release_slot(), failure_receipt_rejects_nonterminal_unknown_wrong_owner_and_mode(), final_publication_rechecks_deadline_even_for_ready_work(), fixture(), input_for() (+11 more)

### Community 141 - "modify_toss_order_for_profile"
Cohesion: 0.32
Nodes (13): broker_money_view_amount(), confirmed_small_order_fill(), ConfirmedSmallOrderFill, is_toss_order_final(), money_view_from_decimal(), poll_toss_order_detail(), small_buy_cap(), storage_money_amount() (+5 more)

### Community 142 - "deviation/tests.rs"
Cohesion: 0.31
Nodes (13): absent_unknown_and_similarly_prefixed_kis_tr_ids_preserve_pending_without_querying(), actual_fill_crosses_same_scope_restart_with_identical_provider_snapshot(), delayed_reconciliation_cannot_mutate_new_scope_and_applies_when_original_is_reselected(), direct_fill_and_zero_fill_terminal_cannot_mutate_another_execution_scope(), exact_eight_order_tr_ids_select_mode_and_toss_bypasses_kis_mode_classification(), executed(), persisted_kis_pending_never_queries_or_applies_in_the_opposite_mode(), Provider (+5 more)

### Community 143 - "HARNESS_MAP.md"
Cohesion: 0.27
Nodes (5): apply_slippage(), bps_cost(), SimulationAssumptions, ReplaySession, SaleProjection

### Community 144 - "2. 목표와 비목표"
Cohesion: 0.25
Nodes (12): fieldError(), fields, isStrongClosePolicyParam(), numericLabel(), profitPairError(), StrongCloseControls(), StrongCloseNumericField(), strongClosePolicyError() (+4 more)

### Community 145 - "react-grid-layout"
Cohesion: 0.18
Nodes (12): buildSnapshot(), fail(), listRepositoryFiles(), mode, parseMode(), REPOSITORY_ROOT, ROOT_CONFIGS, SCRIPT_DIR (+4 more)

### Community 146 - "api/mod.rs"
Cohesion: 0.23
Nodes (9): Band, BollingerParams, candle(), disabled_option_preserves_entries_and_buffer_is_bounded(), downward_breakout_and_insufficient_history_block_entries(), flat_squeeze_needs_completed_upward_breakout(), LeveragedTrendHoldStrategy, live_and_replay_share_confirmed_breakout_and_exit_without_future_prices() (+1 more)

### Community 148 - "Arc"
Cohesion: 0.25
Nodes (13): bodyHtml(), checkFreshness(), escapeHtml(), inlineMarkdown(), outputPath, parseTasks(), renderTodo(), root (+5 more)

### Community 149 - "AtomicBool"
Cohesion: 0.19
Nodes (3): StockEntry, StockListStats, StockStore

### Community 150 - "ChartCandle"
Cohesion: 0.21
Nodes (10): Requests, accept(), actual_submissions_sell_only_the_confirmed_owner_and_never_add_fill_to_account_snapshot(), ownership_recheck_after_preparation_prevents_a_stale_sell_reservation(), publish(), Fixture, Json, OrderSide (+2 more)

### Community 151 - "restore_risk_from_today_trades"
Cohesion: 0.19
Nodes (8): kis_read_min_interval(), KisOrderRejected, Arc, Duration, RwLock, Self, String, kis_rejection_keeps_fixed_message_and_typed_anyhow_identity()

### Community 152 - "Fixture"
Cohesion: 0.36
Nodes (12): list_toss_open_orders(), list_toss_open_orders_for_profile(), modify_toss_order(), modify_toss_order_for_profile(), require_toss_order_profile(), toss_decimal_equals(), toss_decimal_to_storage_units(), toss_open_order_view() (+4 more)

### Community 153 - "ExecutedOrder"
Cohesion: 0.18
Nodes (11): usd_to_cents(), StockListStats, ChartDataInput, get_chart_data(), get_overseas_price(), get_price(), get_stock_list_stats(), OverseasOrderInput (+3 more)

### Community 154 - "HeaderMap"
Cohesion: 0.46
Nodes (12): commit_rejects_scope_profile_client_and_credential_changes(), config(), configuration_fence_blocks_shared_automatic_and_manual_before_any_http_or_budget_change(), configuration_snapshots_include_real_held_and_flat_markets_and_revalidate_fresh_quantity(), duplicate_entries_use_the_same_first_market_as_the_live_strategy(), fixture(), held_target_removal_and_explicit_pc_lth_market_changes_fail_closed(), legacy_inverse_field_cannot_override_another_lth_targets_market() (+4 more)

### Community 155 - ".new"
Cohesion: 0.29
Nodes (3): BrokerCurrency, TossBuyingPower, TossExchangeRateResponse

### Community 156 - "strategy-continuous-replay.spec.ts"
Cohesion: 0.09
Nodes (23): mockApi(), MockOptions, mockResearchResult(), strategyEntries, syntheticCollectionSummary(), ApiAudit, apiAudits, calculate() (+15 more)

### Community 158 - "breakout.rs"
Cohesion: 0.15
Nodes (31): assertNoResultsSelection(), captureResults(), compareSameContextCaret(), expectResultsSelection(), focusFirstResult(), installResultsSourceHarness(), keyboardResultsFixture(), nativeActivationCounter() (+23 more)

### Community 159 - "Strategy"
Cohesion: 0.11
Nodes (18): ReplaySession, lth_preview_signals_for_interval(), execute(), LeveragedTrendHoldPreviewSignal, SimulationAssumptions, F, LeveragedTrendHoldPreviewSignal, BrokerPositionSnapshot (+10 more)

### Community 160 - "collect_pages"
Cohesion: 0.22
Nodes (16): default_atr_stop_multiplier(), default_daily_loss_limit(), default_exchange_rate(), default_fee_bps(), default_in_sample_percent(), default_initial_capital(), default_max_position_ratio(), default_risk_per_trade_bps() (+8 more)

### Community 161 - ".checkpoint_strategy_positions_shared"
Cohesion: 0.22
Nodes (6): clear_database_tables(), drop_database_tables(), ensure_trading_stopped(), export_database_to_json(), import_json_to_database(), set_storage_backend()

### Community 162 - ".capture"
Cohesion: 0.36
Nodes (12): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+4 more)

### Community 163 - ".collect"
Cohesion: 0.09
Nodes (33): a2_us_bar(), a2_us_params(), a2_us_run(), blocked_buy_is_fed_back_before_the_next_raw_signal(), candle(), daily_diagnostic_contains_one_completed_bar_and_no_synthetic_entry_observation(), daily_warmup_preview_uses_closes_and_excludes_future_bars(), FetchAdapter (+25 more)

### Community 164 - "collect_daily_history"
Cohesion: 0.18
Nodes (18): SectorQuoteSource, Fault, fixture_input(), fixture_manifest(), MockSource, Mutex, SectorDataManifest, Self (+10 more)

### Community 165 - "toss/orders.rs"
Cohesion: 0.14
Nodes (13): SliderWithInput(), LeveragedTrendHoldTickerPicker(), StockSearchResults(), StockSearchResultsProps, StrategyUsSearch(), LeveragedTrendHoldEditorModel, Market, StockSearchItem (+5 more)

### Community 166 - "Result"
Cohesion: 0.38
Nodes (10): bar(), request(), run(), volatility_expansion_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average(), volatility_expansion_common_atr_sizing_remains_and_sells_confirmed_adjusted_quantity(), volatility_expansion_daily_full_range_is_current_input_while_prior_mean_excludes_it(), volatility_expansion_diagnostic_serialization_and_other_strategy_omission(), volatility_expansion_optional_validation_both_previews_and_daily_only_preserved() (+2 more)

### Community 167 - "Self"
Cohesion: 0.30
Nodes (4): corrupted_primary_never_resurrects_orders_from_a_valid_backup(), PendingOrderStore, restart_snapshot_preserves_scope_client_id_and_fill_watermark(), snapshot()

### Community 168 - "String"
Cohesion: 0.21
Nodes (8): Builder, head_and_options_keep_existing_origin_and_method_policy(), origin_host_and_exact_bearer_rules_are_preserved_before_body_parsing(), spa_fallback_assets_head_and_unknown_api_behavior_are_preserved(), unsupported_methods_keep_405_and_allow_for_every_registered_url(), sector_data_rest_equals_ipc_and_does_not_access_account_or_provider(), sector_data_rest_rejects_auth_and_large_body_and_reports_not_ready(), authorized()

### Community 169 - "T"
Cohesion: 0.27
Nodes (4): ExecutionRun, ExecutionRunContext, OrderManager, validate_snapshot()

### Community 171 - "Vec"
Cohesion: 0.14
Nodes (28): captureSearchRace(), expectNoSearchError(), expectNoSearchSelection(), expectSearchSelection(), failedSearchReply(), installKeyboardScopeHarness(), invalidSearchReply(), KeyboardRaceFixture (+20 more)

### Community 172 - "Arc"
Cohesion: 0.22
Nodes (9): BudgetRecoveryItem, ApproveBudgetRecoveryInput, BudgetRecoveryItem, BudgetRecoveryView, is_us(), OrderManager, push_bounded(), RecoveryReceipt (+1 more)

### Community 173 - "Client"
Cohesion: 0.44
Nodes (11): candle(), config(), daily_hooks_do_not_create_unknown_symbol_states(), strong_close_completed_day_uses_actual_position_and_keeps_stop_loss(), strong_close_uses_each_completed_day_only_on_the_following_day(), volatility(), volatility_collects_missing_history_after_close_for_later_days(), volatility_compares_current_full_range_against_prior_completed_window() (+3 more)

### Community 174 - "lth_replay_clock.rs"
Cohesion: 0.12
Nodes (21): main(), Box, Error, date(), diagnostic_message(), hash(), note(), prices() (+13 more)

### Community 175 - "Mutex"
Cohesion: 0.41
Nodes (11): bar(), complete_day(), completed_high_not_close_sets_next_reference_without_lookahead(), config(), fifty_two_week_requires_full_window_and_uses_last_completed_high_and_close(), initialization_keeps_last_window_and_daily_bookkeeping_is_symbol_scoped(), many_intraday_prices_and_intraday_initializers_do_not_age_daily_highs(), missing_history_becomes_ready_only_after_252_completed_daily_highs() (+3 more)

### Community 177 - "Result"
Cohesion: 0.26
Nodes (11): SectorPageReceipt, collect_mode(), collect_sector_bundle(), price(), BTreeMap, S, SectorDataBundle, SectorDataManifest (+3 more)

### Community 178 - "Self"
Cohesion: 0.29
Nodes (6): active_slope_bounds_legacy_long_period_and_required_context(), finite(), legacy_params(), MaCrossParams, validate(), validate_ma_cross_entry_params()

### Community 179 - "String"
Cohesion: 0.27
Nodes (15): atomic_write_private_sync(), backup_failure_preserves_primary_and_cleans_staged_content(), durable_atomic_write(), durable_removal_clears_the_original_path(), durable_remove_file(), locked_destination_preserves_content_and_allows_retry_after_release(), private_sync_replacements_keep_owner_only_permissions(), read_only_unix_primary_can_be_replaced_in_a_writable_directory() (+7 more)

### Community 180 - "integer"
Cohesion: 0.19
Nodes (12): ChunkContinuityView, as_of(), candle(), daily_omits_timing_and_empty_minute_is_not_evaluable(), execution_clock_empty_unknown_and_collection_bounds_remain_explicit(), legacy_selection_stays_at_200_and_does_not_refill_rejected_price_rows(), prepared_clock_wire_and_chart_use_actual_admission_not_raw_tail(), DateTime (+4 more)

### Community 181 - "AppHandle"
Cohesion: 0.18
Nodes (9): args, artifactDir, artifacts, checksumPath, manifestPath, names, records, requireSignatures (+1 more)

### Community 182 - "preview_strategy_from_candles"
Cohesion: 0.44
Nodes (9): hysteresisError(), hysteresisOf(), isMomentumEntryParam(), modeOf(), MomentumEntryControls(), momentumEntryParamError(), MomentumEntrySummary(), parsedDraft() (+1 more)

### Community 183 - ".submit_order_shared"
Cohesion: 0.09
Nodes (38): certificates(), command(), containers, directory, environment, interruption, results, root (+30 more)

### Community 184 - "AccountProfile"
Cohesion: 0.35
Nodes (7): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, storage_money_units(), TossManualSession

### Community 185 - "FixedOffset"
Cohesion: 0.36
Nodes (10): date(), days(), history(), holdings(), input(), NaiveDate, SectorDay, SectorHolding (+2 more)

### Community 186 - "RwLock"
Cohesion: 0.16
Nodes (21): broker_chart_collector_accepts_identical_inclusive_overlap_but_rejects_conflict(), broker_chart_collector_default_kis_uses_12_single_ams_routes_and_exports_master_evidence(), broker_chart_collector_freezes_latest_42_spy_sessions_and_marks_unknown_not_verified(), broker_chart_collector_invalid_kis_mapping_or_any_toss_mapping_stops_before_fetch(), broker_chart_collector_manual_mapping_overrides_seed_and_evidence_changes_hash(), broker_chart_collector_manual_route_mapping_never_falls_back_or_accepts_partial_mapping(), broker_chart_collector_never_hides_sector_missing_date_by_intersection(), broker_chart_collector_never_substitutes_route_after_auth_http_rate_generic_rejection_or_oversize() (+13 more)

### Community 187 - "Self"
Cohesion: 0.23
Nodes (18): actual_quantity_unknown_origin_has_no_invented_failure_anchor_or_age(), atr_never_weakens_legacy_percent_limb(), atr_uses_prior_gaps_fractional_threshold_and_zero_range_is_ready(), bar(), candidates_do_not_change_ownership_and_same_zero_does_not_erase_anchor(), checkpoint_v1_unknown_and_v2_known_roundtrip_fail_atomically(), config(), context_updates_do_not_clear_anchor_age_peak_armed_and_stays_bounded() (+10 more)

### Community 188 - "Sender"
Cohesion: 0.12
Nodes (17): chart(), Control, daily, data(), domestic(), enter(), latest(), minutes (+9 more)

### Community 189 - "String"
Cohesion: 0.42
Nodes (9): bar(), candidate_current_range_cannot_change_its_own_prior_atr_filter(), guard_rejection_does_not_recreate_a_consumed_golden_cross(), integer_float_daily_period_stays_active_and_missing_history_blocks_only_golden_cross(), ma_cross_options_are_strict_in_short_and_continuous_preview_paths(), non_daily_filters_reject_explicitly_and_disabled_minute_compatibility_remains(), request(), run() (+1 more)

### Community 190 - "checked_budget_scope"
Cohesion: 0.22
Nodes (19): BudgetRecoveryView, approve_budget_recovery(), AutoTradingBudgetInput, budget_error(), checked_budget_scope(), checked_budget_update_scope(), get_auto_trading_budget(), get_budget_recovery() (+11 more)

### Community 191 - "BrokerAdapterResult"
Cohesion: 0.06
Nodes (59): useAppConfig(), useBalance(), useBrokerHoldings(), useOverseasBalance(), useScopedAccountQuery(), useOverseasPrice(), usePrice(), useTossMarketCalendar() (+51 more)

### Community 192 - "failed_breakout_entry_tests.rs"
Cohesion: 0.26
Nodes (18): LeveragedTrendHoldPreviewStep, LeveragedTrendHoldTimedCandle, LeveragedTrendHoldPreviewSignal, delayed_open_fill_reaches_lth_before_close_and_last_exit_remains_unfilled(), future_bar_does_not_change_prior_signal(), history(), params(), FixedOffset (+10 more)

### Community 193 - "EntryContext"
Cohesion: 0.40
Nodes (9): bar(), request(), run(), strong_close_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average(), strong_close_common_atr_sizing_remains_and_sells_confirmed_adjusted_quantity(), strong_close_optional_validation_both_previews_and_daily_only_preserved(), strong_close_own_current_high_and_volume_do_not_enter_prior_trend(), strong_close_partition_and_future_suffix_are_causal_for_all_options() (+1 more)

### Community 194 - "commands/risk.rs"
Cohesion: 0.24
Nodes (5): sync_kis_strategy_positions(), sync_toss_strategy_positions(), trading_start_handler(), update_strategy_handler(), UpdateStrategyBody

### Community 195 - "ConfigurationEditReservation"
Cohesion: 0.23
Nodes (7): EditResult, ConfigurationIdentity, OrderManager, execution_fingerprint_fixed_utf8_prefixes_and_array_output(), execution_fingerprint_preserves_the_existing_field_boundary(), profile(), profile_fingerprint()

### Community 196 - "SectorPortfolioPanel"
Cohesion: 0.18
Nodes (6): entries(), repeatReads(), retry(), setup(), status(), traceRow

### Community 197 - "DeviationStrategy"
Cohesion: 0.40
Nodes (9): build_provider_request(), prepare_buy_submission(), prepare_order_submission(), prepare_sell_submission(), rest_order_side(), toss_order_side(), toss_price_string(), PrepareDecision (+1 more)

### Community 198 - "toss/orders.rs"
Cohesion: 0.27
Nodes (9): SectorStrategy, PaperMark, PaperSnapshot, parseSnapshot(), SectorPaperComparison(), number(), pct(), SectorPortfolioResults() (+1 more)

### Community 199 - "Self"
Cohesion: 0.47
Nodes (7): accepted_http_response_publishes_before_its_event_under_the_same_gate(), balance(), delayed_http_a_to_b_to_a_response_never_publishes_or_emits(), delayed_http_response_cannot_undo_a_concurrent_buy_or_sell(), invalid_holdings_do_not_clear_the_previous_position_or_emit_success(), refresh(), Server

### Community 200 - "consecutive/tests.rs"
Cohesion: 0.27
Nodes (6): every_read_observes_current_primary_and_structural_errors_are_not_empty_ledgers(), fill(), missing_primary_starts_empty_and_real_file_restart_keeps_exact_idempotent_fill(), primary_corruption_blocks_instead_of_recovering_an_old_valid_backup(), primary_loss_is_detected_after_restart_even_with_no_pending_orders_and_a_valid_backup(), primary_only_creation_crash_gap_anchors_the_existing_generation_and_ownership()

### Community 201 - "trend_filter_protection.rs"
Cohesion: 0.29
Nodes (6): AtrPreparationView, AtrWindow, bar(), downward_gap_on_flat_candle_is_not_a_zero_range(), full_previous_close_history_is_required_and_old_ranges_expire(), large_prices_do_not_overflow_the_range_sum()

### Community 202 - "Arc"
Cohesion: 0.22
Nodes (5): layerRank, layers, root, srcRoot, violations

### Community 203 - "BalanceItem"
Cohesion: 0.22
Nodes (6): appEntry, escapedName, npmLock, pkg, rootDir, workspaceLock

### Community 204 - "ui/OverseasStockChart.tsx"
Cohesion: 0.20
Nodes (22): BrokerChartPrice, BrokerChartCollection, BrokerChartPageReceipt, collect_broker_chart(), CollectBrokerChartInput, hash(), kis_default_mappings(), RawBrokerChartPage (+14 more)

### Community 205 - "config"
Cohesion: 0.15
Nodes (19): CollectBrokerChartInput, check_scope(), collect_broker_chart_data(), error(), get_broker_chart_scope(), NativeSource, preview_broker_chart_portfolio(), Arc (+11 more)

### Community 206 - "input"
Cohesion: 0.33
Nodes (9): buy_preflight_adds_commission_and_blocks_when_cash_is_short(), buy_preflight_passes_when_required_cash_is_available(), evaluate_order_preflight(), format_money_amount(), invalid_commission_ratios_block_instead_of_becoming_zero_fee(), OrderPreflightConstraints, OrderPreflightDecision, OrderPreflightInput (+1 more)

### Community 207 - ".prepare_confirmed_ownership"
Cohesion: 0.26
Nodes (9): risk_receipts_preserve_fixed_keys_payloads_and_restored_deduplication(), apply(), bounded_receipts_and_numeric_overflow_reject_without_partial_mutation(), rebuild_replays_durable_trades_then_missing_intent(), roundtrip(), scope(), scope_is_part_of_event_identity_and_payload_conflicts_fail_closed(), serialized_retries_and_intent_before_risk_are_exactly_once() (+1 more)

### Community 210 - "strong_close/tests.rs"
Cohesion: 0.11
Nodes (11): build_monthly_path(), concurrent_writes_leave_one_complete_json_document(), corrupt_json_recovers_from_last_durable_backup(), json_path_lock(), monthly_path_zero_pads_month_for_stats(), read_json_or_default(), write_json(), RiskStore (+3 more)

### Community 211 - "uiux-history-details.spec.ts"
Cohesion: 0.20
Nodes (10): expected, krw, krwSecond, readAllDetails(), records, search(), setup(), trigger() (+2 more)

### Community 212 - ".new"
Cohesion: 0.20
Nodes (16): Reply, expired_zero_and_pending_deadline_keep_summary_without_post_failure_work(), failures_preserve_errors_and_known_partial_snapshot_without_final_hash(), independent_python_literal_oracle_matches_real_core(), native_precision_currency_volume_and_aliases_survive_hashing(), normalized_adapter_stall_and_invalid_page_are_not_valid_cohort(), outcome(), ready_noncooperative_response_cannot_trigger_prepare_or_hash_after_deadline() (+8 more)

### Community 213 - "HashMap"
Cohesion: 0.25
Nodes (6): endpointInventory, EXPECTED_PATHS, hasRateLimitHeaders, missingPaths, paths, servers

### Community 214 - "HeaderMap"
Cohesion: 0.15
Nodes (12): node, vite.config.ts, vite.research.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution (+4 more)

### Community 216 - "StockStore"
Cohesion: 0.11
Nodes (16): canonical(), Fixture, mark_route(), mock_provider(), Body, Drop, HeaderMap, JoinHandle (+8 more)

### Community 217 - "작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단"
Cohesion: 0.42
Nodes (8): bar(), request(), run(), trend_filter_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average(), trend_filter_confirmation_resets_on_recovery_and_chunk_preserves_consecutive_exits(), trend_filter_partition_and_future_suffix_are_causal_for_all_options(), trend_filter_profit_arms_on_actual_avg_and_reoffers_after_actual_guard_refusal(), trend_filter_strict_validation_in_both_previews_and_minute_support()

### Community 218 - "budget/recovery/tests.rs"
Cohesion: 0.42
Nodes (9): build_strategy(), config(), live_feedback_deviation_manager_observations_match_replay_and_ownership_requires_feedback(), live_feedback_manager_pending_and_unsettled_suppress_only_outward_signal_across_restart(), live_feedback_manager_protection_metadata_requires_exact_owner_and_valid_checkpoint(), live_feedback_manager_targets_exact_owner_scope_and_market(), live_feedback_mean_reversion_manager_observations_match_replay_and_ownership_requires_feedback(), live_feedback_rsi_unfilled_and_ma_share_exact_replay_observations() (+1 more)

### Community 219 - "toss/tests.rs"
Cohesion: 0.12
Nodes (4): maps_holding_to_broker_domain(), maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request()

### Community 220 - "PendingOrderStore"
Cohesion: 0.60
Nodes (5): calculate_sector_portfolio(), preview_sector_portfolio(), CmdResult, SectorPortfolioInput, SectorPortfolioReport

### Community 221 - "BrokerHoldingView"
Cohesion: 0.15
Nodes (17): historyFixture(), strategy(), card(), fillFilters(), filterParams, insufficientEntry, legacyParams, Params (+9 more)

### Community 222 - "BrokerAdapterResult"
Cohesion: 0.40
Nodes (9): release(), strong_bar(), strong_close_manager_consumption_survives_late_legacy_invalidation_and_failure(), strong_close_manager_late_profile_scope_generation_do_not_publish_context_or_erase_owned(), strong_close_manager_new_day_reseeds_same_day_refresh_never_rearms_consumed(), strong_close_manager_pending_without_checkpoint_keeps_age_unknown_after_suppressed_tick(), strong_close_manager_trend200_context_and_held_exit_during_missing_history(), strong_close_manager_unfilled_pending_failure_and_legacy_warmup_preserve_original_proof() (+1 more)

### Community 223 - "BrokerCurrency"
Cohesion: 0.47
Nodes (8): check_toss_order_preflight(), check_toss_order_preflight_for_profile(), parse_toss_order_side(), select_toss_commission(), toss_currency_from_view(), toss_market_country(), TossOrderPreflightInput, TossOrderPreflightView

### Community 224 - "BrokerId"
Cohesion: 0.39
Nodes (7): broker_chart_preview_handler(), preview_handler(), BrokerChartInput, Json, SectorPortfolioInput, StatusCode, Value

### Community 225 - "mean_trend/trend_filter_tests.rs"
Cohesion: 0.44
Nodes (9): known_fill(), strategy(), trend_filter_default_strict_current_inclusive_and_overflow_contract(), trend_filter_optional_protection_before_readiness_uses_native_avg_and_observed_peak(), trend_filter_pending_rejected_partial_and_same_quantity_keep_actual_evidence(), trend_filter_raw_validation_and_corrupt_runtime_preserve_independent_periods(), trend_filter_reinitialize_core_change_reset_and_optional_change_have_distinct_lifecycles(), trend_filter_strict_checkpoint_is_atomic_and_restore_reobserves_confirmation() (+1 more)

### Community 226 - "submission.rs"
Cohesion: 0.46
Nodes (7): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), calculate_atr(), initialize_active_strategy_history(), legacy_history_symbols()

### Community 227 - "strong_close_history_tests.rs"
Cohesion: 0.26
Nodes (11): card(), continuous(), labels, legacyParams, Params, policyParams, response(), runButton() (+3 more)

### Community 228 - "lth_replay_clock.rs"
Cohesion: 0.29
Nodes (10): allocation_cannot_withdraw_committed_cash_or_overflow(), BudgetAccount, BudgetCurrencyView, BudgetLedger, BudgetReservation, CurrencyLedger, BTreeMap, Option (+2 more)

### Community 229 - "Self"
Cohesion: 0.50
Nodes (3): main(), Box, Error

### Community 230 - "initialize_strategy_warmup"
Cohesion: 0.54
Nodes (7): ranges(), release(), volatility_expansion_manager_pending_proof_survives_failure_and_actual_fill(), volatility_expansion_manager_same_day_refresh_failure_and_legacy_never_reset_range(), volatility_expansion_manager_scope_generation_and_short_history_preserve_owned_exits(), volatility_expansion_manager_unknown_pending_never_gains_entry_age_from_suppressed_tick(), volatility_manager()

### Community 231 - "Vec"
Cohesion: 0.24
Nodes (19): CollectSectorDataInput, Option, SectorCorporateAction, SectorPortfolioInput, String, Vec, SectorAdjustedPrice, SectorDataBundle (+11 more)

### Community 232 - "Arc"
Cohesion: 0.20
Nodes (18): BrokerChartAdjustment, BrokerChartBroker, BrokerChartCurrency, BrokerChartDay, BrokerChartInput, BrokerChartInterval, BrokerChartPrice, BrokerChartReport (+10 more)

### Community 234 - "Client"
Cohesion: 0.15
Nodes (13): DividendScenarioPoint, FxScenarioPoint, ExecutionModel, IntrabarExitPolicy, Default, SimulationAssumptions, CostSource, DividendPriceBasis (+5 more)

### Community 236 - "Option"
Cohesion: 0.07
Nodes (26): BalanceResult, OverseasBalanceResult, mockUx(), UxHandler, UxResponse, assertReadOnly(), assertTossGates(), domestic (+18 more)

### Community 239 - "String"
Cohesion: 0.22
Nodes (12): BrokerChartQuoteSource, BrokerChartClient, number(), row(), Arc, BrokerChartDay, ChartCandle, KisRestClient (+4 more)

### Community 240 - "T"
Cohesion: 0.50
Nodes (8): atr_uses_gap_true_ranges_fourteen_denominator_and_prior_close(), bandwidth_uses_twenty_population_samples_and_inclusive_boundary(), bar(), large_native_prices_do_not_overflow_daily_statistics_and_invalid_ohlc_fail_closed(), p(), repeated_rejection_reasons_overlap_without_inflating_candidate_count(), strict_trend_and_inclusive_slope_use_prior_daily_values(), sufficient_zero_ranges_are_ready_rejections_while_missing_history_is_unready()

### Community 241 - "Vec"
Cohesion: 0.23
Nodes (13): card(), chooseMode(), continuous(), crossingParams, Harness, labels, legacyParams, Params (+5 more)

### Community 242 - "CandleDataSummary.tsx"
Cohesion: 0.47
Nodes (3): BalanceSnapshot, BalanceStore, HoldingItem

### Community 244 - "MockSource"
Cohesion: 0.29
Nodes (5): react, react-dom, react-dom, bootstrap(), queryClient

### Community 245 - "StatusCode"
Cohesion: 0.15
Nodes (7): zustand, AccountState, useAccountStore, TradingState, TradingStatus, useTradingStore, zustand

### Community 246 - "String"
Cohesion: 0.21
Nodes (15): Cow, EventLoopProxy, Retained, HostEvent, main(), permitted_navigation(), record(), resource() (+7 more)

### Community 247 - "strategy/tests.rs"
Cohesion: 0.15
Nodes (13): useOverseasChartData(), buildChartOptions(), CandlePoint, ChartType, CrosshairData, LinePoint, OverseaChartPreset, OVERSEAS_CHART_PRESETS (+5 more)

### Community 248 - "Client"
Cohesion: 0.22
Nodes (15): accountingAssumptionsLabel(), AccountingAssumptionSummary(), accountingComparisonKey(), AccountingResults(), ComparisonResults(), EditorProps, numberInRange(), parseScenarios() (+7 more)

### Community 249 - "Response"
Cohesion: 0.27
Nodes (10): bounded(), main(), ProbeSource, Option, Path, PathBuf, String, Vec (+2 more)

### Community 250 - "Result"
Cohesion: 0.50
Nodes (6): restore_risk_from_today_trades(), risk_rebuild_isolates_paper_profit_from_live_losses_for_the_same_account(), risk_rebuild_seeds_trade_event_receipts_and_retries_missing_pending_intent_once(), risk_rebuild_uses_chronological_profits_and_keeps_manual_emergency_stop(), trade(), unknown_mode_or_runtime_save_failure_never_publishes_a_partial_rebuild()

### Community 251 - "main"
Cohesion: 0.20
Nodes (9): category_from_key(), database_scanner_preserves_original_payload_order_and_checksum_input(), document_key(), inventory_from_documents(), atomic_write(), LocalDocument, scan_json_documents(), scan_json_documents_sync() (+1 more)

### Community 252 - "Option"
Cohesion: 0.13
Nodes (22): useCollectSectorData(), useSectorQuoteScope(), useValidateSectorData(), collectSectorData(), getSectorQuoteScope(), validateSectorData(), CollectSectorDataInput, SectorDataBundle (+14 more)

### Community 253 - "Result"
Cohesion: 0.16
Nodes (9): usePreviewSectorPortfolio(), queryClient, theme, parseSectorPortfolioInput(), Completed, PanelProps, SectorPortfolioPanel(), client (+1 more)

### Community 254 - "Self"
Cohesion: 0.23
Nodes (13): basis(), card(), continuous(), labels, legacyParams, Params, policyParams, ready (+5 more)

### Community 255 - "String"
Cohesion: 0.60
Nodes (5): ack(), collectButton(), preparation(), scopeButton(), selectedScope()

### Community 256 - "Vec"
Cohesion: 0.17
Nodes (11): app, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, bundle, shortDescription (+3 more)

### Community 257 - "BrokerCurrency"
Cohesion: 0.26
Nodes (11): card(), continuous(), filterParams, labels, legacyParams, Params, response(), runButton() (+3 more)

### Community 258 - "BrokerMarket"
Cohesion: 0.15
Nodes (23): main(), Box, Error, accounting_input(), preview_broker_chart_price_approximation(), BrokerChartInput, BrokerChartReport, SectorPortfolioInput (+15 more)

### Community 259 - "Option"
Cohesion: 0.27
Nodes (10): AutoTradingBudgetInput, budget_handler(), budget_response(), AutoTradingBudgetView, CmdError, Json, Query, Response (+2 more)

### Community 260 - "Result"
Cohesion: 0.26
Nodes (12): card(), chooseMode(), continuous(), labels, legacyParams, Params, policyParams, response() (+4 more)

### Community 261 - "String"
Cohesion: 0.19
Nodes (10): BrokerAdapterError, default_place_order_returns_unsupported_with_broker_id(), DummyAdapter, LegacyPageAdapter, AtomicUsize, BrokerAdapterResult, BrokerId, Error (+2 more)

### Community 262 - "config"
Cohesion: 0.40
Nodes (13): CandlePageView, collect_pages(), bar(), collector_preserves_raw_volume_on_its_exact_completed_ohlc_row(), collects_252_actual_completed_highs_without_dropping_the_last_completed_bar(), empty_filtered_page_can_advance_and_full_512_live_window_is_bounded(), limits_requests_and_stops_on_repeat_or_no_progress(), only_first_provider_failure_changes_exchange_and_later_failure_is_explicit_partial() (+5 more)

### Community 263 - "failedBreakoutControls.tsx"
Cohesion: 0.57
Nodes (7): bar(), continuous(), momentum_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal(), momentum_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry(), momentum_new_fields_are_strict_in_both_production_preview_paths(), params(), request()

### Community 264 - "From"
Cohesion: 0.54
Nodes (7): list_toss_accounts(), list_toss_profile_accounts(), lookup_toss_accounts_with_credentials(), mask_toss_account_no(), toss_account_option(), TossAccountLookupInput, TossAccountOptionView

### Community 265 - "bounded_optional_integer"
Cohesion: 0.39
Nodes (5): delete_database_password(), entry(), fail_next_mock_operation_for_tests(), load_database_password(), store_database_password()

### Community 266 - "holding"
Cohesion: 0.23
Nodes (9): restart_rebuilds_order_counts_and_consecutive_losses_from_ledgers(), holding(), scope(), toss_fractional_and_zero_quantities_never_create_a_whole_share(), toss_rejects_invalid_quantities_and_prices_including_float_u64_boundary(), toss_rejects_wrong_broker_account_currency_and_duplicate_market_symbol(), toss_us_prices_round_to_cents_and_kr_prices_round_to_won(), broker_scope_serializes_with_camel_case_fields() (+1 more)

### Community 267 - "config"
Cohesion: 0.39
Nodes (4): normalize_overseas_order_exchange(), HoldingsUpdate, money(), quantity()

### Community 268 - "classic/live_feedback_tests.rs"
Cohesion: 0.39
Nodes (3): PhaseDiagnosticsView, ReplaySession, Snapshot

### Community 269 - "T"
Cohesion: 0.50
Nodes (6): lookback(), number(), parse(), validate_volatility_expansion_params(), VolatilityExpansionParams, bounded_optional_integer()

### Community 271 - "fill_accounting.rs"
Cohesion: 0.18
Nodes (10): LiveDailyHistoryStatus, TradingStatus, HistoryRow(), LiveDailyHistoryPanel(), Props, retryTime(), STATE_LABELS, STATE_PRIORITY (+2 more)

### Community 273 - "vite"
Cohesion: 0.14
Nodes (13): DefensiveState, SectorDaily, SectorDay, SectorHolding, SectorPlan, SectorPortfolioResult, SectorPrice, SectorRank (+5 more)

### Community 274 - "DailyHistoryJobs"
Cohesion: 0.23
Nodes (9): TargetKey, collect_daily_history(), CollectedDailyHistory, completed_bar(), DailyHistoryJobs, JobEntry, retry_delay(), price() (+1 more)

### Community 275 - "strategyAccountingPanel.tsx"
Cohesion: 0.52
Nodes (6): broker_chart_input(), broker_chart_rest_equals_pure_ipc_and_rejects_missing_price_or_ack(), Value, sector_portfolio_missing_price_rejects_the_whole_comparison_and_large_body(), sector_portfolio_rest_equals_ipc_without_state_or_provider_access(), supplied_input()

### Community 276 - "prepare"
Cohesion: 0.28
Nodes (12): assessment(), chart(), execution_clock(), prepare(), Prepared, reason_name(), ChartCandle, CmdResult (+4 more)

### Community 277 - "strategy_preview/tests.rs"
Cohesion: 0.18
Nodes (7): currency, destination(), returnToContents(), risk, sections, setup(), toc()

### Community 278 - "collect_daily_history"
Cohesion: 0.21
Nodes (12): ConsecutiveEntryControls(), ConsecutiveEntryField(), consecutiveEntryFiltersDisabled(), consecutiveEntryParamError(), ConsecutiveEntrySummary(), fieldError(), fields, isConsecutiveEntryParam() (+4 more)

### Community 279 - "CmdError"
Cohesion: 0.51
Nodes (10): bar(), continuous(), params(), request(), rsi_cap_counts_after_actual_next_open_fill_across_chunk_boundaries(), rsi_cap_reoffers_after_actual_minimum_profit_guard_refusal(), rsi_filtered_cross_does_not_retry_and_default_has_no_flat_sell_or_held_buy(), rsi_next_open_uses_real_fill_basis_and_stops_before_original_cross() (+2 more)

### Community 280 - "RunFence"
Cohesion: 0.29
Nodes (3): Future, Output, RunFence

### Community 281 - "ChartCandle"
Cohesion: 0.27
Nodes (6): CandleCollection, CandleInterval, CandlePageView, ContinuousStrategyPreviewChunk, ChartCandle, readyPreparation

### Community 282 - "Option"
Cohesion: 0.73
Nodes (5): failed_breakout_held_target_removal_is_rejected_and_successful_edit_preserves_age(), failed_breakout_optional_validation_precedes_save_and_valid_values_roundtrip(), failed_breakout_save_replace_failure_rolls_back_without_losing_owned_basis_or_age(), failed_breakout_state(), update()

### Community 284 - "Self"
Cohesion: 0.40
Nodes (5): Json, StatusCode, ValidateSectorDataInput, Value, validate_handler()

### Community 286 - "String"
Cohesion: 0.52
Nodes (6): config(), deviation_counts_current_price_as_last_required_observation(), deviation_history_is_ready_before_first_evaluation_and_bounded(), momentum_counts_first_comparison_but_excludes_seed_prices(), momentum_history_does_not_create_or_erase_order_state(), rsi_initializer_restores_crossing_baseline_without_historical_signals()

### Community 287 - "deviation_entry_tests.rs"
Cohesion: 0.53
Nodes (9): bar(), continuous(), deviation_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal(), deviation_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry(), deviation_new_fields_are_strict_in_both_production_preview_paths(), deviation_protective_age_counts_actual_held_observations_across_chunks(), deviation_stop_priority_uses_real_fill_basis_and_next_open_execution(), params() (+1 more)

### Community 289 - "mean_reversion_entry_tests.rs"
Cohesion: 0.53
Nodes (9): bar(), continuous(), mean_reversion_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal(), mean_reversion_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry(), mean_reversion_new_fields_are_strict_in_both_production_preview_paths(), mean_reversion_protective_age_counts_actual_held_observations_across_chunks(), mean_reversion_stop_priority_uses_real_fill_basis_and_next_open_execution(), params() (+1 more)

### Community 290 - "capture_cycle"
Cohesion: 0.31
Nodes (7): capture_cycle(), ExecutionCycle, failed_configuration_pause_never_becomes_market_close_and_same_revision_resumes(), market_closed_first_order_settles_remaining_raw_batch_without_touching_submitted_state(), old_market_closed_batch_cannot_restore_positions_into_restarted_run(), strategies(), unsubmitted_feedback_never_assigns_an_unowned_account_holding_to_strategy()

### Community 291 - "get_toss_market_snapshot_for_profile"
Cohesion: 0.22
Nodes (7): TossOrderbookEntry, get_toss_market_snapshot(), get_toss_market_snapshot_for_profile(), TossMarketSnapshotView, TossOrderbookView, TossPriceLimitView, TossTradeView

### Community 292 - "database_archive.rs"
Cohesion: 0.36
Nodes (5): DatabaseArchiveStats, purge(), stats(), trade_date(), trade_rows()

### Community 293 - "scaled_total"
Cohesion: 0.73
Nodes (5): rsi_held_target_removal_is_rejected_and_successful_edit_preserves_age(), rsi_optional_validation_precedes_save_and_valid_values_roundtrip(), rsi_save_replace_failure_rolls_back_without_losing_owned_basis_or_age(), rsi_state(), update()

### Community 294 - "guard/recovery/tests.rs"
Cohesion: 0.47
Nodes (5): append_failed_order(), build_kis_pending_order(), build_pending_order(), build_toss_pending_order(), failed_order_price()

### Community 295 - "StrongCloseState"
Cohesion: 0.80
Nodes (5): config(), reconfigure_fifty_two_week_keeps_actual_entry_while_invalidating_old_window(), reconfigure_strong_close_preserves_stop_but_discards_old_entry_candidate(), reconfigure_volatility_recomputes_range_without_losing_held_stop_or_day(), reject_scope_changes()

### Community 296 - "database/tests.rs"
Cohesion: 0.31
Nodes (5): inventory_groups_documents_by_top_level_category(), keychain_test_dirs(), legacy_file_password_migrates_to_keychain_on_load(), save_config_stores_password_in_keychain_not_in_file(), save_input_with_password()

### Community 297 - "trendFilterControls.tsx"
Cohesion: 0.40
Nodes (4): encode(), Protection, restore(), StrongCloseState

### Community 298 - "KisBrokerAdapter"
Cohesion: 0.80
Nodes (5): config(), reconfigure_deviation_applies_new_threshold_to_retained_observations(), reconfigure_momentum_keeps_owned_quantity_and_retries_unfilled_exits(), reconfigure_rsi_recomputes_baseline_with_new_period_and_keeps_targets_separate(), reject_identity_changes()

### Community 304 - "Error"
Cohesion: 0.43
Nodes (6): card(), chooseContinuous(), dailySeries(), researchResponse(), setup(), SetupOptions

### Community 305 - "RwLock"
Cohesion: 0.50
Nodes (4): input(), BrokerChartInput, String, text()

### Community 306 - "autoTradingBudgetSection.tsx"
Cohesion: 0.22
Nodes (13): RecoveryIdentity, useApproveBudgetRecovery(), useBudgetRecovery(), BudgetRecoverySection(), money(), RecoveryCard(), approveBudgetRecovery(), getBudgetRecovery() (+5 more)

### Community 307 - "String"
Cohesion: 0.60
Nodes (5): config(), consecutive_counts_current_price_and_uses_active_position_branch(), consecutive_history_preserves_flat_and_existing_positions(), failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation(), failed_breakout_history_retains_actual_breakout_reference()

### Community 308 - "CmdError"
Cohesion: 0.08
Nodes (30): File, AppState, balance_summary_total_krw(), CmdError, make_rest_client(), parse_amount_i64(), Arc, AtomicBool (+22 more)

### Community 309 - "CmdResult"
Cohesion: 0.60
Nodes (5): config(), live_feedback_breakout_checkpoint_restores_pending_and_confirmed_exact_anchor(), live_feedback_breakout_freezes_pending_signal_anchor_until_first_fill(), live_feedback_consecutive_keeps_confirmed_flag_and_observations(), live_feedback_unsubmitted_breakout_discards_only_candidate_not_history()

### Community 310 - "DatabaseConfigView"
Cohesion: 0.26
Nodes (8): gate(), keyboardObservationWindow(), KeyboardSearchFixture, keyboardStock, safeKeyboardSearch(), setup(), states, Surface

### Community 311 - "formatters.ts"
Cohesion: 0.39
Nodes (6): phaseName(), StrategyPhaseResults(), validDiagnostics(), BacktestPhaseDiagnostics, BrokerMoneyView, fmtRoundedKrw()

### Community 312 - "DatabaseTransferResult"
Cohesion: 0.16
Nodes (14): Arc, AsyncMutex, Into, Self, shared_toss_token_state(), toss_token_state_key(), TossTokenState, toss_http_client() (+6 more)

### Community 313 - "attach_comparison"
Cohesion: 0.39
Nodes (7): return_pct(), attach_comparison(), BuyHoldComparisonView, comparator_preserves_actual_trades_and_discloses_final_pending(), CostRestoredReferenceView, ReplayComparisonView, same_quantity_reference_discloses_exposure_and_restores_only_recorded_costs()

### Community 314 - "FailedBreakoutParams"
Cohesion: 0.39
Nodes (5): FailedBreakoutBasis, FailedBreakoutParams, number(), parse(), validate_failed_breakout_params()

### Community 315 - "lib.rs"
Cohesion: 0.43
Nodes (6): app_context(), copy_dir_all(), Context, Path, run(), install_database_manager()

### Community 316 - "execution_block_reason"
Cohesion: 0.29
Nodes (7): execution_block_reason(), PositionState, RiskManager, signal_parts(), SimulationEvent, Signal, StrategySignal

### Community 317 - "CandleDataSummary.tsx"
Cohesion: 0.33
Nodes (6): CandleDataSummary(), CandlePoint, focusStyle, number(), Projection, Props

### Community 318 - "continuous.rs"
Cohesion: 0.43
Nodes (6): ChunkContinuityView, ContinuousStrategyPreviewChunk, ContinuousStrategyPreviewInput, ContinuousStrategyPreviewView, invalid(), ReplayContinuityView

### Community 320 - "ChartCandle"
Cohesion: 0.12
Nodes (14): args, digest(), files, generated, hashes, index, manifest, metadata (+6 more)

### Community 321 - "DatabaseManager"
Cohesion: 0.50
Nodes (3): OrderManager, own_fill(), possible_scope()

### Community 323 - "strategy_preview/trend_filter_tests.rs"
Cohesion: 0.70
Nodes (4): config(), live_feedback_fifty_two_week_keeps_crossing_progress_without_optimistic_holdings(), live_feedback_strong_close_consumes_candidate_only_after_buy_fill(), live_feedback_volatility_keeps_intraday_extremes_and_confirmed_position()

### Community 324 - "ContinuousStrategyPreviewInput"
Cohesion: 0.90
Nodes (4): config(), reconfigure_mean_reversion_keeps_stop_active_before_larger_window_is_ready(), reconfigure_trend_filter_retains_owned_quantity_and_uses_new_period_for_exit(), reject_scope_changes()

### Community 325 - "managed_paths"
Cohesion: 0.62
Nodes (6): read_managed_json(), check_size(), history_key(), local_paths(), managed_paths(), records()

### Community 326 - "input"
Cohesion: 0.70
Nodes (4): config(), empty_price_condition_entries_remove_stale_targets_before_manager_validation(), reconfigure_price_condition_canonicalizes_targets_and_keeps_actual_stop(), reconfigure_price_condition_does_not_move_holdings_to_replacement_target()

### Community 327 - "mean_reversion_entry_tests.rs"
Cohesion: 0.90
Nodes (4): config(), reconfigure_consecutive_preserves_holding_and_sell_observations(), reconfigure_failed_breakout_keeps_exact_reference_when_new_history_is_short(), reject_scope_changes()

### Community 328 - ".record_fill_pnl_once"
Cohesion: 0.60
Nodes (3): create(), mariadb_ddl(), postgres_ddl()

### Community 329 - "CmdResult"
Cohesion: 0.53
Nodes (4): DateTime, Utc, PreparedLthPreview, SimulationAssumptions

### Community 330 - "Option"
Cohesion: 0.83
Nodes (3): confirmed_small_fill_requires_original_order_symbol_side_currency_and_whole_share(), detail(), missing_amount_or_zero_fill_is_deferred_without_using_limit_or_average_as_money()

### Community 332 - "OrderSide"
Cohesion: 0.83
Nodes (3): place_overseas_with_retry_shared(), place_prepared_order(), place_with_retry_shared()

### Community 334 - "Self"
Cohesion: 0.33
Nodes (5): body(), Vec, path_values_decode_without_changing_the_route_segment_contract(), production_json_query_and_path_rejections_stop_before_provider_or_mutation(), trailing_json_is_rejected_by_the_framework_before_order_validation()

### Community 335 - "State"
Cohesion: 0.83
Nodes (3): config(), live_feedback_mean_reversion_keeps_entry_and_tick_history_on_sell(), live_feedback_trend_filter_keeps_confirmation_and_updates_indicators()

### Community 349 - "strategy-trend-filter-policy.spec.ts"
Cohesion: 0.22
Nodes (13): card(), continuous(), coreLabels, labels, legacyParams, Params, policyParams, response() (+5 more)

### Community 351 - "strategy-volatility-expansion-policy.spec.ts"
Cohesion: 0.20
Nodes (14): card(), continuous(), coreLabels, labels, legacyParams, Params, policyParams, ready (+6 more)

### Community 357 - "balance-events.spec.ts"
Cohesion: 0.33
Nodes (3): accountConfig, EventHarness, harnessViolations

### Community 358 - "momentum_entry_tests.rs"
Cohesion: 0.40
Nodes (3): OrderManager, PreparedOwnershipFill, realized_pnl()

### Community 359 - ".capture"
Cohesion: 0.26
Nodes (14): basisOf(), BreakoutNumericField(), FailedBreakoutControls(), failedBreakoutDailyFiltersDisabled(), failedBreakoutPolicyError(), FailedBreakoutPolicySummary(), fieldError(), fields (+6 more)

### Community 363 - "StrategyConfig"
Cohesion: 0.53
Nodes (4): number(), parse(), StrongCloseParams, validate_strong_close_params()

### Community 365 - "current_position_snapshot"
Cohesion: 0.40
Nodes (5): OwnedWork, Box, F, Option, Pin

### Community 369 - ".submit_order_shared"
Cohesion: 0.17
Nodes (18): B, CandleResponseSource, DateTime, Utc, String, T, body_snippet(), read_response_text() (+10 more)

### Community 373 - "strategy-strong-close-policy.spec.ts"
Cohesion: 0.22
Nodes (13): card(), continuous(), labels, legacyParams, Params, policyParams, ready, response() (+5 more)

### Community 384 - "install_database_manager"
Cohesion: 0.10
Nodes (35): AsyncRead, AsyncWrite, Poll, ReadBuf, connect_stream(), KisWebSocketClient, parse_realtime_price(), publish_status() (+27 more)

### Community 402 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.23
Nodes (13): chartTimeToMs(), formatChartTimeKst(), formatKstMs(), pad2(), parseTimeMs(), Props, StrategyPreviewChart(), StrategyPreviewChartSignal (+5 more)

### Community 707 - "package.json"
Cohesion: 0.25
Nodes (7): engines, node, npm, name, private, type, version

## Knowledge Gaps
- **558 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+553 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **1245 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `BrokerSymbol` connect `.collect` to `MarketDayCalendar`, `BrokerScope`, `String`, `holding`, `CurrencyLedger`, `RunFence`, `RsiStrategy`, `order/ownership/tests.rs`, `get_toss_market_snapshot_for_profile`, `DatabaseManager`, `ReplaySession`, `integer`, `simulation.rs`, `BrokerSymbol`, `server/trading.rs`, `build_strategy`, `.new`, `toss/tests.rs`, `BrokerCurrency`, `holdings/tests.rs`, `submission.rs`, `leveragedTrendHoldPreviewChart.tsx`, `String`, `strategy/tests.rs`, `generic_tests.rs`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Why does `BrokerAccountId` connect `holding` to `conflicts.rs`, `MarketDayCalendar`, `OwnershipLedger`, `TradeRecord`, `modify_toss_order_for_profile`, `order/ownership/tests.rs`, `.submit_order_shared`, `DatabaseManager`, `Self`, `DiscordNotifier`, `security.rs`, `AccountProfile`, `BrokerSymbol`, `order/budget.rs`, `checked_budget_scope`, `commands/risk.rs`, `.new`, `KisWebSocketClient`, `.prepare_confirmed_ownership`, `support.rs`, `budget/recovery/tests.rs`, `toss/tests.rs`, `@mui/icons-material`, `momentum_entry_tests.rs`, `Signal`, `Result`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **Why does `AppState` connect `CmdError` to `DailyHistoryTicket`, `read_json_or_default`, `config`, `checked_budget_scope`, `DiscordNotifier`, `strong_close/tests.rs`, `setup-local.sh`, `day_event_tests.rs`, `AtomicBool`, `ReplaySession`, `VolatilityExpansionStrategy`, `strategy-configure/index.ts`, `strategy/ui/Page.tsx`?**
  _High betweenness centrality (0.048) - this node is a cross-community bridge._
- **Are the 24 inferred relationships involving `BrokerSymbol` (e.g. with `candle_source_known_bytes_through_adapter()` and `candle_source_raw_identity_chunking_precision_and_legacy_projection()`) actually correct?**
  _`BrokerSymbol` has 24 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `mockApi()` (e.g. with `date()` and `start()`) actually correct?**
  _`mockApi()` has 3 INFERRED edges - model-reasoned connections that need verification._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _558 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `shared/api/commands.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.04102233676975945 - nodes in this community are weakly interconnected._