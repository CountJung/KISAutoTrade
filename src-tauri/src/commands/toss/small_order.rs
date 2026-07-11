use super::*;

use crate::{
    broker::toss::{TossOrder, TossOrderCreateRequest, TossOrderListQuery},
    storage::{
        order_store::{OrderRecord, OrderSide as StoredOrderSide, OrderStatus},
        trade_store::{TradeSide, TradeStatus},
    },
};

const TOSS_SMALL_BUY_QUANTITY: &str = "1";
const TOSS_SMALL_BUY_MAX_KRW: f64 = 1_000_000.0;
const TOSS_SMALL_BUY_MAX_USD: f64 = 1_000.0;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossSmallBuyVerificationInput {
    pub symbol: String,
    pub symbol_name: Option<String>,
    pub expected_account_seq: String,
    pub max_notional_amount: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossSmallBuyVerificationView {
    pub broker_id: BrokerId,
    pub account_seq: String,
    pub symbol: String,
    pub symbol_name: String,
    pub market: BrokerMarket,
    pub side: BrokerOrderSide,
    pub order_type: String,
    pub quantity: String,
    pub estimated_gross_amount: BrokerMoneyView,
    pub required_cash: Option<BrokerMoneyView>,
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub status: String,
    pub filled_quantity: String,
    pub average_filled_price: Option<BrokerMoneyView>,
    pub filled_amount: Option<BrokerMoneyView>,
    pub order_record_id: String,
    pub trade_recorded: bool,
    pub message: String,
}

fn small_buy_cap(currency: BrokerCurrency) -> f64 {
    match currency {
        BrokerCurrency::Krw => TOSS_SMALL_BUY_MAX_KRW,
        BrokerCurrency::Usd => TOSS_SMALL_BUY_MAX_USD,
    }
}

fn broker_money_view_amount(value: &BrokerMoneyView) -> f64 {
    parse_decimal_amount(&value.amount).unwrap_or(0.0)
}

fn order_status_from_toss(status: &str) -> OrderStatus {
    match status {
        "FILLED" => OrderStatus::Filled,
        "PARTIAL_FILLED" => OrderStatus::PartiallyFilled,
        "CANCELED" => OrderStatus::Cancelled,
        "REJECTED" | "CANCEL_REJECTED" | "REPLACE_REJECTED" => OrderStatus::Failed,
        _ => OrderStatus::Pending,
    }
}

fn is_toss_order_final(status: &str) -> bool {
    matches!(
        status,
        "FILLED"
            | "PARTIAL_FILLED"
            | "CANCELED"
            | "REJECTED"
            | "CANCEL_REJECTED"
            | "REPLACE_REJECTED"
    )
}

fn storage_money_amount(value: &str, currency: BrokerCurrency) -> u64 {
    let parsed = parse_decimal_amount(value).unwrap_or(0.0).max(0.0);
    match currency {
        BrokerCurrency::Krw => parsed.round() as u64,
        BrokerCurrency::Usd => (parsed * 100.0).round() as u64,
    }
}

fn storage_quantity(value: &str) -> u64 {
    parse_decimal_amount(value).unwrap_or(0.0).max(0.0).round() as u64
}

fn money_view_from_decimal(
    value: Option<&str>,
    currency: BrokerCurrency,
) -> Option<BrokerMoneyView> {
    value.map(|amount| {
        BrokerMoney {
            amount: format_money_amount(parse_decimal_amount(amount).unwrap_or(0.0), currency),
            currency,
        }
        .into()
    })
}

async fn poll_toss_order_detail(
    adapter: &TossBrokerAdapter,
    account_seq: &str,
    order_id: &str,
) -> Option<TossOrder> {
    let mut last = None;
    for attempt in 0..3 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        }
        match adapter.get_order(Some(account_seq), order_id).await {
            Ok(order) => {
                let done = is_toss_order_final(&order.status);
                last = Some(order);
                if done {
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Toss 소액매매 주문 상세 조회 실패: {}", e);
                break;
            }
        }
    }
    last
}

struct TossSmallBuyRecordContext<'a> {
    exchange_rate_krw: f64,
    symbol: &'a str,
    symbol_name: &'a str,
    preflight: &'a TossOrderPreflightView,
    order_id: &'a str,
    client_order_id: Option<String>,
    order_detail: Option<&'a TossOrder>,
}

