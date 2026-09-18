# Graph Report - KISAutoTrade  (2026-09-11)

## Corpus Check
- 212 files · ~169,102 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 3574 nodes · 8772 edges · 146 communities (139 shown, 7 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 246 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `0e58d59e`
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
- verify-release-artifacts.mjs
- 작업 기록: <짧은 제목>
- KISAutoTrade 검증 Harness 지도
- KISAutoTrade 탐색 지도
- CurrencyLedger
- verify-lockfiles.mjs
- modify_toss_order_for_profile
- 10. 완료 보고
- 2. 목표와 비목표
- react-grid-layout

## God Nodes (most connected - your core abstractions)
1. `AppState` - 122 edges
2. `ServerState` - 109 edges
3. `invoke()` - 91 edges
4. `StrategyConfig` - 73 edges
5. `LeveragedTrendHoldStrategy` - 63 edges
6. `OrderManager` - 60 edges
7. `BrokerScope` - 59 edges
8. `RiskManager` - 57 edges
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

## Communities (146 total, 7 thin omitted)

### Community 0 - "leveraged_trend_hold.rs"
Cohesion: 0.11
Nodes (10): LeveragedRapidReboundSnapshot, LeveragedReboundSnapshot, LeveragedTrendHoldMarketState, LeveragedTrendHoldPosition, LeveragedTrendHoldStrategy, LeveragedTrendSnapshot, HashMap, Option (+2 more)

### Community 1 - "simulation.rs"
Cohesion: 0.05
Nodes (96): SimulationAssumptions, broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc() (+88 more)

### Community 2 - "hooks.ts"
Cohesion: 0.04
Nodes (71): canUseTauriEvents(), useBackendEvents(), frontendLogger, OVERSEAS_CHART_PRESETS, useKisExecutedByRange(), useOverseasExecutedByRange(), BalanceResult, BalanceSummary (+63 more)

### Community 3 - "shared/api/commands.ts"
Cohesion: 0.05
Nodes (86): activateEmergencyStop(), addProfile(), checkConfig(), checkForUpdate(), checkTossOrderPreflight(), checkTossProfileConnection(), clearBuySuspension(), clearEmergencyStop() (+78 more)

### Community 4 - "RiskManager"
Cohesion: 0.06
Nodes (36): RiskStore, Option, PathBuf, Result, Self, config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope() (+28 more)

### Community 5 - "commands/trading.rs"
Cohesion: 0.13
Nodes (39): calculate_atr(), clear_buy_suspension(), fetch_account_risk_balance_krw(), fetch_overseas_tick(), fetch_toss_risk_balance_krw(), fetch_toss_tick(), get_trading_status(), is_market_closed_error() (+31 more)

### Community 6 - "ConsecutiveMoveStrategy"
Cohesion: 0.07
Nodes (16): ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, FailedBreakoutParams, FailedBreakoutState, FiftyTwoWeekHighParams, FiftyTwoWeekState, Default (+8 more)

### Community 7 - "leveragedTrendHoldEditorPanel.tsx"
Cohesion: 0.06
Nodes (61): usePreviewLeveragedTrendHold(), usePreviewStrategy(), useRefreshStockList(), ProfileFormState, clearExperimentSlots(), compactSnapshot(), ExperimentScope, loadExperimentSlots() (+53 more)

### Community 8 - "PositionTracker"
Cohesion: 0.14
Nodes (8): OverseasPosition, OverseasPositionTracker, PositionTracker, Default, HashMap, I, Self, String

### Community 9 - "ui/AppShell.tsx"
Cohesion: 0.10
Nodes (27): useRecentLogs(), useUpdateCheck(), SettingsState, useSettingsStore, LEVEL_COLORS, Log(), LogLevel, AppLogEntry (+19 more)

### Community 10 - "OrderManager"
Cohesion: 0.09
Nodes (20): HashSet, daily_order_limit_key(), estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), order_exchange_code(), OrderManager, PendingOrder (+12 more)

