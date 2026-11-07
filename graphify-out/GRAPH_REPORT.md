# Graph Report - KISAutoTrade  (2026-11-07)

## Corpus Check
- 606 files · ~426,248 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 8019 nodes · 13869 edges · 1794 communities (337 shown, 1457 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 967 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `58600431`
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
- strategyAccountingPanel.tsx
- strategy_preview/tests.rs
- collect_daily_history
- CmdError
- From
- Option
- Result
- Self
- String
- Vec
- CmdResult
- trading/ui/Page.tsx
- scaled_total
- guard/recovery/tests.rs
- StrongCloseState
- run_prepared_preview
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
- DatabaseTransferResult
- lth-minute-timing.spec.ts
- String
- ChartCandle
- ChartCandle
- DatabaseManager
- strategy_preview/trend_filter_tests.rs
- ContinuousStrategyPreviewInput
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
- BrokerAccountId
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
- Result
- S
- rsi_state
- strongCloseControls.tsx
- TossTokenState
- notional/tests.rs
- database_archive.rs
- config
- BrokerAdapterResult
- BrokerId
- Error
- Send
- String
- Sync
- momentumEntryControls.tsx
- DateTime
- restore
- Option
- Result
- String
- Utc
- Vec
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
- CmdResult
- Mutex
- BrokerId
- Debug
- Default
- Formatter
- Into
- Option
- Result
- Self
- String
- Vec
- Arc
- Path
- Result
- Self
- String
- Vec
- Path
- Result
- Default
- Option
- Path
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
- Result
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
- Result
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
- DateTime
- Option
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
- Option
- Path
- PathBuf
- Result
- RwLock
- Self
- StockSearchItem
- String
- Vec
- Mutex
- PathBuf
- Result
- Self
- Vec
- Value
- Vec
- ContinuousStrategyPreviewView
- String
- Value
- ContinuousStrategyPreviewView
- StrategyPreviewInput
- String
- Value
- ChartCandle
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
- Result
- String
- Arc
- BrokerId
- HashMap
- Option
- String
- Value
- Vec
- Client
- Option
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
- Result
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
- Result
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
- Result
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
- Result
- StockSearchItem
- String
- Vec
- Client
- Result
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
- Result
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
- Body
- Drop
- HeaderMap
- JoinHandle
- Next
- PathBuf
- Response
- Router
- Self
- String
- Uri
- Value
- Vec
- BrokerId
- ContinuousStrategyPreviewInput
- exits.rs
- indicators.rs
- live.rs
- Result
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
- Result
- SaveDatabaseConfigInput
- Self
- String
- Duration
- Error
- F
- Result
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
- Result
- Self
- MutexGuard
- Option
- Result
- String
- Option
- Result
- String
- Value
- Vec
- Path
- PathBuf
- Result
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
- Result
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
- Result
- Self
- String
- Vec
- Mutex
- PathBuf
- PendingOrder
- Result
- Self
- Vec
- BrokerId
- Mutex
- NaiveDate
- Option
- PathBuf
- Result
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
- Result
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
- Result
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
- Result
- BrokerId
- BrokerMarket
- Option
- OrderSide
- PendingOrder
- Result
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
- Result
- String
- Arc
- Mutex
- Result
- Self
- StrategyManager
- Arc
- AtomicBool
- Drop
- Fixture
- JoinHandle
- Json
- Mutex
- Result
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
- Result
- Option
- Arc
- BrokerId
- ExecutedOrder
- KisRestClient
- NaiveDate
- Option
- OverseasExecutedOrder
- Result
- String
- Vec
- Option
- PendingOrder
- Result
- Option
- PendingOrder
- Result
- Arc
- BrokerId
- KisRestClient
- Mutex
- Option
- Result
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
- Result
- String
- Option
- String
- TradingHealthStatus
- Arc
- Mutex
- Option
- P
- Result
- Self
- Result
- Self
- BrokerMarket
- Arc
- BrokerId
- KisRestClient
- Option
- Result
- String
- Weak
- Arc
- AtomicUsize
- Drop
- Fixture
- JoinHandle
- Notify
- Result
- Self
- String
- Value
- Option
- OrderResponse
- OrderSide
- Result
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
- Result
- S
- Self
- String
- Uuid
- Value
- Vec
- KisRestClient
- Option
- Result
- Self
- String
- Fixture
- OrderSide
- String
- Option
- OrderSide
- PendingOrder
- Result
- String
- Vec
- Arc
- HashMap
- HashSet
- KisRestClient
- Mutex
- Option
- OrderSide
- Result
- RiskManager
- RwLock
- Self
- String
- TradeGuard
- Value
- Weak
- Option
- OrderManager
- Result
- Arc
- Mutex
- Option
- OrderManager
- Result
- Option
- OrderSide
- RestOrderSide
- String
- Arc
- KisRestClient
- OrderResponse
- Result
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
- Result
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
- Result
- Result
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
- Result
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
- Option
- String
- Tz
- Utc
- Vec
- DateTime
- Utc
- Option
- SimulationAssumptions
- DateTime
- Default
- FixedOffset
- Option
- RiskManager
- Self
- String
- Vec
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
- Result
- Self
- String
- Value
- Option
- Result
- String
- Value
- EntryContext
- EntryFilterPreparation
- HashMap
- Option
- Result
- Self
- String
- Value
- Value
- Default
- Self
- VolatilityExpansionPreparation
- Default
- Option
- Result
- Self
- String
- Value
- Option
- Result
- String
- Value
- Default
- HashMap
- Option
- Result
- Self
- String
- Value
- VecDeque
- VolatilityExpansionPreparation
- Value
- Default
- Result
- Self
- String
- Value
- HashMap
- Option
- Result
- Self
- String
- Value
- Option
- VecDeque
- Value
- Default
- HashMap
- Option
- Result
- Self
- String
- Value
- VecDeque
- Value
- Value
- Value
- Default
- Result
- Self
- String
- Value
- HashMap
- Option
- Result
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
- Result
- String
- Value
- F
- LeveragedTrendHoldPreviewSignal
- Option
- String
- Vec
- Default
- HashMap
- LeveragedTrendHoldPreviewSignal
- Option
- Self
- String
- Value
- Vec
- VecDeque
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
- Result
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
- Result
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
- Result
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
- Result
- Self
- String
- Value
- HashMap
- Option
- Result
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
- Result
- Self
- String
- Value
- Vec
- VecDeque
- Default
- Option
- Result
- Self
- String
- Value
- Option
- Result
- String
- Value
- Value
- Option
- Result
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
- Result
- String
- Value
- VecDeque
- Default
- EntryContext
- EntryFilterPreparation
- HashMap
- Option
- Result
- Self
- String
- Value
- VecDeque
- EntryFilterPreparation
- Value
- EntryFilterPreparation
- Option
- Result
- VecDeque
- Default
- Option
- Result
- Self
- String
- Value
- Option
- Result
- String
- Value
- EntryContext
- EntryFilterPreparation
- HashMap
- Option
- Result
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
- Value
- Vec
- Into
- Option
- Self
- String
- Value
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
- BrokerAdapterResult
- DateTime
- From
- Option
- Result
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
- Result
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
- Result
- RwLock
- Self
- String
- Vec
- Arc
- BrokerId
- Option
- Path
- Result
- Self
- String
- Vec
- Default
- Option
- Path
- PathBuf
- Result
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
- Json
- Option
- Path
- Query
- Result
- String
- Value
- PathBuf
- Result
- Self
- String
- Vec
- NaiveDate
- Option
- R
- Result
- String
- Vec
- Result
- Default
- From
- Option
- Result
- Self
- String
- Vec
- Arc
- Mutex
- NaiveDate
- Path
- PathBuf
- Result
- T
- Option
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
- Option
- Path
- PathBuf
- Result
- RwLock
- Self
- StockSearchItem
- String
- Vec
- Mutex
- Path
- PathBuf
- Result
- Self
- Vec
- BrokerId
- HashMap
- Option
- String
- Value
- Vec
- Client
- Option
- Result
- String

## God Nodes (most connected - your core abstractions)
1. `invoke()` - 106 edges
2. `mockApi()` - 51 edges
3. `AppState` - 49 edges
4. `ServerState` - 47 edges
5. `preview_strategy_from_candles()` - 41 edges
6. `TossOpenApiClient` - 40 edges
7. `run_backtest()` - 39 edges
8. `KEYS` - 39 edges
9. `BrokerAccountId` - 38 edges
10. `prepare()` - 37 edges

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

## Communities (1794 total, 1457 thin omitted)

### Community 0 - "OwnershipLedger"
Cohesion: 0.09
Nodes (46): bars(), completed_252_keeps_final_high_and_uses_high_instead_of_close(), completed_volume_history_is_atomic_scoped_and_does_not_change_owned_protection(), config_aba_and_profile_scope_switch_cannot_revive_old_tickets(), configured_512_window_and_invalid_ohlc_are_not_mislabeled_ready(), consecutive_manager(), consecutive_pending_failure_midnight_and_config_refresh_preserve_held_exit_streak(), consecutive_scoped_daily_apply_preserves_live_streak_and_requires_verified_source() (+38 more)

### Community 1 - "shared/api/commands.ts"
Cohesion: 0.04
Nodes (95): handleLookupTossAccounts(), handleSubmit(), handleSubmit(), activateEmergencyStop(), addProfile(), checkConfig(), checkForUpdate(), checkTossOrderPreflight() (+87 more)

### Community 2 - "rest/types.rs"
Cohesion: 0.05
Nodes (25): FailedBreakoutState, FailedBreakoutStrategy, encode(), restore(), V1, V2, actual_quantity_unknown_origin_has_no_invented_failure_anchor_or_age(), atr_never_weakens_legacy_percent_limb() (+17 more)

### Community 3 - "RiskManager"
Cohesion: 0.03
Nodes (81): canUseTauriEvents(), useBackendEvents(), useKisExecutedByRange(), useOverseasExecutedByRange(), useStatsByRange(), frontendLogger, useWriteFrontendLog(), OVERSEAS_CHART_PRESETS (+73 more)

### Community 4 - "simulation/minute_timing_tests.rs"
Cohesion: 0.10
Nodes (4): DeviationStrategy, capacity(), DeviationState, decode_owned_observation()

### Community 5 - "TradeRecord"
Cohesion: 0.10
Nodes (28): buy(), checkpoint_rejects_old_run_and_configuration_generation_without_writing(), deviation_feedback_pending_fill_partial_and_restart_preserve_exact_owner_age(), feedback_assigns_only_exact_owner_amid_manual_and_other_account_holdings(), feedback_cold_lth_restores_metadata_and_partial_change_preserves_live_high(), feedback_never_transfers_same_account_quantity_across_profile_or_mode(), key(), manager() (+20 more)

### Community 6 - "FiftyTwoWeekHighStrategy"
Cohesion: 0.04
Nodes (81): useRefreshStockList(), useStockSearch(), useTossMarketCalendar(), usePreviewLeveragedTrendHold(), SliderWithInput(), BollingerControls(), handlePickerSelect(), handlePickerUsSearch() (+73 more)

### Community 7 - "StrongCloseStrategy"
Cohesion: 0.14
Nodes (6): legacy_params(), MomentumEntryMode, MomentumParams, MomentumState, MomentumStrategy, validate_momentum_entry_params()

### Community 8 - "TrendFilterStrategy"
Cohesion: 0.11
Nodes (6): config(), prep(), initializer_restores_previous_averages_for_first_evaluation_cross(), insufficient_history_has_no_previous_long_average(), MaCrossState, MovingAverageCrossStrategy

### Community 9 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.17
Nodes (19): brokerChartResultsView(), syntheticBrokerChartInput(), syntheticSectorPortfolioInput(), setup(), brokerChartFixtureReport(), blockUnsafe(), openPreparation(), requestFixture() (+11 more)

### Community 10 - "OrderManager"
Cohesion: 0.07
Nodes (29): scripts, build, build:app, build:app:debug, build:research, build:web, build:web:research, check:fsd (+21 more)

### Community 11 - "ui/AppShell.tsx"
Cohesion: 0.07
Nodes (31): react, useRecentLogs(), SettingsState, queryClient, LEVEL_COLORS, Log(), LogLevel, AppLogEntry (+23 more)

### Community 12 - "toss/types.rs"
Cohesion: 0.15
Nodes (18): Handshake, RootCertStore, SocketAddr, builder_with_roots(), client(), client_builder(), error_chain(), http_auth_and_order_posts_are_not_resubmitted_after_response_disconnect() (+10 more)

### Community 13 - "toss_market.rs"
Cohesion: 0.06
Nodes (61): SectorDaily, SectorPortfolioResult, SectorRank, SectorTrade, plan_sector_portfolio(), rank_sectors(), return_n(), DefensiveState (+53 more)

### Community 14 - "DatabaseManager"
Cohesion: 0.05
Nodes (45): MySqlPool, category_from_key(), database_scanner_preserves_original_payload_order_and_checksum_input(), DatabaseManager, DatabasePool, document_key(), install_database_manager(), inventory_from_documents() (+37 more)

### Community 15 - "market_hours.rs"
Cohesion: 0.11
Nodes (32): parse_minute_instant(), diagnose_minute_timing(), local(), LthMinuteTimingView, MinuteLabelConvention, MinuteTimingDayView, MinuteTimingExample, MinuteTimingInput (+24 more)

### Community 16 - "database_io.rs"
Cohesion: 0.10
Nodes (36): LeveragedRapidReboundSnapshot, LeveragedReboundSnapshot, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldPreviewStep, LeveragedTrendHoldTimedCandle, lth_default_adx_period(), lth_default_breakeven_buffer(), lth_default_buy_adx() (+28 more)

### Community 17 - "BrokerAccountId"
Cohesion: 0.08
Nodes (32): TossAccount, TossApiResponse, TossCandle, TossCandlePageResponse, TossCommission, TossCost, TossCurrencyAmount, TossDailyProfitLoss (+24 more)

### Community 18 - "TossOpenApiClient"
Cohesion: 0.08
Nodes (6): ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, validate_entry_params(), validate_consecutive_entry_params(), bounded_window_with_extra()

### Community 19 - "observation.rs"
Cohesion: 0.15
Nodes (23): options, useBrokerChartScope(), useCollectBrokerChartData(), usePreviewBrokerChartPortfolio(), collectBrokerChartData(), getBrokerChartScope(), previewBrokerChartPortfolio(), BrokerChartCollection (+15 more)

### Community 20 - "accounts.rs"
Cohesion: 0.18
Nodes (16): buildTree(), compareNames(), currentDocument, fail(), generateInventory(), listRepositoryFiles(), mode, nextDocument (+8 more)

### Community 21 - "history/ui/Page.tsx"
Cohesion: 0.12
Nodes (18): order_side_label(), OrderManager, OrderSubmission, OwnedStrategySignal, PendingOrder, current_position_snapshot(), refresh_manual_positions(), PreparedOrderSubmission (+10 more)

### Community 22 - "page_view"
Cohesion: 0.37
Nodes (15): ToString, candle_market_date(), CandlePageInput, CandlePageView, completed_before(), decode_cursor(), domain_request(), get_candle_page() (+7 more)

### Community 23 - "toss_market.rs"
Cohesion: 0.15
Nodes (15): IntoIterator, StdMutex, applies_retry_after_to_group_pause(), epoch_ms_now(), rate_limit_reset_delay(), RateLimitGroupState, RateLimitGroupStatus, RateLimitScheduler (+7 more)

### Community 24 - "ServerState"
Cohesion: 0.05
Nodes (40): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, Korean, app, security (+32 more)

### Community 26 - "StrategyManager"
Cohesion: 0.11
Nodes (3): MeanReversionStrategy, MeanReversionState, bounded_window()

### Community 27 - "RunFence"
Cohesion: 0.11
Nodes (26): get_active_toss_calendar_override(), get_toss_chart_data(), get_toss_chart_data_for_profile(), get_toss_market_calendar(), get_toss_market_calendar_for_profile(), get_toss_market_snapshot(), get_toss_market_snapshot_for_profile(), get_toss_stock_safety() (+18 more)

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
Cohesion: 0.18
Nodes (13): fmt(), fmtUsd(), History(), HistoryQueryStatusProps, isOverseasTrade(), renderSlippage(), renderTradeMoney(), today() (+5 more)

### Community 32 - "RsiStrategy"
Cohesion: 0.12
Nodes (8): B, TossOpenApiClient, toss_error_display_preserves_structured_and_fallback_literals(), toss_typed_rejections_keep_the_existing_status_boundary(), format_toss_error(), toss_rate_limit_group(), validate_toss_order_id(), TossCredentials

### Community 33 - "settings.rs"
Cohesion: 0.11
Nodes (23): UpdateInfo, AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_trading_type(), DetectTradingTypeResult (+15 more)

### Community 34 - "order/ownership/tests.rs"
Cohesion: 0.14
Nodes (24): maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request(), daily_diagnostic_contains_one_completed_bar_and_no_synthetic_entry_observation(), minute_replay_warmup_excludes_replay_day_and_future_daily_candles(), BrokerCandle, BrokerClientOrderId, BrokerCurrency (+16 more)

### Community 35 - ".submit_order_shared"
Cohesion: 0.07
Nodes (23): config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope(), daily_buy_order_limit_is_disabled_even_after_submission_count_increases(), daily_order_limit_isolated_by_broker_account_scope(), daily_sell_order_limit_blocks_after_submission_count_reaches_limit(), DailyOrderCountEntry, DailyOrderSide (+15 more)

### Community 36 - "strategy_preview.rs"
Cohesion: 0.10
Nodes (3): live_high_excludes_unconfirmed_latest_bar_without_shortening_252_requirement(), owned_target_matches(), StrategyManager

### Community 37 - "server/records.rs"
Cohesion: 0.29
Nodes (5): Deserialize, encode(), Protection, restore(), VolatilityExpansionState

### Community 38 - "databaseManagementSection.tsx"
Cohesion: 0.12
Nodes (27): automatic_missing_context_and_stale_reservation_leave_budget_and_pending_unchanged(), buy(), delayed_old_account_insufficient_balance_does_not_suspend_new_account(), legacy_configured_account_without_profile_allows_manual_but_fences_client_replacement(), local_holdings(), local_order(), local_single_order(), LocalProvider (+19 more)

### Community 39 - "DatabaseManager"
Cohesion: 0.12
Nodes (18): account_sync_error(), chart_handler(), ChartQuery, invalid_order_input(), order_handler(), OrderBody, overseas_chart_handler(), overseas_order_handler() (+10 more)

### Community 40 - "AppState"
Cohesion: 0.09
Nodes (18): AtomicU64, Future, Output, capture_cycle(), ExecutionCycle, RunFence, configuration_save_rollback_aba_invalidates_quote_and_position_feedback(), delayed_name_and_late_rollback_do_not_mutate_the_new_run() (+10 more)

### Community 41 - "DatabaseConfig"
Cohesion: 0.24
Nodes (12): MySqlSslMode, PgSslMode, cached_connection_snapshot_reports_current_matching_settings(), concurrent_first_connections_create_one_pool(), connect_pool(), connecting_old_target_is_closed_and_not_cached_after_config_save(), create_missing_mariadb_database(), create_missing_postgres_database() (+4 more)

### Community 42 - "MomentumStrategy"
Cohesion: 0.36
Nodes (29): assumptions(), bar(), buy(), directional_fees_and_native_minimum_are_converted_with_the_current_fx(), dividend_adjusted_prices_reject_known_double_counting(), dividend_entitlement_precedes_pending_open_and_respects_partial_exit_quantity(), future_fx_point_cannot_revalue_earlier_mark_or_buy(), fx_changes_krw_mark_and_realized_basis_without_changing_native_strategy_entry() (+21 more)

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
Cohesion: 0.16
Nodes (12): account_less_scope_matches_any_account_of_same_broker(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), scope(), scoped_record_matches_only_same_broker_account(), TradeMarket (+4 more)

### Community 48 - "autoTradingBudgetSection.tsx"
Cohesion: 0.06
Nodes (57): useBalance(), useBrokerHoldings(), useOverseasBalance(), useTodayStats(), useTradesByRange(), useExchangeRate(), useExchangeRateStatus(), usePendingOrders() (+49 more)

### Community 49 - "FailedBreakoutStrategy"
Cohesion: 0.22
Nodes (28): age_keeps_accumulating_when_option_off_and_survives_warmup_parameter_edits(), age_starts_at_actual_fill_and_counts_only_owned_evaluations(), buy(), config(), crossing_at_boundary_requires_previous_strictly_above(), disabled_unknown_and_other_symbol_observations_cannot_change_held_age(), downward_crossing_retries_zero_feedback_and_cancels_above_threshold(), filled() (+20 more)

### Community 50 - "PositionTracker"
Cohesion: 0.07
Nodes (45): DividendScenarioPoint, FxScenarioPoint, Snapshot, apply_slippage(), BacktestPhaseView, BacktestReportView, BacktestSummaryView, BacktestTradeView (+37 more)

### Community 51 - "ReplaySession"
Cohesion: 0.27
Nodes (13): run_backtest_with_history(), SimulationEvent, atr_history(), atr_risk_uses_previous_history_close_and_evaluation_gap_without_trading_history(), atr_sizing_assumptions(), atr_sizing_never_silently_uses_requested_quantity_when_ranges_are_missing(), daily_loss_and_consecutive_loss_state_roll_over_on_event_date(), deterministic_replay_applies_costs_and_separates_signals_from_fills() (+5 more)

### Community 52 - "leveraged_trend_hold.rs"
Cohesion: 0.10
Nodes (6): config(), FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy, FiftyTwoWeekState, strong_close_ready_is_distinct_from_condition_and_preserves_positions(), volatility_requires_full_range_window_and_preserves_position()

### Community 53 - "AppState"
Cohesion: 0.22
Nodes (24): bar(), buy(), buy_blocked(), config(), daily(), sell(), strategy(), volatility_expansion_actual_fill_only_and_remaining_sell_reoffers() (+16 more)

### Community 54 - "security.rs"
Cohesion: 0.11
Nodes (39): CandlePageView, FetchFuture, Fn, TargetKey, collect_daily_history(), collect_pages(), CollectedDailyHistory, completed_bar() (+31 more)

### Community 55 - "VolatilityExpansionStrategy"
Cohesion: 0.13
Nodes (5): AccountProfile, load_secure_config(), non_empty(), ProfilesConfig, SecureConfig

### Community 56 - "simulation.rs"
Cohesion: 0.40
Nodes (18): assumptions(), assumptions_normalize_nonfinite_protection_and_expose_selected_model(), atr_that_only_becomes_ready_after_execution_cannot_size_the_open_order(), bar(), both_touch_uses_stop_first_only_when_selected_and_reports_ambiguity(), buy(), default_same_close_legacy_numeric_fixture_is_preserved(), delayed_fill_uses_next_observed_open_and_final_signal_remains_unfilled() (+10 more)

### Community 57 - "BrokerSymbol"
Cohesion: 0.07
Nodes (3): Display, select_price_response(), TossBrokerAdapter

### Community 58 - "order/budget.rs"
Cohesion: 0.10
Nodes (33): BudgetCurrencyView, BudgetRecoveryItem, allocation_cannot_withdraw_committed_cash_or_overflow(), AutoTradingBudgetView, budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent(), BudgetAccount, BudgetCurrencyView, BudgetLedger (+25 more)

### Community 59 - "OrderManager"
Cohesion: 0.15
Nodes (11): Barrier, daily_order_limit_key(), estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), OrderManager, order_exchange_code(), OrderManager (+3 more)