async fn append_toss_small_buy_records(
    order_store: &OrderStore,
    trade_store: &TradeStore,
    context: TossSmallBuyRecordContext<'_>,
) -> CmdResult<(String, bool)> {
    let TossSmallBuyRecordContext {
        exchange_rate_krw,
        symbol,
        symbol_name,
        preflight,
        order_id,
        client_order_id,
        order_detail,
    } = context;
    let currency = toss_currency_from_view(&preflight.price);
    let status = order_detail
        .map(|order| order.status.as_str())
        .unwrap_or("PENDING");
    let avg_price_text = order_detail
        .and_then(|order| order.execution.average_filled_price.as_deref())
        .unwrap_or(&preflight.price.amount);
    let order_price = storage_money_amount(avg_price_text, currency);
    let mut record = OrderRecord::new(
        symbol.to_string(),
        symbol_name.to_string(),
        StoredOrderSide::Buy,
        1,
        order_price,
        "TOSS_MARKET_SMALL_BUY".to_string(),
    )
    .with_provider_trace(
        "toss",
        Some(order_id.to_string()),
        client_order_id.clone(),
        None,
    );
    record.kis_order_id = Some(order_id.to_string());
    record.status = order_status_from_toss(status);
    let order_record_id = record.id.clone();
    order_store.append(record).await.map_err(|e| CmdError {
        code: "TOSS_ORDER_RECORD_WRITE_ERROR".into(),
        message: e.to_string(),
    })?;

    let Some(detail) = order_detail else {
        return Ok((order_record_id, false));
    };
    if !matches!(detail.status.as_str(), "FILLED" | "PARTIAL_FILLED") {
        return Ok((order_record_id, false));
    }

    let filled_quantity = storage_quantity(&detail.execution.filled_quantity);
    if filled_quantity == 0 {
        return Ok((order_record_id, false));
    }
    let avg_price = detail
        .execution
        .average_filled_price
        .as_deref()
        .unwrap_or(&preflight.price.amount);
    let fee = detail.execution.commission.as_deref().unwrap_or("0");
    let mut trade_record = if currency == BrokerCurrency::Usd {
        TradeRecord::new_overseas(
            symbol.to_string(),
            symbol_name.to_string(),
            TradeSide::Buy,
            filled_quantity,
            storage_money_amount(avg_price, currency),
            storage_money_amount(fee, currency),
            order_id.to_string(),
            None,
            "Toss 소액매매 검증 1주 시장가 매수".to_string(),
            "TOSS".to_string(),
            exchange_rate_krw,
            None,
        )
    } else {
        TradeRecord::new(
            symbol.to_string(),
            symbol_name.to_string(),
            TradeSide::Buy,
            filled_quantity,
            storage_money_amount(avg_price, currency),
            storage_money_amount(fee, currency),
            order_id.to_string(),
            None,
            "Toss 소액매매 검증 1주 시장가 매수".to_string(),
        )
    }
    .with_execution_prices(storage_money_amount(&preflight.price.amount, currency), 0)
    .with_provider_trace(
        Some("toss".to_string()),
        Some(order_id.to_string()),
        client_order_id,
        None,
    );
    if detail.status == "PARTIAL_FILLED" {
        trade_record.status = TradeStatus::PartiallyFilled;
    }
    trade_store
        .append(trade_record)
        .await
        .map_err(|e| CmdError {
            code: "TOSS_TRADE_RECORD_WRITE_ERROR".into(),
            message: e.to_string(),
        })?;

    Ok((order_record_id, true))
}