### Community 11 - "submission.rs"
Cohesion: 0.06
Nodes (72): RestOrderSide, BalanceItem, BalanceResponse, BalanceSummary, ChartCandle, ExecutedOrder, first_positive_u64(), KisOutput1 (+64 more)

### Community 12 - "dashboard/ui/Page.tsx"
Cohesion: 0.11
Nodes (27): useActivateEmergencyStop(), useBalance(), useBrokerHoldings(), useCheckConfig(), useClearBuySuspension(), useClearEmergencyStop(), useExchangeRate(), useExchangeRateStatus() (+19 more)

### Community 13 - "trading/ui/Page.tsx"
Cohesion: 0.08
Nodes (53): useModifyTossOrder(), useOverseasPrice(), usePlaceOrder(), usePlaceOverseasOrder(), usePrice(), useStockSearch(), useTossMarketSnapshot(), useTossOpenOrders() (+45 more)

### Community 14 - "TossOpenApiClient"
Cohesion: 0.11
Nodes (19): B, BrokerCurrency, Client, Option, Result, String, T, Vec (+11 more)

### Community 15 - "settings.rs"
Cohesion: 0.10
Nodes (44): AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_trading_type(), DetectTradingTypeResult, ExchangeRateSource (+36 more)

### Community 16 - "toss/types.rs"
Cohesion: 0.11
Nodes (40): Option, String, T, Value, Vec, TossAccount, TossApiResponse, TossBuyingPower (+32 more)

### Community 17 - "BrokerSymbol"
Cohesion: 0.10
Nodes (15): Debug, Display, Formatter, BrokerAdapterResult, BrokerCurrency, BrokerId, Default, Into (+7 more)

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
Nodes (15): Signal, MeanReversionParams, MeanReversionState, MeanReversionStrategy, Default, HashMap, Option, Self (+7 more)

### Community 22 - "database_projection.rs"
Cohesion: 0.11
Nodes (37): MySql, Postgres, drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract(), postgresql_rejects_invalid_credentials() (+29 more)

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
Cohesion: 0.06
Nodes (47): useAutoTradingBudget(), useUpdateAutoTradingBudget(), useAppConfig(), useStrategies(), useTossMarketCalendar(), useTradingStatus(), useUpdateStrategy(), amount() (+39 more)

### Community 27 - "commands/toss.rs"
Cohesion: 0.21
Nodes (27): check_toss_profile_connection(), list_toss_accounts(), list_toss_open_orders(), list_toss_open_orders_for_profile(), list_toss_profile_accounts(), lookup_toss_accounts_with_credentials(), mask_toss_account_no(), modify_toss_order() (+19 more)

### Community 28 - "tauri.conf.json"
Cohesion: 0.05
Nodes (40): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, Korean, app, security (+32 more)

### Community 29 - "ui/StockChart.tsx"
Cohesion: 0.07
Nodes (35): useChartData(), useOverseasChartData(), useTossChartData(), buildChartOptions(), CandlePoint, ChartType, CrosshairData, LinePoint (+27 more)

### Community 30 - "StrategyManager"
Cohesion: 0.12
Nodes (8): StrategySignal, Box, Default, F, Self, String, Vec, StrategyManager

### Community 31 - "BrokerScope"
Cohesion: 0.13
Nodes (24): RecentSideHistory, ScopedSymbol, BrokerScope, blocks_reentry_after_stop_loss_sell_on_same_day(), blocks_repeated_same_side_signal_inside_cooldown(), blocks_tiny_profit_after_costs(), blocks_whipsaw_after_repeated_opposite_signals(), buy_signal() (+16 more)

### Community 32 - "server/market.rs"
Cohesion: 0.17
Nodes (36): account_sync_error(), balance_handler(), broker_holdings_handler(), chart_handler(), ChartQuery, executed_handler(), invalid_order_input(), order_handler() (+28 more)

