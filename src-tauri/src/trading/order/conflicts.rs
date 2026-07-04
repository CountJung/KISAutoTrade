use crate::{
    broker::{BrokerId, BrokerScope},
    storage::order_store::OrderSide,
};

use super::PendingOrder;

fn same_order_side(left: &OrderSide, right: &OrderSide) -> bool {
    matches!(
        (left, right),
        (OrderSide::Buy, OrderSide::Buy) | (OrderSide::Sell, OrderSide::Sell)
    )
}

pub(super) fn pending_order_provider(pending: &PendingOrder) -> BrokerId {
    match pending.record.provider.as_deref() {
        Some("toss") => BrokerId::Toss,
        _ => BrokerId::Kis,
    }
}

fn order_side_label(side: &OrderSide) -> &'static str {
    match side {
        OrderSide::Buy => "매수",
        OrderSide::Sell => "매도",
    }
}

fn pending_order_conflict_reason(pending: &PendingOrder, requested_side: &OrderSide) -> String {
    let pending_side = order_side_label(&pending.record.side);
    let requested_label = order_side_label(requested_side);
    let odno = pending.record.kis_order_id.as_deref().unwrap_or("unknown");

    if same_order_side(&pending.record.side, requested_side) {
        format!(
            "{} {}주 {} 미체결 주문 이미 존재 (odno: {})",
            pending.record.symbol, pending.record.quantity, pending_side, odno
        )
    } else {
        format!(
            "{} {}주 {} 미체결 주문 존재 — 요청 {} 차단 (odno: {})",
            pending.record.symbol, pending.record.quantity, pending_side, requested_label, odno
        )
    }
}

pub(super) fn pending_conflict_reason_for_scope<'a>(
    mut pending_orders: impl Iterator<Item = &'a PendingOrder>,
    broker_scope: &BrokerScope,
    symbol: &str,
    requested_side: &OrderSide,
) -> Option<String> {
    pending_orders
        .find(|pending| pending.broker_scope == *broker_scope && pending.record.symbol == symbol)
        .map(|pending| pending_order_conflict_reason(pending, requested_side))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        broker::{BrokerAccountId, BrokerId, BrokerScope},
        storage::order_store::{OrderRecord, OrderSide},
    };

    use super::*;
    use crate::trading::order::PendingOrder;

    fn scope(broker_id: BrokerId, account_id: &str) -> BrokerScope {
        BrokerScope::new(broker_id, Some(BrokerAccountId(account_id.to_string())))
    }

    fn pending_order(side: OrderSide, broker_scope: BrokerScope) -> PendingOrder {
        let mut record = OrderRecord::new(
            "005930".to_string(),
            "삼성전자".to_string(),
            side,
            3,
            0,
            "Market".to_string(),
        );
        record.kis_order_id = Some("ODNO-1".to_string());

        PendingOrder {
            record,
            signal_reason: "test".to_string(),
            strategy_id: Some("strategy".to_string()),
            signal_price: 75_000,
            order_price: 0,
            exchange: None,
            broker_scope,
            filled_quantity: 0,
        }
    }

    #[test]
    fn pending_conflict_blocks_opposite_side_in_same_scope() {
        let broker_scope = scope(BrokerId::Kis, "kis-1");
        let pending = [pending_order(OrderSide::Buy, broker_scope.clone())];
        let reason = pending_conflict_reason_for_scope(
            pending.iter(),
            &broker_scope,
            "005930",
            &OrderSide::Sell,
        )
        .unwrap();

        assert!(reason.contains("매수 미체결 주문 존재"));
        assert!(reason.contains("요청 매도 차단"));
        assert!(reason.contains("ODNO-1"));
    }

    #[test]
    fn pending_conflict_blocks_same_side_in_same_scope() {
        let broker_scope = scope(BrokerId::Kis, "kis-1");
        let pending = [pending_order(OrderSide::Sell, broker_scope.clone())];
        let reason = pending_conflict_reason_for_scope(
            pending.iter(),
            &broker_scope,
            "005930",
            &OrderSide::Sell,
        )
        .unwrap();

        assert!(reason.contains("매도 미체결 주문 이미 존재"));
        assert!(reason.contains("ODNO-1"));
    }

    #[test]
    fn pending_conflict_scan_ignores_different_scope() {
        let mut pending = HashMap::new();
        pending.insert(
            "ODNO-1".to_string(),
            pending_order(OrderSide::Buy, scope(BrokerId::Kis, "kis-1")),
        );

        let requested_scope = scope(BrokerId::Toss, "toss-1");
        let conflict = pending_conflict_reason_for_scope(
            pending.values(),
            &requested_scope,
            "005930",
            &OrderSide::Sell,
        );

        assert!(conflict.is_none());
    }

    #[test]
    fn pending_order_provider_uses_provider_trace() {
        let mut kis_pending = pending_order(OrderSide::Buy, scope(BrokerId::Kis, "kis-1"));
        assert_eq!(pending_order_provider(&kis_pending), BrokerId::Kis);

        kis_pending.record.provider = Some("toss".to_string());
        assert_eq!(pending_order_provider(&kis_pending), BrokerId::Toss);
    }
}
