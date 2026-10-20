use super::*;
use serde_json::{json, Value};

fn config(extra: Value) -> StrategyConfig {
    let mut params = json!({"ma_period":2,"buy_threshold_pct":-5.0,"sell_threshold_pct":5.0});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    StrategyConfig::new(
        "deviation_fixture",
        "fixture",
        true,
        vec!["A".into(), "B".into()],
        2,
        params,
    )
}
fn strategy(extra: Value) -> DeviationStrategy {
    let mut s = DeviationStrategy::new(config(extra));
    s.initialize_historical("A", &[100, 100]);
    s
}
fn buy(signal: Signal) {
    assert!(matches!(signal, Signal::Buy { .. }), "{signal:?}");
}
fn sell(signal: Signal, q: u64) {
    assert!(
        matches!(signal,Signal::Sell{quantity,..} if quantity==q),
        "{signal:?}"
    );
}
fn hold(signal: Signal) {
    assert_eq!(signal, Signal::Hold);
}
fn filled(extra: Value) -> DeviationStrategy {
    let mut s = strategy(extra);
    buy(s.on_tick("A", 80, 0));
    s.sync_position("A", 5, 80);
    s
}

#[test]
fn raw_and_live_candidates_never_manufacture_ownership_or_average() {
    for live in [false, true] {
        let mut s = strategy(json!({}));
        buy(if live {
            s.on_live_tick("A", 80, 0)
        } else {
            s.on_tick("A", 80, 0)
        });
        assert_eq!(s.has_held_position("A"), Some(false));
        assert_eq!(s.states["A"].avg_price, 0);
        hold(s.on_tick("A", 120, 0)); // high deviation while flat cannot sell
        s.sync_owned_position("A", 5, 105);
        hold(s.on_tick("A", 80, 0)); // low deviation while held cannot buy
        sell(s.on_live_tick("A", 120, 0), 2);
        assert_eq!(s.states["A"].owned_quantity, 5);
        assert_eq!(s.states["A"].avg_price, 105);
    }
}

#[test]
fn level_retains_inclusive_boundaries_and_current_inclusive_ma() {
    let mut s = strategy(json!({"buy_threshold_pct":0.0,"sell_threshold_pct":0.0}));
    buy(s.on_tick("A", 100, 0));
    s.sync_position("A", 7, 100);
    sell(s.on_tick("A", 100, 0), 2);
    s.sync_position("A", 0, 0);
    s.initialize_historical("A", &[105]);
    s.params.buy_threshold_pct = -5.0;
    buy(s.on_tick("A", 95, 0)); // mean=(105+95)/2=100
    assert_eq!(s.states["A"].previous_deviation.unwrap().round(), -5.0);
}

#[test]
fn downward_crossing_retries_zero_feedback_and_cancels_above_threshold() {
    let mut s = strategy(json!({"entry_mode":"crossing"}));
    buy(s.on_tick("A", 80, 0));
    s.sync_position("A", 0, 0);
    buy(s.on_live_tick("A", 70, 0));
    s.sync_owned_position("A", 0, 0);
    buy(s.on_tick("A", 60, 0));
    hold(s.on_tick("A", 60, 0));
    assert!(!s.states["A"].retry_candidate);
    buy(s.on_tick("A", 40, 0));
    s.sync_position("A", 1, 40);
    assert!(!s.states["A"].retry_candidate);
    hold(s.on_tick("A", 30, 0));
}

#[test]
fn historical_crossings_seed_baseline_without_retry_or_virtual_age() {
    let mut s = DeviationStrategy::new(config(json!({"entry_mode":"crossing"})));
    s.initialize_historical("A", &[100, 100, 80]);
    assert!(!s.states["A"].retry_candidate);
    assert!(!s.states["A"].entry_candidate);
    hold(s.on_tick("A", 70, 0));
    s.sync_position("A", 1, 70);
    assert_eq!(s.states["A"].held_observations, None);
}

#[test]
fn crossing_at_boundary_requires_previous_strictly_above() {
    let mut s = strategy(json!({"entry_mode":"crossing","buy_threshold_pct":0.0}));
    hold(s.on_tick("A", 100, 0)); // equal -> equal is no crossing
    hold(s.on_tick("A", 110, 0));
    buy(s.on_tick("A", 110, 0)); // positive -> equal
}

#[test]
fn high_deviation_exit_retries_rejections_and_partial_fills() {
    let mut s = strategy(json!({"entry_mode":"crossing"}));
    s.sync_position("A", 5, 100);
    sell(s.on_tick("A", 120, 0), 2);
    s.sync_position("A", 5, 100);
    sell(s.on_tick("A", 140, 0), 2);
    s.sync_position("A", 1, 100);
    sell(s.on_tick("A", 160, 0), 2); // manager clamps legacy configured quantity
    s.sync_position("A", 0, 0);
    hold(s.on_tick("A", 180, 0));
}

