use super::*;
use serde_json::{json, Value};

fn config(extra: Value) -> StrategyConfig {
    let mut p = json!({"period":3,"std_dev":1.0,"stop_loss_pct":50.0});
    p.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    StrategyConfig::new(
        "mean_reversion_fixture",
        "fixture",
        true,
        vec!["A".into(), "B".into()],
        2,
        p,
    )
}
fn strategy(extra: Value) -> MeanReversionStrategy {
    let mut s = MeanReversionStrategy::new(config(extra));
    s.initialize_historical("A", &[100, 100, 100]);
    s
}
fn buy(s: Signal) {
    assert!(matches!(s, Signal::Buy { .. }), "{s:?}");
}
fn sell(s: Signal, q: u64) {
    assert!(
        matches!(s, Signal::Sell {quantity,..} if quantity==q),
        "{s:?}"
    );
}
fn hold(s: Signal) {
    assert_eq!(s, Signal::Hold);
}
fn filled(extra: Value) -> MeanReversionStrategy {
    let mut s = strategy(extra);
    buy(s.on_tick("A", 80, 0));
    s.sync_owned_position("A", 5, 80);
    s
}

#[test]
fn raw_live_and_rejected_candidates_do_not_manufacture_ownership_or_entry_price() {
    for live in [false, true] {
        let mut s = strategy(json!({}));
        buy(if live {
            s.on_live_tick("A", 80, 0)
        } else {
            s.on_tick("A", 80, 0)
        });
        assert_eq!(s.has_held_position("A"), Some(false));
        assert_eq!(s.states["A"].entry_price, None);
        s.sync_position("A", 0, 0);
        buy(s.on_tick("A", 70, 0));
        s.sync_owned_position("A", 5, 75);
        assert_eq!(s.states["A"].entry_price, Some(75));
        hold(s.on_tick("A", 60, 0));
        sell(s.on_tick("A", 120, 0), 2);
        assert_eq!(s.states["A"].owned_quantity, 5);
        assert_eq!(s.states["A"].entry_price, Some(75));
        s.sync_owned_position("A", 1, 75);
        sell(s.on_live_tick("A", 150, 0), 2);
        s.sync_owned_position("A", 0, 0);
        hold(s.on_tick("A", 160, 0));
    }
}

#[test]
fn current_inclusive_population_bands_and_strict_boundaries_are_preserved() {
    let mut s = strategy(json!({"std_dev":0.0}));
    hold(s.on_tick("A", 100, 0)); // exactly lower, upper, and mean
    buy(s.on_tick("A", 99, 0));
    s.sync_position("A", 5, 99);
    s.initialize_historical("A", &[100, 100, 100]);
    hold(s.on_tick("A", 100, 0));
    sell(s.on_tick("A", 101, 0), 2);
    assert_eq!(
        MeanReversionStrategy::bollinger_bands(&VecDeque::from([0, 0, 3]), 3, 1.0),
        Some((1.0, 1.0 + 2.0_f64.sqrt(), 1.0 - 2.0_f64.sqrt()))
    );
}

#[test]
fn middle_band_is_optional_and_strict_and_upper_band_keeps_original_reason() {
    for mode in ["upperBand", "middleBand"] {
        let mut s = strategy(json!({"exit_mode":mode,"std_dev":2.0}));
        s.sync_position("A", 5, 80);
        hold(s.on_tick("A", 100, 0)); // equal mean is not a middle exit
        let signal = s.on_tick("A", 101, 0);
        if mode == "middleBand" {
            sell(signal, 2);
        } else {
            hold(signal);
        }
    }
    let mut s = strategy(json!({}));
    s.sync_position("A", 5, 100);
    match s.on_tick("A", 120, 0) {
        Signal::Sell { reason, .. } => assert_eq!(
            reason,
            "평균회귀 익절: 현재가 120 > 상단밴드 116 (mean=107)"
        ),
        v => panic!("{v:?}"),
    }
}

