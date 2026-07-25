# Graph Report - KISAutoTrade  (2026-07-25)

## Corpus Check
- 199 files · ~159,675 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 3384 nodes · 8400 edges · 135 communities (128 shown, 7 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 241 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `a6fa5ad5`
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
- .to_broker_holding
- initialize_active_strategy_history
- PendingOrderStore
- StrategyStore
- TradeStore
- database_keychain.rs
- .new
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
- model/tradingStore.ts
- TossApiError
- strategy-scrollbar.spec.ts
- model/accountStore.ts
- install_database_manager
- NotificationService
- exchange.rs
- toss_currency_from_view
- @mui/material
- setup-local.sh
- kis-auto-trade
- Position
- .new
- StrategyView
- .new
- .search
- @tauri-apps/api

## God Nodes (most connected - your core abstractions)
1. `AppState` - 120 edges
2. `ServerState` - 107 edges
3. `invoke()` - 89 edges
4. `StrategyConfig` - 73 edges
5. `LeveragedTrendHoldStrategy` - 61 edges
6. `OrderManager` - 59 edges
7. `RiskManager` - 57 edges
8. `BrokerScope` - 48 edges
9. `KisRestClient` - 47 edges
10. `BrokerSymbol` - 47 edges

## Surprising Connections (you probably didn't know these)
- `maps_holding_to_broker_domain()` --calls--> `BrokerAccountId`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `maps_toss_candle_to_broker_candle()` --calls--> `BrokerSymbol`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `selects_price_response_case_insensitively()` --calls--> `BrokerSymbol`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `validates_market_data_query_limits_before_request()` --calls--> `BrokerSymbol`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs
- `fetch_account_risk_balance_krw()` --calls--> `parse_amount_f64()`  [INFERRED]
  src-tauri/src/commands/trading.rs → src-tauri/src/commands.rs

## Import Cycles
- 2-file cycle: `src-tauri/src/storage/database.rs -> src-tauri/src/storage/database_schema.rs -> src-tauri/src/storage/database.rs`
- 2-file cycle: `src-tauri/src/server/mod.rs -> src-tauri/src/server/profiles.rs -> src-tauri/src/server/mod.rs`

## Communities (135 total, 7 thin omitted)

### Community 0 - "leveraged_trend_hold.rs"
Cohesion: 0.12
Nodes (11): calculate_atr(), OhlcCandle, LeveragedRapidReboundSnapshot, LeveragedReboundSnapshot, LeveragedTrendHoldMarketState, LeveragedTrendHoldPosition, LeveragedTrendHoldStrategy, LeveragedTrendSnapshot (+3 more)

### Community 1 - "simulation.rs"
Cohesion: 0.05
Nodes (98): SimulationAssumptions, broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc() (+90 more)

### Community 2 - "hooks.ts"
Cohesion: 0.04
Nodes (73): canUseTauriEvents(), useBackendEvents(), frontendLogger, OVERSEAS_CHART_PRESETS, useKisExecutedByRange(), useOverseasExecutedByRange(), AppLogEntry, BalanceResult (+65 more)

### Community 3 - "shared/api/commands.ts"
Cohesion: 0.05
Nodes (75): activateEmergencyStop(), addProfile(), checkConfig(), checkForUpdate(), checkTossOrderPreflight(), checkTossProfileConnection(), clearBuySuspension(), clearEmergencyStop() (+67 more)

### Community 4 - "RiskManager"
Cohesion: 0.06
Nodes (36): RiskStore, Option, PathBuf, Result, Self, config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope() (+28 more)

### Community 5 - "commands/trading.rs"
Cohesion: 0.05
Nodes (78): FixedOffset, clear_buy_suspension(), fetch_account_risk_balance_krw(), fetch_overseas_tick(), fetch_toss_risk_balance_krw(), fetch_toss_tick(), get_trading_status(), is_market_closed_error() (+70 more)

### Community 6 - "ConsecutiveMoveStrategy"
Cohesion: 0.05
Nodes (20): ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, FailedBreakoutParams, FailedBreakoutState, FailedBreakoutStrategy, FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy (+12 more)

### Community 7 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.06
Nodes (61): usePreviewLeveragedTrendHold(), usePreviewStrategy(), useRefreshStockList(), ProfileFormState, clearExperimentSlots(), compactSnapshot(), ExperimentScope, loadExperimentSlots() (+53 more)

### Community 8 - "PositionTracker"
Cohesion: 0.14
Nodes (8): OverseasPosition, OverseasPositionTracker, PositionTracker, Default, HashMap, I, Self, String

### Community 9 - "ui/AppShell.tsx"
Cohesion: 0.05
Nodes (38): useRecentLogs(), useUpdateCheck(), SettingsState, useSettingsStore, queryClient, LEVEL_COLORS, Log(), LogLevel (+30 more)

### Community 10 - "OrderManager"
Cohesion: 0.09
Nodes (20): HashSet, daily_order_limit_key(), estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), order_exchange_code(), OrderManager, PendingOrder (+12 more)

