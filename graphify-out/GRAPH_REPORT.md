# Graph Report - KISAutoTrade  (2026-10-18)

## Corpus Check
- 276 files · ~199,298 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 4447 nodes · 9221 edges · 531 communities (178 shown, 353 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 431 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2c593de3`
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
- symbol-search/index.ts
- trading-start-stop/index.ts
- vite.config.ts
- input
- .new
- .fmt
- .new
- .search
- @tauri-apps/api
- AppConfigView
- 작업 기록: <짧은 제목>
- log/ui/Page.tsx
- .process_buy
- CurrencyLedger
- modify_toss_order_for_profile
- 10. 완료 보고
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
- refresh_manual_positions
- String
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
- evaluate_order_preflight
- .new
- FixedOffset
- RwLock
- Self
- Sender
- String
- Vec
- BrokerAdapterResult
- modify_toss_order_for_profile
- Self
- Arc
- BalanceItem
- broker/mod.rs
- Arc
- HashMap
- HeaderMap
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
- Path
- PathBuf
- Result
- Self
- Vec
- String
- Vec
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
1. `invoke()` - 92 edges
2. `StrategyConfig` - 85 edges
3. `ServerState` - 81 edges
4. `AppState` - 76 edges
5. `RiskManager` - 50 edges
6. `OhlcCandle` - 47 edges
7. `BrokerScope` - 45 edges
8. `AccountProfile` - 44 edges
9. `Signal` - 41 edges
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
- `parse_domestic_response()` --calls--> `read_kis_response_text()`  [INFERRED]
  src-tauri/src/api/rest/chart.rs → src-tauri/src/api/rest.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/trading/simulation/assessment.rs -> src-tauri/src/trading/simulation/assessment.rs`
- 2-file cycle: `src-tauri/src/trading/simulation.rs -> src-tauri/src/trading/simulation/atr.rs -> src-tauri/src/trading/simulation.rs`
- 2-file cycle: `src-tauri/src/server/mod.rs -> src-tauri/src/server/profiles.rs -> src-tauri/src/server/mod.rs`

## Communities (531 total, 353 thin omitted)

### Community 0 - "leveraged_trend_hold.rs"
Cohesion: 0.08
Nodes (21): config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope(), daily_buy_order_limit_is_disabled_even_after_submission_count_increases(), daily_order_limit_isolated_by_broker_account_scope(), daily_sell_order_limit_blocks_after_submission_count_reaches_limit(), DailyOrderCountEntry, DailyOrderSide (+13 more)

### Community 1 - "simulation.rs"
Cohesion: 0.05
Nodes (79): handleLookupTossAccounts(), handleSubmit(), activateEmergencyStop(), addProfile(), checkConfig(), checkForUpdate(), checkTossOrderPreflight(), checkTossProfileConnection() (+71 more)

### Community 2 - "hooks.ts"
Cohesion: 0.04
Nodes (78): usePreviewLeveragedTrendHold(), usePreviewStrategy(), ProfileFormState, assertPageScope(), CandleCollection, CandleCollectionEnd, candleTimestamp(), checkCancelled() (+70 more)

### Community 3 - "shared/api/commands.ts"
Cohesion: 0.44
Nodes (10): order_side_label(), pending_conflict_blocks_opposite_side_in_same_scope(), pending_conflict_blocks_same_side_in_same_scope(), pending_conflict_reason_for_scope(), pending_conflict_scan_ignores_different_scope(), pending_order(), pending_order_conflict_reason(), pending_order_provider_uses_provider_trace() (+2 more)

### Community 4 - "RiskManager"
Cohesion: 0.04
Nodes (78): canUseTauriEvents(), useBackendEvents(), useAutoTradingBudget(), useUpdateAutoTradingBudget(), useKisExecutedByRange(), useOverseasExecutedByRange(), frontendLogger, OVERSEAS_CHART_PRESETS (+70 more)

### Community 5 - "commands/trading.rs"
Cohesion: 0.14
Nodes (24): account_less_scope_matches_any_account_of_same_broker(), default_currency(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), BrokerId, Mutex (+16 more)

### Community 6 - "ConsecutiveMoveStrategy"
Cohesion: 0.05
Nodes (19): config(), FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy, FiftyTwoWeekState, Default, HashMap, Option, Self (+11 more)

### Community 7 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.06
Nodes (21): config(), deviation_counts_current_price_as_last_required_observation(), deviation_history_is_ready_before_first_evaluation_and_bounded(), DeviationParams, DeviationStrategy, momentum_counts_first_comparison_but_excludes_seed_prices(), momentum_history_does_not_create_or_erase_order_state(), MomentumParams (+13 more)

### Community 8 - "PositionTracker"
Cohesion: 0.06
Nodes (23): config(), daily(), MeanReversionParams, MeanReversionState, MeanReversionStrategy, Default, HashMap, Item (+15 more)

### Community 9 - "ui/AppShell.tsx"
Cohesion: 0.09
Nodes (20): BalanceItem, BalanceResponse, BalanceSummary, ChartCandle, ExecutedOrder, first_positive_u64(), KisOutput1, KisOutput1<T> (+12 more)

### Community 10 - "OrderManager"
Cohesion: 0.08
Nodes (25): scripts, build, build:app, build:app:debug, build:web, check:fsd, check:graphify, check:lockfiles (+17 more)

### Community 11 - "submission.rs"
Cohesion: 0.05
Nodes (41): react, useRecentLogs(), useUpdateCheck(), queryClient, LEVEL_COLORS, Log(), LogLevel, dashboardRoute (+33 more)

### Community 12 - "dashboard/ui/Page.tsx"
Cohesion: 0.06
Nodes (38): market_from_currency(), toss_currency(), toss_market(), TossAccessToken, TossAccessTokenStatus, TossAccount, TossApiResponse, TossBuyingPower (+30 more)

### Community 13 - "trading/ui/Page.tsx"
Cohesion: 0.12
Nodes (25): get_active_toss_calendar_override(), get_toss_chart_data(), get_toss_chart_data_for_profile(), get_toss_market_calendar(), get_toss_market_calendar_for_profile(), get_toss_market_snapshot(), get_toss_market_snapshot_for_profile(), get_toss_stock_safety() (+17 more)

### Community 14 - "TossOpenApiClient"
Cohesion: 0.21
Nodes (5): MySqlPool, PgPool, DatabaseManager, DatabasePool, safe_export_path()

### Community 15 - "settings.rs"
Cohesion: 0.28
Nodes (17): page_view(), bar(), candle_page_wire_query_preserves_camel_case_and_encoded_cursor(), completed_daily_pages_drop_current_and_future_bars_and_deduplicate_before_replay(), completed_provider_minute_page_runs_through_production_replay(), current_week_and_month_are_conservatively_unconfirmed(), cursors_bind_scope_and_cutoff_and_reject_future_or_changed_contract(), daily_timestamps_use_market_date_and_reject_digit_prefix_garbage() (+9 more)

### Community 16 - "toss/types.rs"
Cohesion: 0.08
Nodes (18): config(), consecutive_counts_current_price_and_uses_active_position_branch(), consecutive_history_preserves_flat_and_existing_positions(), ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation(), failed_breakout_history_retains_actual_breakout_reference() (+10 more)

### Community 17 - "BrokerSymbol"
Cohesion: 0.11
Nodes (28): default_place_order_returns_unsupported_with_broker_id(), BrokerAdapterResult, Option, Vec, daily_diagnostic_contains_one_completed_bar_and_no_synthetic_entry_observation(), minute_replay_warmup_excludes_replay_day_and_future_daily_candles(), broker_scope_serializes_with_camel_case_fields(), BrokerAccountId (+20 more)

### Community 18 - "RsiStrategy"
Cohesion: 0.17
Nodes (3): TossOpenApiClient, validate_toss_order_id(), TossCredentials

### Community 19 - "fills.rs"
Cohesion: 0.06
Nodes (41): AtomicU64, Future, Output, capture_cycle(), ExecutionCycle, Arc, FnOnce, KisRestClient (+33 more)

### Community 20 - "accounts.rs"
Cohesion: 0.18
Nodes (16): buildTree(), compareNames(), currentDocument, fail(), generateInventory(), listRepositoryFiles(), mode, nextDocument (+8 more)

### Community 21 - "MeanReversionStrategy"
Cohesion: 0.13
Nodes (19): useStatsByRange(), useTradesByRange(), usePendingOrders(), FilledOrdersPanel(), fmt(), PendingOrdersPanel(), SortDir, todayStr() (+11 more)

### Community 22 - "database_projection.rs"
Cohesion: 0.17
Nodes (27): CandlePageInput, CandlePageView, completed_before(), decode_cursor(), domain_request(), get_candle_page(), get_candle_page_for_profile(), normalize_input() (+19 more)

### Community 23 - "toss_market.rs"
Cohesion: 0.11
Nodes (30): IntoIterator, applies_retry_after_to_group_pause(), epoch_ms_now(), parse_delay_header(), rate_limit_reset_delay(), RateLimitGroupState, RateLimitGroupStatus, RateLimitScheduler (+22 more)

### Community 24 - "ServerState"
Cohesion: 0.05
Nodes (40): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, Korean, app, security (+32 more)

### Community 25 - "RateLimitScheduler"
Cohesion: 0.07
Nodes (38): useSetTradeArchiveConfig(), useTradeArchiveConfig(), useTradeArchiveStats(), useLogConfig(), useSetLogConfig(), useSetStockUpdateInterval(), useStockListStats(), useBrokerRateLimitStatus() (+30 more)

### Community 26 - "strategy/ui/Page.tsx"
Cohesion: 0.07
Nodes (22): live_tick_and_preview_emit_identical_signals_for_normalized_fixture(), startup_skips_52_week_only_symbols_but_preserves_other_strategy_history(), BrokerPositionSnapshot, initialize_strategy_warmup(), BrokerMarket, Send, Sync, Strategy (+14 more)

### Community 27 - "commands/toss.rs"
Cohesion: 0.18
Nodes (12): client(), Fixture, profile(), Arc, Drop, KisRestClient, Mutex, OrderManager (+4 more)

### Community 28 - "tauri.conf.json"
Cohesion: 0.16
Nodes (13): classify_token_response(), detect_trading_type(), DetectedTradingType, DetectTokenReq, DetectTradingTypeError, is_paper_key_rejected_by_real_domain(), is_real_key_rejected_by_paper_domain(), mentions_app_key() (+5 more)

### Community 29 - "ui/StockChart.tsx"
Cohesion: 0.20
Nodes (31): apply(), bars(), completed_252_keeps_final_high_and_uses_high_instead_of_close(), config_aba_and_profile_scope_switch_cannot_revive_old_tickets(), configured_512_window_and_invalid_ohlc_are_not_mislabeled_ready(), current_day_future_missing_and_stale_last_completed_dates_block_readiness(), daily_refresh_never_calls_unrelated_leveraged_intraday_initializers(), disabled_targets_and_config_mutation_remove_ready_history() (+23 more)

### Community 30 - "StrategyManager"
Cohesion: 0.07
Nodes (47): useRefreshStockList(), useStockSearch(), useTossMarketCalendar(), useStrategies(), useUpdateStrategy(), hasInvalidLthEntries(), addSymbol(), brokerLabel() (+39 more)

### Community 31 - "BrokerScope"
Cohesion: 0.12
Nodes (40): AppConfigView, OverseasBalanceItem, OverseasBalanceSummary, add_profile(), AddProfileInput, apply_active_profile(), BalanceResult, broker_market_sort_key() (+32 more)

### Community 32 - "server/market.rs"
Cohesion: 0.12
Nodes (41): MySql, Postgres, drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract(), postgresql_rejects_invalid_credentials() (+33 more)

### Community 33 - "settings/ui/Page.tsx"
Cohesion: 0.15
Nodes (33): AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_profile_trading_type(), detect_trading_type(), DetectTradingTypeResult (+25 more)

### Community 34 - "KisRestClient"
Cohesion: 0.22
Nodes (17): PreviewRow, chart_volume_to_u64(), normalize_minute_time(), normalize_rows(), BrokerId, ChartCandle, CmdResult, Option (+9 more)

### Community 35 - "AccountProfile"
Cohesion: 0.17
Nodes (20): order_side_label(), OrderManager, PendingOrder, ProviderOrderResponse, ReservationDecision, Arc, Mutex, Option (+12 more)

### Community 36 - "src/api/types.ts"
Cohesion: 0.15
Nodes (31): broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc(), daily_chart_time() (+23 more)

### Community 37 - "rest/types.rs"
Cohesion: 0.10
Nodes (22): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+14 more)