#[test]
fn original_stop_expression_inclusive_boundary_zero_and_priority_are_preserved() {
    for (stop, price) in [(10.0, 90), (0.0, 100)] {
        let mut s = strategy(
            json!({"stop_loss_pct":stop,"max_hold_observations":1,"exit_mode":"middleBand"}),
        );
        s.sync_position("A", 5, 100);
        match s.on_tick("A", price, 0) {
            Signal::Sell {
                quantity, reason, ..
            } => {
                assert_eq!(quantity, 2);
                assert!(reason.contains("손절"));
            }
            v => panic!("{v:?}"),
        }
    }
    let mut s = strategy(json!({"stop_loss_pct":0.0}));
    s.sync_position("A", 5, 100);
    s.initialize_historical("A", &[110, 110, 110]);
    hold(s.on_tick("A", 101, 0)); // zero native stop does not stop profits
}

#[test]
fn stop_and_age_remain_evaluable_before_band_or_trend_preparation() {
    let mut s = MeanReversionStrategy::new(config(
        json!({"period":100,"long_trend_period":512,"stop_loss_pct":5.0}),
    ));
    s.sync_position("A", 5, 100);
    assert_eq!(s.can_evaluate_next_tick("A"), Some(true));
    sell(s.on_tick("A", 95, 0), 2);
    assert!(!s.history_readiness("A").unwrap().ready);
    let mut s = MeanReversionStrategy::new(config(
        json!({"period":100,"long_trend_period":512,"max_hold_observations":3}),
    ));
    s.sync_position("A", 5, 0);
    assert_eq!(s.can_evaluate_next_tick("A"), Some(true));
    sell(s.on_tick("A", 100, 0), 5);
}

#[test]
fn trend_uses_strict_prior_observations_without_current_or_future_price() {
    let mut s = strategy(json!({"long_trend_period":10}));
    s.initialize_historical("A", &[1, 1, 1, 1, 1, 1, 1, 100, 100, 100]);
    buy(s.on_tick("A", 80, 0)); // prior trend30.7; band lower83.91 includes current80
    let mut s = strategy(json!({"long_trend_period":4}));
    s.initialize_historical("A", &[0, 0, 100, 100]);
    hold(s.on_tick("A", 50, 0)); // lower59.76 permits entry, equality to prior trend50 does not
    let mut s = strategy(json!({"long_trend_period":4}));
    s.initialize_historical("A", &[100, 100, 100]);
    assert_eq!(s.can_evaluate_next_tick("A"), Some(false));
    hold(s.on_tick("A", 80, 0));
}

#[test]
fn live_trend_rejects_warmup_cadence_and_preserves_real_quotes_on_refetch_edit() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.initialize_historical("A", &vec![20; 300]);
    s.initialize_intraday_prices("A", &[30, 40, 100]); // legacy no-op
    hold(s.on_live_tick("A", 80, 0));
    assert_eq!(s.states["A"].trend_prices, VecDeque::from([80]));
    s.on_live_tick("A", 50, 0);
    s.on_live_tick("A", 100, 0);
    let context = s.states["A"].trend_prices.clone();
    s.initialize_historical("A", &[100, 100, 100]);
    assert_eq!(s.states["A"].trend_prices, context);
    let mut next = s.config.clone();
    next.params["long_trend_period"] = json!(2);
    assert!(s.reconfigure_preserving_position(next));
    assert_eq!(s.states["A"].trend_prices, context);
    buy(s.on_live_tick("A", 80, 0)); // previous live mean75, bands current(100,100,80)
}

#[test]
fn trend_records_off_observations_bounded_and_zero_mean_blocks_only_entry() {
    let mut s = strategy(json!({}));
    for _ in 0..1000 {
        s.on_live_tick("A", 100, 0);
    }
    assert_eq!(s.states["A"].trend_prices.len(), 512);
    let mut next = s.config.clone();
    next.params["long_trend_period"] = json!(512.0);
    s.reconfigure_preserving_position(next);
    assert!(s.history_readiness("A").unwrap().ready);
    assert_eq!(s.history_readiness("A").unwrap().available_bars, 513);
    let mut s = strategy(json!({"long_trend_period":2}));
    s.initialize_historical("A", &[100, 0, 0]);
    hold(s.on_tick("A", 0, 0));
    s.sync_position("A", 5, 100);
    sell(s.on_tick("A", 0, 0), 2);
}