### Community 11 - "submission.rs"
Cohesion: 0.13
Nodes (42): RestOrderSide, append_failed_order(), build_kis_pending_order(), build_pending_order(), build_provider_request(), build_toss_pending_order(), current_position_snapshot(), failed_order_price() (+34 more)

### Community 12 - "dashboard/ui/Page.tsx"
Cohesion: 0.08
Nodes (44): useActivateEmergencyStop(), useAppConfig(), useBalance(), useBrokerHoldings(), useClearBuySuspension(), useClearEmergencyStop(), useExchangeRate(), useExchangeRateStatus() (+36 more)

### Community 13 - "trading/ui/Page.tsx"
Cohesion: 0.08
Nodes (52): useModifyTossOrder(), useOverseasPrice(), usePlaceOrder(), usePlaceOverseasOrder(), usePrice(), useStockSearch(), useTossMarketSnapshot(), useTossOpenOrders() (+44 more)

### Community 14 - "TossOpenApiClient"
Cohesion: 0.12
Nodes (13): B, BrokerCurrency, Client, Option, Result, String, T, Vec (+5 more)

### Community 15 - "settings.rs"
Cohesion: 0.10
Nodes (44): AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_trading_type(), DetectTradingTypeResult, ExchangeRateSource (+36 more)

### Community 16 - "toss/types.rs"
Cohesion: 0.12
Nodes (40): Option, String, T, Value, Vec, TossAccount, TossApiResponse, TossBuyingPower (+32 more)

### Community 17 - "BrokerSymbol"
Cohesion: 0.12
Nodes (13): Debug, Display, Formatter, BrokerSymbol, BrokerAdapterResult, BrokerCurrency, BrokerId, Default (+5 more)

### Community 18 - "RsiStrategy"
Cohesion: 0.07
Nodes (13): DeviationParams, DeviationStrategy, MomentumParams, MomentumState, MomentumStrategy, Default, HashMap, Option (+5 more)

### Community 19 - "fills.rs"
Cohesion: 0.08
Nodes (48): Iterator, OrderStatus, order_side_label(), pending_conflict_blocks_opposite_side_in_same_scope(), pending_conflict_blocks_same_side_in_same_scope(), pending_conflict_reason_for_scope(), pending_conflict_scan_ignores_different_scope(), pending_order() (+40 more)

### Community 20 - "accounts.rs"
Cohesion: 0.12
Nodes (40): AppConfigView, OverseasBalanceItem, OverseasBalanceSummary, add_profile(), AddProfileInput, apply_active_profile(), BalanceResult, broker_market_sort_key() (+32 more)

### Community 21 - "MeanReversionStrategy"
Cohesion: 0.07
Nodes (14): MeanReversionParams, MeanReversionState, MeanReversionStrategy, Default, HashMap, Option, Self, String (+6 more)

### Community 22 - "database_projection.rs"
Cohesion: 0.12
Nodes (36): MySql, Postgres, drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract(), postgresql_rejects_invalid_credentials() (+28 more)