### Community 38 - "databaseManagementSection.tsx"
Cohesion: 0.12
Nodes (27): automatic_missing_context_and_stale_reservation_leave_budget_and_pending_unchanged(), buy(), delayed_old_account_insufficient_balance_does_not_suspend_new_account(), legacy_configured_account_without_profile_allows_manual_but_fences_client_replacement(), local_holdings(), local_order(), local_single_order(), LocalProvider (+19 more)

### Community 39 - "DatabaseManager"
Cohesion: 0.23
Nodes (10): AtrPreparationView, AtrWindow, bar(), downward_gap_on_flat_candle_is_not_a_zero_range(), full_previous_close_history_is_required_and_old_ranges_expire(), large_prices_do_not_overflow_the_range_sum(), Option, String (+2 more)

### Community 40 - "AppState"
Cohesion: 0.15
Nodes (12): KrxItem, KrxProxyItem, KrxProxySearchResponse, KrxResponse, NaverAcItem, NaverAcResponse, save_cache(), StockList (+4 more)

### Community 41 - "server/records.rs"
Cohesion: 0.12
Nodes (4): maps_holding_to_broker_domain(), maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request()

### Community 42 - "StrategyConfig"
Cohesion: 0.23
Nodes (4): OrderRecord, OrderSide, OrderStore, parallel_appends_do_not_lose_order_records()

