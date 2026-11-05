//! Toss open orders and pending-order modification commands.
use super::*;

use crate::broker::toss::{TossOrder, TossOrderListQuery, TossOrderModifyRequest};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOpenOrdersInput {
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOpenOrderView {
    pub broker_id: BrokerId,
    pub account_seq: String,
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub status: String,
    pub price: Option<String>,
    pub quantity: String,
    pub currency: String,
    pub ordered_at: String,
    pub canceled_at: Option<String>,
    pub filled_quantity: String,
    pub average_filled_price: Option<String>,
    pub filled_amount: Option<String>,
    pub commission: Option<String>,
    pub tax: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossModifyOrderInput {
    pub order_id: String,
    pub order_type: String,
    pub quantity: Option<String>,
    pub price: Option<String>,
    pub confirm_high_value_order: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderOperationView {
    pub broker_id: BrokerId,
    pub account_seq: String,
    pub order_id: String,
    pub message: String,
}

fn toss_open_order_view(account_seq: &str, order: TossOrder) -> TossOpenOrderView {
    TossOpenOrderView {
        broker_id: BrokerId::Toss,
        account_seq: account_seq.to_string(),
        order_id: order.order_id,
        symbol: order.symbol,
        side: order.side,
        order_type: order.order_type,
        status: order.status,
        price: order.price,
        quantity: order.quantity,
        currency: order.currency,
        ordered_at: order.ordered_at,
        canceled_at: order.canceled_at,
        filled_quantity: order.execution.filled_quantity,
        average_filled_price: order.execution.average_filled_price,
        filled_amount: order.execution.filled_amount,
        commission: order.execution.commission,
        tax: order.execution.tax,
    }
}

fn require_toss_order_profile(profile: &AccountProfile) -> CmdResult<String> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 주문 관리는 Toss 활성 프로파일에서만 사용할 수 있습니다.".into(),
        });
    }
    let account_seq = profile.broker_account_id();
    if account_seq.trim().is_empty() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "토스증권 accountSeq가 설정되지 않았습니다.".into(),
        });
    }
    Ok(account_seq)
}

fn toss_decimal_to_storage_units(value: &str, currency: &str) -> u64 {
    let parsed = parse_decimal_amount(value).unwrap_or(0.0).max(0.0);
    if currency.eq_ignore_ascii_case("USD") {
        (parsed * 100.0).round() as u64
    } else {
        parsed.round() as u64
    }
}

fn toss_decimal_equals(left: &str, right: &str) -> bool {
    match (parse_decimal_amount(left), parse_decimal_amount(right)) {
        (Some(left), Some(right)) => (left - right).abs() < 0.000_000_001,
        _ => left.trim() == right.trim(),
    }
}

pub async fn list_toss_open_orders_for_profile(
    input: TossOpenOrdersInput,
    profile: AccountProfile,
) -> CmdResult<Vec<TossOpenOrderView>> {
    let account_seq = require_toss_order_profile(&profile)?;
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(account_seq.clone()),
    );

    let mut query = TossOrderListQuery::open();
    query.limit = Some(100);
    query.symbol = input
        .symbol
        .and_then(|symbol| {
            let trimmed = symbol.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .map(normalize_toss_symbol)
        .transpose()?;

    let orders = adapter
        .list_orders(Some(&account_seq), &query)
        .await
        .map_err(|e| CmdError {
            code: "TOSS_OPEN_ORDERS_ERROR".into(),
            message: e.to_string(),
        })?;

    Ok(orders
        .orders
        .into_iter()
        .map(|order| toss_open_order_view(&account_seq, order))
        .collect())
}

#[tauri::command]
pub async fn list_toss_open_orders(
    input: TossOpenOrdersInput,
    state: State<'_, AppState>,
) -> CmdResult<Vec<TossOpenOrderView>> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    list_toss_open_orders_for_profile(input, profile).await
}