### Community 23 - "toss_market.rs"
Cohesion: 0.12
Nodes (41): get_active_toss_calendar_override(), get_toss_chart_data(), get_toss_chart_data_for_profile(), get_toss_market_calendar(), get_toss_market_calendar_for_profile(), get_toss_market_snapshot(), get_toss_market_snapshot_for_profile(), get_toss_stock_safety() (+33 more)

### Community 24 - "ServerState"
Cohesion: 0.15
Nodes (41): activate_emergency_handler(), app_config_handler(), check_config_handler(), check_update_handler(), clear_buy_suspension_handler(), clear_emergency_handler(), exchange_rate_handler(), exchange_rate_status_handler() (+33 more)

### Community 25 - "RateLimitScheduler"
Cohesion: 0.11
Nodes (30): FnOnce, Instant, IntoIterator, applies_retry_after_to_group_pause(), epoch_ms_now(), parse_delay_header(), rate_limit_reset_delay(), RateLimitGroupState (+22 more)

### Community 26 - "strategy/ui/Page.tsx"
Cohesion: 0.08
Nodes (36): useStrategies(), useTossMarketCalendar(), useUpdateStrategy(), hasInvalidLthEntries(), brokerLabel(), EditState, EXCHANGE_SEARCH_ORDER, fmtTossSessionWindow() (+28 more)

### Community 27 - "commands/toss.rs"
Cohesion: 0.18
Nodes (34): check_toss_order_preflight(), check_toss_profile_connection(), list_toss_accounts(), list_toss_open_orders(), list_toss_open_orders_for_profile(), list_toss_profile_accounts(), lookup_toss_accounts_with_credentials(), mask_toss_account_no() (+26 more)

### Community 28 - "tauri.conf.json"
Cohesion: 0.05
Nodes (40): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, Korean, app, security (+32 more)

### Community 29 - "ui/StockChart.tsx"
Cohesion: 0.07
Nodes (35): useChartData(), useOverseasChartData(), useTossChartData(), buildChartOptions(), CandlePoint, ChartType, CrosshairData, LinePoint (+27 more)

### Community 30 - "StrategyManager"
Cohesion: 0.10
Nodes (12): live_tick_and_preview_emit_identical_signals_for_normalized_fixture(), StrategySignal, build_strategy(), Box, BrokerId, Default, F, Option (+4 more)

### Community 31 - "BrokerScope"
Cohesion: 0.13
Nodes (25): RecentSideHistory, ScopedSymbol, BrokerScope, blocks_reentry_after_stop_loss_sell_on_same_day(), blocks_repeated_same_side_signal_inside_cooldown(), blocks_tiny_profit_after_costs(), blocks_whipsaw_after_repeated_opposite_signals(), buy_signal() (+17 more)

### Community 32 - "server/market.rs"
Cohesion: 0.17
Nodes (36): account_sync_error(), balance_handler(), broker_holdings_handler(), chart_handler(), ChartQuery, executed_handler(), invalid_order_input(), order_handler() (+28 more)

### Community 33 - "settings/ui/Page.tsx"
Cohesion: 0.10
Nodes (30): useBrokerRateLimitStatus(), useLogConfig(), useRefreshConfig(), useSaveWebConfig(), useSendTestDiscord(), useSetLogConfig(), useSetRefreshConfig(), useSetStockUpdateInterval() (+22 more)

### Community 34 - "KisRestClient"
Cohesion: 0.18
Nodes (14): OverseasPriceResponse, RequestBuilder, KisRestClient, read_kis_response_text(), AtomicBool, ChartCandle, ExecutedOrder, HeaderMap (+6 more)

### Community 35 - "AccountProfile"
Cohesion: 0.11
Nodes (34): lth_default_adx_period(), lth_default_breakeven_buffer(), lth_default_buy_adx(), lth_default_buy_rsi(), lth_default_ema_long(), lth_default_ema_short(), lth_default_entry_end(), lth_default_entry_failure_observations() (+26 more)

