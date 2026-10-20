use super::*;
use serde_json::json;
fn bar(close: u64) -> OhlcCandle {
    OhlcCandle {
        open: close,
        high: close,
        low: close,
        close,
    }
}
fn p(extra: serde_json::Value) -> MaCrossParams {
    let mut p = json!({"short_period":2,"long_period":3});
    p.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    MaCrossParams::parse(&p).0
}
#[test]
fn strict_trend_and_inclusive_slope_use_prior_daily_values() {
    let mut c = EntryContext::default();
    c.replace(&[bar(100), bar(100), bar(100), bar(103)]);
    let mut params = p(json!({"long_trend_period":3}));
    assert!(!c.accept_candidate(&params, true, 101));
    assert!(c.accept_candidate(&params, true, 102));
    params = p(json!({"min_long_ma_slope_pct":1.0}));
    // Prior3 mean101 / one-day-earlier mean100; candidate's current quote is irrelevant.
    assert!(c.accept_candidate(&params, true, 1));
    params.min_long_ma_slope_pct = 1.000001;
    assert!(!c.accept_candidate(&params, true, 100000));
    assert_eq!(c.preparation(&params, true).slope_rejections, Some(1));
}
#[test]
fn atr_uses_gap_true_ranges_fourteen_denominator_and_prior_close() {
    let mut c = EntryContext::default();
    let mut bars = vec![bar(100); 14];
    bars.push(bar(114));
    c.replace(&bars);
    let mut params = p(json!({"min_atr_pct":100.0/114.0}));
    assert!(c.accept_candidate(&params, true, 1));
    params.min_atr_pct += 0.000001;
    assert!(!c.accept_candidate(&params, true, 100000));
    let prep = c.preparation(&params, true);
    assert!(prep.ready);
    assert_eq!(prep.true_range_samples, 14);
    assert_eq!(prep.atr_rejections, 1);
}
#[test]
fn bandwidth_uses_twenty_population_samples_and_inclusive_boundary() {
    let mut c = EntryContext::default();
    let mut bars = vec![bar(10); 10];
    bars.extend(vec![bar(20); 10]);
    c.replace(&bars);
    let boundary = 4.0 * 5.0 / 15.0 * 100.0;
    let mut params = p(json!({"min_bandwidth_pct":boundary}));
    assert!(c.accept_candidate(&params, true, 1));
    params.min_bandwidth_pct += 0.000001;
    assert!(!c.accept_candidate(&params, true, 100000));
    assert_eq!(c.preparation(&params, true).bandwidth_rejections, Some(1));
}
#[test]
fn sufficient_zero_ranges_are_ready_rejections_while_missing_history_is_unready() {
    let mut c = EntryContext::default();
    let params = p(json!({"min_atr_pct":0.5,"min_bandwidth_pct":2.0}));
    c.replace(&[bar(100); 19]);
    assert!(!c.accept_candidate(&params, true, 100));
    assert_eq!(c.preparation(&params, true).status, "insufficientHistory");
    c.replace(&[bar(100); 20]);
    assert!(!c.accept_candidate(&params, true, 100));
    let prep = c.preparation(&params, true);
    assert!(prep.ready);
    assert_eq!(prep.unready_candidates, 1);
    assert_eq!(prep.atr_rejections, 1);
    assert_eq!(prep.bandwidth_rejections, Some(1));
}
#[test]
fn repeated_rejection_reasons_overlap_without_inflating_candidate_count() {
    let mut c = EntryContext::default();
    c.replace(&[bar(100); 20]);
    let params = p(
        json!({"long_trend_period":20,"min_long_ma_slope_pct":0.1,"min_atr_pct":0.5,"min_bandwidth_pct":2.0}),
    );
    assert!(!c.accept_candidate(&params, true, 99));
    let prep = c.preparation(&params, true);
    assert_eq!(prep.candidates, 1);
    assert_eq!(prep.trend_rejections, 1);
    assert_eq!(prep.slope_rejections, Some(1));
    assert_eq!(prep.atr_rejections, 1);
    assert_eq!(prep.bandwidth_rejections, Some(1));
    assert_eq!(prep.passed_candidates, 0);
}
#[test]
fn large_native_prices_do_not_overflow_daily_statistics_and_invalid_ohlc_fail_closed() {
    let mut c = EntryContext::default();
    c.replace(&[bar(u64::MAX); 20]);
    assert_eq!(mean_close(&c.bars, 20), Some(u64::MAX as f64));
    assert_eq!(c.bandwidth(), Some(0.0));
    assert!(
        c.preparation(&p(json!({"long_trend_period":20})), true)
            .ready
    );
    c.awaiting_completion = true;
    c.complete(OhlcCandle {
        open: 1,
        close: 1,
        low: 0,
        high: 1,
    });
    assert!(!c.preparation(&p(json!({"min_atr_pct":1})), true).ready);
    assert_eq!(c.available(), 0);
}
