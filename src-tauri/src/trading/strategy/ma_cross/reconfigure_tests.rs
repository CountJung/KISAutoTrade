use super::*;

#[test]
fn reconfigure_ma_recomputes_cross_baseline_and_new_quantity() {
    let mut strategy = MovingAverageCrossStrategy::new(StrategyConfig::new(
        "ma_cross",
        "MA",
        true,
        vec!["SOXQ".into()],
        1,
        serde_json::json!({ "short_period": 1, "long_period": 4 }),
    ));
    strategy.initialize_historical("SOXQ", &[100, 100, 100, 100]);
    for change in ["id", "account", "broker"] {
        let mut other = strategy.config().clone();
        match change {
            "id" => other.id.push_str("_other"),
            "account" => other.broker_account_id = Some("other".into()),
            _ => other.broker_id = crate::broker::BrokerId::Toss,
        }
        assert!(!strategy.reconfigure_preserving_position(other));
    }
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "short_period": 2, "long_period": 3 });
    next.order_quantity = 7;
    next.target_symbols.push("NEW".into());
    assert!(strategy.reconfigure_preserving_position(next));
    assert_eq!(strategy.states["SOXQ"].prev_short_ma, Some(100.0));
    assert_eq!(strategy.states["SOXQ"].prev_long_ma, Some(100.0));
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 80, 0),
        Signal::Sell { quantity: 7, .. }
    ));
}