### Community 36 - "src/api/types.ts"
Cohesion: 0.14
Nodes (26): useAddProfile(), useCheckConfig(), useCheckTossProfileConnection(), useDeleteProfile(), useDetectProfileTradingType(), useDetectTradingType(), useListTossAccounts(), useListTossProfileAccounts() (+18 more)

### Community 37 - "rest/types.rs"
Cohesion: 0.12
Nodes (26): BalanceItem, BalanceResponse, BalanceSummary, ChartCandle, ExecutedOrder, KisOutput1, OrderRequest, OrderResponse (+18 more)

### Community 38 - "databaseManagementSection.tsx"
Cohesion: 0.10
Nodes (38): useClearDatabaseTables(), useCreateDatabaseTables(), useDatabaseConfig(), useDropDatabaseTables(), useExportDatabaseToJson(), useImportJsonToDatabase(), useJsonStorageInventory(), useSaveDatabaseConfig() (+30 more)

### Community 39 - "DatabaseManager"
Cohesion: 0.20
Nodes (10): DatabaseManager, install_database_manager(), Arc, DatabaseStatusView, DatabaseTransferResult, Mutex, Result, RwLock (+2 more)

### Community 40 - "AppState"
Cohesion: 0.10
Nodes (27): File, AppState, balance_summary_total_krw(), CmdError, make_rest_client(), normalize_overseas_order_exchange(), parse_amount_f64(), parse_amount_i64() (+19 more)

### Community 41 - "server/records.rs"
Cohesion: 0.20
Nodes (29): archive_config_handler(), archive_stats_handler(), DateRangeQuery, frontend_log_handler(), FrontendLogBody, kis_executed_handler(), log_config_handler(), pending_orders_handler() (+21 more)

### Community 42 - "StrategyConfig"
Cohesion: 0.24
Nodes (11): BrokerPositionSnapshot, default_strategy_broker_id(), BrokerId, BrokerMarket, Into, Option, Self, String (+3 more)

### Community 43 - "domain.rs"
Cohesion: 0.14
Nodes (24): broker_scope_serializes_with_camel_case_fields(), BrokerAccountId, BrokerCandle, BrokerClientOrderId, BrokerCurrency, BrokerHolding, BrokerId, BrokerMarket (+16 more)

### Community 44 - "archive.rs"
Cohesion: 0.17
Nodes (23): collect_trade_archive_stats(), collect_trade_archive_stats_active(), collect_trade_day_dirs(), dir_size_bytes(), get_trade_archive_config(), get_trade_archive_stats(), purge_old_trade_files(), purge_trade_archive_active() (+15 more)

### Community 45 - "commands/records.rs"
Cohesion: 0.21
Nodes (26): FrontendLogInput, get_kis_executed_by_range(), get_overseas_executed_by_range(), get_recent_logs(), get_stats_by_range(), get_today_executed(), get_today_overseas_executed(), get_today_stats() (+18 more)

### Community 46 - "PriceConditionStrategy"
Cohesion: 0.09
Nodes (10): PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick(), HashMap, Option, Self, String (+2 more)

### Community 47 - "detect.rs"
Cohesion: 0.13
Nodes (17): classify_token_response(), detect_trading_type(), DetectedTradingType, DetectTokenReq, DetectTradingTypeError, is_paper_key_rejected_by_real_domain(), is_real_key_rejected_by_paper_domain(), mentions_app_key() (+9 more)

### Community 48 - "submit_toss_small_buy_verification_for_profile"
Cohesion: 0.13
Nodes (29): modify_toss_order_for_profile(), Arc, Mutex, OrderManager, broker_money_view_amount(), is_toss_order_final(), money_view_from_decimal(), poll_toss_order_detail() (+21 more)

### Community 49 - "market/mod.rs"
Cohesion: 0.19
Nodes (23): KrxItem, KrxProxyItem, KrxProxySearchResponse, KrxResponse, lookup_name_by_code(), NaverAcItem, NaverAcResponse, Option (+15 more)