### Community 33 - "settings/ui/Page.tsx"
Cohesion: 0.10
Nodes (30): useBrokerRateLimitStatus(), useLogConfig(), useRefreshConfig(), useSaveWebConfig(), useSendTestDiscord(), useSetLogConfig(), useSetRefreshConfig(), useSetStockUpdateInterval() (+22 more)

### Community 34 - "KisRestClient"
Cohesion: 0.06
Nodes (49): OverseasPriceResponse, RequestBuilder, kis_http_client(), kis_read_min_interval(), KisOrderRejected, KisRestClient, read_kis_response_text(), Arc (+41 more)

### Community 35 - "AccountProfile"
Cohesion: 0.10
Nodes (52): buys_any_target_ticker_when_itself_trends_up(), buys_intraday_rebound_when_next_window_shows_buy_pressure(), buys_rapid_rebound_without_trend_snapshot_when_enabled(), downward_candles(), holds_failed_rebound_until_min_hold_observations_elapsed(), holds_when_protection_profit_has_not_activated(), ignores_intraday_rebound_by_default(), ignores_rapid_rebound_by_default() (+44 more)

### Community 36 - "src/api/types.ts"
Cohesion: 0.15
Nodes (25): useAddProfile(), useCheckTossProfileConnection(), useDeleteProfile(), useDetectProfileTradingType(), useDetectTradingType(), useListTossAccounts(), useListTossProfileAccounts(), useProfiles() (+17 more)

### Community 37 - "rest/types.rs"
Cohesion: 0.18
Nodes (24): FixedOffset, calendar_override_represents_holiday_without_fallback(), calendar_session_window_checks_time_range(), is_domestic_symbol(), is_krx_open(), is_krx_open_at(), is_market_open_for(), is_market_open_for_with_calendar() (+16 more)

### Community 38 - "databaseManagementSection.tsx"
Cohesion: 0.15
Nodes (28): useClearDatabaseTables(), useCreateDatabaseTables(), useDatabaseConfig(), useDropDatabaseTables(), useExportDatabaseToJson(), useImportJsonToDatabase(), useJsonStorageInventory(), useSaveDatabaseConfig() (+20 more)

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
Cohesion: 0.15
Nodes (15): live_tick_and_preview_emit_identical_signals_for_normalized_fixture(), BrokerPositionSnapshot, default_strategy_broker_id(), BrokerId, BrokerMarket, Into, Option, Self (+7 more)

### Community 43 - "domain.rs"
Cohesion: 0.09
Nodes (38): default_place_order_returns_unsupported_with_broker_id(), broker_scope_serializes_with_camel_case_fields(), BrokerAccountId, BrokerCandle, BrokerClientOrderId, BrokerCurrency, BrokerHolding, BrokerId (+30 more)

### Community 44 - "archive.rs"
Cohesion: 0.17
Nodes (23): collect_trade_archive_stats(), collect_trade_archive_stats_active(), collect_trade_day_dirs(), dir_size_bytes(), get_trade_archive_config(), get_trade_archive_stats(), purge_old_trade_files(), purge_trade_archive_active() (+15 more)

### Community 45 - "commands/records.rs"
Cohesion: 0.21
Nodes (26): FrontendLogInput, get_kis_executed_by_range(), get_overseas_executed_by_range(), get_recent_logs(), get_stats_by_range(), get_today_executed(), get_today_overseas_executed(), get_today_stats() (+18 more)

### Community 46 - "PriceConditionStrategy"
Cohesion: 0.10
Nodes (10): PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick(), HashMap, Option, Self, String (+2 more)

### Community 47 - "detect.rs"
Cohesion: 0.13
Nodes (17): classify_token_response(), detect_trading_type(), DetectedTradingType, DetectTokenReq, DetectTradingTypeError, is_paper_key_rejected_by_real_domain(), is_real_key_rejected_by_paper_domain(), mentions_app_key() (+9 more)