#[test]
fn trend_uses_only_prior_observations_and_strict_price_above_mean() {
    let mut s = DeviationStrategy::new(config(json!({"long_trend_period":3})));
    s.initialize_historical("A", &[50, 50, 100]);
    buy(s.on_tick("A", 80, 0)); // prior mean 66.67, current-inclusive mean would differ
    let mut s = DeviationStrategy::new(config(json!({"long_trend_period":2})));
    s.initialize_historical("A", &[50, 100]);
    hold(s.on_tick("A", 75, 0)); // equal prior MA75 is rejected
    let mut s = DeviationStrategy::new(config(json!({"long_trend_period":3})));
    s.initialize_historical("A", &[100, 100]);
    hold(s.on_tick("A", 80, 0)); // insufficient prior trend history
    s.sync_position("A", 3, 100);
    sell(s.on_tick("A", 120, 0), 2); // trend gate never blocks high-deviation exit
}

#[test]
fn live_trend_rejects_daily_minute_seed_and_survives_reconfiguration_and_warmup() {
    let mut s = DeviationStrategy::new(config(json!({"long_trend_period":3})));
    s.initialize_historical("A", &vec![20; 300]);
    s.initialize_intraday_prices("A", &[30, 40, 100]);
    hold(s.on_live_tick("A", 80, 0));
    assert_eq!(
        s.states["A"]
            .trend_prices
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![80]
    );
    assert!(!s.history_readiness("A").unwrap().ready);
    s.on_live_tick("A", 50, 0);
    s.on_live_tick("A", 100, 0);
    let before = s.states["A"].trend_prices.clone();
    s.initialize_historical("A", &vec![1; 500]);
    assert_eq!(s.states["A"].trend_prices, before);
    let mut next = s.config.clone();
    next.params["long_trend_period"] = json!(2);
    assert!(s.reconfigure_preserving_position(next));
    assert_eq!(s.states["A"].trend_prices, before);
    s.initialize_historical("A", &[100]);
    buy(s.on_live_tick("A", 80, 0)); // live prior (50+100)/2=75
}

#[test]
fn trend_tracks_live_history_while_option_off_and_is_bounded() {
    let mut s = strategy(json!({}));
    for _ in 0..1000 {
        s.on_live_tick("A", 100, 0);
    }
    assert_eq!(s.states["A"].trend_prices.len(), 512);
    let mut next = s.config.clone();
    next.params["long_trend_period"] = json!(512.0);
    s.reconfigure_preserving_position(next);
    assert_eq!(s.states["A"].trend_prices.len(), 512);
    assert_eq!(s.params.long_trend_period, 512);
    // A newly enabled long trend uses the independently preserved live history;
    // the legacy MA's shorter buffer must not keep the diagnostic unready.
    assert!(s.history_readiness("A").unwrap().ready);
    assert_eq!(s.history_readiness("A").unwrap().available_bars, 513);
    assert_eq!(s.can_evaluate_next_tick("A"), Some(true));
}

#[test]
fn age_starts_at_actual_fill_and_counts_only_owned_evaluations() {
    let mut s = strategy(json!({"max_hold_observations":3}));
    buy(s.on_tick("A", 80, 0));
    s.sync_position("A", 0, 0);
    buy(s.on_tick("A", 70, 0));
    assert_eq!(s.states["A"].held_observations, None);
    s.sync_position("A", 5, 70);
    assert_eq!(s.states["A"].held_observations, Some(0));
    hold(s.on_tick("A", 70, 0));
    s.sync_position("A", 5, 70);
    hold(s.on_live_tick("A", 70, 0));
    s.sync_position("A", 2, 70);
    sell(s.on_tick("A", 70, 0), 2);
    assert_eq!(s.states["A"].held_observations, Some(3));
    s.sync_position("A", 1, 70);
    sell(s.on_tick("A", 70, 0), 1);
    assert_eq!(s.states["A"].held_observations, Some(4));
}

#[test]
fn unknown_restored_age_exits_conservatively_when_enabled() {
    let mut s = strategy(json!({"max_hold_observations":63}));
    s.sync_owned_position("A", 5, 100);
    assert_eq!(s.states["A"].held_observations, None);
    match s.on_tick("A", 100, 0) {
        Signal::Sell {
            quantity, reason, ..
        } => {
            assert_eq!(quantity, 5);
            assert!(reason.contains("불명"));
        }
        v => panic!("{v:?}"),
    }
    s.sync_position("A", 2, 100);
    sell(s.on_tick("A", 100, 0), 2);
}

