use super::*;
use serde_json::json;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new(
        "momentum_fixture",
        "fixture",
        true,
        vec!["TEST".into()],
        2,
        params,
    )
}
fn strategy(extra: serde_json::Value) -> MomentumStrategy {
    let mut params = json!({"lookback_period":3,"threshold_pct":5.0});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    MomentumStrategy::new(config(params))
}
fn crossing(hysteresis: f64) -> MomentumStrategy {
    strategy(json!({"entry_mode":"crossing","entry_hysteresis_pct":hysteresis}))
}
fn buy(signal: Signal) {
    assert!(matches!(signal, Signal::Buy { .. }), "{signal:?}");
}
fn sell(signal: Signal) {
    assert!(matches!(signal, Signal::Sell { .. }), "{signal:?}");
}

#[test]
fn level_preserves_inclusive_threshold_but_never_sells_flat_or_buys_owned() {
    let mut s = strategy(json!({}));
    s.initialize_historical("TEST", &[10000; 4]);
    assert_eq!(s.on_tick("TEST", 9500, 0), Signal::Hold);
    s.initialize_historical("TEST", &[10000; 4]);
    buy(s.on_tick("TEST", 10500, 0)); // exactly +5%
    assert_eq!(s.has_held_position("TEST"), Some(false));
    s.sync_owned_position("TEST", 2, 10500);
    assert_eq!(s.on_tick("TEST", 10600, 0), Signal::Hold);
    s.initialize_historical("TEST", &[10000; 4]);
    sell(s.on_tick("TEST", 9500, 0)); // exactly -5%
    assert_eq!(s.has_held_position("TEST"), Some(true));
}

#[test]
fn crossing_inclusive_entry_retries_after_equal_zero_sync_and_consumes_only_real_fill() {
    let mut s = crossing(1.0);
    s.initialize_historical("TEST", &[10000; 4]);
    buy(s.on_tick("TEST", 10500, 0));
    s.sync_position("TEST", 0, 0); // guard rejection OR nextOpen pending: indistinguishable
    buy(s.on_tick("TEST", 10600, 0));
    s.sync_position("TEST", 0, 0); // provider failure leaves the retry eligible
    buy(s.on_live_tick("TEST", 10700, 0));
    s.sync_owned_position("TEST", 2, 10650);
    assert!(!s.states["TEST"].retry_candidate);
    assert_eq!(s.on_tick("TEST", 12000, 0), Signal::Hold);
    assert_eq!(s.states["TEST"].owned_quantity, 2);
}

#[test]
fn crossing_cancellation_requires_full_hysteresis_reset_before_reentry() {
    let mut s = crossing(1.0);
    // Warmup crosses5 historically then ends at4.5: candidate absent, disarmed.
    s.initialize_historical("TEST", &[10000, 10000, 10000, 10500, 10450]);
    assert!(!s.states["TEST"].armed);
    assert!(!s.states["TEST"].retry_candidate);
    assert_eq!(s.on_tick("TEST", 10500, 0), Signal::Hold);
    // A new independent prior context seeds exactly4%, enabling a5% crossing.
    s.initialize_historical("TEST", &[10000, 10000, 10000, 10400]);
    assert!(s.states["TEST"].armed);
    buy(s.on_tick("TEST", 10500, 0));
    assert_eq!(s.on_tick("TEST", 10450, 0), Signal::Hold); //4.5 cancels, no rearm
    assert!(!s.states["TEST"].armed);
    assert!(!s.states["TEST"].retry_candidate);
    assert_eq!(s.on_tick("TEST", 11000, 0), Signal::Hold); //5.77% re-cross without reset
}

#[test]
fn threshold_equality_without_prior_strictly_lower_momentum_is_not_crossing() {
    let mut s = crossing(0.0);
    s.initialize_historical("TEST", &[10000, 10000, 10000, 10500]); // prior exactly5
    assert_eq!(s.on_tick("TEST", 10500, 0), Signal::Hold);
    assert_eq!(s.on_tick("TEST", 10600, 0), Signal::Hold);
    assert_eq!(s.has_held_position("TEST"), Some(false));
}

