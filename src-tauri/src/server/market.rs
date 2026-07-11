use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use super::ServerState;
use crate::{
    api::rest::{OrderSide, OrderType},
    broker::{
        BrokerAccountId, BrokerAdapter, BrokerCurrency, BrokerId, BrokerMarket, BrokerScope,
        BrokerSymbol, TossBrokerAdapter,
    },
    trading::order::{OrderManager, SubmissionOutcome},
};

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
) -> Response {
    let side = match parse_order_side(&body.side) {
        Some(side) => side,
        None => return invalid_order_input("INVALID_SIDE", "side는 Buy 또는 Sell이어야 합니다."),
    };
    let order_type = match parse_order_type(&body.order_type) {
        Some(order_type) => order_type,
        None => {
            return invalid_order_input(
                "INVALID_ORDER_TYPE",
                "orderType은 Limit 또는 Market이어야 합니다.",
            )
        }
    };

    let active_profile = s.profiles.read().await.get_active().cloned();
    if let Some(profile) = active_profile
        .as_ref()
        .filter(|profile| profile.broker_id == BrokerId::Toss)
    {
        if !profile.is_configured() || !profile.live_trading_consent {
            return invalid_order_input(
                "CONFIG_NOT_READY",
                "Toss 설정과 실거래 동의를 확인하세요.",
            );
        }
        if let Err(response) = validate_toss_web_order(
            profile,
            &body.symbol,
            side,
            order_type,
            body.quantity,
            body.price.to_string(),
        )
        .await
        {
            return response;
        }
        let account_id = profile.broker_account_id();
        let adapter = TossBrokerAdapter::with_credentials(
            TossBrokerAdapter::DEFAULT_BASE_URL,
            profile.app_key.clone(),
            profile.app_secret.clone(),
            Some(account_id.clone()),
        );
        let quote = match adapter.get_price(&BrokerSymbol(body.symbol.clone())).await {
            Ok(quote) if quote.market == BrokerMarket::Kr => quote,
            Ok(_) => {
                return invalid_order_input(
                    "INVALID_MARKET",
                    "국내 주문 endpoint에는 한국 종목만 허용됩니다.",
                )
            }
            Err(error) => return provider_error(error.to_string()),
        };
        let quote_value = quote
            .last
            .amount
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .unwrap_or(0.0);
        let quote_price = match quote.last.currency {
            BrokerCurrency::Krw => quote_value.round() as u64,
            BrokerCurrency::Usd => (quote_value * 100.0).round() as u64,
        };
        if quote_price == 0 {
            return invalid_order_input("INVALID_QUOTE", "현재가 응답이 올바르지 않습니다.");
        }
        let exchange_rate = *s.exchange_rate_krw.read().await;
        let total_balance =
            match crate::commands::fetch_toss_risk_balance_krw(profile, exchange_rate).await {
                Ok(total) => total,
                Err(error) => return account_sync_error(error),
            };
        let symbol_name = s
            .stock_store
            .get_name(&body.symbol)
            .await
            .unwrap_or_else(|| body.symbol.clone());
        let scope = BrokerScope::new(BrokerId::Toss, Some(BrokerAccountId(account_id)));
        return match OrderManager::submit_manual_order_shared(
            &s.order_manager,
            body.symbol,
            symbol_name,
            side,
            order_type,
            body.quantity,
            body.price,
            quote_price,
            total_balance,
            None,
            scope,
        )
        .await
        {
            Ok(outcome) => submission_response(outcome),
            Err(error) => provider_error(error.to_string()),
        };
    }

    let client = s.rest_client.read().await.clone();
    let quote = match client.get_price(&body.symbol).await {
        Ok(quote) => quote,
        Err(error) => return provider_error(error.to_string()),
    };
    let quote_price = match quote.stck_prpr.trim().replace(',', "").parse::<u64>() {
        Ok(price) if price > 0 => price,
        _ => return invalid_order_input("INVALID_QUOTE", "현재가 응답이 올바르지 않습니다."),
    };
    let total_balance =
        match crate::commands::fetch_account_risk_balance_krw(&client, false, 1.0).await {
            Ok(total) => total,
            Err(error) => return account_sync_error(error),
        };
    let profile = s.profiles.read().await.get_active().cloned();
    let config_account_id = s.config.read().await.broker_account_id.clone();
    let account_id = profile
        .as_ref()
        .map(|value| value.broker_account_id())
        .or_else(|| (!config_account_id.is_empty()).then_some(config_account_id));
    let scope = BrokerScope::new(BrokerId::Kis, account_id.map(BrokerAccountId));
    let symbol_name = if quote.hts_kor_isnm.trim().is_empty() {
        body.symbol.clone()
    } else {
        quote.hts_kor_isnm
    };
    match OrderManager::submit_manual_order_shared(
        &s.order_manager,
        body.symbol,
        symbol_name,
        side,
        order_type,
        body.quantity,
        body.price,
        quote_price,
        total_balance,
        None,
        scope,
    )
    .await
    {
        Ok(outcome) => submission_response(outcome),
        Err(error) => provider_error(error.to_string()),
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
) -> Response {
    let side = match parse_order_side(&body.side) {
        Some(side) => side,
        None => return invalid_order_input("INVALID_SIDE", "side는 Buy 또는 Sell이어야 합니다."),
    };
    let active_profile = s.profiles.read().await.get_active().cloned();
    if let Some(profile) = active_profile
        .as_ref()
        .filter(|profile| profile.broker_id == BrokerId::Toss)
    {
        if !profile.is_configured() || !profile.live_trading_consent {
            return invalid_order_input(
                "CONFIG_NOT_READY",
                "Toss 설정과 실거래 동의를 확인하세요.",
            );
        }
        if let Err(response) = validate_toss_web_order(
            profile,
            &body.symbol,
            side,
            OrderType::Limit,
            body.quantity,
            body.price.to_string(),
        )
        .await
        {
            return response;
        }
        let account_id = profile.broker_account_id();
        let adapter = TossBrokerAdapter::with_credentials(
            TossBrokerAdapter::DEFAULT_BASE_URL,
            profile.app_key.clone(),
            profile.app_secret.clone(),
            Some(account_id.clone()),
        );
        let quote = match adapter.get_price(&BrokerSymbol(body.symbol.clone())).await {
            Ok(quote) if quote.market == BrokerMarket::Us => quote,
            Ok(_) => {
                return invalid_order_input(
                    "INVALID_MARKET",
                    "해외 주문 endpoint에는 미국 종목만 허용됩니다.",
                )
            }
            Err(error) => return provider_error(error.to_string()),
        };
        let quote_price = (quote
            .last
            .amount
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .unwrap_or(0.0)
            * 100.0)
            .round() as u64;
        if quote_price == 0 {
            return invalid_order_input("INVALID_QUOTE", "현재가 응답이 올바르지 않습니다.");
        }
        let exchange_rate = *s.exchange_rate_krw.read().await;
        let total_balance =
            match crate::commands::fetch_toss_risk_balance_krw(profile, exchange_rate).await {
                Ok(total) => total,
                Err(error) => return account_sync_error(error),
            };
        let symbol_name = s
            .stock_store
            .get_name(&body.symbol)
            .await
            .unwrap_or_else(|| body.symbol.clone());
        let scope = BrokerScope::new(BrokerId::Toss, Some(BrokerAccountId(account_id)));
        return match OrderManager::submit_manual_order_shared(
            &s.order_manager,
            body.symbol,
            symbol_name,
            side,
            OrderType::Limit,
            body.quantity,
            (body.price.max(0.0) * 100.0).round() as u64,
            quote_price,
            total_balance,
            Some("TOSS_US".to_string()),
            scope,
        )
        .await
        {
            Ok(outcome) => submission_response(outcome),
            Err(error) => provider_error(error.to_string()),
        };
    }
    let client = s.rest_client.read().await.clone();
    let quote = match client
        .get_overseas_price(&body.symbol, &body.exchange)
        .await
    {
        Ok(quote) => quote,
        Err(error) => return provider_error(error.to_string()),
    };
    let quote_price = (quote
        .last
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .unwrap_or(0.0)
        * 100.0)
        .round() as u64;
    if quote_price == 0 {
        return invalid_order_input("INVALID_QUOTE", "해외 현재가 응답이 올바르지 않습니다.");
    }
    let exchange_rate = *s.exchange_rate_krw.read().await;
    let total_balance =
        match crate::commands::fetch_account_risk_balance_krw(&client, true, exchange_rate).await {
            Ok(total) => total,
            Err(error) => return account_sync_error(error),
        };
    let profile = s.profiles.read().await.get_active().cloned();
    let config_account_id = s.config.read().await.broker_account_id.clone();
    let account_id = profile
        .as_ref()
        .map(|value| value.broker_account_id())
        .or_else(|| (!config_account_id.is_empty()).then_some(config_account_id));
    let scope = BrokerScope::new(BrokerId::Kis, account_id.map(BrokerAccountId));
    let symbol_name = if quote.name.trim().is_empty() {
        body.symbol.clone()
    } else {
        quote.name
    };
    match OrderManager::submit_manual_order_shared(
        &s.order_manager,
        body.symbol,
        symbol_name,
        side,
        OrderType::Limit,
        body.quantity,
        (body.price.max(0.0) * 100.0).round() as u64,
        quote_price,
        total_balance,
        Some(body.exchange),
        scope,
    )
    .await
    {
        Ok(outcome) => submission_response(outcome),
        Err(error) => provider_error(error.to_string()),
    }
}

fn submission_response(outcome: SubmissionOutcome) -> Response {
    match outcome {
        SubmissionOutcome::Submitted { provider_order_id } => Json(serde_json::json!({
            "odno": provider_order_id,
            "ord_tmd": chrono::Local::now().format("%H%M%S").to_string(),
            "tr_id": "SCOPED_ORDER_SERVICE",
            "rt_cd": "0",
            "msg1": "주문 접수"
        }))
        .into_response(),
        SubmissionOutcome::Skipped { reason } => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "code": "ORDER_PREFLIGHT_BLOCKED", "message": reason })),
        )
            .into_response(),
    }
}