#[test]
fn age_starts_after_positive_fill_and_survives_same_partial_and_warmup_feedback() {
    let mut s = strategy(json!({"max_hold_observations":3}));
    buy(s.on_tick("A", 80, 0));
    s.sync_position("A", 0, 0);
    buy(s.on_tick("A", 70, 0));
    assert_eq!(s.states["A"].held_observations, None);
    s.sync_position("A", 5, 70);
    assert_eq!(s.states["A"].held_observations, Some(0));
    hold(s.on_tick("A", 70, 0));
    s.sync_position("A", 5, 70);
    s.initialize_historical("A", &[70, 70, 70]);
    hold(s.on_live_tick("A", 70, 0));
    s.sync_position("A", 2, 70);
    sell(s.on_tick("A", 70, 0), 2);
    assert_eq!(s.states["A"].held_observations, Some(3));
    s.sync_position("A", 1, 70);
    sell(s.on_tick("A", 70, 0), 1);
    assert_eq!(s.states["A"].held_observations, Some(4));
}

#[test]
fn off_age_continues_through_config_edits_and_unknown_restores_exit_conservatively() {
    let mut s = filled(json!({}));
    for _ in 0..5 {
        hold(s.on_tick("A", 80, 0));
    }
    let mut next = s.config.clone();
    next.params["max_hold_observations"] = json!(3);
    next.params["period"] = json!(100);
    s.reconfigure_preserving_position(next);
    assert_eq!(s.states["A"].held_observations, Some(5));
    sell(s.on_tick("A", 80, 0), 5);
    let mut s = strategy(json!({"max_hold_observations":63}));
    s.sync_owned_position("A", 5, 100);
    match s.on_tick("A", 100, 0) {
        Signal::Sell {
            quantity, reason, ..
        } => {
            assert_eq!(quantity, 5);
            assert!(reason.contains("불명"));
        }
        v => panic!("{v:?}"),
    }
}

#[test]
fn same_params_preserve_pending_origin_and_changed_params_clear_only_unconfirmed_origin() {
    let mut s = strategy(json!({"max_hold_observations":3}));
    buy(s.on_tick("A", 80, 0));
    let before = s.owned_position_metadata("A");
    let mut next = s.config.clone();
    next.order_quantity = 8;
    s.reconfigure_preserving_position(next);
    assert_eq!(s.owned_position_metadata("A"), before);
    s.sync_position("A", 5, 80);
    assert_eq!(s.states["A"].held_observations, Some(0));
    let mut s = strategy(json!({"max_hold_observations":3}));
    buy(s.on_tick("A", 80, 0));
    let mut next = s.config.clone();
    next.params["period"] = json!(4);
    s.reconfigure_preserving_position(next);
    s.sync_position("A", 5, 80);
    assert_eq!(s.states["A"].held_observations, None);
}

#[test]
fn exact_metadata_roundtrips_pending_and_confirmed_age_without_inventing_ownership() {
    let mut s = strategy(json!({"max_hold_observations":3}));
    buy(s.on_tick("A", 80, 0));
    let pending = s.owned_position_metadata("A").unwrap();
    let mut restored = strategy(json!({"max_hold_observations":3}));
    restored.sync_position("A", 0, 0);
    restored
        .restore_owned_position_metadata("A", &pending)
        .unwrap();
    assert_eq!(restored.has_held_position("A"), Some(false));
    restored.sync_position("A", 5, 80);
    hold(restored.on_tick("A", 80, 0));
    let confirmed = restored.owned_position_metadata("A").unwrap();
    let mut cold = strategy(json!({"max_hold_observations":3}));
    cold.sync_position("A", 5, 80);
    cold.restore_owned_position_metadata("A", &confirmed)
        .unwrap();
    assert_eq!(cold.states["A"].held_observations, Some(1));
    hold(cold.on_tick("A", 80, 0));
    sell(cold.on_tick("A", 80, 0), 5);
    let mut newly_filled = strategy(json!({"max_hold_observations":3}));
    newly_filled.sync_position("A", 5, 80);
    newly_filled
        .restore_owned_position_metadata("A", &pending)
        .unwrap();
    assert_eq!(newly_filled.states["A"].held_observations, Some(0));
}