### Community 60 - "OrderRecord"
Cohesion: 0.10
Nodes (6): live_feedback_price_condition_changes_position_only_after_confirmed_fill(), strategy(), PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick()

### Community 61 - "settings/ui/Page.tsx"
Cohesion: 0.07
Nodes (37): useSetTradeArchiveConfig(), useTradeArchiveConfig(), useTradeArchiveStats(), useLogConfig(), useSetLogConfig(), useSetStockUpdateInterval(), useStockListStats(), useBrokerRateLimitStatus() (+29 more)

### Community 62 - "run_backtest"
Cohesion: 0.35
Nodes (13): all_strategies_keep_the_same_evaluation_window_without_implicit_warmup(), complete_daily_history_initializes_supported_strategies_without_shortening_defaults(), cross_and_breakout_seed_only_final_bars_are_not_conditions_not_met(), first_ready_bar_is_observed_after_tick_without_discarding_evaluation_bars(), generic_lth_is_rejected_before_live_clock_or_candle_preparation(), input(), insufficient_history_is_not_reported_as_a_zero_trade_strategy(), intraday_daily_indicators_do_not_claim_readiness() (+5 more)

### Community 63 - "DatabasePool"
Cohesion: 0.20
Nodes (13): OverseasPriceResponse, read_kis_response_text(), ExecutedOrder, HeaderMap, OrderResponse, OverseasExecutedOrder, PriceResponse, RequestBuilder (+5 more)

