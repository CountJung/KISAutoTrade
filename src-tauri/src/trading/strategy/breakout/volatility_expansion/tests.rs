use super::*;
use serde_json::{json, Value};
fn config(extra: Value) -> StrategyConfig {
    let mut p = json!({"lookback_days":2,"expansion_factor":2,"stop_loss_pct":3});
    p.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    StrategyConfig::new(
        "volatility_expansion_fixture",
        "fixture",
        true,
        vec!["TEST".into()],
        10,
        p,
    )
}
fn strategy(extra: Value) -> VolatilityExpansionStrategy {
    let mut s = VolatilityExpansionStrategy::new(config(extra));
    s.initialize_range_data("TEST", &[10, 10]);
    s.on_trading_day_start("TEST", 100);
    s
}
fn bar(open: u64, high: u64, low: u64, close: u64) -> OhlcCandle {
    OhlcCandle {
        open,
        high,
        low,
        close,
    }
}
fn daily(b: OhlcCandle) -> CompletedDailyBar {
    CompletedDailyBar {
        ohlc: b,
        volume: None,
    }
}
fn buy(s: &mut VolatilityExpansionStrategy) {
    assert!(matches!(
        s.on_tick("TEST", 125, 0),
        Signal::Buy { quantity: 10, .. }
    ));
}
fn sell(s: &mut VolatilityExpansionStrategy, price: u64, q: u64, reason: &str) {
    match s.on_tick("TEST", price, 0) {
        Signal::Sell {
            quantity,
            reason: r,
            ..
        } => {
            assert_eq!(quantity, q);
            assert!(r.contains(reason), "{r}");
        }
        x => panic!("{x:?}"),
    }
}
#[test]
fn volatility_expansion_defaults_and_raw_validation_are_individual_and_strict() {
    assert_eq!(params::parse(&json!({})).lookback_days, 10);
    for v in [
        json!({"lookback_days":null}),
        json!({"lookback_days":0}),
        json!({"lookback_days":1.5}),
        json!({"lookback_days":513}),
        json!({"expansion_factor":-1}),
        json!({"expansion_factor":1001}),
        json!({"stop_loss_pct":100}),
        json!({"stop_loss_pct":"3"}),
        json!({"profit_activation_pct":5}),
        json!({"profit_trailing_pct":3}),
        json!({"profit_trailing_pct":true}),
        json!({"max_hold_observations":0.5}),
        json!({"max_hold_observations":1000001}),
        json!({"max_hold_observations":null}),
        json!(null),
    ] {
        assert!(
            validate_volatility_expansion_params("volatility_expansion_test", &v).is_err(),
            "{v}"
        );
        assert!(validate_volatility_expansion_params("strong_close", &v).is_ok());
    }
    for v in [
        json!({}),
        json!({"lookback_days":512.0,"expansion_factor":0,"stop_loss_pct":0}),
        json!({"profit_activation_pct":1000,"profit_trailing_pct":99.9,"max_hold_observations":1000000.0}),
    ] {
        assert!(
            validate_volatility_expansion_params("volatility_expansion", &v).is_ok(),
            "{v}"
        );
    }
    let p = params::parse(
        &json!({"lookback_days":20,"expansion_factor":1.5,"stop_loss_pct":9,"profit_activation_pct":"bad"}),
    );
    assert_eq!(
        (p.lookback_days, p.expansion_factor, p.stop_loss_pct),
        (20, 1.5, 9.0)
    );
}
#[test]
fn volatility_expansion_actual_fill_only_and_remaining_sell_reoffers() {
    let mut s = strategy(json!({}));
    buy(&mut s);
    assert_eq!(s.has_held_position("TEST"), Some(false));
    s.sync_position("TEST", 0, 0);
    buy(&mut s);
    s.sync_position("TEST", 7, 120);
    assert_eq!(s.states["TEST"].held_observations, Some(0));
    sell(&mut s, 110, 7, "손절");
    sell(&mut s, 110, 7, "손절");
    s.sync_position("TEST", 3, 120);
    sell(&mut s, 110, 3, "손절");
    assert_eq!(s.states["TEST"].avg_price, 120);
    s.sync_position("TEST", 0, 0);
    assert_eq!(s.states["TEST"].held_observations, None);
    assert!(matches!(s.on_tick("TEST", 110, 0), Signal::Buy { .. }));
}
#[test]
fn volatility_expansion_strict_range_direction_zero_and_overflow_safe_mean() {
    let mut s = strategy(json!({}));
    assert!(matches!(s.on_tick("TEST", 120, 0), Signal::Hold));
    buy(&mut s);
    s.on_trading_day_start("TEST", 150);
    assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Hold));
    assert!(matches!(s.on_tick("TEST", 150, 0), Signal::Hold));
    assert!(matches!(s.on_tick("TEST", 151, 0), Signal::Buy { .. }));
    s.initialize_range_data("TEST", &[0, 0]);
    s.on_trading_day_start("TEST", 100);
    assert!(s.history_readiness("TEST").unwrap().ready);
    assert!(matches!(s.on_tick("TEST", 101, 0), Signal::Buy { .. }));
    s.initialize_range_data("TEST", &[u64::MAX; 1024]);
    assert_eq!(s.states["TEST"].completed_ranges.len(), 512);
    assert_eq!(s.states["TEST"].avg_range, Some(u64::MAX as f64));
}
#[test]
fn volatility_expansion_current_range_published_only_after_decision() {
    let mut s = strategy(json!({}));
    let b = bar(100, 130, 100, 125);
    assert!(matches!(
        s.on_daily_close_tick("TEST", &b, 0),
        Signal::Buy { .. }
    ));
    assert_eq!(s.states["TEST"].avg_range, Some(10.0));
    s.on_completed_candle("TEST", &b);
    assert_eq!(s.states["TEST"].avg_range, Some(20.0));
}
#[test]
fn volatility_expansion_diagnostics_partition_every_valid_observation_without_overlap() {
    let mut s = VolatilityExpansionStrategy::new(config(json!({})));
    assert_eq!(
        s.volatility_expansion_preparation("TEST").unwrap().status,
        "unsupportedObservation"
    );
    s.on_tick("TEST", 100, 0);
    s.initialize_range_data("TEST", &[10, 10]);
    s.on_tick("TEST", 110, 0);
    s.on_tick("TEST", 80, 0);
    s.on_tick("TEST", 125, 0);
    s.sync_position("TEST", 5, 125);
    s.on_tick("TEST", 124, 0);
    let p = s.volatility_expansion_preparation("TEST").unwrap();
    assert_eq!(
        (
            p.evaluated_observations,
            p.owned_observations,
            p.candidate_observations
        ),
        (5, 1, 4)
    );
    assert_eq!(
        (
            p.unready_candidates,
            p.range_rejections,
            p.direction_rejections,
            p.passed_candidates
        ),
        (1, 1, 1, 1)
    );
    s.on_tick("OTHER", 150, 0);
    s.on_tick("TEST", 0, 0);
    s.set_enabled(false);
    s.on_tick("TEST", 150, 0);
    assert_eq!(
        s.volatility_expansion_preparation("TEST")
            .unwrap()
            .evaluated_observations,
        5
    );
}
#[test]
fn volatility_expansion_age_off_then_on_same_partial_reconfigure_and_unknown() {
    let mut s = strategy(json!({}));
    buy(&mut s);
    s.sync_position("TEST", 7, 125);
    s.on_tick("TEST", 125, 0);
    s.sync_position("TEST", 7, 125);
    s.on_tick("TEST", 125, 0);
    s.sync_position("TEST", 3, 125);
    let mut c = s.config().clone();
    c.params["max_hold_observations"] = json!(3);
    assert!(s.reconfigure_preserving_position(c));
    sell(&mut s, 125, 3, "보유 관측");
}
#[test]
fn volatility_expansion_unknown_age_never_becomes_zero_even_during_physical_pending() {
    let mut s = strategy(json!({"max_hold_observations":3}));
    s.set_order_pending("TEST", true);
    buy(&mut s);
    s.sync_position("TEST", 7, 125);
    assert_eq!(s.states["TEST"].held_observations, None);
    sell(&mut s, 125, 7, "보유 관측");
    assert_eq!(s.states["TEST"].held_observations, None);
}
#[test]
fn volatility_expansion_existing_pending_entry_proof_survives_failure_and_reconfigure() {
    let mut s = strategy(json!({}));
    buy(&mut s);
    s.set_order_pending("TEST", true);
    let proof = s.owned_position_metadata("TEST").unwrap();
    s.invalidate_completed_daily_history("TEST");
    s.on_tick("TEST", 130, 0);
    assert!(s.reconfigure_preserving_position(s.config().clone()));
    assert_eq!(s.owned_position_metadata("TEST"), Some(proof));
    s.sync_position("TEST", 5, 130);
    assert_eq!(s.states["TEST"].held_observations, Some(0));
}
#[test]
fn volatility_expansion_unsubmitted_proof_discarded_and_unknown_restored_is_conservative() {
    let mut s = strategy(json!({"max_hold_observations":3}));
    buy(&mut s);
    s.discard_unsubmitted_signal("TEST");
    s.sync_position("TEST", 5, 125);
    assert_eq!(s.states["TEST"].held_observations, None);
    sell(&mut s, 125, 5, "보유 관측");
}
#[test]
fn volatility_expansion_profit_only_owned_quotes_not_daily_high_and_exit_priority() {
    let mut s = strategy(
        json!({"profit_activation_pct":5,"profit_trailing_pct":3,"max_hold_observations":3}),
    );
    let b = bar(100, 10000, 90, 125);
    assert!(matches!(
        s.on_daily_close_tick("TEST", &b, 0),
        Signal::Buy { .. }
    ));
    s.sync_position("TEST", 7, 120);
    assert!(s.states["TEST"].observed_peak.is_none());
    s.on_tick("TEST", 125, 0);
    assert!(!s.states["TEST"].profit_armed);
    s.on_tick("TEST", 130, 0);
    assert!(s.states["TEST"].profit_armed);
    sell(&mut s, 125, 7, "수익 보호");
    s.sync_position("TEST", 3, 120);
    sell(&mut s, 125, 3, "수익 보호");
    sell(&mut s, 110, 3, "손절");
}
#[test]
fn volatility_expansion_stop_zero_is_active_and_bad_optional_preserves_custom_stop() {
    let mut s = strategy(json!({"stop_loss_pct":0}));
    s.sync_position("TEST", 7, 125);
    sell(&mut s, 125, 7, "손절");
    let mut s = strategy(json!({"stop_loss_pct":10,"profit_activation_pct":"bad"}));
    buy_blocked(&mut s);
    s.sync_position("TEST", 7, 125);
    assert!(matches!(s.on_tick("TEST", 120, 0), Signal::Hold));
    sell(&mut s, 110, 7, "손절");
}
fn buy_blocked(s: &mut VolatilityExpansionStrategy) {
    assert!(matches!(s.on_tick("TEST", 125, 0), Signal::Hold));
    assert_eq!(
        s.volatility_expansion_preparation("TEST").unwrap().status,
        "invalidParams"
    );
}
#[test]
fn volatility_expansion_strict_checkpoint_atomic_cross_kind_and_positive_restore() {
    let mut s = strategy(json!({"profit_activation_pct":5,"profit_trailing_pct":3}));
    buy(&mut s);
    let proof = s.owned_position_metadata("TEST").unwrap();
    s.sync_position("TEST", 7, 120);
    s.on_tick("TEST", 130, 0);
    let saved = s.owned_position_metadata("TEST").unwrap();
    for change in [
        json!({"kind":"strongClose"}),
        json!({"version":2}),
        json!({"extra":1}),
        json!({"ageKnown":false}),
        json!({"candidateEntry":true}),
        json!({"observedPeak":0}),
        json!({"observedPeak":null}),
    ] {
        let mut bad = saved.clone();
        bad.as_object_mut()
            .unwrap()
            .extend(change.as_object().unwrap().clone());
        assert!(
            s.restore_owned_position_metadata("TEST", &bad).is_err(),
            "{bad}"
        );
        assert_eq!(s.owned_position_metadata("TEST"), Some(saved.clone()));
    }
    for key in saved.as_object().unwrap().keys() {
        let mut bad = saved.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(s.restore_owned_position_metadata("TEST", &bad).is_err());
    }
    let mut restored = strategy(json!({}));
    restored.sync_position("TEST", 3, 120);
    restored
        .restore_owned_position_metadata("TEST", &saved)
        .unwrap();
    assert_eq!(restored.owned_position_metadata("TEST"), Some(saved));
    restored
        .restore_owned_position_metadata("TEST", &proof)
        .unwrap();
    assert_eq!(restored.states["TEST"].held_observations, Some(0));
}
#[test]
fn volatility_expansion_market_day_same_refresh_failure_and_legacy_preserve_observations() {
    let mut s = strategy(json!({}));
    let bars = [daily(bar(100, 110, 100, 105)); 2];
    s.apply_completed_daily_bars_for_market_day("TEST", "2026-10-22", &bars);
    assert!(s.states["TEST"].day_open.is_none());
    assert!(matches!(s.on_live_tick("TEST", 100, 0), Signal::Hold));
    assert!(matches!(s.on_live_tick("TEST", 125, 0), Signal::Buy { .. }));
    s.apply_completed_daily_bars_for_market_day("TEST", "2026-10-22", &bars);
    assert_eq!(
        (s.states["TEST"].day_open, s.states["TEST"].day_high),
        (Some(100), 125)
    );
    s.invalidate_completed_daily_history("TEST");
    assert!(matches!(s.on_live_tick("TEST", 150, 0), Signal::Hold));
    s.initialize_range_data("TEST", &[0, 0]);
    assert!(!s.history_readiness("TEST").unwrap().ready);
    s.apply_completed_daily_bars_for_market_day("TEST", "2026-10-22", &bars);
    assert_eq!(
        (s.states["TEST"].day_open, s.states["TEST"].day_high),
        (Some(100), 125)
    );
    s.apply_completed_daily_bars_for_market_day("TEST", "2026-10-23", &bars);
    assert!(s.states["TEST"].day_open.is_none());
    s.on_live_tick("TEST", 200, 0);
    assert_eq!(s.states["TEST"].day_open, Some(200));
}
#[test]
fn volatility_expansion_increased_period_does_not_pad_missing_context() {
    let mut s = strategy(json!({}));
    let mut c = s.config().clone();
    c.params["lookback_days"] = json!(20);
    assert!(s.reconfigure_preserving_position(c));
    let r = s.history_readiness("TEST").unwrap();
    assert_eq!((r.required_bars, r.available_bars, r.ready), (20, 2, false));
    s.initialize_range_data("TEST", &[1; 512]);
    assert!(s.history_readiness("TEST").unwrap().ready);
    s.initialize_daily_bars("TEST", &[daily(bar(100, 90, 100, 100))]);
    assert!(!s.history_readiness("TEST").unwrap().ready);
}
#[test]
fn volatility_expansion_identity_and_target_removal_cannot_drop_owned_protection() {
    let mut s = strategy(json!({}));
    s.sync_position("TEST", 5, 125);
    for field in ["id", "broker", "account", "target"] {
        let mut c = s.config().clone();
        match field {
            "id" => c.id.push_str("_other"),
            "broker" => c.broker_id = crate::broker::BrokerId::Toss,
            "account" => c.broker_account_id = Some("other".into()),
            _ => c.target_symbols = vec!["OTHER".into()],
        };
        assert!(!s.reconfigure_preserving_position(c));
    }
    assert!(!s.states.contains_key("OTHER"));
    sell(&mut s, 100, 5, "손절");
}

