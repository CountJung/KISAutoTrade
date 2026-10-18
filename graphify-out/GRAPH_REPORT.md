# Graph Report - KISAutoTrade  (2026-10-18)

## Corpus Check
- 330 files · ~237,425 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 5217 nodes · 11265 edges · 514 communities (226 shown, 288 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 513 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ba74e03d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- leveraged_trend_hold.rs
- simulation.rs
- hooks.ts
- shared/api/commands.ts
- RiskManager
- commands/trading.rs
- ConsecutiveMoveStrategy
- leveragedTrendHoldEditorPanel.tsx
- PositionTracker
- ui/AppShell.tsx
- OrderManager
- submission.rs
- dashboard/ui/Page.tsx
- trading/ui/Page.tsx
- TossOpenApiClient
- settings.rs
- toss/types.rs
- BrokerSymbol
- RsiStrategy
- fills.rs
- accounts.rs
- MeanReversionStrategy
- database_projection.rs
- toss_market.rs
- ServerState
- RateLimitScheduler
- strategy/ui/Page.tsx
- commands/toss.rs
- tauri.conf.json
- ui/StockChart.tsx
- StrategyManager
- BrokerScope
- server/market.rs
- settings/ui/Page.tsx
- KisRestClient
- AccountProfile
- src/api/types.ts
- rest/types.rs
- databaseManagementSection.tsx
- DatabaseManager
- AppState
- server/records.rs
- StrategyConfig
- domain.rs
- archive.rs
- commands/records.rs
- PriceConditionStrategy
- detect.rs
- submit_toss_small_buy_verification_for_profile
- market/mod.rs
- profiles.rs
- server/trading.rs
- OrderRecord
- StockStore
- security.rs
- dependencies
- logging/mod.rs
- active_profile
- compilerOptions
- commands/database.rs
- commands/market.rs
- DiscordNotifier
- DatabaseConfig
- DatabasePool
- TradeRecord
- release-version.mjs
- MovingAverageCrossStrategy
- scripts
- place_toss_order
- conflicts.rs
- toss/tests.rs
- commands/risk.rs
- .new
- storage/database.rs
- KisWebSocketClient
- toss/orders.rs
- read_json_or_default
- StatsStore
- devDependencies
- project-map.mjs
- TokenManager
- BrokerQuantity
- .write_document
- KisBrokerAdapter
- support.rs
- leveragedTrendHoldPreviewChart.tsx
- broker/adapter.rs
- ui/LayoutResizer.tsx
- initialize_active_strategy_history
- PendingOrderStore
- StrategyStore
- TradeStore
- database_keychain.rs
- @mui/icons-material
- balance_store.rs
- .load_sync
- check
- compilerOptions
- check-fsd-imports.mjs
- package.json
- verify-toss-openapi.mjs
- OverseasExecutedOrder
- BrokerAccountId
- TossAccessToken
- atomic_write
- TossApiError
- strategyResearchTypes.ts
- leveragedTrendHoldPreviewChart.tsx
- exchange.rs
- toss_currency_from_view
- strategy/tests.rs
- day_event_tests.rs
- build.rs
- main.rs
- settings-database.spec.ts
- kis-auto-trade
- playwright.config.ts
- sync-codex-skills.ps1
- discord-notification-config/index.ts
- log-filter/index.ts
- strategy-configure/index.ts
- strategy-toggle/index.ts
- Result
- PendingOrderStore
- vite.config.ts
- input
- .new
- .fmt
- .new
- .search
- run_backtest
- AppConfigView
- 작업 기록: <짧은 제목>
- write_json
- .process_buy
- CurrencyLedger
- modify_toss_order_for_profile
- OrderSubmission
- HARNESS_MAP.md
- 2. 목표와 비목표
- react-grid-layout
- api/mod.rs
- Result
- Arc
- AtomicBool
- ChartCandle
- Client
- Duration
- ExecutedOrder
- HeaderMap
- OrderResponse
- OverseasExecutedOrder
- PriceResponse
- Response
- place_toss_order
- collect_pages
- .checkpoint_strategy_positions_shared
- .capture
- ExchangeRateView
- collect_daily_history
- toss/orders.rs
- Self
- String
- T
- Value
- Vec
- Arc
- Client
- DateTime
- Mutex
- Option
- Result
- Utc
- AppHandle
- MutexGuard
- .submit_order_shared
- .new
- FixedOffset
- RwLock
- Self
- Sender
- String
- Vec
- BrokerAdapterResult
- modify_toss_order_for_profile
- order/configuration/tests.rs
- commands/risk.rs
- PreparedOrderSubmission
- MarketDayCalendar
- SubmissionDeps
- first_positive_u64
- Self
- config
- holding
- Arc
- BalanceItem
- StrategyStore
- config
- strategy/tests.rs
- .prepare_confirmed_ownership
- TossTokenState
- broker/mod.rs
- Arc
- .new
- HashMap
- HeaderMap
- TossAccessToken
- place_prepared_order
- 작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단
- Option
- commands.rs
- PendingOrderStore
- BrokerHoldingView
- BrokerAdapterResult
- BrokerCurrency
- BrokerId
- Default
- submission.rs
- config
- Result
- Self
- String
- Vec
- Arc
- BrokerCurrency
- Client
- Into
- Option
- Result
- Self
- String
- T
- Vec
- Error
- HeaderMap
- Option
- StatusCode
- String
- Value
- Client
- Response
- Result
- String
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
- balance-events.spec.ts
- From
- notional/tests.rs
- .record_fill_pnl_once
- config
- RiskManager
- commands.rs
- .new
- database/tests.rs
- config
- TossTokenState
- CmdError
- CmdResult
- From
- Option
- Result
- Self
- State
- String
- Vec
- CmdResult
- Default
- NaiveDate
- Option
- Path
- PathBuf
- Result
- Self
- State
- String
- Vec
- Arc
- BrokerId
- CmdError
- CmdResult
- Error
- RwLock
- State
- String
- CmdError
- CmdResult
- DatabaseConfigView
- DatabaseStatusView
- DatabaseTransferResult
- Error
- JsonStorageInventoryView
- SaveDatabaseConfigInput
- State
- StorageBackend
- String
- ChartCandle
- TossOrderbookView
- DatabaseManager
- submission.rs
- config/mod.rs
- .new
- StrategyStore
- .fmt
- preview_strategy
- CmdResult
- Option
- OrderResponse
- OrderSide
- OrderType
- Self
- State
- String
- CmdResult
- DailyStats
- ExecutedOrder
- Option
- OverseasExecutedOrder
- State
- String
- TradeRecord
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
- State
- String
- Arc
- BrokerCurrency
- BrokerId
- BrokerMarket
- BrokerMoneyView
- CmdResult
- Mutex
- Option
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
- Option
- OrderManager
- Result
- RwLock
- State
- String
- Value
- Vec
- Arc
- BrokerId
- Option
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
- Send
- Sync
- Into
- Option
- Self
- String
- AutoTradingBudgetView
- CmdError
- Json
- Query
- Response
- Result
- State
- Json
- Option
- OrderSide
- OrderType
- Path
- Query
- Response
- Result
- State
- String
- Value
- StockSearchItem
- String
- TradeArchiveConfig
- Value
- Vec
- NaiveDate
- Option
- Result
- String
- Vec
- Query
- Result
- State
- String
- Value
- BrokerCurrency
- BrokerId
- Json
- Option
- Path
- Result
- State
- StatusCode
- String
- Value
- Vec
- PathBuf
- Result
- Self
- String
- Vec
- NaiveDate
- Option
- Result
- String
- Path
- Result
- Option
- Result
- String
- NaiveDate
- PathBuf
- Result
- Self
- String
- Arc
- DatabaseConfigView
- DatabaseStatusView
- DatabaseTransferResult
- Duration
- Error
- JsonStorageInventoryView
- Mutex
- Option
- Path
- PathBuf
- Result
- RwLock
- SaveDatabaseConfigInput
- Self
- StorageBackend
- String
- Vec
- Result
- Option
- Self
- String
- trading/mod.rs
- Option
- PathBuf
- PendingOrder
- Result
- String
- Vec
- buffer_tests.rs
- high_tests.rs
- package.json
- kis-auto-trade

## God Nodes (most connected - your core abstractions)
1. `StrategyConfig` - 120 edges
2. `invoke()` - 92 edges
3. `BrokerScope` - 89 edges
4. `ServerState` - 81 edges
5. `AppState` - 76 edges
6. `Signal` - 59 edges
7. `AccountProfile` - 51 edges
8. `OhlcCandle` - 48 edges
9. `RiskManager` - 42 edges
10. `StrategyManager` - 42 edges

## Surprising Connections (you probably didn't know these)
- `BrokerAccountId` --calls--> `maps_holding_to_broker_domain()`  [INFERRED]
  src-tauri/src/broker/domain.rs → src-tauri/src/broker/toss/tests.rs
- `BrokerSymbol` --calls--> `maps_toss_candle_to_broker_candle()`  [INFERRED]
  src-tauri/src/broker/domain.rs → src-tauri/src/broker/toss/tests.rs
- `BrokerSymbol` --calls--> `selects_price_response_case_insensitively()`  [INFERRED]
  src-tauri/src/broker/domain.rs → src-tauri/src/broker/toss/tests.rs
- `BrokerSymbol` --calls--> `validates_market_data_query_limits_before_request()`  [INFERRED]
  src-tauri/src/broker/domain.rs → src-tauri/src/broker/toss/tests.rs
- `format_toss_error()` --calls--> `body_snippet()`  [INFERRED]
  src-tauri/src/broker/toss/error.rs → src-tauri/src/broker/toss/http.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/trading/simulation/assessment.rs -> src-tauri/src/trading/simulation/assessment.rs`
- 2-file cycle: `src-tauri/src/trading/simulation.rs -> src-tauri/src/trading/simulation/atr.rs -> src-tauri/src/trading/simulation.rs`
- 2-file cycle: `src-tauri/src/server/mod.rs -> src-tauri/src/server/profiles.rs -> src-tauri/src/server/mod.rs`

## Communities (514 total, 288 thin omitted)

### Community 0 - "leveraged_trend_hold.rs"
Cohesion: 0.20
Nodes (20): OwnershipPosition, OrderSide, OwnershipApplied, PositionEntry, OwnershipWatermark, AppliedRecord, Uuid, Vec (+12 more)

### Community 1 - "simulation.rs"
Cohesion: 0.04
Nodes (89): getAutoTradingBudget(), getAppConfig(), listProfiles(), addProfile(), updateProfile(), deleteProfile(), setActiveProfile(), getBalance() (+81 more)

### Community 2 - "hooks.ts"
Cohesion: 0.12
Nodes (27): KisOutput1, String, Option, T, Value, BalanceItem, Default, Self (+19 more)

### Community 3 - "shared/api/commands.ts"
Cohesion: 0.11
Nodes (20): RiskConfigState, default_daily_buy_order_limit(), default_daily_sell_order_limit(), default_max_consecutive_losses(), default_risk_per_trade_bps(), default_atr_stop_multiplier(), Self, I (+12 more)

### Community 4 - "RiskManager"
Cohesion: 0.03
Nodes (92): canUseTauriEvents(), useBackendEvents(), useAutoTradingBudget(), useUpdateAutoTradingBudget(), useKisExecutedByRange(), useOverseasExecutedByRange(), frontendLogger, useCandlePage() (+84 more)

### Community 5 - "commands/trading.rs"
Cohesion: 0.15
Nodes (22): TradeSide, TradeStatus, TradeMarket, default_trade_market(), default_currency(), String, TradeRecord, Option (+14 more)

### Community 6 - "ConsecutiveMoveStrategy"
Cohesion: 0.05
Nodes (20): FiftyTwoWeekHighParams, Default, Self, FiftyTwoWeekState, Option, VecDeque, FiftyTwoWeekHighStrategy, HashMap (+12 more)

### Community 7 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.05
Nodes (22): RsiParams, Default, Self, RsiStrategy, HashMap, String, VecDeque, Option (+14 more)

### Community 8 - "PositionTracker"
Cohesion: 0.06
Nodes (23): initialize_strategy_warmup(), MeanReversionParams, Default, Self, MeanReversionState, VecDeque, Option, MeanReversionStrategy (+15 more)

### Community 9 - "ui/AppShell.tsx"
Cohesion: 0.10
Nodes (13): DailyOrderSide, DailyOrderCountEntry, NaiveDate, String, StrategySymbolCountEntry, RiskRuntimeState, Vec, BTreeMap (+5 more)

### Community 10 - "OrderManager"
Cohesion: 0.08
Nodes (26): scripts, dev, build, build:web, preview, tauri, build:app, build:app:debug (+18 more)

### Community 11 - "submission.rs"
Cohesion: 0.05
Nodes (43): react, useRecentLogs(), useUpdateCheck(), SettingsState, useSettingsStore, LogLevel, LEVEL_COLORS, Log() (+35 more)

### Community 12 - "dashboard/ui/Page.tsx"
Cohesion: 0.08
Nodes (32): TossTokenRequest, TossTokenResponse, TossAccessToken, TossAccessTokenStatus, TossOpenApiOverview, TossApiResponse, TossExchangeRateResponse, TossStockInfo (+24 more)

### Community 13 - "trading/ui/Page.tsx"
Cohesion: 0.13
Nodes (24): TossTradeView, TossPriceLimitView, TossMarketSnapshotView, TossStockInfoView, TossStockWarningView, TossStockSafetyView, toss_warning_label(), normalize_toss_symbol() (+16 more)

### Community 14 - "TossOpenApiClient"
Cohesion: 0.18
Nodes (12): DatabaseManager, PathBuf, RwLock, AtomicBool, Mutex, Result, DatabaseStatusView, DatabaseTransferResult (+4 more)

### Community 15 - "settings.rs"
Cohesion: 0.15
Nodes (18): ExecutionCycle, Arc, KisRestClient, Option, Vec, String, capture_cycle(), Mutex (+10 more)

### Community 16 - "toss/types.rs"
Cohesion: 0.06
Nodes (19): ConsecutiveMoveParams, Default, Self, ConsecutiveMoveState, VecDeque, ConsecutiveMoveStrategy, HashMap, String (+11 more)

### Community 17 - "BrokerSymbol"
Cohesion: 0.12
Nodes (22): BrokerId, BrokerAccountId, BrokerMarket, BrokerCurrency, BrokerMoney, BrokerOrderId, BrokerClientOrderId, BrokerOrderSide (+14 more)

### Community 18 - "RsiStrategy"
Cohesion: 0.10
Nodes (8): TossOpenApiClient, B, format_toss_error(), read_toss_response_text(), toss_rate_limit_group(), validate_toss_symbol(), validate_toss_order_id(), TossCredentials

### Community 19 - "fills.rs"
Cohesion: 0.10
Nodes (7): TossOrderCreateRequest, new_toss_client_order_id(), maps_holding_to_broker_domain(), validates_order_create_request_shape(), selects_price_response_case_insensitively(), maps_toss_candle_to_broker_candle(), validates_market_data_query_limits_before_request()

### Community 20 - "accounts.rs"
Cohesion: 0.18
Nodes (16): SCRIPT_DIR, REPOSITORY_ROOT, PROJECT_MAP_PATH, VALID_MODES, RUNTIME_PREFIXES, fail(), parseMode(), compareNames() (+8 more)

### Community 21 - "MeanReversionStrategy"
Cohesion: 0.06
Nodes (57): useAppConfig(), useScopedAccountQuery(), useBalance(), useOverseasBalance(), useBrokerHoldings(), useCheckConfig(), useTradesByRange(), useTodayStats() (+49 more)

### Community 22 - "database_projection.rs"
Cohesion: 0.08
Nodes (58): NaiveDate, CandleCursor, String, CandlePageRequest, Option, Result, BrokerCandlePage, Vec (+50 more)

### Community 23 - "toss_market.rs"
Cohesion: 0.11
Nodes (30): RateLimitScheduler, Arc, Mutex, HashMap, String, RateLimitGroupState, Duration, Instant (+22 more)

### Community 24 - "ServerState"
Cohesion: 0.05
Nodes (40): $schema, productName, version, identifier, build, beforeDevCommand, devUrl, beforeBuildCommand (+32 more)

### Community 25 - "RateLimitScheduler"
Cohesion: 0.16
Nodes (24): TossSmallBuyVerificationInput, String, Option, TossSmallBuyVerificationView, BrokerId, BrokerMarket, BrokerMoneyView, small_buy_cap() (+16 more)

### Community 26 - "strategy/ui/Page.tsx"
Cohesion: 0.07
Nodes (26): StrategySignal, build_strategy(), Box, StrategyManager, Vec, BTreeMap, String, Option (+18 more)

### Community 27 - "commands/toss.rs"
Cohesion: 0.13
Nodes (15): Probe, Arc, AtomicUsize, AtomicU64, Fixture, Mutex, StrategyManager, OrderManager (+7 more)

### Community 28 - "tauri.conf.json"
Cohesion: 0.16
Nodes (13): TokenDomain, TokenProbe, DetectedTradingType, DetectTradingTypeError, DetectTokenReq, detect_trading_type(), probe_token_domain(), classify_token_response() (+5 more)

### Community 29 - "ui/StockChart.tsx"
Cohesion: 0.20
Nodes (31): now(), DateTime, Utc, scope(), manager(), Vec, String, StrategyManager (+23 more)

### Community 30 - "StrategyManager"
Cohesion: 0.05
Nodes (56): useStockSearch(), useRefreshStockList(), useTossMarketCalendar(), useStrategies(), useUpdateStrategy(), Market, EXCHANGE_SEARCH_ORDER, TOSS_US_SESSION_OPTIONS (+48 more)

### Community 31 - "BrokerScope"
Cohesion: 0.12
Nodes (40): ProfileView, String, BrokerId, profile_to_view(), Option, list_profiles(), State, CmdResult (+32 more)

### Community 32 - "server/market.rs"
Cohesion: 0.22
Nodes (26): RowData, String, Option, rebuild(), Result, rebuild_postgres(), Transaction, Postgres (+18 more)

### Community 33 - "settings/ui/Page.tsx"
Cohesion: 0.13
Nodes (37): RefreshConfig, Default, String, Option, AppConfigView, BrokerId, Vec, get_app_config() (+29 more)

### Community 34 - "KisRestClient"
Cohesion: 0.22
Nodes (17): key(), fill(), OrderSide, rejected_without_mutation(), cumulative_buys_and_sells_track_native_weighted_cost_and_proportional_profit(), duplicate_and_regressing_cumulative_reports_are_noops_but_contradictions_fail(), exact_account_profile_mode_owner_market_and_symbol_are_separate_and_orders_cannot_rebind(), unknown_quantities_and_costs_and_invalid_identities_never_assume_zero_basis() (+9 more)

### Community 35 - "AccountProfile"
Cohesion: 0.18
Nodes (16): OwnedStrategySignal, Option, Value, From, Self, String, OrderType, ProviderOrderResponse (+8 more)

### Community 36 - "src/api/types.ts"
Cohesion: 0.17
Nodes (29): BrokerCandle, LeveragedTrendHoldPreviewInput, String, Value, Option, SimulationAssumptions, LeveragedTrendHoldPreviewSignalView, LeveragedTrendHoldPreviewView (+21 more)

### Community 37 - "rest/types.rs"
Cohesion: 0.13
Nodes (10): DateRangeQuery, stats_by_range_handler(), trades_by_range_handler(), kis_executed_handler(), SetLogConfigBody, set_log_config_handler(), SetArchiveConfigBody, set_archive_config_handler() (+2 more)

### Community 38 - "databaseManagementSection.tsx"
Cohesion: 0.12
Nodes (28): buy(), submit(), Arc, Mutex, OrderManager, Result, automatic_missing_context_and_stale_reservation_leave_budget_and_pending_unchanged(), shared_pipeline_discards_prepared_signals_on_stop_restart_account_and_config_changes() (+20 more)

### Community 39 - "DatabaseManager"
Cohesion: 0.33
Nodes (15): config(), reservation(), Fixture, Arc, fixture(), pending(), PendingOrder, configuration_snapshots_include_real_held_and_flat_markets_and_revalidate_fresh_quantity() (+7 more)

### Community 40 - "AppState"
Cohesion: 0.15
Nodes (12): KrxItem, KrxResponse, StockList, try_load_cache(), try_load_cache_any(), save_cache(), NaverAcItem, NaverAcResponse (+4 more)

### Community 41 - "server/records.rs"
Cohesion: 0.10
Nodes (41): DatabaseManager, PathBuf, Result, Self, DatabaseConfigView, SaveDatabaseConfigInput, FnOnce, Path (+33 more)

### Community 42 - "StrategyConfig"
Cohesion: 0.15
Nodes (14): json_path_lock(), build_daily_path(), build_monthly_path(), read_json_or_default(), write_json(), daily_path_uses_date_components_without_string_separators(), monthly_path_zero_pads_month_for_stats(), corrupt_json_recovers_from_last_durable_backup() (+6 more)

### Community 43 - "domain.rs"
Cohesion: 0.08
Nodes (32): useTradeArchiveConfig(), useSetTradeArchiveConfig(), useTradeArchiveStats(), useLogConfig(), useSetLogConfig(), useStockListStats(), useSetStockUpdateInterval(), useSendTestDiscord() (+24 more)

### Community 44 - "archive.rs"
Cohesion: 0.07
Nodes (48): usePrice(), useOverseasPrice(), useTossMarketSnapshot(), usePlaceOrder(), useTossOpenOrders(), useModifyTossOrder(), usePlaceOverseasOrder(), tossMarketLabel() (+40 more)

### Community 45 - "commands/records.rs"
Cohesion: 0.27
Nodes (12): TradeArchiveConfig, TradeArchiveStats, collect_trade_archive_stats(), collect_trade_archive_stats_active(), purge_trade_archive_active(), collect_trade_day_dirs(), dir_size_bytes(), purge_old_trade_files() (+4 more)

### Community 46 - "PriceConditionStrategy"
Cohesion: 0.19
Nodes (20): TickCycleResult, PriceSource, toss_us_session_policy_from_params(), Value, strategy_market_open_for_symbol(), Option, fetch_toss_tick(), Result (+12 more)

### Community 47 - "detect.rs"
Cohesion: 0.15
Nodes (3): DiscordNotifier, NotificationLevel, NotificationEvent

### Community 48 - "submit_toss_small_buy_verification_for_profile"
Cohesion: 0.19
Nodes (21): kis_pending_from(), BrokerId, String, Vec, toss_pending_from(), collect_kis_pending_fills(), Arc, KisRestClient (+13 more)

### Community 49 - "market/mod.rs"
Cohesion: 0.17
Nodes (12): profile(), scope(), client(), String, Arc, KisRestClient, Fixture, Mutex (+4 more)

### Community 50 - "profiles.rs"
Cohesion: 0.05
Nodes (30): PositionView, String, From, Self, get_positions(), State, CmdResult, Vec (+22 more)

### Community 51 - "server/trading.rs"
Cohesion: 0.11
Nodes (21): PositionState, signal_parts(), units_to_krw(), units_to_display(), krw_to_units(), krw_to_display(), apply_slippage(), bps_cost() (+13 more)

### Community 52 - "OrderRecord"
Cohesion: 0.12
Nodes (31): lth_default_ema_short(), lth_default_ema_long(), lth_default_rsi_period(), lth_default_adx_period(), lth_default_buy_rsi(), lth_default_sell_rsi(), lth_default_buy_adx(), lth_default_no_trade_adx() (+23 more)

### Community 53 - "StockStore"
Cohesion: 0.11
Nodes (26): AppState, File, Arc, RwLock, KisRestClient, PathBuf, Mutex, StrategyManager (+18 more)

### Community 54 - "security.rs"
Cohesion: 0.20
Nodes (26): now(), DateTime, Utc, scope(), manager(), Arc, Mutex, StrategyManager (+18 more)

### Community 55 - "dependencies"
Cohesion: 0.19
Nodes (3): StockEntry, StockListStats, StockStore

### Community 56 - "logging/mod.rs"
Cohesion: 0.18
Nodes (20): ExecutionModel, IntrabarExitPolicy, SimulationAssumptions, Default, Self, default_initial_capital(), default_fee_bps(), default_tax_bps() (+12 more)

### Community 57 - "active_profile"
Cohesion: 0.13
Nodes (12): BrokerSymbol, select_price_response(), Vec, Option, TossBrokerAdapter, BrokerAdapterResult, BrokerCurrency, BrokerId (+4 more)

### Community 58 - "compilerOptions"
Cohesion: 0.22
Nodes (17): BrokerScope, BudgetLedger, Vec, BudgetAccount, AutoTradingBudgetView, BudgetCurrencyView, cost(), Result (+9 more)

### Community 59 - "commands/database.rs"
Cohesion: 0.11
Nodes (23): PendingOrder, String, Option, Value, TradingHealthStatus, OrderManager, HashMap, OrderSide (+15 more)

### Community 60 - "commands/market.rs"
Cohesion: 0.15
Nodes (11): toss_currency(), toss_market(), market_from_currency(), validate_client_order_id(), validate_order_side(), validate_order_type(), validate_time_in_force(), validate_optional_decimal() (+3 more)

### Community 61 - "DiscordNotifier"
Cohesion: 0.10
Nodes (29): useDatabaseConfig(), useJsonStorageInventory(), useStatusMutation(), useSaveDatabaseConfig(), useTestDatabaseConnection(), useCreateDatabaseTables(), useClearDatabaseTables(), useDropDatabaseTables() (+21 more)

### Community 62 - "DatabaseConfig"
Cohesion: 0.19
Nodes (19): DatabasePool, PgPool, MySqlPool, DatabaseManager, Result, F, connect_pool(), is_missing_database_error() (+11 more)

### Community 63 - "DatabasePool"
Cohesion: 0.13
Nodes (18): KisOrderRejected, String, read_kis_response_text(), Response, Result, KisRestClient, Arc, RwLock (+10 more)

### Community 64 - "TradeRecord"
Cohesion: 0.22
Nodes (6): clear_database_tables(), drop_database_tables(), import_json_to_database(), export_database_to_json(), set_storage_backend(), ensure_trading_stopped()

### Community 65 - "release-version.mjs"
Cohesion: 0.16
Nodes (21): trading_status_handler(), State, Json, Value, sync_kis_strategy_positions(), Result, String, sync_toss_strategy_positions() (+13 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.11
Nodes (26): kis_read_min_interval(), Duration, kis_http_client(), Client, synthetic_auth(), HeaderMap, assert_auth(), Request (+18 more)

### Community 67 - "scripts"
Cohesion: 0.13
Nodes (45): ServerState, Arc, RwLock, KisRestClient, Vec, StockSearchItem, PathBuf, Mutex (+37 more)

### Community 68 - "place_toss_order"
Cohesion: 0.06
Nodes (60): ConfigurationEditReservation, String, HashSet, ConfigurationUpdateError, Into, Self, effective_target_markets(), HashMap (+52 more)

### Community 69 - "conflicts.rs"
Cohesion: 0.14
Nodes (42): TossAccount, TossConnectionStep, String, TossAccountOptionView, TossAccountLookupInput, TossConnectionDiagnostic, BrokerId, Option (+34 more)

### Community 71 - "commands/risk.rs"
Cohesion: 0.13
Nodes (26): Provider, Arc, Notify, AtomicUsize, AtomicBool, real_fill_invalidates_an_earlier_whole_holdings_response(), executed(), State (+18 more)

### Community 72 - ".new"
Cohesion: 0.25
Nodes (25): profiles_handler(), State, Json, Value, apply_profile_change(), save_profiles_server(), AddProfileBody, BrokerId (+17 more)

### Community 73 - "storage/database.rs"
Cohesion: 0.12
Nodes (20): OVERSEAS_EXCHANGES, CandleCollectionEnd, CandleCollection, isProviderError(), checkCancelled(), candleTimestamp(), assertPageScope(), collectPreviewCandles() (+12 more)

### Community 74 - "KisWebSocketClient"
Cohesion: 0.08
Nodes (20): GetTradesByRangeInput, get_trades_by_range(), GetStatsByRangeInput, get_stats_by_range(), SaveTradeInput, save_trade(), FrontendLogInput, write_frontend_log() (+12 more)

### Community 75 - "toss/orders.rs"
Cohesion: 0.07
Nodes (52): pc_config(), lth_config(), manager(), Fixture, Vec, Arc, Mutex, StrategyManager (+44 more)

### Community 76 - "read_json_or_default"
Cohesion: 0.39
Nodes (4): WsStatusEvent, RealtimePrice, KisWebSocketClient, parse_realtime_price()

### Community 77 - "StatsStore"
Cohesion: 0.12
Nodes (30): PgTestEnv, String, pg_test_env(), Option, unique_test_database(), temp_dirs(), PathBuf, save_input() (+22 more)

### Community 78 - "devDependencies"
Cohesion: 0.19
Nodes (14): OhlcCandle, lth_default_qty(), LeveragedReboundSnapshot, LeveragedRapidReboundSnapshot, LeveragedTrendHoldTimedCandle, LeveragedTrendHoldPreviewStep, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldStrategy (+6 more)

### Community 79 - "project-map.mjs"
Cohesion: 0.20
Nodes (13): ChartRequest, ChartRequest<'a>, Self, Client, HeaderMap, RequestBuilder, KisRestClient, Result (+5 more)

### Community 80 - "TokenManager"
Cohesion: 0.16
Nodes (19): parse_amount_f64(), TradingStatus, Vec, String, BrokerId, TradingHealthStatus, get_trading_status(), State (+11 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.09
Nodes (12): PriceConditionSymbolConfig, String, PriceConditionParams, Vec, PriceConditionStrategy, HashMap, Option, Value (+4 more)

### Community 82 - ".write_document"
Cohesion: 0.12
Nodes (12): MaCrossParams, Default, Self, MaCrossState, VecDeque, Option, MovingAverageCrossStrategy, HashMap (+4 more)

### Community 83 - "KisBrokerAdapter"
Cohesion: 0.08
Nodes (45): holdingsPrefixes, profileMutationKey, profileMutationScope, trustedAccountConfig(), accountScope(), resetAccountHoldings(), AccountConfigRefreshError, refreshAccountConfig() (+37 more)

### Community 84 - "support.rs"
Cohesion: 0.10
Nodes (24): ReplayExecutionModel, ReplayIntrabarExitPolicy, SimulationAssumptions, ReplayAssessment, BacktestTrade, EquityPoint, BacktestPhase, BacktestSummary (+16 more)

### Community 85 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.14
Nodes (22): ExperimentScope, normalizePart(), storageKey(), loadExperimentSlots(), compactSnapshot(), readIndex(), saveExperimentSlot(), clearExperimentSlots() (+14 more)

### Community 86 - "broker/adapter.rs"
Cohesion: 0.18
Nodes (15): PendingFill, String, OrderManager, Result, Arc, Mutex, Vec, PendingCandidate (+7 more)

### Community 87 - "ui/LayoutResizer.tsx"
Cohesion: 0.16
Nodes (19): PlaceOrderInput, String, Option, place_order(), State, CmdResult, OrderResponse, manual_submission_response() (+11 more)

### Community 88 - "initialize_active_strategy_history"
Cohesion: 0.14
Nodes (29): live_feedback_lth_rejected_and_partial_exits_preserve_current_tick_protection(), live_feedback_lth_replay_still_clears_optimistic_exit(), live_feedback_lth_checkpoint_keeps_peak_age_and_authoritative_owned_average(), reconfigure_lth_keeps_exact_protection_lifecycle_and_uses_new_exit_parameters(), reconfigure_lth_keeps_initial_stop_without_new_window_warmup(), reconfigure_lth_trims_raw_observations_to_new_caps_without_reconstructing_position(), upward_candles(), Vec (+21 more)

### Community 89 - "PendingOrderStore"
Cohesion: 0.17
Nodes (7): Option, NaiveDate, OrderManager, Result, cumulative_fill_delta(), calculate_domestic_fee(), calculate_overseas_fee_cents()

### Community 90 - "StrategyStore"
Cohesion: 0.11
Nodes (16): BollingerControls(), Market, PreviewInterval, PreviewCount, TargetDraftSelection, LeveragedTrendHoldEditorPanelProps, EXCHANGE_SEARCH_ORDER, normalizeUsTicker() (+8 more)

### Community 91 - "TradeStore"
Cohesion: 0.15
Nodes (41): candle_page_handler(), State, Query, CandlePageInput, Response, balance_handler(), Json, Value (+33 more)

### Community 92 - "database_keychain.rs"
Cohesion: 0.32
Nodes (11): LocalDocument, String, Option, Path, read_managed_json(), write_managed_json(), read_managed_json_backup(), quarantine_corrupt_managed_json() (+3 more)

### Community 93 - "@mui/icons-material"
Cohesion: 0.47
Nodes (4): GitHubRelease, UpdateInfo, check(), is_newer()

### Community 94 - "balance_store.rs"
Cohesion: 0.19
Nodes (19): ownership_fill_same_provider_id_across_paper_live_modes_has_distinct_receipts(), ownership_fill_retry_keeps_saved_execution_date_across_midnight(), ownership_fill_profile_switch_same_account_records_only_original_owner_identity(), ownership_fill_restart_loads_partial_cost_and_risk_receipts_from_primary_documents(), ownership_fill_overseas_empty_snapshot_uses_cents_cost_before_fx_conversion(), fixture(), OrderSide, fixture_with_base() (+11 more)

### Community 95 - ".load_sync"
Cohesion: 0.15
Nodes (14): String, LeveragedTrendHoldEntry, Vec, HashMap, LeveragedTrendHoldParams, Default, Self, LeveragedTrendHoldMarketState (+6 more)

### Community 96 - "check"
Cohesion: 0.08
Nodes (23): compilerOptions, target, useDefineForClassFields, lib, ES2020, DOM, DOM.Iterable, module (+15 more)

### Community 97 - "compilerOptions"
Cohesion: 0.60
Nodes (3): create(), postgres_ddl(), mariadb_ddl()

### Community 98 - "check-fsd-imports.mjs"
Cohesion: 0.33
Nodes (6): copy_dir_all(), Path, Result, run(), Arc, install_database_manager()

### Community 99 - "package.json"
Cohesion: 0.11
Nodes (10): Option, resolve_scoped_profile(), Arc, RwLock, AccountProfile, ProfilesConfig, super::OwnershipIdentity, Option (+2 more)

### Community 100 - "verify-toss-openapi.mjs"
Cohesion: 0.11
Nodes (17): devDependencies, @playwright/test, @playwright/test, @tauri-apps/cli, @tauri-apps/cli, @types/react, @types/react, @types/react-dom (+9 more)

### Community 101 - "OverseasExecutedOrder"
Cohesion: 0.12
Nodes (22): useChartData(), useTossChartData(), PeriodCode, TossInterval, ChartSource, ChartPreset, CHART_PRESETS, TOSS_CHART_PRESETS (+14 more)

### Community 102 - "BrokerAccountId"
Cohesion: 0.15
Nodes (17): rootDir, options, positional, args, versionFiles, changedFiles, branchName, syncVersionFiles() (+9 more)

### Community 103 - "TossAccessToken"
Cohesion: 0.20
Nodes (7): OrderManager, Result, Option, String, Vec, PendingOrder, OrderSide

### Community 104 - "atomic_write"
Cohesion: 0.09
Nodes (22): dependencies, @emotion/react, @emotion/react, @emotion/styled, @emotion/styled, @mui/icons-material, @mui/icons-material, @mui/material (+14 more)

### Community 106 - "TossApiError"
Cohesion: 0.18
Nodes (12): AccessToken, String, DateTime, Utc, TokenIssueResponse, TokenManager, Client, Arc (+4 more)

### Community 107 - "strategyResearchTypes.ts"
Cohesion: 0.25
Nodes (6): lth_preview_signals_for_interval(), F, LeveragedTrendHoldPreviewSignal, daily_diagnostic_never_evaluates_strategy_or_execution_even_with_intraday_input(), Self, empty_minute_observations_are_not_evaluable()

### Community 116 - "build.rs"
Cohesion: 0.12
Nodes (26): GuardSide, GuardDecision, String, TradeGuardConfig, Default, Self, TradeGuard, HashMap (+18 more)

### Community 118 - "settings-database.spec.ts"
Cohesion: 0.50
Nodes (3): TossHttpError, TossErrorResponse, TossApiError

### Community 119 - "kis-auto-trade"
Cohesion: 0.30
Nodes (17): preview_strategy_from_candles(), rows(), Vec, ChartCandle, input(), StrategyPreviewInput, all_strategies_keep_the_same_evaluation_window_without_implicit_warmup(), separate_history_matches_explicit_legacy_prefix_and_hashes_history() (+9 more)

### Community 120 - "playwright.config.ts"
Cohesion: 0.13
Nodes (15): lightweight-charts, lightweight-charts, useOverseasChartData(), OverseaChartPreset, OVERSEAS_CHART_PRESETS, toISODate(), CandlePoint, VolumePoint (+7 more)

### Community 121 - "sync-codex-skills.ps1"
Cohesion: 0.22
Nodes (22): atomic_write(), Path, Result, atomic_write_private_sync(), durable_remove_file(), remove_file_using_sync(), FnOnce, sync_parent() (+14 more)

### Community 122 - "discord-notification-config/index.ts"
Cohesion: 0.21
Nodes (10): BrokerAdapterError, BrokerId, String, Error, BrokerAdapter, Send, Sync, DummyAdapter (+2 more)

### Community 123 - "log-filter/index.ts"
Cohesion: 0.08
Nodes (6): Signal, LeveragedTrendHoldStrategy, Option, Value, Result, String

### Community 124 - "strategy-configure/index.ts"
Cohesion: 0.21
Nodes (13): WebSecurity, parse_bool_env(), constant_time_eq(), bearer_token(), has_same_origin(), rejection(), require_web_security(), Next (+5 more)

### Community 125 - "strategy-toggle/index.ts"
Cohesion: 0.22
Nodes (17): chart_volume_to_u64(), StrategyPreviewInput, String, Value, Vec, ChartCandle, Option, BrokerId (+9 more)

### Community 126 - "Result"
Cohesion: 0.37
Nodes (8): Result, Value, Ok, validate_metadata(), cumulative(), signed_difference(), proportional_cost(), validate_application()

### Community 127 - "PendingOrderStore"
Cohesion: 0.26
Nodes (10): PendingOrderStore, PathBuf, Mutex, Self, Result, Vec, PendingOrder, snapshot() (+2 more)

### Community 128 - "vite.config.ts"
Cohesion: 0.28
Nodes (16): same_order_side(), OrderSide, PendingOrder, order_side_label(), pending_order_conflict_reason(), String, pending_conflict_reason_for_scope(), Iterator (+8 more)

### Community 129 - "input"
Cohesion: 0.23
Nodes (10): SimulationEvent, AtrPreparationView, Option, String, AtrWindow, VecDeque, bar(), full_previous_close_history_is_required_and_old_ranges_expire() (+2 more)

### Community 130 - ".new"
Cohesion: 0.15
Nodes (17): candle_market_date(), DateTime, Utc, BrokerMarket, DailyHistoryTicket, String, BrokerId, Option (+9 more)

### Community 131 - ".fmt"
Cohesion: 0.31
Nodes (7): pending_order_provider(), BrokerId, validate_pending_kis_mode(), PendingOrder, Result, OrderManager, Option

### Community 132 - ".new"
Cohesion: 0.17
Nodes (10): TossTokenState, AsyncMutex, toss_token_state_key(), shared_toss_token_state(), toss_http_client(), body_snippet(), trim_base_url(), url_encode() (+2 more)

### Community 133 - ".search"
Cohesion: 0.19
Nodes (16): active_profile(), toss_market_snapshot_handler(), toss_stock_safety_handler(), toss_order_preflight_handler(), TossOrderPreflightInput, toss_open_orders_handler(), TossOpenOrdersInput, toss_modify_order_handler() (+8 more)

### Community 134 - "run_backtest"
Cohesion: 0.24
Nodes (16): run_backtest(), run_backtest_with_history(), atr_history(), Vec, atr_sizing_assumptions(), SimulationAssumptions, flat_gap_entry(), atr_risk_uses_previous_history_close_and_evaluation_gap_without_trading_history() (+8 more)

### Community 135 - "AppConfigView"
Cohesion: 0.20
Nodes (4): OrderManager, TradingHealthStatus, Option, String

### Community 136 - "작업 기록: <짧은 제목>"
Cohesion: 0.23
Nodes (17): broker_money_to_strategy_units(), Option, broker_candles_to_ohlc(), Vec, apply_ohlc_history(), Arc, Mutex, StrategyManager (+9 more)

### Community 137 - "write_json"
Cohesion: 0.47
Nodes (3): HoldingItem, BalanceSnapshot, BalanceStore

### Community 138 - ".process_buy"
Cohesion: 0.42
Nodes (7): order_exchange_code(), OrderManager, String, Option, Result, OrderSide, OrderResponse

### Community 139 - "CurrencyLedger"
Cohesion: 0.23
Nodes (7): LeveragedTrendSnapshot, LeveragedTrendHoldStrategy, VecDeque, Vec, Option, PreviewPosition, Option

### Community 141 - "modify_toss_order_for_profile"
Cohesion: 0.29
Nodes (9): strategies(), Fixture, Arc, Mutex, StrategyManager, failed_configuration_pause_never_becomes_market_close_and_same_revision_resumes(), unsubmitted_feedback_never_assigns_an_unowned_account_holding_to_strategy(), market_closed_first_order_settles_remaining_raw_batch_without_touching_submitted_state() (+1 more)

### Community 142 - "OrderSubmission"
Cohesion: 0.27
Nodes (14): OrderSubmission, PrepareDecision, Box, prepare_order_submission(), prepare_buy_submission(), prepare_sell_submission(), build_provider_request(), Option (+6 more)

### Community 143 - "HARNESS_MAP.md"
Cohesion: 0.18
Nodes (14): BollingerParams, Default, Self, Band, Option, LeveragedTrendHoldStrategy, Vec, String (+6 more)

### Community 145 - "react-grid-layout"
Cohesion: 0.18
Nodes (12): SCRIPT_DIR, REPOSITORY_ROOT, SNAPSHOT_PATH, VALID_MODES, SOURCE_PREFIXES, SOURCE_EXTENSIONS, ROOT_CONFIGS, fail() (+4 more)

### Community 147 - "Result"
Cohesion: 0.21
Nodes (15): candle(), ChartCandle, input(), StrategyPreviewInput, minute_provider_offsets_produce_identical_actual_replay_and_no_gap_fill(), minute_replay_sorts_by_instant_rather_than_offset_text(), minute_replay_rejects_same_instant_duplicates_and_history_overlap(), legacy_compact_minutes_are_explicit_kst_and_invalid_times_fail_closed() (+7 more)

### Community 148 - "Arc"
Cohesion: 0.25
Nodes (13): statuses, root, sourcePath, outputPath, escapeHtml(), safeLink(), inlineMarkdown(), parseTasks() (+5 more)

### Community 149 - "AtomicBool"
Cohesion: 0.35
Nodes (20): buy(), Option, sell(), bar(), assumptions(), SimulationAssumptions, replay(), delayed_fill_uses_next_observed_open_and_final_signal_remains_unfilled() (+12 more)

### Community 150 - "ChartCandle"
Cohesion: 0.19
Nodes (11): accept(), State, Requests, Json, Value, publish(), Fixture, submit() (+3 more)

### Community 151 - "Client"
Cohesion: 0.16
Nodes (12): restore_risk_from_today_trades(), Arc, Mutex, RiskManager, CmdResult, trade(), TradeRecord, risk_rebuild_seeds_trade_event_receipts_and_retries_missing_pending_intent_once() (+4 more)

### Community 152 - "Duration"
Cohesion: 0.31
Nodes (9): OrderManager, Result, Option, NaiveDate, BrokerCurrency, parts(), scaled_total(), execution_quantity() (+1 more)

### Community 153 - "ExecutedOrder"
Cohesion: 0.18
Nodes (11): usd_to_cents(), ChartDataInput, get_chart_data(), get_price(), search_stock(), get_stock_list_stats(), StockListStats, OverseasPriceView (+3 more)

### Community 154 - "HeaderMap"
Cohesion: 0.27
Nodes (10): scope(), apply(), RiskManager, Result, roundtrip(), serialized_retries_and_intent_before_risk_are_exactly_once(), rebuild_replays_durable_trades_then_missing_intent(), scope_is_part_of_event_identity_and_payload_conflicts_fail_closed() (+2 more)

### Community 155 - "OrderResponse"
Cohesion: 0.22
Nodes (17): BrokerQuantity, check_toss_order_preflight_for_profile(), OrderPreflightInput, OrderPreflightConstraints, Option, String, OrderPreflightDecision, Vec (+9 more)

### Community 156 - "OverseasExecutedOrder"
Cohesion: 0.29
Nodes (11): candle(), ChartCandle, input(), Value, Vec, StrategyPreviewInput, strong_close_uses_previous_completed_day_and_keeps_position_across_day_changes(), blocked_strong_close_buy_does_not_discard_next_days_completed_condition() (+3 more)

### Community 157 - "PriceResponse"
Cohesion: 0.12
Nodes (20): JobEntry, Vec, Instant, Option, JoinHandle, Result, String, DailyHistoryJobs (+12 more)

### Community 158 - "Response"
Cohesion: 0.17
Nodes (20): String, Option, ReplayMetadataView, ReplayEventTime, BacktestTradeView, EquityPointView, BacktestPhaseView, BacktestSummaryView (+12 more)

### Community 159 - "place_toss_order"
Cohesion: 0.09
Nodes (16): String, BrokerId, StrategyConfig, Option, Vec, Value, default_strategy_broker_id(), Into (+8 more)

### Community 160 - "collect_pages"
Cohesion: 0.13
Nodes (29): CollectedDailyHistory, Vec, String, Option, price(), completed_bar(), ChartCandle, collect_daily_history() (+21 more)

### Community 161 - ".checkpoint_strategy_positions_shared"
Cohesion: 0.47
Nodes (6): OrderManager, Arc, Mutex, Self, StrategyManager, Result

### Community 162 - ".capture"
Cohesion: 0.15
Nodes (10): Option, OwnershipLedger, Default, Self, OwnershipManifest, String, Result, OrderManager (+2 more)

### Community 163 - "ExchangeRateView"
Cohesion: 0.25
Nodes (8): Self, ExchangeRateSource, ExchangeRateView, Result, refresh_exchange_rate_status(), Arc, RwLock, resolve_exchange_rate_policy()

### Community 164 - "collect_daily_history"
Cohesion: 0.19
Nodes (17): Server, String, Arc, Notify, JoinHandle, Value, Self, Drop (+9 more)

### Community 165 - "toss/orders.rs"
Cohesion: 0.24
Nodes (7): fill(), Fixture, missing_primary_starts_empty_and_real_file_restart_keeps_exact_idempotent_fill(), primary_corruption_blocks_instead_of_recovering_an_old_valid_backup(), every_read_observes_current_primary_and_structural_errors_are_not_empty_ledgers(), primary_loss_is_detected_after_restart_even_with_no_pending_orders_and_a_valid_backup(), primary_only_creation_crash_gap_anchors_the_existing_generation_and_ownership()

### Community 175 - "Mutex"
Cohesion: 0.38
Nodes (12): config(), Value, candle(), volatility(), strong_close_uses_each_completed_day_only_on_the_following_day(), strong_close_completed_day_uses_actual_position_and_keeps_stop_loss(), volatility_compares_current_full_range_against_prior_completed_window(), volatility_day_range_and_direction_are_independent_across_days() (+4 more)

### Community 181 - "AppHandle"
Cohesion: 0.18
Nodes (9): args, artifactDir, checksumPath, manifestPath, requireSignatures, names, artifacts, signatures (+1 more)

### Community 182 - "MutexGuard"
Cohesion: 0.22
Nodes (10): OwnershipIdentity, String, Debug, Formatter, OwnershipOwner, OwnershipKey, BrokerMarket, OwnershipFill (+2 more)

### Community 183 - ".submit_order_shared"
Cohesion: 0.21
Nodes (12): root, runId, directory, containers, results, environment, interruption, terminate() (+4 more)

### Community 184 - ".new"
Cohesion: 0.26
Nodes (9): ExecutionRunContext, Option, String, ExecutionRun, Arc, KisRestClient, OrderManager, Result (+1 more)

### Community 190 - "Vec"
Cohesion: 0.24
Nodes (10): AutoTradingBudgetInput, UpdateAutoTradingBudgetInput, checked_budget_scope(), get_auto_trading_budget(), update_auto_trading_budget(), budget_handler(), AutoTradingBudgetInput, update_budget_handler() (+2 more)

### Community 192 - "modify_toss_order_for_profile"
Cohesion: 0.28
Nodes (11): StrategyPreviewChartSignal, Props, toNumber(), parseTimeMs(), toChartTime(), chartTimeToMs(), pad2(), formatKstMs() (+3 more)

### Community 193 - "order/configuration/tests.rs"
Cohesion: 0.18
Nodes (6): toss_market_session_window(), MarketSessionWindow, MarketDayCalendar, calendar_session_window_checks_time_range(), calendar_override_represents_holiday_without_fallback(), us_session_policy_checks_extended_sessions()

### Community 194 - "commands/risk.rs"
Cohesion: 0.36
Nodes (12): RiskConfigView, build_risk_view(), persist_risk_runtime(), get_risk_config(), UpdateRiskConfigInput, update_risk_config(), clear_emergency_stop(), activate_emergency_stop() (+4 more)

### Community 195 - "PreparedOrderSubmission"
Cohesion: 0.29
Nodes (11): PreparedOrderSubmission, build_pending_order(), PendingOrder, build_kis_pending_order(), OrderResponse, build_toss_pending_order(), String, Option (+3 more)

### Community 196 - "MarketDayCalendar"
Cohesion: 0.50
Nodes (5): quantity(), Result, money(), HoldingsUpdate, Self

### Community 197 - "SubmissionDeps"
Cohesion: 0.22
Nodes (9): SubmissionDeps, Arc, KisRestClient, RiskManager, RwLock, prepare_ownership(), OrderManager, Option (+1 more)

### Community 198 - "first_positive_u64"
Cohesion: 0.21
Nodes (9): daily_diagnostic_contains_one_completed_bar_and_no_synthetic_entry_observation(), future_candle_change_does_not_change_prior_replay_signals(), minute_replay_warmup_excludes_replay_day_and_future_daily_candles(), blocked_buy_is_fed_back_before_the_next_raw_signal(), reproduction_hash_covers_ohlcv_and_order_quantity(), live_tick_and_preview_emit_identical_signals_for_normalized_fixture(), daily_warmup_preview_uses_closes_and_excludes_future_bars(), candle() (+1 more)

### Community 199 - "Self"
Cohesion: 0.40
Nodes (3): LeveragedTrendHoldStrategy, Option, String

### Community 200 - "config"
Cohesion: 0.16
Nodes (19): config(), Value, reject_identity_changes(), reconfigure_rsi_recomputes_baseline_with_new_period_and_keeps_targets_separate(), reconfigure_momentum_keeps_actual_entry_and_repeat_sell_state(), reconfigure_deviation_applies_new_threshold_to_retained_observations(), Strategy, Send (+11 more)

### Community 201 - "holding"
Cohesion: 0.38
Nodes (7): scope(), holding(), BrokerMarket, toss_fractional_and_zero_quantities_never_create_a_whole_share(), toss_us_prices_round_to_cents_and_kr_prices_round_to_won(), toss_rejects_invalid_quantities_and_prices_including_float_u64_boundary(), toss_rejects_wrong_broker_account_currency_and_duplicate_market_symbol()

### Community 202 - "Arc"
Cohesion: 0.22
Nodes (5): root, srcRoot, layers, layerRank, violations

### Community 203 - "BalanceItem"
Cohesion: 0.22
Nodes (6): rootDir, pkg, npmLock, escapedName, workspaceLock, appEntry

### Community 204 - "StrategyStore"
Cohesion: 0.42
Nodes (6): CurrencyLedger, BTreeMap, String, Option, BudgetReservation, BudgetCurrencyView

### Community 205 - "config"
Cohesion: 0.53
Nodes (5): config(), Value, reconfigure_price_condition_canonicalizes_targets_and_keeps_actual_stop(), reconfigure_price_condition_does_not_move_holdings_to_replacement_target(), empty_price_condition_entries_remove_stale_targets_before_manager_validation()

### Community 206 - "strategy/tests.rs"
Cohesion: 0.10
Nodes (10): TossOrderListStatus, TossOrderListQuery, TossOrderModifyRequest, TossOrderResponse, TossPaginatedOrderResponse, TossOrder, TossOrderExecution, validate_iso_date() (+2 more)

### Community 207 - ".prepare_confirmed_ownership"
Cohesion: 0.33
Nodes (6): PreparedOwnershipFill, Option, OrderManager, PendingOrder, Result, realized_pnl()

### Community 208 - "TossTokenState"
Cohesion: 0.40
Nodes (7): OrderManager, Arc, Mutex, Self, Result, Option, P

### Community 209 - "broker/mod.rs"
Cohesion: 0.44
Nodes (10): params(), history(), Vec, timed(), run(), Value, timed_replay_has_nonempty_repeatable_signals_and_execution_results(), session_and_blackout_use_input_minutes() (+2 more)

### Community 210 - "Arc"
Cohesion: 0.28
Nodes (14): now_kst(), UsTradingSessionPolicy, UsMarketSessionCalendar, MarketCalendarOverride, is_domestic_symbol(), is_krx_open(), is_krx_open_at(), is_us_open() (+6 more)

### Community 212 - ".new"
Cohesion: 0.39
Nodes (4): KisOutput1<T>, OverseasExecutedOrder, parse_decimal_cents(), first_positive_u64()

### Community 213 - "HashMap"
Cohesion: 0.25
Nodes (6): EXPECTED_PATHS, paths, servers, missingPaths, hasRateLimitHeaders, endpointInventory

### Community 214 - "HeaderMap"
Cohesion: 0.22
Nodes (8): compilerOptions, composite, skipLibCheck, module, moduleResolution, allowSyntheticDefaultImports, include, vite.config.ts

### Community 215 - "TossAccessToken"
Cohesion: 0.32
Nodes (7): refresh_manual_positions(), Arc, Mutex, OrderManager, Result, current_position_snapshot(), Option

### Community 216 - "place_prepared_order"
Cohesion: 0.50
Nodes (7): place_prepared_order(), Result, place_with_retry_shared(), Arc, KisRestClient, OrderResponse, place_overseas_with_retry_shared()

### Community 218 - "Option"
Cohesion: 0.43
Nodes (5): DatabaseArchiveStats, stats(), purge(), trade_rows(), trade_date()

### Community 226 - "submission.rs"
Cohesion: 0.27
Nodes (10): entry(), Result, store_database_password(), load_database_password(), Option, String, delete_database_password(), fail_next_mock_operation_for_tests() (+2 more)

### Community 227 - "config"
Cohesion: 0.25
Nodes (7): KisBrokerAdapter, Arc, KisRestClient, Self, BalanceItem, BrokerId, maps_kis_balance_item_to_broker_holding()

### Community 245 - "StatusCode"
Cohesion: 0.15
Nodes (7): zustand, zustand, AccountState, useAccountStore, TradingStatus, TradingState, useTradingStore

### Community 262 - "config"
Cohesion: 0.48
Nodes (6): config(), Value, live_feedback_consecutive_keeps_confirmed_flag_and_observations(), live_feedback_breakout_freezes_pending_signal_anchor_until_first_fill(), live_feedback_unsubmitted_breakout_discards_only_candidate_not_history(), live_feedback_breakout_checkpoint_restores_pending_and_confirmed_exact_anchor()

### Community 265 - "notional/tests.rs"
Cohesion: 0.83
Nodes (3): detail(), confirmed_small_fill_requires_original_order_symbol_side_currency_and_whole_share(), missing_amount_or_zero_fill_is_deferred_without_using_limit_or_average_as_money()

### Community 267 - "config"
Cohesion: 0.53
Nodes (5): config(), Value, live_feedback_fifty_two_week_keeps_crossing_progress_without_optimistic_holdings(), live_feedback_strong_close_consumes_candidate_only_after_buy_fill(), live_feedback_volatility_keeps_intraday_extremes_and_confirmed_position()

### Community 274 - "commands.rs"
Cohesion: 0.22
Nodes (8): Option, String, Self, parse_amount_i64(), balance_summary_total_krw(), BalanceSummary, CmdError, Error

### Community 275 - ".new"
Cohesion: 0.33
Nodes (6): SubmissionReservation, Mutex, ReservationDecision, PendingOrder, OrderManager, submission_is_stale()

### Community 276 - "database/tests.rs"
Cohesion: 0.19
Nodes (9): JsonStorageInventoryView, inventory_from_documents(), inventory_groups_documents_by_top_level_category(), keychain_test_dirs(), PathBuf, save_input_with_password(), SaveDatabaseConfigInput, save_config_stores_password_in_keychain_not_in_file() (+1 more)

### Community 277 - "config"
Cohesion: 0.62
Nodes (6): config(), Value, reject_scope_changes(), reconfigure_fifty_two_week_keeps_actual_entry_while_invalidating_old_window(), reconfigure_strong_close_preserves_stop_but_discards_old_entry_candidate(), reconfigure_volatility_recomputes_range_without_losing_held_stop_or_day()

### Community 278 - "TossTokenState"
Cohesion: 0.31
Nodes (4): LeveragedTrendHoldStrategy, Option, String, parse_hhmm()

### Community 320 - "TossOrderbookView"
Cohesion: 0.25
Nodes (8): MarketReadiness, market_readiness(), StrategyManager, FnOnce, finish_closed_batch(), Vec, Option, String

### Community 322 - "submission.rs"
Cohesion: 0.43
Nodes (3): OrderSide, order_side_label(), submitting_conflict_reason()

### Community 323 - "config/mod.rs"
Cohesion: 0.43
Nodes (4): DiscordConfig, SecureConfig, load_secure_config(), non_empty()

### Community 324 - ".new"
Cohesion: 0.60
Nodes (3): Into, String, Self

### Community 327 - "preview_strategy"
Cohesion: 0.67
Nodes (3): preview_strategy(), StrategyPreviewInput, StrategyPreviewView

### Community 704 - "buffer_tests.rs"
Cohesion: 0.32
Nodes (13): params(), timed(), Vec, daily_context_cannot_prepare_minute_indicators_or_rebound(), daily_reinitialization_keeps_intraday_bollinger_and_actual_position(), repeated_intraday_seed_replaces_buffers_without_daily_mix_or_position_reset(), replay_daily_values_cannot_change_minute_signals_or_indicator_snapshot(), failed_buy_feedback_does_not_create_phantom_position() (+5 more)

### Community 705 - "high_tests.rs"
Cohesion: 0.36
Nodes (12): config(), Value, fifty_two_week_requires_full_window_and_uses_last_completed_high_and_close(), bar(), strategy(), complete_day(), oldest_high_expires_after_one_completed_day_and_changes_next_day_signal(), completed_high_not_close_sets_next_reference_without_lookahead() (+4 more)

### Community 707 - "package.json"
Cohesion: 0.25
Nodes (7): name, private, version, type, engines, node, npm

## Knowledge Gaps
- **301 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+296 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **288 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `BrokerScope` connect `compilerOptions` to `vite.config.ts`, `.new`, `shared/api/commands.ts`, `commands/trading.rs`, `ui/AppShell.tsx`, `.process_buy`, `.record_fill_pnl_once`, `OrderSubmission`, `BrokerSymbol`, `.new`, `Client`, `HeaderMap`, `strategy/ui/Page.tsx`, `ui/StockChart.tsx`, `AccountProfile`, `databaseManagementSection.tsx`, `DatabaseManager`, `market/mod.rs`, `server/trading.rs`, `security.rs`, `MutexGuard`, `.new`, `commands/database.rs`, `submission.rs`, `place_toss_order`, `MarketDayCalendar`, `SubmissionDeps`, `holding`, `toss/orders.rs`, `broker/adapter.rs`, `package.json`, `TossAccessToken`, `build.rs`?**
  _High betweenness centrality (0.104) - this node is a cross-community bridge._
- **Why does `ServerState` connect `scripts` to `release-version.mjs`, `package.json`, `commands/trading.rs`, `toss/tests.rs`, `StrategyStore`, `.new`, `detect.rs`, `profiles.rs`, `StockStore`, `dependencies`, `Client`, `TradeStore`, `strategy-configure/index.ts`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Why does `OhlcCandle` connect `devDependencies` to `collect_pages`, `high_tests.rs`, `.new`, `src/api/types.ts`, `ConsecutiveMoveStrategy`, `작업 기록: <짧은 제목>`, `PositionTracker`, `strategyResearchTypes.ts`, `CurrencyLedger`, `Mutex`, `HARNESS_MAP.md`, `broker/mod.rs`, `initialize_active_strategy_history`, `strategy/ui/Page.tsx`, `log-filter/index.ts`, `ui/StockChart.tsx`, `.load_sync`, `place_toss_order`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _301 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `simulation.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.04371184371184371 - nodes in this community are weakly interconnected._
- **Should `hooks.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.12121212121212122 - nodes in this community are weakly interconnected._
- **Should `shared/api/commands.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.10810810810810811 - nodes in this community are weakly interconnected._