### Community 64 - "TradeRecord"
Cohesion: 0.23
Nodes (24): adding_a_target_does_not_transfer_other_strategy_or_manual_holdings(), cancelled_waiter_does_not_cancel_durable_save_and_publication(), delayed_save_pauses_ticks_without_locks_and_stop_can_complete(), deviation_options_reject_before_save_and_integer_float_values_roundtrip(), empty_price_condition_entries_reject_held_removal_without_touching_disk(), empty_saved_profile_does_not_inherit_price_condition_or_lth_entries(), failed_post_save_restore_blocks_all_new_orders_and_reports_persistence_error(), held_effective_target_removal_is_rejected_before_persistence() (+16 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.11
Nodes (27): assert_auth(), broker_chart_daily_response_keeps_read_and_size_errors_fixed(), candle_values(), chart_response_handles_empty_output_and_malformed_multibyte_body(), chart_response_preserves_http_and_provider_errors(), chart_response_propagates_announced_and_observed_body_caps(), domestic_latest_first_fixture_reverses_and_maps_ohlcv(), domestic_request_keeps_date_bounds_period_adjustment_and_query_order() (+19 more)

### Community 67 - "scripts"
Cohesion: 0.12
Nodes (44): activate_emergency_handler(), app_config_handler(), check_config_handler(), check_update_handler(), clear_buy_suspension_handler(), clear_emergency_handler(), exchange_rate_handler(), exchange_rate_status_handler() (+36 more)

### Community 68 - "build_strategy"
Cohesion: 0.15
Nodes (7): DailyHistoryEntry, DailyHistoryStatus, DailyHistoryTicket, daily_history_configuration_keeps_exact_json_and_digest(), fingerprint(), pending_status(), StrategyManager

### Community 69 - "commands/toss.rs"
Cohesion: 0.19
Nodes (22): align_accounting_scenarios(), broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc() (+14 more)

### Community 70 - "strategyPolicyControls.tsx"
Cohesion: 0.08
Nodes (48): DeviationControls(), DeviationNumericField(), deviationPolicyError(), DeviationPolicySummary(), fieldError(), fields, isDeviationPolicyParam(), modeOf() (+40 more)

### Community 71 - "commands/risk.rs"
Cohesion: 0.22
Nodes (7): PendingCandidate, pending_order_provider(), OrderManager, validate_pending_kis_mode(), OrderManager, ReconciliationSnapshot, same_profile()

### Community 72 - ".new"
Cohesion: 0.19
Nodes (19): AppConfigView, ProfileView, add_profile(), AddProfileInput, apply_active_profile(), broker_market_sort_key(), BrokerHoldingView, BrokerMoneyView (+11 more)

### Community 73 - "strategyPreviewPanel.tsx"
Cohesion: 0.11
Nodes (29): usePreviewStrategy(), usePreviewStrategyContinuous(), assertPageScope(), calendarLabel(), CandleCollectionEnd, candleTimestamp(), checkCancelled(), collectContinuousCandles() (+21 more)

### Community 74 - "KisWebSocketClient"
Cohesion: 0.08
Nodes (20): FormatTime, FrontendLogInput, get_recent_logs(), get_stats_by_range(), get_trades_by_range(), GetStatsByRangeInput, GetTradesByRangeInput, save_trade() (+12 more)

### Community 75 - "OhlcCandle"
Cohesion: 0.30
Nodes (21): age_starts_after_positive_fill_and_survives_same_partial_and_warmup_feedback(), buy(), config(), current_inclusive_population_bands_and_strict_boundaries_are_preserved(), disabled_unknown_symbol_and_reset_preserve_observation_contract(), exact_metadata_roundtrips_pending_and_confirmed_age_without_inventing_ownership(), filled(), hold() (+13 more)

### Community 76 - "read_json_or_default"
Cohesion: 0.06
Nodes (25): Strategy(), dashboardRoute, historyRoute, logRoute, Register, rootRoute, router, routeTree (+17 more)

### Community 77 - "database_projection.rs"
Cohesion: 0.11
Nodes (34): MySql, Postgres, Transaction, drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract() (+26 more)

### Community 78 - "ConsecutiveMoveStrategy"
Cohesion: 0.16
Nodes (13): classify_token_response(), detect_trading_type(), DetectedTradingType, DetectTokenReq, DetectTradingTypeError, is_paper_key_rejected_by_real_domain(), is_real_key_rejected_by_paper_domain(), mentions_app_key() (+5 more)

### Community 79 - "project-map.mjs"
Cohesion: 0.21
Nodes (16): ChartRequest, ChartRequest<'a>, KisRestClient, parse_broker_chart_daily_body(), parse_broker_chart_daily_response(), parse_domestic_response(), parse_overseas_response(), parse_sector_daily_body() (+8 more)

### Community 80 - "start_trading"
Cohesion: 0.30
Nodes (19): preview_strategy_continuous_from_candles(), actual_indicator_warmup_and_state_survive_more_than_500_bars_and_arbitrary_partitions(), actual_strategy_single_and_chunk_supply_match_and_pending_orders_cross_both_boundaries(), bounded_request_rejects_empty_or_oversized_chunks_history_and_total(), candle(), changing_future_chunk_prices_preserves_all_earlier_strategy_fills_and_equity(), common_phase_diagnostics_and_warning_match_flat_single_and_partitioned_replays(), consecutive_realized_losses_preserve_the_strategy_symbol_block_across_chunks() (+11 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.18
Nodes (16): collect_kis_pending_fills(), collect_toss_pending_fills(), domestic_order_is_terminal(), domestic_order_terminal_status(), domestic_provider_status(), kis_pending_from(), normalized_execution_u64(), execution_quantity() (+8 more)

### Community 82 - "MovingAverageCrossStrategy"
Cohesion: 0.11
Nodes (5): parse(), RsiParams, validate_rsi_params(), RsiState, RsiStrategy

### Community 83 - "accountHooks.ts"
Cohesion: 0.09
Nodes (44): AccountConfigRefreshError, accountScope(), holdingsPrefixes, profileMutationKey, profileMutationScope, reconcileRejectedProfileMutation(), refreshAccountConfig(), resetAccountHoldings() (+36 more)

### Community 84 - "shared/api/types.ts"
Cohesion: 0.20
Nodes (19): checkpoint_validation_is_atomic_and_kind_separated(), cold_owned_age_stays_unknown_and_active_cap_offers_conservative_exit(), config(), damaged_optional_runtime_fails_buy_closed_and_retains_custom_core_and_valid_exits(), disabled_cap_still_counts_known_age_and_reconfigure_does_not_reset_it(), entry(), flat_sell_and_held_buy_are_removed_without_mutating_actual_position(), known_age_starts_after_actual_fill_and_preserves_partial_same_quantity_retries() (+11 more)

### Community 85 - "strategyResearchPanel.tsx"
Cohesion: 0.10
Nodes (30): clearExperimentSlots(), compactSnapshot(), ExperimentScope, loadExperimentSlots(), normalizePart(), readIndex(), saveExperimentSlot(), storageKey() (+22 more)

### Community 86 - "collect_kis_pending_fills"
Cohesion: 0.11
Nodes (9): advance_owned_age(), age_preserves_pending_partial_same_quantity_and_unknown_origins(), begin_live_trend(), both_kinds_preserve_exact_checkpoint_shape_and_pending_fill_origin(), checkpoints_reject_cross_kind_unknown_fields_and_contradictions(), encode_owned_observation(), observe_trend(), prior_trend_and_first_live_clear_remain_bounded_and_saturating() (+1 more)

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
Cohesion: 0.13
Nodes (10): DateRangeQuery, frontend_log_handler(), FrontendLogBody, kis_executed_handler(), set_archive_config_handler(), set_log_config_handler(), SetArchiveConfigBody, SetLogConfigBody (+2 more)

### Community 92 - "strategyResearchTypes.ts"
Cohesion: 0.08
Nodes (28): BacktestAtrPreparation, BacktestPhase, BacktestReport, BacktestSummary, BacktestTrade, ChunkContinuityView, EquityPoint, LthMinuteTimingDay (+20 more)

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
Cohesion: 0.12
Nodes (22): useChartData(), useTossChartData(), buildChartOptions(), CandlePoint, CHART_PRESETS, ChartPreset, ChartSource, ChartType (+14 more)

### Community 102 - "BrokerAccountId"
Cohesion: 0.15
Nodes (17): args, assertGitClean(), assertTagAvailable(), branchName, changedFiles, currentBranchName(), escapeRegExp(), fail() (+9 more)

### Community 103 - "TossAccessToken"
Cohesion: 0.13
Nodes (14): money(), assertDialogScope(), currentScope(), ExpectedMutation, focusedOpener(), ScopeSnapshot, setup(), snapshot() (+6 more)

### Community 104 - "dependencies"
Cohesion: 0.08
Nodes (25): @emotion/react, @emotion/styled, lightweight-charts, @mui/icons-material, @mui/material, dependencies, @emotion/react, @emotion/styled (+17 more)

### Community 105 - "model/tradingStore.ts"
Cohesion: 0.31
Nodes (10): defaults, fieldError(), isTrendFilterParam(), NumericField(), pairError(), Props, TrendFilterControls(), trendFilterParamError() (+2 more)

### Community 106 - "TossApiError"
Cohesion: 0.18
Nodes (13): AccessToken, kis_token_client_never_forwards_post_body_to_redirect_destination(), Arc, Client, DateTime, Mutex, Option, Result (+5 more)

### Community 108 - "model/accountStore.ts"
Cohesion: 0.17
Nodes (24): SectorPrice, DefensiveState, BTreeMap, Option, SectorCorporateAction, SectorDay, String, Vec (+16 more)

### Community 109 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.16
Nodes (9): validates_order_create_request_shape(), TossOrderCreateRequest, new_toss_client_order_id(), validate_client_order_id(), validate_optional_decimal(), validate_order_side(), validate_order_type(), validate_time_in_force() (+1 more)

### Community 110 - "NotificationService"
Cohesion: 0.29
Nodes (8): StrategyView, get_positions(), get_strategies(), PositionView, update_strategy(), UpdateStrategyInput, build_strategy_view(), StrategyView

### Community 111 - "exchange.rs"
Cohesion: 0.24
Nodes (12): request(), ConfigurationEditReservation, ConfigurationUpdateError, ConfigurationUpdateRequest, ConfigurationUpdateState, effective_target_markets(), run_update(), spawn_update() (+4 more)

### Community 112 - "toss_currency_from_view"
Cohesion: 0.19
Nodes (16): LeveragedTrendHoldPreviewInput, TossModifyOrderInput, TossOpenOrdersInput, TossOrderPreflightInput, TossSmallBuyVerificationInput, active_profile(), leveraged_trend_hold_preview_handler(), toss_chart_handler() (+8 more)

### Community 113 - "strategy/tests.rs"
Cohesion: 0.13
Nodes (35): assert_private(), body_limit_preserves_header_stream_and_exact_boundary(), broker_chart_toss_raw_provider_reuses_token_and_sends_quote_only_contract(), candles(), chunked(), encoded_response(), example(), explicit_identity_and_optional_next_cursor_remain_valid() (+27 more)

### Community 114 - "day_event_tests.rs"
Cohesion: 0.14
Nodes (22): CollectSectorDataInput, check_scope(), collect_sector_data(), error(), get_sector_quote_scope(), NativeQuoteSource, QuoteSnapshot, Arc (+14 more)

### Community 115 - "setup-local.sh"
Cohesion: 0.09
Nodes (18): OrderRecord, OrderSide, OrderStatus, OrderStore, parallel_appends_do_not_lose_order_records(), BalanceSnapshot, BalanceStore, HoldingItem (+10 more)

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
Cohesion: 0.31
Nodes (6): ExchangeRateSource, ExchangeRateView, get_exchange_rate(), get_exchange_rate_status(), refresh_exchange_rate_status(), resolve_exchange_rate_policy()

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
Cohesion: 0.14
Nodes (9): TossOrder, TossOrderExecution, TossOrderListQuery, TossOrderListStatus, TossOrderModifyRequest, TossOrderOperationResponse, TossOrderResponse, TossPaginatedOrderResponse (+1 more)

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
Cohesion: 0.21
Nodes (11): parse_amount_f64(), fetch_account_risk_balance_krw(), fetch_toss_risk_balance_krw(), clear_buy_suspension(), get_trading_status(), sync_kis_strategy_positions(), sync_strategy_positions_from_active_broker(), sync_toss_strategy_positions() (+3 more)

### Community 129 - "MarketDayCalendar"
Cohesion: 0.15
Nodes (8): maps_holding_to_broker_domain(), restart_rebuilds_order_counts_and_consecutive_losses_from_ledgers(), OrderManager, PreparedOwnershipFill, realized_pnl(), risk_receipts_preserve_fixed_keys_payloads_and_restored_deduplication(), broker_scope_serializes_with_camel_case_fields(), BrokerAccountId

### Community 130 - "DailyHistoryTicket"
Cohesion: 0.21
Nodes (9): BrokerCandlePage, CandleCursor, CandlePageRequest, kis_cursor_date(), next_toss_cursor(), previous_kis_cursor(), toss_cursor_before(), toss_inclusive_overlap_is_preserved_but_repeated_and_forward_cursors_stop() (+1 more)

### Community 131 - "validate_pending_kis_mode"
Cohesion: 0.15
Nodes (21): preview_strategy_from_candles(), accounting_scenario_labels_match_actual_evaluation_instants_without_shifting(), candle(), daily_rows_keep_existing_trading_dates_and_hash_semantics(), frozen_qqq_cost_and_next_open_feedback_change_the_real_strategy_exit_path(), future_open_cannot_change_an_earlier_signal_or_fill(), input(), legacy_compact_minutes_are_explicit_kst_and_invalid_times_fail_closed() (+13 more)

### Community 132 - "BrokerScope"
Cohesion: 0.29
Nodes (13): fetch_overseas_tick(), fetch_toss_tick(), finish_closed_batch(), market_readiness(), MarketReadiness, is_market_closed_error(), poll_symbols_tick(), PriceSource (+5 more)

### Community 133 - ".search"
Cohesion: 0.23
Nodes (11): app(), bearer_token(), constant_time_eq(), cross_origin_mutation_is_rejected_even_with_token(), has_same_origin(), mutation_without_token_is_rejected(), parse_bool_env(), rejection() (+3 more)

### Community 134 - "run_backtest_with_history"
Cohesion: 0.22
Nodes (3): mean_close(), Counters, EntryContext

### Community 135 - "AppConfigView"
Cohesion: 0.18
Nodes (6): BollingerParams, LeveragedTrendHoldEntry, LeveragedTrendHoldMarketState, LeveragedTrendHoldParams, LeveragedTrendHoldPosition, LeveragedTrendHoldStrategy

### Community 136 - "작업 기록: <짧은 제목>"
Cohesion: 0.36
Nodes (13): bar(), config(), context_refresh_failure_and_readiness_queries_never_reseed_cross_observations(), cross_state(), current_completed_daily_arrives_after_decision_and_duplicates_are_ignored(), disabled_filters_keep_golden_death_cross_quantity_and_metadata(), malformed_optional_does_not_replace_custom_legacy_periods_or_sell(), minute_seed_and_live_quotes_never_manufacture_completed_daily_context() (+5 more)

### Community 137 - "StatsStore"
Cohesion: 0.09
Nodes (22): fixture, realm(), DailyStats, mockSettingsApi(), apply(), Control, dates(), expectZero() (+14 more)

### Community 138 - ".process_buy"
Cohesion: 0.22
Nodes (11): application_error_status_and_envelopes_remain_distinct(), local_views_keep_casing_types_and_never_return_the_web_api_token(), pending_view_matches_the_ipc_projection_without_losing_scope_or_partial_quantity(), preview_input(), profiles_risk_and_strategy_views_match_production_ipc_builders_exactly(), Value, supplied_candle_preview_rest_matches_the_same_ipc_calculation(), production_kis_routes_preserve_capture_query_headers_and_response_mapping() (+3 more)

### Community 139 - "CurrencyLedger"
Cohesion: 0.38
Nodes (13): bar(), candle_page_wire_query_preserves_camel_case_and_encoded_cursor(), completed_daily_pages_drop_current_and_future_bars_and_deduplicate_before_replay(), completed_provider_minute_page_runs_through_production_replay(), current_week_and_month_are_conservatively_unconfirmed(), cursors_bind_scope_and_cutoff_and_reject_future_or_changed_contract(), daily_timestamps_use_market_date_and_reject_digit_prefix_garbage(), input() (+5 more)

### Community 140 - "verify-lockfiles.mjs"
Cohesion: 0.35
Nodes (13): bar(), failed_breakout_candidate_high_low_volume_cannot_influence_its_own_entry(), failed_breakout_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average(), failed_breakout_entry_atr_keeps_common_sizing_and_sells_actual_adjusted_quantity(), failed_breakout_hash_distinguishes_unknown_daily_volume_from_verified_zero(), failed_breakout_next_open_promotes_original_anchor_after_same_zero_feedback(), failed_breakout_optional_params_are_strict_in_both_previews_and_daily_filters_reject_minutes(), failed_breakout_partition_and_future_suffix_are_causal_for_all_options() (+5 more)

### Community 141 - "modify_toss_order_for_profile"
Cohesion: 0.32
Nodes (13): broker_money_view_amount(), confirmed_small_order_fill(), ConfirmedSmallOrderFill, is_toss_order_final(), money_view_from_decimal(), poll_toss_order_detail(), small_buy_cap(), storage_money_amount() (+5 more)

### Community 142 - "deviation/tests.rs"
Cohesion: 0.31
Nodes (13): absent_unknown_and_similarly_prefixed_kis_tr_ids_preserve_pending_without_querying(), actual_fill_crosses_same_scope_restart_with_identical_provider_snapshot(), delayed_reconciliation_cannot_mutate_new_scope_and_applies_when_original_is_reselected(), direct_fill_and_zero_fill_terminal_cannot_mutate_another_execution_scope(), exact_eight_order_tr_ids_select_mode_and_toss_bypasses_kis_mode_classification(), executed(), persisted_kis_pending_never_queries_or_applies_in_the_opposite_mode(), Provider (+5 more)

### Community 143 - "HARNESS_MAP.md"
Cohesion: 0.48
Nodes (6): attach_comparison(), BuyHoldComparisonView, comparator_preserves_actual_trades_and_discloses_final_pending(), CostRestoredReferenceView, ReplayComparisonView, same_quantity_reference_discloses_exposure_and_restores_only_recorded_costs()

### Community 144 - "2. 목표와 비목표"
Cohesion: 0.25
Nodes (12): fieldError(), fields, isStrongClosePolicyParam(), numericLabel(), profitPairError(), StrongCloseControls(), StrongCloseNumericField(), strongClosePolicyError() (+4 more)

### Community 145 - "react-grid-layout"
Cohesion: 0.18
Nodes (12): buildSnapshot(), fail(), listRepositoryFiles(), mode, parseMode(), REPOSITORY_ROOT, ROOT_CONFIGS, SCRIPT_DIR (+4 more)

### Community 146 - "api/mod.rs"
Cohesion: 0.29
Nodes (8): Band, candle(), disabled_option_preserves_entries_and_buffer_is_bounded(), downward_breakout_and_insufficient_history_block_entries(), flat_squeeze_needs_completed_upward_breakout(), LeveragedTrendHoldStrategy, live_and_replay_share_confirmed_breakout_and_exit_without_future_prices(), strategy()

### Community 148 - "Arc"
Cohesion: 0.11
Nodes (19): bodyHtml(), checkFreshness(), escapeHtml(), inlineMarkdown(), outputPath, parseTasks(), renderTodo(), root (+11 more)

### Community 149 - "AtomicBool"
Cohesion: 0.19
Nodes (3): StockEntry, StockListStats, StockStore

### Community 150 - "ChartCandle"
Cohesion: 0.21
Nodes (10): Requests, accept(), actual_submissions_sell_only_the_confirmed_owner_and_never_add_fill_to_account_snapshot(), ownership_recheck_after_preparation_prevents_a_stale_sell_reservation(), publish(), Fixture, Json, OrderSide (+2 more)

### Community 151 - "restore_risk_from_today_trades"
Cohesion: 0.13
Nodes (10): kis_read_min_interval(), KisOrderRejected, KisRestClient, Arc, AtomicBool, Duration, RwLock, Self (+2 more)

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
Cohesion: 0.25
Nodes (5): market_from_currency(), toss_currency(), toss_market(), TossBuyingPower, TossPriceResponse

### Community 156 - "strategy-continuous-replay.spec.ts"
Cohesion: 0.11
Nodes (16): CandlePageView, mockApi(), MockOptions, mockResearchResult(), readyPreparation, strategyEntries, setup(), failures (+8 more)

### Community 158 - "breakout.rs"
Cohesion: 0.15
Nodes (31): assertNoResultsSelection(), captureResults(), compareSameContextCaret(), expectResultsSelection(), focusFirstResult(), installResultsSourceHarness(), keyboardResultsFixture(), nativeActivationCounter() (+23 more)

### Community 159 - "Strategy"
Cohesion: 0.22
Nodes (11): BrokerPositionSnapshot, CompletedDailyBar, EntryFilterPreparation, HistoryReadiness, initialize_strategy_warmup_with_daily_bars(), OhlcCandle, Signal, Strategy (+3 more)

### Community 160 - "collect_pages"
Cohesion: 0.22
Nodes (3): lth_default_qty(), LeveragedTrendHoldStrategy, PreviewPosition

### Community 161 - ".checkpoint_strategy_positions_shared"
Cohesion: 0.22
Nodes (6): clear_database_tables(), drop_database_tables(), ensure_trading_stopped(), export_database_to_json(), import_json_to_database(), set_storage_backend()

### Community 162 - ".capture"
Cohesion: 0.36
Nodes (12): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+4 more)

### Community 163 - ".collect"
Cohesion: 0.15
Nodes (13): useOverseasChartData(), buildChartOptions(), CandlePoint, ChartType, CrosshairData, LinePoint, OverseaChartPreset, OVERSEAS_CHART_PRESETS (+5 more)

### Community 164 - "collect_daily_history"
Cohesion: 0.24
Nodes (15): Fault, fixture_input(), fixture_manifest(), SectorDataManifest, Self, ValidateSectorDataInput, sector_data_manifest_requires_exact_past_window_and_manual_mapping(), sector_data_mock_collection_is_sequential_paired_anchored_and_bounded() (+7 more)

### Community 165 - "toss/orders.rs"
Cohesion: 0.29
Nodes (6): OverseasBalanceItem, OverseasBalanceSummary, BalanceResult, get_balance(), get_overseas_balance(), OverseasBalanceResult

### Community 166 - "Result"
Cohesion: 0.38
Nodes (10): bar(), request(), run(), volatility_expansion_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average(), volatility_expansion_common_atr_sizing_remains_and_sells_confirmed_adjusted_quantity(), volatility_expansion_daily_full_range_is_current_input_while_prior_mean_excludes_it(), volatility_expansion_diagnostic_serialization_and_other_strategy_omission(), volatility_expansion_optional_validation_both_previews_and_daily_only_preserved() (+2 more)

### Community 167 - "Self"
Cohesion: 0.30
Nodes (4): corrupted_primary_never_resurrects_orders_from_a_valid_backup(), PendingOrderStore, restart_snapshot_preserves_scope_client_id_and_fill_watermark(), snapshot()

### Community 168 - "String"
Cohesion: 0.14
Nodes (13): Builder, head_and_options_keep_existing_origin_and_method_policy(), origin_host_and_exact_bearer_rules_are_preserved_before_body_parsing(), spa_fallback_assets_head_and_unknown_api_behavior_are_preserved(), unsupported_methods_keep_405_and_allow_for_every_registered_url(), sector_data_rest_equals_ipc_and_does_not_access_account_or_provider(), sector_data_rest_rejects_auth_and_large_body_and_reports_not_ready(), budget_save_requires_explicit_profile_and_mode_and_accepts_current_context() (+5 more)

### Community 169 - "T"
Cohesion: 0.27
Nodes (4): ExecutionRun, ExecutionRunContext, OrderManager, validate_snapshot()

### Community 171 - "Vec"
Cohesion: 0.14
Nodes (28): captureSearchRace(), expectNoSearchError(), expectNoSearchSelection(), expectSearchSelection(), failedSearchReply(), installKeyboardScopeHarness(), invalidSearchReply(), KeyboardRaceFixture (+20 more)

### Community 172 - "Arc"
Cohesion: 0.42
Nodes (15): assumptions(), bar(), buy(), diagnostics(), diagnostics_are_optional_camel_case_and_do_not_rewrite_legacy_phase_fields(), empty_single_and_all_in_sample_preserve_existing_phase_shape(), fx_dividend_partial_exit_preserves_original_entry_cost_and_reconciles_split(), near() (+7 more)

### Community 173 - "Client"
Cohesion: 0.44
Nodes (11): candle(), config(), daily_hooks_do_not_create_unknown_symbol_states(), strong_close_completed_day_uses_actual_position_and_keeps_stop_loss(), strong_close_uses_each_completed_day_only_on_the_following_day(), volatility(), volatility_collects_missing_history_after_close_for_later_days(), volatility_compares_current_full_range_against_prior_completed_window() (+3 more)

### Community 174 - "lth_replay_clock.rs"
Cohesion: 0.20
Nodes (14): date(), diagnostic_message(), hash(), note(), prices(), NaiveDate, Result, SectorDataBundle (+6 more)

### Community 175 - "Mutex"
Cohesion: 0.41
Nodes (11): bar(), complete_day(), completed_high_not_close_sets_next_reference_without_lookahead(), config(), fifty_two_week_requires_full_window_and_uses_last_completed_high_and_close(), initialization_keeps_last_window_and_daily_bookkeeping_is_symbol_scoped(), many_intraday_prices_and_intraday_initializers_do_not_age_daily_highs(), missing_history_becomes_ready_only_after_252_completed_daily_highs() (+3 more)

### Community 177 - "Result"
Cohesion: 0.23
Nodes (13): SectorPageReceipt, collect_mode(), collect_sector_bundle(), price(), BTreeMap, Result, S, SectorDataBundle (+5 more)

### Community 178 - "Self"
Cohesion: 0.29
Nodes (6): active_slope_bounds_legacy_long_period_and_required_context(), finite(), legacy_params(), MaCrossParams, validate(), validate_ma_cross_entry_params()

### Community 179 - "String"
Cohesion: 0.29
Nodes (7): Client, toss_http_client(), trim_base_url(), shared_toss_token_state(), toss_token_state_key(), TossTokenState, toss_rate_limiter()

### Community 180 - "integer"
Cohesion: 0.26
Nodes (9): MeanReversionExitMode, MeanReversionParams, parse(), validate_mean_reversion_params(), integer(), parse(), percent(), TrendFilterParams (+1 more)

### Community 181 - "AppHandle"
Cohesion: 0.18
Nodes (9): args, artifactDir, artifacts, checksumPath, manifestPath, names, records, requireSignatures (+1 more)

### Community 182 - "preview_strategy_from_candles"
Cohesion: 0.44
Nodes (9): hysteresisError(), hysteresisOf(), isMomentumEntryParam(), modeOf(), MomentumEntryControls(), momentumEntryParamError(), MomentumEntrySummary(), parsedDraft() (+1 more)

### Community 183 - ".submit_order_shared"
Cohesion: 0.09
Nodes (39): certificates(), command(), containers, directory, environment, interruption, results, root (+31 more)

### Community 184 - "AccountProfile"
Cohesion: 0.35
Nodes (7): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, storage_money_units(), TossManualSession

### Community 185 - "FixedOffset"
Cohesion: 0.38
Nodes (11): date(), days(), history(), holdings(), input(), NaiveDate, Result, SectorDay (+3 more)

### Community 186 - "RwLock"
Cohesion: 0.15
Nodes (22): broker_chart_collector_accepts_identical_inclusive_overlap_but_rejects_conflict(), broker_chart_collector_default_kis_uses_12_single_ams_routes_and_exports_master_evidence(), broker_chart_collector_freezes_latest_42_spy_sessions_and_marks_unknown_not_verified(), broker_chart_collector_invalid_kis_mapping_or_any_toss_mapping_stops_before_fetch(), broker_chart_collector_manual_mapping_overrides_seed_and_evidence_changes_hash(), broker_chart_collector_manual_route_mapping_never_falls_back_or_accepts_partial_mapping(), broker_chart_collector_never_hides_sector_missing_date_by_intersection(), broker_chart_collector_never_substitutes_route_after_auth_http_rate_generic_rejection_or_oversize() (+14 more)

### Community 187 - "Self"
Cohesion: 0.29
Nodes (10): chart_volume_to_u64(), normalize_minute_time(), normalize_ordered_rows(), normalize_rows(), normalize_rows_with_order(), signal_to_preview_view(), StrategyPreparationView, StrategyPreviewInput (+2 more)

### Community 188 - "Sender"
Cohesion: 0.13
Nodes (16): chart(), Control, daily, data(), domestic(), enter(), latest(), minutes (+8 more)

### Community 189 - "String"
Cohesion: 0.42
Nodes (9): bar(), candidate_current_range_cannot_change_its_own_prior_atr_filter(), guard_rejection_does_not_recreate_a_consumed_golden_cross(), integer_float_daily_period_stays_active_and_missing_history_blocks_only_golden_cross(), ma_cross_options_are_strict_in_short_and_continuous_preview_paths(), non_daily_filters_reject_explicitly_and_disabled_minute_compatibility_remains(), request(), run() (+1 more)

### Community 190 - "checked_budget_scope"
Cohesion: 0.22
Nodes (19): BudgetRecoveryView, approve_budget_recovery(), AutoTradingBudgetInput, budget_error(), checked_budget_scope(), checked_budget_update_scope(), get_auto_trading_budget(), get_budget_recovery() (+11 more)

### Community 191 - "BrokerAdapterResult"
Cohesion: 0.06
Nodes (58): useOverseasPrice(), usePrice(), useTossMarketSnapshot(), useTossStockSafety(), useModifyTossOrder(), usePlaceOrder(), usePlaceOverseasOrder(), useSubmitTossSmallBuyVerification() (+50 more)

### Community 192 - "failed_breakout_entry_tests.rs"
Cohesion: 0.51
Nodes (10): bar(), continuous(), params(), request(), rsi_cap_counts_after_actual_next_open_fill_across_chunk_boundaries(), rsi_cap_reoffers_after_actual_minimum_profit_guard_refusal(), rsi_filtered_cross_does_not_retry_and_default_has_no_flat_sell_or_held_buy(), rsi_next_open_uses_real_fill_basis_and_stops_before_original_cross() (+2 more)

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
Cohesion: 0.21
Nodes (5): queryClient, theme, SectorPortfolioPanel(), client, client

### Community 197 - "DeviationStrategy"
Cohesion: 0.47
Nodes (8): build_provider_request(), prepare_buy_submission(), prepare_order_submission(), prepare_sell_submission(), rest_order_side(), toss_order_side(), toss_price_string(), PrepareDecision

### Community 198 - "toss/orders.rs"
Cohesion: 0.27
Nodes (10): ChunkContinuityView, run_prepared_preview(), lth_preview_assessment(), as_of(), assessment_uses_prepared_lexical_tail_and_distinguishes_price_exclusions(), candle(), daily_omits_timing_and_empty_minute_is_not_evaluable(), legacy_selection_stays_at_200_and_does_not_refill_rejected_price_rows() (+2 more)

### Community 199 - "Self"
Cohesion: 0.47
Nodes (7): accepted_http_response_publishes_before_its_event_under_the_same_gate(), balance(), delayed_http_a_to_b_to_a_response_never_publishes_or_emits(), delayed_http_response_cannot_undo_a_concurrent_buy_or_sell(), invalid_holdings_do_not_clear_the_previous_position_or_emit_success(), refresh(), Server

### Community 200 - "consecutive/tests.rs"
Cohesion: 0.27
Nodes (6): every_read_observes_current_primary_and_structural_errors_are_not_empty_ledgers(), fill(), missing_primary_starts_empty_and_real_file_restart_keeps_exact_idempotent_fill(), primary_corruption_blocks_instead_of_recovering_an_old_valid_backup(), primary_loss_is_detected_after_restart_even_with_no_pending_orders_and_a_valid_backup(), primary_only_creation_crash_gap_anchors_the_existing_generation_and_ownership()

### Community 201 - "trend_filter_protection.rs"
Cohesion: 0.50
Nodes (4): encode(), Protection, restore(), TrendFilterState

### Community 202 - "Arc"
Cohesion: 0.22
Nodes (5): layerRank, layers, root, srcRoot, violations

### Community 203 - "BalanceItem"
Cohesion: 0.22
Nodes (6): appEntry, escapedName, npmLock, pkg, rootDir, workspaceLock

### Community 204 - "ui/OverseasStockChart.tsx"
Cohesion: 0.20
Nodes (23): BrokerChartPrice, BrokerChartCollection, BrokerChartPageReceipt, collect_broker_chart(), CollectBrokerChartInput, hash(), kis_default_mappings(), RawBrokerChartPage (+15 more)

### Community 205 - "config"
Cohesion: 0.14
Nodes (21): CollectBrokerChartInput, check_scope(), collect_broker_chart_data(), error(), get_broker_chart_scope(), NativeSource, preview_broker_chart_portfolio(), Arc (+13 more)

### Community 206 - "input"
Cohesion: 0.33
Nodes (9): buy_preflight_adds_commission_and_blocks_when_cash_is_short(), buy_preflight_passes_when_required_cash_is_available(), evaluate_order_preflight(), format_money_amount(), invalid_commission_ratios_block_instead_of_becoming_zero_fee(), OrderPreflightConstraints, OrderPreflightDecision, OrderPreflightInput (+1 more)

### Community 207 - ".prepare_confirmed_ownership"
Cohesion: 0.33
Nodes (8): apply(), bounded_receipts_and_numeric_overflow_reject_without_partial_mutation(), rebuild_replays_durable_trades_then_missing_intent(), roundtrip(), scope(), scope_is_part_of_event_identity_and_payload_conflicts_fail_closed(), serialized_retries_and_intent_before_risk_are_exactly_once(), stale_runtime_and_legacy_runtime_have_no_current_receipts()

### Community 211 - "uiux-history-details.spec.ts"
Cohesion: 0.20
Nodes (10): expected, krw, krwSecond, readAllDetails(), records, search(), setup(), trigger() (+2 more)

### Community 212 - ".new"
Cohesion: 0.53
Nodes (9): bar(), continuous(), deviation_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal(), deviation_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry(), deviation_new_fields_are_strict_in_both_production_preview_paths(), deviation_protective_age_counts_actual_held_observations_across_chunks(), deviation_stop_priority_uses_real_fill_basis_and_next_open_execution(), params() (+1 more)

### Community 213 - "HashMap"
Cohesion: 0.25
Nodes (6): endpointInventory, EXPECTED_PATHS, hasRateLimitHeaders, missingPaths, paths, servers

### Community 214 - "HeaderMap"
Cohesion: 0.15
Nodes (12): node, vite.config.ts, vite.research.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution (+4 more)

### Community 215 - "broker/candles.rs"
Cohesion: 0.53
Nodes (9): bar(), continuous(), mean_reversion_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal(), mean_reversion_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry(), mean_reversion_new_fields_are_strict_in_both_production_preview_paths(), mean_reversion_protective_age_counts_actual_held_observations_across_chunks(), mean_reversion_stop_priority_uses_real_fill_basis_and_next_open_execution(), params() (+1 more)

### Community 216 - "StockStore"
Cohesion: 0.16
Nodes (5): build_router(), Router, Fixture, mock_provider(), WireRequests

### Community 217 - "작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단"
Cohesion: 0.42
Nodes (8): bar(), request(), run(), trend_filter_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average(), trend_filter_confirmation_resets_on_recovery_and_chunk_preserves_consecutive_exits(), trend_filter_partition_and_future_suffix_are_causal_for_all_options(), trend_filter_profit_arms_on_actual_avg_and_reoffers_after_actual_guard_refusal(), trend_filter_strict_validation_in_both_previews_and_minute_support()

### Community 218 - "budget/recovery/tests.rs"
Cohesion: 0.42
Nodes (9): build_strategy(), config(), live_feedback_deviation_manager_observations_match_replay_and_ownership_requires_feedback(), live_feedback_manager_pending_and_unsettled_suppress_only_outward_signal_across_restart(), live_feedback_manager_protection_metadata_requires_exact_owner_and_valid_checkpoint(), live_feedback_manager_targets_exact_owner_scope_and_market(), live_feedback_mean_reversion_manager_observations_match_replay_and_ownership_requires_feedback(), live_feedback_rsi_unfilled_and_ma_share_exact_replay_observations() (+1 more)

### Community 220 - "PendingOrderStore"
Cohesion: 0.60
Nodes (5): calculate_sector_portfolio(), preview_sector_portfolio(), CmdResult, SectorPortfolioInput, SectorPortfolioReport

### Community 221 - "BrokerHoldingView"
Cohesion: 0.15
Nodes (16): historyFixture(), card(), fillFilters(), filterParams, insufficientEntry, legacyParams, Params, response() (+8 more)

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
Cohesion: 0.39
Nodes (8): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), calculate_atr(), initialize_active_strategy_history(), legacy_history_symbols(), resolve_scoped_profile()

