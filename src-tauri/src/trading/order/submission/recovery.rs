//! Pre-provider order provenance; no credentials and no simulated response.
use super::*;

pub(super) fn snapshot(prepared: &PreparedOrderSubmission, deps: &SubmissionDeps) -> PendingOrder {
    let submission = &prepared.submission;
    let (provider, client_id, tr_id) = match &prepared.provider_request {
        ProviderOrderRequest::Toss(request) => ("toss", request.client_order_id.clone(), None),
        request => {
            // Same request mapping as KisRestClient::place[_overseas]_order.
            let overseas = matches!(request, ProviderOrderRequest::Overseas(_));
            let tr = match (overseas, deps.rest_client.is_paper(), submission.side) {
                (false, false, OrderSide::Buy) => "TTTC0802U",
                (false, false, OrderSide::Sell) => "TTTC0801U",
                (false, true, OrderSide::Buy) => "VTTC0802U",
                (false, true, OrderSide::Sell) => "VTTC0801U",
                (true, false, OrderSide::Buy) => "TTTT1002U",
                (true, false, OrderSide::Sell) => "TTTT1006U",
                (true, true, OrderSide::Buy) => "VTTT1002U",
                (true, true, OrderSide::Sell) => "VTTT1006U",
            };
            ("kis", None, Some(tr.to_string()))
        }
    };
    let mut record = OrderRecord::new(
        submission.symbol.clone(),
        submission.symbol_name.clone(),
        submission.side,
        prepared.quantity,
        prepared.order_price,
        prepared.order_type.into(),
    )
    .with_strategy_id(submission.strategy_id.clone())
    .with_provider_trace(provider, None, client_id.clone(), tr_id)
    .with_broker_scope(&submission.broker_scope);
    record.id = submission
        .budget_id
        .clone()
        .expect("reserved automatic order");
    PendingOrder {
        record,
        signal_reason: submission.reason.clone(),
        strategy_id: submission.strategy_id.clone(),
        signal_price: submission.tick_price,
        order_price: prepared.order_price,
        exchange: prepared.order_exchange.clone(),
        broker_scope: submission.broker_scope.clone(),
        ownership_key: submission.ownership_key.clone(),
        ownership_generation: submission.ownership_generation.clone(),
        strategy_position_metadata: submission.strategy_position_metadata.clone(),
        submitted_cost_basis: submission.submitted_cost_basis,
        filled_quantity: 0,
        filled_notional: 0,
        confirmed_filled_quantity: 0,
        confirmed_avg_price: 0,
        confirmed_filled_notional: None,
        application_started: false,
        application_execution_date: None,
        application_exchange_rate: None,
        application_pnl: None,
        client_order_id: client_id,
        provider_status: None,
    }
}
