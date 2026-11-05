use super::*;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new(
        "price_condition",
        "PC",
        true,
        vec!["IGNORED".into()],
        1,
        params,
    )
}

#[test]
fn reconfigure_price_condition_canonicalizes_targets_and_keeps_actual_stop() {
    let mut strategy = PriceConditionStrategy::new(config(serde_json::json!({
        "symbols": [{ "symbol": "SOXQ", "buy_trigger_price": 100.0, "stop_loss_pct": 10.0 }],
    })));
    assert_eq!(strategy.config.target_symbols, vec!["SOXQ"]);
    strategy.sync_position("SOXQ", 2, 100);
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
    next.target_symbols = vec!["IGNORED".into()];
    next.params = serde_json::json!({ "symbols": [
        { "symbol": "SOXQ", "quantity": 7, "buy_trigger_price": 100.0, "stop_loss_pct": 2.0 },
        { "symbol": "NEW", "buy_trigger_price": 0.0 },
    ] });
    assert!(strategy.reconfigure_preserving_position(next));
    assert_eq!(strategy.config.target_symbols, vec!["SOXQ", "NEW"]);
    assert_eq!(strategy.positions["SOXQ"], (true, Some(100)));
    assert!(!strategy.positions.contains_key("NEW"));
    assert!(matches!(
        strategy.on_tick("SOXQ", 97, 0),
        Signal::Sell { quantity: 7, .. }
    ));
    assert!(matches!(strategy.on_tick("NEW", 97, 0), Signal::Hold));
}

#[test]
fn reconfigure_price_condition_does_not_move_holdings_to_replacement_target() {
    let mut strategy = PriceConditionStrategy::new(config(serde_json::json!({
        "symbols": [{ "symbol": "SOXQ" }],
    })));
    strategy.sync_position("SOXQ", 2, 100);
    let mut next = strategy.config().clone();
    next.params = serde_json::json!({ "symbols": [{ "symbol": "NEW", "buy_trigger_price": 0.0 }] });
    // Public manager rejects removal while held; the strategy itself never copies it to NEW.
    assert!(strategy.reconfigure_preserving_position(next));
    assert!(strategy.positions.is_empty());
    assert!(matches!(strategy.on_tick("NEW", 100, 0), Signal::Hold));
}

#[test]
fn empty_price_condition_entries_remove_stale_targets_before_manager_validation() {
    let mut old = config(serde_json::json!({
        "symbols": [{ "symbol": "SOXQ" }],
    }));
    old.target_symbols = vec!["SOXQ".into()];
    let mut strategy = PriceConditionStrategy::new(old.clone());
    strategy.sync_position("SOXQ", 2, 100);
    old.params = serde_json::json!({ "symbols": [] });

    // The manager validates this canonical candidate before changing live state.
    let candidate = PriceConditionStrategy::new(old);
    assert!(candidate.config().target_symbols.is_empty());
    assert!(!candidate.config().targets_symbol("SOXQ"));
    assert_eq!(strategy.has_held_position("SOXQ"), Some(true));
    assert!(matches!(
        strategy.on_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
}