### Community 50 - "profiles.rs"
Cohesion: 0.25
Nodes (25): add_profile_handler(), AddProfileBody, apply_profile_change(), default_body_broker_id(), delete_profile_handler(), DeleteProfileBody, detect_profile_handler(), detect_trading_type_handler() (+17 more)

### Community 51 - "server/trading.rs"
Cohesion: 0.17
Nodes (25): decimal_quantity_units(), money_units(), normalized_f64(), normalized_u64(), BrokerCurrency, BrokerId, Json, Option (+17 more)

### Community 52 - "OrderRecord"
Cohesion: 0.34
Nodes (7): OrderStore, parallel_appends_do_not_lose_order_records(), Mutex, NaiveDate, PathBuf, Result, Vec

### Community 53 - "StockStore"
Cohesion: 0.15
Nodes (13): Arc, HashMap, I, Option, Path, PathBuf, Result, RwLock (+5 more)

### Community 54 - "security.rs"
Cohesion: 0.12
Nodes (22): Body, Next, Request, Router, app(), bearer_token(), constant_time_eq(), cross_origin_mutation_is_rejected_even_with_token() (+14 more)

### Community 55 - "dependencies"
Cohesion: 0.08
Nodes (25): @emotion/react, @emotion/styled, lightweight-charts, @mui/icons-material, @mui/material, dependencies, @emotion/react, @emotion/styled (+17 more)

### Community 56 - "logging/mod.rs"
Cohesion: 0.16
Nodes (20): FormatTime, cleanup(), default_log_dir(), find_most_recent_log_file(), init(), LocalTimer, LogConfig, LogEntry (+12 more)

### Community 57 - "active_profile"
Cohesion: 0.22
Nodes (24): LeveragedTrendHoldPreviewInput, active_profile(), leveraged_trend_hold_preview_handler(), Json, Option, Path, Query, Result (+16 more)

### Community 58 - "compilerOptions"
Cohesion: 0.08
Nodes (23): DOM, DOM.Iterable, ES2020, src, compilerOptions, allowImportingTsExtensions, isolatedModules, jsx (+15 more)

### Community 59 - "commands/database.rs"
Cohesion: 0.20
Nodes (23): clear_database_tables(), create_database_tables(), database_error(), drop_database_tables(), ensure_trading_stopped(), export_database_to_json(), get_database_config(), import_json_to_database() (+15 more)

### Community 60 - "commands/market.rs"
Cohesion: 0.20
Nodes (23): ChartDataInput, get_chart_data(), get_overseas_chart_data(), get_overseas_price(), get_price(), get_stock_list_stats(), OverseasOrderInput, OverseasPriceView (+15 more)

### Community 61 - "DiscordNotifier"
Cohesion: 0.14
Nodes (11): DiscordNotifier, Client, Result, Self, String, NotificationEvent, NotificationLevel, Into (+3 more)

### Community 62 - "DatabaseConfig"
Cohesion: 0.18
Nodes (18): DatabaseConfig, DatabaseConfigView, DatabaseProvider, DatabaseStatusView, DatabaseTableStatus, DatabaseTlsMode, DatabaseTransferResult, JsonStorageCategoryView (+10 more)

### Community 63 - "DatabasePool"
Cohesion: 0.17
Nodes (20): MySqlPool, PgPool, R, DatabaseArchiveStats, purge(), row_values(), NaiveDate, Option (+12 more)

### Community 64 - "TradeRecord"
Cohesion: 0.15
Nodes (22): account_less_scope_matches_any_account_of_same_broker(), default_currency(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), BrokerId, Mutex (+14 more)