### Community 43 - "domain.rs"
Cohesion: 0.23
Nodes (11): DatabaseConfig, DatabaseConfigView, DatabaseProvider, DatabaseStatusView, DatabaseTableStatus, DatabaseTlsMode, DatabaseTransferResult, JsonStorageCategoryView (+3 more)

### Community 44 - "archive.rs"
Cohesion: 0.05
Nodes (76): useAppConfig(), useBalance(), useBrokerHoldings(), useOverseasBalance(), useTodayStats(), useExchangeRate(), useExchangeRateStatus(), useOverseasPrice() (+68 more)

### Community 45 - "commands/records.rs"
Cohesion: 0.27
Nodes (12): collect_trade_archive_stats(), collect_trade_archive_stats_active(), collect_trade_day_dirs(), dir_size_bytes(), get_trade_archive_config(), get_trade_archive_stats(), purge_old_trade_files(), purge_trade_archive_active() (+4 more)

### Community 46 - "PriceConditionStrategy"
Cohesion: 0.21
Nodes (19): fetch_overseas_tick(), fetch_toss_tick(), is_market_closed_error(), poll_symbols_tick(), PriceSource, Arc, KisRestClient, Mutex (+11 more)

### Community 47 - "detect.rs"
Cohesion: 0.15
Nodes (3): DiscordNotifier, NotificationEvent, NotificationLevel