#[test]
fn malformed_cross_strategy_and_unowned_confirmed_checkpoints_fail_atomically() {
    let mut s = filled(json!({"max_hold_observations":3}));
    let original = s.owned_position_metadata("A").unwrap();
    for patch in [
        json!({"kind":"deviation"}),
        json!({"version":2}),
        json!({"heldObservations":-1}),
        json!({"ageKnown":false,"heldObservations":1}),
        json!({"candidateEntry":true}),
        json!({"extra":true}),
        json!({"ageKnown":"true"}),
    ] {
        let mut bad = original.clone();
        bad.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(s.restore_owned_position_metadata("A", &bad).is_err());
        assert_eq!(s.owned_position_metadata("A"), Some(original.clone()));
    }
    assert!(s
        .restore_owned_position_metadata("OTHER", &original)
        .is_err());
    let mut flat = strategy(json!({}));
    assert!(flat
        .restore_owned_position_metadata("A", &original)
        .is_err());
    assert_eq!(flat.has_held_position("A"), Some(false));
}

#[test]
fn optional_validation_is_strict_and_runtime_damage_keeps_legacy_custom_exit_and_valid_age() {
    for bad in [
        json!({"exit_mode":null}),
        json!({"exit_mode":"bad"}),
        json!({"long_trend_period":"1"}),
        json!({"long_trend_period":513}),
        json!({"long_trend_period":1.5}),
        json!({"max_hold_observations":-1}),
        json!({"max_hold_observations":1000001}),
        json!({"max_hold_observations":null}),
    ] {
        assert!(validate_mean_reversion_params("mean_reversion_fixture", &bad).is_err());
        assert!(validate_mean_reversion_params("trend_filter", &bad).is_ok());
        let mut s = strategy(bad.clone());
        hold(s.on_tick("A", 80, 0));
    }
    let mut s = strategy(
        json!({"exit_mode":"bad","period":7,"std_dev":4.0,"stop_loss_pct":3.0,"max_hold_observations":4}),
    );
    assert_eq!(s.params.period, 7);
    assert_eq!(s.params.std_dev, 4.0);
    assert_eq!(s.params.stop_loss_pct, 3.0);
    assert_eq!(s.params.max_hold_observations, 4);
    s.sync_position("A", 5, 100);
    sell(s.on_tick("A", 97, 0), 2);
    sell(s.on_tick("A", 100, 0), 5); // valid age protection survives invalid exit mode
    let p = params::parse(
        &json!({"period":3,"std_dev":1.0,"stop_loss_pct":7,"long_trend_period":512.0,"max_hold_observations":1000000.0}),
    );
    assert_eq!(p.long_trend_period, 512);
    assert_eq!(p.max_hold_observations, 1000000);
    let legacy: MeanReversionParams =
        serde_json::from_value(json!({"period":3,"std_dev":1.0,"stop_loss_pct":7})).unwrap();
    assert_eq!(legacy.exit_mode, MeanReversionExitMode::UpperBand);
}

#[test]
fn disabled_unknown_symbol_and_reset_preserve_observation_contract() {
    let mut s = filled(json!({}));
    let prices = s.states["A"].prices.clone();
    s.set_enabled(false);
    hold(s.on_live_tick("A", 1, 0));
    assert_eq!(s.states["A"].prices, prices);
    s.set_enabled(true);
    hold(s.on_tick("OTHER", 1, 0));
    s.sync_position("OTHER", 5, 1);
    assert!(!s.states.contains_key("OTHER"));
    s.reset();
    assert_eq!(s.states["A"].prices, prices);
    assert_eq!(s.has_held_position("A"), Some(false));
    assert_eq!(s.states["A"].entry_price, None);
}