### Community 48 - "submit_toss_small_buy_verification_for_profile"
Cohesion: 0.17
Nodes (22): broker_money_view_amount(), is_toss_order_final(), money_view_from_decimal(), poll_toss_order_detail(), Arc, BrokerCurrency, BrokerId, BrokerMarket (+14 more)

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
Cohesion: 0.18
Nodes (14): OrderRecord, OrderSide, OrderStore, parallel_appends_do_not_lose_order_records(), BrokerId, Into, Mutex, NaiveDate (+6 more)

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
Cohesion: 0.20
Nodes (16): account_less_scope_matches_any_account_of_same_broker(), default_currency(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), BrokerId, Option (+8 more)

### Community 65 - "release-version.mjs"
Cohesion: 0.15
Nodes (17): args, assertGitClean(), assertTagAvailable(), branchName, changedFiles, currentBranchName(), escapeRegExp(), fail() (+9 more)

### Community 66 - "MovingAverageCrossStrategy"
Cohesion: 0.11
Nodes (9): MaCrossParams, MaCrossState, MovingAverageCrossStrategy, Default, HashMap, Option, Self, String (+1 more)

### Community 67 - "scripts"
Cohesion: 0.09
Nodes (23): scripts, build, build:app, build:app:debug, build:web, check:fsd, check:graphify, check:lockfiles (+15 more)

### Community 68 - "place_toss_order"
Cohesion: 0.20
Nodes (16): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, BrokerCurrency, CmdResult, Option (+8 more)

### Community 69 - "conflicts.rs"
Cohesion: 0.22
Nodes (10): LeveragedTrendHoldEntry, LeveragedTrendHoldParams, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldTimedCandle, lth_default_qty(), parse_hhmm(), Default, F (+2 more)

### Community 70 - "toss/tests.rs"
Cohesion: 0.11
Nodes (7): new_toss_client_order_id(), String, maps_holding_to_broker_domain(), maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request(), validates_order_create_request_shape()

### Community 71 - "commands/risk.rs"
Cohesion: 0.26
Nodes (18): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+10 more)

### Community 72 - ".new"
Cohesion: 0.22
Nodes (11): AsyncMutex, Arc, Into, Self, shared_toss_token_state(), toss_token_state_key(), TossTokenState, Client (+3 more)

### Community 73 - "storage/database.rs"
Cohesion: 0.16
Nodes (13): MySqlSslMode, PgSslMode, connect_pool(), create_missing_mariadb_database(), create_missing_postgres_database(), inventory_from_documents(), inventory_groups_documents_by_top_level_category(), is_missing_database_error() (+5 more)

### Community 74 - "KisWebSocketClient"
Cohesion: 0.14
Nodes (16): AccountProfile, AppConfig, default_broker_id(), DiscordConfig, load_secure_config(), non_empty(), ProfilesConfig, Arc (+8 more)

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
Cohesion: 0.21
Nodes (16): BudgetCurrencyView, allocation_cannot_withdraw_committed_cash_or_overflow(), AutoTradingBudgetView, budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent(), BudgetAccount, BudgetLedger, cost(), manager() (+8 more)

### Community 81 - "BrokerQuantity"
Cohesion: 0.26
Nodes (13): buy_preflight_adds_commission_and_blocks_when_cash_is_short(), buy_preflight_passes_when_required_cash_is_available(), evaluate_order_preflight(), format_money_amount(), format_quantity(), OrderPreflightConstraints, OrderPreflightDecision, OrderPreflightInput (+5 more)

### Community 82 - ".write_document"
Cohesion: 0.28
Nodes (12): category_from_key(), document_key(), LocalDocument, read_managed_json(), read_managed_json_backup(), Option, Path, String (+4 more)

### Community 83 - "KisBrokerAdapter"
Cohesion: 0.12
Nodes (27): AutoTradingBudgetInput, AutoTradingBudgetInput, budget_error(), checked_budget_scope(), get_auto_trading_budget(), Arc, AutoTradingBudgetView, BrokerId (+19 more)