### Community 48 - "submit_toss_small_buy_verification_for_profile"
Cohesion: 0.18
Nodes (24): collect_kis_pending_fills(), collect_toss_pending_fills(), domestic_order_is_terminal(), domestic_order_terminal_status(), domestic_provider_status(), kis_pending_from(), normalized_execution_u64(), overseas_order_terminal_status() (+16 more)

### Community 49 - "market/mod.rs"
Cohesion: 0.23
Nodes (15): run_backtest(), atr_history(), atr_risk_uses_previous_history_close_and_evaluation_gap_without_trading_history(), atr_sizing_assumptions(), atr_sizing_never_silently_uses_requested_quantity_when_ranges_are_missing(), daily_loss_and_consecutive_loss_state_roll_over_on_event_date(), deterministic_replay_applies_costs_and_separates_signals_from_fills(), event() (+7 more)

### Community 50 - "profiles.rs"
Cohesion: 0.06
Nodes (21): get_positions(), get_strategies(), PositionView, BrokerId, CmdResult, From, Option, Self (+13 more)

### Community 51 - "server/trading.rs"
Cohesion: 0.24
Nodes (12): candle(), daily_rows_keep_existing_trading_dates_and_hash_semantics(), input(), legacy_compact_minutes_are_explicit_kst_and_invalid_times_fail_closed(), market_midnight_fixture(), minute_provider_offsets_produce_identical_actual_replay_and_no_gap_fill(), minute_replay_rejects_same_instant_duplicates_and_history_overlap(), minute_replay_sorts_by_instant_rather_than_offset_text() (+4 more)

### Community 52 - "OrderRecord"
Cohesion: 0.12
Nodes (31): lth_default_adx_period(), lth_default_breakeven_buffer(), lth_default_buy_adx(), lth_default_buy_rsi(), lth_default_ema_long(), lth_default_ema_short(), lth_default_entry_end(), lth_default_entry_failure_observations() (+23 more)

### Community 53 - "StockStore"
Cohesion: 0.09
Nodes (28): File, AppState, balance_summary_total_krw(), CmdError, make_rest_client(), normalize_overseas_order_exchange(), parse_amount_i64(), Arc (+20 more)

### Community 54 - "security.rs"
Cohesion: 0.20
Nodes (26): FetchFuture, Fn, at_most_two_jobs_run_and_other_instruments_are_not_starved(), completed(), failures_use_backoff_and_do_not_retry_on_every_quote_tick(), history(), insufficient_history_reports_real_count_reason_and_retry(), maintain_captures_the_new_scope_atomically_after_a_waiting_start() (+18 more)

### Community 55 - "dependencies"
Cohesion: 0.19
Nodes (3): StockEntry, StockListStats, StockStore

### Community 56 - "logging/mod.rs"
Cohesion: 0.10
Nodes (46): apply_slippage(), BacktestPhaseView, BacktestReportView, BacktestSummaryView, BacktestTradeView, bps_cost(), current_equity(), default_atr_stop_multiplier() (+38 more)

### Community 57 - "active_profile"
Cohesion: 0.10
Nodes (15): Debug, Display, Formatter, BrokerAdapterResult, BrokerCurrency, BrokerId, Default, Option (+7 more)

### Community 58 - "compilerOptions"
Cohesion: 0.18
Nodes (15): BudgetCurrencyView, allocation_cannot_withdraw_committed_cash_or_overflow(), AutoTradingBudgetView, budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent(), BudgetAccount, BudgetCurrencyView, BudgetLedger, BudgetReservation (+7 more)

### Community 59 - "commands/database.rs"
Cohesion: 0.14
Nodes (17): Barrier, HashSet, daily_order_limit_key(), OrderManager, PendingOrder, Arc, HashMap, KisRestClient (+9 more)

### Community 60 - "commands/market.rs"
Cohesion: 0.24
Nodes (7): new_toss_client_order_id(), validate_client_order_id(), validate_optional_decimal(), validate_order_side(), validate_order_type(), validate_time_in_force(), validates_order_create_request_shape()

### Community 61 - "DiscordNotifier"
Cohesion: 0.07
Nodes (39): useClearDatabaseTables(), useCreateDatabaseTables(), useDatabaseConfig(), useDropDatabaseTables(), useExportDatabaseToJson(), useImportJsonToDatabase(), useJsonStorageInventory(), useSaveDatabaseConfig() (+31 more)

### Community 62 - "DatabaseConfig"
Cohesion: 0.13
Nodes (16): MySqlSslMode, PgSslMode, category_from_key(), connect_pool(), create_missing_mariadb_database(), create_missing_postgres_database(), document_key(), inventory_from_documents() (+8 more)

### Community 63 - "DatabasePool"
Cohesion: 0.14
Nodes (18): OverseasPriceResponse, KisOrderRejected, KisRestClient, read_kis_response_text(), AtomicBool, Client, ExecutedOrder, HeaderMap (+10 more)

