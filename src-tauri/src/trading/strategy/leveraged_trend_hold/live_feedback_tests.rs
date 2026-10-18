use super::tests::{entry, strategy_with_params};
use super::*;

#[test]
fn live_feedback_lth_rejected_and_partial_exits_preserve_current_tick_protection() {
    let mut strategy = strategy_with_params(LeveragedTrendHoldParams {
        entries: vec![entry("SOXQ")],
        initial_stop_loss_pct: 3.0,
        ..Default::default()
    });
    strategy.sync_owned_position("SOXQ", 2, 100);
    assert_eq!(strategy.positions["SOXQ"].held_observations, 0);
    strategy.positions.get_mut("SOXQ").unwrap().high_water = Some(120);
    strategy
        .positions
        .get_mut("SOXQ")
        .unwrap()
        .held_observations = 37;
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
    let held = &strategy.positions["SOXQ"];
    assert!(held.in_position);
    assert_eq!(held.high_water, Some(120));
    assert_eq!(held.held_observations, 38);
    let observations = strategy.states["SOXQ"].rebound_prices.len();
    strategy.sync_owned_position("SOXQ", 1, 105);
    let held = &strategy.positions["SOXQ"];
    assert_eq!(held.entry_price, Some(105));
    assert_eq!(held.high_water, Some(120));
    assert_eq!(held.held_observations, 38);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 89, 0),
        Signal::Sell { .. }
    ));
    assert_eq!(strategy.positions["SOXQ"].held_observations, 39);
    assert_eq!(
        strategy.states["SOXQ"].rebound_prices.len(),
        observations + 1
    );
    strategy.sync_owned_position("SOXQ", 0, 0);
    assert!(!strategy.positions.contains_key("SOXQ"));
}

#[test]
fn live_feedback_lth_replay_still_clears_optimistic_exit() {
    let mut strategy = strategy_with_params(LeveragedTrendHoldParams {
        entries: vec![entry("SOXQ")],
        initial_stop_loss_pct: 3.0,
        ..Default::default()
    });
    strategy.sync_owned_position("SOXQ", 2, 100);
    assert!(matches!(
        strategy.on_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
    assert!(!strategy.positions["SOXQ"].in_position);
    assert_eq!(strategy.positions["SOXQ"].high_water, None);
}

#[test]
fn live_feedback_lth_checkpoint_keeps_peak_age_and_authoritative_owned_average() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXQ")],
        ..Default::default()
    };
    let mut before = strategy_with_params(params.clone());
    before.sync_owned_position("SOXQ", 2, 100);
    before.positions.get_mut("SOXQ").unwrap().high_water = Some(150);
    before.positions.get_mut("SOXQ").unwrap().held_observations = 73;
    let checkpoint = before.owned_position_metadata("SOXQ").unwrap();
    let mut after = strategy_with_params(params);
    assert!(after
        .restore_owned_position_metadata("SOXQ", &checkpoint)
        .is_err());
    after.sync_owned_position("SOXQ", 1, 110);
    after
        .restore_owned_position_metadata("SOXQ", &checkpoint)
        .unwrap();
    assert_eq!(after.positions["SOXQ"].entry_price, Some(110));
    assert_eq!(after.positions["SOXQ"].high_water, Some(150));
    assert_eq!(after.positions["SOXQ"].held_observations, 73);
    let mut corrupt = checkpoint;
    corrupt["highWater"] = serde_json::json!(0);
    assert!(after
        .restore_owned_position_metadata("SOXQ", &corrupt)
        .is_err());
    assert_eq!(after.positions["SOXQ"].high_water, Some(150));
}
