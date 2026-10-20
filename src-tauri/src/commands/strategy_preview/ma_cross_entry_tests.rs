//! MA candidate filters use the shared production driver and actual daily input only.
use super::*;
use serde_json::{json, Value};
fn bar(date: String, price: u64) -> Value {
    json!({"date":date,"open":price.to_string(),"high":price.to_string(),"low":price.to_string(),"close":price.to_string(),"volume":"1"})
}
fn request(extra: Value, model: &str, chunk_size: usize) -> Value {
    let mut params = json!({"short_period":2,"long_period":3});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let prices = [120, 70, 60, 150, 160, 40, 30, 180];
    let candles = prices
        .iter()
        .enumerate()
        .map(|(i, p)| bar(format!("202610{:02}", i + 1), *p))
        .collect::<Vec<_>>();
    let mut history = (1..=20)
        .map(|d| bar(format!("202609{d:02}"), 100))
        .collect::<Vec<_>>();
    history[18] = bar("20260919".into(), 90);
    history[19] = bar("20260920".into(), 80);
    json!({"strategyId":"ma_cross_fixture","strategyName":"fixture","symbol":"TEST","isOverseas":false,"orderQuantity":2,"params":params,"historyCandles":history,"chunks":candles.chunks(chunk_size).map(|c|json!({"candles":c})).collect::<Vec<_>>(),"interval":"1d","assumptions":{"initialCapitalKrw":1000000,"feeBps":0,"slippageBps":0,"taxBps":0,"executionModel":model,"maxPositionRatio":1.0}})
}
fn flat(mut value: Value) -> StrategyPreviewInput {
    let chunks = value.as_object_mut().unwrap().remove("chunks").unwrap();
    value["candles"] = Value::Array(
        chunks
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|c| c["candles"].as_array().unwrap().clone())
            .collect(),
    );
    serde_json::from_value(value).unwrap()
}
fn run(value: Value) -> ContinuousStrategyPreviewView {
    preview_strategy_continuous_from_candles(serde_json::from_value(value).unwrap()).unwrap()
}
#[test]
fn ma_cross_options_are_strict_in_short_and_continuous_preview_paths() {
    for extra in [
        json!({"long_trend_period":0.5}),
        json!({"long_trend_period":513}),
        json!({"min_atr_pct":null}),
        json!({"min_atr_pct":"1"}),
        json!({"min_atr_pct":101}),
        json!({"min_long_ma_slope_pct":-1}),
        json!({"min_long_ma_slope_pct":0.1,"long_period":512}),
        json!({"min_long_ma_slope_pct":0.1,"long_period":"512"}),
        json!({"min_long_ma_slope_pct":0.1,"long_period":8,"short_period":"bad"}),
        json!({"min_bandwidth_pct":1001}),
    ] {
        let r = request(extra, "nextOpen", 8);
        assert_eq!(
            preview_strategy_from_candles(flat(r.clone()))
                .unwrap_err()
                .code,
            "INVALID_STRATEGY_PARAMS"
        );
        assert_eq!(
            preview_strategy_continuous_from_candles(serde_json::from_value(r).unwrap())
                .unwrap_err()
                .code,
            "INVALID_STRATEGY_PARAMS"
        );
    }
}
#[test]
fn non_daily_filters_reject_explicitly_and_disabled_minute_compatibility_remains() {
    for extra in [
        json!({"long_trend_period":2}),
        json!({"min_long_ma_slope_pct":0.1}),
        json!({"min_atr_pct":0.5}),
        json!({"min_bandwidth_pct":2}),
    ] {
        let mut r = request(extra, "nextOpen", 8);
        r["interval"] = json!("W");
        assert_eq!(
            preview_strategy_from_candles(flat(r.clone()))
                .unwrap_err()
                .code,
            "UNSUPPORTED_ENTRY_FILTER_INTERVAL"
        );
        assert_eq!(
            preview_strategy_continuous_from_candles(serde_json::from_value(r).unwrap())
                .unwrap_err()
                .code,
            "UNSUPPORTED_ENTRY_FILTER_INTERVAL"
        );
    }
    let mut r = request(json!({}), "nextOpen", 8);
    r["interval"] = json!("1m");
    r["historyCandles"] = json!([]);
    for (i, c) in r["chunks"][0]["candles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        c["date"] = json!(format!("2026100109{i:02}00"));
    }
    let result = run(r);
    assert!(result.preview.entry_filter_preparation.is_none());
}
#[test]
fn integer_float_daily_period_stays_active_and_missing_history_blocks_only_golden_cross() {
    let mut r = request(json!({"long_trend_period":512.0}), "sameClose", 8);
    r["historyCandles"] = json!([]);
    let result = run(r);
    let p = result.preview.entry_filter_preparation.unwrap();
    assert_eq!(p.required_completed_bars, 512);
    assert_eq!(p.status, "insufficientHistory");
    assert!(p.unready_candidates > 0);
    assert!(result.preview.signals.iter().any(|s| s.side == "sell"));
    assert!(!result.preview.signals.iter().any(|s| s.side == "buy"));
}
#[test]
fn candidate_current_range_cannot_change_its_own_prior_atr_filter() {
    let left = request(json!({"min_atr_pct":0.5}), "nextOpen", 8);
    let mut right = left.clone();
    right["chunks"][0]["candles"][0]["high"] = json!("10000");
    right["chunks"][0]["candles"][0]["low"] = json!("1");
    let a = run(left);
    let b = run(right);
    assert_eq!(json!(a.preview.signals[0]), json!(b.preview.signals[0]));
    assert_eq!(a.preview.signals[0].side, "buy");
}
#[test]
fn strategy_reports_match_across_chunk_partitions_and_future_suffix_is_causal() {
    for model in ["sameClose", "nextOpen"] {
        for extra in [
            json!({}),
            json!({"long_trend_period":3}),
            json!({"min_long_ma_slope_pct":0.1}),
            json!({"min_atr_pct":0.5}),
            json!({"min_bandwidth_pct":2}),
        ] {
            let a = run(request(extra.clone(), model, 8));
            let b = run(request(extra.clone(), model, 1));
            assert_eq!(json!(a.preview.signals), json!(b.preview.signals));
            assert_eq!(json!(a.preview.backtest), json!(b.preview.backtest));
            assert_eq!(
                json!(a.preview.entry_filter_preparation),
                json!(b.preview.entry_filter_preparation)
            );
            let mut altered = request(extra, model, 8);
            altered["chunks"][0]["candles"][6] = bar("20261007".into(), 10000);
            altered["chunks"][0]["candles"][7] = bar("20261008".into(), 1);
            let changed = run(altered);
            let signals = |p: &StrategyPreviewView| {
                p.signals
                    .iter()
                    .filter(|s| s.time.as_str() <= "20261006")
                    .map(|s| json!(s))
                    .collect::<Vec<_>>()
            };
            let trades = |p: &StrategyPreviewView| {
                p.backtest
                    .trades
                    .iter()
                    .filter(|s| s.time.as_str() <= "20261006")
                    .map(|s| json!(s))
                    .collect::<Vec<_>>()
            };
            assert_eq!(signals(&a.preview), signals(&changed.preview));
            assert_eq!(trades(&a.preview), trades(&changed.preview));
            assert_eq!(
                json!(&a.preview.backtest.equity_curve[..6]),
                json!(&changed.preview.backtest.equity_curve[..6])
            );
        }
    }
}
#[test]
fn guard_rejection_does_not_recreate_a_consumed_golden_cross() {
    for model in ["sameClose", "nextOpen"] {
        let mut r = request(json!({"long_trend_period":3}), model, 1);
        r["assumptions"]["initialCapitalKrw"] = json!(10000);
        r["orderQuantity"] = json!(10000);
        let result = run(r);
        assert_eq!(result.preview.signals[0].side, "buy");
        assert_eq!(result.preview.signals[0].time, "20261001");
        assert!(!result
            .preview
            .signals
            .iter()
            .any(|s| s.side == "buy" && s.time == "20261002"));
        assert!(!result
            .preview
            .backtest
            .trades
            .iter()
            .any(|s| s.status == "filled"));
    }
}
