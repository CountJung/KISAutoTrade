use super::*;
use serde_json::json;
fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new(
        "rsi_fixture",
        "fixture",
        true,
        vec!["SOXQ".into()],
        2,
        params,
    )
}
fn strategy(extra: serde_json::Value) -> RsiStrategy {
    let mut params = json!({"period":2,"oversold":30.0,"overbought":70.0});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    RsiStrategy::new(config(params))
}
fn entry(s: &mut RsiStrategy) {
    s.initialize_historical("SOXQ", &[100, 90, 80]);
    assert!(matches!(s.on_tick("SOXQ", 100, 0), Signal::Buy { .. }));
}
#[test]
fn arithmetic_is_simple_change_mean_and_flat_zero_loss_remains_100() {
    assert_eq!(
        RsiStrategy::calc_rsi(&VecDeque::from([100, 90, 110]), 2),
        Some(100.0 - 100.0 / 3.0)
    );
    assert_eq!(
        RsiStrategy::calc_rsi(&VecDeque::from([100, 100, 100]), 2),
        Some(100.0)
    );
    assert_eq!(
        RsiStrategy::calc_rsi(&VecDeque::from([100, 90, 80]), 2),
        Some(0.0)
    );
    assert_eq!(RsiStrategy::calc_rsi(&VecDeque::from([100]), 2), None);
    assert_eq!(RsiStrategy::calc_rsi(&VecDeque::from([100]), 0), None);
}
#[test]
fn flat_sell_and_held_buy_are_removed_without_mutating_actual_position() {
    let mut s = strategy(json!({}));
    s.initialize_historical("SOXQ", &[80, 90, 100]);
    assert!(matches!(s.on_tick("SOXQ", 80, 0), Signal::Hold));
    assert_eq!(s.states["SOXQ"].previous_rsi, Some(100.0 - 100.0 / 1.5));
    s.initialize_historical("SOXQ", &[100, 90, 80]);
    s.sync_position("SOXQ", 7, 95);
    assert!(matches!(s.on_tick("SOXQ", 100, 0), Signal::Hold));
    assert_eq!(s.has_held_position("SOXQ"), Some(true));
    assert_eq!(s.states["SOXQ"].owned_quantity, 7);
}
#[test]
fn raw_entry_is_not_fill_and_false_feedback_does_not_retry_crossing() {
    let mut s = strategy(json!({}));
    entry(&mut s);
    s.sync_position("SOXQ", 0, 0);
    assert_eq!(s.has_held_position("SOXQ"), Some(false));
    assert!(matches!(s.on_tick("SOXQ", 110, 0), Signal::Hold));
    assert_eq!(s.states["SOXQ"].previous_rsi, Some(100.0));
    assert!(s.states["SOXQ"].entry_candidate);
}
#[test]
fn original_cross_exit_sells_actual_quantity_once_even_if_not_filled() {
    let mut s = strategy(json!({}));
    s.sync_position("SOXQ", 7, 100);
    s.initialize_historical("SOXQ", &[80, 90, 100]);
    assert!(matches!(
        s.on_tick("SOXQ", 80, 0),
        Signal::Sell { quantity: 7, .. }
    ));
    s.sync_position("SOXQ", 3, 100);
    assert!(matches!(s.on_tick("SOXQ", 79, 0), Signal::Hold));
    assert_eq!(s.states["SOXQ"].owned_quantity, 3);
}
#[test]
fn trend_uses_previous_observations_strictly_and_does_not_retry_filtered_cross() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.initialize_historical("SOXQ", &[100, 90, 80]);
    assert!(matches!(s.on_tick("SOXQ", 90, 0), Signal::Hold)); // prior mean=90 equality rejected
    assert!(matches!(s.on_tick("SOXQ", 100, 0), Signal::Hold)); // original crossing consumed
    let mut allowed = strategy(json!({"long_trend_period":3}));
    entry(&mut allowed);
    assert_eq!(allowed.states["SOXQ"].trend_prices.len(), 4);
}
#[test]
fn live_trend_drops_seed_once_and_uses_observations_not_days_without_erasing_rsi() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.initialize_historical("SOXQ", &[100, 90, 80]);
    assert!(matches!(s.on_live_tick("SOXQ", 100, 0), Signal::Hold));
    assert_eq!(s.states["SOXQ"].prices.len(), 4);
    assert!(s.states["SOXQ"].previous_rsi.unwrap() > 30.0);
    assert_eq!(s.states["SOXQ"].trend_prices, VecDeque::from([100]));
    s.on_live_tick("SOXQ", 110, 0);
    s.initialize_historical("SOXQ", &[900, 800, 700]);
    assert_eq!(s.states["SOXQ"].trend_prices, VecDeque::from([100, 110]));
    s.on_live_tick("SOXQ", 120, 0);
    assert_eq!(
        s.states["SOXQ"].trend_prices,
        VecDeque::from([100, 110, 120])
    );
}
#[test]
fn stop_uses_confirmed_native_average_and_zero_disables_it() {
    let mut s = strategy(json!({"stop_loss_pct":3.0}));
    s.sync_position("SOXQ", 7, 100);
    assert!(matches!(s.on_tick("SOXQ", 98, 0), Signal::Hold));
    assert!(
        matches!(s.on_tick("SOXQ",97,0),Signal::Sell{quantity:7,ref reason,..} if reason.contains("손절"))
    );
    s.sync_position("SOXQ", 3, 120);
    assert!(matches!(
        s.on_tick("SOXQ", 100, 0),
        Signal::Sell { quantity: 3, .. }
    ));
    let mut disabled = strategy(json!({"stop_loss_pct":0}));
    disabled.sync_position("SOXQ", 7, 100);
    assert!(matches!(disabled.on_tick("SOXQ", 1, 0), Signal::Hold));
}
#[test]
fn protection_precedes_readiness_trend_validation_and_original_cross() {
    let mut s = strategy(
        json!({"period":500,"long_trend_period":"bad","stop_loss_pct":3,"max_hold_observations":1}),
    );
    s.sync_position("SOXQ", 7, 100);
    assert_eq!(s.can_evaluate_next_tick("SOXQ"), Some(true));
    assert!(
        matches!(s.on_tick("SOXQ",97,0),Signal::Sell{ref reason,..} if reason.contains("손절"))
    );
    assert!(
        matches!(s.on_tick("SOXQ",98,0),Signal::Sell{ref reason,..} if reason.contains("나이 불명"))
    );
}
#[test]
fn known_age_starts_after_actual_fill_and_preserves_partial_same_quantity_retries() {
    let mut s = strategy(json!({"max_hold_observations":2}));
    entry(&mut s);
    s.sync_position("SOXQ", 0, 0);
    assert_eq!(s.states["SOXQ"].held_observations, None);
    s.sync_position("SOXQ", 7, 100);
    assert_eq!(s.states["SOXQ"].held_observations, Some(0));
    assert!(matches!(s.on_tick("SOXQ", 100, 0), Signal::Hold));
    s.sync_position("SOXQ", 7, 100);
    s.sync_position("SOXQ", 3, 100);
    assert!(matches!(
        s.on_tick("SOXQ", 100, 0),
        Signal::Sell { quantity: 3, .. }
    ));
    assert_eq!(s.states["SOXQ"].held_observations, Some(2));
    assert!(matches!(
        s.on_tick("SOXQ", 100, 0),
        Signal::Sell { quantity: 3, .. }
    ));
    s.sync_position("SOXQ", 0, 0);
    assert!(matches!(s.on_tick("SOXQ", 100, 0), Signal::Hold));
}
#[test]
fn cold_owned_age_stays_unknown_and_active_cap_offers_conservative_exit() {
    let mut s = strategy(json!({"max_hold_observations":63}));
    s.sync_position("SOXQ", 3, 100);
    assert!(
        matches!(s.on_tick("SOXQ",100,0),Signal::Sell{quantity:3,ref reason,..} if reason.contains("나이 불명"))
    );
    assert_eq!(s.states["SOXQ"].held_observations, None);
    assert_eq!(
        s.owned_position_metadata("SOXQ").unwrap()["ageKnown"],
        false
    );
}
#[test]
fn disabled_cap_still_counts_known_age_and_reconfigure_does_not_reset_it() {
    let mut s = strategy(json!({}));
    entry(&mut s);
    s.sync_position("SOXQ", 3, 100);
    s.on_tick("SOXQ", 100, 0);
    s.on_tick("SOXQ", 100, 0);
    let mut cfg = s.config.clone();
    cfg.params["max_hold_observations"] = 2.into();
    assert!(s.reconfigure_preserving_position(cfg));
    assert_eq!(s.states["SOXQ"].held_observations, Some(2));
    assert!(matches!(
        s.on_tick("SOXQ", 100, 0),
        Signal::Sell { quantity: 3, .. }
    ));
}
#[test]
fn pending_origin_survives_history_reconfiguration_and_checkpoint_then_fill() {
    let mut s = strategy(json!({"max_hold_observations":3}));
    entry(&mut s);
    let metadata = s.owned_position_metadata("SOXQ").unwrap();
    s.initialize_historical("SOXQ", &[100, 90, 80]);
    let mut cfg = s.config.clone();
    cfg.params["period"] = 3.into();
    assert!(s.reconfigure_preserving_position(cfg));
    assert!(s.states["SOXQ"].entry_candidate);
    let mut restored = strategy(json!({"max_hold_observations":3}));
    restored.sync_position("SOXQ", 0, 0);
    restored
        .restore_owned_position_metadata("SOXQ", &metadata)
        .unwrap();
    restored.sync_position("SOXQ", 4, 120);
    assert_eq!(restored.states["SOXQ"].held_observations, Some(0));
    assert_eq!(restored.states["SOXQ"].avg_price, 120);
}
#[test]
fn checkpoint_validation_is_atomic_and_kind_separated() {
    let mut s = strategy(json!({}));
    entry(&mut s);
    s.sync_position("SOXQ", 4, 120);
    s.on_tick("SOXQ", 120, 0);
    let metadata = s.owned_position_metadata("SOXQ").unwrap();
    for patch in [
        json!({"kind":"mean_reversion"}),
        json!({"version":2}),
        json!({"extra":true}),
        json!({"heldObservations":-1}),
        json!({"ageKnown":false}),
    ] {
        let mut bad = metadata.clone();
        bad.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(s.restore_owned_position_metadata("SOXQ", &bad).is_err());
        assert_eq!(s.owned_position_metadata("SOXQ").unwrap(), metadata);
    }
    let mut restored = strategy(json!({}));
    restored.sync_position("SOXQ", 2, 120);
    restored
        .restore_owned_position_metadata("SOXQ", &metadata)
        .unwrap();
    restored.initialize_historical("SOXQ", &[100, 90, 80]);
    assert_eq!(restored.states["SOXQ"].held_observations, Some(1));
    assert_eq!(restored.states["SOXQ"].owned_quantity, 2);
}
#[test]
fn strict_optional_contract_rejects_damaged_values_but_accepts_integer_floats() {
    for key in [
        "long_trend_period",
        "max_hold_observations",
        "stop_loss_pct",
    ] {
        for invalid in [json!(null), json!("1"), json!(false), json!(-1)] {
            let mut value = json!({});
            value[key] = invalid;
            assert!(validate_rsi_params("rsi_test", &value).is_err());
            assert!(validate_rsi_params("ma_cross", &value).is_ok());
        }
    }
    for value in [
        json!({"long_trend_period":513}),
        json!({"long_trend_period":0.5}),
        json!({"max_hold_observations":1000001}),
        json!({"max_hold_observations":0.5}),
        json!({"stop_loss_pct":100}),
    ] {
        assert!(validate_rsi_params("rsi", &value).is_err());
    }
    assert!(validate_rsi_params(
        "rsi",
        &json!({"long_trend_period":512.0,"max_hold_observations":1000000.0,"stop_loss_pct":99.9})
    )
    .is_ok());
    let s = strategy(json!({"long_trend_period":2.0,"max_hold_observations":3.0}));
    assert_eq!(
        (s.params.long_trend_period, s.params.max_hold_observations),
        (2, 3)
    );
}
#[test]
fn damaged_optional_runtime_fails_buy_closed_and_retains_custom_core_and_valid_exits() {
    let mut s = strategy(json!({"long_trend_period":null,"overbought":60.0,"stop_loss_pct":3.0}));
    assert_eq!(s.params.period, 2);
    assert_eq!(s.params.overbought, 60.0);
    s.initialize_historical("SOXQ", &[100, 90, 80]);
    assert!(matches!(s.on_tick("SOXQ", 100, 0), Signal::Hold));
    s.sync_position("SOXQ", 4, 110);
    assert!(matches!(
        s.on_tick("SOXQ", 100, 0),
        Signal::Sell { quantity: 4, .. }
    ));
    let mut legacy = strategy(json!({"stop_loss_pct":null,"overbought":60.0}));
    legacy.sync_position("SOXQ", 4, 100);
    legacy.initialize_historical("SOXQ", &[80, 90, 100]);
    assert!(matches!(
        legacy.on_tick("SOXQ", 80, 0),
        Signal::Sell { quantity: 4, .. }
    ));
}
#[test]
fn targets_scope_disable_and_reset_keep_explicit_contracts() {
    let mut s = strategy(json!({"stop_loss_pct":3}));
    s.sync_position("SOXQ", 3, 100);
    s.sync_position("OTHER", 8, 100);
    assert!(!s.states.contains_key("OTHER"));
    assert!(matches!(s.on_tick("OTHER", 1, 0), Signal::Hold));
    let mut cfg = s.config.clone();
    cfg.target_symbols.clear();
    assert!(!s.reconfigure_preserving_position(cfg));
    let mut cfg = s.config.clone();
    cfg.broker_account_id = Some("other".into());
    assert!(!s.reconfigure_preserving_position(cfg));
    s.set_enabled(false);
    assert!(matches!(s.on_tick("SOXQ", 1, 0), Signal::Hold));
    assert_eq!(s.states["SOXQ"].held_observations, None);
    s.reset();
    assert!(s.states.is_empty());
}
#[test]
fn rsi_and_prior_trend_buffers_remain_bounded_without_integer_sum_overflow() {
    let mut s = strategy(json!({"period":14,"long_trend_period":512}));
    s.initialize_historical("SOXQ", &vec![u64::MAX; 10000]);
    assert_eq!(s.states["SOXQ"].prices.len(), 16);
    assert_eq!(s.states["SOXQ"].trend_prices.len(), 512);
    assert_eq!(s.states["SOXQ"].previous_rsi, Some(100.0));
    for _ in 0..1000 {
        s.on_tick("SOXQ", u64::MAX, 0);
    }
    assert_eq!(s.states["SOXQ"].prices.len(), 16);
    assert_eq!(s.states["SOXQ"].trend_prices.len(), 512);
}
