use super::*;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
}

fn reject_scope_changes(strategy: &mut dyn Strategy) {
    for change in ["id", "account", "broker"] {
        let mut other = strategy.config().clone();
        match change {
            "id" => other.id.push_str("_other"),
            "account" => other.broker_account_id = Some("other".into()),
            _ => other.broker_id = crate::broker::BrokerId::Toss,
        }
        assert!(!strategy.reconfigure_preserving_position(other));
    }
}

#[test]
fn reconfigure_consecutive_preserves_holding_and_sell_observations() {
    let mut strategy = ConsecutiveMoveStrategy::new(config(serde_json::json!({
        "buy_days": 3, "sell_days": 3,
    })));
    strategy.initialize_historical("SOXQ", &[100, 95, 90]);
    strategy.sync_position("SOXQ", 2, 100);
    reject_scope_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "buy_days": 3, "sell_days": 2 });
    next.target_symbols.push("NEW".into());
    next.order_quantity = 7;
    assert!(strategy.reconfigure_preserving_position(next));
    assert!(strategy.states["SOXQ"].in_position);
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 80, 0),
        Signal::Sell { quantity: 7, .. }
    ));
}

#[test]
fn reconfigure_failed_breakout_keeps_exact_reference_when_new_history_is_short() {
    let mut strategy = FailedBreakoutStrategy::new(config(serde_json::json!({
        "lookback_days": 3, "buffer_pct": 0.5,
    })));
    strategy.initialize_historical("SOXQ", &[95, 100, 98]);
    assert!(matches!(
        strategy.on_tick("SOXQ", 110, 0),
        Signal::Buy { .. }
    ));
    strategy.sync_position("SOXQ", 7, 110);
    reject_scope_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "lookback_days": 100, "buffer_pct": 1.0 });
    next.target_symbols.push("NEW".into());
    next.order_quantity = 7;
    assert!(strategy.reconfigure_preserving_position(next));
    assert_eq!(strategy.states["SOXQ"].breakout_prev_high, Some(100));
    assert!(strategy.states["SOXQ"].owned_quantity > 0);
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 99, 0),
        Signal::Sell { quantity: 7, .. }
    ));
}