### Community 65 - "release-version.mjs"
Cohesion: 0.15
Nodes (17): args, assertGitClean(), assertTagAvailable(), branchName, changedFiles, currentBranchName(), escapeRegExp(), fail() (+9 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.13
Nodes (9): MaCrossParams, MaCrossState, MovingAverageCrossStrategy, Default, HashMap, Option, Self, String (+1 more)

### Community 67 - "scripts"
Cohesion: 0.10
Nodes (21): scripts, build, build:app, build:app:debug, build:web, check:fsd, check:graphify, check:project-map (+13 more)

### Community 68 - "place_toss_order"
Cohesion: 0.20
Nodes (16): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, BrokerCurrency, CmdResult, Option (+8 more)

### Community 69 - "conflicts.rs"
Cohesion: 0.16
Nodes (11): LeveragedTrendHoldEntry, LeveragedTrendHoldParams, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldTimedCandle, lth_default_qty(), parse_hhmm(), Default, F (+3 more)

### Community 70 - "toss/tests.rs"
Cohesion: 0.11
Nodes (7): new_toss_client_order_id(), String, maps_holding_to_broker_domain(), maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request(), validates_order_create_request_shape()

### Community 71 - "commands/risk.rs"
Cohesion: 0.26
Nodes (18): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+10 more)

### Community 72 - ".new"
Cohesion: 0.16
Nodes (15): AsyncMutex, Arc, Into, Self, shared_toss_token_state(), toss_token_state_key(), TossTokenState, body_snippet() (+7 more)

### Community 73 - "storage/database.rs"
Cohesion: 0.16
Nodes (13): MySqlSslMode, PgSslMode, connect_pool(), create_missing_mariadb_database(), create_missing_postgres_database(), inventory_from_documents(), inventory_groups_documents_by_top_level_category(), is_missing_database_error() (+5 more)

### Community 74 - "KisWebSocketClient"
Cohesion: 0.19
Nodes (14): KisWebSocketClient, parse_realtime_price(), RealtimePrice, AppHandle, Arc, AtomicBool, Option, Result (+6 more)

### Community 75 - "toss/orders.rs"
Cohesion: 0.23
Nodes (13): Option, Self, String, Vec, TossOrder, TossOrderCreateRequest, TossOrderExecution, TossOrderListQuery (+5 more)

### Community 76 - "read_json_or_default"
Cohesion: 0.24
Nodes (17): build_daily_path(), build_monthly_path(), concurrent_writes_leave_one_complete_json_document(), corrupt_json_recovers_from_last_durable_backup(), daily_path_uses_date_components_without_string_separators(), ensure_dir(), json_path_lock(), monthly_path_zero_pads_month_for_stats() (+9 more)

### Community 77 - "StatsStore"
Cohesion: 0.25
Nodes (9): DailyStats, Mutex, NaiveDate, PathBuf, Result, Self, String, Vec (+1 more)

### Community 78 - "devDependencies"
Cohesion: 0.12
Nodes (17): devDependencies, @playwright/test, @tauri-apps/cli, @types/react, @types/react-dom, @types/react-grid-layout, typescript, vite (+9 more)

### Community 79 - "project-map.mjs"
Cohesion: 0.18
Nodes (16): buildTree(), compareNames(), currentDocument, fail(), generateInventory(), listRepositoryFiles(), mode, nextDocument (+8 more)

