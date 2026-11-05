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
fn reconfigure_fifty_two_week_keeps_actual_entry_while_invalidating_old_window() {
    let mut strategy = FiftyTwoWeekHighStrategy::new(config(serde_json::json!({
        "lookback_days": 3, "stop_loss_pct": 10.0,
    })));
    strategy.initialize_historical("SOXQ", &[100, 100, 100]);
    strategy.sync_position("SOXQ", 2, 100);
    reject_scope_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "lookback_days": 100, "stop_loss_pct": 2.0 });
    next.order_quantity = 7;
    next.target_symbols.push("NEW".into());
    assert!(strategy.reconfigure_preserving_position(next));
    assert_eq!(strategy.states["SOXQ"].buy_price, Some(100));
    assert_eq!(strategy.states["SOXQ"].high_52w, None);
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 97, 0),
        Signal::Sell { quantity: 7, .. }
    ));
}

#[test]
fn reconfigure_strong_close_preserves_stop_but_discards_old_entry_candidate() {
    let mut strategy = StrongCloseStrategy::new(config(serde_json::json!({
        "threshold_pct": 3.0, "stop_loss_pct": 10.0,
    })));
    strategy.initialize_candles("SOXQ", &[(100, 100)]);
    strategy.sync_position("SOXQ", 2, 100);
    reject_scope_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "threshold_pct": 1.0, "stop_loss_pct": 2.0 });
    next.order_quantity = 7;
    next.target_symbols.push("NEW".into());
    assert!(strategy.reconfigure_preserving_position(next));
    assert!(strategy.states["SOXQ"].owned_quantity > 0);
    assert_eq!(strategy.states["SOXQ"].avg_price, 100);
    assert!(!strategy.states["SOXQ"].pending_buy);
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 97, 0),
        Signal::Sell { quantity: 2, .. }
    ));
    strategy.sync_position("SOXQ", 0, 0);
    strategy.initialize_candles("SOXQ", &[(100, 100)]);
    assert!(strategy.states["SOXQ"].pending_buy);
    let next = strategy.config().clone();
    assert!(strategy.reconfigure_preserving_position(next));
    assert!(!strategy.states["SOXQ"].pending_buy);
    assert!(matches!(strategy.on_tick("SOXQ", 100, 0), Signal::Hold));
}

#[test]
fn reconfigure_volatility_recomputes_range_without_losing_held_stop_or_day() {
    let mut strategy = VolatilityExpansionStrategy::new(config(serde_json::json!({
        "lookback_days": 3, "expansion_factor": 2.0, "stop_loss_pct": 10.0,
    })));
    strategy.initialize_range_data("SOXQ", &[10, 20, 30]);
    strategy.on_trading_day_start("SOXQ", 105);
    strategy.sync_position("SOXQ", 2, 100);
    reject_scope_changes(&mut strategy);
    let mut next = strategy.config().clone();
    next.params =
        serde_json::json!({ "lookback_days": 2, "expansion_factor": 3.0, "stop_loss_pct": 2.0 });
    next.order_quantity = 7;
    next.target_symbols.push("NEW".into());
    assert!(strategy.reconfigure_preserving_position(next));
    let state = &strategy.states["SOXQ"];
    assert!(state.owned_quantity > 0);
    assert_eq!(state.avg_price, 100);
    assert_eq!(state.day_open, Some(105));
    assert_eq!(state.avg_range, Some(25.0));
    assert!(!strategy.states.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 97, 0),
        Signal::Sell { quantity: 2, .. }
    ));
}
