use super::*;
use serde_json::json;
fn strategy(params: serde_json::Value) -> StrongCloseStrategy {
    StrongCloseStrategy::new(StrategyConfig::new(
        "strong_close",
        "test",
        true,
        vec!["TEST".into()],
        100,
        params,
    ))
}
fn bar(close: u64) -> CompletedDailyBar {
    CompletedDailyBar {
        ohlc: OhlcCandle {
            open: close,
            high: close,
            low: close,
            close,
        },
        volume: None,
    }
}
fn seed(s: &mut StrongCloseStrategy) {
    s.initialize_candles("TEST", &[(100, 99)]);
}
fn fill(s: &mut StrongCloseStrategy, avg: u64) {
    seed(s);
    assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Buy { .. }));
    s.sync_position("TEST", 7, avg);
}
#[test]
fn strong_close_actual_quantity_average_partial_stop_zero_and_retry() {
    let mut s = strategy(json!({"threshold_pct":3,"stop_loss_pct":0}));
    seed(&mut s);
    for _ in 0..3 {
        assert!(matches!(
            s.on_live_tick("TEST", 100, 0),
            Signal::Buy { quantity: 100, .. }
        ));
        s.sync_position("TEST", 0, 0);
    }
    s.sync_position("TEST", 7, 110);
    assert!(matches!(
        s.on_tick("TEST", 110, 0),
        Signal::Sell { quantity: 7, .. }
    ));
    s.sync_position("TEST", 3, 110);
    assert!(matches!(
        s.on_tick("TEST", 109, 0),
        Signal::Sell { quantity: 3, .. }
    ));
    assert_eq!(s.states["TEST"].owned_quantity, 3);
    s.sync_position("TEST", 0, 0);
    assert!(matches!(s.on_tick("TEST", 109, 0), Signal::Hold));
}
#[test]
fn strong_close_legacy_high_close_boundaries_and_flat() {
    for (pair, buy, ready) in [
        ((100, 97), true, true),
        ((100, 96), false, true),
        ((100, 100), true, true),
        ((0, 0), false, false),
        ((100, 101), false, false),
    ] {
        let mut s = strategy(json!({}));
        s.initialize_candles("TEST", &[pair]);
        assert_eq!(s.history_readiness("TEST").unwrap().ready, ready);
        assert_eq!(matches!(s.on_tick("TEST", 100, 0), Signal::Buy { .. }), buy);
    }
}
#[test]
fn strong_close_age_unknown_disabled_partial_and_protection_priority() {
    let mut s = strategy(
        json!({"max_hold_observations":3,"profit_activation_pct":5,"profit_trailing_pct":3}),
    );
    fill(&mut s, 100);
    s.set_enabled(false);
    s.on_tick("TEST", 110, 0);
    s.sync_position("TEST", 4, 100);
    s.set_enabled(true);
    assert_eq!(s.states["TEST"].held_observations, Some(1));
    s.on_tick("TEST", 110, 0);
    assert!(
        matches!(s.on_tick("TEST",105,0),Signal::Sell{reason,..}if reason.contains("수익 보호"))
    );
    assert!(matches!(s.on_tick("TEST",96,0),Signal::Sell{reason,..}if reason.contains("손절")));
    s.sync_position("TEST", 0, 0);
    s.sync_position("TEST", 9, 100);
    assert_eq!(s.states["TEST"].held_observations, None);
    assert!(
        matches!(s.on_tick("TEST",101,0),Signal::Sell{reason,quantity:9,..}if reason.contains("관측"))
    );
}
#[test]
fn strong_close_long200_prior_completed_only_bounded_current_exclusion() {
    let mut s = strategy(json!({"long_trend_period":200}));
    s.initialize_daily_bars("TEST", &vec![bar(100); 199]);
    assert!(matches!(s.on_tick("TEST", 1000, 0), Signal::Hold));
    s.on_completed_daily_bar("TEST", &bar(100));
    assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Hold));
    assert!(matches!(s.on_tick("TEST", 101, 0), Signal::Buy { .. }));
    s.initialize_daily_bars("TEST", &vec![bar(100); 900]);
    assert_eq!(s.states["TEST"].context.len(), 512);
    let p = s.entry_filter_preparation("TEST").unwrap();
    assert_eq!(p.cadence, "priorCompletedDailyOhlc");
}
#[test]
fn strong_close_dated_same_day_failure_late_legacy_and_new_day() {
    let mut s = strategy(json!({}));
    s.apply_completed_daily_bars_for_market_day("TEST", "20261021", &[bar(100)]);
    assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Buy { .. }));
    s.sync_position("TEST", 2, 100);
    s.sync_position("TEST", 0, 0);
    s.apply_completed_daily_bars_for_market_day("TEST", "20261021", &[bar(101)]);
    assert!(matches!(s.on_tick("TEST", 101, 0), Signal::Hold));
    s.invalidate_completed_daily_history("TEST");
    s.initialize_candles("TEST", &[(100, 100)]);
    s.initialize_daily_bars("TEST", &[bar(100)]);
    assert!(!s.history_readiness("TEST").unwrap().ready);
    assert_eq!(s.states["TEST"].market_day.as_deref(), Some("20261021"));
    s.apply_completed_daily_bars_for_market_day("TEST", "20261022", &[bar(100)]);
    assert!(matches!(s.on_tick("TEST", 101, 0), Signal::Buy { .. }));
    s.set_order_pending("TEST", true);
    s.invalidate_completed_daily_history("TEST");
    s.apply_completed_daily_bars_for_market_day("TEST", "20261022", &[bar(101)]);
    assert!(matches!(s.on_tick("TEST", 101, 0), Signal::Buy { .. }));
}
#[test]
fn strong_close_peak_not_seeded_from_preentry_or_bar_high_and_reconfigure_preserves() {
    let mut s = strategy(json!({"profit_activation_pct":5,"profit_trailing_pct":3}));
    s.initialize_daily_bars(
        "TEST",
        &[CompletedDailyBar {
            ohlc: OhlcCandle {
                open: 100,
                high: 10000,
                low: 100,
                close: 9999,
            },
            volume: None,
        }],
    );
    s.on_tick("TEST", 5000, 0);
    s.sync_position("TEST", 4, 100);
    assert_eq!(s.states["TEST"].observed_peak, None);
    assert!(!s.states["TEST"].profit_armed);
    s.on_tick("TEST", 106, 0);
    let checkpoint = s.owned_position_metadata("TEST").unwrap();
    let mut c = s.config.clone();
    c.order_quantity = 1;
    assert!(s.reconfigure_preserving_position(c));
    s.initialize_candles("TEST", &[(9999, 9999)]);
    s.sync_position("TEST", 2, 100);
    assert!(matches!(
        s.on_tick("TEST", 102, 0),
        Signal::Sell { quantity: 2, .. }
    ));
    let mut restored = strategy(json!({}));
    restored.sync_position("TEST", 2, 100);
    restored
        .restore_owned_position_metadata("TEST", &checkpoint)
        .unwrap();
    assert!(restored.states["TEST"].profit_armed);
    assert_eq!(restored.states["TEST"].held_observations, Some(1));
}
#[test]
fn strong_close_strict_metadata_atomic_and_pending_fill_age_proof() {
    let mut s = strategy(json!({}));
    seed(&mut s);
    s.on_tick("TEST", 100, 0);
    s.set_order_pending("TEST", true);
    let p = s.owned_position_metadata("TEST").unwrap();
    let mut r = strategy(json!({}));
    r.sync_position("TEST", 0, 0);
    r.restore_owned_position_metadata("TEST", &p).unwrap();
    r.sync_position("TEST", 4, 100);
    assert_eq!(r.states["TEST"].held_observations, Some(0));
    for patch in [
        json!({"kind":"other"}),
        json!({"version":2}),
        json!({"extra":0}),
        json!({"ageKnown":true}),
        json!({"observedPeak":0}),
        json!({"profitArmed":true}),
        json!({"candidateEntry":null}),
    ] {
        let before = r.owned_position_metadata("TEST");
        let mut bad = p.clone();
        bad.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(r.restore_owned_position_metadata("TEST", &bad).is_err());
        assert_eq!(r.owned_position_metadata("TEST"), before);
    }
    let mut missing = p;
    missing.as_object_mut().unwrap().remove("observedPeak");
    assert!(r.restore_owned_position_metadata("TEST", &missing).is_err());
}
#[test]
fn strong_close_reconfigure_preserves_physical_pending_proof_reset_clears() {
    let mut s = strategy(json!({"max_hold_observations":2}));
    seed(&mut s);
    s.on_tick("TEST", 100, 0);
    s.set_order_pending("TEST", true);
    let c = s.config.clone();
    assert!(s.reconfigure_preserving_position(c));
    assert!(!s.states["TEST"].pending_buy);
    s.sync_position("TEST", 2, 100);
    assert_eq!(s.states["TEST"].held_observations, Some(0));
    s.on_tick("TEST", 101, 0);
    assert!(matches!(s.on_tick("TEST", 101, 0), Signal::Sell { .. }));
    s.reset();
    assert!(s.states.is_empty());
}
#[test]
fn strong_close_optional_validation_and_damaged_runtime_preserve_custom_legacy() {
    for patch in [
        json!({"long_trend_period":0.5}),
        json!({"long_trend_period":513}),
        json!({"max_hold_observations":1000001}),
        json!({"profit_activation_pct":5}),
        json!({"profit_trailing_pct":100}),
        json!({"profit_activation_pct":null}),
        json!({"long_trend_period":"200"}),
        json!({"max_hold_observations":false}),
    ] {
        assert!(validate_strong_close_params("strong_close", &patch).is_err());
        let mut v = json!({"threshold_pct":1,"stop_loss_pct":7});
        v.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        let mut s = strategy(v);
        assert_eq!(s.params.threshold_pct, 1.0);
        assert_eq!(s.params.stop_loss_pct, 7.0);
        seed(&mut s);
        assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Hold));
        s.sync_position("TEST", 4, 100);
        assert!(matches!(
            s.on_tick("TEST", 92, 0),
            Signal::Sell { quantity: 4, .. }
        ));
    }
    for v in [
        json!({}),
        json!({"long_trend_period":512.0,"max_hold_observations":1000000}),
        json!({"profit_activation_pct":5,"profit_trailing_pct":3}),
    ] {
        assert!(validate_strong_close_params("strong_close", &v).is_ok());
    }
}
#[test]
fn strong_close_reconfigure_clears_unsubmitted_proof_but_preserves_owned_observations() {
    let mut s = strategy(json!({"max_hold_observations":63}));
    seed(&mut s);
    s.on_tick("TEST", 100, 0);
    assert!(s.states["TEST"].entry_candidate);
    assert!(!s.states["TEST"].order_pending);
    let config = s.config.clone();
    assert!(s.reconfigure_preserving_position(config));
    assert!(!s.states["TEST"].entry_candidate);
    s.sync_position("TEST", 2, 100);
    assert_eq!(s.states["TEST"].held_observations, None);
    s.sync_position("TEST", 0, 0);
    seed(&mut s);
    s.on_tick("TEST", 100, 0);
    s.sync_position("TEST", 3, 100);
    s.on_tick("TEST", 104, 0);
    let before = s.owned_position_metadata("TEST");
    let config = s.config.clone();
    assert!(s.reconfigure_preserving_position(config));
    assert_eq!(s.owned_position_metadata("TEST"), before);
}
#[test]
fn strong_close_invalid_optional_diagnostics_count_unready_without_trend() {
    let mut s = strategy(json!({"profit_activation_pct":null}));
    seed(&mut s);
    assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Hold));
    let p = s.entry_filter_preparation("TEST").unwrap();
    assert_eq!(p.status, "invalidParams");
    assert_eq!(p.candidates, 1);
    assert_eq!(p.unready_candidates, 1);
    assert_eq!(p.trend_rejections, 0);
    assert_eq!(p.passed_candidates, 0);
}
#[test]
fn strong_close_unknown_physical_pending_tick_cannot_invent_first_fill_age() {
    let mut s = strategy(json!({"max_hold_observations":63}));
    seed(&mut s);
    // A restored physical order has no checkpoint identifying its first signal.
    s.set_order_pending("TEST", true);
    assert_eq!(s.owned_position_metadata("TEST"), None);
    assert!(matches!(s.on_live_tick("TEST", 101, 0), Signal::Buy { .. }));
    assert_eq!(s.owned_position_metadata("TEST"), None);
    s.sync_owned_position("TEST", 7, 100);
    assert_eq!(s.states["TEST"].held_observations, None);
    s.set_order_pending("TEST", false);
    assert!(
        matches!(s.on_live_tick("TEST",101,0),Signal::Sell{quantity:7,reason,..}if reason.contains("관측"))
    );
}
#[test]
fn strong_close_known_physical_pending_tick_keeps_original_first_fill_proof() {
    let mut s = strategy(json!({"max_hold_observations":63}));
    seed(&mut s);
    s.on_live_tick("TEST", 101, 0);
    s.set_order_pending("TEST", true);
    let checkpoint = s.owned_position_metadata("TEST").unwrap();
    s.on_live_tick("TEST", 102, 0);
    assert_eq!(s.owned_position_metadata("TEST"), Some(checkpoint));
    s.sync_owned_position("TEST", 7, 100);
    assert_eq!(s.states["TEST"].held_observations, Some(0));
}
