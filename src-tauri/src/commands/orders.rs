use super::*;

use crate::{
    broker::toss::{TossOrderCreateRequest, TossOrderListQuery},
    storage::order_store::{OrderRecord, OrderSide as StoredOrderSide},
    trading::order::PendingOrder,
};

// ────────────────────────────────────────────────────────────────────
// 주문
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PlaceOrderInput {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: u64,
    pub price: f64,
    pub toss_session: Option<String>,
}

#[tauri::command]
pub async fn place_order(
    input: PlaceOrderInput,
    state: State<'_, AppState>,
) -> CmdResult<OrderResponse> {
    use crate::api::rest::{OrderSide, OrderType};

    let side = match input.side.as_str() {
        "buy" | "Buy" => OrderSide::Buy,
        "sell" | "Sell" => OrderSide::Sell,
        other => {
            return Err(CmdError {
                code: "INVALID_SIDE".into(),
                message: format!("알 수 없는 주문 방향: {}", other),
            })
        }
    };

    let order_type = match input.order_type.as_str() {
        "limit" | "Limit" => OrderType::Limit,
        "market" | "Market" => OrderType::Market,
        other => {
            return Err(CmdError {
                code: "INVALID_ORDER_TYPE".into(),
                message: format!("알 수 없는 주문 유형: {}", other),
            })
        }
    };

    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    if matches!(profile.as_ref().map(|p| p.broker_id), Some(BrokerId::Toss)) {
        return place_toss_order(input, side, order_type, profile.unwrap(), &state).await;
    }

    let req = OrderRequest {
        symbol: input.symbol,
        side,
        order_type,
        quantity: input.quantity,
        price: input.price.round().max(0.0) as u64,
    };
    let client = state.rest_client.read().await.clone();
    client.place_order(&req).await.map_err(CmdError::from)
}

