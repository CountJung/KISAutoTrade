# Graph Report - KISAutoTrade  (2026-10-31)

## Corpus Check
- 514 files · ~361,446 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 7267 nodes · 16095 edges · 534 communities (320 shown, 214 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 892 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8a9d1b54`
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
- TossApiError
- LeveragedTrendHoldStrategy
- leveragedTrendHoldPreviewChart.tsx
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
- MarketDayCalendar
- DeviationStrategy
- toss/orders.rs
- Self
- consecutive/tests.rs
- holding
- Arc
- BalanceItem
- ui/OverseasStockChart.tsx
- config
- input
- .prepare_confirmed_ownership
- TossTokenState
- .preview_signals_with_execution
- strong_close/tests.rs
- .new
- HashMap
- HeaderMap
- broker/candles.rs
- StockStore
- 작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단
- budget/recovery/tests.rs
- commands.rs
- PendingOrderStore
- BrokerHoldingView
- BrokerAdapterResult
- BrokerCurrency
- BrokerId
- Default
- submission.rs
- strong_close_history_tests.rs
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
- failedBreakoutControls.tsx
- From
- bounded_optional_integer
- EntryContext
- config
- T
- RiskManager
- read_json_or_default
- ma_cross_entry_tests.rs
- strategy_preview/tests.rs
- collect_daily_history
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
- fills.rs
- trading/ui/Page.tsx
- SubmissionDeps
- scaled_total
- guard/recovery/tests.rs
- StrongCloseState
- run_prepared_preview
- trendFilterControls.tsx
- KisBrokerAdapter
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
- ChartCandle
- DatabaseManager
- MaCrossParams
- strategy_preview/trend_filter_tests.rs
- ContinuousStrategyPreviewInput
- Option
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
- DailyStats
- ExecutedOrder
- Option
- OverseasExecutedOrder
- State
- String
- TradeRecord
- lth_preview_assessment
- OrderSubmission
- volatility_manager
- submission.rs
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
- config/mod.rs
- strategy-failed-breakout-policy.spec.ts
- StrongCloseParams
- StrategyConfig
- attach_comparison
- current_position_snapshot
- ReplayAssessmentView
- place_prepared_order
- finish_closed_batch
- .submit_order_shared
- .restore_recovered_submission_at
- mockApi
- strategy-strong-close-policy.spec.ts
- TradeStore
- managed_paths
- candles/tests.rs
- .validate_recovery_history
- report_json
- database/tests.rs
- initialize_strategy_warmup_with_daily_bars
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
- State
- String
- volatility_entry_tests.rs
- leveragedTrendHoldPreviewChart.tsx
- AtrWindow
- VolatilityExpansionState
- strongCloseControls.tsx
- TossTokenState
- notional/tests.rs
- database_archive.rs
- momentumEntryControls.tsx
- restore
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
- high_tests.rs
- package.json
- kis-auto-trade

## God Nodes (most connected - your core abstractions)
1. `StrategyConfig` - 128 edges
2. `BrokerScope` - 103 edges
3. `invoke()` - 96 edges
4. `ServerState` - 83 edges
5. `OhlcCandle` - 81 edges
6. `AppState` - 80 edges
7. `Signal` - 79 edges
8. `AccountProfile` - 54 edges
9. `BrokerSymbol` - 51 edges
10. `mockApi()` - 46 edges

## Surprising Connections (you probably didn't know these)
- `Strategy()` --indirect_call--> `entries()`  [INFERRED]
  src/pages/strategy/ui/Page.tsx → tests/e2e/uiux-log-status.spec.ts
- `useLeveragedTrendHoldEditorModel()` --indirect_call--> `quantity()`  [INFERRED]
  src/pages/strategy/ui/useLeveragedTrendHoldEditorModel.ts → tests/e2e/trading-form-contract.spec.ts
- `useTradingOrderModel()` --indirect_call--> `qty()`  [INFERRED]
  src/pages/trading/ui/useTradingOrderModel.ts → tests/e2e/uiux-keyboard-selection.spec.ts
- `mockNative()` --indirect_call--> `command()`  [INFERRED]
  tests/e2e/budget-recovery.spec.ts → scripts/database-tls-contract.mjs
- `renderTodo()` --indirect_call--> `status()`  [INFERRED]
  scripts/render-todo.mjs → tests/e2e/uiux-log-status.spec.ts

## Import Cycles
- 2-file cycle: `src-tauri/src/trading/simulation.rs -> src-tauri/src/trading/simulation/atr.rs -> src-tauri/src/trading/simulation.rs`
- 2-file cycle: `src-tauri/src/server/mod.rs -> src-tauri/src/server/profiles.rs -> src-tauri/src/server/mod.rs`

## Communities (534 total, 214 thin omitted)

### Community 0 - "OwnershipLedger"
Cohesion: 0.38
Nodes (4): OrderManager, OwnershipManifest, Result, String

### Community 1 - "shared/api/commands.ts"
Cohesion: 0.04
Nodes (87): handleLookupTossAccounts(), handleSubmit(), handleSubmit(), activateEmergencyStop(), addProfile(), checkConfig(), checkForUpdate(), checkTossOrderPreflight() (+79 more)

### Community 2 - "rest/types.rs"
Cohesion: 0.12
Nodes (27): BalanceItem, BalanceResponse, BalanceSummary, ChartCandle, ExecutedOrder, KisOutput1, OrderRequest, OrderResponse (+19 more)

### Community 3 - "RiskManager"
Cohesion: 0.04
Nodes (77): canUseTauriEvents(), useBackendEvents(), useAutoTradingBudget(), useUpdateAutoTradingBudget(), useKisExecutedByRange(), useOverseasExecutedByRange(), useStatsByRange(), useTradesByRange() (+69 more)

### Community 4 - "simulation/minute_timing_tests.rs"
Cohesion: 0.08
Nodes (43): parse_minute_instant(), DateTime, Option, Utc, diagnose_minute_timing(), local(), LthMinuteTimingView, MinuteLabelConvention (+35 more)

### Community 5 - "TradeRecord"
Cohesion: 0.14
Nodes (22): account_less_scope_matches_any_account_of_same_broker(), default_currency(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), BrokerId, Mutex (+14 more)

### Community 6 - "FiftyTwoWeekHighStrategy"
Cohesion: 0.11
Nodes (5): FiftyTwoWeekHighStrategy, HashMap, Option, String, bounded_window()

### Community 7 - "StrongCloseStrategy"
Cohesion: 0.11
Nodes (3): HashMap, Self, StrongCloseStrategy

### Community 8 - "TrendFilterStrategy"
Cohesion: 0.07
Nodes (21): initialize_strategy_warmup(), initialize_strategy_warmup_with_daily_bars(), config(), daily(), HashMap, Item, Iterator, Option (+13 more)

### Community 9 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.11
Nodes (14): default_strategy_broker_id(), EntryFilterPreparation, BrokerId, Into, Option, Self, String, Value (+6 more)

### Community 10 - "OrderManager"
Cohesion: 0.08
Nodes (26): scripts, build, build:app, build:app:debug, build:web, check:fsd, check:graphify, check:lockfiles (+18 more)

### Community 11 - "ui/AppShell.tsx"
Cohesion: 0.05
Nodes (40): react, SettingsState, LEVEL_COLORS, Log(), LogLevel, dashboardRoute, historyRoute, logRoute (+32 more)

### Community 12 - "toss/types.rs"
Cohesion: 0.08
Nodes (37): Handshake, RootCertStore, SocketAddr, builder_with_roots(), client(), client_builder(), ClientBuilder, echo() (+29 more)

### Community 13 - "toss_market.rs"
Cohesion: 0.12
Nodes (23): TossOrderbookEntry, get_toss_chart_data(), get_toss_chart_data_for_profile(), get_toss_market_calendar(), get_toss_market_calendar_for_profile(), get_toss_market_snapshot(), get_toss_market_snapshot_for_profile(), get_toss_stock_safety() (+15 more)

### Community 14 - "DatabaseManager"
Cohesion: 0.17
Nodes (13): MySqlPool, DatabaseManager, DatabasePool, install_database_manager(), Arc, AtomicBool, DatabaseStatusView, DatabaseTransferResult (+5 more)

### Community 15 - "market_hours.rs"
Cohesion: 0.11
Nodes (20): config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope(), daily_buy_order_limit_is_disabled_even_after_submission_count_increases(), daily_order_limit_isolated_by_broker_account_scope(), daily_sell_order_limit_blocks_after_submission_count_reaches_limit(), default_atr_stop_multiplier(), default_daily_buy_order_limit() (+12 more)

### Community 16 - "database_io.rs"
Cohesion: 0.12
Nodes (22): app(), bearer_token(), constant_time_eq(), cross_origin_mutation_is_rejected_even_with_token(), has_same_origin(), mutation_without_token_is_rejected(), parse_bool_env(), rejection() (+14 more)

### Community 17 - "BrokerAccountId"
Cohesion: 0.12
Nodes (21): BrokerAdapterResult, Option, Vec, restart_rebuilds_order_counts_and_consecutive_losses_from_ledgers(), broker_scope_serializes_with_camel_case_fields(), BrokerAccountId, BrokerClientOrderId, BrokerCurrency (+13 more)

### Community 19 - "observation.rs"
Cohesion: 0.11
Nodes (16): advance_owned_age(), age_preserves_pending_partial_same_quantity_and_unknown_origins(), begin_live_trend(), both_kinds_preserve_exact_checkpoint_shape_and_pending_fill_origin(), checkpoints_reject_cross_kind_unknown_fields_and_contradictions(), decode_owned_observation(), encode_owned_observation(), observe_trend() (+8 more)

### Community 20 - "accounts.rs"
Cohesion: 0.18
Nodes (16): buildTree(), compareNames(), currentDocument, fail(), generateInventory(), listRepositoryFiles(), mode, nextDocument (+8 more)

### Community 21 - "history/ui/Page.tsx"
Cohesion: 0.14
Nodes (15): chart(), Control, daily, data(), domestic(), enter(), latest(), minutes (+7 more)

### Community 22 - "page_view"
Cohesion: 0.07
Nodes (65): BrokerCandlePage, candle_market_date(), CandleCursor, CandlePageRequest, kis_cursor_date(), next_toss_cursor(), previous_kis_cursor(), BrokerMarket (+57 more)

### Community 23 - "toss_market.rs"
Cohesion: 0.11
Nodes (30): IntoIterator, applies_retry_after_to_group_pause(), epoch_ms_now(), parse_delay_header(), rate_limit_reset_delay(), RateLimitGroupState, RateLimitGroupStatus, RateLimitScheduler (+22 more)

### Community 24 - "ServerState"
Cohesion: 0.05
Nodes (40): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, Korean, app, security (+32 more)

### Community 25 - "BrokerQuantity"
Cohesion: 0.16
Nodes (25): broker_money_view_amount(), confirmed_small_order_fill(), ConfirmedSmallOrderFill, is_toss_order_final(), money_view_from_decimal(), poll_toss_order_detail(), Arc, BrokerCurrency (+17 more)

### Community 26 - "StrategyManager"
Cohesion: 0.07
Nodes (19): StrategySignal, live_high_excludes_unconfirmed_latest_bar_without_shortening_252_requirement(), owned_target_matches(), Box, BrokerId, BrokerMarket, BTreeMap, Default (+11 more)

### Community 27 - "RunFence"
Cohesion: 0.13
Nodes (15): AtomicU64, configuration_save_rollback_aba_invalidates_quote_and_position_feedback(), delayed_name_and_late_rollback_do_not_mutate_the_new_run(), delayed_quote_cannot_tick_a_restarted_same_scope_run(), Fixture, Probe, Arc, AtomicUsize (+7 more)

### Community 28 - "tauri.conf.json"
Cohesion: 0.11
Nodes (12): DailyOrderCountEntry, DailyOrderSide, RiskManager, RiskRuntimeState, BTreeMap, Default, HashMap, NaiveDate (+4 more)

### Community 29 - "prepare"
Cohesion: 0.10
Nodes (54): apply(), bars(), completed_252_keeps_final_high_and_uses_high_instead_of_close(), completed_volume_history_is_atomic_scoped_and_does_not_change_owned_protection(), config_aba_and_profile_scope_switch_cannot_revive_old_tickets(), configured_512_window_and_invalid_ohlc_are_not_mislabeled_ready(), consecutive_manager(), consecutive_pending_failure_midnight_and_config_refresh_preserve_held_exit_streak() (+46 more)

### Community 30 - "strategy/ui/Page.tsx"
Cohesion: 0.04
Nodes (67): useRefreshStockList(), useStockSearch(), usePreviewLeveragedTrendHold(), useStrategies(), useUpdateStrategy(), BollingerControls(), handlePickerSelect(), handlePickerUsSearch() (+59 more)

### Community 31 - "accounts.rs"
Cohesion: 0.12
Nodes (41): AppConfigView, OverseasBalanceItem, OverseasBalanceSummary, add_profile(), AddProfileInput, apply_active_profile(), BalanceResult, broker_market_sort_key() (+33 more)

### Community 32 - "RsiStrategy"
Cohesion: 0.11
Nodes (9): HashMap, Option, Result, Self, String, Value, VecDeque, RsiState (+1 more)

### Community 33 - "settings.rs"
Cohesion: 0.10
Nodes (44): AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_trading_type(), DetectTradingTypeResult, ExchangeRateSource (+36 more)

### Community 34 - "order/ownership/tests.rs"
Cohesion: 0.19
Nodes (19): bytes(), checked_quantity_cost_and_signed_profit_overflows_are_atomic(), cumulative_buys_and_sells_track_native_weighted_cost_and_proportional_profit(), cumulative_progress_replaces_only_the_latest_durable_application_result(), duplicate_and_regressing_cumulative_reports_are_noops_but_contradictions_fail(), exact_account_profile_mode_owner_market_and_symbol_are_separate_and_orders_cannot_rebind(), fill(), invalid_oversized_unowned_and_duplicate_metadata_fail_atomically_and_on_restore() (+11 more)

### Community 35 - ".submit_order_shared"
Cohesion: 0.19
Nodes (16): OwnedStrategySignal, ProviderOrderResponse, From, Into, Mutex, Option, OrderResponse, RestOrderSide (+8 more)

### Community 36 - "strategy_preview.rs"
Cohesion: 0.13
Nodes (36): align_accounting_scenarios(), broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc() (+28 more)

### Community 37 - "server/records.rs"
Cohesion: 0.13
Nodes (10): DateRangeQuery, frontend_log_handler(), FrontendLogBody, kis_executed_handler(), set_archive_config_handler(), set_log_config_handler(), SetArchiveConfigBody, SetLogConfigBody (+2 more)

### Community 38 - "databaseManagementSection.tsx"
Cohesion: 0.12
Nodes (28): automatic_missing_context_and_stale_reservation_leave_budget_and_pending_unchanged(), buy(), delayed_old_account_insufficient_balance_does_not_suspend_new_account(), legacy_configured_account_without_profile_allows_manual_but_fences_client_replacement(), local_holdings(), local_order(), local_single_order(), LocalProvider (+20 more)

### Community 39 - "DatabaseManager"
Cohesion: 0.15
Nodes (24): buy(), checkpoint_rejects_old_run_and_configuration_generation_without_writing(), deviation_feedback_pending_fill_partial_and_restart_preserve_exact_owner_age(), feedback_assigns_only_exact_owner_amid_manual_and_other_account_holdings(), feedback_cold_lth_restores_metadata_and_partial_change_preserves_live_high(), feedback_never_transfers_same_account_quantity_across_profile_or_mode(), key(), lth_config() (+16 more)

### Community 40 - "AppState"
Cohesion: 0.15
Nodes (18): Future, Output, capture_cycle(), ExecutionCycle, Arc, FnOnce, KisRestClient, Mutex (+10 more)

### Community 41 - "DatabaseConfig"
Cohesion: 0.07
Nodes (57): MySqlSslMode, PgSslMode, assert_failed_save_preserves_pair(), cancelled_save_before_memory_publication_blocks_further_connections(), clear_recovery_marker(), committed_file_without_memory_publication_stays_fail_closed(), create_recovery_marker(), DatabaseManager (+49 more)

### Community 42 - "MomentumStrategy"
Cohesion: 0.10
Nodes (14): legacy_params(), MomentumEntryMode, MomentumParams, MomentumState, MomentumStrategy, Default, HashMap, Option (+6 more)

### Community 43 - "databaseManagementSection.tsx"
Cohesion: 0.07
Nodes (39): useClearDatabaseTables(), useCreateDatabaseTables(), useDatabaseConfig(), useDropDatabaseTables(), useExportDatabaseToJson(), useImportJsonToDatabase(), useJsonStorageInventory(), useSaveDatabaseConfig() (+31 more)

### Community 44 - "dashboard/ui/Page.tsx"
Cohesion: 0.28
Nodes (12): lth_preview_signals_for_interval(), F, LeveragedTrendHoldPreviewSignal, OhlcCandle, LeveragedTrendHoldPreviewStep, LeveragedTrendHoldTimedCandle, lth_default_qty(), LeveragedTrendHoldStrategy (+4 more)

### Community 45 - "commands/records.rs"
Cohesion: 0.27
Nodes (12): collect_trade_archive_stats(), collect_trade_archive_stats_active(), collect_trade_day_dirs(), dir_size_bytes(), get_trade_archive_config(), get_trade_archive_stats(), purge_old_trade_files(), purge_trade_archive_active() (+4 more)

### Community 46 - "PriceConditionStrategy"
Cohesion: 0.19
Nodes (20): fetch_overseas_tick(), fetch_toss_tick(), is_market_closed_error(), poll_symbols_tick(), PriceSource, Arc, KisRestClient, Mutex (+12 more)

### Community 47 - "DiscordNotifier"
Cohesion: 0.14
Nodes (7): DiscordNotifier, Client, Result, Self, String, NotificationEvent, NotificationLevel

### Community 48 - "autoTradingBudgetSection.tsx"
Cohesion: 0.06
Nodes (60): useOverseasPrice(), usePrice(), useTossMarketCalendar(), useTossMarketSnapshot(), useTossStockSafety(), useModifyTossOrder(), usePlaceOrder(), usePlaceOverseasOrder() (+52 more)

### Community 49 - "FailedBreakoutStrategy"
Cohesion: 0.09
Nodes (5): FailedBreakoutStrategy, HashMap, Self, prep(), EntryFilterPreparation

### Community 50 - "PositionTracker"
Cohesion: 0.05
Nodes (30): get_positions(), get_strategies(), PositionView, BrokerId, CmdResult, From, Option, Self (+22 more)

### Community 51 - "ReplaySession"
Cohesion: 0.11
Nodes (19): krw_to_display(), parse_replay_time(), phase_for_index(), replay_time_preserves_rfc3339_instant_and_subseconds(), DateTime, FixedOffset, PendingSignal, ReplaySession (+11 more)

### Community 52 - "leveraged_trend_hold.rs"
Cohesion: 0.10
Nodes (34): BollingerParams, Default, Self, lth_default_adx_period(), lth_default_breakeven_buffer(), lth_default_buy_adx(), lth_default_buy_rsi(), lth_default_ema_long() (+26 more)

### Community 53 - "AppState"
Cohesion: 0.07
Nodes (36): File, AppState, balance_summary_total_krw(), CmdError, make_rest_client(), parse_amount_f64(), parse_amount_i64(), Arc (+28 more)

### Community 54 - "security.rs"
Cohesion: 0.20
Nodes (26): FetchFuture, Fn, at_most_two_jobs_run_and_other_instruments_are_not_starved(), completed(), failures_use_backoff_and_do_not_retry_on_every_quote_tick(), history(), insufficient_history_reports_real_count_reason_and_retry(), maintain_captures_the_new_scope_atomically_after_a_waiting_start() (+18 more)

### Community 55 - "VolatilityExpansionStrategy"
Cohesion: 0.08
Nodes (11): Default, HashMap, Option, Result, Self, String, Value, VecDeque (+3 more)

### Community 56 - "simulation.rs"
Cohesion: 0.07
Nodes (55): DividendScenarioPoint, FxScenarioPoint, ReplayAccountingView, Vec, apply_slippage(), BacktestPhaseView, BacktestReportView, BacktestSummaryView (+47 more)

### Community 57 - "BrokerSymbol"
Cohesion: 0.08
Nodes (24): Display, BrokerAdapterResult, BrokerCurrency, BrokerId, Debug, Default, Formatter, Into (+16 more)

### Community 58 - "order/budget.rs"
Cohesion: 0.20
Nodes (17): BudgetCurrencyView, allocation_cannot_withdraw_committed_cash_or_overflow(), AutoTradingBudgetView, budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent(), BudgetAccount, BudgetLedger, cost(), manager() (+9 more)

### Community 59 - "OrderManager"
Cohesion: 0.09
Nodes (25): Barrier, daily_order_limit_key(), estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), OrderManager, PendingOrder, Arc (+17 more)

### Community 60 - "OrderRecord"
Cohesion: 0.18
Nodes (14): OrderRecord, OrderSide, OrderStore, parallel_appends_do_not_lose_order_records(), BrokerId, Into, Mutex, NaiveDate (+6 more)

### Community 61 - "settings/ui/Page.tsx"
Cohesion: 0.06
Nodes (42): useSetTradeArchiveConfig(), useTradeArchiveConfig(), useTradeArchiveStats(), useLogConfig(), useSetLogConfig(), useWriteFrontendLog(), useSetStockUpdateInterval(), useStockListStats() (+34 more)

### Community 62 - "run_backtest"
Cohesion: 0.33
Nodes (31): assumptions(), bar(), buy(), directional_fees_and_native_minimum_are_converted_with_the_current_fx(), dividend_adjusted_prices_reject_known_double_counting(), dividend_entitlement_precedes_pending_open_and_respects_partial_exit_quantity(), future_fx_point_cannot_revalue_earlier_mark_or_buy(), fx_changes_krw_mark_and_realized_basis_without_changing_native_strategy_entry() (+23 more)

### Community 63 - "DatabasePool"
Cohesion: 0.13
Nodes (18): OverseasPriceResponse, KisOrderRejected, KisRestClient, read_kis_response_text(), Arc, AtomicBool, ExecutedOrder, HeaderMap (+10 more)

### Community 64 - "TradeRecord"
Cohesion: 0.10
Nodes (7): maps_holding_to_broker_domain(), maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request(), BrokerSymbol, TossOrderbookResponse, TossPriceLimitResponse

### Community 65 - "server/trading.rs"
Cohesion: 0.18
Nodes (22): BrokerId, ContinuousStrategyPreviewInput, Json, Option, Path, Result, State, StatusCode (+14 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.11
Nodes (26): assert_auth(), candle_values(), chart_response_handles_empty_output_and_malformed_multibyte_body(), chart_response_preserves_http_and_provider_errors(), chart_response_propagates_announced_and_observed_body_caps(), domestic_latest_first_fixture_reverses_and_maps_ohlcv(), domestic_request_keeps_date_bounds_period_adjustment_and_query_order(), overseas_dailyprice_request_uses_its_own_tr_id_and_period_codes() (+18 more)

### Community 67 - "scripts"
Cohesion: 0.13
Nodes (45): activate_emergency_handler(), app_config_handler(), check_config_handler(), check_update_handler(), clear_buy_suspension_handler(), clear_emergency_handler(), exchange_rate_handler(), exchange_rate_status_handler() (+37 more)

### Community 68 - "build_strategy"
Cohesion: 0.09
Nodes (60): request(), ConfigurationUpdateError, ConfigurationUpdateRequest, ConfigurationUpdateState, Arc, JoinHandle, Mutex, Option (+52 more)

### Community 69 - "commands/toss.rs"
Cohesion: 0.21
Nodes (18): check_toss_order_preflight(), check_toss_order_preflight_for_profile(), parse_toss_order_side(), BrokerCurrency, BrokerId, BrokerMarket, BrokerMoneyView, CmdResult (+10 more)

### Community 70 - "strategyPolicyControls.tsx"
Cohesion: 0.05
Nodes (84): ConsecutiveEntryControls(), ConsecutiveEntryField(), consecutiveEntryParamError(), ConsecutiveEntrySummary(), fieldError(), fields, isConsecutiveEntryParam(), parsedDraft() (+76 more)

### Community 71 - "commands/risk.rs"
Cohesion: 0.13
Nodes (26): absent_unknown_and_similarly_prefixed_kis_tr_ids_preserve_pending_without_querying(), actual_fill_crosses_same_scope_restart_with_identical_provider_snapshot(), delayed_reconciliation_cannot_mutate_new_scope_and_applies_when_original_is_reselected(), direct_fill_and_zero_fill_terminal_cannot_mutate_another_execution_scope(), exact_eight_order_tr_ids_select_mode_and_toss_bypasses_kis_mode_classification(), executed(), persisted_kis_pending_never_queries_or_applies_in_the_opposite_mode(), Provider (+18 more)

### Community 72 - ".new"
Cohesion: 0.25
Nodes (25): add_profile_handler(), AddProfileBody, apply_profile_change(), default_body_broker_id(), delete_profile_handler(), DeleteProfileBody, detect_profile_handler(), detect_trading_type_handler() (+17 more)

### Community 73 - "strategyPreviewPanel.tsx"
Cohesion: 0.13
Nodes (25): usePreviewStrategy(), usePreviewStrategyContinuous(), assertPageScope(), calendarLabel(), CandleCollection, CandleCollectionEnd, candleTimestamp(), checkCancelled() (+17 more)

### Community 74 - "KisWebSocketClient"
Cohesion: 0.08
Nodes (20): FormatTime, FrontendLogInput, get_recent_logs(), get_stats_by_range(), get_trades_by_range(), GetStatsByRangeInput, GetTradesByRangeInput, save_trade() (+12 more)

### Community 75 - "OhlcCandle"
Cohesion: 0.13
Nodes (12): atr14(), mean_close(), mean_true_range14(), Option, VecDeque, Counters, EntryContext, Counters (+4 more)

### Community 76 - "read_json_or_default"
Cohesion: 0.15
Nodes (14): LeveragedTrendHoldEntry, LeveragedTrendHoldMarketState, LeveragedTrendHoldParams, LeveragedTrendHoldPosition, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldStrategy, Default, HashMap (+6 more)

### Community 77 - "database_projection.rs"
Cohesion: 0.08
Nodes (56): MySql, Postgres, drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract(), postgresql_rejects_invalid_credentials() (+48 more)

### Community 78 - "ConsecutiveMoveStrategy"
Cohesion: 0.08
Nodes (12): HistoryReadiness, ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, Default, EntryContext, EntryFilterPreparation, HashMap (+4 more)

### Community 79 - "project-map.mjs"
Cohesion: 0.20
Nodes (13): ChartRequest, ChartRequest<'a>, KisRestClient, parse_domestic_response(), parse_overseas_response(), ChartCandle, Client, HeaderMap (+5 more)

### Community 80 - "start_trading"
Cohesion: 0.09
Nodes (31): fetch_account_risk_balance_krw(), fetch_toss_risk_balance_krw(), Arc, KisRestClient, Result, String, clear_buy_suspension(), failed_configuration_pause_never_becomes_market_close_and_same_revision_resumes() (+23 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.09
Nodes (12): live_feedback_price_condition_changes_position_only_after_confirmed_fill(), strategy(), PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick(), HashMap, Option (+4 more)

### Community 82 - "MovingAverageCrossStrategy"
Cohesion: 0.10
Nodes (12): config(), initializer_restores_previous_averages_for_first_evaluation_cross(), insufficient_history_has_no_previous_long_average(), MaCrossState, MovingAverageCrossStrategy, EntryContext, EntryFilterPreparation, HashMap (+4 more)

### Community 83 - "accountHooks.ts"
Cohesion: 0.08
Nodes (50): AccountConfigRefreshError, accountScope(), holdingsPrefixes, profileMutationKey, profileMutationScope, reconcileRejectedProfileMutation(), refreshAccountConfig(), resetAccountHoldings() (+42 more)

### Community 84 - "shared/api/types.ts"
Cohesion: 0.22
Nodes (15): buy_preflight_adds_commission_and_blocks_when_cash_is_short(), buy_preflight_passes_when_required_cash_is_available(), evaluate_order_preflight(), format_money_amount(), format_quantity(), invalid_commission_ratios_block_instead_of_becoming_zero_fee(), OrderPreflightConstraints, OrderPreflightDecision (+7 more)

### Community 85 - "strategyResearchPanel.tsx"
Cohesion: 0.07
Nodes (42): clearExperimentSlots(), compactSnapshot(), ExperimentScope, loadExperimentSlots(), normalizePart(), readIndex(), saveExperimentSlot(), storageKey() (+34 more)

### Community 86 - "collect_kis_pending_fills"
Cohesion: 0.17
Nodes (24): OrderStatus, collect_kis_pending_fills(), collect_toss_pending_fills(), domestic_order_is_terminal(), domestic_order_terminal_status(), domestic_provider_status(), kis_pending_from(), normalized_execution_u64() (+16 more)

### Community 87 - "support.rs"
Cohesion: 0.20
Nodes (16): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, BrokerCurrency, CmdResult, Option (+8 more)

### Community 88 - "strategy_with_params"
Cohesion: 0.10
Nodes (44): LeveragedTrendHoldEntry, LeveragedTrendHoldStrategy, average_fill_price_drives_preview_stop_even_when_observation_price_rises(), daily_context_cannot_prepare_minute_indicators_or_rebound(), daily_reinitialization_keeps_intraday_bollinger_and_actual_position(), failed_buy_feedback_does_not_create_phantom_position(), failed_sell_feedback_keeps_actual_position_for_next_risk_exit(), intraday_and_daily_context_buffers_have_independent_bounds() (+36 more)

### Community 89 - "HoldingsQuery"
Cohesion: 0.19
Nodes (17): DomesticHoldings, OverseasHoldings, ExpectedSource, HoldingsMarkets, HoldingsQuery, HoldingsUpdate, normalize_overseas_order_exchange(), OrderManager (+9 more)

### Community 90 - "simulation/accounting.rs"
Cohesion: 0.21
Nodes (12): CostSource, DividendPriceBasis, DividendScenarioPoint, FxScenarioPoint, HashSet, Item, Iterator, Result (+4 more)

### Community 91 - "TradeStore"
Cohesion: 0.15
Nodes (41): account_sync_error(), balance_handler(), broker_holdings_handler(), candle_page_handler(), chart_handler(), ChartQuery, executed_handler(), invalid_order_input() (+33 more)

### Community 92 - "strategyResearchTypes.ts"
Cohesion: 0.06
Nodes (46): accountingAssumptionsLabel(), AccountingAssumptionSummary(), AccountingResults(), ComparisonResults(), EditorProps, numberInRange(), parseScenarios(), percent() (+38 more)

### Community 93 - "@mui/icons-material"
Cohesion: 0.14
Nodes (11): Arc, AsyncMutex, shared_toss_token_state(), toss_token_state_key(), TossTokenState, TossAccessToken, TossAccessTokenStatus, check() (+3 more)

### Community 94 - "balance_store.rs"
Cohesion: 0.19
Nodes (19): fixture(), fixture_with_base(), ownership_fill_empty_sell_snapshot_uses_exact_owner_cost_across_partial_and_repeat(), ownership_fill_exact_native_notional_preserves_301_total_and_partial_remainder(), ownership_fill_larger_terminal_cumulative_waits_for_earlier_intent_then_applies_every_share(), ownership_fill_legacy_applying_intent_requires_durable_date_fx_and_notional(), ownership_fill_legacy_sell_does_not_adopt_other_owners_account_average(), ownership_fill_manual_saved_cost_survives_empty_account_snapshot() (+11 more)

### Community 95 - "LeveragedTrendHoldParams"
Cohesion: 0.06
Nodes (42): useAppConfig(), useProfiles(), useTodayStats(), usePendingOrders(), useRefreshInterval(), useActivateEmergencyStop(), useClearEmergencyStop(), useRiskConfig() (+34 more)

### Community 96 - "check"
Cohesion: 0.08
Nodes (23): DOM, DOM.Iterable, ES2020, src, compilerOptions, allowImportingTsExtensions, isolatedModules, jsx (+15 more)

### Community 97 - "compilerOptions"
Cohesion: 0.60
Nodes (3): create(), mariadb_ddl(), postgres_ddl()

### Community 98 - "holdings/tests.rs"
Cohesion: 0.31
Nodes (16): HoldingsUpdate, a_change_to_either_market_rejects_both_without_partial_publication(), configuration_lease_rejects_capture_and_inflight_response_even_after_release(), domestic(), empty_and_zero_holdings_clear_only_the_queried_market(), fixture(), held(), identity_changes_reject_without_mutating_either_tracker() (+8 more)

### Community 99 - "MeanReversionStrategy"
Cohesion: 0.11
Nodes (8): MeanReversionStrategy, HashMap, Option, Result, Self, String, Value, VecDeque

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
Cohesion: 0.20
Nodes (7): OrderManager, Option, OrderSide, PendingOrder, Result, String, Vec

### Community 104 - "dependencies"
Cohesion: 0.09
Nodes (22): @emotion/react, @emotion/styled, @mui/icons-material, @mui/material, dependencies, @emotion/react, @emotion/styled, @mui/icons-material (+14 more)

### Community 106 - "TossApiError"
Cohesion: 0.18
Nodes (12): AccessToken, Arc, Client, DateTime, Mutex, Option, Result, Self (+4 more)

### Community 107 - "LeveragedTrendHoldStrategy"
Cohesion: 0.08
Nodes (9): LeveragedTrendHoldStrategy, Option, Result, String, Value, LeveragedTrendHoldStrategy, parse_hhmm(), Option (+1 more)

### Community 111 - "exchange.rs"
Cohesion: 0.27
Nodes (14): is_domestic_symbol(), is_krx_open(), is_krx_open_at(), is_market_open_for(), is_market_open_for_with_calendar(), is_market_open_for_with_calendar_policy(), is_us_open(), is_us_open_at() (+6 more)

### Community 112 - "toss_currency_from_view"
Cohesion: 0.19
Nodes (16): LeveragedTrendHoldPreviewInput, TossModifyOrderInput, TossOpenOrdersInput, TossOrderPreflightInput, TossSmallBuyVerificationInput, active_profile(), leveraged_trend_hold_preview_handler(), toss_chart_handler() (+8 more)

### Community 113 - "strategy/tests.rs"
Cohesion: 0.06
Nodes (50): kis_rejection_keeps_fixed_message_and_typed_anyhow_identity(), definitively_rejected(), Error, Option, StatusCode, String, Value, TossApiError (+42 more)

### Community 114 - "day_event_tests.rs"
Cohesion: 0.13
Nodes (15): lightweight-charts, lightweight-charts, useOverseasChartData(), buildChartOptions(), CandlePoint, ChartType, CrosshairData, LinePoint (+7 more)

### Community 115 - "setup-local.sh"
Cohesion: 0.13
Nodes (17): classify_token_response(), detect_trading_type(), DetectedTradingType, DetectTokenReq, DetectTradingTypeError, is_paper_key_rejected_by_real_domain(), is_real_key_rejected_by_paper_domain(), mentions_app_key() (+9 more)

### Community 116 - "Signal"
Cohesion: 0.09
Nodes (33): RecentSideHistory, ScopedSymbol, blocks_reentry_after_stop_loss_sell_on_same_day(), blocks_repeated_same_side_signal_inside_cooldown(), blocks_tiny_profit_after_costs(), blocks_whipsaw_after_repeated_opposite_signals(), buy_signal(), cooldown_is_isolated_by_broker_account_scope() (+25 more)

### Community 117 - "main.rs"
Cohesion: 0.23
Nodes (17): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), calculate_atr(), initialize_active_strategy_history(), legacy_history_symbols(), Arc (+9 more)

### Community 118 - "volatility_expansion/tests.rs"
Cohesion: 0.21
Nodes (25): bar(), buy(), buy_blocked(), config(), daily(), Value, sell(), strategy() (+17 more)

### Community 119 - "generic_tests.rs"
Cohesion: 0.20
Nodes (16): accounting_scenario_labels_match_actual_evaluation_instants_without_shifting(), candle(), daily_rows_keep_existing_trading_dates_and_hash_semantics(), frozen_qqq_cost_and_next_open_feedback_change_the_real_strategy_exit_path(), future_open_cannot_change_an_earlier_signal_or_fill(), input(), legacy_compact_minutes_are_explicit_kst_and_invalid_times_fail_closed(), market_midnight_fixture() (+8 more)

### Community 120 - "preview_strategy_continuous_from_candles"
Cohesion: 0.23
Nodes (24): preview_strategy_continuous_from_candles(), CmdResult, actual_indicator_warmup_and_state_survive_more_than_500_bars_and_arbitrary_partitions(), actual_strategy_single_and_chunk_supply_match_and_pending_orders_cross_both_boundaries(), bounded_request_rejects_empty_or_oversized_chunks_history_and_total(), candle(), changing_future_chunk_prices_preserves_all_earlier_strategy_fills_and_equity(), common_phase_diagnostics_and_warning_match_flat_single_and_partitioned_replays() (+16 more)

### Community 121 - "failed_breakout/tests.rs"
Cohesion: 0.20
Nodes (20): actual_quantity_unknown_origin_has_no_invented_failure_anchor_or_age(), atr_never_weakens_legacy_percent_limb(), atr_uses_prior_gaps_fractional_threshold_and_zero_range_is_ready(), bar(), candidates_do_not_change_ownership_and_same_zero_does_not_erase_anchor(), checkpoint_v1_unknown_and_v2_known_roundtrip_fail_atomically(), config(), context_updates_do_not_clear_anchor_age_peak_armed_and_stays_bounded() (+12 more)

### Community 122 - "discord-notification-config/index.ts"
Cohesion: 0.21
Nodes (10): BrokerAdapter, BrokerAdapterError, default_place_order_returns_unsupported_with_broker_id(), DummyAdapter, BrokerAdapterResult, BrokerId, Error, Send (+2 more)

### Community 123 - "manager"
Cohesion: 0.14
Nodes (22): pc_config(), order(), pc_live_accepted_unfilled_stop_terminal_outcomes_keep_scoped_submission_ban(), pc_live_blocked_and_provider_rejected_stops_record_no_new_daily_ban(), pc_live_config_sized_sale_leaves_remainder_protected_by_default_risk_and_cooldown(), pc_live_partial_buy_uses_ceil_native_cost_and_sell_clamps_below_config(), pc_live_rejected_buy_and_accepted_zero_fill_never_become_owned(), PcLive (+14 more)

### Community 124 - "strategy-configure/index.ts"
Cohesion: 0.33
Nodes (15): commit_rejects_scope_profile_client_and_credential_changes(), config(), configuration_fence_blocks_shared_automatic_and_manual_before_any_http_or_budget_change(), configuration_snapshots_include_real_held_and_flat_markets_and_revalidate_fresh_quantity(), duplicate_entries_use_the_same_first_market_as_the_live_strategy(), fixture(), held_target_removal_and_explicit_pc_lth_market_changes_fail_closed(), legacy_inverse_field_cannot_override_another_lth_targets_market() (+7 more)

### Community 125 - "generic.rs"
Cohesion: 0.21
Nodes (21): chart_volume_to_u64(), normalize_minute_time(), normalize_ordered_rows(), normalize_rows(), normalize_rows_with_order(), BrokerId, ChartCandle, CmdResult (+13 more)

### Community 126 - "Result"
Cohesion: 0.24
Nodes (11): Ok, cumulative(), proportional_cost(), Formatter, Result, S, Value, serialize_generation() (+3 more)

### Community 127 - "PendingOrderStore"
Cohesion: 0.26
Nodes (10): corrupted_primary_never_resurrects_orders_from_a_valid_backup(), PendingOrderStore, restart_snapshot_preserves_scope_client_id_and_fill_watermark(), Mutex, PathBuf, PendingOrder, Result, Self (+2 more)

### Community 128 - "conflicts.rs"
Cohesion: 0.25
Nodes (18): order_side_label(), pending_conflict_blocks_opposite_side_in_same_scope(), pending_conflict_blocks_same_side_in_same_scope(), pending_conflict_reason_for_scope(), pending_conflict_scan_ignores_different_scope(), pending_order(), pending_order_conflict_reason(), pending_order_provider() (+10 more)

### Community 129 - "MarketDayCalendar"
Cohesion: 0.36
Nodes (17): assumptions(), bar(), buy(), diagnostics(), diagnostics_are_optional_camel_case_and_do_not_rewrite_legacy_phase_fields(), empty_single_and_all_in_sample_preserve_existing_phase_shape(), fx_dividend_partial_exit_preserves_original_entry_cost_and_reconciles_split(), near() (+9 more)

### Community 130 - "DailyHistoryTicket"
Cohesion: 0.17
Nodes (14): DailyHistoryEntry, DailyHistoryStatus, DailyHistoryTicket, daily_history_configuration_keeps_exact_json_and_digest(), fingerprint(), pending_status(), BrokerId, DateTime (+6 more)

### Community 131 - "validate_pending_kis_mode"
Cohesion: 0.39
Nodes (5): OrderManager, Option, PendingOrder, Result, validate_pending_kis_mode()

### Community 132 - "BrokerScope"
Cohesion: 0.17
Nodes (18): BudgetRecoveryItem, ApproveBudgetRecoveryInput, BudgetRecoveryItem, BudgetRecoveryView, is_us(), OrderManager, push_bounded(), RecoveryReceipt (+10 more)

### Community 133 - ".search"
Cohesion: 0.19
Nodes (23): KrxItem, KrxProxyItem, KrxProxySearchResponse, KrxResponse, lookup_name_by_code(), NaverAcItem, NaverAcResponse, Option (+15 more)

### Community 134 - "run_backtest_with_history"
Cohesion: 0.23
Nodes (14): atr_history(), atr_risk_uses_previous_history_close_and_evaluation_gap_without_trading_history(), atr_sizing_assumptions(), atr_sizing_never_silently_uses_requested_quantity_when_ranges_are_missing(), daily_loss_and_consecutive_loss_state_roll_over_on_event_date(), deterministic_replay_applies_costs_and_separates_signals_from_fills(), event(), flat_gap_entry() (+6 more)

### Community 135 - "AppConfigView"
Cohesion: 0.20
Nodes (4): OrderManager, Option, String, TradingHealthStatus

### Community 136 - "작업 기록: <짧은 제목>"
Cohesion: 0.37
Nodes (13): bar(), continuous(), params(), request(), ContinuousStrategyPreviewView, String, Value, rsi_cap_counts_after_actual_next_open_fill_across_chunk_boundaries() (+5 more)

### Community 137 - "StatsStore"
Cohesion: 0.06
Nodes (35): fixture, realm(), fmt(), fmtUsd(), History(), HistoryQueryStatusProps, isOverseasTrade(), renderSlippage() (+27 more)

### Community 138 - ".process_buy"
Cohesion: 0.42
Nodes (7): OrderManager, Option, OrderResponse, OrderSide, Result, String, order_exchange_code()

### Community 139 - "CurrencyLedger"
Cohesion: 0.17
Nodes (10): LeveragedTrendHoldStrategy, Option, Vec, VecDeque, LeveragedRapidReboundSnapshot, LeveragedReboundSnapshot, LeveragedTrendSnapshot, PreviewPosition (+2 more)

### Community 141 - "modify_toss_order_for_profile"
Cohesion: 0.16
Nodes (22): AppliedRecord, OwnershipApplied, OwnershipFill, OwnershipIdentity, OwnershipKey, OwnershipLedger, OwnershipOwner, OwnershipPosition (+14 more)

### Community 142 - "deviation/tests.rs"
Cohesion: 0.22
Nodes (29): age_keeps_accumulating_when_option_off_and_survives_warmup_parameter_edits(), age_starts_at_actual_fill_and_counts_only_owned_evaluations(), buy(), config(), crossing_at_boundary_requires_previous_strictly_above(), disabled_unknown_and_other_symbol_observations_cannot_change_held_age(), downward_crossing_retries_zero_feedback_and_cancels_above_threshold(), filled() (+21 more)

### Community 143 - "HARNESS_MAP.md"
Cohesion: 0.25
Nodes (11): Band, candle(), disabled_option_preserves_entries_and_buffer_is_bounded(), downward_breakout_and_insufficient_history_block_entries(), flat_squeeze_needs_completed_upward_breakout(), LeveragedTrendHoldStrategy, live_and_replay_share_confirmed_breakout_and_exit_without_future_prices(), Option (+3 more)

### Community 144 - "2. 목표와 비목표"
Cohesion: 0.16
Nodes (18): lookback(), number(), parse(), Option, Result, String, Value, validate_volatility_expansion_params() (+10 more)

### Community 145 - "react-grid-layout"
Cohesion: 0.18
Nodes (12): buildSnapshot(), fail(), listRepositoryFiles(), mode, parseMode(), REPOSITORY_ROOT, ROOT_CONFIGS, SCRIPT_DIR (+4 more)

### Community 146 - "api/mod.rs"
Cohesion: 0.17
Nodes (11): client(), Fixture, profile(), Arc, Drop, KisRestClient, Mutex, OrderManager (+3 more)

### Community 147 - "rsi/tests.rs"
Cohesion: 0.19
Nodes (20): checkpoint_validation_is_atomic_and_kind_separated(), cold_owned_age_stays_unknown_and_active_cap_offers_conservative_exit(), config(), damaged_optional_runtime_fails_buy_closed_and_retains_custom_core_and_valid_exits(), disabled_cap_still_counts_known_age_and_reconfigure_does_not_reset_it(), entry(), flat_sell_and_held_buy_are_removed_without_mutating_actual_position(), known_age_starts_after_actual_fill_and_preserves_partial_same_quantity_retries() (+12 more)

### Community 148 - "Arc"
Cohesion: 0.25
Nodes (13): bodyHtml(), checkFreshness(), escapeHtml(), inlineMarkdown(), outputPath, parseTasks(), renderTodo(), root (+5 more)

### Community 149 - "AtomicBool"
Cohesion: 0.33
Nodes (21): assumptions(), assumptions_normalize_nonfinite_protection_and_expose_selected_model(), atr_that_only_becomes_ready_after_execution_cannot_size_the_open_order(), bar(), both_touch_uses_stop_first_only_when_selected_and_reports_ambiguity(), buy(), default_same_close_legacy_numeric_fixture_is_preserved(), delayed_fill_uses_next_observed_open_and_final_signal_remains_unfilled() (+13 more)

### Community 150 - "ChartCandle"
Cohesion: 0.19
Nodes (11): Requests, accept(), actual_submissions_sell_only_the_confirmed_owner_and_never_add_fill_to_account_snapshot(), ownership_recheck_after_preparation_prevents_a_stale_sell_reservation(), publish(), Fixture, Json, OrderSide (+3 more)

### Community 151 - "restore_risk_from_today_trades"
Cohesion: 0.24
Nodes (11): restore_risk_from_today_trades(), Arc, CmdResult, Mutex, RiskManager, risk_rebuild_isolates_paper_profit_from_live_losses_for_the_same_account(), risk_rebuild_seeds_trade_event_receipts_and_retries_missing_pending_intent_once(), risk_rebuild_uses_chronological_profits_and_keeps_manual_emergency_stop() (+3 more)

### Community 152 - "Fixture"
Cohesion: 0.19
Nodes (18): recovered_native_stop_guard_survives_full_fill_flat_restart_and_budget_load(), recovery_binding_restart_partial_reverse_terminal_and_reapproval_are_idempotent(), recovery_history_corrupt_primary_cannot_fall_back_to_empty_backup(), recovery_intent_survives_pending_write_failure_and_does_not_clear_block(), recovery_intent_write_failure_never_publishes_pending(), recovery_list_hides_normal_pending_and_prioritizes_unresolved_over_completed_receipts(), recovery_missing_history_primary_with_backup_or_quarantine_fails_closed(), recovery_never_invents_legacy_or_partially_applied_pending() (+10 more)

### Community 153 - "ExecutedOrder"
Cohesion: 0.18
Nodes (11): usd_to_cents(), StockListStats, ChartDataInput, get_chart_data(), get_overseas_price(), get_price(), get_stock_list_stats(), OverseasOrderInput (+3 more)

### Community 154 - "HeaderMap"
Cohesion: 0.22
Nodes (11): risk_receipts_preserve_fixed_keys_payloads_and_restored_deduplication(), apply(), bounded_receipts_and_numeric_overflow_reject_without_partial_mutation(), rebuild_replays_durable_trades_then_missing_intent(), roundtrip(), Result, RiskManager, scope() (+3 more)

### Community 155 - ".new"
Cohesion: 0.12
Nodes (26): Cow, EventLoopProxy, Retained, HostEvent, main(), permitted_navigation(), record(), resource() (+18 more)

### Community 156 - "strategy-continuous-replay.spec.ts"
Cohesion: 0.04
Nodes (65): CandlePageView, ChartCandle, historyFixture(), mockApi(), MockOptions, mockResearchResult(), readyPreparation, strategy() (+57 more)

### Community 157 - "PriceResponse"
Cohesion: 0.12
Nodes (20): DailyHistoryJobs, JobEntry, retry_delay(), Arc, BTreeMap, DateTime, Drop, Duration (+12 more)

### Community 158 - "breakout.rs"
Cohesion: 0.23
Nodes (18): bar(), config(), context_refresh_failure_and_readiness_queries_never_reseed_cross_observations(), cross_state(), current_completed_daily_arrives_after_decision_and_duplicates_are_ignored(), disabled_filters_keep_golden_death_cross_quantity_and_metadata(), malformed_optional_does_not_replace_custom_legacy_periods_or_sell(), minute_seed_and_live_quotes_never_manufacture_completed_daily_context() (+10 more)

### Community 159 - "Strategy"
Cohesion: 0.29
Nodes (18): bounded_history_and_future_suffix_cannot_change_past_signals_or_state(), buy(), config(), crossing(), crossing_cancellation_requires_full_hysteresis_reset_before_reentry(), crossing_inclusive_entry_retries_after_equal_zero_sync_and_consumes_only_real_fill(), invalid_optional_fields_preserve_custom_legacy_exit_and_fail_closed_entries(), level_hysteresis_is_inert_but_range_is_validated_for_every_mode() (+10 more)

### Community 160 - "collect_pages"
Cohesion: 0.23
Nodes (19): CandlePageView, collect_pages(), F, bar(), collector_preserves_raw_volume_on_its_exact_completed_ohlc_row(), collects_252_actual_completed_highs_without_dropping_the_last_completed_bar(), empty_filtered_page_can_advance_and_full_512_live_window_is_bounded(), limits_requests_and_stops_on_repeat_or_no_progress() (+11 more)

### Community 161 - ".checkpoint_strategy_positions_shared"
Cohesion: 0.47
Nodes (6): OrderManager, Arc, Mutex, Result, Self, StrategyManager

### Community 162 - ".capture"
Cohesion: 0.38
Nodes (12): bar(), continuous(), mean_reversion_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal(), mean_reversion_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry(), mean_reversion_new_fields_are_strict_in_both_production_preview_paths(), mean_reversion_protective_age_counts_actual_held_observations_across_chunks(), mean_reversion_stop_priority_uses_real_fill_basis_and_next_open_execution(), params() (+4 more)

### Community 163 - ".collect"
Cohesion: 0.20
Nodes (13): PendingCandidate, OrderManager, ReconciliationSnapshot, Arc, BrokerId, KisRestClient, Mutex, Option (+5 more)

### Community 164 - "collect_daily_history"
Cohesion: 0.19
Nodes (17): accepted_http_response_publishes_before_its_event_under_the_same_gate(), balance(), delayed_http_a_to_b_to_a_response_never_publishes_or_emits(), delayed_http_response_cannot_undo_a_concurrent_buy_or_sell(), invalid_holdings_do_not_clear_the_previous_position_or_emit_success(), refresh(), Arc, AtomicUsize (+9 more)

### Community 165 - "toss/orders.rs"
Cohesion: 0.24
Nodes (7): every_read_observes_current_primary_and_structural_errors_are_not_empty_ledgers(), fill(), missing_primary_starts_empty_and_real_file_restart_keeps_exact_idempotent_fill(), primary_corruption_blocks_instead_of_recovering_an_old_valid_backup(), primary_loss_is_detected_after_restart_even_with_no_pending_orders_and_a_valid_backup(), primary_only_creation_crash_gap_anchors_the_existing_generation_and_ownership(), Fixture

### Community 167 - "Self"
Cohesion: 0.15
Nodes (9): get_active_toss_calendar_override(), toss_calendar_override(), toss_market_day_calendar(), toss_market_session_window(), calendar_override_represents_holiday_without_fallback(), calendar_session_window_checks_time_range(), MarketDayCalendar, MarketSessionWindow (+1 more)

### Community 168 - "String"
Cohesion: 0.05
Nodes (41): Builder, path_values_decode_without_changing_the_route_segment_contract(), production_json_query_and_path_rejections_stop_before_provider_or_mutation(), trailing_json_is_rejected_by_the_framework_before_order_validation(), production_kis_routes_preserve_capture_query_headers_and_response_mapping(), provider_http_failure_keeps_the_legacy_200_error_envelope(), toss_and_control_routes_keep_safe_errors_without_provider_or_engine_start(), head_and_options_keep_existing_origin_and_method_policy() (+33 more)

### Community 169 - "T"
Cohesion: 0.22
Nodes (22): atomic_write(), atomic_write_private_sync(), backup_failure_preserves_primary_and_cleans_staged_content(), durable_atomic_write(), durable_removal_clears_the_original_path(), durable_remove_file(), locked_destination_preserves_content_and_allows_retry_after_release(), private_sync_replacements_keep_owner_only_permissions() (+14 more)

### Community 170 - "Value"
Cohesion: 0.22
Nodes (12): collect_daily_history(), CollectedDailyHistory, completed_bar(), price(), Arc, ChartCandle, KisRestClient, Option (+4 more)

### Community 171 - "Vec"
Cohesion: 0.21
Nodes (16): category_from_key(), database_scanner_preserves_original_payload_order_and_checksum_input(), document_key(), LocalDocument, quarantine_corrupt_managed_json(), read_managed_json(), read_managed_json_backup(), Option (+8 more)

### Community 172 - "Arc"
Cohesion: 0.17
Nodes (21): TossApiResponse, TossCandle, TossCandlePageResponse, TossCost, TossCurrencyAmount, TossDailyProfitLoss, TossHoldingsItem, TossHoldingsOverview (+13 more)

### Community 173 - "Client"
Cohesion: 0.22
Nodes (9): BalanceResult, domestic, first, focusRing(), openWithKeyboard(), overseas, qty(), second (+1 more)

### Community 174 - "DateTime"
Cohesion: 0.13
Nodes (19): RecoveryIdentity, useApproveBudgetRecovery(), useBudgetRecovery(), amount(), BudgetEditor(), BudgetRecoverySection(), money(), RecoveryCard() (+11 more)

### Community 175 - "Mutex"
Cohesion: 0.38
Nodes (12): candle(), config(), daily_hooks_do_not_create_unknown_symbol_states(), Value, strong_close_completed_day_uses_actual_position_and_keeps_stop_loss(), strong_close_uses_each_completed_day_only_on_the_following_day(), volatility(), volatility_collects_missing_history_after_close_for_later_days() (+4 more)

### Community 176 - "Option"
Cohesion: 0.26
Nodes (8): gate(), keyboardObservationWindow(), KeyboardSearchFixture, keyboardStock, safeKeyboardSearch(), setup(), states, Surface

### Community 177 - "Result"
Cohesion: 0.21
Nodes (3): StockEntry, StockListStats, StockStore

### Community 181 - "AppHandle"
Cohesion: 0.18
Nodes (9): args, artifactDir, artifacts, checksumPath, manifestPath, names, records, requireSignatures (+1 more)

### Community 182 - "preview_strategy_from_candles"
Cohesion: 0.30
Nodes (17): preview_strategy_from_candles(), all_strategies_keep_the_same_evaluation_window_without_implicit_warmup(), complete_daily_history_initializes_supported_strategies_without_shortening_defaults(), cross_and_breakout_seed_only_final_bars_are_not_conditions_not_met(), first_ready_bar_is_observed_after_tick_without_discarding_evaluation_bars(), generic_lth_is_rejected_before_live_clock_or_candle_preparation(), input(), insufficient_history_is_not_reported_as_a_zero_trade_strategy() (+9 more)

### Community 183 - ".submit_order_shared"
Cohesion: 0.21
Nodes (12): certificates(), command(), containers, directory, environment, interruption, results, root (+4 more)

### Community 184 - "AccountProfile"
Cohesion: 0.26
Nodes (9): ExecutionRun, ExecutionRunContext, OrderManager, Arc, KisRestClient, Option, Result, String (+1 more)

### Community 190 - "checked_budget_scope"
Cohesion: 0.15
Nodes (24): AutoTradingBudgetInput, BudgetRecoveryView, approve_budget_recovery(), AutoTradingBudgetInput, budget_error(), checked_budget_scope(), get_auto_trading_budget(), get_budget_recovery() (+16 more)

### Community 192 - "failed_breakout_entry_tests.rs"
Cohesion: 0.25
Nodes (17): bar(), failed_breakout_candidate_high_low_volume_cannot_influence_its_own_entry(), failed_breakout_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average(), failed_breakout_entry_atr_keeps_common_sizing_and_sells_actual_adjusted_quantity(), failed_breakout_hash_distinguishes_unknown_daily_volume_from_verified_zero(), failed_breakout_next_open_promotes_original_anchor_after_same_zero_feedback(), failed_breakout_optional_params_are_strict_in_both_previews_and_daily_filters_reject_minutes(), failed_breakout_partition_and_future_suffix_are_causal_for_all_options() (+9 more)

### Community 193 - "EntryContext"
Cohesion: 0.19
Nodes (6): ConsecutiveMoveParams, EntryContext, Counters, EntryFilterPreparation, Option, VecDeque

### Community 194 - "commands/risk.rs"
Cohesion: 0.36
Nodes (12): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+4 more)

### Community 195 - "ConfigurationEditReservation"
Cohesion: 0.09
Nodes (21): EditResult, ConfigurationEditReservation, effective_target_markets(), HashMap, HashSet, Into, Self, String (+13 more)

### Community 196 - "MarketDayCalendar"
Cohesion: 0.50
Nodes (5): HoldingsUpdate, money(), quantity(), Result, Self

### Community 197 - "DeviationStrategy"
Cohesion: 0.10
Nodes (11): DeviationStrategy, HashMap, Option, Result, Self, String, Value, capacity() (+3 more)

### Community 198 - "toss/orders.rs"
Cohesion: 0.12
Nodes (19): lth_preview_assessment(), as_of(), assessment_uses_prepared_lexical_tail_and_distinguishes_price_exclusions(), candle(), daily_omits_timing_and_empty_minute_is_not_evaluable(), legacy_selection_stays_at_200_and_does_not_refill_rejected_price_rows(), DateTime, Utc (+11 more)

### Community 199 - "Self"
Cohesion: 0.40
Nodes (3): LeveragedTrendHoldStrategy, Option, String

### Community 200 - "consecutive/tests.rs"
Cohesion: 0.25
Nodes (18): atr_includes_gaps_but_excludes_candidate_ohlc_and_completion_is_once(), bar(), combined_threshold_rejections_are_independent_and_require_all_filters(), completed_daily_context_remains_bounded_at_512_and_has_no_current_price_duplicates(), config(), entry_atr_retains_fractional_true_range_average_and_inclusive_threshold(), huge_legacy_move_counts_do_not_overflow_or_grow_unbounded_buffers(), invalid_optional_fields_fail_closed_without_replacing_custom_held_exit_parameters() (+10 more)

### Community 201 - "holding"
Cohesion: 0.32
Nodes (6): OrderManager, PendingOrder, ReservationDecision, Arc, submission_is_stale(), SubmissionReservation

### Community 202 - "Arc"
Cohesion: 0.22
Nodes (5): layerRank, layers, root, srcRoot, violations

### Community 203 - "BalanceItem"
Cohesion: 0.22
Nodes (6): appEntry, escapedName, npmLock, pkg, rootDir, workspaceLock

### Community 204 - "ui/OverseasStockChart.tsx"
Cohesion: 0.26
Nodes (12): card(), chooseMode(), continuous(), labels, legacyParams, Params, policyParams, response() (+4 more)

### Community 205 - "config"
Cohesion: 0.53
Nodes (5): config(), empty_price_condition_entries_remove_stale_targets_before_manager_validation(), reconfigure_price_condition_canonicalizes_targets_and_keeps_actual_stop(), reconfigure_price_condition_does_not_move_holdings_to_replacement_target(), Value

### Community 206 - "input"
Cohesion: 0.29
Nodes (11): blocked_strong_close_buy_does_not_discard_next_days_completed_condition(), candle(), daily_event_strategies_reject_other_cadences(), flat_completed_days_fill_history_without_compressing_the_window(), input(), ChartCandle, StrategyPreviewInput, Value (+3 more)

### Community 207 - ".prepare_confirmed_ownership"
Cohesion: 0.33
Nodes (6): OrderManager, PreparedOwnershipFill, realized_pnl(), Option, PendingOrder, Result

### Community 208 - "TossTokenState"
Cohesion: 0.40
Nodes (7): OrderManager, Arc, Mutex, Option, P, Result, Self

### Community 209 - ".preview_signals_with_execution"
Cohesion: 0.19
Nodes (25): bar(), canonical(), economics(), input(), near(), partial_bars(), partition(), pc_configured_sell_request_is_preserved_when_actual_fill_is_smaller() (+17 more)

### Community 210 - "strong_close/tests.rs"
Cohesion: 0.25
Nodes (18): bar(), fill(), Value, seed(), strategy(), strong_close_actual_quantity_average_partial_stop_zero_and_retry(), strong_close_age_unknown_disabled_partial_and_protection_priority(), strong_close_dated_same_day_failure_late_legacy_and_new_day() (+10 more)

### Community 212 - ".new"
Cohesion: 0.39
Nodes (4): first_positive_u64(), KisOutput1<T>, OverseasExecutedOrder, parse_decimal_cents()

### Community 213 - "HashMap"
Cohesion: 0.25
Nodes (6): endpointInventory, EXPECTED_PATHS, hasRateLimitHeaders, missingPaths, paths, servers

### Community 214 - "HeaderMap"
Cohesion: 0.18
Nodes (10): node, vite.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck (+2 more)

### Community 215 - "broker/candles.rs"
Cohesion: 0.50
Nodes (3): DatabaseManager, Path, Self

### Community 217 - "작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단"
Cohesion: 0.26
Nodes (11): card(), continuous(), labels, legacyParams, Params, policyParams, response(), runButton() (+3 more)

### Community 218 - "budget/recovery/tests.rs"
Cohesion: 0.20
Nodes (5): entries(), repeatReads(), retry(), status(), traceRow

### Community 219 - "commands.rs"
Cohesion: 0.24
Nodes (9): config(), FiftyTwoWeekHighParams, FiftyTwoWeekState, Default, Self, Value, VecDeque, strong_close_ready_is_distinct_from_condition_and_preserves_positions() (+1 more)

### Community 220 - "PendingOrderStore"
Cohesion: 0.24
Nodes (9): MeanReversionExitMode, MeanReversionParams, parse(), Default, Result, Self, String, Value (+1 more)

### Community 221 - "BrokerHoldingView"
Cohesion: 0.38
Nodes (10): known_fill(), Value, strategy(), trend_filter_default_strict_current_inclusive_and_overflow_contract(), trend_filter_optional_protection_before_readiness_uses_native_avg_and_observed_peak(), trend_filter_pending_rejected_partial_and_same_quantity_keep_actual_evidence(), trend_filter_raw_validation_and_corrupt_runtime_preserve_independent_periods(), trend_filter_reinitialize_core_change_reset_and_optional_change_have_distinct_lifecycles() (+2 more)

### Community 222 - "BrokerAdapterResult"
Cohesion: 0.33
Nodes (9): D, bounded_metadata(), bounded_positions(), deserialize_generation(), explicit_optional_profile(), Error, Self, Vec (+1 more)

### Community 223 - "BrokerCurrency"
Cohesion: 0.25
Nodes (8): finish_closed_batch(), market_readiness(), MarketReadiness, FnOnce, Option, StrategyManager, String, Vec

### Community 225 - "Default"
Cohesion: 0.29
Nodes (3): MeanReversionState, Option, VecDeque

### Community 226 - "submission.rs"
Cohesion: 0.27
Nodes (10): delete_database_password(), entry(), fail_next_mock_operation_for_tests(), keychain_test_lock(), load_database_password(), MutexGuard, Option, Result (+2 more)

### Community 227 - "strong_close_history_tests.rs"
Cohesion: 0.20
Nodes (7): currency, destination(), returnToContents(), risk, sections, setup(), toc()

### Community 228 - "Result"
Cohesion: 0.25
Nodes (7): KisBrokerAdapter, maps_kis_balance_item_to_broker_holding(), Arc, BalanceItem, BrokerId, KisRestClient, Self

### Community 230 - "String"
Cohesion: 0.44
Nodes (9): hysteresisError(), hysteresisOf(), isMomentumEntryParam(), modeOf(), MomentumEntryControls(), momentumEntryParamError(), MomentumEntrySummary(), parsedDraft() (+1 more)

### Community 231 - "Vec"
Cohesion: 0.18
Nodes (10): LiveDailyHistoryStatus, TradingStatus, HistoryRow(), LiveDailyHistoryPanel(), Props, retryTime(), STATE_LABELS, STATE_PRIORITY (+2 more)

### Community 232 - "Arc"
Cohesion: 0.44
Nodes (10): delayed_open_fill_reaches_lth_before_close_and_last_exit_remains_unfilled(), future_bar_does_not_change_prior_signal(), history(), params(), Value, Vec, run(), session_and_blackout_use_input_minutes() (+2 more)

### Community 233 - "BrokerCurrency"
Cohesion: 0.22
Nodes (9): expected, krw, krwSecond, readAllDetails(), records, search(), trigger(), usd (+1 more)

### Community 234 - "Client"
Cohesion: 0.60
Nodes (4): config(), live_feedback_mean_reversion_keeps_entry_and_tick_history_on_sell(), live_feedback_trend_filter_keeps_confirmation_and_updates_indicators(), Value

### Community 235 - "Into"
Cohesion: 0.42
Nodes (9): atr_uses_gap_true_ranges_fourteen_denominator_and_prior_close(), bandwidth_uses_twenty_population_samples_and_inclusive_boundary(), bar(), large_native_prices_do_not_overflow_daily_statistics_and_invalid_ohlc_fail_closed(), p(), repeated_rejection_reasons_overlap_without_inflating_candidate_count(), Value, strict_trend_and_inclusive_slope_use_prior_daily_values() (+1 more)

### Community 236 - "Option"
Cohesion: 0.12
Nodes (8): mockUx(), UxHandler, UxResponse, setup(), setup(), mockToss(), setup(), setup()

### Community 237 - "Result"
Cohesion: 0.21
Nodes (9): blocked_buy_is_fed_back_before_the_next_raw_signal(), candle(), daily_diagnostic_contains_one_completed_bar_and_no_synthetic_entry_observation(), daily_warmup_preview_uses_closes_and_excludes_future_bars(), future_candle_change_does_not_change_prior_replay_signals(), live_tick_and_preview_emit_identical_signals_for_normalized_fixture(), minute_replay_warmup_excludes_replay_day_and_future_daily_candles(), reproduction_hash_covers_ohlcv_and_order_quantity() (+1 more)

### Community 239 - "String"
Cohesion: 0.22
Nodes (15): config(), reconfigure_fifty_two_week_keeps_actual_entry_while_invalidating_old_window(), reconfigure_strong_close_preserves_stop_but_discards_old_entry_candidate(), reconfigure_volatility_recomputes_range_without_losing_held_stop_or_day(), reject_scope_changes(), Value, config(), reconfigure_deviation_applies_new_threshold_to_retained_observations() (+7 more)

### Community 245 - "StatusCode"
Cohesion: 0.15
Nodes (7): zustand, AccountState, useAccountStore, TradingState, TradingStatus, useTradingStore, zustand

### Community 262 - "config"
Cohesion: 0.48
Nodes (6): config(), live_feedback_breakout_checkpoint_restores_pending_and_confirmed_exact_anchor(), live_feedback_breakout_freezes_pending_signal_anchor_until_first_fill(), live_feedback_consecutive_keeps_confirmed_flag_and_observations(), live_feedback_unsubmitted_breakout_discards_only_candidate_not_history(), Value

### Community 263 - "failedBreakoutControls.tsx"
Cohesion: 0.23
Nodes (10): Deserialize, fetch_usd_krw_rate(), Result, encode(), Protection, restore(), Option, Result (+2 more)

### Community 265 - "bounded_optional_integer"
Cohesion: 0.22
Nodes (21): list_toss_open_orders(), list_toss_open_orders_for_profile(), modify_toss_order(), modify_toss_order_for_profile(), require_toss_order_profile(), Arc, BrokerId, CmdResult (+13 more)

### Community 266 - "EntryContext"
Cohesion: 0.12
Nodes (9): EntryContext, VecDeque, valid_ohlc(), CompletedDailyBar, EntryContext, EntryFilterPreparation, Option, Result (+1 more)

### Community 267 - "config"
Cohesion: 0.53
Nodes (5): config(), live_feedback_fifty_two_week_keeps_crossing_progress_without_optimistic_holdings(), live_feedback_strong_close_consumes_candidate_only_after_buy_fill(), live_feedback_volatility_keeps_intraday_extremes_and_confirmed_position(), Value

### Community 269 - "T"
Cohesion: 0.33
Nodes (8): encode(), Protection, restore(), Option, Result, String, Value, VolatilityExpansionState

### Community 274 - "read_json_or_default"
Cohesion: 0.13
Nodes (13): BalanceSnapshot, BalanceStore, HoldingItem, build_daily_path(), build_monthly_path(), concurrent_writes_leave_one_complete_json_document(), corrupt_json_recovers_from_last_durable_backup(), daily_path_uses_date_components_without_string_separators() (+5 more)

### Community 276 - "ma_cross_entry_tests.rs"
Cohesion: 0.29
Nodes (14): bar(), candidate_current_range_cannot_change_its_own_prior_atr_filter(), flat(), guard_rejection_does_not_recreate_a_consumed_golden_cross(), integer_float_daily_period_stays_active_and_missing_history_blocks_only_golden_cross(), ma_cross_options_are_strict_in_short_and_continuous_preview_paths(), non_daily_filters_reject_explicitly_and_disabled_minute_compatibility_remains(), request() (+6 more)

### Community 277 - "strategy_preview/tests.rs"
Cohesion: 0.54
Nodes (5): BudgetCurrencyView, BudgetReservation, CurrencyLedger, Option, String

### Community 278 - "collect_daily_history"
Cohesion: 0.43
Nodes (7): config(), deviation_counts_current_price_as_last_required_observation(), deviation_history_is_ready_before_first_evaluation_and_bounded(), momentum_counts_first_comparison_but_excludes_seed_prices(), momentum_history_does_not_create_or_erase_order_state(), Value, rsi_initializer_restores_crossing_baseline_without_historical_signals()

### Community 290 - "fills.rs"
Cohesion: 0.17
Nodes (7): calculate_domestic_fee(), calculate_overseas_fee_cents(), cumulative_fill_delta(), OrderManager, NaiveDate, Option, Result

### Community 291 - "trading/ui/Page.tsx"
Cohesion: 0.19
Nodes (11): EntryFilterPreparation, number(), parse(), Default, Option, Result, Self, String (+3 more)

### Community 292 - "SubmissionDeps"
Cohesion: 0.40
Nodes (3): Result, String, Value

### Community 293 - "scaled_total"
Cohesion: 0.31
Nodes (9): execution_quantity(), OrderManager, parts(), BrokerCurrency, NaiveDate, Option, Result, scaled_total() (+1 more)

### Community 294 - "guard/recovery/tests.rs"
Cohesion: 0.32
Nodes (11): buy(), evaluate(), live_recovery_uses_local_calendar_and_bounds_identity_receipts(), recovered_native_stop_preserves_original_day_and_next_day_sell_cooldown(), recovering_older_order_preserves_newer_last_side_date_and_recent_ordering(), recovery_retry_does_not_extend_time_or_duplicate_whipsaw_history(), DateTime, FixedOffset (+3 more)

### Community 295 - "StrongCloseState"
Cohesion: 0.29
Nodes (5): EntryContext, Result, String, Value, StrongCloseState

### Community 296 - "run_prepared_preview"
Cohesion: 0.33
Nodes (6): CandleDataSummary(), CandlePoint, focusStyle, number(), Projection, Props

### Community 297 - "trendFilterControls.tsx"
Cohesion: 0.22
Nodes (10): OverseasBalanceResult, assertReadOnly(), assertTossGates(), domestic, empty, failure, markets, overseas (+2 more)

### Community 298 - "KisBrokerAdapter"
Cohesion: 0.17
Nodes (16): Into, Self, body_snippet(), read_response_text(), read_toss_candle_response_text(), read_toss_response_text(), Client, Option (+8 more)

### Community 320 - "ChartCandle"
Cohesion: 0.12
Nodes (14): args, digest(), files, generated, hashes, index, manifest, metadata (+6 more)

### Community 322 - "MaCrossParams"
Cohesion: 0.21
Nodes (12): active_slope_bounds_legacy_long_period_and_required_context(), finite(), legacy_params(), MaCrossParams, Default, Option, Result, Self (+4 more)

### Community 323 - "strategy_preview/trend_filter_tests.rs"
Cohesion: 0.29
Nodes (13): bar(), request(), ContinuousStrategyPreviewView, StrategyPreviewInput, String, Value, run(), short() (+5 more)

### Community 324 - "ContinuousStrategyPreviewInput"
Cohesion: 0.18
Nodes (15): ChunkContinuityView, ContinuousStrategyPreviewChunk, ContinuousStrategyPreviewInput, ContinuousStrategyPreviewView, invalid(), ReplayContinuityView, BrokerId, ChartCandle (+7 more)

### Community 326 - "input"
Cohesion: 0.27
Nodes (10): bar(), changed_candidate_ohlc_range_cannot_change_its_own_prior_atr_entry_filter(), default_diagnostic_is_omitted_while_threshold_and_preparation_failures_are_distinct(), frozen_spy_default_cost_paths_preserve_predeclared_exit_dates(), input(), integer_valued_json_float_periods_remain_enabled_in_production_preview(), invalid_filter_params_and_non_daily_ohlc_filter_requests_are_explicit_errors(), ChartCandle (+2 more)

### Community 327 - "mean_reversion_entry_tests.rs"
Cohesion: 0.38
Nodes (12): bar(), continuous(), deviation_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal(), deviation_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry(), deviation_new_fields_are_strict_in_both_production_preview_paths(), deviation_protective_age_counts_actual_held_observations_across_chunks(), deviation_stop_priority_uses_real_fill_basis_and_next_open_execution(), params() (+4 more)

### Community 328 - ".record_fill_pnl_once"
Cohesion: 0.33
Nodes (3): RiskManager, NaiveDate, Result

### Community 345 - "lth_preview_assessment"
Cohesion: 0.25
Nodes (8): Result, String, Value, validate_entry_params(), Result, String, Value, validate_consecutive_entry_params()

### Community 346 - "OrderSubmission"
Cohesion: 0.14
Nodes (23): OrderSubmission, prepare_ownership(), Option, OrderManager, Result, build_provider_request(), prepare_buy_submission(), prepare_order_submission() (+15 more)

### Community 347 - "volatility_manager"
Cohesion: 0.28
Nodes (14): bar(), request(), ContinuousStrategyPreviewView, StrategyPreviewInput, String, Value, run(), short() (+6 more)

### Community 348 - "submission.rs"
Cohesion: 0.43
Nodes (3): order_side_label(), OrderSide, submitting_conflict_reason()

### Community 349 - "strategy-trend-filter-policy.spec.ts"
Cohesion: 0.22
Nodes (13): card(), continuous(), coreLabels, labels, legacyParams, Params, policyParams, response() (+5 more)

### Community 350 - "mean_reversion/tests.rs"
Cohesion: 0.28
Nodes (22): age_starts_after_positive_fill_and_survives_same_partial_and_warmup_feedback(), buy(), config(), current_inclusive_population_bands_and_strict_boundaries_are_preserved(), disabled_unknown_symbol_and_reset_preserve_observation_contract(), exact_metadata_roundtrips_pending_and_confirmed_age_without_inventing_ownership(), filled(), hold() (+14 more)

### Community 351 - "strategy-volatility-expansion-policy.spec.ts"
Cohesion: 0.20
Nodes (14): card(), continuous(), coreLabels, labels, legacyParams, Params, policyParams, ready (+6 more)

### Community 352 - "FailedBreakoutState"
Cohesion: 0.24
Nodes (11): FailedBreakoutState, encode(), restore(), Option, Result, String, Value, V1 (+3 more)

### Community 353 - "FailedBreakoutParams"
Cohesion: 0.21
Nodes (11): FailedBreakoutBasis, FailedBreakoutParams, number(), parse(), Default, Option, Result, Self (+3 more)

### Community 354 - "PreparedOrderSubmission"
Cohesion: 0.21
Nodes (13): PreparedOrderSubmission, append_failed_order(), build_kis_pending_order(), build_pending_order(), build_toss_pending_order(), failed_order_price(), Arc, Option (+5 more)

### Community 355 - "config"
Cohesion: 0.67
Nodes (5): config(), reconfigure_mean_reversion_keeps_stop_active_before_larger_window_is_ready(), reconfigure_trend_filter_retains_owned_quantity_and_uses_new_period_for_exit(), reject_scope_changes(), Value

### Community 356 - "config"
Cohesion: 0.67
Nodes (5): config(), reconfigure_consecutive_preserves_holding_and_sell_observations(), reconfigure_failed_breakout_keeps_exact_reference_when_new_history_is_short(), reject_scope_changes(), Value

### Community 358 - "momentum_entry_tests.rs"
Cohesion: 0.40
Nodes (10): bar(), continuous(), momentum_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal(), momentum_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry(), momentum_new_fields_are_strict_in_both_production_preview_paths(), params(), request(), ContinuousStrategyPreviewView (+2 more)

### Community 359 - ".capture"
Cohesion: 0.26
Nodes (14): basisOf(), BreakoutNumericField(), FailedBreakoutControls(), failedBreakoutDailyFiltersDisabled(), failedBreakoutPolicyError(), FailedBreakoutPolicySummary(), fieldError(), fields (+6 more)

### Community 360 - "config/mod.rs"
Cohesion: 0.47
Nodes (3): load_secure_config(), non_empty(), SecureConfig

### Community 361 - "strategy-failed-breakout-policy.spec.ts"
Cohesion: 0.23
Nodes (13): basis(), card(), continuous(), labels, legacyParams, Params, policyParams, ready (+5 more)

### Community 362 - "StrongCloseParams"
Cohesion: 0.50
Nodes (3): LthMinuteTiming, card(), preview()

### Community 363 - "StrategyConfig"
Cohesion: 0.13
Nodes (13): validates_order_create_request_shape(), TossOrderCreateRequest, market_from_currency(), new_toss_client_order_id(), toss_currency(), toss_market(), validate_client_order_id(), validate_optional_decimal() (+5 more)

### Community 364 - "attach_comparison"
Cohesion: 0.50
Nodes (4): toss_error_display_preserves_structured_and_fallback_literals(), toss_typed_rejections_keep_the_existing_status_boundary(), format_toss_error(), HeaderMap

### Community 365 - "current_position_snapshot"
Cohesion: 0.32
Nodes (7): current_position_snapshot(), refresh_manual_positions(), Arc, Mutex, Option, OrderManager, Result

### Community 366 - "ReplayAssessmentView"
Cohesion: 0.42
Nodes (8): config(), live_feedback_deviation_manager_observations_match_replay_and_ownership_requires_feedback(), live_feedback_manager_pending_and_unsettled_suppress_only_outward_signal_across_restart(), live_feedback_manager_protection_metadata_requires_exact_owner_and_valid_checkpoint(), live_feedback_manager_targets_exact_owner_scope_and_market(), live_feedback_mean_reversion_manager_observations_match_replay_and_ownership_requires_feedback(), live_feedback_rsi_unfilled_and_ma_share_exact_replay_observations(), scope()

### Community 367 - "place_prepared_order"
Cohesion: 0.50
Nodes (7): place_overseas_with_retry_shared(), place_prepared_order(), place_with_retry_shared(), Arc, KisRestClient, OrderResponse, Result

### Community 368 - "finish_closed_batch"
Cohesion: 0.41
Nodes (11): list_toss_accounts(), list_toss_profile_accounts(), lookup_toss_accounts_with_credentials(), mask_toss_account_no(), CmdResult, State, String, Vec (+3 more)

### Community 369 - ".submit_order_shared"
Cohesion: 0.14
Nodes (12): B, BrokerCurrency, Client, Option, Result, String, T, Vec (+4 more)

### Community 370 - ".restore_recovered_submission_at"
Cohesion: 0.83
Nodes (3): confirmed_small_fill_requires_original_order_symbol_side_currency_and_whole_share(), detail(), missing_amount_or_zero_fill_is_deferred_without_using_limit_or_average_as_money()

### Community 372 - "mockApi"
Cohesion: 0.27
Nodes (12): check_toss_profile_connection(), BrokerId, CmdResult, Into, Option, State, String, Vec (+4 more)

### Community 373 - "strategy-strong-close-policy.spec.ts"
Cohesion: 0.22
Nodes (13): card(), continuous(), labels, legacyParams, Params, policyParams, ready, response() (+5 more)

### Community 374 - "TradeStore"
Cohesion: 0.38
Nodes (7): holding(), BrokerMarket, scope(), toss_fractional_and_zero_quantities_never_create_a_whole_share(), toss_rejects_invalid_quantities_and_prices_including_float_u64_boundary(), toss_rejects_wrong_broker_account_currency_and_duplicate_market_symbol(), toss_us_prices_round_to_cents_and_kr_prices_round_to_won()

### Community 375 - "managed_paths"
Cohesion: 0.47
Nodes (10): check_size(), history_key(), local_paths(), managed_paths(), records(), Path, PathBuf, Result (+2 more)

### Community 376 - "candles/tests.rs"
Cohesion: 0.25
Nodes (8): ChunkContinuityView, CmdResult, PreviewRow, StrategyPreviewInput, StrategyPreviewView, String, Vec, run_prepared_preview()

### Community 377 - ".validate_recovery_history"
Cohesion: 0.28
Nodes (7): OrderManager, own_fill(), possible_scope(), BrokerId, Option, PendingOrder, Result

### Community 378 - "report_json"
Cohesion: 0.67
Nodes (3): report_json(), StrategyPreviewView, Value

### Community 379 - "database/tests.rs"
Cohesion: 0.19
Nodes (9): inventory_from_documents(), JsonStorageInventoryView, inventory_groups_documents_by_top_level_category(), keychain_test_dirs(), legacy_file_password_migrates_to_keychain_on_load(), PathBuf, SaveDatabaseConfigInput, save_config_stores_password_in_keychain_not_in_file() (+1 more)

### Community 380 - "initialize_strategy_warmup_with_daily_bars"
Cohesion: 0.48
Nodes (6): config(), consecutive_counts_current_price_and_uses_active_position_branch(), consecutive_history_preserves_flat_and_existing_positions(), failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation(), failed_breakout_history_retains_actual_breakout_reference(), Value

### Community 384 - "install_database_manager"
Cohesion: 0.05
Nodes (72): AsyncRead, AsyncWrite, Context, Pin, Poll, ReadBuf, Receiver, connect_stream() (+64 more)

### Community 385 - "integer"
Cohesion: 0.14
Nodes (19): parse(), Default, Result, Self, String, Value, RsiParams, validate_rsi_params() (+11 more)

### Community 401 - "volatility_entry_tests.rs"
Cohesion: 0.27
Nodes (15): bar(), request(), ContinuousStrategyPreviewView, StrategyPreviewInput, String, Value, run(), short() (+7 more)

### Community 402 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.23
Nodes (13): chartTimeToMs(), formatChartTimeKst(), formatKstMs(), pad2(), parseTimeMs(), Props, StrategyPreviewChart(), StrategyPreviewChartSignal (+5 more)

### Community 403 - "AtrWindow"
Cohesion: 0.24
Nodes (9): AtrPreparationView, AtrWindow, bar(), downward_gap_on_flat_candle_is_not_a_zero_range(), full_previous_close_history_is_required_and_old_ranges_expire(), large_prices_do_not_overflow_the_range_sum(), Option, String (+1 more)

### Community 404 - "VolatilityExpansionState"
Cohesion: 0.21
Nodes (5): VolatilityExpansionPreparation, VolatilityExpansionState, Default, Self, VolatilityExpansionParams

### Community 408 - "strongCloseControls.tsx"
Cohesion: 0.13
Nodes (10): execution_fingerprint_fixed_utf8_prefixes_and_array_output(), execution_fingerprint_preserves_the_existing_field_boundary(), profile(), profile_fingerprint(), KisRestClient, Option, Self, super::OwnershipIdentity (+2 more)

### Community 409 - "TossTokenState"
Cohesion: 0.29
Nodes (3): Diagnostics, Default, Self

### Community 410 - "notional/tests.rs"
Cohesion: 0.10
Nodes (10): Option, toss(), TossOrder, TossOrderExecution, TossOrderListQuery, TossOrderListStatus, TossOrderModifyRequest, TossOrderOperationResponse (+2 more)

### Community 411 - "database_archive.rs"
Cohesion: 0.43
Nodes (5): DatabaseArchiveStats, purge(), stats(), trade_date(), trade_rows()

### Community 419 - "momentumEntryControls.tsx"
Cohesion: 0.36
Nodes (4): PhaseDiagnosticsView, PhaseObservation, ReplaySession, Snapshot

### Community 422 - "restore"
Cohesion: 0.31
Nodes (7): encode(), Protection, restore(), Option, Result, String, Value

### Community 705 - "high_tests.rs"
Cohesion: 0.36
Nodes (12): bar(), complete_day(), completed_high_not_close_sets_next_reference_without_lookahead(), config(), fifty_two_week_requires_full_window_and_uses_last_completed_high_and_close(), initialization_keeps_last_window_and_daily_bookkeeping_is_symbol_scoped(), many_intraday_prices_and_intraday_initializers_do_not_age_daily_highs(), missing_history_becomes_ready_only_after_252_completed_daily_highs() (+4 more)

### Community 707 - "package.json"
Cohesion: 0.25
Nodes (7): engines, node, npm, name, private, type, version

## Knowledge Gaps
- **436 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+431 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **214 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `StrategyConfig` connect `leveragedTrendHoldEditorPanel.tsx` to `DailyHistoryTicket`, `FiftyTwoWeekHighStrategy`, `StrongCloseStrategy`, `TrendFilterStrategy`, `config`, `config`, `deviation/tests.rs`, `rsi/tests.rs`, `collect_daily_history`, `StrategyManager`, `RunFence`, `breakout.rs`, `Strategy`, `RsiStrategy`, `DatabaseManager`, `MomentumStrategy`, `PriceConditionStrategy`, `Mutex`, `FailedBreakoutStrategy`, `VolatilityExpansionStrategy`, `high_tests.rs`, `ConfigurationEditReservation`, `build_strategy`, `DeviationStrategy`, `consecutive/tests.rs`, `read_json_or_default`, `config`, `ConsecutiveMoveStrategy`, `BrokerQuantity`, `MovingAverageCrossStrategy`, `commands.rs`, `mean_reversion/tests.rs`, `MeanReversionStrategy`, `config`, `config`, `Client`, `LeveragedTrendHoldStrategy`, `ReplayAssessmentView`, `String`, `volatility_expansion/tests.rs`, `initialize_strategy_warmup_with_daily_bars`, `failed_breakout/tests.rs`, `manager`, `strategy-configure/index.ts`?**
  _High betweenness centrality (0.101) - this node is a cross-community bridge._
- **Why does `BrokerScope` connect `BrokerScope` to `conflicts.rs`, `DailyHistoryTicket`, `TradeRecord`, `.process_buy`, `modify_toss_order_for_profile`, `market_hours.rs`, `BrokerAccountId`, `restore_risk_from_today_trades`, `Fixture`, `strongCloseControls.tsx`, `HeaderMap`, `StrategyManager`, `tauri.conf.json`, `prepare`, `.collect`, `.submit_order_shared`, `databaseManagementSection.tsx`, `DatabaseManager`, `ReplaySession`, `security.rs`, `AccountProfile`, `order/budget.rs`, `OrderManager`, `OrderRecord`, `checked_budget_scope`, `ConfigurationEditReservation`, `build_strategy`, `MarketDayCalendar`, `.record_fill_pnl_once`, `holding`, `start_trading`, `HoldingsQuery`, `OrderSubmission`, `submission.rs`, `TossAccessToken`, `ReplayAssessmentView`, `Signal`, `TradeStore`, `.validate_recovery_history`, `strategy-configure/index.ts`?**
  _High betweenness centrality (0.076) - this node is a cross-community bridge._
- **Why does `Signal` connect `Signal` to `MarketDayCalendar`, `run_backtest_with_history`, `FiftyTwoWeekHighStrategy`, `TrendFilterStrategy`, `leveragedTrendHoldEditorPanel.tsx`, `deviation/tests.rs`, `observation.rs`, `AtomicBool`, `StrategyManager`, `RunFence`, `Strategy`, `RsiStrategy`, `.submit_order_shared`, `guard/recovery/tests.rs`, `databaseManagementSection.tsx`, `MomentumStrategy`, `ReplaySession`, `simulation.rs`, `OrderManager`, `run_backtest`, `holding`, `ConsecutiveMoveStrategy`, `BrokerQuantity`, `MovingAverageCrossStrategy`, `OrderSubmission`, `mean_reversion/tests.rs`, `MeanReversionStrategy`, `LeveragedTrendHoldStrategy`, `generic.rs`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _436 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `shared/api/commands.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.04443309499489275 - nodes in this community are weakly interconnected._
- **Should `rest/types.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12121212121212122 - nodes in this community are weakly interconnected._
- **Should `RiskManager` be split into smaller, more focused modules?**
  _Cohesion score 0.03829160530191458 - nodes in this community are weakly interconnected._