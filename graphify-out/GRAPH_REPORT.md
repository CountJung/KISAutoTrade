# Graph Report - KISAutoTrade  (2026-10-18)

## Corpus Check
- 296 files · ~212,667 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 4728 nodes · 9926 edges · 529 communities (178 shown, 351 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 458 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `323727c9`
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
- vite.config.ts
- input
- .new
- .fmt
- .new
- .search
- AppConfigView
- 작업 기록: <짧은 제목>
- .process_buy
- CurrencyLedger
- modify_toss_order_for_profile
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
- MarketDayCalendar
- first_positive_u64
- Self
- config
- Arc
- BalanceItem
- StrategyStore
- config
- strategy/tests.rs
- TossTokenState
- broker/mod.rs
- Arc
- .new
- HashMap
- HeaderMap
- TossAccessToken
- 작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단
- Option
- commands.rs
- PendingOrderStore
- BrokerHoldingView
- BrokerAdapterResult
- BrokerCurrency
- BrokerId
- Default
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
- balance-events.spec.ts
- From
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
- Option
- Result
- DailyStats
- HashMap
- Json
- Option
- Query
- State
- String
- TradeRecord
- Value
- Arc
- HeaderMap
- Option
- Response
- Self
- State
- StatusCode
- Json
- Option
- Path
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
- Vec
- Option
- Path
- PathBuf
- SaveDatabaseConfigInput
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
- NaiveDate
- Path
- PathBuf
- Result
- T
- BrokerId
- Into
- Mutex
- NaiveDate
- Option
- PathBuf
- Result
- Self
- BrokerCurrency
- Option
- String
- Vec
- Default
- HashMap
- I
- DateTime
- Default
- HashMap
- Local
- NaiveDate
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
1. `StrategyConfig` - 113 edges
2. `invoke()` - 92 edges
3. `ServerState` - 81 edges
4. `AppState` - 76 edges
5. `BrokerScope` - 53 edges
6. `AccountProfile` - 50 edges
7. `RiskManager` - 50 edges
8. `OhlcCandle` - 47 edges
9. `Signal` - 42 edges
10. `BrokerSymbol` - 39 edges

## Surprising Connections (you probably didn't know these)
- `maps_holding_to_broker_domain()` --calls--> `BrokerAccountId`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `maps_toss_candle_to_broker_candle()` --calls--> `BrokerSymbol`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `selects_price_response_case_insensitively()` --calls--> `BrokerSymbol`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `validates_market_data_query_limits_before_request()` --calls--> `BrokerSymbol`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `format_toss_error()` --calls--> `body_snippet()`  [INFERRED]
  src-tauri/src/broker/toss/error.rs → src-tauri/src/broker/toss/http.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/trading/simulation/assessment.rs -> src-tauri/src/trading/simulation/assessment.rs`
- 2-file cycle: `src-tauri/src/trading/simulation.rs -> src-tauri/src/trading/simulation/atr.rs -> src-tauri/src/trading/simulation.rs`
- 2-file cycle: `src-tauri/src/server/mod.rs -> src-tauri/src/server/profiles.rs -> src-tauri/src/server/mod.rs`

## Communities (529 total, 351 thin omitted)

### Community 0 - "leveraged_trend_hold.rs"
Cohesion: 0.08
Nodes (22): config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope(), daily_buy_order_limit_is_disabled_even_after_submission_count_increases(), daily_order_limit_isolated_by_broker_account_scope(), daily_sell_order_limit_blocks_after_submission_count_reaches_limit(), DailyOrderCountEntry, DailyOrderSide (+14 more)

### Community 1 - "simulation.rs"
Cohesion: 0.04
Nodes (89): handleLookupTossAccounts(), handleSubmit(), activateEmergencyStop(), addProfile(), checkConfig(), checkForUpdate(), checkTossOrderPreflight(), checkTossProfileConnection() (+81 more)

### Community 2 - "hooks.ts"
Cohesion: 0.09
Nodes (5): MomentumStrategy, Option, HistoryReadiness, bounded_window(), bounded_window_with_extra()

### Community 3 - "shared/api/commands.ts"
Cohesion: 0.44
Nodes (10): order_side_label(), pending_conflict_blocks_opposite_side_in_same_scope(), pending_conflict_blocks_same_side_in_same_scope(), pending_conflict_reason_for_scope(), pending_conflict_scan_ignores_different_scope(), pending_order(), pending_order_conflict_reason(), pending_order_provider_uses_provider_trace() (+2 more)

### Community 4 - "RiskManager"
Cohesion: 0.03
Nodes (84): canUseTauriEvents(), useBackendEvents(), useAutoTradingBudget(), useUpdateAutoTradingBudget(), useKisExecutedByRange(), useOverseasExecutedByRange(), frontendLogger, OVERSEAS_CHART_PRESETS (+76 more)

### Community 5 - "commands/trading.rs"
Cohesion: 0.15
Nodes (22): account_less_scope_matches_any_account_of_same_broker(), default_currency(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), BrokerId, Mutex (+14 more)

### Community 6 - "ConsecutiveMoveStrategy"
Cohesion: 0.05
Nodes (28): config(), FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy, FiftyTwoWeekState, Default, HashMap, Option, Self (+20 more)

### Community 7 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.07
Nodes (18): config(), deviation_counts_current_price_as_last_required_observation(), deviation_history_is_ready_before_first_evaluation_and_bounded(), DeviationParams, DeviationStrategy, momentum_counts_first_comparison_but_excludes_seed_prices(), momentum_history_does_not_create_or_erase_order_state(), MomentumParams (+10 more)

### Community 8 - "PositionTracker"
Cohesion: 0.07
Nodes (21): config(), daily(), MeanReversionParams, MeanReversionState, MeanReversionStrategy, Default, HashMap, Item (+13 more)

### Community 9 - "ui/AppShell.tsx"
Cohesion: 0.09
Nodes (22): BalanceItem, BalanceResponse, BalanceSummary, ChartCandle, ExecutedOrder, first_positive_u64(), KisOutput1, KisOutput1<T> (+14 more)

### Community 10 - "OrderManager"
Cohesion: 0.08
Nodes (25): scripts, build, build:app, build:app:debug, build:web, check:fsd, check:graphify, check:lockfiles (+17 more)

### Community 11 - "submission.rs"
Cohesion: 0.05
Nodes (42): useRecentLogs(), useUpdateCheck(), SettingsState, useSettingsStore, LEVEL_COLORS, Log(), LogLevel, dashboardRoute (+34 more)

### Community 12 - "dashboard/ui/Page.tsx"
Cohesion: 0.08
Nodes (32): TossAccessToken, TossAccessTokenStatus, TossAccount, TossApiResponse, TossCommission, TossCost, TossCurrencyAmount, TossDailyProfitLoss (+24 more)

### Community 13 - "trading/ui/Page.tsx"
Cohesion: 0.12
Nodes (25): get_active_toss_calendar_override(), get_toss_chart_data(), get_toss_chart_data_for_profile(), get_toss_market_calendar(), get_toss_market_calendar_for_profile(), get_toss_market_snapshot(), get_toss_market_snapshot_for_profile(), get_toss_stock_safety() (+17 more)

### Community 14 - "TossOpenApiClient"
Cohesion: 0.21
Nodes (5): MySqlPool, PgPool, DatabaseManager, DatabasePool, safe_export_path()

### Community 15 - "settings.rs"
Cohesion: 0.16
Nodes (17): Future, Output, capture_cycle(), ExecutionCycle, Arc, FnOnce, KisRestClient, Mutex (+9 more)

### Community 16 - "toss/types.rs"
Cohesion: 0.07
Nodes (18): config(), consecutive_counts_current_price_and_uses_active_position_branch(), consecutive_history_preserves_flat_and_existing_positions(), ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation(), failed_breakout_history_retains_actual_breakout_reference() (+10 more)

### Community 17 - "BrokerSymbol"
Cohesion: 0.13
Nodes (22): BrokerAdapterResult, Option, Vec, scope(), broker_scope_serializes_with_camel_case_fields(), BrokerAccountId, BrokerClientOrderId, BrokerCurrency (+14 more)

### Community 18 - "RsiStrategy"
Cohesion: 0.10
Nodes (8): B, TossOpenApiClient, format_toss_error(), read_toss_response_text(), toss_rate_limit_group(), validate_toss_order_id(), validate_toss_symbol(), TossCredentials

### Community 19 - "fills.rs"
Cohesion: 0.23
Nodes (22): adding_a_target_does_not_transfer_other_strategy_or_manual_holdings(), cancelled_waiter_does_not_cancel_durable_save_and_publication(), config(), delayed_save_pauses_ticks_without_locks_and_stop_can_complete(), empty_price_condition_entries_reject_held_removal_without_touching_disk(), empty_saved_profile_does_not_inherit_price_condition_or_lth_entries(), failed_post_save_restore_blocks_all_new_orders_and_reports_persistence_error(), held_effective_target_removal_is_rejected_before_persistence() (+14 more)

### Community 20 - "accounts.rs"
Cohesion: 0.18
Nodes (16): buildTree(), compareNames(), currentDocument, fail(), generateInventory(), listRepositoryFiles(), mode, nextDocument (+8 more)

### Community 21 - "MeanReversionStrategy"
Cohesion: 0.04
Nodes (87): useAppConfig(), useBalance(), useBrokerHoldings(), useOverseasBalance(), useStatsByRange(), useTodayStats(), useTradesByRange(), useExchangeRate() (+79 more)

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
Cohesion: 0.08
Nodes (31): useSetTradeArchiveConfig(), useTradeArchiveConfig(), useTradeArchiveStats(), useLogConfig(), useSetLogConfig(), useSetStockUpdateInterval(), useStockListStats(), useBrokerRateLimitStatus() (+23 more)

### Community 26 - "strategy/ui/Page.tsx"
Cohesion: 0.10
Nodes (15): startup_skips_52_week_only_symbols_but_preserves_other_strategy_history(), build_strategy(), live_high_excludes_unconfirmed_latest_bar_without_shortening_252_requirement(), Box, BrokerId, BTreeMap, Default, F (+7 more)

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
Nodes (59): useRefreshStockList(), useStockSearch(), useTossMarketCalendar(), usePreviewLeveragedTrendHold(), useStrategies(), useUpdateStrategy(), BollingerControls(), boolParam() (+51 more)

### Community 31 - "BrokerScope"
Cohesion: 0.12
Nodes (40): AppConfigView, OverseasBalanceItem, OverseasBalanceSummary, add_profile(), AddProfileInput, apply_active_profile(), BalanceResult, broker_market_sort_key() (+32 more)

### Community 32 - "server/market.rs"
Cohesion: 0.12
Nodes (41): MySql, Postgres, drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract(), postgresql_rejects_invalid_credentials() (+33 more)

### Community 33 - "settings/ui/Page.tsx"
Cohesion: 0.14
Nodes (36): AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_profile_trading_type(), detect_trading_type(), DetectTradingTypeResult (+28 more)

### Community 34 - "KisRestClient"
Cohesion: 0.26
Nodes (13): blocks_reentry_after_stop_loss_sell_on_same_day(), blocks_repeated_same_side_signal_inside_cooldown(), blocks_tiny_profit_after_costs(), blocks_whipsaw_after_repeated_opposite_signals(), buy_signal(), cooldown_is_isolated_by_broker_account_scope(), explicit_market_dates_control_stop_reentry_and_dst_elapsed_cooldown(), live_wrappers_keep_local_calendar_and_instant() (+5 more)

### Community 35 - "AccountProfile"
Cohesion: 0.06
Nodes (66): order_side_label(), OrderManager, OrderSubmission, PendingOrder, current_position_snapshot(), refresh_manual_positions(), Arc, Mutex (+58 more)

### Community 36 - "src/api/types.ts"
Cohesion: 0.15
Nodes (32): broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc(), daily_chart_time() (+24 more)

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
Cohesion: 0.12
Nodes (4): maps_holding_to_broker_domain(), maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request()

### Community 42 - "StrategyConfig"
Cohesion: 0.12
Nodes (16): BalanceSnapshot, BalanceStore, HoldingItem, build_daily_path(), build_monthly_path(), concurrent_writes_leave_one_complete_json_document(), corrupt_json_recovers_from_last_durable_backup(), daily_path_uses_date_components_without_string_separators() (+8 more)

### Community 43 - "domain.rs"
Cohesion: 0.23
Nodes (11): DatabaseConfig, DatabaseConfigView, DatabaseProvider, DatabaseStatusView, DatabaseTableStatus, DatabaseTlsMode, DatabaseTransferResult, JsonStorageCategoryView (+3 more)

### Community 44 - "archive.rs"
Cohesion: 0.21
Nodes (16): useTossMarketSnapshot(), useModifyTossOrder(), useTossOpenOrders(), fmtTossMoney(), fmtTossSession(), shortTossOrderId(), TossMarketCalendarStrip(), tossMarketLabel() (+8 more)

### Community 45 - "commands/records.rs"
Cohesion: 0.27
Nodes (12): collect_trade_archive_stats(), collect_trade_archive_stats_active(), collect_trade_day_dirs(), dir_size_bytes(), get_trade_archive_config(), get_trade_archive_stats(), purge_old_trade_files(), purge_trade_archive_active() (+4 more)

### Community 46 - "PriceConditionStrategy"
Cohesion: 0.13
Nodes (27): fetch_overseas_tick(), fetch_toss_tick(), finish_closed_batch(), market_readiness(), MarketReadiness, FnOnce, Option, StrategyManager (+19 more)

### Community 47 - "detect.rs"
Cohesion: 0.15
Nodes (3): DiscordNotifier, NotificationEvent, NotificationLevel

### Community 48 - "submit_toss_small_buy_verification_for_profile"
Cohesion: 0.18
Nodes (24): collect_kis_pending_fills(), collect_toss_pending_fills(), domestic_order_is_terminal(), domestic_order_terminal_status(), domestic_provider_status(), kis_pending_from(), normalized_execution_u64(), overseas_order_terminal_status() (+16 more)

### Community 49 - "market/mod.rs"
Cohesion: 0.17
Nodes (11): client(), Fixture, profile(), Arc, Drop, KisRestClient, Mutex, OrderManager (+3 more)

### Community 50 - "profiles.rs"
Cohesion: 0.06
Nodes (30): get_positions(), get_strategies(), PositionView, BrokerId, CmdResult, From, Option, Self (+22 more)

### Community 51 - "server/trading.rs"
Cohesion: 0.19
Nodes (13): EditResult, ConfigurationEditReservation, HashSet, ConfigurationIdentity, OrderManager, Arc, HashMap, KisRestClient (+5 more)

### Community 52 - "OrderRecord"
Cohesion: 0.12
Nodes (31): lth_default_adx_period(), lth_default_breakeven_buffer(), lth_default_buy_adx(), lth_default_buy_rsi(), lth_default_ema_long(), lth_default_ema_short(), lth_default_entry_end(), lth_default_entry_failure_observations() (+23 more)

### Community 53 - "StockStore"
Cohesion: 0.08
Nodes (29): File, AppState, balance_summary_total_krw(), CmdError, make_rest_client(), parse_amount_f64(), parse_amount_i64(), Arc (+21 more)

### Community 54 - "security.rs"
Cohesion: 0.20
Nodes (26): FetchFuture, Fn, at_most_two_jobs_run_and_other_instruments_are_not_starved(), completed(), failures_use_backoff_and_do_not_retry_on_every_quote_tick(), history(), insufficient_history_reports_real_count_reason_and_retry(), maintain_captures_the_new_scope_atomically_after_a_waiting_start() (+18 more)

### Community 55 - "dependencies"
Cohesion: 0.19
Nodes (3): StockEntry, StockListStats, StockStore

### Community 56 - "logging/mod.rs"
Cohesion: 0.05
Nodes (78): daily_diagnostic_never_evaluates_strategy_or_execution_even_with_intraday_input(), apply_slippage(), empty_minute_observations_are_not_evaluable(), ReplayAssessmentView, Self, String, Vec, AtrPreparationView (+70 more)

### Community 57 - "active_profile"
Cohesion: 0.13
Nodes (13): Debug, Display, BrokerAdapterResult, BrokerCurrency, BrokerId, Default, Option, Vec (+5 more)

### Community 58 - "compilerOptions"
Cohesion: 0.18
Nodes (15): BudgetCurrencyView, allocation_cannot_withdraw_committed_cash_or_overflow(), AutoTradingBudgetView, budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent(), BudgetAccount, BudgetCurrencyView, BudgetLedger, BudgetReservation (+7 more)

### Community 59 - "commands/database.rs"
Cohesion: 0.14
Nodes (18): Barrier, daily_order_limit_key(), OrderManager, PendingOrder, Arc, HashMap, HashSet, KisRestClient (+10 more)

### Community 60 - "commands/market.rs"
Cohesion: 0.15
Nodes (11): market_from_currency(), toss_currency(), toss_market(), validate_client_order_id(), validate_optional_decimal(), validate_order_side(), validate_order_type(), validate_time_in_force() (+3 more)

### Community 61 - "DiscordNotifier"
Cohesion: 0.10
Nodes (29): useClearDatabaseTables(), useCreateDatabaseTables(), useDatabaseConfig(), useDropDatabaseTables(), useExportDatabaseToJson(), useImportJsonToDatabase(), useJsonStorageInventory(), useSaveDatabaseConfig() (+21 more)

### Community 62 - "DatabaseConfig"
Cohesion: 0.13
Nodes (16): MySqlSslMode, PgSslMode, category_from_key(), connect_pool(), create_missing_mariadb_database(), create_missing_postgres_database(), document_key(), inventory_from_documents() (+8 more)

### Community 63 - "DatabasePool"
Cohesion: 0.13
Nodes (20): OverseasPriceResponse, kis_read_min_interval(), KisOrderRejected, KisRestClient, read_kis_response_text(), Arc, AtomicBool, Duration (+12 more)

### Community 64 - "TradeRecord"
Cohesion: 0.22
Nodes (6): clear_database_tables(), drop_database_tables(), ensure_trading_stopped(), export_database_to_json(), import_json_to_database(), set_storage_backend()

### Community 65 - "release-version.mjs"
Cohesion: 0.17
Nodes (25): decimal_quantity_units(), money_units(), normalized_f64(), normalized_u64(), BrokerCurrency, BrokerId, Json, Option (+17 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.13
Nodes (24): assert_auth(), candle_values(), chart_response_handles_empty_output_and_malformed_multibyte_body(), chart_response_preserves_http_and_provider_errors(), chart_response_propagates_announced_and_observed_body_caps(), domestic_latest_first_fixture_reverses_and_maps_ohlcv(), domestic_request_keeps_date_bounds_period_adjustment_and_query_order(), overseas_dailyprice_request_uses_its_own_tr_id_and_period_codes() (+16 more)

### Community 67 - "scripts"
Cohesion: 0.14
Nodes (43): activate_emergency_handler(), app_config_handler(), check_config_handler(), check_update_handler(), clear_buy_suspension_handler(), clear_emergency_handler(), exchange_rate_handler(), exchange_rate_status_handler() (+35 more)

### Community 68 - "place_toss_order"
Cohesion: 0.15
Nodes (22): ConfigurationUpdateError, ConfigurationUpdateRequest, ConfigurationUpdateState, effective_target_markets(), Arc, HashMap, Into, JoinHandle (+14 more)

### Community 69 - "conflicts.rs"
Cohesion: 0.05
Nodes (93): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, BrokerCurrency, CmdResult, Option (+85 more)

### Community 71 - "commands/risk.rs"
Cohesion: 0.13
Nodes (26): absent_unknown_and_similarly_prefixed_kis_tr_ids_preserve_pending_without_querying(), actual_fill_crosses_same_scope_restart_with_identical_provider_snapshot(), delayed_reconciliation_cannot_mutate_new_scope_and_applies_when_original_is_reselected(), direct_fill_and_zero_fill_terminal_cannot_mutate_another_execution_scope(), exact_eight_order_tr_ids_select_mode_and_toss_bypasses_kis_mode_classification(), executed(), persisted_kis_pending_never_queries_or_applies_in_the_opposite_mode(), Provider (+18 more)

### Community 72 - ".new"
Cohesion: 0.25
Nodes (25): add_profile_handler(), AddProfileBody, apply_profile_change(), default_body_broker_id(), delete_profile_handler(), DeleteProfileBody, detect_profile_handler(), detect_trading_type_handler() (+17 more)

### Community 73 - "storage/database.rs"
Cohesion: 0.04
Nodes (69): usePreviewStrategy(), ProfileFormState, assertPageScope(), CandleCollection, CandleCollectionEnd, candleTimestamp(), checkCancelled(), collectPreviewCandles() (+61 more)

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
Cohesion: 0.32
Nodes (4): keychain_test_dirs(), legacy_file_password_migrates_to_keychain_on_load(), save_config_stores_password_in_keychain_not_in_file(), save_input_with_password()

### Community 78 - "devDependencies"
Cohesion: 0.15
Nodes (14): lth_preview_signals_for_interval(), F, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldTimedCandle, lth_default_qty(), LeveragedTrendHoldStrategy, PreviewPosition, F (+6 more)

### Community 79 - "project-map.mjs"
Cohesion: 0.20
Nodes (13): ChartRequest, ChartRequest<'a>, KisRestClient, parse_domestic_response(), parse_overseas_response(), ChartCandle, Client, HeaderMap (+5 more)

### Community 80 - "TokenManager"
Cohesion: 0.13
Nodes (23): fetch_account_risk_balance_krw(), fetch_toss_risk_balance_krw(), Arc, KisRestClient, Result, String, clear_buy_suspension(), failed_configuration_pause_never_becomes_market_close_and_same_revision_resumes() (+15 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.09
Nodes (10): PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick(), HashMap, Option, Self, String (+2 more)

### Community 82 - ".write_document"
Cohesion: 0.12
Nodes (12): config(), initializer_restores_previous_averages_for_first_evaluation_cross(), insufficient_history_has_no_previous_long_average(), MaCrossParams, MaCrossState, MovingAverageCrossStrategy, Default, HashMap (+4 more)

### Community 83 - "KisBrokerAdapter"
Cohesion: 0.08
Nodes (46): react, AccountConfigRefreshError, accountScope(), holdingsPrefixes, profileMutationKey, profileMutationScope, reconcileRejectedProfileMutation(), refreshAccountConfig() (+38 more)

### Community 84 - "support.rs"
Cohesion: 0.18
Nodes (3): profile_fingerprint(), AccountProfile, ProfilesConfig

### Community 85 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.21
Nodes (10): AppConfigView, brokerLabel(), BrokerScopeIndicator(), BrokerScopeIndicatorProps, HistoryRow(), LiveDailyHistoryPanel(), Props, retryTime() (+2 more)

### Community 86 - "broker/adapter.rs"
Cohesion: 0.18
Nodes (15): PendingCandidate, PendingFill, OrderManager, ReconciliationSnapshot, Arc, BrokerId, KisRestClient, Mutex (+7 more)

### Community 87 - "ui/LayoutResizer.tsx"
Cohesion: 0.13
Nodes (15): lightweight-charts, lightweight-charts, useOverseasChartData(), buildChartOptions(), CandlePoint, ChartType, CrosshairData, LinePoint (+7 more)

### Community 88 - "initialize_active_strategy_history"
Cohesion: 0.16
Nodes (26): LeveragedTrendHoldEntry, LeveragedTrendHoldStrategy, reconfigure_lth_keeps_exact_protection_lifecycle_and_uses_new_exit_parameters(), reconfigure_lth_keeps_initial_stop_without_new_window_warmup(), reconfigure_lth_trims_raw_observations_to_new_caps_without_reconstructing_position(), buys_any_target_ticker_when_itself_trends_up(), buys_intraday_rebound_when_next_window_shows_buy_pressure(), buys_rapid_rebound_without_trend_snapshot_when_enabled() (+18 more)

### Community 89 - "PendingOrderStore"
Cohesion: 0.16
Nodes (8): calculate_domestic_fee(), calculate_overseas_fee_cents(), cumulative_fill_delta(), cumulative_fill_delta_price(), OrderManager, NaiveDate, Option, Result

### Community 90 - "StrategyStore"
Cohesion: 0.61
Nodes (7): parse_holding_price(), parse_holding_u64(), parse_holding_usd_cents(), CmdResult, sync_kis_strategy_positions(), sync_strategy_positions_from_active_broker(), sync_toss_strategy_positions()

### Community 91 - "TradeStore"
Cohesion: 0.15
Nodes (41): account_sync_error(), balance_handler(), broker_holdings_handler(), candle_page_handler(), chart_handler(), ChartQuery, executed_handler(), invalid_order_input() (+33 more)

### Community 92 - "database_keychain.rs"
Cohesion: 0.43
Nodes (4): delete_database_password(), entry(), load_database_password(), store_database_password()

### Community 93 - "@mui/icons-material"
Cohesion: 0.47
Nodes (4): check(), GitHubRelease, is_newer(), UpdateInfo

### Community 95 - ".load_sync"
Cohesion: 0.14
Nodes (16): LeveragedRapidReboundSnapshot, LeveragedReboundSnapshot, LeveragedTrendHoldEntry, LeveragedTrendHoldMarketState, LeveragedTrendHoldParams, LeveragedTrendHoldPosition, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldStrategy (+8 more)

### Community 96 - "check"
Cohesion: 0.08
Nodes (23): DOM, DOM.Iterable, ES2020, src, compilerOptions, allowImportingTsExtensions, isolatedModules, jsx (+15 more)

### Community 97 - "compilerOptions"
Cohesion: 0.60
Nodes (3): create(), mariadb_ddl(), postgres_ddl()

### Community 98 - "check-fsd-imports.mjs"
Cohesion: 0.40
Nodes (5): copy_dir_all(), Path, Result, run(), install_database_manager()

### Community 99 - "package.json"
Cohesion: 0.43
Nodes (4): DiscordConfig, load_secure_config(), non_empty(), SecureConfig

### Community 100 - "verify-toss-openapi.mjs"
Cohesion: 0.13
Nodes (15): devDependencies, @playwright/test, @tauri-apps/cli, @types/react, @types/react-dom, @types/react-grid-layout, typescript, @vitejs/plugin-react (+7 more)

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
Cohesion: 0.09
Nodes (22): @emotion/react, @emotion/styled, @mui/icons-material, @mui/material, dependencies, @emotion/react, @emotion/styled, @mui/icons-material (+14 more)

### Community 106 - "TossApiError"
Cohesion: 0.18
Nodes (12): AccessToken, Arc, Client, DateTime, Mutex, Option, Result, Self (+4 more)

### Community 116 - "build.rs"
Cohesion: 0.17
Nodes (13): RecentSideHistory, ScopedSymbol, GuardDecision, GuardSide, is_stop_loss_reason(), DateTime, FixedOffset, HashMap (+5 more)

### Community 118 - "settings-database.spec.ts"
Cohesion: 0.50
Nodes (3): TossApiError, TossErrorResponse, TossHttpError

### Community 120 - "playwright.config.ts"
Cohesion: 0.17
Nodes (9): KisBrokerAdapter, maps_kis_balance_item_to_broker_holding(), Arc, BalanceItem, BrokerId, KisRestClient, Self, daily_diagnostic_contains_one_completed_bar_and_no_synthetic_entry_observation() (+1 more)

### Community 121 - "sync-codex-skills.ps1"
Cohesion: 0.25
Nodes (18): atomic_write(), atomic_write_private(), atomic_write_private_sync(), backup_failure_preserves_primary_and_cleans_staged_content(), durable_atomic_write(), locked_destination_preserves_content_and_allows_retry_after_release(), private_async_replacements_keep_owner_only_permissions(), private_sync_replacements_keep_owner_only_permissions() (+10 more)

### Community 122 - "discord-notification-config/index.ts"
Cohesion: 0.21
Nodes (10): BrokerAdapter, BrokerAdapterError, default_place_order_returns_unsupported_with_broker_id(), DummyAdapter, BrokerAdapterResult, BrokerId, Error, Send (+2 more)

### Community 124 - "strategy-configure/index.ts"
Cohesion: 0.21
Nodes (13): Next, Router, app(), bearer_token(), constant_time_eq(), cross_origin_mutation_is_rejected_even_with_token(), has_same_origin(), mutation_without_token_is_rejected() (+5 more)

### Community 125 - "strategy-toggle/index.ts"
Cohesion: 0.06
Nodes (65): PreviewRow, chart_volume_to_u64(), blocked_strong_close_buy_does_not_discard_next_days_completed_condition(), candle(), daily_event_strategies_reject_other_cadences(), flat_completed_days_fill_history_without_compressing_the_window(), input(), ChartCandle (+57 more)

### Community 129 - "input"
Cohesion: 0.67
Nodes (5): config(), reconfigure_consecutive_preserves_holding_and_sell_observations(), reconfigure_failed_breakout_keeps_exact_reference_when_new_history_is_short(), reject_scope_changes(), Value

### Community 130 - ".new"
Cohesion: 0.18
Nodes (13): DailyHistoryEntry, DailyHistoryStatus, DailyHistoryTicket, fingerprint(), pending_status(), BrokerId, DateTime, NaiveDate (+5 more)

### Community 131 - ".fmt"
Cohesion: 0.43
Nodes (5): OrderManager, PendingOrder, Result, validate_pending_kis_mode(), pending_order_provider()

### Community 132 - ".new"
Cohesion: 0.17
Nodes (10): AsyncMutex, shared_toss_token_state(), toss_token_state_key(), TossTokenState, body_snippet(), toss_http_client(), trim_base_url(), url_encode() (+2 more)

### Community 133 - ".search"
Cohesion: 0.19
Nodes (16): LeveragedTrendHoldPreviewInput, TossModifyOrderInput, TossOpenOrdersInput, TossOrderPreflightInput, TossSmallBuyVerificationInput, active_profile(), leveraged_trend_hold_preview_handler(), toss_chart_handler() (+8 more)

### Community 135 - "AppConfigView"
Cohesion: 0.20
Nodes (4): OrderManager, Option, String, TradingHealthStatus

### Community 136 - "작업 기록: <짧은 제목>"
Cohesion: 0.14
Nodes (19): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), calculate_atr(), initialize_active_strategy_history(), legacy_history_symbols(), Arc (+11 more)

### Community 138 - ".process_buy"
Cohesion: 0.30
Nodes (10): estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), OrderManager, Option, OrderResponse, OrderSide, Result (+2 more)

### Community 139 - "CurrencyLedger"
Cohesion: 0.32
Nodes (5): LeveragedTrendHoldStrategy, Option, Vec, VecDeque, LeveragedTrendSnapshot

### Community 141 - "modify_toss_order_for_profile"
Cohesion: 0.31
Nodes (8): market_closed_first_order_settles_remaining_raw_batch_without_touching_submitted_state(), old_market_closed_batch_cannot_restore_positions_into_restarted_run(), Arc, Fixture, Mutex, StrategyManager, strategies(), unsubmitted_feedback_uses_the_latest_holding_at_the_apply_gate()

### Community 143 - "HARNESS_MAP.md"
Cohesion: 0.18
Nodes (14): Band, BollingerParams, candle(), disabled_option_preserves_entries_and_buffer_is_bounded(), downward_breakout_and_insufficient_history_block_entries(), flat_squeeze_needs_completed_upward_breakout(), LeveragedTrendHoldStrategy, live_and_replay_share_confirmed_breakout_and_exit_without_future_prices() (+6 more)

### Community 145 - "react-grid-layout"
Cohesion: 0.18
Nodes (12): buildSnapshot(), fail(), listRepositoryFiles(), mode, parseMode(), REPOSITORY_ROOT, ROOT_CONFIGS, SCRIPT_DIR (+4 more)

### Community 148 - "Arc"
Cohesion: 0.25
Nodes (13): bodyHtml(), checkFreshness(), escapeHtml(), inlineMarkdown(), outputPath, parseTasks(), renderTodo(), root (+5 more)

### Community 153 - "ExecutedOrder"
Cohesion: 0.18
Nodes (11): usd_to_cents(), StockListStats, ChartDataInput, get_chart_data(), get_overseas_price(), get_price(), get_stock_list_stats(), OverseasOrderInput (+3 more)

### Community 157 - "PriceResponse"
Cohesion: 0.12
Nodes (20): DailyHistoryJobs, JobEntry, retry_delay(), Arc, BTreeMap, DateTime, Drop, Duration (+12 more)

### Community 158 - "Response"
Cohesion: 0.29
Nodes (4): LeveragedTrendHoldStrategy, parse_hhmm(), Option, String

### Community 159 - "place_toss_order"
Cohesion: 0.10
Nodes (17): config(), reconfigure_deviation_applies_new_threshold_to_retained_observations(), reconfigure_momentum_keeps_actual_entry_and_repeat_sell_state(), reconfigure_rsi_recomputes_baseline_with_new_period_and_keeps_targets_separate(), reject_identity_changes(), Value, Send, Sync (+9 more)

### Community 160 - "collect_pages"
Cohesion: 0.14
Nodes (28): CandlePageView, collect_daily_history(), collect_pages(), CollectedDailyHistory, completed_bar(), price(), Arc, ChartCandle (+20 more)

### Community 163 - "ExchangeRateView"
Cohesion: 0.23
Nodes (9): ExchangeRateSource, ExchangeRateView, get_exchange_rate(), refresh_exchange_rate_status(), resolve_exchange_rate_policy(), Arc, Result, RwLock (+1 more)

### Community 164 - "collect_daily_history"
Cohesion: 0.19
Nodes (17): accepted_http_response_publishes_before_its_event_under_the_same_gate(), balance(), delayed_http_a_to_b_to_a_response_never_publishes_or_emits(), delayed_http_response_cannot_undo_a_concurrent_buy_or_sell(), invalid_holdings_do_not_clear_the_previous_position_or_emit_success(), refresh(), Arc, AtomicUsize (+9 more)

### Community 175 - "Mutex"
Cohesion: 0.38
Nodes (12): candle(), config(), daily_hooks_do_not_create_unknown_symbol_states(), Value, strong_close_completed_day_uses_actual_position_and_keeps_stop_loss(), strong_close_uses_each_completed_day_only_on_the_following_day(), volatility(), volatility_collects_missing_history_after_close_for_later_days() (+4 more)

### Community 181 - "AppHandle"
Cohesion: 0.18
Nodes (9): args, artifactDir, artifacts, checksumPath, manifestPath, names, records, requireSignatures (+1 more)

### Community 184 - ".new"
Cohesion: 0.26
Nodes (9): ExecutionRun, ExecutionRunContext, OrderManager, Arc, KisRestClient, Option, Result, String (+1 more)

### Community 190 - "Vec"
Cohesion: 0.24
Nodes (10): AutoTradingBudgetInput, UpdateAutoTradingBudgetInput, AutoTradingBudgetInput, checked_budget_scope(), get_auto_trading_budget(), update_auto_trading_budget(), UpdateAutoTradingBudgetInput, budget_handler() (+2 more)

### Community 192 - "modify_toss_order_for_profile"
Cohesion: 0.23
Nodes (13): chartTimeToMs(), formatChartTimeKst(), formatKstMs(), pad2(), parseTimeMs(), Props, StrategyPreviewChart(), StrategyPreviewChartSignal (+5 more)

### Community 193 - "order/configuration/tests.rs"
Cohesion: 0.18
Nodes (6): toss_market_session_window(), calendar_override_represents_holiday_without_fallback(), calendar_session_window_checks_time_range(), MarketDayCalendar, MarketSessionWindow, us_session_policy_checks_extended_sessions()

### Community 194 - "commands/risk.rs"
Cohesion: 0.36
Nodes (12): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+4 more)

### Community 196 - "MarketDayCalendar"
Cohesion: 0.12
Nodes (11): TossOrder, TossOrderCreateRequest, TossOrderExecution, TossOrderListQuery, TossOrderListStatus, TossOrderModifyRequest, TossOrderResponse, TossPaginatedOrderResponse (+3 more)

### Community 198 - "first_positive_u64"
Cohesion: 0.31
Nodes (16): HoldingsUpdate, a_change_to_either_market_rejects_both_without_partial_publication(), configuration_lease_rejects_capture_and_inflight_response_even_after_release(), domestic(), empty_and_zero_holdings_clear_only_the_queried_market(), fixture(), held(), identity_changes_reject_without_mutating_either_tracker() (+8 more)

### Community 199 - "Self"
Cohesion: 0.40
Nodes (3): LeveragedTrendHoldStrategy, Option, String

### Community 200 - "config"
Cohesion: 0.62
Nodes (6): config(), reconfigure_fifty_two_week_keeps_actual_entry_while_invalidating_old_window(), reconfigure_strong_close_preserves_stop_but_discards_old_entry_candidate(), reconfigure_volatility_recomputes_range_without_losing_held_stop_or_day(), reject_scope_changes(), Value

### Community 202 - "Arc"
Cohesion: 0.22
Nodes (5): layerRank, layers, root, srcRoot, violations

### Community 203 - "BalanceItem"
Cohesion: 0.22
Nodes (6): appEntry, escapedName, npmLock, pkg, rootDir, workspaceLock

### Community 205 - "config"
Cohesion: 0.53
Nodes (5): config(), empty_price_condition_entries_remove_stale_targets_before_manager_validation(), reconfigure_price_condition_canonicalizes_targets_and_keeps_actual_stop(), reconfigure_price_condition_does_not_move_holdings_to_replacement_target(), Value

### Community 206 - "strategy/tests.rs"
Cohesion: 0.15
Nodes (6): restore_risk_from_today_trades(), Arc, CmdResult, Mutex, RiskStore, RiskConfigState

### Community 208 - "TossTokenState"
Cohesion: 0.23
Nodes (12): HoldingsUpdate, money(), OrderManager, quantity(), Arc, Mutex, Option, P (+4 more)

### Community 209 - "broker/mod.rs"
Cohesion: 0.44
Nodes (9): future_bar_does_not_change_prior_signal(), history(), params(), Value, Vec, run(), session_and_blackout_use_input_minutes(), timed() (+1 more)

### Community 210 - "Arc"
Cohesion: 0.28
Nodes (14): is_domestic_symbol(), is_krx_open(), is_krx_open_at(), is_market_open_for(), is_market_open_for_with_calendar(), is_market_open_for_with_calendar_policy(), is_us_open(), is_us_open_at() (+6 more)

### Community 212 - ".new"
Cohesion: 0.60
Nodes (3): Into, Self, String

### Community 213 - "HashMap"
Cohesion: 0.25
Nodes (6): endpointInventory, EXPECTED_PATHS, hasRateLimitHeaders, missingPaths, paths, servers

### Community 214 - "HeaderMap"
Cohesion: 0.22
Nodes (8): vite.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 218 - "Option"
Cohesion: 0.43
Nodes (5): DatabaseArchiveStats, purge(), stats(), trade_date(), trade_rows()

### Community 245 - "StatusCode"
Cohesion: 0.15
Nodes (7): zustand, AccountState, useAccountStore, TradingState, TradingStatus, useTradingStore, zustand

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
- **297 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+292 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **351 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ServerState` connect `scripts` to `leveraged_trend_hold.rs`, `release-version.mjs`, `commands/trading.rs`, `toss/tests.rs`, `.new`, `StrategyStore`, `TossOpenApiClient`, `detect.rs`, `strategy/tests.rs`, `profiles.rs`, `support.rs`, `StockStore`, `dependencies`, `TradeStore`, `strategy-configure/index.ts`?**
  _High betweenness centrality (0.094) - this node is a cross-community bridge._
- **Why does `StrategyConfig` connect `ConsecutiveMoveStrategy` to `input`, `hooks.ts`, `.new`, `leveragedTrendHoldEditorPanel.tsx`, `PositionTracker`, `toss/types.rs`, `fills.rs`, `strategy/ui/Page.tsx`, `commands/toss.rs`, `place_toss_order`, `DatabaseManager`, `PriceConditionStrategy`, `Mutex`, `server/trading.rs`, `high_tests.rs`, `place_toss_order`, `config`, `config`, `BrokerQuantity`, `.write_document`, `.load_sync`, `log-filter/index.ts`?**
  _High betweenness centrality (0.082) - this node is a cross-community bridge._
- **Why does `AppState` connect `StockStore` to `leveraged_trend_hold.rs`, `commands/trading.rs`, `TossOpenApiClient`, `database_projection.rs`, `BrokerScope`, `settings/ui/Page.tsx`, `ExchangeRateView`, `src/api/types.ts`, `StrategyConfig`, `detect.rs`, `profiles.rs`, `dependencies`, `conflicts.rs`, `toss/tests.rs`, `StrategyStore`, `strategy/tests.rs`, `TokenManager`, `support.rs`, `StrategyStore`, `package.json`?**
  _High betweenness centrality (0.079) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _297 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `leveraged_trend_hold.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0750151240169389 - nodes in this community are weakly interconnected._
- **Should `simulation.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.04371184371184371 - nodes in this community are weakly interconnected._
- **Should `hooks.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.09475806451612903 - nodes in this community are weakly interconnected._