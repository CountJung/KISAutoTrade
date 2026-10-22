//! Synthetic production driver tests; actual accounts and provider calls are absent.
use super::*;
use serde_json::{json, Value};
fn bar(date: String, price: u64) -> Value {
    json!({"date":date,"open":price.to_string(),"high":(price+1).to_string(),"low":(price-1).to_string(),"close":price.to_string(),"volume":"100"})
}
fn request(extra: Value, execution: &str, prices: &[u64], chunk: usize) -> Value {
    let bars = prices
        .iter()
        .enumerate()
        .map(|(i, p)| bar(format!("202610{:02}", i + 1), *p))
        .collect::<Vec<_>>();
    let mut params = json!({"lookback_days":3,"buffer_pct":0.5});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    json!({"strategyId":"failed_breakout_fixture","strategyName":"fixture","symbol":"TEST","isOverseas":false,"orderQuantity":2,"params":params,"historyCandles":(1..=21).map(|i|bar(format!("202609{i:02}"),10000)).collect::<Vec<_>>(),"chunks":bars.chunks(chunk).map(|c|json!({"candles":c})).collect::<Vec<_>>(),"interval":"1d","assumptions":{"initialCapitalKrw":1000000,"feeBps":0,"taxBps":0,"slippageBps":0,"executionModel":execution,"maxPositionRatio":1.0}})
}
fn run(v: Value) -> ContinuousStrategyPreviewView {
    preview_strategy_continuous_from_candles(serde_json::from_value(v).unwrap()).unwrap()
}
fn short(mut v: Value) -> StrategyPreviewInput {
    let rows = v["chunks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["candles"].as_array().unwrap().clone())
        .collect::<Vec<_>>();
    v.as_object_mut().unwrap().remove("chunks");
    v["candles"] = json!(rows);
    serde_json::from_value(v).unwrap()
}
#[test]
fn failed_breakout_optional_params_are_strict_in_both_previews_and_daily_filters_reject_minutes() {
    for extra in [
        json!({"breakout_basis":null}),
        json!({"breakout_basis":"bad"}),
        json!({"atr_buffer_multiplier":null}),
        json!({"atr_buffer_multiplier":21}),
        json!({"min_volume_ratio":101}),
        json!({"min_volume_ratio":"1"}),
        json!({"profit_activation_pct":1}),
        json!({"profit_activation_pct":0,"profit_trailing_pct":1}),
        json!({"profit_activation_pct":5,"profit_trailing_pct":100}),
        json!({"max_hold_observations":false}),
        json!({"max_hold_observations":0.5}),
        json!({"max_hold_observations":1000001}),
    ] {
        let value = request(extra, "nextOpen", &[11000, 12000], 2);
        assert_eq!(
            preview_strategy_continuous_from_candles(
                serde_json::from_value(value.clone()).unwrap()
            )
            .unwrap_err()
            .code,
            "INVALID_STRATEGY_PARAMS"
        );
        assert_eq!(
            preview_strategy_from_candles(short(value))
                .unwrap_err()
                .code,
            "INVALID_STRATEGY_PARAMS"
        );
    }
    for extra in [
        json!({"breakout_basis":"completedHigh"}),
        json!({"atr_buffer_multiplier":0.5}),
        json!({"min_volume_ratio":1.5}),
    ] {
        let mut value = request(extra, "nextOpen", &[11000, 12000], 2);
        value["interval"] = json!("W");
        assert_eq!(
            preview_strategy_continuous_from_candles(
                serde_json::from_value(value.clone()).unwrap()
            )
            .unwrap_err()
            .code,
            "UNSUPPORTED_ENTRY_FILTER_INTERVAL"
        );
        assert_eq!(
            preview_strategy_from_candles(short(value))
                .unwrap_err()
                .code,
            "UNSUPPORTED_ENTRY_FILTER_INTERVAL"
        );
    }
}
#[test]
fn failed_breakout_next_open_promotes_original_anchor_after_same_zero_feedback() {
    let value = request(json!({}), "nextOpen", &[11000, 12000, 9999, 9800], 1);
    let result = run(value.clone());
    let trades = &result.preview.backtest.trades;
    assert!(trades.iter().any(|t| t.side == "buy"
        && t.status == "filled"
        && t.time == "20261002"
        && t.fill_price == Some(12000.0)));
    assert!(result
        .preview
        .signals
        .iter()
        .any(|s| s.side == "sell" && s.time == "20261003" && s.reason.contains("전고점 10000")));
    assert!(trades.iter().any(|t| t.side == "sell"
        && t.status == "filled"
        && t.time == "20261004"
        && t.fill_price == Some(9800.0)));
    assert!(result.preview.entry_filter_preparation.is_none());
    let flat = preview_strategy_from_candles(short(value)).unwrap();
    assert_eq!(json!(flat.signals), json!(result.preview.signals));
    assert_eq!(json!(flat.backtest), json!(result.preview.backtest));
}
#[test]
fn failed_breakout_partition_and_future_suffix_are_causal_for_all_options() {
    let extras = [
        json!({}),
        json!({"breakout_basis":"completedHigh"}),
        json!({"atr_buffer_multiplier":0.5}),
        json!({"min_volume_ratio":1}),
        json!({"profit_activation_pct":5,"profit_trailing_pct":3}),
        json!({"max_hold_observations":2.0}),
    ];
    for extra in extras {
        for execution in ["sameClose", "nextOpen"] {
            let prices = [11000, 12000, 12500, 12000, 9999, 9800, 11000, 12000];
            let a = run(request(extra.clone(), execution, &prices, 8));
            let b = run(request(extra.clone(), execution, &prices, 1));
            assert_eq!(json!(a.preview.signals), json!(b.preview.signals));
            assert_eq!(json!(a.preview.backtest), json!(b.preview.backtest));
            assert_eq!(a.preview.replay.input_hash, b.preview.replay.input_hash);
            let mut changed = prices;
            changed[5..].copy_from_slice(&[100, 100000, 2]);
            let c = run(request(extra.clone(), execution, &changed, 2));
            let prefix = |values: Vec<Value>| {
                values
                    .into_iter()
                    .filter(|v| v["time"].as_str().unwrap() <= "20261005")
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                prefix(a.preview.signals.iter().map(|s| json!(s)).collect()),
                prefix(c.preview.signals.iter().map(|s| json!(s)).collect())
            );
            assert_eq!(
                prefix(a.preview.backtest.trades.iter().map(|s| json!(s)).collect()),
                prefix(c.preview.backtest.trades.iter().map(|s| json!(s)).collect())
            );
        }
    }
}
#[test]
fn failed_breakout_candidate_high_low_volume_cannot_influence_its_own_entry() {
    for extra in [
        json!({"breakout_basis":"completedHigh"}),
        json!({"atr_buffer_multiplier":0.5}),
        json!({"min_volume_ratio":1}),
    ] {
        let a = request(extra, "sameClose", &[11000], 1);
        let mut b = a.clone();
        b["chunks"][0]["candles"][0]["high"] = json!("999999");
        b["chunks"][0]["candles"][0]["low"] = json!("1");
        b["chunks"][0]["candles"][0]["volume"] = json!("9999999999");
        let a = run(a);
        let b = run(b);
        assert_eq!(json!(a.preview.signals), json!(b.preview.signals));
        assert_eq!(a.preview.signals.len(), 1);
    }
}
#[test]
fn failed_breakout_raw_unknown_volume_is_not_recovered_from_lossy_zero() {
    for (raw, status, buy) in [
        ("", "unverifiedVolume", false),
        ("1.0", "unverifiedVolume", false),
        ("-1", "unverifiedVolume", false),
        ("18446744073709551616", "unverifiedVolume", false),
        ("0", "ready", false),
        ("150", "ready", true),
    ] {
        let mut v = request(json!({"min_volume_ratio":1.5}), "sameClose", &[11000], 1);
        v["historyCandles"][20]["volume"] = json!(raw);
        let result = run(v);
        let p = result.preview.entry_filter_preparation.unwrap();
        // The current completed bar has entered the context after the decision; counters preserve that decision.
        assert_eq!(result.preview.signals.len(), usize::from(buy));
        assert_eq!(p.unready_candidates, u64::from(status != "ready"));
        assert_eq!(p.volume_rejections, Some(u64::from(raw == "0")));
    }
}
#[test]
fn failed_breakout_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average() {
    let mut value = request(
        json!({"max_hold_observations":1}),
        "nextOpen",
        &[11000, 12000, 12005, 12500, 13000],
        1,
    );
    value["chunks"][1]["candles"][0]["open"] = json!("11990");
    value["chunks"][1]["candles"][0]["low"] = json!("11989");
    value["assumptions"]["feeBps"] = json!(10);
    value["chunks"][3]["candles"][0]["open"] = json!("12010");
    value["chunks"][3]["candles"][0]["low"] = json!("12009");
    let result = run(value);
    let exits = result
        .preview
        .signals
        .iter()
        .filter(|s| s.side == "sell")
        .collect::<Vec<_>>();
    assert_eq!(
        exits.iter().map(|s| s.time.as_str()).collect::<Vec<_>>(),
        ["20261002", "20261003", "20261004"]
    );
    assert!(exits.iter().all(|s| s.quantity == 2));
    let blocked = result
        .preview
        .backtest
        .trades
        .iter()
        .filter(|t| t.side == "sell" && t.status == "blocked")
        .collect::<Vec<_>>();
    assert!(blocked.len() >= 2, "{:?}", result.preview.backtest.trades);
    assert!(blocked[0]
        .blocked_reason
        .as_deref()
        .unwrap()
        .contains("기대 순익"));
    assert!(result
        .preview
        .backtest
        .trades
        .iter()
        .any(|t| t.side == "sell" && t.status == "filled" && t.fill_price == Some(13000.0)));
}
#[test]
fn failed_breakout_profit_arms_on_actual_avg_and_reoffers_after_actual_guard_refusal() {
    let mut value = request(
        json!({"profit_activation_pct":5,"profit_trailing_pct":3}),
        "nextOpen",
        &[11000, 12600, 13000, 12500, 12400, 12000, 13000],
        1,
    );
    // Entry fills at12000; close12600 arms. Peak13000 -> trailing threshold12610.
    value["chunks"][1]["candles"][0]["open"] = json!("12000");
    value["chunks"][1]["candles"][0]["low"] = json!("11999");
    // Pending profit sale first encounters tiny positive net-unprofitable next open.
    value["chunks"][4]["candles"][0]["open"] = json!("12001");
    value["chunks"][4]["candles"][0]["low"] = json!("12000");
    value["assumptions"]["feeBps"] = json!(10);
    let result = run(value);
    let sells = result
        .preview
        .signals
        .iter()
        .filter(|s| s.side == "sell")
        .collect::<Vec<_>>();
    assert!(sells.len() >= 2, "{:?}", result.preview.signals);
    assert_eq!(sells[0].time, "20261004");
    assert_eq!(sells[1].time, "20261005");
    assert!(sells
        .iter()
        .all(|s| s.reason.contains("수익 보호") && s.quantity == 2));
    assert!(result
        .preview
        .backtest
        .trades
        .iter()
        .any(|t| t.side == "sell" && t.status == "blocked" && t.time == "20261005"));
}

#[test]
fn failed_breakout_hash_distinguishes_unknown_daily_volume_from_verified_zero() {
    let mut v = request(json!({"min_volume_ratio":1.5}), "sameClose", &[11000], 1);
    v["historyCandles"][20]["volume"] = json!("");
    let mut zero = v.clone();
    zero["historyCandles"][20]["volume"] = json!("0");
    let unknown = run(v);
    let zero = run(zero);
    assert_ne!(
        unknown.preview.replay.input_hash,
        zero.preview.replay.input_hash
    );
    assert_eq!(
        unknown
            .preview
            .entry_filter_preparation
            .unwrap()
            .unready_candidates,
        1
    );
    assert_eq!(
        zero.preview
            .entry_filter_preparation
            .unwrap()
            .volume_rejections,
        Some(1)
    );
}

#[test]
fn failed_breakout_entry_atr_keeps_common_sizing_and_sells_actual_adjusted_quantity() {
    let mut value = request(
        json!({"atr_buffer_multiplier":0.5}),
        "nextOpen",
        &[11000, 11000, 9999, 9800],
        1,
    );
    value["orderQuantity"] = json!(100);
    value["assumptions"]["volatilitySizingEnabled"] = json!(true);
    value["assumptions"]["riskPerTradeBps"] = json!(100);
    value["assumptions"]["atrStopMultiplier"] = json!(2);
    let mut off = value.clone();
    off["params"]["atr_buffer_multiplier"] = json!(0);
    let a = run(value);
    let b = run(off);
    assert_eq!(json!(a.preview.backtest), json!(b.preview.backtest));
    let buy = a
        .preview
        .backtest
        .trades
        .iter()
        .find(|t| t.side == "buy" && t.status == "filled")
        .unwrap();
    assert!(buy.quantity > 0 && buy.quantity < 100);
    let sells = a
        .preview
        .signals
        .iter()
        .filter(|s| s.side == "sell")
        .collect::<Vec<_>>();
    assert!(!sells.is_empty());
    assert!(sells.iter().all(|s| s.quantity == buy.quantity));
    assert!(a.preview.backtest.atr_preparation.ready_at_start);
}
