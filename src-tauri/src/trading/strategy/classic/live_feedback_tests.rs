use super::*;

#[test]
fn live_feedback_momentum_keeps_signal_suppression_confirmed_and_prices_current() {
    let config = StrategyConfig::new(
        "momentum",
        "M",
        true,
        vec!["SOXQ".into()],
        1,
        serde_json::json!({"lookback_period":2,"threshold_pct":5.0}),
    );
    let mut live = MomentumStrategy::new(config.clone());
    live.initialize_historical("SOXQ", &[100, 100]);
    assert!(matches!(
        live.on_live_tick("SOXQ", 110, 0),
        Signal::Buy { .. }
    ));
    assert_eq!(live.states["SOXQ"].owned_quantity, 0);
    assert_eq!(live.states["SOXQ"].prices.back(), Some(&110));
    live.sync_owned_position("SOXQ", 2, 107);
    assert_eq!(live.states["SOXQ"].owned_quantity, 2);
    assert!(matches!(
        live.on_live_tick("SOXQ", 80, 0),
        Signal::Sell { .. }
    ));
    assert_eq!(live.states["SOXQ"].owned_quantity, 2);
    assert_eq!(live.states["SOXQ"].prices.back(), Some(&80));
    live.sync_owned_position("SOXQ", 0, 0);
    assert_eq!(live.states["SOXQ"].owned_quantity, 0);
    let mut replay = MomentumStrategy::new(config);
    replay.initialize_historical("SOXQ", &[100, 100]);
    assert!(matches!(replay.on_tick("SOXQ", 110, 0), Signal::Buy { .. }));
    assert_eq!(replay.states["SOXQ"].owned_quantity, 0);
}