### Community 227 - "strong_close_history_tests.rs"
Cohesion: 0.26
Nodes (11): card(), continuous(), labels, legacyParams, Params, policyParams, response(), runButton() (+3 more)

### Community 228 - "lth_replay_clock.rs"
Cohesion: 0.58
Nodes (8): delayed_open_fill_reaches_lth_before_close_and_last_exit_remains_unfilled(), future_bar_does_not_change_prior_signal(), history(), params(), run(), session_and_blackout_use_input_minutes(), timed(), timed_replay_has_nonempty_repeatable_signals_and_execution_results()

### Community 229 - "Self"
Cohesion: 0.40
Nodes (4): main(), Box, Error, Result

### Community 230 - "initialize_strategy_warmup"
Cohesion: 0.64
Nodes (6): initialize_strategy_warmup(), config(), daily(), warmup_mean_reversion_uses_close_bands_and_first_signal(), warmup_preserves_positions_and_ignores_unrelated_symbol(), warmup_trend_filter_uses_200_closes_and_first_signal()

### Community 231 - "Vec"
Cohesion: 0.24
Nodes (19): CollectSectorDataInput, Option, SectorCorporateAction, SectorPortfolioInput, String, Vec, SectorAdjustedPrice, SectorDataBundle (+11 more)

### Community 232 - "Arc"
Cohesion: 0.19
Nodes (19): BrokerChartAdjustment, BrokerChartBroker, BrokerChartCurrency, BrokerChartDay, BrokerChartInput, BrokerChartInterval, BrokerChartPrice, BrokerChartReport (+11 more)