### Community 80 - "TokenManager"
Cohesion: 0.19
Nodes (12): AccessToken, Arc, Client, DateTime, Mutex, Option, Result, Self (+4 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.16
Nodes (20): BrokerOrderSide, BrokerQuantity, check_toss_order_preflight_for_profile(), parse_toss_order_side(), BrokerCurrency, BrokerMoneyView, toss_currency_from_view(), buy_preflight_adds_commission_and_blocks_when_cash_is_short() (+12 more)

### Community 82 - ".write_document"
Cohesion: 0.28
Nodes (12): category_from_key(), document_key(), LocalDocument, read_managed_json(), read_managed_json_backup(), Option, Path, String (+4 more)

### Community 83 - "KisBrokerAdapter"
Cohesion: 0.17
Nodes (9): KisBrokerAdapter, maps_kis_balance_item_to_broker_holding(), Arc, BalanceItem, BrokerAdapterResult, BrokerId, Option, Self (+1 more)

### Community 84 - "support.rs"
Cohesion: 0.14
Nodes (18): Result, market_from_currency(), BrokerCurrency, BrokerMarket, Option, Result, toss_currency(), toss_currency_code() (+10 more)

### Community 85 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.25
Nodes (13): chartTimeToMs(), formatChartTimeKst(), formatKstMs(), pad2(), parseTimeMs(), Props, StrategyPreviewChart(), StrategyPreviewChartSignal (+5 more)

### Community 86 - "broker/adapter.rs"
Cohesion: 0.21
Nodes (10): BrokerAdapter, BrokerAdapterError, default_place_order_returns_unsupported_with_broker_id(), DummyAdapter, BrokerAdapterResult, BrokerId, Error, Send (+2 more)

### Community 87 - ".to_broker_holding"
Cohesion: 0.29
Nodes (18): buys_any_target_ticker_when_itself_trends_up(), buys_intraday_rebound_when_next_window_shows_buy_pressure(), buys_rapid_rebound_without_trend_snapshot_when_enabled(), downward_candles(), holds_failed_rebound_until_min_hold_observations_elapsed(), holds_when_protection_profit_has_not_activated(), ignores_intraday_rebound_by_default(), ignores_rapid_rebound_by_default() (+10 more)

### Community 88 - "initialize_active_strategy_history"
Cohesion: 0.32
Nodes (11): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), initialize_active_strategy_history(), Arc, Mutex, Option (+3 more)

### Community 89 - "PendingOrderStore"
Cohesion: 0.32
Nodes (8): PendingOrderStore, restart_snapshot_preserves_scope_client_id_and_fill_watermark(), Mutex, PathBuf, PendingOrder, Result, Self, Vec

### Community 90 - "StrategyStore"
Cohesion: 0.26
Nodes (7): Mutex, Path, PathBuf, Result, Self, Vec, StrategyStore

### Community 91 - "TradeStore"
Cohesion: 0.20
Nodes (15): get_positions(), get_strategies(), PositionView, BrokerId, CmdResult, From, Option, Self (+7 more)

### Community 92 - "database_keychain.rs"
Cohesion: 0.29
Nodes (9): MutexGuard, delete_database_password(), entry(), keychain_test_lock(), load_database_password(), Option, Result, String (+1 more)

### Community 93 - ".new"
Cohesion: 0.24
Nodes (8): kis_http_client(), kis_read_min_interval(), Arc, Client, Duration, RwLock, Self, String

### Community 94 - "balance_store.rs"
Cohesion: 0.29
Nodes (8): BalanceSnapshot, BalanceStore, HoldingItem, PathBuf, Result, Self, String, Vec

### Community 95 - ".load_sync"
Cohesion: 0.27
Nodes (8): keychain_test_dirs(), legacy_file_password_migrates_to_keychain_on_load(), DatabaseConfigView, PathBuf, SaveDatabaseConfigInput, Self, save_config_stores_password_in_keychain_not_in_file(), save_input_with_password()

### Community 96 - "check"
Cohesion: 0.31
Nodes (8): check(), GitHubRelease, is_newer(), Client, Option, Result, String, UpdateInfo

### Community 97 - "compilerOptions"
Cohesion: 0.22
Nodes (8): vite.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 98 - "check-fsd-imports.mjs"
Cohesion: 0.22
Nodes (5): layerRank, layers, root, srcRoot, violations

### Community 99 - "package.json"
Cohesion: 0.25
Nodes (7): engines, node, npm, name, private, type, version

### Community 100 - "verify-toss-openapi.mjs"
Cohesion: 0.25
Nodes (6): endpointInventory, EXPECTED_PATHS, hasRateLimitHeaders, missingPaths, paths, servers

### Community 101 - "OverseasExecutedOrder"
Cohesion: 0.39
Nodes (4): first_positive_u64(), KisOutput1<T>, OverseasExecutedOrder, parse_decimal_cents()