### Community 84 - "support.rs"
Cohesion: 0.24
Nodes (13): Result, BrokerMarket, Option, Result, toss_currency_code(), toss_market(), validate_client_order_id(), validate_iso_date() (+5 more)

### Community 85 - "leveragedTrendHoldPreviewChart.tsx"
Cohesion: 0.25
Nodes (13): chartTimeToMs(), formatChartTimeKst(), formatKstMs(), pad2(), parseTimeMs(), Props, StrategyPreviewChart(), StrategyPreviewChartSignal (+5 more)

### Community 86 - "broker/adapter.rs"
Cohesion: 0.13
Nodes (14): BrokerAdapter, BrokerAdapterError, DummyAdapter, BrokerAdapterResult, BrokerId, Error, Send, String (+6 more)

### Community 87 - ".to_broker_holding"
Cohesion: 0.15
Nodes (16): initialize_strategy_warmup(), OhlcCandle, Band, BollingerParams, candle(), disabled_option_preserves_entries_and_buffer_is_bounded(), downward_breakout_and_insufficient_history_block_entries(), flat_squeeze_needs_completed_upward_breakout() (+8 more)

### Community 88 - "initialize_active_strategy_history"
Cohesion: 0.32
Nodes (11): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), initialize_active_strategy_history(), Arc, Mutex, Option (+3 more)

### Community 89 - "PendingOrderStore"
Cohesion: 0.29
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
Cohesion: 0.15
Nodes (18): usePendingOrders(), useStatsByRange(), useTradesByRange(), FilledOrdersPanel(), fmt(), PendingOrdersPanel(), SortDir, todayStr() (+10 more)

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
Cohesion: 0.10
Nodes (12): queryClient, dashboardRoute, historyRoute, logRoute, Register, rootRoute, router, routeTree (+4 more)

### Community 102 - "BrokerAccountId"
Cohesion: 0.07
Nodes (8): FailedBreakoutStrategy, FiftyTwoWeekHighStrategy, HashMap, String, VolatilityExpansionStrategy, Send, Sync, Strategy

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
Cohesion: 0.24
Nodes (9): definitively_rejected(), Error, Option, StatusCode, String, Value, TossApiError, TossErrorResponse (+1 more)

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
Cohesion: 0.15
Nodes (13): 10. 백그라운드 작업, 11. 변경 도구와 검증, 1. 시스템 경계, 2. Frontend, 3. IPC 계약, 4. AppState와 동시성, 5. Broker와 provider, 6. 주문 파이프라인 (+5 more)

### Community 129 - "Position"
Cohesion: 0.18
Nodes (3): Position, Option, Vec

### Community 131 - "StrategyView"
Cohesion: 0.24
Nodes (8): build_strategy_view(), BrokerId, HashMap, Option, String, Value, Vec, StrategyView

### Community 132 - ".new"
Cohesion: 0.26
Nodes (13): BrokerOrderSide, check_toss_order_preflight(), check_toss_order_preflight_for_profile(), parse_toss_order_side(), BrokerCurrency, BrokerMarket, BrokerMoneyView, Option (+5 more)

### Community 134 - "@tauri-apps/api"
Cohesion: 0.38
Nodes (6): Mutex, NaiveDate, PathBuf, Result, Vec, TradeStore

### Community 135 - "verify-release-artifacts.mjs"
Cohesion: 0.18
Nodes (9): args, artifactDir, artifacts, checksumPath, manifestPath, names, records, requireSignatures (+1 more)

### Community 136 - "작업 기록: <짧은 제목>"
Cohesion: 0.18
Nodes (11): 1. 메타데이터, 3. 시작 상태, 4. 조사 근거, 5. 영향 범위, 6. 금융·자동매매 안전 검토, 7. 구현 계획, 8. 도구 선택 기록, 9. 검증 계획과 결과 (+3 more)