#[test]
fn warmup_crossings_never_create_virtual_orders_or_erase_actual_holdings() {
    let mut s = crossing(1.0);
    s.initialize_historical("TEST", &[10000, 10000, 10000, 10600]);
    assert_eq!(s.on_tick("TEST", 10700, 0), Signal::Hold);
    assert_eq!(s.has_held_position("TEST"), Some(false));
    s.sync_owned_position("TEST", 3, 10000);
    s.initialize_historical("TEST", &[10000; 4]);
    assert_eq!(s.states["TEST"].owned_quantity, 3);
    sell(s.on_live_tick("TEST", 9500, 0));
    s.initialize_intraday_prices("TEST", &[10000; 4]);
    assert_eq!(s.states["TEST"].owned_quantity, 3);
    sell(s.on_tick("TEST", 9500, 0));
}

#[test]
fn partial_sell_and_rejected_sell_keep_negative_level_exit_eligible() {
    let mut s = crossing(1.0);
    s.initialize_historical("TEST", &[10000; 4]);
    s.sync_owned_position("TEST", 3, 10000);
    for qty in [3, 2, 1] {
        sell(s.on_live_tick("TEST", 9000, 0));
        s.sync_owned_position("TEST", qty, 10000);
        assert_eq!(s.has_held_position("TEST"), Some(true));
    }
    s.sync_owned_position("TEST", 0, 0);
    assert_eq!(s.on_tick("TEST", 8000, 0), Signal::Hold);
}

#[test]
fn invalid_optional_fields_preserve_custom_legacy_exit_and_fail_closed_entries() {
    for extra in [
        json!({"entry_mode":"other"}),
        json!({"entry_mode":null}),
        json!({"entry_hysteresis_pct":"1"}),
        json!({"entry_hysteresis_pct":101}),
        json!({"entry_mode":"crossing","entry_hysteresis_pct":6}),
    ] {
        let mut s = strategy(extra);
        assert_eq!(s.params.lookback_period, 3);
        assert_eq!(s.params.threshold_pct, 5.0);
        s.initialize_historical("TEST", &[10000; 4]);
        assert_eq!(s.on_tick("TEST", 10600, 0), Signal::Hold);
        s.sync_owned_position("TEST", 2, 10000);
        sell(s.on_tick("TEST", 9000, 0));
    }
    let mut custom = MomentumStrategy::new(config(
        json!({"lookback_period":2,"threshold_pct":7.0,"entry_mode":"broken"}),
    ));
    custom.initialize_historical("TEST", &[10000; 3]);
    assert_eq!(custom.on_tick("TEST", 12000, 0), Signal::Hold);
    custom.sync_owned_position("TEST", 1, 10000);
    custom.initialize_historical("TEST", &[10000; 3]);
    assert_eq!(custom.on_tick("TEST", 9400, 0), Signal::Hold); // custom7%, not fallback5%
    sell(custom.on_tick("TEST", 9300, 0));
}

#[test]
fn level_hysteresis_is_inert_but_range_is_validated_for_every_mode() {
    let mut s = strategy(json!({"entry_hysteresis_pct":100}));
    s.initialize_historical("TEST", &[10000; 4]);
    buy(s.on_tick("TEST", 10500, 0));
    for value in [json!(-1), json!(100.1), json!(null), json!(true)] {
        assert!(
            validate_momentum_entry_params("momentum", &json!({"entry_hysteresis_pct":value}))
                .is_err()
        );
    }
    assert!(validate_momentum_entry_params("rsi", &json!({"entry_mode":"other"})).is_ok());
}

#[test]
fn reconfigure_preserves_candidate_when_settings_unchanged_but_reseeds_changed_entry_rule() {
    let mut s = crossing(1.0);
    s.initialize_historical("TEST", &[10000; 4]);
    buy(s.on_tick("TEST", 10500, 0));
    let mut config = s.config().clone();
    config.order_quantity = 7;
    assert!(s.reconfigure_preserving_position(config));
    assert!(matches!(
        s.on_tick("TEST", 10600, 0),
        Signal::Buy { quantity: 7, .. }
    ));
    let mut config = s.config().clone();
    config.params["entry_hysteresis_pct"] = json!(2.0);
    assert!(s.reconfigure_preserving_position(config));
    assert!(!s.states["TEST"].retry_candidate);
    assert_eq!(s.on_tick("TEST", 10700, 0), Signal::Hold);
    s.sync_owned_position("TEST", 2, 10600);
    let mut config = s.config().clone();
    config.params["lookback_period"] = json!(2);
    assert!(s.reconfigure_preserving_position(config));
    assert_eq!(s.states["TEST"].owned_quantity, 2);
    sell(s.on_tick("TEST", 9000, 0));
}