#[test]
fn age_keeps_accumulating_when_option_off_and_survives_warmup_parameter_edits() {
    let mut s = filled(json!({}));
    for _ in 0..5 {
        hold(s.on_tick("A", 80, 0));
    }
    s.initialize_historical("A", &vec![80; 1000]);
    assert_eq!(s.states["A"].held_observations, Some(5));
    let mut next = s.config.clone();
    next.params["ma_period"] = json!(20);
    next.params["max_hold_observations"] = json!(3);
    assert!(s.reconfigure_preserving_position(next));
    assert_eq!(s.states["A"].held_observations, Some(5));
    sell(s.on_tick("A", 80, 0), 5);
}

#[test]
fn stop_precedes_age_and_high_exit_before_indicator_readiness() {
    let mut s = DeviationStrategy::new(config(
        json!({"ma_period":512,"stop_loss_pct":10.0,"max_hold_observations":1,"sell_threshold_pct":-100.0,"long_trend_period":512}),
    ));
    s.sync_position("A", 7, 100);
    assert_eq!(s.can_evaluate_next_tick("A"), Some(true));
    let signal = s.on_live_tick("A", 90, 0);
    match signal {
        Signal::Sell {
            quantity, reason, ..
        } => {
            assert_eq!(quantity, 7);
            assert!(reason.contains("손절"));
        }
        v => panic!("{v:?}"),
    }
    s.sync_position("A", 3, 200);
    sell(s.on_tick("A", 180, 0), 3); // latest actual snapshot average
    assert_eq!(s.states["A"].avg_price, 200);
}

#[test]
fn stop_zero_is_off_and_boundary_hundred_handles_zero_price() {
    let mut s = strategy(json!({"stop_loss_pct":0.0}));
    s.sync_position("A", 1, 100);
    hold(s.on_tick("A", 0, 0));
    let mut s = strategy(json!({"stop_loss_pct":100.0}));
    s.sync_position("A", 3, 100);
    sell(s.on_tick("A", 0, 0), 3);
    s.sync_position("A", 3, 0);
    hold(s.on_tick("A", 0, 0)); // unknown/zero average cannot invent a loss basis
}

#[test]
fn metadata_roundtrips_confirmed_and_pending_ages_without_reconstructing_average() {
    let mut original = filled(json!({"max_hold_observations":3}));
    original.on_tick("A", 80, 0);
    let checkpoint = original.owned_position_metadata("A").unwrap();
    let mut restored = strategy(json!({"max_hold_observations":3}));
    restored.sync_position("A", 2, 81);
    restored
        .restore_owned_position_metadata("A", &checkpoint)
        .unwrap();
    assert_eq!(restored.states["A"].held_observations, Some(1));
    assert_eq!(restored.states["A"].avg_price, 81);
    restored.on_tick("A", 80, 0);
    sell(restored.on_tick("A", 80, 0), 2);
    let mut pending = strategy(json!({}));
    buy(pending.on_tick("A", 80, 0));
    let checkpoint = pending.owned_position_metadata("A").unwrap();
    let mut restored = strategy(json!({}));
    restored.sync_position("A", 0, 0);
    restored
        .restore_owned_position_metadata("A", &checkpoint)
        .unwrap();
    restored.sync_position("A", 1, 80);
    assert_eq!(restored.states["A"].held_observations, Some(0));
    let mut already_filled = strategy(json!({}));
    already_filled.sync_position("A", 1, 80);
    already_filled
        .restore_owned_position_metadata("A", &checkpoint)
        .unwrap();
    assert_eq!(already_filled.states["A"].held_observations, Some(0));
}

#[test]
fn metadata_rejects_wrong_identity_types_unknown_fields_and_impossible_ages_atomically() {
    let mut s = filled(json!({}));
    let good = s.owned_position_metadata("A").unwrap();
    for extra in [
        json!({"version":2}),
        json!({"kind":"momentum"}),
        json!({"ageKnown":false,"heldObservations":1}),
        json!({"candidateEntry":true}),
        json!({"heldObservations":-1}),
        json!({"heldObservations":0.5}),
        json!({"ageKnown":null}),
        json!({"other":1}),
    ] {
        let mut bad = good.clone();
        bad.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        assert!(s.restore_owned_position_metadata("A", &bad).is_err());
        assert_eq!(s.states["A"].held_observations, Some(0));
    }
    assert!(s.restore_owned_position_metadata("B", &good).is_err());
    let mut flat = strategy(json!({}));
    assert!(flat.restore_owned_position_metadata("A", &good).is_err());
}

#[test]
fn malformed_optional_fields_fail_new_entries_closed_but_preserve_custom_and_valid_exits() {
    let mut s = strategy(
        json!({"entry_mode":"broken","ma_period":2,"buy_threshold_pct":-1.0,"sell_threshold_pct":20.0,"stop_loss_pct":10.0}),
    );
    assert_eq!(s.params.sell_threshold_pct, 20.0);
    hold(s.on_tick("A", 50, 0));
    s.sync_position("A", 3, 100);
    sell(s.on_tick("A", 90, 0), 3);
    let mut s = strategy(json!({"stop_loss_pct":"bad","sell_threshold_pct":1.0}));
    s.sync_position("A", 3, 100);
    sell(s.on_tick("A", 120, 0), 2);
}

