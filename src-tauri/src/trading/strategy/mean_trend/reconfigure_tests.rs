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
fn reconfigure_mean_reversion_keeps_stop_active_before_larger_window_is_ready() {
    let mut strategy = MeanReversionStrategy::new(config(serde_json::json!({
        "period": 3, "std_dev": 2.0, "stop_loss_pct": 10.0,
    })));
    strategy.initialize_historical("SOXQ", &[100, 100, 100]);
    strategy.sync_position("SOXQ", 2, 100);
    reject_scope_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "period": 100, "std_dev": 2.0, "stop_loss_pct": 2.0 });
    next.target_symbols.push("NEW".into());
    next.order_quantity = 7;
    assert!(strategy.reconfigure_preserving_position(next));
    assert!(strategy.states["SOXQ"].in_position);
    assert_eq!(strategy.states["SOXQ"].entry_price, Some(100));
    assert!(!strategy.states.contains_key("NEW"));
    assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    assert!(matches!(
        strategy.on_tick("SOXQ", 97, 0),
        Signal::Sell { quantity: 7, .. }
    ));
}

#[test]
fn reconfigure_trend_filter_retains_held_flag_and_uses_new_period_for_exit() {
    let mut strategy = TrendFilterStrategy::new(config(serde_json::json!({
        "long_period": 3, "mid_period": 2, "short_period": 1,
    })));
    strategy.initialize_historical("SOXQ", &[100, 100, 100]);
    strategy.sync_position("SOXQ", 2, 100);
    reject_scope_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "long_period": 2, "mid_period": 2, "short_period": 1 });
    next.target_symbols.push("NEW".into());
    next.order_quantity = 7;
    assert!(strategy.reconfigure_preserving_position(next));
    assert!(strategy.states["SOXQ"].in_position);
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 90, 0),
        Signal::Sell { quantity: 7, .. }
    ));
}
