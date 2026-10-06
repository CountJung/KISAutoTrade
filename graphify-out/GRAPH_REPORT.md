# Graph Report - KISAutoTrade  (2026-10-06)

## Corpus Check
- 233 files · ~181,754 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 3920 nodes · 9835 edges · 160 communities (157 shown, 3 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 762 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `015a28a8`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- storage/database.rs
- shared/api/commands.ts
- strategyPreviewPanel.tsx
- rest/types.rs
- hooks.ts
- submit_toss_small_buy_verification_for_profile
- VolatilityExpansionStrategy
- MomentumStrategy
- bounded_window_with_extra
- KisRestClient
- scripts
- router/index.ts
- trading/ui/Page.tsx
- commands/risk.rs
- DatabaseManager
- TossOpenApiClient
- ConsecutiveMoveStrategy
- TokenManager
- toss_market.rs
- toss/orders.rs
- project-map.mjs
- KisBrokerAdapter
- commands.rs
- RateLimitScheduler
- tauri.conf.json
- settings/ui/Page.tsx
- StrategyManager
- toss/types.rs
- AccountProfile
- support.rs
- strategy/ui/Page.tsx
- accounts.rs
- .ok
- settings.rs
- log/ui/Page.tsx
- .new
- strategy_preview.rs
- broker/adapter.rs
- fills.rs
- server/records.rs
- commands/records.rs
- PositionTracker
- model/settingsStore.ts
- .with_credentials
- 반영됨
- .to_broker_holding
- commands/trading.rs
- package.json
- ServerState
- TossAccessToken
- OverseasExecutedOrder
- leveraged_trend_hold.rs
- ui/LayoutResizer.tsx
- BrokerQuantity
- market/mod.rs
- simulation.rs
- BollingerParams
- BrokerScope
- OrderManager
- databaseManagementSection.tsx
- connect_pool
- TossBrokerAdapter
- database_schema.rs
- BrokerAccountId
- commands/toss.rs
- run
- PriceConditionStrategy
- MovingAverageCrossStrategy
- accountProfiles.tsx
- submission.rs
- profiles.rs
- leveraged_trend_hold/tests.rs
- BrokerSymbol
- server/market.rs
- database_contract_tests.rs
- LeveragedTrendHoldStrategy
- compilerOptions
- RiskManager
- 작업 기록: <짧은 제목>
- ui/StockChart.tsx
- release-version.mjs
- dependencies
- TradeRecord
- detect.rs
- Signal
- react
- database_io.rs
- devDependencies
- security.rs
- preview_strategy_from_candles
- DatabaseConfig
- read_json_or_default
- KISAutoTrade 검증 Harness 지도
- .new
- active_profile
- normalize_rows
- StrategyConfig
- OhlcCandle
- .preview_signals_with_execution
- LeveragedTrendHoldStrategy
- ui/OverseasStockChart.tsx
- bollinger.rs
- KISAutoTrade 아키텍처
- graphify.mjs
- .default
- server/trading.rs
- search_stock
- StockStore
- KISAutoTrade 탐색 지도
- TradeStore
- AppState
- 작업 기록: AppleDouble 점검 및 SOXQ-P1-03
- 작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단
- day_event_tests.rs
- conflicts.rs
- place_toss_order
- 작업 기록: SOXQ-P1-05 일봉 진단과 장중 평가 분리
- verify-release-artifacts.mjs
- input
- ui/AppShell.tsx
- checked_budget_scope
- OrderRecord
- StatsStore
- 프로젝트 상태·캐시 정리와 SOXQ-P1-01
- buffer_tests.rs
- LeveragedTrendHoldStrategy
- check-fsd-imports.mjs
- verify-lockfiles.mjs
- LeveragedTrendHoldTimedCandle
- LeveragedTrendHoldStrategy
- verify-toss-openapi.mjs
- compilerOptions
- .subscribe
- DatabasePool
- ReplayAssessmentView
- simulation/tests.rs
- check
- PendingOrderStore
- StrategyStore
- .save_snapshot
- model/tradingStore.ts
- error.rs
- NotificationService
- setup-local.sh
- kis-auto-trade

## God Nodes (most connected - your core abstractions)
1. `AppState` - 122 edges
2. `ServerState` - 109 edges
3. `invoke()` - 91 edges
4. `StrategyConfig` - 79 edges
5. `OrderManager` - 60 edges
6. `BrokerScope` - 59 edges
7. `RiskManager` - 57 edges
8. `KisRestClient` - 47 edges
9. `BrokerSymbol` - 47 edges
10. `TossOpenApiClient` - 44 edges

## Surprising Connections (you probably didn't know these)
- `fetch_account_risk_balance_krw()` --calls--> `parse_amount_f64()`  [INFERRED]
  src-tauri/src/commands/trading.rs → src-tauri/src/commands.rs
- `deterministic_replay_applies_costs_and_separates_signals_from_fills()` --calls--> `run_backtest()`  [INFERRED]
  src-tauri/src/trading/simulation/tests.rs → src-tauri/src/trading/simulation.rs
- `identical_fixture_has_identical_report_and_hash()` --calls--> `run_backtest()`  [INFERRED]
  src-tauri/src/trading/simulation/tests.rs → src-tauri/src/trading/simulation.rs
- `sell_guard_uses_the_same_round_trip_cost_assumptions_as_settlement()` --calls--> `run_backtest()`  [INFERRED]
  src-tauri/src/trading/simulation/tests.rs → src-tauri/src/trading/simulation.rs
- `maps_holding_to_broker_domain()` --calls--> `BrokerAccountId`  [INFERRED]
  src-tauri/src/broker/toss/tests.rs → src-tauri/src/broker/domain.rs

## Import Cycles
- 1-file cycle: `src-tauri/src/trading/simulation/assessment.rs -> src-tauri/src/trading/simulation/assessment.rs`
- 2-file cycle: `src-tauri/src/storage/database.rs -> src-tauri/src/storage/database_schema.rs -> src-tauri/src/storage/database.rs`
- 2-file cycle: `src-tauri/src/server/mod.rs -> src-tauri/src/server/profiles.rs -> src-tauri/src/server/mod.rs`

## Communities (160 total, 3 thin omitted)

### Community 0 - "storage/database.rs"
Cohesion: 0.19
Nodes (16): category_from_key(), document_key(), inventory_from_documents(), inventory_groups_documents_by_top_level_category(), LocalDocument, quarantine_corrupt_managed_json(), read_managed_json(), read_managed_json_backup() (+8 more)

### Community 1 - "shared/api/commands.ts"
Cohesion: 0.04
Nodes (89): handleLookupTossAccounts(), handleSubmit(), activateEmergencyStop(), addProfile(), checkConfig(), checkForUpdate(), checkTossOrderPreflight(), checkTossProfileConnection() (+81 more)

### Community 2 - "strategyPreviewPanel.tsx"
Cohesion: 0.05
Nodes (69): usePreviewStrategy(), clearExperimentSlots(), compactSnapshot(), ExperimentScope, loadExperimentSlots(), normalizePart(), readIndex(), saveExperimentSlot() (+61 more)

### Community 3 - "rest/types.rs"
Cohesion: 0.12
Nodes (26): BalanceItem, BalanceResponse, BalanceSummary, ChartCandle, ExecutedOrder, KisOutput1, OrderRequest, OrderResponse (+18 more)

### Community 4 - "hooks.ts"
Cohesion: 0.04
Nodes (66): canUseTauriEvents(), useBackendEvents(), frontendLogger, OVERSEAS_CHART_PRESETS, AppLogEntry, BalanceResult, BalanceSummary, BrokerCurrency (+58 more)

### Community 5 - "submit_toss_small_buy_verification_for_profile"
Cohesion: 0.17
Nodes (23): broker_money_view_amount(), is_toss_order_final(), money_view_from_decimal(), poll_toss_order_detail(), Arc, BrokerCurrency, BrokerId, BrokerMarket (+15 more)

### Community 6 - "VolatilityExpansionStrategy"
Cohesion: 0.05
Nodes (20): config(), fifty_two_week_requires_full_window_and_uses_last_completed_high_and_close(), FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy, FiftyTwoWeekState, Default, HashMap, Option (+12 more)

### Community 7 - "MomentumStrategy"
Cohesion: 0.06
Nodes (21): config(), deviation_counts_current_price_as_last_required_observation(), deviation_history_is_ready_before_first_evaluation_and_bounded(), DeviationParams, DeviationStrategy, momentum_counts_first_comparison_but_excludes_seed_prices(), momentum_history_does_not_create_or_erase_order_state(), MomentumParams (+13 more)

### Community 8 - "bounded_window_with_extra"
Cohesion: 0.07
Nodes (23): config(), daily(), MeanReversionParams, MeanReversionState, MeanReversionStrategy, Default, HashMap, Item (+15 more)

### Community 9 - "KisRestClient"
Cohesion: 0.18
Nodes (14): OverseasPriceResponse, RequestBuilder, KisRestClient, read_kis_response_text(), AtomicBool, ChartCandle, ExecutedOrder, HeaderMap (+6 more)

### Community 10 - "scripts"
Cohesion: 0.09
Nodes (23): scripts, build, build:app, build:app:debug, build:web, check:fsd, check:graphify, check:lockfiles (+15 more)

### Community 11 - "router/index.ts"
Cohesion: 0.10
Nodes (12): queryClient, dashboardRoute, historyRoute, logRoute, Register, rootRoute, router, routeTree (+4 more)

### Community 12 - "trading/ui/Page.tsx"
Cohesion: 0.06
Nodes (60): useAutoTradingBudget(), useKisExecutedByRange(), useModifyTossOrder(), useOverseasExecutedByRange(), useOverseasPrice(), usePlaceOrder(), usePlaceOverseasOrder(), usePrice() (+52 more)

### Community 13 - "commands/risk.rs"
Cohesion: 0.26
Nodes (18): activate_emergency_stop(), build_risk_view(), clear_emergency_stop(), get_pending_orders(), get_risk_config(), pending_order_to_view(), PendingOrderView, persist_risk_runtime() (+10 more)

### Community 14 - "DatabaseManager"
Cohesion: 0.19
Nodes (13): MySqlPool, PgPool, DatabaseManager, DatabasePool, install_database_manager(), Arc, DatabaseStatusView, DatabaseTransferResult (+5 more)

### Community 15 - "TossOpenApiClient"
Cohesion: 0.11
Nodes (16): B, BrokerCurrency, Client, Option, Result, String, T, Vec (+8 more)

### Community 16 - "ConsecutiveMoveStrategy"
Cohesion: 0.07
Nodes (18): config(), consecutive_counts_current_price_and_uses_active_position_branch(), consecutive_history_preserves_flat_and_existing_positions(), ConsecutiveMoveParams, ConsecutiveMoveState, ConsecutiveMoveStrategy, failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation(), failed_breakout_history_retains_actual_breakout_reference() (+10 more)

### Community 17 - "TokenManager"
Cohesion: 0.19
Nodes (12): AccessToken, Arc, Client, DateTime, Mutex, Option, Result, Self (+4 more)

### Community 18 - "toss_market.rs"
Cohesion: 0.13
Nodes (38): get_toss_chart_data(), get_toss_chart_data_for_profile(), get_toss_market_calendar(), get_toss_market_calendar_for_profile(), get_toss_market_snapshot(), get_toss_market_snapshot_for_profile(), get_toss_stock_safety(), get_toss_stock_safety_for_profile() (+30 more)

### Community 19 - "toss/orders.rs"
Cohesion: 0.24
Nodes (13): Option, Self, String, Vec, TossOrder, TossOrderCreateRequest, TossOrderExecution, TossOrderListQuery (+5 more)

### Community 20 - "project-map.mjs"
Cohesion: 0.18
Nodes (16): buildTree(), compareNames(), currentDocument, fail(), generateInventory(), listRepositoryFiles(), mode, nextDocument (+8 more)

### Community 21 - "KisBrokerAdapter"
Cohesion: 0.17
Nodes (9): KisBrokerAdapter, maps_kis_balance_item_to_broker_holding(), Arc, BalanceItem, BrokerAdapterResult, BrokerId, Option, Self (+1 more)

### Community 22 - "commands.rs"
Cohesion: 0.16
Nodes (11): balance_summary_total_krw(), CmdError, normalize_overseas_order_exchange(), parse_amount_f64(), parse_amount_i64(), BalanceSummary, Error, Option (+3 more)

### Community 23 - "RateLimitScheduler"
Cohesion: 0.11
Nodes (30): FnOnce, Instant, IntoIterator, applies_retry_after_to_group_pause(), epoch_ms_now(), parse_delay_header(), rate_limit_reset_delay(), RateLimitGroupState (+22 more)

### Community 24 - "tauri.conf.json"
Cohesion: 0.05
Nodes (40): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, Korean, app, security (+32 more)

### Community 25 - "settings/ui/Page.tsx"
Cohesion: 0.09
Nodes (29): useBrokerRateLimitStatus(), useLogConfig(), useRefreshConfig(), useSaveWebConfig(), useSendTestDiscord(), useSetLogConfig(), useSetRefreshConfig(), useSetStockUpdateInterval() (+21 more)

### Community 26 - "StrategyManager"
Cohesion: 0.11
Nodes (12): StrategySignal, build_strategy(), live_high_excludes_unconfirmed_latest_bar_without_shortening_252_requirement(), Box, BrokerId, Default, F, Option (+4 more)

### Community 27 - "toss/types.rs"
Cohesion: 0.11
Nodes (40): Option, String, T, Value, Vec, TossAccount, TossApiResponse, TossBuyingPower (+32 more)

### Community 28 - "AccountProfile"
Cohesion: 0.14
Nodes (16): AccountProfile, AppConfig, default_broker_id(), DiscordConfig, load_secure_config(), non_empty(), ProfilesConfig, Arc (+8 more)

### Community 29 - "support.rs"
Cohesion: 0.19
Nodes (16): Result, new_toss_client_order_id(), BrokerMarket, Option, Result, String, toss_currency_code(), toss_market() (+8 more)

### Community 30 - "strategy/ui/Page.tsx"
Cohesion: 0.04
Nodes (64): usePreviewLeveragedTrendHold(), useRefreshStockList(), useStockSearch(), useStrategies(), useTossMarketCalendar(), useUpdateStrategy(), BollingerControls(), boolParam() (+56 more)

### Community 31 - "accounts.rs"
Cohesion: 0.12
Nodes (40): AppConfigView, OverseasBalanceItem, OverseasBalanceSummary, add_profile(), AddProfileInput, apply_active_profile(), BalanceResult, broker_market_sort_key() (+32 more)

### Community 32 - ".ok"
Cohesion: 0.21
Nodes (27): MySql, Postgres, KisOutput1<T>, balance_snapshot_projects_nested_holdings(), clear_mariadb(), clear_postgres(), delete_source_mariadb(), delete_source_postgres() (+19 more)

### Community 33 - "settings.rs"
Cohesion: 0.09
Nodes (46): fetch_usd_krw_rate(), Result, AppConfigView, BrokerRateLimitScopeView, check_config(), check_for_update(), ConfigDiagnostic, detect_trading_type() (+38 more)

### Community 34 - "log/ui/Page.tsx"
Cohesion: 0.23
Nodes (13): useRecentLogs(), LEVEL_COLORS, Log(), LogLevel, clampNumber(), readStoredNumber(), writeStoredNumber(), chip() (+5 more)

### Community 35 - ".new"
Cohesion: 0.23
Nodes (9): kis_http_client(), kis_read_min_interval(), KisOrderRejected, Arc, Client, Duration, RwLock, Self (+1 more)

### Community 36 - "strategy_preview.rs"
Cohesion: 0.13
Nodes (34): broker_candles_to_chart(), broker_candles_to_ohlc(), broker_candles_to_timed_ohlc(), broker_money_amount(), broker_money_to_strategy_units(), chart_amount_to_units(), chart_candle_to_ohlc(), daily_chart_time() (+26 more)

### Community 37 - "broker/adapter.rs"
Cohesion: 0.23
Nodes (9): BrokerAdapter, BrokerAdapterError, DummyAdapter, BrokerAdapterResult, BrokerId, Error, Send, String (+1 more)

### Community 38 - "fills.rs"
Cohesion: 0.07
Nodes (42): DiscordNotifier, Client, Result, Self, String, test_discord_message_format(), NotificationEvent, NotificationLevel (+34 more)

### Community 39 - "server/records.rs"
Cohesion: 0.20
Nodes (29): archive_config_handler(), archive_stats_handler(), DateRangeQuery, frontend_log_handler(), FrontendLogBody, kis_executed_handler(), log_config_handler(), pending_orders_handler() (+21 more)

### Community 40 - "commands/records.rs"
Cohesion: 0.21
Nodes (26): FrontendLogInput, get_kis_executed_by_range(), get_overseas_executed_by_range(), get_recent_logs(), get_stats_by_range(), get_today_executed(), get_today_overseas_executed(), get_today_stats() (+18 more)

### Community 41 - "PositionTracker"
Cohesion: 0.05
Nodes (34): get_positions(), get_strategies(), PositionView, BrokerId, CmdResult, From, Option, Self (+26 more)

### Community 43 - ".with_credentials"
Cohesion: 0.33
Nodes (6): Into, Self, String, get_active_toss_calendar_override(), Arc, RwLock

### Community 44 - "반영됨"
Cohesion: 0.06
Nodes (59): useActivateEmergencyStop(), useAppConfig(), useBalance(), useBrokerHoldings(), useCheckConfig(), useClearBuySuspension(), useClearEmergencyStop(), useExchangeRate() (+51 more)

### Community 45 - ".to_broker_holding"
Cohesion: 0.33
Nodes (5): market_from_currency(), BrokerCurrency, toss_currency(), BrokerAdapterResult, Result

### Community 46 - "commands/trading.rs"
Cohesion: 0.07
Nodes (64): FixedOffset, calculate_atr(), clear_buy_suspension(), fetch_account_risk_balance_krw(), fetch_overseas_tick(), fetch_toss_risk_balance_krw(), fetch_toss_tick(), get_trading_status() (+56 more)

### Community 47 - "package.json"
Cohesion: 0.25
Nodes (7): engines, node, npm, name, private, type, version

### Community 48 - "ServerState"
Cohesion: 0.15
Nodes (40): activate_emergency_handler(), check_config_handler(), check_update_handler(), clear_buy_suspension_handler(), clear_emergency_handler(), exchange_rate_handler(), exchange_rate_status_handler(), guess_mime() (+32 more)

### Community 49 - "TossAccessToken"
Cohesion: 0.36
Nodes (6): DateTime, From, Self, Utc, TossAccessToken, TossAccessTokenStatus

### Community 51 - "OverseasExecutedOrder"
Cohesion: 0.53
Nodes (3): first_positive_u64(), OverseasExecutedOrder, parse_decimal_cents()

### Community 52 - "leveraged_trend_hold.rs"
Cohesion: 0.12
Nodes (31): lth_default_adx_period(), lth_default_breakeven_buffer(), lth_default_buy_adx(), lth_default_buy_rsi(), lth_default_ema_long(), lth_default_ema_short(), lth_default_entry_end(), lth_default_entry_failure_observations() (+23 more)

### Community 53 - "ui/LayoutResizer.tsx"
Cohesion: 0.40
Nodes (3): LayoutResizer(), LayoutResizerProps, ResizeDirection

### Community 54 - "BrokerQuantity"
Cohesion: 0.14
Nodes (21): default_place_order_returns_unsupported_with_broker_id(), BrokerQuantity, Into, Self, serializes_currency_as_uppercase(), daily_diagnostic_contains_one_completed_bar_and_no_synthetic_entry_observation(), minute_replay_warmup_excludes_replay_day_and_future_daily_candles(), failed_buy_uses_signal_price_even_for_domestic_market_order() (+13 more)

### Community 55 - "market/mod.rs"
Cohesion: 0.19
Nodes (23): KrxItem, KrxProxyItem, KrxProxySearchResponse, KrxResponse, lookup_name_by_code(), NaverAcItem, NaverAcResponse, Option (+15 more)

### Community 56 - "simulation.rs"
Cohesion: 0.14
Nodes (35): apply_slippage(), BacktestPhaseView, BacktestReportView, BacktestSummaryView, BacktestTradeView, bps_cost(), current_equity(), EquityPointView (+27 more)

### Community 57 - "BollingerParams"
Cohesion: 0.50
Nodes (3): BollingerParams, Default, Self

### Community 58 - "BrokerScope"
Cohesion: 0.15
Nodes (23): BTreeMap, BudgetCurrencyView, BrokerScope, allocation_cannot_withdraw_committed_cash_or_overflow(), AutoTradingBudgetView, budget_reservation_fill_restart_and_cancel_are_isolated_and_idempotent(), BudgetAccount, BudgetCurrencyView (+15 more)

### Community 59 - "OrderManager"
Cohesion: 0.10
Nodes (20): HashSet, daily_order_limit_key(), estimate_order_amount_krw(), is_insufficient_balance_error(), is_paper_unsupported_error(), order_exchange_code(), OrderManager, PendingOrder (+12 more)

### Community 61 - "databaseManagementSection.tsx"
Cohesion: 0.10
Nodes (29): useClearDatabaseTables(), useCreateDatabaseTables(), useDatabaseConfig(), useDropDatabaseTables(), useExportDatabaseToJson(), useImportJsonToDatabase(), useJsonStorageInventory(), useSaveDatabaseConfig() (+21 more)

### Community 62 - "connect_pool"
Cohesion: 0.28
Nodes (9): MySqlSslMode, PgSslMode, connect_pool(), create_missing_mariadb_database(), create_missing_postgres_database(), is_missing_database_error(), Duration, Error (+1 more)

### Community 63 - "TossBrokerAdapter"
Cohesion: 0.11
Nodes (12): Debug, Display, Formatter, BrokerAdapterResult, BrokerCurrency, BrokerId, Default, Option (+4 more)

### Community 64 - "database_schema.rs"
Cohesion: 0.60
Nodes (5): create(), drop(), mariadb_ddl(), postgres_ddl(), Result

### Community 66 - "BrokerAccountId"
Cohesion: 0.19
Nodes (22): broker_scope_serializes_with_camel_case_fields(), BrokerAccountId, BrokerCandle, BrokerClientOrderId, BrokerCurrency, BrokerHolding, BrokerId, BrokerMarket (+14 more)

### Community 69 - "commands/toss.rs"
Cohesion: 0.13
Nodes (45): check_toss_order_preflight(), check_toss_order_preflight_for_profile(), check_toss_profile_connection(), list_toss_accounts(), list_toss_open_orders(), list_toss_open_orders_for_profile(), list_toss_profile_accounts(), lookup_toss_accounts_with_credentials() (+37 more)

### Community 74 - "run"
Cohesion: 0.07
Nodes (47): FormatTime, collect_trade_archive_stats(), collect_trade_archive_stats_active(), collect_trade_day_dirs(), dir_size_bytes(), get_trade_archive_config(), get_trade_archive_stats(), purge_old_trade_files() (+39 more)

### Community 81 - "PriceConditionStrategy"
Cohesion: 0.10
Nodes (10): PriceConditionParams, PriceConditionStrategy, PriceConditionSymbolConfig, rejected_buy_can_restore_flat_position_before_next_tick(), HashMap, Option, Self, String (+2 more)

### Community 82 - "MovingAverageCrossStrategy"
Cohesion: 0.13
Nodes (12): config(), initializer_restores_previous_averages_for_first_evaluation_cross(), insufficient_history_has_no_previous_long_average(), MaCrossParams, MaCrossState, MovingAverageCrossStrategy, Default, HashMap (+4 more)

### Community 83 - "accountProfiles.tsx"
Cohesion: 0.11
Nodes (28): useAddProfile(), useCheckTossProfileConnection(), useDeleteProfile(), useDetectProfileTradingType(), useDetectTradingType(), useListTossAccounts(), useListTossProfileAccounts(), useSetActiveProfile() (+20 more)

### Community 85 - "submission.rs"
Cohesion: 0.13
Nodes (42): RestOrderSide, append_failed_order(), build_kis_pending_order(), build_pending_order(), build_provider_request(), build_toss_pending_order(), current_position_snapshot(), failed_order_price() (+34 more)

### Community 87 - "profiles.rs"
Cohesion: 0.23
Nodes (26): app_config_handler(), add_profile_handler(), AddProfileBody, apply_profile_change(), default_body_broker_id(), delete_profile_handler(), DeleteProfileBody, detect_profile_handler() (+18 more)

### Community 88 - "leveraged_trend_hold/tests.rs"
Cohesion: 0.20
Nodes (23): LeveragedTrendHoldEntry, LeveragedTrendHoldStrategy, buys_any_target_ticker_when_itself_trends_up(), buys_intraday_rebound_when_next_window_shows_buy_pressure(), buys_rapid_rebound_without_trend_snapshot_when_enabled(), downward_candles(), entry(), holds_failed_rebound_until_min_hold_observations_elapsed() (+15 more)

### Community 89 - "BrokerSymbol"
Cohesion: 0.14
Nodes (4): BrokerSymbol, maps_toss_candle_to_broker_candle(), selects_price_response_case_insensitively(), validates_market_data_query_limits_before_request()

### Community 91 - "server/market.rs"
Cohesion: 0.17
Nodes (36): account_sync_error(), balance_handler(), broker_holdings_handler(), chart_handler(), ChartQuery, executed_handler(), invalid_order_input(), order_handler() (+28 more)

### Community 92 - "database_contract_tests.rs"
Cohesion: 0.11
Nodes (33): MutexGuard, drop_test_database(), pg_test_env(), PgTestEnv, postgresql_destructive_operations_require_exact_confirmation(), postgresql_full_roundtrip_contract(), postgresql_rejects_invalid_credentials(), Option (+25 more)

### Community 95 - "LeveragedTrendHoldStrategy"
Cohesion: 0.15
Nodes (14): LeveragedTrendHoldEntry, LeveragedTrendHoldMarketState, LeveragedTrendHoldParams, LeveragedTrendHoldPosition, LeveragedTrendHoldPreviewSignal, LeveragedTrendHoldStrategy, Default, HashMap (+6 more)

### Community 96 - "compilerOptions"
Cohesion: 0.08
Nodes (23): DOM, DOM.Iterable, ES2020, src, compilerOptions, allowImportingTsExtensions, isolatedModules, jsx (+15 more)

### Community 97 - "RiskManager"
Cohesion: 0.06
Nodes (36): RiskStore, Option, PathBuf, Result, Self, config_state_roundtrip_restores_all_settings(), consecutive_loss_block_only_clears_after_profit(), consecutive_loss_blocks_are_isolated_by_broker_account_scope() (+28 more)

### Community 98 - "작업 기록: <짧은 제목>"
Cohesion: 0.09
Nodes (22): 작업 기록: <짧은 제목>, 1. 메타데이터, 10. 완료 보고, 변경 요약, 변경 파일(절대경로), 기존 사용자 변경 보존 결과, 알려진 제한/잔여 위험, 운영자 확인 필요 (+14 more)

### Community 101 - "ui/StockChart.tsx"
Cohesion: 0.12
Nodes (22): useChartData(), useTossChartData(), buildChartOptions(), CandlePoint, CHART_PRESETS, ChartPreset, ChartSource, ChartType (+14 more)

### Community 102 - "release-version.mjs"
Cohesion: 0.15
Nodes (17): args, assertGitClean(), assertTagAvailable(), branchName, changedFiles, currentBranchName(), escapeRegExp(), fail() (+9 more)

### Community 104 - "dependencies"
Cohesion: 0.09
Nodes (22): @emotion/react, @emotion/styled, @mui/icons-material, @mui/material, dependencies, @emotion/react, @emotion/styled, @mui/icons-material (+14 more)

### Community 106 - "TradeRecord"
Cohesion: 0.20
Nodes (16): account_less_scope_matches_any_account_of_same_broker(), default_currency(), default_trade_market(), json_roundtrip_preserves_broker_scope_and_legacy_defaults_to_none(), legacy_record_without_scope_counts_as_kis_only(), record(), BrokerId, Option (+8 more)

### Community 108 - "detect.rs"
Cohesion: 0.13
Nodes (17): classify_token_response(), detect_trading_type(), DetectedTradingType, DetectTokenReq, DetectTradingTypeError, is_paper_key_rejected_by_real_domain(), is_real_key_rejected_by_paper_domain(), mentions_app_key() (+9 more)

### Community 116 - "Signal"
Cohesion: 0.13
Nodes (24): RecentSideHistory, ScopedSymbol, blocks_reentry_after_stop_loss_sell_on_same_day(), blocks_repeated_same_side_signal_inside_cooldown(), blocks_tiny_profit_after_costs(), blocks_whipsaw_after_repeated_opposite_signals(), buy_signal(), cooldown_is_isolated_by_broker_account_scope() (+16 more)

### Community 120 - "react"
Cohesion: 0.14
Nodes (11): react, useUpdateAutoTradingBudget(), amount(), BudgetEditor(), AutoTradingBudgetInput, AutoTradingBudgetView, BudgetCurrencyView, CmdError (+3 more)

### Community 121 - "database_io.rs"
Cohesion: 0.23
Nodes (18): Drop, atomic_write(), atomic_write_private(), atomic_write_private_sync(), backup_failure_preserves_primary_and_cleans_staged_content(), durable_atomic_write(), locked_destination_preserves_content_and_allows_retry_after_release(), private_async_replacements_keep_owner_only_permissions() (+10 more)

### Community 122 - "devDependencies"
Cohesion: 0.11
Nodes (17): devDependencies, @playwright/test, @tauri-apps/cli, @types/react, @types/react-dom, @types/react-grid-layout, typescript, vite (+9 more)

### Community 124 - "security.rs"
Cohesion: 0.12
Nodes (22): Body, Next, Request, Router, app(), bearer_token(), constant_time_eq(), cross_origin_mutation_is_rejected_even_with_token() (+14 more)

### Community 125 - "preview_strategy_from_candles"
Cohesion: 0.17
Nodes (24): preview_strategy_from_candles(), all_strategies_keep_the_same_evaluation_window_without_implicit_warmup(), complete_daily_history_initializes_supported_strategies_without_shortening_defaults(), cross_and_breakout_seed_only_final_bars_are_not_conditions_not_met(), first_ready_bar_is_observed_after_tick_without_discarding_evaluation_bars(), generic_lth_is_rejected_before_live_clock_or_candle_preparation(), input(), insufficient_history_is_not_reported_as_a_zero_trade_strategy() (+16 more)

### Community 128 - "DatabaseConfig"
Cohesion: 0.18
Nodes (18): DatabaseConfig, DatabaseConfigView, DatabaseProvider, DatabaseStatusView, DatabaseTableStatus, DatabaseTlsMode, DatabaseTransferResult, JsonStorageCategoryView (+10 more)

### Community 129 - "read_json_or_default"
Cohesion: 0.24
Nodes (17): build_daily_path(), build_monthly_path(), concurrent_writes_leave_one_complete_json_document(), corrupt_json_recovers_from_last_durable_backup(), daily_path_uses_date_components_without_string_separators(), ensure_dir(), json_path_lock(), monthly_path_zero_pads_month_for_stats() (+9 more)

### Community 131 - "KISAutoTrade 검증 Harness 지도"
Cohesion: 0.12
Nodes (16): 1. 환경 기준, 2. 기본 정적·단위 게이트, 3. 변경 유형별 필수 조합, 4. Focused Rust 테스트, 5. Playwright, 6. 구조·생성물 검사, 7. Provider·릴리스 검사, 8. CI와 로컬의 대응 (+8 more)

### Community 132 - ".new"
Cohesion: 0.17
Nodes (14): AsyncMutex, Arc, Into, Self, shared_toss_token_state(), toss_token_state_key(), TossTokenState, body_snippet() (+6 more)

### Community 133 - "active_profile"
Cohesion: 0.22
Nodes (24): LeveragedTrendHoldPreviewInput, active_profile(), leveraged_trend_hold_preview_handler(), Json, Option, Path, Query, Result (+16 more)

### Community 135 - "normalize_rows"
Cohesion: 0.21
Nodes (16): PreviewRow, chart_volume_to_u64(), normalize_rows(), BrokerId, ChartCandle, CmdResult, Option, SimulationAssumptions (+8 more)

### Community 136 - "StrategyConfig"
Cohesion: 0.27
Nodes (10): BrokerPositionSnapshot, BrokerId, BrokerMarket, Into, Option, Self, String, Value (+2 more)

### Community 137 - "OhlcCandle"
Cohesion: 0.08
Nodes (19): apply_intraday_ohlc(), apply_ohlc_history(), broker_candles_to_ohlc(), broker_money_to_strategy_units(), initialize_active_strategy_history(), Arc, Mutex, Option (+11 more)

### Community 138 - ".preview_signals_with_execution"
Cohesion: 0.20
Nodes (10): LeveragedRapidReboundSnapshot, LeveragedReboundSnapshot, lth_default_qty(), LeveragedTrendHoldStrategy, F, LeveragedTrendHoldPreviewSignal, String, Vec (+2 more)

### Community 139 - "LeveragedTrendHoldStrategy"
Cohesion: 0.29
Nodes (6): LeveragedTrendHoldStrategy, Option, Vec, VecDeque, LeveragedTrendSnapshot, Option

### Community 141 - "ui/OverseasStockChart.tsx"
Cohesion: 0.13
Nodes (15): lightweight-charts, lightweight-charts, useOverseasChartData(), buildChartOptions(), CandlePoint, ChartType, CrosshairData, LinePoint (+7 more)

### Community 143 - "bollinger.rs"
Cohesion: 0.25
Nodes (11): Band, candle(), disabled_option_preserves_entries_and_buffer_is_bounded(), downward_breakout_and_insufficient_history_block_entries(), flat_squeeze_needs_completed_upward_breakout(), LeveragedTrendHoldStrategy, live_and_replay_share_confirmed_breakout_and_exit_without_future_prices(), Option (+3 more)

### Community 144 - "KISAutoTrade 아키텍처"
Cohesion: 0.12
Nodes (15): 자동매매 예산 경계, 1. 시스템 경계, 10. 백그라운드 작업, 11. 변경 도구와 검증, 2. Frontend, 3. IPC 계약, 4. AppState와 동시성, 5. Broker와 provider (+7 more)

### Community 145 - "graphify.mjs"
Cohesion: 0.18
Nodes (12): buildSnapshot(), fail(), listRepositoryFiles(), mode, parseMode(), REPOSITORY_ROOT, ROOT_CONFIGS, SCRIPT_DIR (+4 more)

### Community 148 - ".default"
Cohesion: 0.23
Nodes (12): default_atr_stop_multiplier(), default_daily_loss_limit(), default_exchange_rate(), default_fee_bps(), default_in_sample_percent(), default_initial_capital(), default_max_position_ratio(), default_risk_per_trade_bps() (+4 more)

### Community 149 - "server/trading.rs"
Cohesion: 0.17
Nodes (25): decimal_quantity_units(), money_units(), normalized_f64(), normalized_u64(), BrokerCurrency, BrokerId, Json, Option (+17 more)

### Community 153 - "search_stock"
Cohesion: 0.21
Nodes (22): ChartDataInput, get_chart_data(), get_overseas_chart_data(), get_overseas_price(), get_price(), get_stock_list_stats(), OverseasOrderInput, OverseasPriceView (+14 more)

### Community 155 - "StockStore"
Cohesion: 0.13
Nodes (15): Arc, HashMap, I, Option, Path, PathBuf, Result, RwLock (+7 more)

### Community 160 - "KISAutoTrade 탐색 지도"
Cohesion: 0.14
Nodes (13): 주문·리스크 변경, 1. 런타임 한눈에 보기, 2. 작업별 첫 진입점, 3. Frontend 경계, 4. Backend 경계, 5. 중요한 동기화 묶음, 6. 데이터와 민감 영역, 7. 지도 유지 원칙 (+5 more)

### Community 161 - "TradeStore"
Cohesion: 0.38
Nodes (6): Mutex, NaiveDate, PathBuf, Result, Vec, TradeStore

### Community 165 - "AppState"
Cohesion: 0.10
Nodes (40): File, AppState, clear_database_tables(), create_database_tables(), database_error(), drop_database_tables(), ensure_trading_stopped(), export_database_to_json() (+32 more)

### Community 168 - "작업 기록: AppleDouble 점검 및 SOXQ-P1-03"
Cohesion: 0.15
Nodes (12): 1. 메타데이터, 10. 완료 보고, 2. 목표와 비목표, 3. 시작 상태, 4. 조사 근거, 5. 영향 범위와 계약, 6. 금융·자동매매 안전 검토, 7. 구현과 롤백 (+4 more)

### Community 169 - "작업 기록: SOXQ-P1-04 LTH 현재시각 경로 차단"
Cohesion: 0.15
Nodes (12): 1. 메타데이터, 10. 완료 보고, 2. 목표와 비목표, 3. 시작 상태, 4. 조사 근거, 5. 변경과 계약, 6. 금융·자동매매 안전, 7. 구현·롤백 (+4 more)

### Community 175 - "day_event_tests.rs"
Cohesion: 0.38
Nodes (12): candle(), config(), daily_hooks_do_not_create_unknown_symbol_states(), Value, strong_close_completed_day_uses_actual_position_and_keeps_stop_loss(), strong_close_uses_each_completed_day_only_on_the_following_day(), volatility(), volatility_collects_missing_history_after_close_for_later_days() (+4 more)

### Community 177 - "conflicts.rs"
Cohesion: 0.25
Nodes (18): order_side_label(), pending_conflict_blocks_opposite_side_in_same_scope(), pending_conflict_blocks_same_side_in_same_scope(), pending_conflict_reason_for_scope(), pending_conflict_scan_ignores_different_scope(), pending_order(), pending_order_conflict_reason(), pending_order_provider() (+10 more)

### Community 178 - "place_toss_order"
Cohesion: 0.20
Nodes (16): ensure_toss_manual_session_open(), manual_submission_response(), place_order(), place_toss_order(), PlaceOrderInput, BrokerCurrency, CmdResult, Option (+8 more)

### Community 180 - "작업 기록: SOXQ-P1-05 일봉 진단과 장중 평가 분리"
Cohesion: 0.17
Nodes (11): 1. 메타데이터, 10. 전달 상태와 잔여 작업, 2. 목표와 완료 조건, 3. 시작 상태, 4. 조사와 영향 범위, 5. 계약 체크, 6. 금융·자동매매 안전 검토, 7. 구현과 롤백 (+3 more)

### Community 181 - "verify-release-artifacts.mjs"
Cohesion: 0.18
Nodes (9): args, artifactDir, artifacts, checksumPath, manifestPath, names, records, requireSignatures (+1 more)

### Community 185 - "input"
Cohesion: 0.29
Nodes (11): blocked_strong_close_buy_does_not_discard_next_days_completed_condition(), candle(), daily_event_strategies_reject_other_cadences(), flat_completed_days_fill_history_without_compressing_the_window(), input(), ChartCandle, StrategyPreviewInput, Value (+3 more)

### Community 189 - "ui/AppShell.tsx"
Cohesion: 0.18
Nodes (11): useUpdateCheck(), createAppTheme(), getResolvedMode(), AppShell(), BOTTOM_NAV_ITEMS, handleMainScroll(), MainScrollbarDrag, observeMainScrollContent() (+3 more)

### Community 190 - "checked_budget_scope"
Cohesion: 0.12
Nodes (27): AutoTradingBudgetInput, AutoTradingBudgetInput, budget_error(), checked_budget_scope(), get_auto_trading_budget(), Arc, AutoTradingBudgetView, BrokerId (+19 more)

### Community 192 - "OrderRecord"
Cohesion: 0.18
Nodes (14): OrderRecord, OrderSide, OrderStore, parallel_appends_do_not_lose_order_records(), BrokerId, Into, Mutex, NaiveDate (+6 more)

### Community 193 - "StatsStore"
Cohesion: 0.25
Nodes (9): DailyStats, Mutex, NaiveDate, PathBuf, Result, Self, String, Vec (+1 more)

### Community 194 - "프로젝트 상태·캐시 정리와 SOXQ-P1-01"
Cohesion: 0.33
Nodes (5): 프로젝트 상태 점검 완료, 도구와 계약, 검증·잔여 기록, P1-01 완료 조건, 프로젝트 상태·캐시 정리와 SOXQ-P1-01

### Community 198 - "buffer_tests.rs"
Cohesion: 0.36
Nodes (10): daily_context_cannot_prepare_minute_indicators_or_rebound(), daily_reinitialization_keeps_intraday_bollinger_and_actual_position(), failed_buy_feedback_does_not_create_phantom_position(), failed_sell_feedback_keeps_actual_position_for_next_risk_exit(), intraday_and_daily_context_buffers_have_independent_bounds(), params(), repeated_intraday_seed_replaces_buffers_without_daily_mix_or_position_reset(), replay_daily_values_cannot_change_minute_signals_or_indicator_snapshot() (+2 more)

### Community 199 - "LeveragedTrendHoldStrategy"
Cohesion: 0.40
Nodes (3): LeveragedTrendHoldStrategy, Option, String

### Community 202 - "check-fsd-imports.mjs"
Cohesion: 0.22
Nodes (5): layerRank, layers, root, srcRoot, violations

### Community 203 - "verify-lockfiles.mjs"
Cohesion: 0.22
Nodes (6): appEntry, escapedName, npmLock, pkg, rootDir, workspaceLock

### Community 209 - "LeveragedTrendHoldTimedCandle"
Cohesion: 0.40
Nodes (10): LeveragedTrendHoldTimedCandle, future_bar_does_not_change_prior_signal(), history(), params(), Value, Vec, run(), session_and_blackout_use_input_minutes() (+2 more)

### Community 210 - "LeveragedTrendHoldStrategy"
Cohesion: 0.29
Nodes (4): LeveragedTrendHoldStrategy, parse_hhmm(), Option, String

### Community 213 - "verify-toss-openapi.mjs"
Cohesion: 0.25
Nodes (6): endpointInventory, EXPECTED_PATHS, hasRateLimitHeaders, missingPaths, paths, servers

### Community 214 - "compilerOptions"
Cohesion: 0.22
Nodes (8): vite.config.ts, compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 217 - ".subscribe"
Cohesion: 0.19
Nodes (14): KisWebSocketClient, parse_realtime_price(), RealtimePrice, AppHandle, Arc, AtomicBool, Option, Result (+6 more)

### Community 218 - "DatabasePool"
Cohesion: 0.26
Nodes (12): R, DatabaseArchiveStats, purge(), row_values(), NaiveDate, Option, Result, String (+4 more)

### Community 226 - "ReplayAssessmentView"
Cohesion: 0.29
Nodes (6): daily_diagnostic_never_evaluates_strategy_or_execution_even_with_intraday_input(), empty_minute_observations_are_not_evaluable(), ReplayAssessmentView, Self, String, Vec

### Community 227 - "simulation/tests.rs"
Cohesion: 0.33
Nodes (6): daily_loss_and_consecutive_loss_state_roll_over_on_event_date(), deterministic_replay_applies_costs_and_separates_signals_from_fills(), event(), identical_fixture_has_identical_report_and_hash(), Option, sell_guard_uses_the_same_round_trip_cost_assumptions_as_settlement()

### Community 229 - "check"
Cohesion: 0.31
Nodes (8): check(), GitHubRelease, is_newer(), Client, Option, Result, String, UpdateInfo

### Community 230 - "PendingOrderStore"
Cohesion: 0.29
Nodes (8): PendingOrderStore, restart_snapshot_preserves_scope_client_id_and_fill_watermark(), Mutex, PathBuf, PendingOrder, Result, Self, Vec

### Community 238 - "StrategyStore"
Cohesion: 0.26
Nodes (7): Mutex, Path, PathBuf, Result, Self, Vec, StrategyStore

### Community 239 - ".save_snapshot"
Cohesion: 0.29
Nodes (8): BalanceSnapshot, BalanceStore, HoldingItem, PathBuf, Result, Self, String, Vec

### Community 245 - "model/tradingStore.ts"
Cohesion: 0.15
Nodes (7): zustand, AccountState, useAccountStore, TradingState, TradingStatus, useTradingStore, zustand

### Community 247 - "error.rs"
Cohesion: 0.24
Nodes (9): definitively_rejected(), Error, Option, StatusCode, String, Value, TossApiError, TossErrorResponse (+1 more)

### Community 262 - "NotificationService"
Cohesion: 0.50
Nodes (3): NotificationService, Send, Sync

## Knowledge Gaps
- **343 isolated node(s):** `name`, `private`, `version`, `type`, `node` (+338 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `AppState` to `submit_toss_small_buy_verification_for_profile`, `KisRestClient`, `commands/risk.rs`, `DatabaseManager`, `toss_market.rs`, `commands.rs`, `search_stock`, `StrategyManager`, `StockStore`, `AccountProfile`, `accounts.rs`, `TradeStore`, `settings.rs`, `strategy_preview.rs`, `fills.rs`, `commands/records.rs`, `PositionTracker`, `commands/trading.rs`, `place_toss_order`, `checked_budget_scope`, `OrderRecord`, `StatsStore`, `commands/toss.rs`, `run`, `RiskManager`, `StrategyStore`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Why does `ServerState` connect `ServerState` to `RiskManager`, `StatsStore`, `TradeStore`, `active_profile`, `fills.rs`, `server/records.rs`, `KisRestClient`, `PositionTracker`, `StockStore`, `DatabaseManager`, `StrategyStore`, `security.rs`, `server/trading.rs`, `profiles.rs`, `StrategyManager`, `server/market.rs`, `AccountProfile`, `checked_budget_scope`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Why does `StrategyConfig` connect `StrategyConfig` to `VolatilityExpansionStrategy`, `MomentumStrategy`, `bounded_window_with_extra`, `OhlcCandle`, `PositionTracker`, `commands/trading.rs`, `day_event_tests.rs`, `StrategyStore`, `PriceConditionStrategy`, `MovingAverageCrossStrategy`, `ConsecutiveMoveStrategy`, `StrategyManager`, `LeveragedTrendHoldStrategy`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _343 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `shared/api/commands.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.04371184371184371 - nodes in this community are weakly interconnected._
- **Should `strategyPreviewPanel.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.04727491719361638 - nodes in this community are weakly interconnected._
- **Should `rest/types.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.125 - nodes in this community are weakly interconnected._