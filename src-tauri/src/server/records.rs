use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use super::ServerState;
use crate::commands::{collect_trade_archive_stats, pending_order_to_view, TradeArchiveConfig};
use crate::logging::LogConfig;

/// GET /api/positions
pub(super) async fn positions_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let tracker = s.position_tracker.lock().await;
    let mut positions: Vec<serde_json::Value> = tracker
        .all()
        .iter()
        .map(|p| {
            serde_json::json!({
                "symbol":             p.symbol,
                "symbolName":         p.symbol_name,
                "quantity":           p.quantity,
                "avgPrice":           p.avg_price,
                "currentPrice":       p.current_price,
                "unrealizedPnl":      p.unrealized_pnl(),
                "unrealizedPnlRate":  p.unrealized_pnl_rate(),
            })
        })
        .collect();
    positions.sort_by(|a, b| {
        let qa = a.get("quantity").and_then(|v| v.as_u64()).unwrap_or(0);
        let qb = b.get("quantity").and_then(|v| v.as_u64()).unwrap_or(0);
        qb.cmp(&qa)
    });
    Json(serde_json::to_value(positions).unwrap_or_default())
}

/// GET /api/today-stats
pub(super) async fn today_stats_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let today = chrono::Local::now().date_naive();
    match s.stats_store.get_by_date(today).await {
        Ok(stats) => Json(serde_json::to_value(stats).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub(super) struct DateRangeQuery {
    from: Option<String>,
    to: Option<String>,
}

/// GET /api/stats?from=YYYY-MM-DD&to=YYYY-MM-DD
pub(super) async fn stats_by_range_handler(
    State(s): State<ServerState>,
    Query(params): Query<DateRangeQuery>,
) -> Json<serde_json::Value> {
    use chrono::NaiveDate;
    let from_str = params.from.as_deref().unwrap_or("2020-01-01");
    let to_str = params.to.as_deref().unwrap_or("");
    let today = chrono::Local::now().date_naive().to_string();
    let to_str = if to_str.is_empty() {
        today.as_str()
    } else {
        to_str
    };

    let from = match NaiveDate::parse_from_str(from_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(serde_json::json!({ "error": format!("from 날짜 오류: {}", e) })),
    };
    let to = match NaiveDate::parse_from_str(to_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(serde_json::json!({ "error": format!("to 날짜 오류: {}", e) })),
    };
    match s.stats_store.get_by_range(from, to).await {
        Ok(stats) => Json(serde_json::to_value(stats).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/trades?from=YYYY-MM-DD&to=YYYY-MM-DD
pub(super) async fn trades_by_range_handler(
    State(s): State<ServerState>,
    Query(params): Query<DateRangeQuery>,
) -> Json<serde_json::Value> {
    use chrono::NaiveDate;
    let from_str = params.from.as_deref().unwrap_or("2020-01-01");
    let today = chrono::Local::now().date_naive().to_string();
    let to_str = params.to.as_deref().unwrap_or(today.as_str());

    let from = match NaiveDate::parse_from_str(from_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(serde_json::json!({ "error": format!("from 날짜 오류: {}", e) })),
    };
    let to = match NaiveDate::parse_from_str(to_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(serde_json::json!({ "error": format!("to 날짜 오류: {}", e) })),
    };
    match s.trade_store.get_by_range(from, to).await {
        Ok(trades) => Json(serde_json::to_value(trades).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/kis-executed?from=YYYY-MM-DD&to=YYYY-MM-DD
pub(super) async fn kis_executed_handler(
    State(s): State<ServerState>,
    Query(params): Query<DateRangeQuery>,
) -> Json<serde_json::Value> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let from_fmt = params
        .from
        .as_deref()
        .map(|d| d.replace('-', ""))
        .unwrap_or_else(|| today.clone());
    let to_fmt = params
        .to
        .as_deref()
        .map(|d| d.replace('-', ""))
        .unwrap_or(today);
    let client = s.rest_client.read().await.clone();
    match client.get_executed_orders_range(&from_fmt, &to_fmt).await {
        Ok(orders) => Json(serde_json::to_value(orders).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/pending-orders
pub(super) async fn pending_orders_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let mgr = s.order_manager.lock().await;
    let views = mgr
        .pending_orders()
        .iter()
        .map(|p| pending_order_to_view(p))
        .collect::<Vec<_>>();
    Json(serde_json::to_value(views).unwrap_or_default())
}

/// GET /api/log-config
pub(super) async fn log_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let cfg = s.log_config.read().await.clone();
    Json(serde_json::to_value(cfg).unwrap_or_default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SetLogConfigBody {
    retention_days: Option<u32>,
    max_size_mb: Option<u64>,
    api_debug: Option<bool>,
}

/// POST /api/log-config
pub(super) async fn set_log_config_handler(
    State(s): State<ServerState>,
    Json(body): Json<SetLogConfigBody>,
) -> Json<serde_json::Value> {
    let current = s.log_config.read().await.clone();
    let new_cfg = LogConfig {
        retention_days: body
            .retention_days
            .unwrap_or(current.retention_days)
            .clamp(1, 365),
        max_size_mb: body
            .max_size_mb
            .unwrap_or(current.max_size_mb)
            .clamp(10, 10240),
        api_debug: body.api_debug.unwrap_or(current.api_debug),
    };
    *s.log_config.write().await = new_cfg.clone();
    s.rest_client.read().await.set_api_debug(new_cfg.api_debug);
    new_cfg.save_sync(&s.log_dir).ok();
    crate::logging::cleanup(&s.log_dir, &new_cfg);
    Json(serde_json::to_value(new_cfg).unwrap_or_default())
}

/// GET /api/recent-logs?count=100
pub(super) async fn recent_logs_handler(
    State(s): State<ServerState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let count = params
        .get("count")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100)
        .clamp(1, crate::logging::MAX_RECENT_LOG_COUNT);
    let log_dir = s.log_dir.clone();
    match tokio::task::spawn_blocking(move || crate::logging::read_recent_entries(&log_dir, count))
        .await
    {
        Ok(entries) => Json(serde_json::to_value(entries).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/archive-config
pub(super) async fn archive_config_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let cfg = s.trade_archive_config.read().await.clone();
    Json(serde_json::to_value(cfg).unwrap_or_default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SetArchiveConfigBody {
    retention_days: u32,
    max_size_mb: u64,
}

/// POST /api/archive-config
pub(super) async fn set_archive_config_handler(
    State(s): State<ServerState>,
    Json(body): Json<SetArchiveConfigBody>,
) -> Json<serde_json::Value> {
    let new_cfg = TradeArchiveConfig {
        retention_days: body.retention_days.clamp(1, 3650),
        max_size_mb: body.max_size_mb.clamp(50, 102400),
    };
    *s.trade_archive_config.write().await = new_cfg.clone();
    new_cfg.save_sync(&s.data_dir).ok();
    let data_dir = s.data_dir.clone();
    let cfg_clone = new_cfg.clone();
    tokio::task::spawn_blocking(move || {
        crate::commands::purge_old_trade_files(&data_dir, &cfg_clone)
    });
    tracing::info!(
        "체결 기록 보관 설정 변경 (웹 API): 보관 {}일, 최대 {}MB",
        new_cfg.retention_days,
        new_cfg.max_size_mb
    );
    Json(serde_json::to_value(new_cfg).unwrap_or_default())
}

/// GET /api/archive-stats
pub(super) async fn archive_stats_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let data_dir = s.data_dir.clone();
    let result = tokio::task::spawn_blocking(move || collect_trade_archive_stats(&data_dir)).await;
    match result {
        Ok(stats) => Json(serde_json::to_value(stats).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/today-trades
pub(super) async fn today_trades_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let today = chrono::Local::now().date_naive();
    match s.trade_store.get_by_date(today).await {
        Ok(trades) => Json(serde_json::to_value(trades).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// POST /api/save-trade
pub(super) async fn save_trade_handler(
    State(s): State<ServerState>,
    Json(record): Json<crate::storage::trade_store::TradeRecord>,
) -> Json<serde_json::Value> {
    let saved = serde_json::to_value(&record).unwrap_or_default();
    match s.trade_store.append(record).await {
        Ok(_) => Json(saved),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// POST /api/upsert-stats
pub(super) async fn upsert_stats_handler(
    State(s): State<ServerState>,
    Json(stats): Json<crate::storage::stats_store::DailyStats>,
) -> Json<serde_json::Value> {
    match s.stats_store.upsert(stats).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub(super) struct FrontendLogBody {
    level: Option<String>,
    message: String,
    context: Option<String>,
}

/// POST /api/frontend-log
pub(super) async fn frontend_log_handler(
    Json(body): Json<FrontendLogBody>,
) -> Json<serde_json::Value> {
    let ctx = body.context.as_deref().unwrap_or("ui");
    match body.level.as_deref().unwrap_or("INFO") {
        "ERROR" => tracing::error!("[Frontend:{}] {}", ctx, body.message),
        "WARN" => tracing::warn!("[Frontend:{}] {}", ctx, body.message),
        _ => tracing::info!("[Frontend:{}] {}", ctx, body.message),
    }
    Json(serde_json::json!({ "ok": true }))
}
