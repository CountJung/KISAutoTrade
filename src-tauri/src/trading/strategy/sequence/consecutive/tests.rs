use super::*;
use serde_json::json;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new(
        "consecutive_move_test",
        "fixture",
        true,
        vec!["TEST".into()],
        2,
        params,
    )
}
fn strategy(extra: serde_json::Value) -> ConsecutiveMoveStrategy {
    let mut params = json!({"buy_days":2,"sell_days":4});
    for (k, v) in extra.as_object().unwrap() {
        params[k] = v.clone();
    }
    ConsecutiveMoveStrategy::new(config(params))
}
fn bar(close: u64, range: u64) -> OhlcCandle {
    OhlcCandle {
        open: close,
        high: close + range,
        low: close.saturating_sub(range).max(1),
        close,
    }
}
fn prep(s: &ConsecutiveMoveStrategy) -> EntryFilterPreparation {
    s.entry_filter_preparation("TEST").unwrap()
}

#[test]
fn omitted_and_explicit_zero_filters_preserve_custom_legacy_observations_and_exit() {
    for extra in [
        json!({}),
        json!({"min_cumulative_move_pct":0,"min_move_atr_multiplier":0,"long_trend_period":0}),
    ] {
        let mut s = strategy(extra);
        s.initialize_historical("TEST", &[100, 101]);
        assert!(matches!(s.on_tick("TEST", 102, 0), Signal::Buy { .. }));
        assert!(s.entry_filter_preparation("TEST").is_none());
        for p in [101, 100, 99] {
            assert_eq!(s.on_tick("TEST", p, 0), Signal::Hold);
        }
        assert!(matches!(s.on_tick("TEST", 98, 0), Signal::Sell { .. }));
    }
}
#[test]
fn pct_threshold_counts_only_actual_entry_candidates_and_uses_entire_rising_run() {
    let mut s = strategy(json!({"min_cumulative_move_pct":1.0}));
    s.initialize_historical("TEST", &[1000, 1001]);
    assert_eq!(s.on_tick("TEST", 1002, 0), Signal::Hold);
    assert_eq!(prep(&s).min_move_rejections, 1);
    assert!(matches!(s.on_tick("TEST", 1012, 0), Signal::Buy { .. }));
    let p = prep(&s);
    assert_eq!(
        (p.candidates, p.passed_candidates, p.unready_candidates),
        (2, 1, 0)
    );
    s.on_tick("TEST", 1013, 0);
    assert_eq!(prep(&s).candidates, 2); // held is never an entry candidate
}
#[test]
fn atr_includes_gaps_but_excludes_candidate_ohlc_and_completion_is_once() {
    let mut s = strategy(json!({"min_move_atr_multiplier":0.5}));
    let mut history = vec![bar(100, 1); 15];
    history[14] = bar(200, 1);
    s.initialize_ohlc("TEST", &history); // TRs: 13*2+101, ATR=127/14
    s.initialize_intraday_prices("TEST", &[200, 201]); // price-only seed invalidates OHLC provenance
    assert!(!prep(&s).ready);
    s.apply_completed_daily_history("TEST", &history);
    s.on_trading_day_start("TEST", 201);
    assert_eq!(
        s.on_daily_close_tick("TEST", &bar(202, 100), 0),
        Signal::Hold
    ); // delta2 < ATR*.5
    assert_eq!(prep(&s).atr_rejections, 1);
    assert_eq!(prep(&s).available_completed_bars, 15);
    s.on_completed_candle("TEST", &bar(202, 100));
    let before = prep(&s);
    s.on_completed_candle("TEST", &bar(500, 400));
    assert_eq!(
        prep(&s).available_completed_bars,
        before.available_completed_bars
    );
    // The already-completed high range now tightens the NEXT candidate threshold.
    assert_eq!(s.on_tick("TEST", 207, 0), Signal::Hold);
    assert_eq!(prep(&s).atr_rejections, 2);
}
#[test]
fn zero_atr_and_price_only_history_do_not_become_successful_ohlc_readiness() {
    let mut s = strategy(json!({"min_move_atr_multiplier":0.5}));
    s.initialize_historical("TEST", &[100, 101]);
    assert_eq!(s.on_tick("TEST", 102, 0), Signal::Hold);
    assert_eq!(prep(&s).status, "unsupportedObservation");
    s.apply_completed_daily_history("TEST", &vec![bar(100, 0); 15]);
    assert_eq!(prep(&s).status, "zeroAtr");
    assert_eq!(s.on_tick("TEST", 103, 0), Signal::Hold);
    assert_eq!(prep(&s).unready_candidates, 2);
}
#[test]
fn long_ma_is_prior_completed_daily_and_strictly_greater_than_candidate() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.initialize_historical("TEST", &[98, 99]);
    s.apply_completed_daily_history("TEST", &[bar(100, 1); 3]);
    assert_eq!(s.on_tick("TEST", 100, 0), Signal::Hold);
    assert_eq!(prep(&s).trend_rejections, 1);
    assert!(matches!(s.on_tick("TEST", 101, 0), Signal::Buy { .. }));
    assert_eq!(prep(&s).available_completed_bars, 3); // no quote creates a completed bar
}
#[test]
fn combined_threshold_rejections_are_independent_and_require_all_filters() {
    let mut s = strategy(
        json!({"min_cumulative_move_pct":2,"min_move_atr_multiplier":2,"long_trend_period":15}),
    );
    s.initialize_historical("TEST", &[100, 101]);
    s.apply_completed_daily_history("TEST", &[bar(200, 10); 15]);
    assert_eq!(s.on_tick("TEST", 102, 0), Signal::Hold);
    let p = prep(&s);
    assert_eq!(
        (
            p.candidates,
            p.min_move_rejections,
            p.atr_rejections,
            p.trend_rejections,
            p.passed_candidates
        ),
        (1, 0, 1, 1, 0)
    );
}
#[test]
fn invalid_optional_fields_fail_closed_without_replacing_custom_held_exit_parameters() {
    for extra in [
        json!({"min_cumulative_move_pct":"bad"}),
        json!({"min_move_atr_multiplier":-1}),
        json!({"long_trend_period":2.5}),
        json!({"long_trend_period":513}),
    ] {
        let mut s = strategy(extra);
        assert_eq!(s.params.sell_days, 4);
        assert_eq!(s.params.buy_days, 2);
        s.initialize_historical("TEST", &[100, 101]);
        assert_eq!(s.on_tick("TEST", 102, 0), Signal::Hold);
        assert_eq!(prep(&s).status, "invalidParams");
        s.sync_position("TEST", 2, 102);
        for price in [101, 100, 99] {
            assert_eq!(s.on_live_tick("TEST", price, 0), Signal::Hold);
        }
        assert!(s.can_evaluate_next_tick("TEST").unwrap());
        assert!(matches!(s.on_live_tick("TEST", 98, 0), Signal::Sell { .. }));
        assert!(s.states["TEST"].in_position); // live publication still needs exact owner fill
    }
}
#[test]
fn live_context_refresh_pending_failure_and_invalidation_preserve_streak_and_owned_exit() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.initialize_historical("TEST", &[100, 101]);
    let prices = s.states["TEST"].prices.clone();
    s.apply_completed_daily_history("TEST", &[bar(90, 1); 3]);
    assert_eq!(s.states["TEST"].prices, prices);
    assert!(matches!(s.on_live_tick("TEST", 102, 0), Signal::Buy { .. }));
    assert!(!s.states["TEST"].in_position);
    s.sync_owned_position("TEST", 2, 102);
    for price in [101, 100, 99] {
        s.on_live_tick("TEST", price, 0);
    }
    let prices = s.states["TEST"].prices.clone();
    s.invalidate_completed_daily_history("TEST");
    assert_eq!(s.states["TEST"].prices, prices);
    assert!(s.states["TEST"].in_position);
    assert!(matches!(s.on_live_tick("TEST", 98, 0), Signal::Sell { .. }));
    assert!(s.states["TEST"].in_position);
}
#[test]
fn reconfigure_retains_owned_position_and_legacy_streak_but_reports_larger_context_shortage() {
    let mut s = strategy(json!({"long_trend_period":3}));
    s.initialize_historical("TEST", &[104, 103, 102, 101]);
    s.apply_completed_daily_history("TEST", &[bar(90, 1); 3]);
    s.sync_owned_position("TEST", 2, 105);
    let prices = s.states["TEST"].prices.clone();
    let mut next = s.config().clone();
    next.params["long_trend_period"] = json!(200);
    assert!(s.reconfigure_preserving_position(next));
    assert_eq!(s.states["TEST"].prices, prices);
    assert_eq!(prep(&s).status, "insufficientHistory");
    assert_eq!(prep(&s).candidates, 0);
    assert!(matches!(s.on_tick("TEST", 100, 0), Signal::Sell { .. }));
}
#[test]
fn huge_legacy_move_counts_do_not_overflow_or_grow_unbounded_buffers() {
    let mut s = ConsecutiveMoveStrategy::new(config(
        json!({"buy_days":usize::MAX,"sell_days":usize::MAX}),
    ));
    s.initialize_historical("TEST", &vec![100; 2000]);
    assert!(s.states["TEST"].prices.len() <= 513);
    assert_eq!(s.on_tick("TEST", 101, 0), Signal::Hold);
    s.sync_position("TEST", 2, 100);
    assert_eq!(s.on_tick("TEST", 99, 0), Signal::Hold);
}
#[test]
fn completed_daily_context_remains_bounded_at_512_and_has_no_current_price_duplicates() {
    let mut s = strategy(json!({"long_trend_period":512}));
    s.initialize_ohlc("TEST", &vec![bar(100, 1); 2000]);
    assert_eq!(prep(&s).available_completed_bars, 512);
    for price in 101..1000 {
        s.on_trading_day_start("TEST", price);
        s.on_daily_close_tick("TEST", &bar(price, 1), 0);
        s.on_completed_candle("TEST", &bar(price, 1));
    }
    assert_eq!(prep(&s).available_completed_bars, 512);
    assert!(s.states["TEST"].prices.len() <= 5);
}

#[test]
fn entry_atr_retains_fractional_true_range_average_and_inclusive_threshold() {
    let mut s = strategy(json!({"min_move_atr_multiplier":2.0}));
    s.initialize_historical("TEST", &[100, 103]);
    let completed = (0..15)
        .map(|i| OhlcCandle {
            open: 100,
            high: 102,
            low: if i % 2 == 0 { 98 } else { 99 },
            close: 100,
        })
        .collect::<Vec<_>>();
    s.apply_completed_daily_history("TEST", &completed);
    // Seven TR3 and seven TR4 produce ATR3.5. Flooring to3 would admit delta6.
    assert_eq!(s.on_tick("TEST", 106, 0), Signal::Hold);
    assert_eq!(prep(&s).atr_rejections, 1);
    assert!(matches!(s.on_tick("TEST", 110, 0), Signal::Buy { .. })); // delta7 == 3.5*2
}