### Community 64 - "TradeRecord"
Cohesion: 0.22
Nodes (6): clear_database_tables(), drop_database_tables(), ensure_trading_stopped(), export_database_to_json(), import_json_to_database(), set_storage_backend()

### Community 65 - "release-version.mjs"
Cohesion: 0.17
Nodes (25): decimal_quantity_units(), money_units(), normalized_f64(), normalized_u64(), BrokerCurrency, BrokerId, Json, Option (+17 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.13
Nodes (23): assert_auth(), candle_values(), chart_response_handles_empty_output_and_malformed_multibyte_body(), chart_response_preserves_http_and_provider_errors(), chart_response_propagates_announced_and_observed_body_caps(), domestic_latest_first_fixture_reverses_and_maps_ohlcv(), domestic_request_keeps_date_bounds_period_adjustment_and_query_order(), overseas_dailyprice_request_uses_its_own_tr_id_and_period_codes() (+15 more)

### Community 67 - "scripts"
Cohesion: 0.14
Nodes (43): activate_emergency_handler(), app_config_handler(), check_config_handler(), check_update_handler(), clear_buy_suspension_handler(), clear_emergency_handler(), exchange_rate_handler(), exchange_rate_status_handler() (+35 more)

### Community 68 - "place_toss_order"
Cohesion: 0.15
Nodes (7): restore_risk_from_today_trades(), Arc, CmdResult, Mutex, RiskStore, RiskRuntimeState, StrategySymbolCountEntry

### Community 69 - "conflicts.rs"
Cohesion: 0.17
Nodes (36): check_toss_order_preflight(), check_toss_order_preflight_for_profile(), check_toss_profile_connection(), list_toss_accounts(), list_toss_open_orders(), list_toss_open_orders_for_profile(), list_toss_profile_accounts(), lookup_toss_accounts_with_credentials() (+28 more)

### Community 70 - "toss/tests.rs"
Cohesion: 0.15
Nodes (10): build_monthly_path(), concurrent_writes_leave_one_complete_json_document(), corrupt_json_recovers_from_last_durable_backup(), json_path_lock(), monthly_path_zero_pads_month_for_stats(), read_json_or_default(), write_json(), DailyStats (+2 more)

### Community 71 - "commands/risk.rs"
Cohesion: 0.14
Nodes (25): Fixture, absent_unknown_and_similarly_prefixed_kis_tr_ids_preserve_pending_without_querying(), actual_fill_crosses_same_scope_restart_with_identical_provider_snapshot(), delayed_reconciliation_cannot_mutate_new_scope_and_applies_when_original_is_reselected(), direct_fill_and_zero_fill_terminal_cannot_mutate_another_execution_scope(), exact_eight_order_tr_ids_select_mode_and_toss_bypasses_kis_mode_classification(), executed(), persisted_kis_pending_never_queries_or_applies_in_the_opposite_mode() (+17 more)

### Community 72 - ".new"
Cohesion: 0.25
Nodes (25): add_profile_handler(), AddProfileBody, apply_profile_change(), default_body_broker_id(), delete_profile_handler(), DeleteProfileBody, detect_profile_handler(), detect_trading_type_handler() (+17 more)

### Community 73 - "storage/database.rs"
Cohesion: 0.11
Nodes (20): CandlePageView, LiveDailyHistoryStatus, AppConfigView, TradingStatus, brokerLabel(), BrokerScopeIndicator(), BrokerScopeIndicatorProps, HistoryRow() (+12 more)

### Community 74 - "KisWebSocketClient"
Cohesion: 0.08
Nodes (20): FormatTime, FrontendLogInput, get_recent_logs(), get_stats_by_range(), get_trades_by_range(), GetStatsByRangeInput, GetTradesByRangeInput, save_trade() (+12 more)

### Community 75 - "toss/orders.rs"
Cohesion: 0.33
Nodes (5): kis_read_min_interval(), Arc, Duration, RwLock, Self

### Community 76 - "read_json_or_default"
Cohesion: 0.39
Nodes (4): KisWebSocketClient, parse_realtime_price(), RealtimePrice, WsStatusEvent

### Community 77 - "StatsStore"
Cohesion: 0.32
Nodes (4): keychain_test_dirs(), legacy_file_password_migrates_to_keychain_on_load(), save_config_stores_password_in_keychain_not_in_file(), save_input_with_password()

### Community 78 - "devDependencies"
Cohesion: 0.21
Nodes (13): lth_preview_signals_for_interval(), F, LeveragedTrendHoldPreviewSignal, OhlcCandle, LeveragedTrendHoldTimedCandle, lth_default_qty(), LeveragedTrendHoldStrategy, F (+5 more)

### Community 79 - "project-map.mjs"
Cohesion: 0.20
Nodes (13): ChartRequest, ChartRequest<'a>, KisRestClient, parse_domestic_response(), parse_overseas_response(), ChartCandle, Client, HeaderMap (+5 more)

### Community 80 - "TokenManager"
Cohesion: 0.13
Nodes (23): parse_amount_f64(), fetch_account_risk_balance_krw(), fetch_toss_risk_balance_krw(), Arc, KisRestClient, Result, String, clear_buy_suspension() (+15 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.10
Nodes (10): PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick(), HashMap, Option, Self, String (+2 more)

### Community 82 - ".write_document"
Cohesion: 0.13
Nodes (12): config(), initializer_restores_previous_averages_for_first_evaluation_cross(), insufficient_history_has_no_previous_long_average(), MaCrossParams, MaCrossState, MovingAverageCrossStrategy, Default, HashMap (+4 more)

### Community 83 - "KisBrokerAdapter"
Cohesion: 0.09
Nodes (36): useAddProfile(), useCheckConfig(), useCheckTossProfileConnection(), useDeleteProfile(), useDetectProfileTradingType(), useDetectTradingType(), useListTossAccounts(), useListTossProfileAccounts() (+28 more)

### Community 84 - "support.rs"
Cohesion: 0.19
Nodes (18): BrokerCandlePage, candle_market_date(), CandleCursor, CandlePageRequest, kis_cursor_date(), next_toss_cursor(), previous_kis_cursor(), BrokerMarket (+10 more)

### Community 85 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.47
Nodes (3): BalanceSnapshot, BalanceStore, HoldingItem

### Community 86 - "broker/adapter.rs"
Cohesion: 0.18
Nodes (15): PendingCandidate, PendingFill, OrderManager, ReconciliationSnapshot, Arc, BrokerId, KisRestClient, Mutex (+7 more)

### Community 87 - "ui/LayoutResizer.tsx"
Cohesion: 0.24
Nodes (5): B, format_toss_error(), body_snippet(), read_toss_response_text(), toss_rate_limit_group()

### Community 88 - "initialize_active_strategy_history"
Cohesion: 0.20
Nodes (23): LeveragedTrendHoldEntry, LeveragedTrendHoldStrategy, buys_any_target_ticker_when_itself_trends_up(), buys_intraday_rebound_when_next_window_shows_buy_pressure(), buys_rapid_rebound_without_trend_snapshot_when_enabled(), downward_candles(), entry(), holds_failed_rebound_until_min_hold_observations_elapsed() (+15 more)

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

### Community 94 - "balance_store.rs"
Cohesion: 0.50
Nodes (3): TossApiError, TossErrorResponse, TossHttpError

### Community 95 - ".load_sync"
Cohesion: 0.13
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

### Community 100 - "verify-toss-openapi.mjs"
Cohesion: 0.11
Nodes (17): devDependencies, @playwright/test, @tauri-apps/cli, @types/react, @types/react-dom, @types/react-grid-layout, typescript, vite (+9 more)

### Community 101 - "OverseasExecutedOrder"
Cohesion: 0.05
Nodes (50): lightweight-charts, lightweight-charts, useChartData(), useOverseasChartData(), useTossChartData(), chartTimeToMs(), formatChartTimeKst(), formatKstMs() (+42 more)

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
Cohesion: 0.15
Nodes (13): AccessToken, Arc, Client, DateTime, Mutex, Option, Result, Self (+5 more)

### Community 116 - "build.rs"
Cohesion: 0.13
Nodes (28): RecentSideHistory, ScopedSymbol, blocks_reentry_after_stop_loss_sell_on_same_day(), blocks_repeated_same_side_signal_inside_cooldown(), blocks_tiny_profit_after_costs(), blocks_whipsaw_after_repeated_opposite_signals(), buy_signal(), cooldown_is_isolated_by_broker_account_scope() (+20 more)

### Community 118 - "settings-database.spec.ts"
Cohesion: 0.29
Nodes (11): blocked_strong_close_buy_does_not_discard_next_days_completed_condition(), candle(), daily_event_strategies_reject_other_cadences(), flat_completed_days_fill_history_without_compressing_the_window(), input(), ChartCandle, StrategyPreviewInput, Value (+3 more)

### Community 119 - "kis-auto-trade"
Cohesion: 0.19
Nodes (18): OrderSubmission, build_provider_request(), prepare_buy_submission(), prepare_order_submission(), prepare_sell_submission(), rest_order_side(), Option, OrderSide (+10 more)

### Community 120 - "playwright.config.ts"
Cohesion: 0.24
Nodes (16): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), calculate_atr(), initialize_active_strategy_history(), legacy_history_symbols(), Arc (+8 more)

### Community 121 - "sync-codex-skills.ps1"
Cohesion: 0.25
Nodes (18): atomic_write(), atomic_write_private(), atomic_write_private_sync(), backup_failure_preserves_primary_and_cleans_staged_content(), durable_atomic_write(), locked_destination_preserves_content_and_allows_retry_after_release(), private_async_replacements_keep_owner_only_permissions(), private_sync_replacements_keep_owner_only_permissions() (+10 more)

### Community 122 - "discord-notification-config/index.ts"
Cohesion: 0.23
Nodes (9): BrokerAdapter, BrokerAdapterError, DummyAdapter, BrokerAdapterResult, BrokerId, Error, Send, String (+1 more)

### Community 124 - "strategy-configure/index.ts"
Cohesion: 0.21
Nodes (13): Next, Router, app(), bearer_token(), constant_time_eq(), cross_origin_mutation_is_rejected_even_with_token(), has_same_origin(), mutation_without_token_is_rejected() (+5 more)

### Community 125 - "strategy-toggle/index.ts"
Cohesion: 0.17
Nodes (23): preview_strategy_from_candles(), all_strategies_keep_the_same_evaluation_window_without_implicit_warmup(), complete_daily_history_initializes_supported_strategies_without_shortening_defaults(), cross_and_breakout_seed_only_final_bars_are_not_conditions_not_met(), first_ready_bar_is_observed_after_tick_without_discarding_evaluation_bars(), generic_lth_is_rejected_before_live_clock_or_candle_preparation(), input(), insufficient_history_is_not_reported_as_a_zero_trade_strategy() (+15 more)

### Community 126 - "symbol-search/index.ts"
Cohesion: 0.25
Nodes (7): daily_diagnostic_never_evaluates_strategy_or_execution_even_with_intraday_input(), empty_minute_observations_are_not_evaluable(), ReplayAssessmentView, Self, String, Vec, ReplayMetadataView

### Community 127 - "trading-start-stop/index.ts"
Cohesion: 0.25
Nodes (7): KisBrokerAdapter, maps_kis_balance_item_to_broker_holding(), Arc, BalanceItem, BrokerId, KisRestClient, Self

### Community 130 - ".new"
Cohesion: 0.18
Nodes (13): DailyHistoryEntry, DailyHistoryStatus, DailyHistoryTicket, fingerprint(), pending_status(), BrokerId, DateTime, NaiveDate (+5 more)

### Community 131 - ".fmt"
Cohesion: 0.43
Nodes (5): OrderManager, PendingOrder, Result, validate_pending_kis_mode(), pending_order_provider()

### Community 132 - ".new"
Cohesion: 0.33
Nodes (4): toss_http_client(), trim_base_url(), url_encode(), toss_rate_limiter()

### Community 133 - ".search"
Cohesion: 0.19
Nodes (16): LeveragedTrendHoldPreviewInput, TossModifyOrderInput, TossOpenOrdersInput, TossOrderPreflightInput, TossSmallBuyVerificationInput, active_profile(), leveraged_trend_hold_preview_handler(), toss_chart_handler() (+8 more)

### Community 134 - "@tauri-apps/api"
Cohesion: 0.14
Nodes (6): AccountProfile, DiscordConfig, load_secure_config(), non_empty(), ProfilesConfig, SecureConfig

### Community 135 - "AppConfigView"
Cohesion: 0.20
Nodes (4): OrderManager, Option, String, TradingHealthStatus

### Community 136 - "작업 기록: <짧은 제목>"
Cohesion: 0.12
Nodes (12): default_strategy_broker_id(), BrokerId, Into, Option, Self, String, Value, Vec (+4 more)

### Community 137 - "log/ui/Page.tsx"
Cohesion: 0.47
Nodes (4): AsyncMutex, shared_toss_token_state(), toss_token_state_key(), TossTokenState

### Community 138 - ".process_buy"
Cohesion: 0.30
Nodes (10): estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), OrderManager, Option, OrderResponse, OrderSide, Result (+2 more)

