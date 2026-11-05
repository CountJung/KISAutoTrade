use super::*;
use serde_json::{json, Value};
fn strategy(extra: Value) -> TrendFilterStrategy {
    let mut p = json!({"long_period":4,"short_period":1,"mid_period":2});
    p.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let mut s = TrendFilterStrategy::new(StrategyConfig::new(
        "trend_filter_fixture",
        "fixture",
        true,
        vec!["TEST".into()],
        7,
        p,
    ));
    s.initialize_historical("TEST", &[100; 4]);
    s
}
fn known_fill(s: &mut TrendFilterStrategy) {
    assert!(matches!(
        s.on_tick("TEST", 110, 0),
        Signal::Buy { quantity: 7, .. }
    ));
    assert_eq!(s.states["TEST"].owned_quantity, 0);
    s.set_order_pending("TEST", true);
    s.sync_position("TEST", 0, 0);
    s.sync_owned_position("TEST", 2, 100);
    s.set_order_pending("TEST", false);
    assert_eq!(s.states["TEST"].held_observations, Some(0));
}
#[test]
fn trend_filter_default_strict_current_inclusive_and_overflow_contract() {
    let p = TrendFilterParams::default();
    assert_eq!(
        (
            p.long_period,
            p.short_period,
            p.mid_period,
            p.exit_confirm_observations
        ),
        (200, 5, 20, 1)
    );
    let mut s = strategy(json!({}));
    assert_eq!(s.on_tick("TEST", 100, 0), Signal::Hold);
    assert!(matches!(s.on_tick("TEST", 120, 0), Signal::Buy { .. }));
    assert_eq!(
        TrendFilterStrategy::moving_avg(&s.states["TEST"].prices, 4),
        Some(105.0)
    );
    s.sync_position("TEST", 2, 100);
    s.initialize_historical("TEST", &[100, 100, 100, 120]);
    assert_eq!(s.on_tick("TEST", 110, 0), Signal::Hold);
    s.initialize_historical("TEST", &[100; 4]);
    assert_eq!(s.on_tick("TEST", 100, 0), Signal::Hold);
    assert!(matches!(
        s.on_tick("TEST", 99, 0),
        Signal::Sell { quantity: 2, .. }
    ));
    let prices = VecDeque::from([u64::MAX; 512]);
    assert_eq!(
        TrendFilterStrategy::moving_avg(&prices, 512),
        Some(u64::MAX as f64)
    );
}
#[test]
fn trend_filter_pending_rejected_partial_and_same_quantity_keep_actual_evidence() {
    let mut s = strategy(json!({"exit_confirm_observations":2}));
    known_fill(&mut s);
    assert_eq!(s.on_live_tick("TEST", 90, 0), Signal::Hold);
    s.sync_position("TEST", 2, 100);
    assert!(matches!(
        s.on_live_tick("TEST", 80, 0),
        Signal::Sell { quantity: 2, .. }
    ));
    s.discard_unsubmitted_signal("TEST");
    assert_eq!(s.states["TEST"].owned_quantity, 2);
    s.set_order_pending("TEST", true);
    s.sync_position("TEST", 1, 101);
    assert_eq!(s.states["TEST"].held_observations, Some(2));
    assert!(matches!(
        s.on_tick("TEST", 70, 0),
        Signal::Sell { quantity: 1, .. }
    ));
    assert_eq!(s.states["TEST"].exit_streak, 3);
    assert_eq!(s.states["TEST"].avg_price, 101);
    s.sync_position("TEST", 0, 0);
    assert_eq!(s.states["TEST"].exit_streak, 0);
    assert_eq!(s.owned_position_metadata("TEST"), None);
    s.initialize_historical("TEST", &[100; 4]);
    assert!(matches!(s.on_tick("TEST", 110, 0), Signal::Buy { .. }));
    assert_eq!(s.owned_position_metadata("TEST"), None); // 외부 pending 중 새 기원 금지
    s.sync_position("TEST", 1, 110);
    assert_eq!(s.states["TEST"].held_observations, None);
    assert!(s.owned_position_metadata("TEST").is_some());
}
#[test]
fn trend_filter_unsubmitted_and_rejected_entry_do_not_fabricate_known_age() {
    let mut s = strategy(json!({"max_hold_observations":63}));
    assert!(matches!(s.on_tick("TEST", 110, 0), Signal::Buy { .. }));
    s.discard_unsubmitted_signal("TEST");
    s.sync_position("TEST", 2, 110);
    assert_eq!(s.states["TEST"].held_observations, None);
    assert!(
        matches!(s.on_tick("TEST",120,0),Signal::Sell{quantity:2,reason,..} if reason.contains("관측"))
    );
    let mut s = strategy(json!({}));
    s.on_tick("TEST", 110, 0);
    s.set_order_pending("TEST", true);
    s.sync_position("TEST", 0, 0);
    assert!(s.states["TEST"].entry_candidate);
    s.set_order_pending("TEST", false);
    assert_eq!(s.owned_position_metadata("TEST"), None);
}
#[test]
fn trend_filter_optional_protection_before_readiness_uses_native_avg_and_observed_peak() {
    let mut s = strategy(json!({"profit_activation_pct":5,"profit_trailing_pct":3}));
    known_fill(&mut s);
    let mut next = s.config.clone();
    next.params["mid_period"] = json!(512);
    assert!(s.reconfigure_preserving_position(next));
    assert_eq!(s.on_tick("TEST", 105, 0), Signal::Hold);
    assert!(s.states["TEST"].profit_armed);
    assert_eq!(s.states["TEST"].observed_peak, Some(105));
    assert!(
        matches!(s.on_tick("TEST",101,0),Signal::Sell{quantity:2,reason,..} if reason.contains("수익 보호"))
    );
    assert_eq!(s.states["TEST"].owned_quantity, 2);
    let mut s = strategy(json!({"mid_period":512}));
    s.sync_position("TEST", 2, 100);
    assert_eq!(s.on_tick("TEST", 90, 0), Signal::Hold); // 장기MA만 준비된 원래 청산 금지
    let mut next = s.config.clone();
    next.params["max_hold_observations"] = json!(1);
    assert!(s.reconfigure_preserving_position(next));
    assert!(
        matches!(s.on_tick("TEST",90,0),Signal::Sell{quantity:2,reason,..} if reason.contains("관측"))
    );
}
#[test]
fn trend_filter_reinitialize_core_change_reset_and_optional_change_have_distinct_lifecycles() {
    let mut s = strategy(json!({"exit_confirm_observations":3}));
    known_fill(&mut s);
    s.on_tick("TEST", 90, 0);
    let before = s.owned_position_metadata("TEST");
    assert_eq!(s.states["TEST"].exit_streak, 1);
    let mut next = s.config.clone();
    next.params["max_hold_observations"] = json!(100);
    assert!(s.reconfigure_preserving_position(next.clone()));
    assert_eq!(s.states["TEST"].exit_streak, 1);
    assert_eq!(s.owned_position_metadata("TEST"), before);
    s.initialize_historical("TEST", &[100; 4]);
    assert_eq!(s.states["TEST"].exit_streak, 0);
    assert_eq!(s.owned_position_metadata("TEST"), before);
    s.on_tick("TEST", 90, 0);
    next.params["long_period"] = json!(5);
    assert!(s.reconfigure_preserving_position(next));
    assert_eq!(s.states["TEST"].exit_streak, 0);
    let before_prices = s.states["TEST"].prices.clone();
    s.reset();
    assert_eq!(s.states["TEST"].prices, before_prices);
    assert_eq!(s.owned_position_metadata("TEST"), None);
    s.initialize_historical("OTHER", &[1; 512]);
    assert_eq!(s.states.len(), 1);
}
#[test]
fn trend_filter_strict_checkpoint_is_atomic_and_restore_reobserves_confirmation() {
    let mut s = strategy(json!({"exit_confirm_observations":2}));
    known_fill(&mut s);
    s.on_tick("TEST", 90, 0);
    let valid = s.owned_position_metadata("TEST").unwrap();
    assert_eq!(valid.as_object().unwrap().len(), 7);
    assert_eq!(valid["kind"], "trendFilter");
    let mut malformed = Vec::new();
    for key in valid.as_object().unwrap().keys() {
        let mut bad = valid.clone();
        bad.as_object_mut().unwrap().remove(key);
        malformed.push(bad);
    }
    for patch in [
        json!({"version":2}),
        json!({"kind":"strongClose"}),
        json!({"extra":0}),
        json!({"ageKnown":false}),
        json!({"candidateEntry":true}),
        json!({"observedPeak":0}),
        json!({"observedPeak":null,"profitArmed":true}),
        json!({"heldObservations":-1}),
        json!({"heldObservations":0.5}),
        json!({"ageKnown":"true"}),
    ] {
        let mut bad = valid.clone();
        bad.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        malformed.push(bad);
    }
    for bad in malformed {
        assert!(s.restore_owned_position_metadata("TEST", &bad).is_err());
        assert_eq!(s.states["TEST"].exit_streak, 1);
        assert_eq!(s.owned_position_metadata("TEST"), Some(valid.clone()));
    }
    s.restore_owned_position_metadata("TEST", &valid).unwrap();
    assert_eq!(s.states["TEST"].exit_streak, 0);
    assert_eq!(s.on_tick("TEST", 80, 0), Signal::Hold);
    assert!(matches!(
        s.on_tick("TEST", 70, 0),
        Signal::Sell { quantity: 2, .. }
    ));
    let mut cold = strategy(json!({}));
    assert!(cold
        .restore_owned_position_metadata("TEST", &valid)
        .is_err());
    cold.sync_position("TEST", 2, 100);
    let unknown = cold.owned_position_metadata("TEST").unwrap();
    assert_eq!(unknown["ageKnown"], false);
    assert_eq!(unknown["observedPeak"], Value::Null);
    cold.restore_owned_position_metadata("TEST", &valid)
        .unwrap();
    assert_eq!(cold.owned_position_metadata("TEST"), Some(valid));
}
#[test]
fn trend_filter_raw_validation_and_corrupt_runtime_preserve_independent_periods() {
    for key in [
        "long_period",
        "short_period",
        "mid_period",
        "exit_confirm_observations",
        "max_hold_observations",
        "profit_activation_pct",
        "profit_trailing_pct",
    ] {
        for v in [
            json!(null),
            json!("2"),
            json!(false),
            json!(-1),
            json!(1000001),
        ] {
            let bad = json!({key:v});
            assert!(validate_trend_filter_params("trend_filter", &bad).is_err());
            assert!(validate_trend_filter_params("other", &bad).is_ok());
        }
    }
    for bad in [
        json!({"exit_confirm_observations":0}),
        json!({"max_hold_observations":0.5}),
        json!({"profit_activation_pct":5}),
    ] {
        assert!(validate_trend_filter_params("trend_filter", &bad).is_err());
    }
    assert!(validate_trend_filter_params("trend_filter",&json!({"long_period":1.0,"mid_period":512,"short_period":200,"exit_confirm_observations":1000000,"max_hold_observations":1000000,"profit_activation_pct":1000,"profit_trailing_pct":99.9})).is_ok());
    let mut s = strategy(json!({"max_hold_observations":null}));
    assert_eq!(
        (
            s.params.long_period,
            s.params.short_period,
            s.params.mid_period
        ),
        (4, 1, 2)
    );
    assert_eq!(s.on_tick("TEST", 110, 0), Signal::Hold);
    s.sync_position("TEST", 2, 100);
    assert!(
        matches!(s.on_tick("TEST",120,0),Signal::Sell{quantity:2,reason,..} if reason.contains("설정"))
    );
}