fn provider_error(message: String) -> Response {
    (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({ "code": "PROVIDER_ERROR", "message": message })),
    )
        .into_response()
}

fn account_sync_error(message: String) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({ "code": "ACCOUNT_SYNC_FAILED", "message": message })),
    )
        .into_response()
}

async fn validate_toss_web_order(
    profile: &crate::config::AccountProfile,
    symbol: &str,
    side: OrderSide,
    order_type: OrderType,
    quantity: u64,
    price: String,
) -> Result<(), Response> {
    let preflight = crate::commands::check_toss_order_preflight_for_profile(
        crate::commands::TossOrderPreflightInput {
            symbol: symbol.to_string(),
            side: if side == OrderSide::Buy {
                "Buy"
            } else {
                "Sell"
            }
            .to_string(),
            quantity: quantity.to_string(),
            price: (order_type == OrderType::Limit).then_some(price),
        },
        profile.clone(),
    )
    .await
    .map_err(|error| invalid_order_input(&error.code, &error.message))?;
    if !preflight.can_submit {
        return Err(invalid_order_input(
            "TOSS_PREFLIGHT_BLOCKED",
            preflight
                .blocked_reasons
                .first()
                .map(String::as_str)
                .unwrap_or("Toss 주문 사전검증을 통과하지 못했습니다."),
        ));
    }
    let open_orders = crate::commands::list_toss_open_orders_for_profile(
        crate::commands::TossOpenOrdersInput {
            symbol: Some(symbol.to_string()),
        },
        profile.clone(),
    )
    .await
    .map_err(|error| provider_error(error.message))?;
    if let Some(order) = open_orders.first() {
        return Err(invalid_order_input(
            "TOSS_PENDING_ORDER_EXISTS",
            &format!("provider 미체결 주문이 있습니다: {}", order.order_id),
        ));
    }
    Ok(())
}