### Community 139 - "CurrencyLedger"
Cohesion: 0.23
Nodes (7): LeveragedTrendHoldStrategy, Option, Vec, VecDeque, LeveragedTrendSnapshot, PreviewPosition, Option

### Community 141 - "modify_toss_order_for_profile"
Cohesion: 0.17
Nodes (23): broker_money_view_amount(), is_toss_order_final(), money_view_from_decimal(), poll_toss_order_detail(), Arc, BrokerCurrency, BrokerId, BrokerMarket (+15 more)

### Community 142 - "10. 완료 보고"
Cohesion: 0.23
Nodes (13): PreparedOrderSubmission, ProviderOrderRequest, append_failed_order(), build_kis_pending_order(), build_pending_order(), build_toss_pending_order(), failed_order_price(), Arc (+5 more)

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
Cohesion: 0.16
Nodes (19): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, BrokerCurrency, CmdResult, Option (+11 more)

### Community 160 - "collect_pages"
Cohesion: 0.25
Nodes (17): CandlePageView, collect_pages(), F, bar(), collects_252_actual_completed_highs_without_dropping_the_last_completed_bar(), empty_filtered_page_can_advance_and_full_512_live_window_is_bounded(), limits_requests_and_stops_on_repeat_or_no_progress(), only_first_provider_failure_changes_exchange_and_later_failure_is_explicit_partial() (+9 more)