#[test]
fn volatility_expansion_reset_preserves_completed_ranges_but_clears_owned_day_and_checkpoint() {
    let mut s = strategy(json!({"profit_activation_pct":5,"profit_trailing_pct":3}));
    buy(&mut s);
    s.sync_position("TEST", 5, 120);
    s.on_tick("TEST", 130, 0);
    let ranges = s.states["TEST"].completed_ranges.clone();
    let avg = s.states["TEST"].avg_range;
    s.reset();
    let state = &s.states["TEST"];
    assert_eq!(state.completed_ranges, ranges);
    assert_eq!(state.avg_range, avg);
    assert_eq!(state.owned_quantity, 0);
    assert!(state.day_open.is_none());
    assert!(state.market_day.is_none());
    assert_eq!(s.owned_position_metadata("TEST"), None);
    assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Hold));
    buy(&mut s);
}

#[test]
fn volatility_expansion_new_day_and_warmup_preserve_owned_age_peak_and_armed_exit() {
    let mut s = strategy(json!({"profit_activation_pct":5,"profit_trailing_pct":3}));
    buy(&mut s);
    s.sync_position("TEST", 5, 120);
    s.on_tick("TEST", 130, 0);
    let saved = s.owned_position_metadata("TEST").unwrap();
    let bars = [daily(bar(100, 110, 100, 105)); 2];
    s.apply_completed_daily_bars_for_market_day("TEST", "2026-10-22", &bars);
    assert_eq!(s.owned_position_metadata("TEST"), Some(saved.clone()));
    s.apply_completed_daily_bars_for_market_day("TEST", "2026-10-23", &bars);
    s.initialize_range_data("TEST", &[999, 999]);
    s.initialize_daily_bars("TEST", &[]);
    assert_eq!(s.owned_position_metadata("TEST"), Some(saved));
    sell(&mut s, 125, 5, "수익 보호");
}

#[test]
fn volatility_expansion_context_remains_bounded_after_many_completed_days() {
    let mut s = strategy(json!({"lookback_days":512}));
    for i in 1..=1100 {
        s.on_completed_candle("TEST", &bar(100, 100 + i, 100, 100));
    }
    assert_eq!(s.states["TEST"].completed_ranges.len(), 512);
    assert_eq!(s.states["TEST"].avg_range, Some((589.0 + 1100.0) / 2.0));
}