### Community 234 - "Client"
Cohesion: 0.13
Nodes (11): CostSource, DividendPriceBasis, DividendScenarioPoint, FxScenarioPoint, ReplayAccountingView, SimulationAssumptions, valid_time(), validate_labels() (+3 more)

### Community 236 - "Option"
Cohesion: 0.07
Nodes (25): BalanceResult, OverseasBalanceResult, mockUx(), UxHandler, UxResponse, assertReadOnly(), assertTossGates(), domestic (+17 more)

### Community 239 - "String"
Cohesion: 0.26
Nodes (12): BrokerChartClient, number(), row(), Arc, BrokerChartDay, ChartCandle, KisRestClient, Option (+4 more)

### Community 240 - "T"
Cohesion: 0.50
Nodes (8): atr_uses_gap_true_ranges_fourteen_denominator_and_prior_close(), bandwidth_uses_twenty_population_samples_and_inclusive_boundary(), bar(), large_native_prices_do_not_overflow_daily_statistics_and_invalid_ohlc_fail_closed(), p(), repeated_rejection_reasons_overlap_without_inflating_candidate_count(), strict_trend_and_inclusive_slope_use_prior_daily_values(), sufficient_zero_ranges_are_ready_rejections_while_missing_history_is_unready()

### Community 241 - "Vec"
Cohesion: 0.21
Nodes (14): strategy(), card(), chooseMode(), continuous(), crossingParams, Harness, labels, legacyParams (+6 more)