#[test]
fn restart_unknown_crossing_and_zero_price_history_require_a_new_observed_baseline() {
    let mut s = crossing(0.0);
    for price in [10000, 10000, 10000] {
        assert_eq!(s.on_tick("TEST", price, 0), Signal::Hold);
    }
    assert_eq!(s.on_tick("TEST", 10600, 0), Signal::Hold); // no previous valid m
    s.initialize_historical("TEST", &[0, 0, 0, 10000]);
    assert_eq!(s.on_tick("TEST", 11000, 0), Signal::Hold);
    s.reset();
    s.sync_owned_position("TEST", 1, 10000);
    s.initialize_historical("TEST", &[10000; 4]);
    sell(s.on_tick("TEST", 9500, 0));
}

#[test]
fn bounded_history_and_future_suffix_cannot_change_past_signals_or_state() {
    let mut left = crossing(1.0);
    let mut right = crossing(1.0);
    for s in [&mut left, &mut right] {
        s.initialize_historical("TEST", &vec![10000; 2000]);
    }
    assert_eq!(left.states["TEST"].prices.len(), 4);
    for price in [10400, 10500, 10600] {
        assert_eq!(
            left.on_tick("TEST", price, 0),
            right.on_tick("TEST", price, 0)
        );
        left.sync_position("TEST", 0, 0);
        right.sync_position("TEST", 0, 0);
    }
    assert_eq!(
        left.states["TEST"].previous_momentum,
        right.states["TEST"].previous_momentum
    );
    assert_eq!(
        left.states["TEST"].retry_candidate,
        right.states["TEST"].retry_candidate
    );
    assert_eq!(left.on_tick("TEST", 1, 0), Signal::Hold);
    assert_eq!(right.has_held_position("TEST"), Some(false));
    buy(right.on_tick("TEST", 12000, 0));
}

#[test]
fn live_manager_advances_pending_observations_and_retries_after_no_fill() {
    use crate::broker::{BrokerId, BrokerMarket, BrokerScope};
    use crate::trading::strategy::StrategyManager;
    let scope = BrokerScope::new(BrokerId::Kis, None);
    let mut manager = StrategyManager::new();
    manager.add(Box::new(crossing(1.0)));
    manager.set_history_execution_identity(scope.clone(), Some("fixture".into()));
    manager.initialize_historical("TEST", &[10000; 4]);
    assert!(manager.note_order_pending("momentum_fixture", &scope, BrokerMarket::Us, "TEST", true));
    assert!(manager.on_tick("TEST", 10500, 0).is_empty()); // captures crossing while suppressed
    assert!(manager.on_tick("TEST", 10600, 0).is_empty()); // keeps pending observations moving
    assert!(manager.note_order_pending(
        "momentum_fixture",
        &scope,
        BrokerMarket::Us,
        "TEST",
        false
    ));
    assert!(manager.sync_owned_position_for_strategy(
        "momentum_fixture",
        &scope,
        BrokerMarket::Us,
        "TEST",
        0,
        0
    ));
    let retry = manager.on_tick("TEST", 10700, 0);
    assert_eq!(retry.len(), 1);
    buy(retry[0].signal.clone());
    manager.complete_signal("momentum_fixture", "TEST", manager.history_generation());
    assert!(manager.sync_owned_position_for_strategy(
        "momentum_fixture",
        &scope,
        BrokerMarket::Us,
        "TEST",
        1,
        10600
    ));
    assert!(manager.on_tick("TEST", 12000, 0).is_empty()); // confirmed ownership blocks pyramiding
    let exit = manager.on_tick("TEST", 9000, 0);
    assert_eq!(exit.len(), 1);
    sell(exit[0].signal.clone());
}