pub async fn modify_toss_order_for_profile(
    input: TossModifyOrderInput,
    profile: AccountProfile,
    order_manager: Option<&Arc<Mutex<OrderManager>>>,
) -> CmdResult<TossOrderOperationView> {
    let account_seq = require_toss_order_profile(&profile)?;
    if !profile.live_trading_consent {
        return Err(CmdError {
            code: "LIVE_TRADING_CONSENT_REQUIRED".into(),
            message: "Toss 실거래 동의를 먼저 저장해야 접수 주문을 정정할 수 있습니다.".into(),
        });
    }

    let order_id = input.order_id.trim().to_string();
    if order_id.is_empty() {
        return Err(CmdError {
            code: "INVALID_ORDER_ID".into(),
            message: "정정할 Toss 주문 ID가 비어 있습니다.".into(),
        });
    }

    let modification_reserved = if let Some(manager) = order_manager {
        manager
            .lock()
            .await
            .begin_pending_modification(&order_id)
            .map_err(|error| CmdError {
                code: "ORDER_OPERATION_IN_PROGRESS".into(),
                message: error.to_string(),
            })?
    } else {
        false
    };

    let result = async {
        let order_type = input.order_type.trim().to_ascii_uppercase();
        let quantity = input
            .quantity
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let price = input
            .price
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);

        let adapter = TossBrokerAdapter::with_credentials(
            TossBrokerAdapter::DEFAULT_BASE_URL,
            profile.app_key.clone(),
            profile.app_secret.clone(),
            Some(account_seq.clone()),
        );
        let original_order = adapter
            .get_order(Some(&account_seq), &order_id)
            .await
            .map_err(|e| CmdError {
                code: "TOSS_ORDER_DETAIL_ERROR".into(),
                message: e.to_string(),
            })?;
        let request_quantity = if original_order.currency.eq_ignore_ascii_case("USD") {
            if quantity
                .as_deref()
                .is_some_and(|value| !toss_decimal_equals(value, &original_order.quantity))
            {
                return Err(CmdError {
                code: "TOSS_US_MODIFY_PRICE_ONLY".into(),
                message:
                    "미국 주식 주문 정정은 가격만 지원합니다. 수량 변경 없이 가격만 정정해 주세요."
                        .into(),
            });
            }
            None
        } else {
            Some(quantity.unwrap_or_else(|| original_order.quantity.clone()))
        };

        let request = TossOrderModifyRequest {
            order_type: order_type.clone(),
            quantity: request_quantity,
            price: price.clone(),
            confirm_high_value_order: input.confirm_high_value_order,
        };

        let response = adapter
            .modify_order(Some(&account_seq), &order_id, &request)
            .await
            .map_err(|e| CmdError {
                code: "TOSS_MODIFY_ORDER_ERROR".into(),
                message: e.to_string(),
            })?;

        if modification_reserved {
            let order_manager = order_manager.expect("reserved modification has manager");
            match adapter
                .get_order(Some(&account_seq), &response.order_id)
                .await
            {
                Ok(detail) => {
                    let quantity_units = parse_decimal_amount(&detail.quantity)
                        .map(|value| value.max(0.0).floor() as u64);
                    let price_units = detail
                        .price
                        .as_deref()
                        .map(|price| toss_decimal_to_storage_units(price, &detail.currency));
                    let mut manager = order_manager.lock().await;
                    if !manager.update_pending_order_snapshot(
                        &order_id,
                        Some(&response.order_id),
                        quantity_units,
                        price_units,
                        Some(format!("TOSS_{}", detail.order_type)),
                    ) {
                        return Err(CmdError {
                            code: "ORDER_STATE_CHANGED".into(),
                            message:
                                "정정 중 주문 상태가 변경되었습니다. 접수 주문을 다시 조회하세요."
                                    .into(),
                        });
                    }
                    if let Err(error) = manager.persist_pending_orders().await {
                        manager.block_for_persistence_failure(format!(
                            "정정 주문의 미체결 스냅샷 저장 실패: {error}"
                        ));
                        return Err(CmdError {
                            code: "PENDING_ORDER_WRITE_ERROR".into(),
                            message: format!("정정 주문의 미체결 스냅샷 저장 실패: {error}"),
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Toss 정정 후 주문 상세 조회 실패: orderId={} newOrderId={} {}",
                        order_id,
                        response.order_id,
                        e
                    );
                    let quantity_units = parse_decimal_amount(&original_order.quantity)
                        .map(|value| value.max(0.0).floor() as u64);
                    let price_units =
                        price
                            .as_deref()
                            .or(original_order.price.as_deref())
                            .map(|price| {
                                toss_decimal_to_storage_units(price, &original_order.currency)
                            });
                    let mut manager = order_manager.lock().await;
                    if !manager.update_pending_order_snapshot(
                        &order_id,
                        Some(&response.order_id),
                        quantity_units,
                        price_units,
                        Some(format!("TOSS_{}", request.order_type)),
                    ) {
                        return Err(CmdError {
                            code: "ORDER_STATE_CHANGED".into(),
                            message:
                                "정정 중 주문 상태가 변경되었습니다. 접수 주문을 다시 조회하세요."
                                    .into(),
                        });
                    }
                    if let Err(error) = manager.persist_pending_orders().await {
                        manager.block_for_persistence_failure(format!(
                            "정정 주문의 미체결 스냅샷 저장 실패: {error}"
                        ));
                        return Err(CmdError {
                            code: "PENDING_ORDER_WRITE_ERROR".into(),
                            message: format!("정정 주문의 미체결 스냅샷 저장 실패: {error}"),
                        });
                    }
                }
            }
        }

        Ok(TossOrderOperationView {
            broker_id: BrokerId::Toss,
            account_seq,
            order_id: response.order_id,
            message: "Toss 접수 주문 정정 요청이 완료되었습니다.".to_string(),
        })
    }
    .await;

    if modification_reserved {
        if let Some(manager) = order_manager {
            manager.lock().await.finish_pending_modification(&order_id);
        }
    }
    result
}

#[tauri::command]
pub async fn modify_toss_order(
    input: TossModifyOrderInput,
    state: State<'_, AppState>,
) -> CmdResult<TossOrderOperationView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    modify_toss_order_for_profile(input, profile, Some(&state.order_manager)).await
}