### Community 242 - "CandleDataSummary.tsx"
Cohesion: 0.33
Nodes (6): CandleDataSummary(), CandlePoint, focusStyle, number(), Projection, Props

### Community 244 - "MockSource"
Cohesion: 0.47
Nodes (5): MockSource, Mutex, Result, String, Vec

### Community 245 - "StatusCode"
Cohesion: 0.15
Nodes (7): zustand, AccountState, useAccountStore, TradingState, TradingStatus, useTradingStore, zustand

### Community 246 - "String"
Cohesion: 0.14
Nodes (20): Cow, EventLoopProxy, Retained, app_context(), copy_dir_all(), Path, Result, run() (+12 more)

### Community 248 - "Client"
Cohesion: 0.16
Nodes (19): accountingAssumptionsLabel(), AccountingAssumptionSummary(), AccountingResults(), ComparisonResults(), EditorProps, numberInRange(), parseScenarios(), percent() (+11 more)

### Community 249 - "Response"
Cohesion: 0.27
Nodes (11): bounded(), main(), ProbeSource, Option, Path, PathBuf, Result, String (+3 more)

### Community 250 - "Result"
Cohesion: 0.50
Nodes (6): restore_risk_from_today_trades(), risk_rebuild_isolates_paper_profit_from_live_losses_for_the_same_account(), risk_rebuild_seeds_trade_event_receipts_and_retries_missing_pending_intent_once(), risk_rebuild_uses_chronological_profits_and_keeps_manual_emergency_stop(), trade(), unknown_mode_or_runtime_save_failure_never_publishes_a_partial_rebuild()

