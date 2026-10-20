use super::*;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
}

fn reject_identity_changes(strategy: &mut dyn Strategy) {
    let original = strategy.config().clone();
    let mut other = original.clone();
    other.id.push_str("_other");
    assert!(!strategy.reconfigure_preserving_position(other));
    let mut other = original.clone();
    other.broker_account_id = Some("other-account".into());
    assert!(!strategy.reconfigure_preserving_position(other));
    let mut other = original.clone();
    other.broker_id = crate::broker::BrokerId::Toss;
    assert!(!strategy.reconfigure_preserving_position(other));
    assert_eq!(strategy.config().id, original.id);
    assert_eq!(strategy.config().broker_id, original.broker_id);
    assert_eq!(
        strategy.config().broker_account_id,
        original.broker_account_id
    );
}

#[test]
fn reconfigure_rsi_recomputes_baseline_with_new_period_and_keeps_targets_separate() {
    let mut strategy = RsiStrategy::new(config(serde_json::json!({
        "period": 3, "oversold": 30.0, "overbought": 70.0,
    })));
    strategy.initialize_historical("SOXQ", &[100, 90, 80, 70]);
    reject_identity_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params["period"] = 2.into();
    next.order_quantity = 7;
    next.target_symbols.push("NEW".into());
    assert!(strategy.reconfigure_preserving_position(next));
    assert_eq!(strategy.params.period, 2);
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 120, 0),
        Signal::Buy { quantity: 7, .. }
    ));
}

#[test]
fn reconfigure_momentum_keeps_owned_quantity_and_retries_unfilled_exits() {
    let mut strategy = MomentumStrategy::new(config(serde_json::json!({
        "lookback_period": 3, "threshold_pct": 50.0,
    })));
    strategy.initialize_historical("SOXQ", &[100, 100, 100, 100]);
    strategy.sync_position("SOXQ", 2, 105);
    reject_identity_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "lookback_period": 2, "threshold_pct": 5.0 });
    next.target_symbols.push("NEW".into());
    next.order_quantity = 7;
    assert!(strategy.reconfigure_preserving_position(next));
    assert_eq!(strategy.states["SOXQ"].owned_quantity, 2);
    assert!(!strategy.states.contains_key("NEW"));
    // A raw sell is not a fill: negative level remains eligible while owned.
    assert!(matches!(
        strategy.on_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
    assert!(matches!(
        strategy.on_tick("SOXQ", 80, 0),
        Signal::Sell { quantity: 7, .. }
    ));
}

#[test]
fn reconfigure_deviation_applies_new_threshold_to_retained_observations() {
    let mut strategy = DeviationStrategy::new(config(serde_json::json!({
        "ma_period": 3, "buy_threshold_pct": -20.0, "sell_threshold_pct": 50.0,
    })));
    strategy.initialize_historical("SOXQ", &[100, 100, 100]);
    strategy.sync_position("SOXQ", 2, 100);
    reject_identity_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "ma_period": 2, "buy_threshold_pct": -20.0, "sell_threshold_pct": 5.0 });
    next.target_symbols.push("NEW".into());
    next.order_quantity = 7;
    assert!(strategy.reconfigure_preserving_position(next));
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 120, 0),
        Signal::Sell { quantity: 7, .. }
    ));
}