### Community 102 - "BrokerAccountId"
Cohesion: 0.12
Nodes (5): StrongCloseStrategy, initialize_strategy_warmup(), Send, Sync, Strategy

### Community 103 - "TossAccessToken"
Cohesion: 0.36
Nodes (6): DateTime, From, Self, Utc, TossAccessToken, TossAccessTokenStatus

### Community 104 - "atomic_write"
Cohesion: 0.81
Nodes (6): atomic_write(), atomic_write_private(), atomic_write_private_sync(), durable_atomic_write(), Path, Result

### Community 105 - "model/tradingStore.ts"
Cohesion: 0.33
Nodes (3): TradingState, TradingStatus, useTradingStore

### Community 106 - "TossApiError"
Cohesion: 0.22
Nodes (9): format_toss_error(), Error, HeaderMap, Option, StatusCode, String, Value, TossApiError (+1 more)

### Community 107 - "strategy-scrollbar.spec.ts"
Cohesion: 0.47
Nodes (5): mockApi(), MockOptions, mockResearchResult(), strategy(), strategyEntries

### Community 109 - "install_database_manager"
Cohesion: 0.50
Nodes (4): copy_dir_all(), Path, Result, run()

### Community 110 - "NotificationService"
Cohesion: 0.50
Nodes (3): NotificationService, Send, Sync

### Community 112 - "toss_currency_from_view"
Cohesion: 0.18
Nodes (12): buildSnapshot(), fail(), listRepositoryFiles(), mode, parseMode(), REPOSITORY_ROOT, ROOT_CONFIGS, SCRIPT_DIR (+4 more)

### Community 114 - "@mui/material"
Cohesion: 0.30
Nodes (7): OrderRecord, OrderSide, BrokerId, Into, Option, Self, String

### Community 129 - "Position"
Cohesion: 0.18
Nodes (3): Position, Option, Vec

### Community 131 - "StrategyView"
Cohesion: 0.24
Nodes (8): build_strategy_view(), BrokerId, HashMap, Option, String, Value, Vec, StrategyView

### Community 132 - ".new"
Cohesion: 0.60
Nodes (3): Into, Self, String

## Knowledge Gaps
- **249 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+244 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `AppState` to `simulation.rs`, `RiskManager`, `commands/trading.rs`, `PositionTracker`, `settings.rs`, `accounts.rs`, `toss_market.rs`, `commands/toss.rs`, `StrategyManager`, `KisRestClient`, `DatabaseManager`, `archive.rs`, `commands/records.rs`, `submit_toss_small_buy_verification_for_profile`, `OrderRecord`, `StockStore`, `commands/database.rs`, `commands/market.rs`, `DiscordNotifier`, `TradeRecord`, `place_toss_order`, `commands/risk.rs`, `StatsStore`, `StrategyStore`, `TradeStore`?**
  _High betweenness centrality (0.150) - this node is a cross-community bridge._
- **Why does `KisRestClient` connect `KisRestClient` to `commands/trading.rs`, `AppState`, `OrderManager`, `submission.rs`, `TokenManager`, `KisBrokerAdapter`, `accounts.rs`, `fills.rs`, `initialize_active_strategy_history`, `RateLimitScheduler`, `.new`, `ServerState`?**
  _High betweenness centrality (0.091) - this node is a cross-community bridge._
- **Why does `ServerState` connect `ServerState` to `server/market.rs`, `TradeRecord`, `KisRestClient`, `RiskManager`, `commands/trading.rs`, `DatabaseManager`, `PositionTracker`, `server/records.rs`, `StatsStore`, `profiles.rs`, `server/trading.rs`, `StockStore`, `security.rs`, `active_profile`, `StrategyStore`, `DiscordNotifier`, `StrategyManager`?**
  _High betweenness centrality (0.088) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _249 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `leveraged_trend_hold.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1214574898785425 - nodes in this community are weakly interconnected._
- **Should `simulation.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05054945054945055 - nodes in this community are weakly interconnected._
- **Should `hooks.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.035017375033413525 - nodes in this community are weakly interconnected._