#[test]
fn strict_optional_parameter_types_ranges_and_integer_floats() {
    for extra in [
        json!({"entry_mode":null}),
        json!({"entry_mode":1}),
        json!({"entry_mode":"LEVEL"}),
        json!({"long_trend_period":513}),
        json!({"long_trend_period":0.5}),
        json!({"long_trend_period":"20"}),
        json!({"max_hold_observations":1_000_001}),
        json!({"max_hold_observations":-1}),
        json!({"max_hold_observations":2.5}),
        json!({"max_hold_observations":null}),
        json!({"stop_loss_pct":"10"}),
        json!({"stop_loss_pct":101}),
        json!({"stop_loss_pct":-0.1}),
        json!({"stop_loss_pct":null}),
    ] {
        assert!(
            validate_deviation_entry_params("deviation_fixture", &extra).is_err(),
            "{extra}"
        );
    }
    let good = json!({"entry_mode":"crossing","long_trend_period":512.0,"max_hold_observations":1_000_000.0,"stop_loss_pct":100.0});
    assert!(validate_deviation_entry_params("deviation_fixture", &good).is_ok());
    let s = DeviationStrategy::new(config(good));
    assert_eq!(s.params.long_trend_period, 512);
    assert_eq!(s.params.max_hold_observations, 1_000_000);
    assert!(validate_deviation_entry_params("momentum", &json!({"entry_mode":"bad"})).is_ok());
}

#[test]
fn same_params_preserve_pending_crossing_changed_params_reseed_without_age_reset() {
    let mut s = strategy(json!({"entry_mode":"crossing"}));
    buy(s.on_tick("A", 80, 0));
    assert!(s.reconfigure_preserving_position(s.config.clone()));
    assert!(s.states["A"].retry_candidate);
    buy(s.on_tick("A", 70, 0));
    s.sync_position("A", 5, 70);
    s.on_tick("A", 70, 0);
    let mut next = s.config.clone();
    next.params["ma_period"] = json!(3);
    next.params["long_trend_period"] = json!(10);
    next.order_quantity = 7;
    assert!(s.reconfigure_preserving_position(next));
    assert_eq!(s.states["A"].held_observations, Some(1));
    assert_eq!(s.states["A"].owned_quantity, 5);
    assert_eq!(s.states["A"].avg_price, 70);
    assert!(!s.states["A"].retry_candidate);
    let before = s.config.clone();
    let mut wrong = before.clone();
    wrong.broker_account_id = Some("other".into());
    assert!(!s.reconfigure_preserving_position(wrong));
    assert_eq!(s.config.params, before.params);
}

#[test]
fn zero_ma_overflow_and_buffer_bounds_are_safe() {
    let mut s = DeviationStrategy::new(config(
        json!({"ma_period":usize::MAX,"long_trend_period":512}),
    ));
    s.initialize_historical("A", &vec![u64::MAX; 10000]);
    assert_eq!(s.states["A"].prices.len(), 513);
    assert_eq!(s.states["A"].trend_prices.len(), 512);
    hold(s.on_tick("A", u64::MAX, 0));
    s.initialize_historical("A", &vec![0; 10000]);
    hold(s.on_tick("A", 0, 0));
    assert!(!s.history_readiness("A").unwrap().ready);
}

#[test]
fn disabled_unknown_and_other_symbol_observations_cannot_change_held_age() {
    let mut s = filled(json!({"max_hold_observations":2}));
    s.set_enabled(false);
    hold(s.on_live_tick("A", 0, 0));
    assert_eq!(s.states["A"].held_observations, Some(0));
    assert!(!s.states["A"].live_started);
    s.set_enabled(true);
    hold(s.on_live_tick("UNKNOWN", 0, 0));
    s.sync_position("UNKNOWN", 3, 100);
    assert!(!s.states.contains_key("UNKNOWN"));
    s.initialize_historical("B", &[100, 100]);
    buy(s.on_tick("B", 80, 0));
    assert_eq!(s.states["A"].held_observations, Some(0));
    s.on_tick("A", 80, 0);
    assert_eq!(s.states["A"].held_observations, Some(1));
    assert_eq!(s.states["B"].held_observations, None);
}

#[test]
fn stop_inclusive_integer_percentage_boundary_avoids_division_rounding() {
    let mut s = strategy(json!({"stop_loss_pct":7.0,"ma_period":512}));
    s.sync_position("A", 3, 1000);
    hold(s.on_tick("A", 931, 0));
    sell(s.on_tick("A", 930, 0), 3);
}
