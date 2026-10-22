use super::*;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 2, params)
}

#[test]
fn live_feedback_consecutive_keeps_confirmed_flag_and_observations() {
    let mut strategy =
        ConsecutiveMoveStrategy::new(config(serde_json::json!({"buy_days":2,"sell_days":2})));
    strategy.initialize_historical("SOXQ", &[100, 110]);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 120, 0),
        Signal::Buy { .. }
    ));
    assert!(!strategy.states["SOXQ"].in_position);
    assert_eq!(strategy.states["SOXQ"].prices.back(), Some(&120));
    strategy.sync_owned_position("SOXQ", 2, 120);
    strategy.on_live_tick("SOXQ", 110, 0);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 100, 0),
        Signal::Sell { .. }
    ));
    assert!(strategy.states["SOXQ"].in_position);
    assert_eq!(strategy.states["SOXQ"].prices.back(), Some(&100));
}

#[test]
fn live_feedback_breakout_freezes_pending_signal_anchor_until_first_fill() {
    let mut strategy = FailedBreakoutStrategy::new(config(
        serde_json::json!({"lookback_days":3,"buffer_pct":0.5}),
    ));
    strategy.initialize_historical("SOXQ", &[95, 100, 98]);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 101, 0),
        Signal::Buy { .. }
    ));
    assert_eq!(strategy.states["SOXQ"].owned_quantity, 0);
    assert_eq!(strategy.states["SOXQ"].candidate_prev_high, Some(100));
    strategy.set_order_pending("SOXQ", true);
    strategy.on_live_tick("SOXQ", 110, 0);
    strategy.discard_unsubmitted_signal("SOXQ");
    assert_eq!(strategy.states["SOXQ"].candidate_prev_high, Some(100));
    strategy.sync_owned_position("SOXQ", 1, 108);
    assert_eq!(strategy.states["SOXQ"].breakout_prev_high, Some(100));
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 99, 0),
        Signal::Sell { .. }
    ));
    assert!(strategy.states["SOXQ"].owned_quantity > 0);
    assert_eq!(strategy.states["SOXQ"].breakout_prev_high, Some(100));
    strategy.sync_owned_position("SOXQ", 1, 108);
    assert_eq!(strategy.states["SOXQ"].breakout_prev_high, Some(100));
    strategy.sync_owned_position("SOXQ", 0, 0);
    assert_eq!(strategy.states["SOXQ"].breakout_prev_high, None);
}

#[test]
fn live_feedback_unsubmitted_breakout_discards_only_candidate_not_history() {
    let mut strategy = FailedBreakoutStrategy::new(config(
        serde_json::json!({"lookback_days":3,"buffer_pct":0.5}),
    ));
    strategy.initialize_historical("SOXQ", &[95, 100, 98]);
    strategy.on_live_tick("SOXQ", 101, 0);
    strategy.discard_unsubmitted_signal("SOXQ");
    assert_eq!(strategy.states["SOXQ"].candidate_prev_high, None);
    assert_eq!(strategy.states["SOXQ"].prices.back(), Some(&101));
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 110, 0),
        Signal::Buy { .. }
    ));
    assert_eq!(strategy.states["SOXQ"].candidate_prev_high, Some(101));
}

#[test]
fn live_feedback_breakout_checkpoint_restores_pending_and_confirmed_exact_anchor() {
    let config = config(serde_json::json!({"lookback_days":3,"buffer_pct":0.5}));
    let mut before = FailedBreakoutStrategy::new(config.clone());
    before.initialize_historical("SOXQ", &[95, 100, 98]);
    before.on_live_tick("SOXQ", 101, 0);
    let candidate = before.owned_position_metadata("SOXQ").unwrap();
    let mut after = FailedBreakoutStrategy::new(config.clone());
    after.sync_owned_position("SOXQ", 0, 0);
    after
        .restore_owned_position_metadata("SOXQ", &candidate)
        .unwrap();
    assert_eq!(after.states["SOXQ"].candidate_prev_high, Some(100));
    after.sync_owned_position("SOXQ", 1, 111);
    assert_eq!(after.states["SOXQ"].breakout_prev_high, Some(100));
    let confirmed = after.owned_position_metadata("SOXQ").unwrap();
    let mut restarted = FailedBreakoutStrategy::new(config);
    restarted.sync_owned_position("SOXQ", 1, 111);
    restarted
        .restore_owned_position_metadata("SOXQ", &confirmed)
        .unwrap();
    assert!(matches!(
        restarted.on_live_tick("SOXQ", 99, 0),
        Signal::Sell { .. }
    ));
    assert_eq!(restarted.states["SOXQ"].breakout_prev_high, Some(100));
    let mut corrupt = confirmed;
    corrupt["kind"] = serde_json::json!("wrong");
    assert!(restarted
        .restore_owned_position_metadata("SOXQ", &corrupt)
        .is_err());
    assert_eq!(restarted.states["SOXQ"].breakout_prev_high, Some(100));
}
