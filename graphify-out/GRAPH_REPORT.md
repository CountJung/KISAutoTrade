# Graph Report - KISAutoTrade  (2026-10-18)

## Corpus Check
- 336 files · ~245,295 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 5310 nodes · 11570 edges · 508 communities (222 shown, 286 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 546 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `1192673f`
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
- detail
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
- config/mod.rs
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
6. `Signal` - 64 edges
7. `AccountProfile` - 51 edges
8. `OhlcCandle` - 48 edges
9. `RiskManager` - 42 edges
10. `StrategyManager` - 42 edges

## Surprising Connections (you probably didn't know these)
- `maps_holding_to_broker_domain()` --calls--> `BrokerAccountId`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `format_toss_error()` --calls--> `body_snippet()`  [INFERRED]
  src-tauri/src/broker/toss/error.rs → src-tauri/src/broker/toss/http.rs
- `deterministic_replay_applies_costs_and_separates_signals_from_fills()` --calls--> `run_backtest()`  [INFERRED]
  src-tauri/src/trading/simulation/tests.rs → src-tauri/src/trading/simulation.rs
- `identical_fixture_has_identical_report_and_hash()` --calls--> `run_backtest()`  [INFERRED]
  src-tauri/src/trading/simulation/tests.rs → src-tauri/src/trading/simulation.rs
- `sell_guard_uses_the_same_round_trip_cost_assumptions_as_settlement()` --calls--> `run_backtest()`  [INFERRED]
  src-tauri/src/trading/simulation/tests.rs → src-tauri/src/trading/simulation.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/trading/simulation/assessment.rs -> src-tauri/src/trading/simulation/assessment.rs`
- 2-file cycle: `src-tauri/src/trading/simulation.rs -> src-tauri/src/trading/simulation/atr.rs -> src-tauri/src/trading/simulation.rs`
- 2-file cycle: `src-tauri/src/server/mod.rs -> src-tauri/src/server/profiles.rs -> src-tauri/src/server/mod.rs`

## Communities (508 total, 286 thin omitted)

### Community 0 - "leveraged_trend_hold.rs"
Cohesion: 0.20
Nodes (20): D, S, AppliedRecord, bounded_metadata(), bounded_positions(), deserialize_generation(), explicit_optional_profile(), OwnershipApplied (+12 more)

### Community 1 - "simulation.rs"
Cohesion: 0.04
Nodes (90): handleLookupTossAccounts(), handleSubmit(), activateEmergencyStop(), addProfile(), checkConfig(), checkForUpdate(), checkTossOrderPreflight(), checkTossProfileConnection() (+82 more)

### Community 2 - "hooks.ts"
Cohesion: 0.12
Nodes (27): BalanceItem, BalanceResponse, BalanceSummary, ChartCandle, ExecutedOrder, KisOutput1, OrderRequest, OrderResponse (+19 more)

### Community 3 - "shared/api/commands.ts"
Cohesion: 0.11
Nodes (20): config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope(), daily_buy_order_limit_is_disabled_even_after_submission_count_increases(), daily_order_limit_isolated_by_broker_account_scope(), daily_sell_order_limit_blocks_after_submission_count_reaches_limit(), default_atr_stop_multiplier(), default_daily_buy_order_limit() (+12 more)

### Community 4 - "RiskManager"
Cohesion: 0.04
Nodes (79): canUseTauriEvents(), useBackendEvents(), useAutoTradingBudget(), useUpdateAutoTradingBudget(), useKisExecutedByRange(), useOverseasExecutedByRange(), frontendLogger, OVERSEAS_CHART_PRESETS (+71 more)

### Community 5 - "commands/trading.rs"
Cohesion: 0.14
Nodes (24): account_less_scope_matches_any_account_of_same_broker(), default_currency(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), BrokerId, Mutex (+16 more)

### Community 6 - "ConsecutiveMoveStrategy"
Cohesion: 0.05
Nodes (20): config(), FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy, FiftyTwoWeekState, Default, HashMap, Option, Self (+12 more)

### Community 7 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.05
Nodes (22): config(), deviation_counts_current_price_as_last_required_observation(), deviation_history_is_ready_before_first_evaluation_and_bounded(), DeviationParams, DeviationStrategy, momentum_counts_first_comparison_but_excludes_seed_prices(), momentum_history_does_not_create_or_erase_order_state(), MomentumParams (+14 more)

### Community 8 - "PositionTracker"
Cohesion: 0.05
Nodes (24): initialize_strategy_warmup(), config(), daily(), MeanReversionParams, MeanReversionState, MeanReversionStrategy, Default, HashMap (+16 more)

### Community 9 - "ui/AppShell.tsx"
Cohesion: 0.11
Nodes (12): DailyOrderCountEntry, DailyOrderSide, RiskManager, RiskRuntimeState, BTreeMap, Default, HashMap, NaiveDate (+4 more)

### Community 10 - "OrderManager"
Cohesion: 0.08
Nodes (26): scripts, build, build:app, build:app:debug, build:web, check:fsd, check:graphify, check:lockfiles (+18 more)

### Community 11 - "submission.rs"
Cohesion: 0.05
Nodes (38): useRecentLogs(), useUpdateCheck(), LEVEL_COLORS, Log(), LogLevel, dashboardRoute, historyRoute, logRoute (+30 more)

### Community 12 - "dashboard/ui/Page.tsx"
Cohesion: 0.12
Nodes (22): TossApiResponse, TossCost, TossCurrencyAmount, TossDailyProfitLoss, TossExchangeRateResponse, TossHoldingsItem, TossHoldingsOverview, TossKrIntegratedHour (+14 more)

### Community 13 - "trading/ui/Page.tsx"
Cohesion: 0.12
Nodes (25): get_active_toss_calendar_override(), get_toss_chart_data(), get_toss_chart_data_for_profile(), get_toss_market_calendar(), get_toss_market_calendar_for_profile(), get_toss_market_snapshot(), get_toss_market_snapshot_for_profile(), get_toss_stock_safety() (+17 more)

### Community 14 - "TossOpenApiClient"
Cohesion: 0.05
Nodes (68): MySqlPool, copy_dir_all(), Path, Result, run(), category_from_key(), DatabaseManager, DatabasePool (+60 more)

### Community 15 - "settings.rs"
Cohesion: 0.15
Nodes (18): Future, Output, capture_cycle(), ExecutionCycle, Arc, FnOnce, KisRestClient, Mutex (+10 more)

### Community 16 - "toss/types.rs"
Cohesion: 0.06
Nodes (19): config(), consecutive_counts_current_price_and_uses_active_position_branch(), consecutive_history_preserves_flat_and_existing_positions(), ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation(), failed_breakout_history_retains_actual_breakout_reference() (+11 more)

### Community 17 - "BrokerSymbol"
Cohesion: 0.12
Nodes (21): BrokerAdapterResult, Option, Vec, restart_rebuilds_order_counts_and_consecutive_losses_from_ledgers(), broker_scope_serializes_with_camel_case_fields(), BrokerAccountId, BrokerClientOrderId, BrokerCurrency (+13 more)

### Community 18 - "RsiStrategy"
Cohesion: 0.10
Nodes (8): B, TossOpenApiClient, format_toss_error(), read_toss_response_text(), toss_rate_limit_group(), validate_toss_order_id(), validate_toss_symbol(), TossCredentials

### Community 19 - "fills.rs"
Cohesion: 0.10
Nodes (8): select_price_response(), maps_holding_to_broker_domain(), maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request(), BrokerSymbol, TossPriceLimitResponse, TossPriceResponse

### Community 20 - "accounts.rs"
Cohesion: 0.18
Nodes (16): buildTree(), compareNames(), currentDocument, fail(), generateInventory(), listRepositoryFiles(), mode, nextDocument (+8 more)

### Community 21 - "MeanReversionStrategy"
Cohesion: 0.13
Nodes (19): useStatsByRange(), useTradesByRange(), usePendingOrders(), FilledOrdersPanel(), fmt(), PendingOrdersPanel(), SortDir, todayStr() (+11 more)

### Community 22 - "database_projection.rs"
Cohesion: 0.08
Nodes (62): BrokerCandlePage, candle_market_date(), CandleCursor, CandlePageRequest, kis_cursor_date(), next_toss_cursor(), previous_kis_cursor(), BrokerMarket (+54 more)

### Community 23 - "toss_market.rs"
Cohesion: 0.11
Nodes (30): IntoIterator, applies_retry_after_to_group_pause(), epoch_ms_now(), parse_delay_header(), rate_limit_reset_delay(), RateLimitGroupState, RateLimitGroupStatus, RateLimitScheduler (+22 more)

### Community 24 - "ServerState"
Cohesion: 0.05
Nodes (40): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, Korean, app, security (+32 more)

### Community 25 - "RateLimitScheduler"
Cohesion: 0.16
Nodes (23): broker_money_view_amount(), confirmed_small_order_fill(), ConfirmedSmallOrderFill, is_toss_order_final(), poll_toss_order_detail(), Arc, BrokerCurrency, BrokerId (+15 more)

### Community 26 - "strategy/ui/Page.tsx"
Cohesion: 0.07
Nodes (27): startup_skips_52_week_only_symbols_but_preserves_other_strategy_history(), StrategySignal, build_strategy(), config(), live_feedback_manager_pending_and_unsettled_suppress_only_outward_signal_across_restart(), live_feedback_manager_protection_metadata_requires_exact_owner_and_valid_checkpoint(), live_feedback_manager_targets_exact_owner_scope_and_market(), live_feedback_stateless_three_strategies_share_exact_replay_observations() (+19 more)

### Community 27 - "commands/toss.rs"
Cohesion: 0.13
Nodes (15): AtomicU64, configuration_save_rollback_aba_invalidates_quote_and_position_feedback(), delayed_name_and_late_rollback_do_not_mutate_the_new_run(), delayed_quote_cannot_tick_a_restarted_same_scope_run(), Fixture, Probe, Arc, AtomicUsize (+7 more)

### Community 28 - "tauri.conf.json"
Cohesion: 0.16
Nodes (13): classify_token_response(), detect_trading_type(), DetectedTradingType, DetectTokenReq, DetectTradingTypeError, is_paper_key_rejected_by_real_domain(), is_real_key_rejected_by_paper_domain(), mentions_app_key() (+5 more)

### Community 29 - "ui/StockChart.tsx"
Cohesion: 0.20
Nodes (31): apply(), bars(), completed_252_keeps_final_high_and_uses_high_instead_of_close(), config_aba_and_profile_scope_switch_cannot_revive_old_tickets(), configured_512_window_and_invalid_ohlc_are_not_mislabeled_ready(), current_day_future_missing_and_stale_last_completed_dates_block_readiness(), daily_refresh_never_calls_unrelated_leveraged_intraday_initializers(), disabled_targets_and_config_mutation_remove_ready_history() (+23 more)

### Community 30 - "StrategyManager"
Cohesion: 0.05
Nodes (63): useRefreshStockList(), useStockSearch(), useTossMarketCalendar(), usePreviewLeveragedTrendHold(), useStrategies(), useUpdateStrategy(), BollingerControls(), boolParam() (+55 more)

### Community 31 - "BrokerScope"
Cohesion: 0.12
Nodes (41): AppConfigView, OverseasBalanceItem, OverseasBalanceSummary, add_profile(), AddProfileInput, apply_active_profile(), BalanceResult, broker_market_sort_key() (+33 more)

### Community 32 - "server/market.rs"
Cohesion: 0.22
Nodes (26): MySql, Postgres, balance_snapshot_projects_nested_holdings(), clear_mariadb(), clear_postgres(), delete_source_mariadb(), delete_source_postgres(), extracts_scoped_order_projection() (+18 more)

### Community 33 - "settings/ui/Page.tsx"
Cohesion: 0.13
Nodes (36): AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_trading_type(), DetectTradingTypeResult, get_app_config() (+28 more)

### Community 34 - "KisRestClient"
Cohesion: 0.22
Nodes (17): checked_quantity_cost_and_signed_profit_overflows_are_atomic(), cumulative_buys_and_sells_track_native_weighted_cost_and_proportional_profit(), cumulative_progress_replaces_only_the_latest_durable_application_result(), duplicate_and_regressing_cumulative_reports_are_noops_but_contradictions_fail(), exact_account_profile_mode_owner_market_and_symbol_are_separate_and_orders_cannot_rebind(), fill(), invalid_oversized_unowned_and_duplicate_metadata_fail_atomically_and_on_restore(), key() (+9 more)

### Community 35 - "AccountProfile"
Cohesion: 0.18
Nodes (16): OwnedStrategySignal, ProviderOrderResponse, From, Into, Option, OrderResponse, OrderType, RestOrderSide (+8 more)

### Community 36 - "src/api/types.ts"
Cohesion: 0.15
Nodes (33): align_accounting_scenarios(), broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc() (+25 more)

### Community 37 - "rest/types.rs"
Cohesion: 0.13
Nodes (10): DateRangeQuery, frontend_log_handler(), FrontendLogBody, kis_executed_handler(), set_archive_config_handler(), set_log_config_handler(), SetArchiveConfigBody, SetLogConfigBody (+2 more)

### Community 38 - "databaseManagementSection.tsx"
Cohesion: 0.12
Nodes (28): automatic_missing_context_and_stale_reservation_leave_budget_and_pending_unchanged(), buy(), delayed_old_account_insufficient_balance_does_not_suspend_new_account(), legacy_configured_account_without_profile_allows_manual_but_fences_client_replacement(), local_holdings(), local_order(), local_single_order(), LocalProvider (+20 more)

### Community 39 - "DatabaseManager"
Cohesion: 0.33
Nodes (15): commit_rejects_scope_profile_client_and_credential_changes(), config(), configuration_fence_blocks_shared_automatic_and_manual_before_any_http_or_budget_change(), configuration_snapshots_include_real_held_and_flat_markets_and_revalidate_fresh_quantity(), duplicate_entries_use_the_same_first_market_as_the_live_strategy(), fixture(), held_target_removal_and_explicit_pc_lth_market_changes_fail_closed(), legacy_inverse_field_cannot_override_another_lth_targets_market() (+7 more)

### Community 40 - "AppState"
Cohesion: 0.15
Nodes (12): KrxItem, KrxProxyItem, KrxProxySearchResponse, KrxResponse, NaverAcItem, NaverAcResponse, save_cache(), StockList (+4 more)

### Community 41 - "server/records.rs"
Cohesion: 0.07
Nodes (57): MySqlSslMode, PgSslMode, assert_failed_save_preserves_pair(), cancelled_save_before_memory_publication_blocks_further_connections(), clear_recovery_marker(), committed_file_without_memory_publication_stays_fail_closed(), create_recovery_marker(), DatabaseManager (+49 more)

### Community 42 - "StrategyConfig"
Cohesion: 0.22
Nodes (5): OrderRecord, OrderSide, OrderStatus, OrderStore, parallel_appends_do_not_lose_order_records()

### Community 43 - "domain.rs"
Cohesion: 0.07
Nodes (34): useSetTradeArchiveConfig(), useTradeArchiveConfig(), useTradeArchiveStats(), useLogConfig(), useSetLogConfig(), useSetStockUpdateInterval(), useStockListStats(), useBrokerRateLimitStatus() (+26 more)

### Community 44 - "archive.rs"
Cohesion: 0.04
Nodes (84): useAppConfig(), useBalance(), useBrokerHoldings(), useOverseasBalance(), useScopedAccountQuery(), useTodayStats(), useExchangeRate(), useExchangeRateStatus() (+76 more)

### Community 45 - "commands/records.rs"
Cohesion: 0.27
Nodes (12): collect_trade_archive_stats(), collect_trade_archive_stats_active(), collect_trade_day_dirs(), dir_size_bytes(), get_trade_archive_config(), get_trade_archive_stats(), purge_old_trade_files(), purge_trade_archive_active() (+4 more)

### Community 46 - "PriceConditionStrategy"
Cohesion: 0.19
Nodes (20): fetch_overseas_tick(), fetch_toss_tick(), is_market_closed_error(), poll_symbols_tick(), PriceSource, Arc, KisRestClient, Mutex (+12 more)

### Community 47 - "detect.rs"
Cohesion: 0.15
Nodes (3): DiscordNotifier, NotificationEvent, NotificationLevel

### Community 48 - "submit_toss_small_buy_verification_for_profile"
Cohesion: 0.19
Nodes (21): collect_kis_pending_fills(), collect_toss_pending_fills(), domestic_order_is_terminal(), domestic_order_terminal_status(), domestic_provider_status(), kis_pending_from(), normalized_execution_u64(), overseas_order_terminal_status() (+13 more)

### Community 49 - "market/mod.rs"
Cohesion: 0.09
Nodes (15): profile_fingerprint(), client(), Fixture, profile(), Arc, Drop, KisRestClient, Mutex (+7 more)

### Community 50 - "profiles.rs"
Cohesion: 0.05
Nodes (30): get_positions(), get_strategies(), PositionView, BrokerId, CmdResult, From, Option, Self (+22 more)

### Community 51 - "server/trading.rs"
Cohesion: 0.08
Nodes (26): apply_slippage(), bps_cost(), current_equity(), execution_block_reason(), krw_to_display(), phase_for_index(), PositionState, return_pct() (+18 more)

### Community 52 - "OrderRecord"
Cohesion: 0.12
Nodes (31): lth_default_adx_period(), lth_default_breakeven_buffer(), lth_default_buy_adx(), lth_default_buy_rsi(), lth_default_ema_long(), lth_default_ema_short(), lth_default_entry_end(), lth_default_entry_failure_observations() (+23 more)

### Community 53 - "StockStore"
Cohesion: 0.09
Nodes (30): File, AppState, CmdError, make_rest_client(), Arc, AtomicBool, BrokerId, DatabaseManager (+22 more)

### Community 54 - "security.rs"
Cohesion: 0.20
Nodes (26): FetchFuture, Fn, at_most_two_jobs_run_and_other_instruments_are_not_starved(), completed(), failures_use_backoff_and_do_not_retry_on_every_quote_tick(), history(), insufficient_history_reports_real_count_reason_and_retry(), maintain_captures_the_new_scope_atomically_after_a_waiting_start() (+18 more)

### Community 55 - "dependencies"
Cohesion: 0.19
Nodes (3): StockEntry, StockListStats, StockStore

### Community 56 - "logging/mod.rs"
Cohesion: 0.09
Nodes (44): DividendScenarioPoint, FxScenarioPoint, daily_diagnostic_never_evaluates_strategy_or_execution_even_with_intraday_input(), CostSource, empty_minute_observations_are_not_evaluable(), ReplayAssessmentView, Self, String (+36 more)

### Community 57 - "active_profile"
Cohesion: 0.07
Nodes (27): Display, BrokerAdapterResult, BrokerCurrency, BrokerId, Debug, Default, Formatter, Into (+19 more)

### Community 58 - "compilerOptions"
Cohesion: 0.22
Nodes (17): BudgetCurrencyView, allocation_cannot_withdraw_committed_cash_or_overflow(), AutoTradingBudgetView, budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent(), BudgetAccount, BudgetLedger, cost(), manager() (+9 more)

### Community 59 - "commands/database.rs"
Cohesion: 0.11
Nodes (23): Barrier, daily_order_limit_key(), estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), OrderManager, PendingOrder, Arc (+15 more)

### Community 60 - "commands/market.rs"
Cohesion: 0.13
Nodes (13): validates_order_create_request_shape(), TossOrderCreateRequest, market_from_currency(), new_toss_client_order_id(), toss_currency(), toss_market(), validate_client_order_id(), validate_optional_decimal() (+5 more)

### Community 61 - "DiscordNotifier"
Cohesion: 0.10
Nodes (29): useClearDatabaseTables(), useCreateDatabaseTables(), useDatabaseConfig(), useDropDatabaseTables(), useExportDatabaseToJson(), useImportJsonToDatabase(), useJsonStorageInventory(), useSaveDatabaseConfig() (+21 more)

### Community 62 - "DatabaseConfig"
Cohesion: 0.33
Nodes (31): assumptions(), bar(), buy(), directional_fees_and_native_minimum_are_converted_with_the_current_fx(), dividend_adjusted_prices_reject_known_double_counting(), dividend_entitlement_precedes_pending_open_and_respects_partial_exit_quantity(), future_fx_point_cannot_revalue_earlier_mark_or_buy(), fx_changes_krw_mark_and_realized_basis_without_changing_native_strategy_entry() (+23 more)

### Community 63 - "DatabasePool"
Cohesion: 0.13
Nodes (18): OverseasPriceResponse, KisOrderRejected, KisRestClient, read_kis_response_text(), Arc, AtomicBool, ExecutedOrder, HeaderMap (+10 more)

### Community 64 - "TradeRecord"
Cohesion: 0.22
Nodes (6): clear_database_tables(), drop_database_tables(), ensure_trading_stopped(), export_database_to_json(), import_json_to_database(), set_storage_backend()

### Community 65 - "release-version.mjs"
Cohesion: 0.16
Nodes (21): Deserialize, BrokerId, Json, Option, Path, Result, State, StatusCode (+13 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.11
Nodes (26): assert_auth(), candle_values(), chart_response_handles_empty_output_and_malformed_multibyte_body(), chart_response_preserves_http_and_provider_errors(), chart_response_propagates_announced_and_observed_body_caps(), domestic_latest_first_fixture_reverses_and_maps_ohlcv(), domestic_request_keeps_date_bounds_period_adjustment_and_query_order(), overseas_dailyprice_request_uses_its_own_tr_id_and_period_codes() (+18 more)

### Community 67 - "scripts"
Cohesion: 0.13
Nodes (45): activate_emergency_handler(), app_config_handler(), check_config_handler(), check_update_handler(), clear_buy_suspension_handler(), clear_emergency_handler(), exchange_rate_handler(), exchange_rate_status_handler() (+37 more)

### Community 68 - "place_toss_order"
Cohesion: 0.06
Nodes (57): EditResult, ConfigurationEditReservation, ConfigurationUpdateError, ConfigurationUpdateRequest, ConfigurationUpdateState, effective_target_markets(), Arc, HashMap (+49 more)

### Community 69 - "conflicts.rs"
Cohesion: 0.12
Nodes (46): check_toss_order_preflight(), check_toss_order_preflight_for_profile(), check_toss_profile_connection(), list_toss_accounts(), list_toss_open_orders(), list_toss_open_orders_for_profile(), list_toss_profile_accounts(), lookup_toss_accounts_with_credentials() (+38 more)

### Community 70 - "toss/tests.rs"
Cohesion: 0.33
Nodes (8): attach_comparison(), BuyHoldComparisonView, comparator_preserves_actual_trades_and_discloses_final_pending(), CostRestoredReferenceView, ReplayComparisonView, String, Vec, same_quantity_reference_discloses_exposure_and_restores_only_recorded_costs()

### Community 71 - "commands/risk.rs"
Cohesion: 0.13
Nodes (26): absent_unknown_and_similarly_prefixed_kis_tr_ids_preserve_pending_without_querying(), actual_fill_crosses_same_scope_restart_with_identical_provider_snapshot(), delayed_reconciliation_cannot_mutate_new_scope_and_applies_when_original_is_reselected(), direct_fill_and_zero_fill_terminal_cannot_mutate_another_execution_scope(), exact_eight_order_tr_ids_select_mode_and_toss_bypasses_kis_mode_classification(), executed(), persisted_kis_pending_never_queries_or_applies_in_the_opposite_mode(), Provider (+18 more)

### Community 72 - ".new"
Cohesion: 0.25
Nodes (25): add_profile_handler(), AddProfileBody, apply_profile_change(), default_body_broker_id(), delete_profile_handler(), DeleteProfileBody, detect_profile_handler(), detect_trading_type_handler() (+17 more)

### Community 73 - "storage/database.rs"
Cohesion: 0.05
Nodes (50): usePreviewStrategy(), ProfileFormState, assertPageScope(), CandleCollection, CandleCollectionEnd, candleTimestamp(), checkCancelled(), collectPreviewCandles() (+42 more)

### Community 74 - "KisWebSocketClient"
Cohesion: 0.08
Nodes (20): FormatTime, FrontendLogInput, get_recent_logs(), get_stats_by_range(), get_trades_by_range(), GetStatsByRangeInput, GetTradesByRangeInput, save_trade() (+12 more)

### Community 75 - "toss/orders.rs"
Cohesion: 0.22
Nodes (15): DomesticHoldings, OverseasHoldings, ExpectedSource, HoldingsMarkets, HoldingsQuery, HoldingsUpdate, OrderManager, Arc (+7 more)

### Community 76 - "read_json_or_default"
Cohesion: 0.39
Nodes (4): KisWebSocketClient, parse_realtime_price(), RealtimePrice, WsStatusEvent

### Community 77 - "StatsStore"
Cohesion: 0.12
Nodes (30): drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract(), postgresql_rejects_invalid_credentials(), Option, Path (+22 more)

### Community 78 - "devDependencies"
Cohesion: 0.17
Nodes (17): lth_preview_signals_for_interval(), F, LeveragedTrendHoldPreviewSignal, OhlcCandle, LeveragedRapidReboundSnapshot, LeveragedReboundSnapshot, LeveragedTrendHoldPreviewStep, LeveragedTrendHoldTimedCandle (+9 more)

### Community 79 - "project-map.mjs"
Cohesion: 0.20
Nodes (13): ChartRequest, ChartRequest<'a>, KisRestClient, parse_domestic_response(), parse_overseas_response(), ChartCandle, Client, HeaderMap (+5 more)

### Community 80 - "TokenManager"
Cohesion: 0.10
Nodes (27): balance_summary_total_krw(), parse_amount_f64(), parse_amount_i64(), BalanceSummary, Option, fetch_account_risk_balance_krw(), fetch_toss_risk_balance_krw(), Arc (+19 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.08
Nodes (12): live_feedback_price_condition_changes_position_only_after_confirmed_fill(), strategy(), PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick(), HashMap, Option (+4 more)

### Community 82 - ".write_document"
Cohesion: 0.12
Nodes (12): config(), initializer_restores_previous_averages_for_first_evaluation_cross(), insufficient_history_has_no_previous_long_average(), MaCrossParams, MaCrossState, MovingAverageCrossStrategy, Default, HashMap (+4 more)

### Community 83 - "KisBrokerAdapter"
Cohesion: 0.08
Nodes (48): react, AccountConfigRefreshError, accountScope(), holdingsPrefixes, profileMutationKey, profileMutationScope, reconcileRejectedProfileMutation(), refreshAccountConfig() (+40 more)

### Community 84 - "support.rs"
Cohesion: 0.18
Nodes (9): BacktestAtrPreparation, BacktestReport, SimulationAssumptions, calculateAtr(), maCard(), mockAtrResult(), readyAtr, mockExecution() (+1 more)

### Community 85 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.17
Nodes (12): ExperimentScope, accountingComparisonKey(), assumptionFields, AssumptionsProps, eventTimeLabel(), executionAssumptionsLabel(), executionModelLabel(), fmtPct() (+4 more)

### Community 86 - "broker/adapter.rs"
Cohesion: 0.18
Nodes (15): PendingCandidate, PendingFill, OrderManager, ReconciliationSnapshot, Arc, BrokerId, KisRestClient, Mutex (+7 more)

### Community 87 - "ui/LayoutResizer.tsx"
Cohesion: 0.20
Nodes (16): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, BrokerCurrency, CmdResult, Option (+8 more)

### Community 88 - "initialize_active_strategy_history"
Cohesion: 0.14
Nodes (29): LeveragedTrendHoldEntry, LeveragedTrendHoldStrategy, live_feedback_lth_checkpoint_keeps_peak_age_and_authoritative_owned_average(), live_feedback_lth_rejected_and_partial_exits_preserve_current_tick_protection(), live_feedback_lth_replay_still_clears_optimistic_exit(), reconfigure_lth_keeps_exact_protection_lifecycle_and_uses_new_exit_parameters(), reconfigure_lth_keeps_initial_stop_without_new_window_warmup(), reconfigure_lth_trims_raw_observations_to_new_caps_without_reconstructing_position() (+21 more)

### Community 89 - "PendingOrderStore"
Cohesion: 0.17
Nodes (7): calculate_domestic_fee(), calculate_overseas_fee_cents(), cumulative_fill_delta(), OrderManager, NaiveDate, Option, Result

### Community 90 - "StrategyStore"
Cohesion: 0.20
Nodes (13): DividendPriceBasis, DividendScenarioPoint, FxScenarioPoint, ReplayAccountingView, HashSet, Item, Iterator, Result (+5 more)

### Community 91 - "TradeStore"
Cohesion: 0.15
Nodes (41): account_sync_error(), balance_handler(), broker_holdings_handler(), candle_page_handler(), chart_handler(), ChartQuery, executed_handler(), invalid_order_input() (+33 more)

### Community 92 - "database_keychain.rs"
Cohesion: 0.11
Nodes (19): BacktestPhase, BacktestSummary, BacktestTrade, EquityPoint, FxScenarioPoint, LeveragedTrendHoldPreviewSignal, ReplayAccounting, ReplayAssessment (+11 more)

### Community 93 - "@mui/icons-material"
Cohesion: 0.47
Nodes (4): check(), GitHubRelease, is_newer(), UpdateInfo

### Community 94 - "balance_store.rs"
Cohesion: 0.19
Nodes (19): fixture(), fixture_with_base(), ownership_fill_empty_sell_snapshot_uses_exact_owner_cost_across_partial_and_repeat(), ownership_fill_exact_native_notional_preserves_301_total_and_partial_remainder(), ownership_fill_larger_terminal_cumulative_waits_for_earlier_intent_then_applies_every_share(), ownership_fill_legacy_applying_intent_requires_durable_date_fx_and_notional(), ownership_fill_legacy_sell_does_not_adopt_other_owners_account_average(), ownership_fill_manual_saved_cost_survives_empty_account_snapshot() (+11 more)

### Community 95 - ".load_sync"
Cohesion: 0.15
Nodes (14): LeveragedTrendHoldEntry, LeveragedTrendHoldMarketState, LeveragedTrendHoldParams, LeveragedTrendHoldPosition, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldStrategy, Default, HashMap (+6 more)

### Community 96 - "check"
Cohesion: 0.08
Nodes (23): DOM, DOM.Iterable, ES2020, src, compilerOptions, allowImportingTsExtensions, isolatedModules, jsx (+15 more)

### Community 97 - "compilerOptions"
Cohesion: 0.60
Nodes (3): create(), mariadb_ddl(), postgres_ddl()

### Community 98 - "check-fsd-imports.mjs"
Cohesion: 0.19
Nodes (19): buy(), checkpoint_rejects_old_run_and_configuration_generation_without_writing(), feedback_assigns_only_exact_owner_amid_manual_and_other_account_holdings(), feedback_cold_lth_restores_metadata_and_partial_change_preserves_live_high(), feedback_never_transfers_same_account_quantity_across_profile_or_mode(), key(), lth_config(), manager() (+11 more)

### Community 99 - "package.json"
Cohesion: 0.40
Nodes (4): KisRestClient, Option, Self, super::OwnershipIdentity

### Community 100 - "verify-toss-openapi.mjs"
Cohesion: 0.11
Nodes (17): devDependencies, @playwright/test, @tauri-apps/cli, @types/react, @types/react-dom, @types/react-grid-layout, typescript, vite (+9 more)

### Community 101 - "OverseasExecutedOrder"
Cohesion: 0.12
Nodes (22): useChartData(), useTossChartData(), buildChartOptions(), CandlePoint, CHART_PRESETS, ChartPreset, ChartSource, ChartType (+14 more)

### Community 102 - "BrokerAccountId"
Cohesion: 0.15
Nodes (17): args, assertGitClean(), assertTagAvailable(), branchName, changedFiles, currentBranchName(), escapeRegExp(), fail() (+9 more)

### Community 103 - "TossAccessToken"
Cohesion: 0.20
Nodes (7): OrderManager, Option, OrderSide, PendingOrder, Result, String, Vec

### Community 104 - "atomic_write"
Cohesion: 0.08
Nodes (24): @emotion/react, @emotion/styled, lightweight-charts, @mui/icons-material, @mui/material, dependencies, @emotion/react, @emotion/styled (+16 more)

### Community 106 - "TossApiError"
Cohesion: 0.18
Nodes (12): AccessToken, Arc, Client, DateTime, Mutex, Option, Result, Self (+4 more)

### Community 107 - "strategyResearchTypes.ts"
Cohesion: 0.54
Nodes (7): clearExperimentSlots(), compactSnapshot(), loadExperimentSlots(), normalizePart(), readIndex(), saveExperimentSlot(), storageKey()

### Community 116 - "build.rs"
Cohesion: 0.12
Nodes (26): RecentSideHistory, ScopedSymbol, blocks_reentry_after_stop_loss_sell_on_same_day(), blocks_repeated_same_side_signal_inside_cooldown(), blocks_tiny_profit_after_costs(), blocks_whipsaw_after_repeated_opposite_signals(), buy_signal(), cooldown_is_isolated_by_broker_account_scope() (+18 more)

### Community 118 - "settings-database.spec.ts"
Cohesion: 0.50
Nodes (3): TossApiError, TossErrorResponse, TossHttpError

### Community 119 - "kis-auto-trade"
Cohesion: 0.06
Nodes (70): PreviewRow, chart_volume_to_u64(), blocked_strong_close_buy_does_not_discard_next_days_completed_condition(), candle(), daily_event_strategies_reject_other_cadences(), flat_completed_days_fill_history_without_compressing_the_window(), input(), ChartCandle (+62 more)

### Community 120 - "playwright.config.ts"
Cohesion: 0.15
Nodes (13): useOverseasChartData(), buildChartOptions(), CandlePoint, ChartType, CrosshairData, LinePoint, OverseaChartPreset, OVERSEAS_CHART_PRESETS (+5 more)

### Community 121 - "sync-codex-skills.ps1"
Cohesion: 0.43
Nodes (3): order_side_label(), OrderSide, submitting_conflict_reason()

### Community 122 - "discord-notification-config/index.ts"
Cohesion: 0.21
Nodes (10): BrokerAdapter, BrokerAdapterError, default_place_order_returns_unsupported_with_broker_id(), DummyAdapter, BrokerAdapterResult, BrokerId, Error, Send (+2 more)

### Community 123 - "log-filter/index.ts"
Cohesion: 0.10
Nodes (5): LeveragedTrendHoldStrategy, Option, Result, String, Value

### Community 124 - "strategy-configure/index.ts"
Cohesion: 0.21
Nodes (13): Next, Router, app(), bearer_token(), constant_time_eq(), cross_origin_mutation_is_rejected_even_with_token(), has_same_origin(), mutation_without_token_is_rejected() (+5 more)

### Community 125 - "strategy-toggle/index.ts"
Cohesion: 0.31
Nodes (16): HoldingsUpdate, a_change_to_either_market_rejects_both_without_partial_publication(), configuration_lease_rejects_capture_and_inflight_response_even_after_release(), domestic(), empty_and_zero_holdings_clear_only_the_queried_market(), fixture(), held(), identity_changes_reject_without_mutating_either_tracker() (+8 more)

### Community 126 - "Result"
Cohesion: 0.37
Nodes (8): Ok, cumulative(), proportional_cost(), Result, Value, signed_difference(), validate_application(), validate_metadata()

### Community 127 - "PendingOrderStore"
Cohesion: 0.26
Nodes (10): corrupted_primary_never_resurrects_orders_from_a_valid_backup(), PendingOrderStore, restart_snapshot_preserves_scope_client_id_and_fill_watermark(), Mutex, PathBuf, PendingOrder, Result, Self (+2 more)

### Community 128 - "vite.config.ts"
Cohesion: 0.28
Nodes (16): order_side_label(), pending_conflict_blocks_opposite_side_in_same_scope(), pending_conflict_blocks_same_side_in_same_scope(), pending_conflict_reason_for_scope(), pending_conflict_scan_ignores_different_scope(), pending_order(), pending_order_conflict_reason(), pending_order_provider_uses_provider_trace() (+8 more)

### Community 129 - "input"
Cohesion: 0.23
Nodes (10): AtrPreparationView, AtrWindow, bar(), downward_gap_on_flat_candle_is_not_a_zero_range(), full_previous_close_history_is_required_and_old_ranges_expire(), large_prices_do_not_overflow_the_range_sum(), Option, String (+2 more)

### Community 130 - ".new"
Cohesion: 0.18
Nodes (13): DailyHistoryEntry, DailyHistoryStatus, DailyHistoryTicket, fingerprint(), pending_status(), BrokerId, DateTime, NaiveDate (+5 more)

### Community 131 - ".fmt"
Cohesion: 0.31
Nodes (7): pending_order_provider(), BrokerId, OrderManager, Option, PendingOrder, Result, validate_pending_kis_mode()

### Community 132 - ".new"
Cohesion: 0.17
Nodes (10): AsyncMutex, shared_toss_token_state(), toss_token_state_key(), TossTokenState, body_snippet(), toss_http_client(), trim_base_url(), url_encode() (+2 more)

### Community 133 - ".search"
Cohesion: 0.19
Nodes (16): LeveragedTrendHoldPreviewInput, TossModifyOrderInput, TossOpenOrdersInput, TossOrderPreflightInput, TossSmallBuyVerificationInput, active_profile(), leveraged_trend_hold_preview_handler(), toss_chart_handler() (+8 more)

### Community 134 - "run_backtest"
Cohesion: 0.23
Nodes (15): run_backtest_with_history(), atr_history(), atr_risk_uses_previous_history_close_and_evaluation_gap_without_trading_history(), atr_sizing_assumptions(), atr_sizing_never_silently_uses_requested_quantity_when_ranges_are_missing(), daily_loss_and_consecutive_loss_state_roll_over_on_event_date(), deterministic_replay_applies_costs_and_separates_signals_from_fills(), event() (+7 more)

### Community 135 - "AppConfigView"
Cohesion: 0.20
Nodes (4): OrderManager, Option, String, TradingHealthStatus

### Community 136 - "작업 기록: <짧은 제목>"
Cohesion: 0.23
Nodes (17): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), calculate_atr(), initialize_active_strategy_history(), legacy_history_symbols(), Arc (+9 more)

### Community 137 - "write_json"
Cohesion: 0.15
Nodes (10): build_monthly_path(), concurrent_writes_leave_one_complete_json_document(), corrupt_json_recovers_from_last_durable_backup(), json_path_lock(), monthly_path_zero_pads_month_for_stats(), read_json_or_default(), write_json(), DailyStats (+2 more)

### Community 138 - ".process_buy"
Cohesion: 0.42
Nodes (7): OrderManager, Option, OrderResponse, OrderSide, Result, String, order_exchange_code()

### Community 139 - "CurrencyLedger"
Cohesion: 0.23
Nodes (7): LeveragedTrendHoldStrategy, Option, Vec, VecDeque, LeveragedTrendSnapshot, PreviewPosition, Option

### Community 141 - "modify_toss_order_for_profile"
Cohesion: 0.29
Nodes (9): failed_configuration_pause_never_becomes_market_close_and_same_revision_resumes(), market_closed_first_order_settles_remaining_raw_batch_without_touching_submitted_state(), old_market_closed_batch_cannot_restore_positions_into_restarted_run(), Arc, Fixture, Mutex, StrategyManager, strategies() (+1 more)

### Community 142 - "OrderSubmission"
Cohesion: 0.27
Nodes (14): OrderSubmission, build_provider_request(), prepare_buy_submission(), prepare_order_submission(), prepare_sell_submission(), rest_order_side(), Option, OrderSide (+6 more)

### Community 143 - "HARNESS_MAP.md"
Cohesion: 0.18
Nodes (14): Band, BollingerParams, candle(), disabled_option_preserves_entries_and_buffer_is_bounded(), downward_breakout_and_insufficient_history_block_entries(), flat_squeeze_needs_completed_upward_breakout(), LeveragedTrendHoldStrategy, live_and_replay_share_confirmed_breakout_and_exit_without_future_prices() (+6 more)

### Community 145 - "react-grid-layout"
Cohesion: 0.18
Nodes (12): buildSnapshot(), fail(), listRepositoryFiles(), mode, parseMode(), REPOSITORY_ROOT, ROOT_CONFIGS, SCRIPT_DIR (+4 more)

### Community 147 - "Result"
Cohesion: 0.26
Nodes (13): accountingAssumptionsLabel(), AccountingAssumptionSummary(), AccountingResults(), ComparisonResults(), EditorProps, numberInRange(), parseScenarios(), percent() (+5 more)

### Community 148 - "Arc"
Cohesion: 0.25
Nodes (13): bodyHtml(), checkFreshness(), escapeHtml(), inlineMarkdown(), outputPath, parseTasks(), renderTodo(), root (+5 more)

### Community 149 - "AtomicBool"
Cohesion: 0.35
Nodes (20): assumptions(), assumptions_normalize_nonfinite_protection_and_expose_selected_model(), atr_that_only_becomes_ready_after_execution_cannot_size_the_open_order(), bar(), both_touch_uses_stop_first_only_when_selected_and_reports_ambiguity(), buy(), default_same_close_legacy_numeric_fixture_is_preserved(), delayed_fill_uses_next_observed_open_and_final_signal_remains_unfilled() (+12 more)

### Community 150 - "ChartCandle"
Cohesion: 0.19
Nodes (11): Requests, accept(), actual_submissions_sell_only_the_confirmed_owner_and_never_add_fill_to_account_snapshot(), ownership_recheck_after_preparation_prevents_a_stale_sell_reservation(), publish(), Fixture, Json, OrderSide (+3 more)

### Community 151 - "Client"
Cohesion: 0.16
Nodes (12): restore_risk_from_today_trades(), Arc, CmdResult, Mutex, RiskManager, risk_rebuild_isolates_paper_profit_from_live_losses_for_the_same_account(), risk_rebuild_seeds_trade_event_receipts_and_retries_missing_pending_intent_once(), risk_rebuild_uses_chronological_profits_and_keeps_manual_emergency_stop() (+4 more)

### Community 152 - "Duration"
Cohesion: 0.31
Nodes (9): execution_quantity(), OrderManager, parts(), BrokerCurrency, NaiveDate, Option, Result, scaled_total() (+1 more)

### Community 153 - "ExecutedOrder"
Cohesion: 0.18
Nodes (11): usd_to_cents(), StockListStats, ChartDataInput, get_chart_data(), get_overseas_price(), get_price(), get_stock_list_stats(), OverseasOrderInput (+3 more)

### Community 154 - "HeaderMap"
Cohesion: 0.27
Nodes (10): apply(), bounded_receipts_and_numeric_overflow_reject_without_partial_mutation(), rebuild_replays_durable_trades_then_missing_intent(), roundtrip(), Result, RiskManager, scope(), scope_is_part_of_event_identity_and_payload_conflicts_fail_closed() (+2 more)

### Community 155 - "OrderResponse"
Cohesion: 0.21
Nodes (17): money_view_from_decimal(), buy_preflight_adds_commission_and_blocks_when_cash_is_short(), buy_preflight_passes_when_required_cash_is_available(), evaluate_order_preflight(), format_money_amount(), format_quantity(), invalid_commission_ratios_block_instead_of_becoming_zero_fee(), OrderPreflightConstraints (+9 more)

### Community 156 - "OverseasExecutedOrder"
Cohesion: 0.62
Nodes (6): config(), reconfigure_fifty_two_week_keeps_actual_entry_while_invalidating_old_window(), reconfigure_strong_close_preserves_stop_but_discards_old_entry_candidate(), reconfigure_volatility_recomputes_range_without_losing_held_stop_or_day(), reject_scope_changes(), Value

### Community 157 - "PriceResponse"
Cohesion: 0.12
Nodes (20): DailyHistoryJobs, JobEntry, retry_delay(), Arc, BTreeMap, DateTime, Drop, Duration (+12 more)

### Community 159 - "place_toss_order"
Cohesion: 0.11
Nodes (14): BrokerPositionSnapshot, default_strategy_broker_id(), BrokerId, BrokerMarket, Into, Option, Self, String (+6 more)

### Community 160 - "collect_pages"
Cohesion: 0.14
Nodes (28): CandlePageView, collect_daily_history(), collect_pages(), CollectedDailyHistory, completed_bar(), price(), Arc, ChartCandle (+20 more)

### Community 161 - ".checkpoint_strategy_positions_shared"
Cohesion: 0.47
Nodes (6): OrderManager, Arc, Mutex, Result, Self, StrategyManager

### Community 162 - ".capture"
Cohesion: 0.15
Nodes (10): OwnershipLedger, Default, Option, Self, OrderManager, OwnershipManifest, Result, String (+2 more)

### Community 163 - "ExchangeRateView"
Cohesion: 0.25
Nodes (8): ExchangeRateSource, ExchangeRateView, refresh_exchange_rate_status(), resolve_exchange_rate_policy(), Arc, Result, RwLock, Self

### Community 164 - "collect_daily_history"
Cohesion: 0.19
Nodes (17): accepted_http_response_publishes_before_its_event_under_the_same_gate(), balance(), delayed_http_a_to_b_to_a_response_never_publishes_or_emits(), delayed_http_response_cannot_undo_a_concurrent_buy_or_sell(), invalid_holdings_do_not_clear_the_previous_position_or_emit_success(), refresh(), Arc, AtomicUsize (+9 more)

### Community 165 - "toss/orders.rs"
Cohesion: 0.24
Nodes (7): every_read_observes_current_primary_and_structural_errors_are_not_empty_ledgers(), fill(), missing_primary_starts_empty_and_real_file_restart_keeps_exact_idempotent_fill(), primary_corruption_blocks_instead_of_recovering_an_old_valid_backup(), primary_loss_is_detected_after_restart_even_with_no_pending_orders_and_a_valid_backup(), primary_only_creation_crash_gap_anchors_the_existing_generation_and_ownership(), Fixture

### Community 175 - "Mutex"
Cohesion: 0.38
Nodes (12): candle(), config(), daily_hooks_do_not_create_unknown_symbol_states(), Value, strong_close_completed_day_uses_actual_position_and_keeps_stop_loss(), strong_close_uses_each_completed_day_only_on_the_following_day(), volatility(), volatility_collects_missing_history_after_close_for_later_days() (+4 more)

### Community 181 - "AppHandle"
Cohesion: 0.18
Nodes (9): args, artifactDir, artifacts, checksumPath, manifestPath, names, records, requireSignatures (+1 more)

### Community 182 - "MutexGuard"
Cohesion: 0.22
Nodes (10): OwnershipFill, OwnershipIdentity, OwnershipKey, OwnershipOwner, BrokerMarket, Debug, Formatter, Item (+2 more)

### Community 183 - ".submit_order_shared"
Cohesion: 0.21
Nodes (12): certificates(), command(), containers, directory, environment, interruption, results, root (+4 more)

### Community 184 - ".new"
Cohesion: 0.26
Nodes (9): ExecutionRun, ExecutionRunContext, OrderManager, Arc, KisRestClient, Option, Result, String (+1 more)

### Community 190 - "Vec"
Cohesion: 0.24
Nodes (10): AutoTradingBudgetInput, UpdateAutoTradingBudgetInput, AutoTradingBudgetInput, checked_budget_scope(), get_auto_trading_budget(), update_auto_trading_budget(), UpdateAutoTradingBudgetInput, budget_handler() (+2 more)

### Community 192 - "modify_toss_order_for_profile"
Cohesion: 0.28
Nodes (11): chartTimeToMs(), formatChartTimeKst(), formatKstMs(), pad2(), parseTimeMs(), Props, StrategyPreviewChart(), StrategyPreviewChartSignal (+3 more)

### Community 193 - "order/configuration/tests.rs"
Cohesion: 0.18
Nodes (6): toss_market_session_window(), calendar_override_represents_holiday_without_fallback(), calendar_session_window_checks_time_range(), MarketDayCalendar, MarketSessionWindow, us_session_policy_checks_extended_sessions()

### Community 194 - "commands/risk.rs"
Cohesion: 0.36
Nodes (12): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+4 more)

### Community 195 - "PreparedOrderSubmission"
Cohesion: 0.29
Nodes (11): PreparedOrderSubmission, append_failed_order(), build_kis_pending_order(), build_pending_order(), build_toss_pending_order(), failed_order_price(), Arc, Option (+3 more)

### Community 196 - "MarketDayCalendar"
Cohesion: 0.36
Nodes (7): normalize_overseas_order_exchange(), HoldingsUpdate, money(), quantity(), Result, Self, String

### Community 197 - "SubmissionDeps"
Cohesion: 0.22
Nodes (9): prepare_ownership(), Option, OrderManager, Result, Arc, KisRestClient, RiskManager, RwLock (+1 more)

### Community 199 - "Self"
Cohesion: 0.40
Nodes (3): LeveragedTrendHoldStrategy, Option, String

### Community 200 - "config"
Cohesion: 0.23
Nodes (13): Send, Sync, Strategy, config(), reconfigure_mean_reversion_keeps_stop_active_before_larger_window_is_ready(), reconfigure_trend_filter_retains_held_flag_and_uses_new_period_for_exit(), reject_scope_changes(), Value (+5 more)

### Community 201 - "holding"
Cohesion: 0.38
Nodes (7): holding(), BrokerMarket, scope(), toss_fractional_and_zero_quantities_never_create_a_whole_share(), toss_rejects_invalid_quantities_and_prices_including_float_u64_boundary(), toss_rejects_wrong_broker_account_currency_and_duplicate_market_symbol(), toss_us_prices_round_to_cents_and_kr_prices_round_to_won()

### Community 202 - "Arc"
Cohesion: 0.22
Nodes (5): layerRank, layers, root, srcRoot, violations

### Community 203 - "BalanceItem"
Cohesion: 0.22
Nodes (6): appEntry, escapedName, npmLock, pkg, rootDir, workspaceLock

### Community 204 - "StrategyStore"
Cohesion: 0.42
Nodes (6): BudgetCurrencyView, BudgetReservation, CurrencyLedger, BTreeMap, Option, String

### Community 205 - "config"
Cohesion: 0.53
Nodes (5): config(), empty_price_condition_entries_remove_stale_targets_before_manager_validation(), reconfigure_price_condition_canonicalizes_targets_and_keeps_actual_stop(), reconfigure_price_condition_does_not_move_holdings_to_replacement_target(), Value

### Community 206 - "strategy/tests.rs"
Cohesion: 0.22
Nodes (3): TossOrderListQuery, TossOrderListStatus, validate_iso_date()

### Community 207 - ".prepare_confirmed_ownership"
Cohesion: 0.33
Nodes (6): OrderManager, PreparedOwnershipFill, realized_pnl(), Option, PendingOrder, Result

### Community 208 - "TossTokenState"
Cohesion: 0.40
Nodes (7): OrderManager, Arc, Mutex, Option, P, Result, Self

### Community 209 - "broker/mod.rs"
Cohesion: 0.44
Nodes (10): delayed_open_fill_reaches_lth_before_close_and_last_exit_remains_unfilled(), future_bar_does_not_change_prior_signal(), history(), params(), Value, Vec, run(), session_and_blackout_use_input_minutes() (+2 more)

### Community 210 - "Arc"
Cohesion: 0.28
Nodes (14): is_domestic_symbol(), is_krx_open(), is_krx_open_at(), is_market_open_for(), is_market_open_for_with_calendar(), is_market_open_for_with_calendar_policy(), is_us_open(), is_us_open_at() (+6 more)

### Community 212 - ".new"
Cohesion: 0.39
Nodes (4): first_positive_u64(), KisOutput1<T>, OverseasExecutedOrder, parse_decimal_cents()

### Community 213 - "HashMap"
Cohesion: 0.25
Nodes (6): endpointInventory, EXPECTED_PATHS, hasRateLimitHeaders, missingPaths, paths, servers

### Community 214 - "HeaderMap"
Cohesion: 0.22
Nodes (8): vite.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 215 - "TossAccessToken"
Cohesion: 0.32
Nodes (7): current_position_snapshot(), refresh_manual_positions(), Arc, Mutex, Option, OrderManager, Result

### Community 216 - "place_prepared_order"
Cohesion: 0.50
Nodes (7): place_overseas_with_retry_shared(), place_prepared_order(), place_with_retry_shared(), Arc, KisRestClient, OrderResponse, Result

### Community 218 - "Option"
Cohesion: 0.62
Nodes (6): config(), reconfigure_deviation_applies_new_threshold_to_retained_observations(), reconfigure_momentum_keeps_actual_entry_and_repeat_sell_state(), reconfigure_rsi_recomputes_baseline_with_new_period_and_keeps_targets_separate(), reject_identity_changes(), Value

### Community 226 - "submission.rs"
Cohesion: 0.27
Nodes (10): delete_database_password(), entry(), fail_next_mock_operation_for_tests(), keychain_test_lock(), load_database_password(), MutexGuard, Option, Result (+2 more)

### Community 227 - "config"
Cohesion: 0.25
Nodes (7): KisBrokerAdapter, maps_kis_balance_item_to_broker_holding(), Arc, BalanceItem, BrokerId, KisRestClient, Self

### Community 245 - "StatusCode"
Cohesion: 0.15
Nodes (7): zustand, AccountState, useAccountStore, TradingState, TradingStatus, useTradingStore, zustand

### Community 262 - "config"
Cohesion: 0.48
Nodes (6): config(), live_feedback_breakout_checkpoint_restores_pending_and_confirmed_exact_anchor(), live_feedback_breakout_freezes_pending_signal_anchor_until_first_fill(), live_feedback_consecutive_keeps_confirmed_flag_and_observations(), live_feedback_unsubmitted_breakout_discards_only_candidate_not_history(), Value

### Community 265 - "notional/tests.rs"
Cohesion: 0.33
Nodes (3): RiskManager, NaiveDate, Result

### Community 266 - ".record_fill_pnl_once"
Cohesion: 0.60
Nodes (4): config(), live_feedback_mean_reversion_keeps_entry_and_tick_history_on_sell(), live_feedback_trend_filter_keeps_confirmation_and_updates_indicators(), Value

### Community 267 - "config"
Cohesion: 0.53
Nodes (5): config(), live_feedback_fifty_two_week_keeps_crossing_progress_without_optimistic_holdings(), live_feedback_strong_close_consumes_candidate_only_after_buy_fill(), live_feedback_volatility_keeps_intraday_extremes_and_confirmed_position(), Value

### Community 274 - "commands.rs"
Cohesion: 0.47
Nodes (3): BalanceSnapshot, BalanceStore, HoldingItem

### Community 275 - ".new"
Cohesion: 0.33
Nodes (6): OrderManager, PendingOrder, ReservationDecision, Mutex, submission_is_stale(), SubmissionReservation

### Community 276 - "detail"
Cohesion: 0.83
Nodes (3): confirmed_small_fill_requires_original_order_symbol_side_currency_and_whole_share(), detail(), missing_amount_or_zero_fill_is_deferred_without_using_limit_or_average_as_money()

### Community 278 - "TossTokenState"
Cohesion: 0.31
Nodes (4): LeveragedTrendHoldStrategy, parse_hhmm(), Option, String

### Community 320 - "TossOrderbookView"
Cohesion: 0.25
Nodes (8): finish_closed_batch(), market_readiness(), MarketReadiness, FnOnce, Option, StrategyManager, String, Vec

### Community 323 - "config/mod.rs"
Cohesion: 0.43
Nodes (4): DiscordConfig, load_secure_config(), non_empty(), SecureConfig

### Community 704 - "buffer_tests.rs"
Cohesion: 0.32
Nodes (13): average_fill_price_drives_preview_stop_even_when_observation_price_rises(), daily_context_cannot_prepare_minute_indicators_or_rebound(), daily_reinitialization_keeps_intraday_bollinger_and_actual_position(), failed_buy_feedback_does_not_create_phantom_position(), failed_sell_feedback_keeps_actual_position_for_next_risk_exit(), intraday_and_daily_context_buffers_have_independent_bounds(), params(), partial_exit_snapshot_keeps_the_remaining_quantity_under_risk_management() (+5 more)

### Community 705 - "high_tests.rs"
Cohesion: 0.36
Nodes (12): bar(), complete_day(), completed_high_not_close_sets_next_reference_without_lookahead(), config(), fifty_two_week_requires_full_window_and_uses_last_completed_high_and_close(), initialization_keeps_last_window_and_daily_bookkeeping_is_symbol_scoped(), many_intraday_prices_and_intraday_initializers_do_not_age_daily_highs(), missing_history_becomes_ready_only_after_252_completed_daily_highs() (+4 more)

### Community 707 - "package.json"
Cohesion: 0.25
Nodes (7): engines, node, npm, name, private, type, version

## Knowledge Gaps
- **304 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+299 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **286 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `BrokerScope` connect `compilerOptions` to `vite.config.ts`, `.new`, `shared/api/commands.ts`, `commands/trading.rs`, `ui/AppShell.tsx`, `.process_buy`, `notional/tests.rs`, `OrderSubmission`, `BrokerSymbol`, `.new`, `Client`, `HeaderMap`, `strategy/ui/Page.tsx`, `ui/StockChart.tsx`, `AccountProfile`, `databaseManagementSection.tsx`, `DatabaseManager`, `market/mod.rs`, `server/trading.rs`, `security.rs`, `MutexGuard`, `.new`, `commands/database.rs`, `place_toss_order`, `MarketDayCalendar`, `SubmissionDeps`, `holding`, `toss/orders.rs`, `TokenManager`, `broker/adapter.rs`, `package.json`, `TossAccessToken`, `build.rs`, `sync-codex-skills.ps1`?**
  _High betweenness centrality (0.114) - this node is a cross-community bridge._
- **Why does `remove_file_using_sync()` connect `TossOpenApiClient` to `CurrencyLedger`?**
  _High betweenness centrality (0.069) - this node is a cross-community bridge._
- **Why does `OhlcCandle` connect `devDependencies` to `collect_pages`, `high_tests.rs`, `.new`, `src/api/types.ts`, `ConsecutiveMoveStrategy`, `PositionTracker`, `작업 기록: <짧은 제목>`, `CurrencyLedger`, `Mutex`, `HARNESS_MAP.md`, `broker/mod.rs`, `initialize_active_strategy_history`, `strategy/ui/Page.tsx`, `log-filter/index.ts`, `ui/StockChart.tsx`, `.load_sync`, `place_toss_order`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _304 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `simulation.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.04323936932632585 - nodes in this community are weakly interconnected._
- **Should `hooks.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.12121212121212122 - nodes in this community are weakly interconnected._
- **Should `shared/api/commands.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.10810810810810811 - nodes in this community are weakly interconnected._