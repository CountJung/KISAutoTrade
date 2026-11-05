//! Shared production router assembly, independent of binding and environment loading.
use super::*;

pub(super) fn build_router(state: ServerState) -> Router {
    let web_security = state.web_security.clone();
    Router::new()
        .route("/api/info", get(info_handler))
        .route("/api/app-config", get(app_config_handler))
        .route("/api/profiles", get(profiles_handler))
        .route("/api/balance", get(balance_handler))
        .route("/api/overseas-balance", get(overseas_balance_handler))
        .route("/api/broker-holdings", get(broker_holdings_handler))
        .route(
            "/api/toss-market-snapshot/{symbol}",
            get(toss_market_snapshot_handler),
        )
        .route(
            "/api/toss-stock-safety/{symbol}",
            get(toss_stock_safety_handler),
        )
        .route(
            "/api/toss-market-calendar",
            get(toss_market_calendar_handler),
        )
        .route("/api/toss-chart/{symbol}", get(toss_chart_handler))
        .route("/api/candles/page", get(candle_page_handler))
        .route(
            "/api/strategy/leveraged-trend-hold/preview",
            post(leveraged_trend_hold_preview_handler),
        )
        .route("/api/positions", get(positions_handler))
        .route("/api/price/{symbol}", get(price_handler))
        .route(
            "/api/overseas-price/{ex}/{symbol}",
            get(overseas_price_handler),
        )
        .route("/api/executed", get(executed_handler))
        .route("/api/kis-executed", get(kis_executed_handler))
        .route("/api/pending-orders", get(pending_orders_handler))
        .route("/api/search/{query}", get(search_handler))
        .route("/api/order", post(order_handler))
        .route("/api/overseas-order", post(overseas_order_handler))
        .route("/api/chart/{symbol}", get(chart_handler))
        .route(
            "/api/overseas-chart/{ex}/{symbol}",
            get(overseas_chart_handler),
        )
        .route("/api/today-stats", get(today_stats_handler))
        .route("/api/stats", get(stats_by_range_handler))
        .route("/api/trades", get(trades_by_range_handler))
        .route(
            "/api/log-config",
            get(log_config_handler).post(set_log_config_handler),
        )
        .route("/api/recent-logs", get(recent_logs_handler))
        .route(
            "/api/archive-config",
            get(archive_config_handler).post(set_archive_config_handler),
        )
        .route("/api/archive-stats", get(archive_stats_handler))
        .route(
            "/api/risk-config",
            get(risk_config_handler).post(update_risk_config_handler),
        )
        .route(
            "/api/auto-trading-budget",
            get(budget::budget_handler).post(budget::update_budget_handler),
        )
        .route(
            "/api/risk-config/clear-emergency",
            post(clear_emergency_handler),
        )
        .route("/api/web-config", get(web_config_handler))
        // ── 자동매매 제어 ──
        .route("/api/trading/status", get(trading_status_handler))
        .route("/api/trading/start", post(trading_start_handler))
        .route("/api/trading/stop", post(trading_stop_handler))
        .route("/api/strategies", get(strategies_handler))
        .route("/api/strategies/{id}", post(update_strategy_handler))
        .route("/api/strategy/preview", post(strategy_preview_handler))
        .route(
            "/api/strategy/preview/continuous",
            post(strategy_continuous_preview_handler),
        )
        // ── 프로파일 관리 ──
        .route("/api/profiles/add", post(add_profile_handler))
        .route("/api/profiles/update", post(update_profile_handler))
        .route("/api/profiles/delete", post(delete_profile_handler))
        .route(
            "/api/profiles/{id}/set-active",
            post(set_active_profile_handler),
        )
        .route("/api/profiles/{id}/detect", post(detect_profile_handler))
        .route("/api/toss-accounts", post(toss_accounts_handler))
        .route(
            "/api/profiles/{id}/toss-accounts",
            post(toss_profile_accounts_handler),
        )
        .route(
            "/api/profiles/{id}/toss-diagnostic",
            post(toss_profile_diagnostic_handler),
        )
        .route(
            "/api/toss-order-preflight",
            post(toss_order_preflight_handler),
        )
        .route("/api/toss-open-orders", post(toss_open_orders_handler))
        .route("/api/toss-order-modify", post(toss_modify_order_handler))
        .route(
            "/api/toss-small-buy-verification",
            post(toss_small_buy_verification_handler),
        )
        // ── 설정 진단 / 감지 ──
        .route("/api/check-config", get(check_config_handler))
        .route(
            "/api/detect-trading-type",
            post(detect_trading_type_handler),
        )
        // ── 종목 목록 ──
        .route("/api/stock-list-stats", get(stock_list_stats_handler))
        .route(
            "/api/stock-update-interval",
            post(set_stock_update_interval_handler),
        )
        .route("/api/refresh-stock-list", post(refresh_stock_list_handler))
        // ── Discord 테스트 ──
        .route("/api/test-discord", post(test_discord_handler))
        // ── 당일 체결 기록 ──
        .route("/api/today-trades", get(today_trades_handler))
        // ── 업데이트 확인 ──
        .route("/api/check-update", get(check_update_handler))
        // ── 웹 설정 저장 ──
        .route("/api/web-config/save", post(save_web_config_handler))
        // ── 환율 / 갱신 주기 ──
        .route("/api/exchange-rate", get(exchange_rate_handler))
        .route(
            "/api/exchange-rate/status",
            get(exchange_rate_status_handler),
        )
        .route("/api/refresh-interval", get(refresh_interval_handler))
        // ── 매수 정지 / 비상 정지 ──
        .route(
            "/api/buy-suspension/clear",
            post(clear_buy_suspension_handler),
        )
        .route("/api/activate-emergency", post(activate_emergency_handler))
        // ── 체결 기록 / 통계 저장 ──
        .route("/api/save-trade", post(save_trade_handler))
        .route("/api/upsert-stats", post(upsert_stats_handler))
        // ── 프론트엔드 로그 ──
        .route("/api/frontend-log", post(frontend_log_handler))
        .fallback(get(spa_handler))
        .layer(middleware::from_fn_with_state(
            web_security.clone(),
            require_web_security,
        ))
        .with_state(state)
}
