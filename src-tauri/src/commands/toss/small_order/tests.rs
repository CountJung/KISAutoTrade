use super::*;

fn detail() -> TossOrder {
    serde_json::from_value(serde_json::json!({
        "orderId":"synthetic-order", "symbol":"SOXQ", "side":"BUY", "orderType":"MARKET", "timeInForce":"DAY",
        "status":"FILLED", "price":null, "quantity":"1", "orderAmount":null, "currency":"USD",
        "orderedAt":"2026-10-18T23:30:00+09:00", "canceledAt":null,
        "execution":{"filledQuantity":"1.0", "averageFilledPrice":"3.01", "filledAmount":"3.01",
            "commission":null,"tax":null,"filledAt":"2026-10-18T23:30:01+09:00","settlementDate":null}
    })).unwrap()
}

#[test]
fn confirmed_small_fill_requires_original_order_symbol_side_currency_and_whole_share() {
    let source = detail();
    for change in 0..7 {
        let mut other = source.clone();
        match change {
            0 => other.order_id = "another-order".into(),
            1 => other.symbol = "AAPL".into(),
            2 => other.side = "SELL".into(),
            3 => other.currency = "KRW".into(),
            4 => other.execution.filled_quantity = "0.5".into(),
            5 => other.execution.filled_quantity = "2".into(),
            _ => other.execution.filled_amount = Some("0".into()),
        }
        assert!(
            confirmed_small_order_fill(&other, "synthetic-order", "SOXQ", BrokerCurrency::Usd)
                .is_err()
        );
    }
    let fill = confirmed_small_order_fill(&source, "synthetic-order", "SOXQ", BrokerCurrency::Usd)
        .unwrap()
        .unwrap();
    assert_eq!(fill.quantity, 1);
    assert_eq!(fill.notional, 301);
    assert_eq!(fill.average, 301);
    assert_eq!(fill.execution_date.to_string(), "2026-10-18");
}

#[test]
fn missing_amount_or_zero_fill_is_deferred_without_using_limit_or_average_as_money() {
    let mut source = detail();
    source.execution.filled_amount = None;
    source.execution.average_filled_price = Some("999999".into());
    assert!(
        confirmed_small_order_fill(&source, "synthetic-order", "SOXQ", BrokerCurrency::Usd)
            .unwrap()
            .is_none()
    );
    source.execution.filled_quantity = "0".into();
    assert!(
        confirmed_small_order_fill(&source, "synthetic-order", "SOXQ", BrokerCurrency::Usd)
            .unwrap()
            .is_none()
    );
}