### Community 251 - "main"
Cohesion: 0.40
Nodes (4): main(), Box, Error, Result

### Community 252 - "Option"
Cohesion: 0.13
Nodes (24): useCollectSectorData(), useSectorQuoteScope(), useValidateSectorData(), collectSectorData(), getSectorQuoteScope(), validateSectorData(), CollectSectorDataInput, SectorDataBundle (+16 more)

### Community 253 - "Result"
Cohesion: 0.09
Nodes (30): usePreviewSectorPortfolio(), previewSectorPortfolio(), DefensiveState, SectorDaily, SectorDay, SectorHolding, SectorPlan, SectorPortfolioInput (+22 more)

### Community 254 - "Self"
Cohesion: 0.23
Nodes (13): basis(), card(), continuous(), labels, legacyParams, Params, policyParams, ready (+5 more)

### Community 255 - "String"
Cohesion: 0.48
Nodes (6): assertSimulationSafe(), ack(), collectButton(), preparation(), scopeButton(), selectedScope()

### Community 256 - "Vec"
Cohesion: 0.17
Nodes (11): app, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, bundle, shortDescription (+3 more)

### Community 257 - "BrokerCurrency"
Cohesion: 0.26
Nodes (11): card(), continuous(), filterParams, labels, legacyParams, Params, response(), runButton() (+3 more)

### Community 258 - "BrokerMarket"
Cohesion: 0.11
Nodes (29): main(), Box, Error, Result, accounting_input(), preview_broker_chart_price_approximation(), BrokerChartInput, BrokerChartReport (+21 more)

### Community 259 - "Option"
Cohesion: 0.24
Nodes (11): AutoTradingBudgetInput, budget_handler(), budget_response(), AutoTradingBudgetView, CmdError, Json, Query, Response (+3 more)

### Community 260 - "Result"
Cohesion: 0.26
Nodes (12): card(), chooseMode(), continuous(), labels, legacyParams, Params, policyParams, response() (+4 more)

### Community 261 - "String"
Cohesion: 0.46
Nodes (7): blocked_strong_close_buy_does_not_discard_next_days_completed_condition(), candle(), daily_event_strategies_reject_other_cadences(), flat_completed_days_fill_history_without_compressing_the_window(), input(), strong_close_uses_previous_completed_day_and_keeps_position_across_day_changes(), volatility_uses_previous_days_rolling_ranges_and_independent_today_ohlc()

### Community 262 - "config"
Cohesion: 0.43
Nodes (7): bar(), changed_candidate_ohlc_range_cannot_change_its_own_prior_atr_entry_filter(), default_diagnostic_is_omitted_while_threshold_and_preparation_failures_are_distinct(), frozen_spy_default_cost_paths_preserve_predeclared_exit_dates(), input(), integer_valued_json_float_periods_remain_enabled_in_production_preview(), invalid_filter_params_and_non_daily_ohlc_filter_requests_are_explicit_errors()

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
Cohesion: 0.44
Nodes (6): holding(), scope(), toss_fractional_and_zero_quantities_never_create_a_whole_share(), toss_rejects_invalid_quantities_and_prices_including_float_u64_boundary(), toss_rejects_wrong_broker_account_currency_and_duplicate_market_symbol(), toss_us_prices_round_to_cents_and_kr_prices_round_to_won()

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
Cohesion: 0.39
Nodes (5): FailedBreakoutBasis, FailedBreakoutParams, number(), parse(), validate_failed_breakout_params()

### Community 275 - "strategyAccountingPanel.tsx"
Cohesion: 0.52
Nodes (6): broker_chart_input(), broker_chart_rest_equals_pure_ipc_and_rejects_missing_price_or_ack(), Value, sector_portfolio_missing_price_rejects_the_whole_comparison_and_large_body(), sector_portfolio_rest_equals_ipc_without_state_or_provider_access(), supplied_input()