### Community 137 - "KISAutoTrade 검증 Harness 지도"
Cohesion: 0.20
Nodes (10): 1. 환경 기준, 2. 기본 정적·단위 게이트, 3. 변경 유형별 필수 조합, 4. Focused Rust 테스트, 5. Playwright, 6. 구조·생성물 검사, 7. Provider·릴리스 검사, 8. CI와 로컬의 대응 (+2 more)

### Community 138 - "KISAutoTrade 탐색 지도"
Cohesion: 0.20
Nodes (10): 1. 런타임 한눈에 보기, 2. 작업별 첫 진입점, 3. Frontend 경계, 4. Backend 경계, 5. 중요한 동기화 묶음, 6. 데이터와 민감 영역, 7. 지도 유지 원칙, IPC 계약 변경 (+2 more)

### Community 139 - "CurrencyLedger"
Cohesion: 0.42
Nodes (6): BTreeMap, BudgetCurrencyView, BudgetReservation, CurrencyLedger, Option, String

### Community 140 - "verify-lockfiles.mjs"
Cohesion: 0.22
Nodes (6): appEntry, escapedName, npmLock, pkg, rootDir, workspaceLock

### Community 141 - "modify_toss_order_for_profile"
Cohesion: 0.38
Nodes (7): modify_toss_order_for_profile(), Arc, Mutex, OrderManager, toss_decimal_equals(), toss_decimal_to_storage_units(), parse_decimal_amount()

### Community 142 - "10. 완료 보고"
Cohesion: 0.33
Nodes (6): 10. 완료 보고, 기존 사용자 변경 보존 결과, 변경 요약, 변경 파일(절대경로), 알려진 제한/잔여 위험, 운영자 확인 필요

### Community 144 - "2. 목표와 비목표"
Cohesion: 0.50
Nodes (4): 2. 목표와 비목표, 목표, 비목표, 사용자 완료 조건

## Knowledge Gaps
- **312 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+307 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `AppState` to `simulation.rs`, `RiskManager`, `.new`, `@tauri-apps/api`, `commands/trading.rs`, `PositionTracker`, `settings.rs`, `accounts.rs`, `toss_market.rs`, `commands/toss.rs`, `StrategyManager`, `KisRestClient`, `DatabaseManager`, `archive.rs`, `commands/records.rs`, `submit_toss_small_buy_verification_for_profile`, `OrderRecord`, `StockStore`, `commands/database.rs`, `commands/market.rs`, `DiscordNotifier`, `place_toss_order`, `commands/risk.rs`, `KisWebSocketClient`, `StatsStore`, `KisBrokerAdapter`, `StrategyStore`, `TradeStore`?**
  _High betweenness centrality (0.145) - this node is a cross-community bridge._
- **Why does `ServerState` connect `ServerState` to `server/market.rs`, `KisRestClient`, `RiskManager`, `@tauri-apps/api`, `DatabaseManager`, `PositionTracker`, `server/records.rs`, `KisWebSocketClient`, `StatsStore`, `profiles.rs`, `KisBrokerAdapter`, `server/trading.rs`, `StockStore`, `security.rs`, `active_profile`, `StrategyStore`, `DiscordNotifier`, `StrategyManager`?**
  _High betweenness centrality (0.100) - this node is a cross-community bridge._
- **Why does `KisRestClient` connect `KisRestClient` to `commands/trading.rs`, `AppState`, `OrderManager`, `submission.rs`, `fills.rs`, `accounts.rs`, `broker/adapter.rs`, `initialize_active_strategy_history`, `RateLimitScheduler`, `ServerState`?**
  _High betweenness centrality (0.082) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _312 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `leveraged_trend_hold.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11025641025641025 - nodes in this community are weakly interconnected._
- **Should `simulation.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.051970302684180465 - nodes in this community are weakly interconnected._
- **Should `hooks.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.037037037037037035 - nodes in this community are weakly interconnected._