pub async fn submit_toss_small_buy_verification_for_profile(
    input: TossSmallBuyVerificationInput,
    profile: AccountProfile,
    order_store: &OrderStore,
    trade_store: &TradeStore,
    exchange_rate_krw: f64,
) -> CmdResult<TossSmallBuyVerificationView> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 소액매매 검증은 Toss 활성 프로파일에서만 사용할 수 있습니다.".into(),
        });
    }
    if !profile.live_trading_consent {
        return Err(CmdError {
            code: "LIVE_TRADING_CONSENT_REQUIRED".into(),
            message: "Toss 실거래 동의를 먼저 저장해야 실제 1주 매수를 실행할 수 있습니다.".into(),
        });
    }
    if !input.confirmed {
        return Err(CmdError {
            code: "LIVE_ORDER_CONFIRMATION_REQUIRED".into(),
            message: "실제 계좌에서 1주 시장가 매수가 실행될 수 있음을 확인해야 합니다.".into(),
        });
    }

    let account_seq = profile.broker_account_id();
    if account_seq.trim().is_empty() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "토스증권 accountSeq가 설정되지 않았습니다.".into(),
        });
    }
    if input.expected_account_seq.trim() != account_seq {
        return Err(CmdError {
            code: "ACCOUNT_SCOPE_MISMATCH".into(),
            message: "화면의 accountSeq와 현재 활성 Toss accountSeq가 다릅니다. 새로고침 후 다시 확인하세요.".into(),
        });
    }

    let symbol = normalize_toss_symbol(input.symbol)?;
    let preflight = check_toss_order_preflight_for_profile(
        TossOrderPreflightInput {
            symbol: symbol.clone(),
            side: "Buy".to_string(),
            quantity: TOSS_SMALL_BUY_QUANTITY.to_string(),
            price: None,
        },
        profile.clone(),
    )
    .await?;

    if preflight.symbol != symbol {
        return Err(CmdError {
            code: "SYMBOL_SCOPE_MISMATCH".into(),
            message: "사전검증 종목과 제출 종목이 다릅니다. 다시 선택하세요.".into(),
        });
    }
    if !preflight.liquidity_ok || !preflight.safety_ok {
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

    let currency = toss_currency_from_view(&preflight.price);
    let cap = small_buy_cap(currency);
    let gross = broker_money_view_amount(&preflight.gross_amount);
    let required_cash = preflight
        .required_cash
        .as_ref()
        .map(broker_money_view_amount)
        .unwrap_or(gross);
    let max_notional = parse_decimal_amount(&input.max_notional_amount).unwrap_or(0.0);
    if max_notional <= 0.0 {
        return Err(CmdError {
            code: "INVALID_MAX_NOTIONAL".into(),
            message: "최대 허용 주문금액을 0보다 크게 입력하세요.".into(),
        });
    }
    if required_cash > max_notional {
        return Err(CmdError {
            code: "MAX_NOTIONAL_EXCEEDED".into(),
            message: format!(
                "현재 사전검증 필요금액이 최대 허용금액을 초과했습니다: 필요 {} / 허용 {}",
                format_money_amount(required_cash, currency),
                format_money_amount(max_notional, currency)
            ),
        });
    }
    if required_cash > cap {
        return Err(CmdError {
            code: "SMALL_ORDER_CAP_EXCEEDED".into(),
            message: format!(
                "소액매매 검증 한도를 초과했습니다: 필요 {} / 한도 {}",
                format_money_amount(required_cash, currency),
                format_money_amount(cap, currency)
            ),
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(account_seq.clone()),
    );
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
                "{} 미체결 Toss 주문이 있어 소액매매 검증 매수를 차단했습니다. orderId={}",
                symbol, open_order.order_id
            ),
        });
    }

    let order_input = TossOrderCreateRequest {
        client_order_id: None,
        symbol: symbol.clone(),
        side: "BUY".to_string(),
        order_type: "MARKET".to_string(),
        time_in_force: Some("DAY".to_string()),
        quantity: Some(TOSS_SMALL_BUY_QUANTITY.to_string()),
        price: None,
        order_amount: None,
        confirm_high_value_order: Some(false),
    }
    .with_generated_client_order_id();
    let client_order_id = order_input.client_order_id.clone();

    let receipt = adapter
        .create_order(Some(&account_seq), &order_input)
        .await
        .map_err(|e| CmdError {
            code: "TOSS_SMALL_BUY_ORDER_ERROR".into(),
            message: e.to_string(),
        })?;
    let order_detail = poll_toss_order_detail(&adapter, &account_seq, &receipt.order_id).await;
    let symbol_name = input
        .symbol_name
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| symbol.clone());
    let (order_record_id, trade_recorded) = append_toss_small_buy_records(
        order_store,
        trade_store,
        TossSmallBuyRecordContext {
            exchange_rate_krw,
            symbol: &symbol,
            symbol_name: &symbol_name,
            preflight: &preflight,
            order_id: &receipt.order_id,
            client_order_id: client_order_id.clone().or(receipt.client_order_id.clone()),
            order_detail: order_detail.as_ref(),
        },
    )
    .await?;

    let status = order_detail
        .as_ref()
        .map(|order| order.status.clone())
        .unwrap_or_else(|| "PENDING".to_string());
    let filled_quantity = order_detail
        .as_ref()
        .map(|order| order.execution.filled_quantity.clone())
        .unwrap_or_else(|| "0".to_string());
    let average_filled_price = order_detail.as_ref().and_then(|order| {
        money_view_from_decimal(order.execution.average_filled_price.as_deref(), currency)
    });
    let filled_amount = order_detail.as_ref().and_then(|order| {
        money_view_from_decimal(order.execution.filled_amount.as_deref(), currency)
    });
    let message = match status.as_str() {
        "FILLED" => "Toss 1주 시장가 매수 주문이 체결되었습니다.".to_string(),
        "PARTIAL_FILLED" => "Toss 1주 시장가 매수 주문이 부분 체결되었습니다.".to_string(),
        "REJECTED" => {
            "Toss 1주 시장가 매수 주문이 거부되었습니다. 주문 상세와 로그를 확인하세요.".to_string()
        }
        "PENDING" => "Toss 1주 시장가 매수 주문이 접수되었고 체결 대기 중입니다.".to_string(),
        other => format!("Toss 1주 시장가 매수 주문 상태: {other}"),
    };

    Ok(TossSmallBuyVerificationView {
        broker_id: BrokerId::Toss,
        account_seq,
        symbol,
        symbol_name,
        market: preflight.market,
        side: BrokerOrderSide::Buy,
        order_type: "MARKET".to_string(),
        quantity: TOSS_SMALL_BUY_QUANTITY.to_string(),
        estimated_gross_amount: preflight.gross_amount,
        required_cash: preflight.required_cash,
        order_id: receipt.order_id,
        client_order_id: receipt.client_order_id.or(client_order_id),
        status,
        filled_quantity,
        average_filled_price,
        filled_amount,
        order_record_id,
        trade_recorded,
        message,
    })
}

#[tauri::command]
pub async fn submit_toss_small_buy_verification(
    input: TossSmallBuyVerificationInput,
    state: State<'_, AppState>,
) -> CmdResult<TossSmallBuyVerificationView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;
    let exchange_rate_krw = *state.exchange_rate_krw.read().await;

    submit_toss_small_buy_verification_for_profile(
        input,
        profile,
        &state.order_store,
        &state.trade_store,
        exchange_rate_krw,
    )
    .await
}