### Community 277 - "strategy_preview/tests.rs"
Cohesion: 0.18
Nodes (7): currency, destination(), returnToContents(), risk, sections, setup(), toc()

### Community 278 - "collect_daily_history"
Cohesion: 0.16
Nodes (15): ConsecutiveEntryControls(), ConsecutiveEntryField(), consecutiveEntryFiltersDisabled(), consecutiveEntryParamError(), ConsecutiveEntrySummary(), fieldError(), fields, isConsecutiveEntryParam() (+7 more)

### Community 279 - "CmdError"
Cohesion: 0.38
Nodes (4): BrokerAdapter, BrokerAdapterError, default_place_order_returns_unsupported_with_broker_id(), DummyAdapter

### Community 281 - "From"
Cohesion: 0.43
Nodes (6): ChunkContinuityView, ContinuousStrategyPreviewChunk, ContinuousStrategyPreviewInput, ContinuousStrategyPreviewView, invalid(), ReplayContinuityView

### Community 282 - "Option"
Cohesion: 0.57
Nodes (6): config(), failed_breakout_held_target_removal_is_rejected_and_successful_edit_preserves_age(), failed_breakout_optional_validation_precedes_save_and_valid_values_roundtrip(), failed_breakout_save_replace_failure_rolls_back_without_losing_owned_basis_or_age(), failed_breakout_state(), update()

### Community 284 - "Self"
Cohesion: 0.40
Nodes (5): Json, StatusCode, ValidateSectorDataInput, Value, validate_handler()

### Community 286 - "String"
Cohesion: 0.52
Nodes (6): config(), deviation_counts_current_price_as_last_required_observation(), deviation_history_is_ready_before_first_evaluation_and_bounded(), momentum_counts_first_comparison_but_excludes_seed_prices(), momentum_history_does_not_create_or_erase_order_state(), rsi_initializer_restores_crossing_baseline_without_historical_signals()

### Community 287 - "Vec"
Cohesion: 0.38
Nodes (4): DeviationEntryMode, DeviationParams, parse(), validate_deviation_entry_params()

### Community 291 - "trading/ui/Page.tsx"
Cohesion: 0.73
Nodes (5): check_toss_profile_connection(), run_toss_connection_diagnostic(), toss_diag_step(), TossConnectionDiagnostic, TossConnectionStep

### Community 293 - "scaled_total"
Cohesion: 0.73
Nodes (5): rsi_held_target_removal_is_rejected_and_successful_edit_preserves_age(), rsi_optional_validation_precedes_save_and_valid_values_roundtrip(), rsi_save_replace_failure_rolls_back_without_losing_owned_basis_or_age(), rsi_state(), update()

### Community 294 - "guard/recovery/tests.rs"
Cohesion: 0.47
Nodes (5): append_failed_order(), build_kis_pending_order(), build_pending_order(), build_toss_pending_order(), failed_order_price()

### Community 295 - "StrongCloseState"
Cohesion: 0.80
Nodes (5): config(), reconfigure_fifty_two_week_keeps_actual_entry_while_invalidating_old_window(), reconfigure_strong_close_preserves_stop_but_discards_old_entry_candidate(), reconfigure_volatility_recomputes_range_without_losing_held_stop_or_day(), reject_scope_changes()

### Community 296 - "run_prepared_preview"
Cohesion: 0.53
Nodes (4): number(), parse(), StrongCloseParams, validate_strong_close_params()

### Community 297 - "trendFilterControls.tsx"
Cohesion: 0.40
Nodes (4): encode(), Protection, restore(), StrongCloseState

### Community 298 - "KisBrokerAdapter"
Cohesion: 0.80
Nodes (5): config(), reconfigure_deviation_applies_new_threshold_to_retained_observations(), reconfigure_momentum_keeps_owned_quantity_and_retries_unfilled_exits(), reconfigure_rsi_recomputes_baseline_with_new_period_and_keeps_targets_separate(), reject_identity_changes()

### Community 304 - "Error"
Cohesion: 0.27
Nodes (9): CandleCollection, ContinuousStrategyPreviewChunk, ChartCandle, card(), chooseContinuous(), dailySeries(), researchResponse(), setup() (+1 more)

### Community 305 - "RwLock"
Cohesion: 0.40
Nodes (5): input(), BrokerChartInput, Result, String, text()

### Community 306 - "autoTradingBudgetSection.tsx"
Cohesion: 0.11
Nodes (24): useAutoTradingBudget(), useUpdateAutoTradingBudget(), RecoveryIdentity, useApproveBudgetRecovery(), useBudgetRecovery(), amount(), AutoTradingBudgetSection(), BudgetEditor() (+16 more)

### Community 307 - "String"
Cohesion: 0.60
Nodes (5): config(), consecutive_counts_current_price_and_uses_active_position_branch(), consecutive_history_preserves_flat_and_existing_positions(), failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation(), failed_breakout_history_retains_actual_breakout_reference()

### Community 308 - "CmdError"
Cohesion: 0.08
Nodes (31): File, AppState, balance_summary_total_krw(), CmdError, make_rest_client(), parse_amount_i64(), Arc, AtomicBool (+23 more)

### Community 309 - "CmdResult"
Cohesion: 0.60
Nodes (5): config(), live_feedback_breakout_checkpoint_restores_pending_and_confirmed_exact_anchor(), live_feedback_breakout_freezes_pending_signal_anchor_until_first_fill(), live_feedback_consecutive_keeps_confirmed_flag_and_observations(), live_feedback_unsubmitted_breakout_discards_only_candidate_not_history()

### Community 310 - "DatabaseConfigView"
Cohesion: 0.26
Nodes (8): gate(), keyboardObservationWindow(), KeyboardSearchFixture, keyboardStock, safeKeyboardSearch(), setup(), states, Surface

### Community 312 - "DatabaseTransferResult"
Cohesion: 0.47
Nodes (4): check(), GitHubRelease, is_newer(), UpdateInfo

### Community 313 - "lth-minute-timing.spec.ts"
Cohesion: 0.50
Nodes (3): LthMinuteTiming, card(), preview()

### Community 318 - "String"
Cohesion: 0.50
Nodes (3): TossApiError, TossErrorResponse, TossHttpError

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
Cohesion: 0.83
Nodes (3): cursor_default_count_and_unhashed_credentials_keep_the_fixed_identity(), cursor_scope_preserves_fixed_input_bytes_and_digest(), profile()

### Community 330 - "Option"
Cohesion: 0.83
Nodes (3): confirmed_small_fill_requires_original_order_symbol_side_currency_and_whole_share(), detail(), missing_amount_or_zero_fill_is_deferred_without_using_limit_or_average_as_money()

### Community 332 - "OrderSide"
Cohesion: 0.83
Nodes (3): place_overseas_with_retry_shared(), place_prepared_order(), place_with_retry_shared()

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

### Community 359 - ".capture"
Cohesion: 0.26
Nodes (14): basisOf(), BreakoutNumericField(), FailedBreakoutControls(), failedBreakoutDailyFiltersDisabled(), failedBreakoutPolicyError(), FailedBreakoutPolicySummary(), fieldError(), fields (+6 more)

### Community 369 - ".submit_order_shared"
Cohesion: 0.28
Nodes (9): body_snippet(), read_response_text(), read_toss_candle_response_text(), read_toss_response_text(), Option, Response, Result, String (+1 more)

### Community 373 - "strategy-strong-close-policy.spec.ts"
Cohesion: 0.22
Nodes (13): card(), continuous(), labels, legacyParams, Params, policyParams, ready, response() (+5 more)

### Community 384 - "install_database_manager"
Cohesion: 0.09
Nodes (37): AsyncRead, AsyncWrite, Pin, Poll, ReadBuf, Receiver, connect_stream(), KisWebSocketClient (+29 more)

### Community 402 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.23
Nodes (13): chartTimeToMs(), formatChartTimeKst(), formatKstMs(), pad2(), parseTimeMs(), Props, StrategyPreviewChart(), StrategyPreviewChartSignal (+5 more)

### Community 707 - "package.json"
Cohesion: 0.25
Nodes (7): engines, node, npm, name, private, type, version

## Knowledge Gaps
- **565 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+560 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **1457 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `BrokerSymbol` connect `order/ownership/tests.rs` to `holdings/tests.rs`, `trading/ui/Page.tsx`, `BrokerScope`, `commands/toss.rs`, `toss/orders.rs`, `submission.rs`, `DatabaseManager`, `holding`, `CurrencyLedger`, `String`, `strategy/tests.rs`, `page_view`, `RunFence`, `BrokerCurrency`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **Why does `BrokerAccountId` connect `MarketDayCalendar` to `conflicts.rs`, `OwnershipLedger`, `TradeRecord`, `holding`, `modify_toss_order_for_profile`, `order/ownership/tests.rs`, `trading/ui/Page.tsx`, `.submit_order_shared`, `DatabaseManager`, `Self`, `DiscordNotifier`, `security.rs`, `AccountProfile`, `order/budget.rs`, `checked_budget_scope`, `commands/risk.rs`, `.new`, `KisWebSocketClient`, `.prepare_confirmed_ownership`, `support.rs`, `budget/recovery/tests.rs`, `@mui/icons-material`, `Signal`, `Result`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **Why does `advance_owned_age()` connect `collect_kis_pending_fills` to `simulation/minute_timing_tests.rs`, `LeveragedTrendHoldStrategy`, `MovingAverageCrossStrategy`, `BrokerQuantity`, `StrategyManager`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._
- **Are the 37 inferred relationships involving `preview_strategy_from_candles()` (e.g. with `completed_provider_minute_page_runs_through_production_replay()` and `common_phase_diagnostics_and_warning_match_flat_single_and_partitioned_replays()`) actually correct?**
  _`preview_strategy_from_candles()` has 37 INFERRED edges - model-reasoned connections that need verification._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _565 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `OwnershipLedger` be split into smaller, more focused modules?**
  _Cohesion score 0.08743169398907104 - nodes in this community are weakly interconnected._
- **Should `shared/api/commands.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.03978561121418264 - nodes in this community are weakly interconnected._