fn invalid_order_input(code: &str, message: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "code": code, "message": message })),
    )
        .into_response()
}

fn parse_order_side(value: &str) -> Option<OrderSide> {
    match value.trim().to_ascii_lowercase().as_str() {
        "buy" => Some(OrderSide::Buy),
        "sell" => Some(OrderSide::Sell),
        _ => None,
    }
}

fn parse_order_type(value: &str) -> Option<OrderType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "limit" => Some(OrderType::Limit),
        "market" => Some(OrderType::Market),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_order_enums_never_fall_back_to_sell_or_market() {
        assert!(parse_order_side("hold").is_none());
        assert!(parse_order_side("").is_none());
        assert!(parse_order_type("best").is_none());
        assert!(parse_order_type("").is_none());
        assert_eq!(parse_order_side("BUY"), Some(OrderSide::Buy));
        assert_eq!(parse_order_type("LIMIT"), Some(OrderType::Limit));
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
        Ok(candles) => {
            let start = candles.len().saturating_sub(count as usize);
            Json(serde_json::to_value(&candles[start..]).unwrap_or_default())
        }
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

    let client = s.rest_client.read().await.clone();
    match client
        .get_overseas_chart_data(&symbol, &exchange, period, "")
        .await
    {
        Ok(candles) => {
            let start = candles.len().saturating_sub(count as usize);
            Json(serde_json::to_value(&candles[start..]).unwrap_or_default())
        }
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
    if let Err(error) = crate::storage::write_json(&cache_path, &items).await {
        return Json(serde_json::json!({
            "error": format!("종목 목록 캐시 저장 실패: {error}")
        }));
    }
    s.stock_store
        .upsert_many(items.iter().map(|i| (i.pdno.clone(), i.prdt_name.clone())))
        .await;
    tracing::info!("종목 목록 웹 API 갱신 완료: {}개", count);
    Json(serde_json::json!(count))
}