### Community 161 - "refresh_manual_positions"
Cohesion: 0.32
Nodes (7): current_position_snapshot(), refresh_manual_positions(), Arc, Mutex, Option, OrderManager, Result

### Community 162 - "String"
Cohesion: 0.50
Nodes (7): place_overseas_with_retry_shared(), place_prepared_order(), place_with_retry_shared(), Arc, KisRestClient, OrderResponse, Result

### Community 163 - "ExchangeRateView"
Cohesion: 0.18
Nodes (12): ExchangeRateSource, ExchangeRateView, refresh_exchange_rate_status(), RefreshConfig, resolve_exchange_rate_policy(), Arc, Default, Path (+4 more)

### Community 164 - "collect_daily_history"
Cohesion: 0.24
Nodes (11): collect_daily_history(), CollectedDailyHistory, completed_bar(), price(), Arc, ChartCandle, KisRestClient, Option (+3 more)

### Community 165 - "toss/orders.rs"
Cohesion: 0.22
Nodes (7): TossOrder, TossOrderExecution, TossOrderListQuery, TossOrderListStatus, TossOrderModifyRequest, TossOrderResponse, TossPaginatedOrderResponse

### Community 175 - "Mutex"
Cohesion: 0.38
Nodes (12): candle(), config(), daily_hooks_do_not_create_unknown_symbol_states(), Value, strong_close_completed_day_uses_actual_position_and_keeps_stop_loss(), strong_close_uses_each_completed_day_only_on_the_following_day(), volatility(), volatility_collects_missing_history_after_close_for_later_days() (+4 more)

