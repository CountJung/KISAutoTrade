use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use super::ServerState;
use crate::api::rest::{OrderRequest, OrderSide, OrderType, OverseasOrderRequest};

pub(super) async fn balance_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_balance().await {
        Ok(b) => Json(serde_json::to_value(b).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub(super) async fn overseas_balance_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_overseas_balance().await {
        Ok(b) => Json(serde_json::to_value(b).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub(super) async fn broker_holdings_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let profile = {
        let profiles = s.profiles.read().await;
        profiles.get_active().cloned()
    };
    let Some(profile) = profile else {
        return Json(serde_json::json!([]));
    };

    let rest_client = s.rest_client.read().await.clone();
    match crate::commands::list_broker_holdings_for_profile(profile, rest_client).await {
        Ok(holdings) => Json(serde_json::to_value(holdings).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({
            "code": e.code,
            "error": e.message,
        })),
    }
}

pub(super) async fn price_handler(
    State(s): State<ServerState>,
    Path(symbol): Path<String>,
) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_price(&symbol).await {
        Ok(p) => Json(serde_json::to_value(p).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub(super) async fn overseas_price_handler(
    State(s): State<ServerState>,
    Path((exchange, symbol)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_overseas_price(&symbol, &exchange).await {
        Ok(p) => {
            let mut val = serde_json::to_value(p).unwrap_or_default();
            if let serde_json::Value::Object(ref mut m) = val {
                m.insert("exchange".into(), serde_json::Value::String(exchange));
                m.insert("symbol".into(), serde_json::Value::String(symbol));
            }
            Json(val)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub(super) async fn executed_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_today_executed_orders().await {
        Ok(e) => Json(serde_json::to_value(e).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/search/:query — KRX 로컬 캐시에서 이름/코드 검색
pub(super) async fn search_handler(
    State(s): State<ServerState>,
    Path(query): Path<String>,
) -> Json<serde_json::Value> {
    let list = s.stock_list.read().await;
    let results = crate::market::search_local(&list, &query, 30);
    Json(serde_json::to_value(results).unwrap_or_default())
}

#[derive(Deserialize)]
pub(super) struct OrderBody {
    symbol: String,
    side: String,
    order_type: String,
    quantity: u64,
    price: u64,
}

/// POST /api/order — 국내 주식 주문
pub(super) async fn order_handler(
    State(s): State<ServerState>,
    Json(body): Json<OrderBody>,
) -> Json<serde_json::Value> {
    let side = if body.side == "Buy" {
        OrderSide::Buy
    } else {
        OrderSide::Sell
    };
    let order_type = if body.order_type == "Limit" {
        OrderType::Limit
    } else {
        OrderType::Market
    };

    let req = OrderRequest {
        symbol: body.symbol,
        side,
        order_type,
        quantity: body.quantity,
        price: body.price,
    };
    let client = s.rest_client.read().await.clone();
    match client.place_order(&req).await {
        Ok(r) => Json(serde_json::to_value(r).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub(super) struct OverseasOrderBody {
    symbol: String,
    exchange: String,
    side: String,
    quantity: u64,
    price: f64,
}

/// POST /api/overseas-order — 해외 주식 주문
pub(super) async fn overseas_order_handler(
    State(s): State<ServerState>,
    Json(body): Json<OverseasOrderBody>,
) -> Json<serde_json::Value> {
    let side = if body.side == "Buy" {
        OrderSide::Buy
    } else {
        OrderSide::Sell
    };
    let req = OverseasOrderRequest {
        symbol: body.symbol,
        exchange: body.exchange,
        side,
        quantity: body.quantity,
        price: body.price,
    };
    let client = s.rest_client.read().await.clone();
    match client.place_overseas_order(&req).await {
        Ok(r) => Json(serde_json::to_value(r).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub(super) struct ChartQuery {
    period: Option<String>,
    count: Option<i64>,
}

/// GET /api/chart/:symbol?period=D&count=100
pub(super) async fn chart_handler(
    State(s): State<ServerState>,
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> Json<serde_json::Value> {
    let period = params.period.as_deref().unwrap_or("D");
    let count = params.count.unwrap_or(100).clamp(1, 500);

    let factor: i64 = match period {
        "W" => 7,
        "M" => 31,
        _ => 2,
    };
    let today = chrono::Local::now().date_naive();
    let start_day = today - chrono::Duration::days(count * factor + 10);
    let end_date = today.format("%Y%m%d").to_string();
    let start_date = start_day.format("%Y%m%d").to_string();

    let client = s.rest_client.read().await.clone();
    match client
        .get_chart_data(&symbol, period, &start_date, &end_date)
        .await
    {
        Ok(candles) => Json(serde_json::to_value(candles).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/overseas-chart/:ex/:symbol?period=D&count=100
pub(super) async fn overseas_chart_handler(
    State(s): State<ServerState>,
    Path((exchange, symbol)): Path<(String, String)>,
    Query(params): Query<ChartQuery>,
) -> Json<serde_json::Value> {
    let period = params.period.as_deref().unwrap_or("D");
    let count = params.count.unwrap_or(100).clamp(1, 500);

    let factor: i64 = match period {
        "W" => 7,
        "M" => 31,
        _ => 2,
    };
    let base_day = chrono::Local::now().date_naive() - chrono::Duration::days(count * factor);
    let base_date = base_day.format("%Y%m%d").to_string();

    let client = s.rest_client.read().await.clone();
    match client
        .get_overseas_chart_data(&symbol, &exchange, period, &base_date)
        .await
    {
        Ok(candles) => Json(serde_json::to_value(candles).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/stock-list-stats
pub(super) async fn stock_list_stats_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let count = s.stock_store.size().await;
    let last_updated_at = s.stock_store.last_updated_at().await;
    let update_interval_hours = s.stock_store.get_interval_hours().await;
    Json(serde_json::json!({
        "count":               count,
        "lastUpdatedAt":       last_updated_at,
        "filePath":            "",
        "updateIntervalHours": update_interval_hours,
    }))
}

#[derive(Deserialize)]
pub(super) struct StockIntervalBody {
    hours: u32,
}

/// POST /api/stock-update-interval
pub(super) async fn set_stock_update_interval_handler(
    State(s): State<ServerState>,
    Json(body): Json<StockIntervalBody>,
) -> Json<serde_json::Value> {
    match s.stock_store.set_interval_hours(body.hours).await {
        Ok(()) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// POST /api/refresh-stock-list
pub(super) async fn refresh_stock_list_handler(
    State(s): State<ServerState>,
) -> Json<serde_json::Value> {
    let items = match crate::market::StockList::fetch_from_krx().await {
        Ok(items) if !items.is_empty() => items,
        Ok(_) => {
            return Json(
                serde_json::json!({ "error": "KRX에서 종목 목록을 가져오지 못했습니다 (0개)." }),
            )
        }
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };
    let count = items.len();
    *s.stock_list.write().await = items.clone();
    let cache_path = s.data_dir.join("stock_list.json");
    if let Some(dir) = cache_path.parent() {
        let _ = tokio::fs::create_dir_all(dir).await;
    }
    if let Ok(json) = serde_json::to_string_pretty(&items) {
        let _ = tokio::fs::write(&cache_path, json).await;
    }
    s.stock_store
        .upsert_many(items.iter().map(|i| (i.pdno.clone(), i.prdt_name.clone())))
        .await;
    tracing::info!("종목 목록 웹 API 갱신 완료: {}개", count);
    Json(serde_json::json!(count))
}
