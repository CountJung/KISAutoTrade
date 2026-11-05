use super::*;
use serde_json::{json, Value};

fn config(extra: Value) -> StrategyConfig {
    let mut p = json!({"short_period":2,"long_period":3});
    p.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    StrategyConfig::new(
        "ma_cross_fixture",
        "fixture",
        true,
        vec!["TEST".into()],
        7,
        p,
    )
}
fn bar(close: u64) -> OhlcCandle {
    OhlcCandle {
        open: close,
        high: close,
        low: close,
        close,
    }
}
fn strategy(extra: Value) -> MovingAverageCrossStrategy {
    let mut s = MovingAverageCrossStrategy::new(config(extra));
    s.initialize_historical("TEST", &[100, 90, 80]);
    s
}
fn prep(s: &MovingAverageCrossStrategy) -> EntryFilterPreparation {
    s.entry_filter_preparation("TEST").unwrap()
}
fn cross_state(s: &MovingAverageCrossStrategy) -> (Vec<u64>, Option<f64>, Option<f64>) {
    let state = &s.states["TEST"];
    (
        state.prices.iter().copied().collect(),
        state.prev_short_ma,
        state.prev_long_ma,
    )
}
#[test]
fn disabled_filters_keep_golden_death_cross_quantity_and_metadata() {
    let mut a = strategy(json!({}));
    let mut b = strategy(
        json!({"long_trend_period":0.0,"min_long_ma_slope_pct":0,"min_atr_pct":0,"min_bandwidth_pct":0}),
    );
    assert!(a.entry_filter_preparation("TEST").is_none());
    assert_eq!(a.required_completed_daily_bars(), None);
    assert!(!a.legacy_warmup_requires_daily_verification());
    for (price, side) in [(120, "buy"), (70, "hold"), (60, "sell")] {
        let left = a.on_tick("TEST", price, 0);
        let right = b.on_tick("TEST", price, 0);
        assert_eq!(left, right);
        match (left, side) {
            (
                Signal::Buy {
                    quantity: 7,
                    reason,
                    ..
                },
                "buy",
            ) => assert_eq!(reason, "골든크로스 S100 > L97"),
            (
                Signal::Sell {
                    quantity: 7,
                    reason,
                    ..
                },
                "sell",
            ) => assert_eq!(reason, "데드크로스 S65 < L83"),
            (Signal::Hold, "hold") => (),
            other => panic!("unexpected legacy path {other:?}"),
        }
    }
}
#[test]
fn rejected_cross_is_consumed_and_does_not_revive_after_daily_filter_improves() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.apply_completed_daily_history("TEST", &[bar(200); 3]);
    assert_eq!(s.on_tick("TEST", 120, 0), Signal::Hold);
    assert_eq!(prep(&s).trend_rejections, 1);
    s.apply_completed_daily_history("TEST", &[bar(50); 3]);
    assert_eq!(s.on_tick("TEST", 130, 0), Signal::Hold);
    assert_eq!(prep(&s).candidates, 1);
    assert!(matches!(s.on_tick("TEST", 10, 0), Signal::Sell { .. }));
    assert_eq!(s.on_tick("TEST", 200, 0), Signal::Hold);
    assert!(matches!(s.on_tick("TEST", 300, 0), Signal::Buy { .. }));
    assert_eq!((prep(&s).candidates, prep(&s).passed_candidates), (2, 1));
}
#[test]
fn readiness_never_blocks_death_cross_and_position_feedback_keeps_legacy_raw_behavior() {
    for extra in [
        json!({"min_atr_pct":1}),
        json!({"long_trend_period":512}),
        json!({"min_bandwidth_pct":null}),
    ] {
        let mut s = MovingAverageCrossStrategy::new(config(extra));
        s.initialize_historical("TEST", &[100; 3]);
        assert!(!s.history_readiness("TEST").unwrap().ready);
        s.sync_owned_position("TEST", 0, 0);
        assert!(matches!(
            s.on_tick("TEST", 80, 0),
            Signal::Sell { quantity: 7, .. }
        ));
        s.sync_owned_position("TEST", 2, 80);
        assert!(matches!(s.on_tick("TEST", 200, 0), Signal::Hold));
    }
    let mut held = strategy(json!({}));
    held.sync_owned_position("TEST", 2, 80);
    assert!(matches!(
        held.on_tick("TEST", 120, 0),
        Signal::Buy { quantity: 7, .. }
    ));
}
#[test]
fn malformed_optional_does_not_replace_custom_legacy_periods_or_sell() {
    let mut s = MovingAverageCrossStrategy::new(config(
        json!({"short_period":1,"long_period":3,"min_atr_pct":"bad"}),
    ));
    assert_eq!((s.params.short_period, s.params.long_period), (1, 3));
    s.initialize_historical("TEST", &[100; 3]);
    assert!(matches!(s.on_tick("TEST", 80, 0), Signal::Sell { .. }));
    assert_eq!(s.on_tick("TEST", 200, 0), Signal::Hold);
    assert_eq!(prep(&s).status, "invalidParams");
    assert_eq!(prep(&s).unready_candidates, 1);
}
#[test]
fn context_refresh_failure_and_readiness_queries_never_reseed_cross_observations() {
    let mut s = strategy(json!({"long_trend_period":2}));
    let before = cross_state(&s);
    assert_eq!(prep(&s).status, "unsupportedObservation");
    s.apply_completed_daily_history("TEST", &[bar(50); 2]);
    assert_eq!(cross_state(&s), before);
    assert!(s.completed_daily_history_readiness("TEST").unwrap().ready);
    s.invalidate_completed_daily_history("TEST");
    assert_eq!(cross_state(&s), before);
    assert_eq!(prep(&s).status, "insufficientHistory");
    s.apply_completed_daily_history(
        "TEST",
        &[OhlcCandle {
            open: 100,
            close: 100,
            low: 101,
            high: 100,
        }],
    );
    assert_eq!(cross_state(&s), before);
    assert!(!prep(&s).ready);
    s.apply_completed_daily_history("FOREIGN", &[bar(50); 600]);
    assert!(!s.states.contains_key("FOREIGN"));
}
#[test]
fn current_completed_daily_arrives_after_decision_and_duplicates_are_ignored() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.apply_completed_daily_history("TEST", &[bar(200); 3]);
    s.on_completed_candle("TEST", &bar(50));
    assert_eq!(prep(&s).available_completed_bars, 3);
    s.on_trading_day_start("TEST", 50);
    s.on_trading_day_start("TEST", 50);
    assert_eq!(s.on_tick("TEST", 120, 0), Signal::Hold);
    s.on_completed_candle("TEST", &bar(50));
    assert_eq!(prep(&s).available_completed_bars, 4);
    s.on_completed_candle("TEST", &bar(50));
    assert_eq!(prep(&s).available_completed_bars, 4);
    assert_eq!(prep(&s).trend_rejections, 1);
}
#[test]
fn minute_seed_and_live_quotes_never_manufacture_completed_daily_context() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.apply_completed_daily_history("TEST", &[bar(50); 3]);
    s.initialize_intraday_prices("TEST", &[100, 90, 80]);
    s.initialize_intraday_ohlc("TEST", &[bar(10000); 300]);
    assert_eq!(prep(&s).available_completed_bars, 3);
    assert!(matches!(s.on_live_tick("TEST", 120, 0), Signal::Buy { .. }));
    for p in 121..1000 {
        s.on_live_tick("TEST", p, 0);
    }
    assert_eq!(prep(&s).available_completed_bars, 3);
    let mut no_daily = strategy(json!({"min_atr_pct":0.5}));
    no_daily.initialize_intraday_prices("TEST", &[100, 90, 80]);
    no_daily.initialize_intraday_ohlc("TEST", &[bar(50); 30]);
    assert_eq!(prep(&no_daily).status, "unsupportedObservation");
    assert_eq!(no_daily.on_live_tick("TEST", 120, 0), Signal::Hold);
}
#[test]
fn reconfigure_preserves_bounded_daily_history_and_resets_only_diagnostics() {
    let mut s = strategy(json!({}));
    s.apply_completed_daily_history("TEST", &[bar(50); 800]);
    let mut next = s.config().clone();
    next.params["long_trend_period"] = json!(512);
    assert!(s.reconfigure_preserving_position(next.clone()));
    assert_eq!(prep(&s).available_completed_bars, 512);
    assert!(prep(&s).ready);
    assert!(matches!(s.on_tick("TEST", 120, 0), Signal::Buy { .. }));
    assert_eq!(prep(&s).passed_candidates, 1);
    assert!(s.reconfigure_preserving_position(next.clone()));
    assert_eq!(prep(&s).passed_candidates, 0);
    assert_eq!(prep(&s).available_completed_bars, 512);
    s.on_trading_day_start("TEST", 60);
    s.on_completed_candle("TEST", &bar(60));
    assert_eq!(prep(&s).available_completed_bars, 512);
    let before = cross_state(&s);
    next.broker_account_id = Some("other".into());
    assert!(!s.reconfigure_preserving_position(next));
    assert_eq!(before, cross_state(&s));
    let mut remove = s.config().clone();
    remove.target_symbols = vec!["NEW".into()];
    assert!(s.reconfigure_preserving_position(remove));
    assert!(s.states.is_empty());
    s.on_trading_day_start("NEW", 100);
    s.on_completed_candle("NEW", &bar(100));
    assert_eq!(s.states["NEW"].context.available(), 1);
    s.reset();
    assert!(s.states.is_empty());
}
#[test]
fn new_diagnostics_are_optional_and_existing_consecutive_json_shape_is_preserved() {
    let mut s = strategy(json!({"min_atr_pct":1}));
    s.apply_completed_daily_history("TEST", &[bar(100); 20]);
    let serialized = json!(prep(&s));
    assert_eq!(serialized["slopeRejections"], 0);
    assert_eq!(serialized["bandwidthRejections"], 0);
    let c = super::super::sequence::ConsecutiveMoveStrategy::new(StrategyConfig::new(
        "consecutive_move",
        "fixture",
        true,
        vec!["TEST".into()],
        1,
        json!({"buy_days":1,"sell_days":1,"min_cumulative_move_pct":1}),
    ));
    let serialized = json!(c.entry_filter_preparation("TEST").unwrap());
    assert!(serialized.get("slopeRejections").is_none());
    assert!(serialized.get("bandwidthRejections").is_none());
}
