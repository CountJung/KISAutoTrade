//! 제출 결과의 미체결·실패 기록 생성.

use super::*;

pub(super) fn build_pending_order(
    prepared: &PreparedOrderSubmission,
    response: ProviderOrderResponse,
) -> PendingOrder {
    match response {
        ProviderOrderResponse::Kis(response) => build_kis_pending_order(prepared, response),
        ProviderOrderResponse::Toss {
            order_id,
            client_order_id,
        } => build_toss_pending_order(prepared, order_id, client_order_id),
    }
}

fn build_kis_pending_order(
    prepared: &PreparedOrderSubmission,
    response: OrderResponse,
) -> PendingOrder {
    tracing::info!(
        "{} 주문 접수: {} {}주 — {} (provider=kis tr_id={} odno={})",
        order_side_label(&prepared.submission.side),
        prepared.submission.symbol,
        prepared.quantity,
        prepared.submission.reason,
        response.tr_id,
        response.odno
    );

    let mut record = OrderRecord::new(
        prepared.submission.symbol.clone(),
        prepared.submission.symbol_name.clone(),
        prepared.submission.side,
        prepared.quantity,
        prepared.order_price,
        prepared.order_type.to_string(),
    )
    .with_strategy_id(prepared.submission.strategy_id.clone())
    .with_provider_trace(
        "kis",
        Some(response.odno.clone()),
        None,
        Some(response.tr_id.clone()),
    )
    .with_broker_scope(&prepared.submission.broker_scope);
    let odno = if response.odno.is_empty() {
        format!("LOCAL-{}", uuid::Uuid::new_v4())
    } else {
        response.odno
    };
    record.kis_order_id = Some(odno.clone());
    record.provider_order_id = Some(odno);

    tracing::info!(
        "주문 trace 저장: provider={} tr_id={} order_id={}",
        record.provider.as_deref().unwrap_or("unknown"),
        record.provider_tr_id.as_deref().unwrap_or("-"),
        record.provider_order_id.as_deref().unwrap_or("-")
    );

    PendingOrder {
        record,
        signal_reason: prepared.submission.reason.clone(),
        exchange: prepared.order_exchange.clone(),
        filled_quantity: 0,
        filled_notional: 0,
        confirmed_filled_quantity: 0,
        confirmed_avg_price: 0,
        application_started: false,
        application_pnl: None,
        strategy_id: prepared.submission.strategy_id.clone(),
        signal_price: prepared.submission.tick_price,
        order_price: prepared.order_price,
        broker_scope: prepared.submission.broker_scope.clone(),
        client_order_id: None,
        provider_status: Some("pending".to_string()),
    }
}

fn build_toss_pending_order(
    prepared: &PreparedOrderSubmission,
    order_id: String,
    client_order_id: Option<String>,
) -> PendingOrder {
    tracing::info!(
        "Toss {} 주문 접수: {} {}주 — {} (order_id={})",
        order_side_label(&prepared.submission.side),
        prepared.submission.symbol,
        prepared.quantity,
        prepared.submission.reason,
        order_id
    );

    let client_order_id_for_snapshot = client_order_id.clone();
    let mut record = OrderRecord::new(
        prepared.submission.symbol.clone(),
        prepared.submission.symbol_name.clone(),
        prepared.submission.side,
        prepared.quantity,
        prepared.order_price,
        format!("TOSS_{}", prepared.order_type.to_uppercase()),
    )
    .with_strategy_id(prepared.submission.strategy_id.clone())
    .with_provider_trace("toss", Some(order_id.clone()), client_order_id, None)
    .with_broker_scope(&prepared.submission.broker_scope);
    record.kis_order_id = Some(order_id.clone());
    record.provider_order_id = Some(order_id.clone());

    PendingOrder {
        record,
        signal_reason: prepared.submission.reason.clone(),
        exchange: prepared.order_exchange.clone(),
        filled_quantity: 0,
        filled_notional: 0,
        confirmed_filled_quantity: 0,
        confirmed_avg_price: 0,
        application_started: false,
        application_pnl: None,
        strategy_id: prepared.submission.strategy_id.clone(),
        signal_price: prepared.submission.tick_price,
        order_price: prepared.order_price,
        broker_scope: prepared.submission.broker_scope.clone(),
        client_order_id: client_order_id_for_snapshot,
        provider_status: Some("pending".to_string()),
    }
}

pub(super) async fn append_failed_order(
    order_store: &Arc<OrderStore>,
    submission: &OrderSubmission,
    quantity: u64,
    price: u64,
    order_type: &str,
    error_message: String,
) {
    let mut record = OrderRecord::new(
        submission.symbol.clone(),
        submission.symbol_name.clone(),
        submission.side,
        quantity,
        price,
        order_type.to_string(),
    )
    .with_strategy_id(submission.strategy_id.clone())
    .with_broker_scope(&submission.broker_scope);
    record.status = OrderStatus::Failed;
    record.error_message = Some(error_message);
    if let Err(e) = order_store.append(record).await {
        tracing::error!("주문 기록 저장 실패 (Failed): {}", e);
    }
}

pub(super) fn failed_order_price(prepared: &PreparedOrderSubmission) -> u64 {
    if matches!(prepared.submission.side, OrderSide::Buy) || prepared.submission.exchange.is_some()
    {
        prepared.submission.tick_price
    } else {
        prepared.order_price
    }
}
