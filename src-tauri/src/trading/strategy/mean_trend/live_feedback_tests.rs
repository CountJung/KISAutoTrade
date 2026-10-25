use super::*;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 2, params)
}

#[test]
fn live_feedback_mean_reversion_keeps_entry_and_tick_history_on_sell() {
    let mut strategy = MeanReversionStrategy::new(config(
        serde_json::json!({"period":3,"std_dev":1.0,"stop_loss_pct":3.0}),
    ));
    strategy.initialize_historical("SOXQ", &[100, 100, 100]);
    strategy.sync_owned_position("SOXQ", 2, 100);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
    assert!(strategy.states["SOXQ"].owned_quantity > 0);
    assert_eq!(strategy.states["SOXQ"].entry_price, Some(100));
    assert_eq!(strategy.states["SOXQ"].prices.back(), Some(&90));
    strategy.sync_owned_position("SOXQ", 1, 100);
    assert!(strategy.states["SOXQ"].owned_quantity > 0);
    strategy.sync_owned_position("SOXQ", 0, 0);
    assert!(strategy.states["SOXQ"].owned_quantity == 0);
}

#[test]
fn live_feedback_trend_filter_keeps_confirmation_and_updates_indicators() {
    let mut strategy = TrendFilterStrategy::new(config(
        serde_json::json!({"long_period":3,"short_period":1,"mid_period":2}),
    ));
    strategy.initialize_historical("SOXQ", &[100, 110, 120]);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 130, 0),
        Signal::Buy { .. }
    ));
    assert!(!(strategy.states["SOXQ"].owned_quantity > 0));
    assert_eq!(strategy.states["SOXQ"].prices.back(), Some(&130));
    strategy.sync_owned_position("SOXQ", 1, 130);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
    assert!((strategy.states["SOXQ"].owned_quantity > 0));
}