async fn place_toss_order(
    input: PlaceOrderInput,
    side: crate::api::rest::OrderSide,
    order_type: crate::api::rest::OrderType,
    profile: AccountProfile,
    state: &State<'_, AppState>,
) -> CmdResult<OrderResponse> {
    if !profile.live_trading_consent {
        return Err(CmdError {
            code: "LIVE_TRADING_CONSENT_REQUIRED".into(),
            message: "Toss 실거래 동의를 먼저 저장해야 수동 주문을 제출할 수 있습니다.".into(),
        });
    }
    let account_seq = profile.broker_account_id();
    if account_seq.trim().is_empty() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "토스증권 accountSeq가 설정되지 않았습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(input.symbol)?;
    let side_text = match side {
        crate::api::rest::OrderSide::Buy => "Buy",
        crate::api::rest::OrderSide::Sell => "Sell",
    };
    let stored_side = match side {
        crate::api::rest::OrderSide::Buy => StoredOrderSide::Buy,
        crate::api::rest::OrderSide::Sell => StoredOrderSide::Sell,
    };
    let is_market = matches!(order_type, crate::api::rest::OrderType::Market);
    let preflight = check_toss_order_preflight_for_profile(
        TossOrderPreflightInput {
            symbol: symbol.clone(),
            side: side_text.to_string(),
            quantity: input.quantity.to_string(),
            price: (!is_market).then(|| input.price.to_string()),
        },
        profile.clone(),
    )
    .await?;
    if !preflight.can_submit {
        let reason = preflight
            .blocked_reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "Toss 주문 전 사전검증을 통과하지 못했습니다.".to_string());
        return Err(CmdError {
            code: "TOSS_PREFLIGHT_BLOCKED".into(),
            message: reason,
        });
    }

    let broker_scope = BrokerScope::new(BrokerId::Toss, Some(BrokerAccountId(account_seq.clone())));
    if let Some(reason) = state
        .order_manager
        .lock()
        .await
        .pending_conflict_reason_for_scope(&broker_scope, &symbol, &stored_side)
    {
        return Err(CmdError {
            code: "LOCAL_PENDING_ORDER_EXISTS".into(),
            message: reason,
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(account_seq.clone()),
    );
    ensure_toss_manual_session_open(input.toss_session.as_deref(), &symbol, &adapter).await?;

    let mut open_query = TossOrderListQuery::open();
    open_query.symbol = Some(symbol.clone());
    let open_orders = adapter
        .list_orders(Some(&account_seq), &open_query)
        .await
        .map_err(|e| CmdError {
            code: "TOSS_OPEN_ORDERS_ERROR".into(),
            message: e.to_string(),
        })?;
    if let Some(open_order) = open_orders.orders.first() {
        return Err(CmdError {
            code: "TOSS_PENDING_ORDER_EXISTS".into(),
            message: format!(
                "{} 미체결 Toss 주문이 있어 수동 주문을 차단했습니다. orderId={}",
                symbol, open_order.order_id
            ),
        });
    }

    let currency = toss_currency_from_view(&preflight.price);
    let order_price = if is_market {
        0
    } else {
        storage_money_units(&input.price.to_string(), currency)
    };
    let request = TossOrderCreateRequest {
        client_order_id: None,
        symbol: symbol.clone(),
        side: match side {
            crate::api::rest::OrderSide::Buy => "BUY",
            crate::api::rest::OrderSide::Sell => "SELL",
        }
        .to_string(),
        order_type: if is_market { "MARKET" } else { "LIMIT" }.to_string(),
        time_in_force: Some("DAY".to_string()),
        quantity: Some(input.quantity.to_string()),
        price: (!is_market).then(|| format_toss_price(input.price, currency)),
        order_amount: None,
        confirm_high_value_order: Some(false),
    }
    .with_generated_client_order_id();
    let client_order_id = request.client_order_id.clone();
    let receipt = adapter
        .create_order(Some(&account_seq), &request)
        .await
        .map_err(|e| CmdError {
            code: "TOSS_ORDER_ERROR".into(),
            message: e.to_string(),
        })?;

    let symbol_name = state
        .stock_store
        .get_name(&symbol)
        .await
        .unwrap_or_else(|| symbol.clone());
    let mut record = OrderRecord::new(
        symbol.clone(),
        symbol_name.clone(),
        stored_side,
        input.quantity,
        order_price,
        format!("TOSS_{}", request.order_type),
    )
    .with_provider_trace(
        "toss",
        Some(receipt.order_id.clone()),
        receipt.client_order_id.clone().or(client_order_id),
        None,
    );
    record.kis_order_id = Some(receipt.order_id.clone());

    let pending = PendingOrder {
        record: record.clone(),
        signal_reason: "Toss 수동 주문".to_string(),
        strategy_id: None,
        signal_price: storage_money_units(&preflight.price.amount, currency),
        order_price,
        exchange: (currency == BrokerCurrency::Usd).then(|| "TOSS_US".to_string()),
        broker_scope,
        filled_quantity: 0,
    };
    state
        .order_manager
        .lock()
        .await
        .track_pending_order(receipt.order_id.clone(), pending);
    state
        .order_store
        .append(record)
        .await
        .map_err(|e| CmdError {
            code: "ORDER_RECORD_WRITE_ERROR".into(),
            message: e.to_string(),
        })?;

    Ok(OrderResponse {
        odno: receipt.order_id,
        ord_tmd: chrono::Local::now().format("%H%M%S").to_string(),
        tr_id: "TOSS".to_string(),
        rt_cd: "0".to_string(),
        msg1: "Toss 주문 접수".to_string(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TossManualSession {
    Auto,
    Day,
    Pre,
    Regular,
    After,
}

impl TossManualSession {
    fn parse(value: Option<&str>) -> CmdResult<Self> {
        match value.unwrap_or("auto").trim().to_ascii_lowercase().as_str() {
            "" | "auto" => Ok(Self::Auto),
            "day" | "daymarket" | "day_market" => Ok(Self::Day),
            "pre" | "premarket" | "pre_market" => Ok(Self::Pre),
            "regular" | "regularmarket" | "regular_market" => Ok(Self::Regular),
            "after" | "aftermarket" | "after_market" => Ok(Self::After),
            other => Err(CmdError {
                code: "INVALID_TOSS_SESSION".into(),
                message: format!("알 수 없는 Toss 미국 주문 세션입니다: {}", other),
            }),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Auto => "자동",
            Self::Day => "데이마켓",
            Self::Pre => "프리마켓",
            Self::Regular => "정규장",
            Self::After => "애프터마켓",
        }
    }

    fn select<'a>(self, day: &'a TossUsMarketCalendarResponse) -> Option<&'a TossMarketSession> {
        match self {
            Self::Auto => None,
            Self::Day => day.today.day_market.as_ref(),
            Self::Pre => day.today.pre_market.as_ref(),
            Self::Regular => day.today.regular_market.as_ref(),
            Self::After => day.today.after_market.as_ref(),
        }
    }
}

async fn ensure_toss_manual_session_open(
    session: Option<&str>,
    symbol: &str,
    adapter: &TossBrokerAdapter,
) -> CmdResult<()> {
    let session = TossManualSession::parse(session)?;
    if session == TossManualSession::Auto {
        return Ok(());
    }
    if is_domestic_symbol(symbol) {
        return Err(CmdError {
            code: "TOSS_SESSION_UNSUPPORTED".into(),
            message: "Toss 거래 세션 선택은 미국 주식 수동 주문에서만 사용할 수 있습니다.".into(),
        });
    }

    let calendar = adapter
        .get_us_market_calendar(None)
        .await
        .map_err(|e| CmdError {
            code: "TOSS_MARKET_CALENDAR_ERROR".into(),
            message: e.to_string(),
        })?;
    let Some(window) = session
        .select(&calendar)
        .and_then(|session| MarketSessionWindow::parse(&session.start_time, &session.end_time))
    else {
        return Err(CmdError {
            code: "TOSS_SESSION_CLOSED".into(),
            message: format!(
                "오늘은 Toss 미국 {} 세션이 없어 주문을 제출하지 않았습니다.",
                session.label()
            ),
        });
    };
    let kst = chrono::FixedOffset::east_opt(9 * 3600).expect("KST FixedOffset 생성 실패");
    let now = chrono::Utc::now().with_timezone(&kst);
    if window.contains(now) {
        return Ok(());
    }

    Err(CmdError {
        code: "TOSS_SESSION_CLOSED".into(),
        message: format!(
            "현재 시간은 선택한 Toss 미국 {} 세션이 아닙니다. 세션 시간: {} ~ {}",
            session.label(),
            window.start_at.format("%Y-%m-%d %H:%M"),
            window.end_at.format("%Y-%m-%d %H:%M")
        ),
    })
}

fn storage_money_units(value: &str, currency: BrokerCurrency) -> u64 {
    let parsed = parse_decimal_amount(value).unwrap_or(0.0).max(0.0);
    match currency {
        BrokerCurrency::Krw => parsed.round() as u64,
        BrokerCurrency::Usd => (parsed * 100.0).round() as u64,
    }
}

fn format_toss_price(price: f64, currency: BrokerCurrency) -> String {
    match currency {
        BrokerCurrency::Krw => format!("{:.0}", price.round().max(0.0)),
        BrokerCurrency::Usd => format!("{:.4}", price.max(0.0)),
    }
}