### Community 181 - "AppHandle"
Cohesion: 0.18
Nodes (9): args, artifactDir, artifacts, checksumPath, manifestPath, names, records, requireSignatures (+1 more)

### Community 183 - "evaluate_order_preflight"
Cohesion: 0.36
Nodes (8): buy_preflight_adds_commission_and_blocks_when_cash_is_short(), buy_preflight_passes_when_required_cash_is_available(), evaluate_order_preflight(), format_money_amount(), OrderPreflightConstraints, OrderPreflightDecision, OrderPreflightInput, sell_preflight_blocks_when_sellable_quantity_is_short()

### Community 184 - ".new"
Cohesion: 0.60
Nodes (3): Into, Self, String

### Community 190 - "Vec"
Cohesion: 0.24
Nodes (10): AutoTradingBudgetInput, UpdateAutoTradingBudgetInput, AutoTradingBudgetInput, checked_budget_scope(), get_auto_trading_budget(), update_auto_trading_budget(), UpdateAutoTradingBudgetInput, budget_handler() (+2 more)

### Community 192 - "modify_toss_order_for_profile"
Cohesion: 0.33
Nodes (6): modify_toss_order_for_profile(), Arc, Mutex, OrderManager, toss_decimal_equals(), toss_decimal_to_storage_units()

### Community 199 - "Self"
Cohesion: 0.40
Nodes (3): LeveragedTrendHoldStrategy, Option, String

### Community 202 - "Arc"
Cohesion: 0.22
Nodes (5): layerRank, layers, root, srcRoot, violations

### Community 203 - "BalanceItem"
Cohesion: 0.22
Nodes (6): appEntry, escapedName, npmLock, pkg, rootDir, workspaceLock

### Community 209 - "broker/mod.rs"
Cohesion: 0.44
Nodes (9): future_bar_does_not_change_prior_signal(), history(), params(), Value, Vec, run(), session_and_blackout_use_input_minutes(), timed() (+1 more)

### Community 210 - "Arc"
Cohesion: 0.16
Nodes (19): calendar_override_represents_holiday_without_fallback(), calendar_session_window_checks_time_range(), is_domestic_symbol(), is_krx_open(), is_krx_open_at(), is_market_open_for(), is_market_open_for_with_calendar(), is_market_open_for_with_calendar_policy() (+11 more)

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
- **292 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+287 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **353 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ServerState` connect `scripts` to `leveraged_trend_hold.rs`, `release-version.mjs`, `place_toss_order`, `commands/trading.rs`, `@tauri-apps/api`, `toss/tests.rs`, `.new`, `TossApiError`, `TossOpenApiClient`, `detect.rs`, `profiles.rs`, `dependencies`, `TradeStore`, `strategy-configure/index.ts`?**
  _High betweenness centrality (0.092) - this node is a cross-community bridge._
- **Why does `AppState` connect `StockStore` to `leveraged_trend_hold.rs`, `commands/trading.rs`, `@tauri-apps/api`, `modify_toss_order_for_profile`, `TossOpenApiClient`, `database_projection.rs`, `place_toss_order`, `BrokerScope`, `settings/ui/Page.tsx`, `src/api/types.ts`, `StrategyConfig`, `detect.rs`, `profiles.rs`, `dependencies`, `place_toss_order`, `conflicts.rs`, `toss/tests.rs`, `TokenManager`, `StrategyStore`, `TossApiError`?**
  _High betweenness centrality (0.077) - this node is a cross-community bridge._
- **Why does `RiskManager` connect `leveraged_trend_hold.rs` to `scripts`, `place_toss_order`, `PriceConditionStrategy`, `fills.rs`, `StockStore`, `kis-auto-trade`, `playwright.config.ts`, `logging/mod.rs`, `commands/database.rs`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _292 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `leveraged_trend_hold.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0750151240169389 - nodes in this community are weakly interconnected._
- **Should `simulation.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.04968354430379747 - nodes in this community are weakly interconnected._
- **Should `hooks.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.03636363636363636 - nodes in this community are weakly interconnected._