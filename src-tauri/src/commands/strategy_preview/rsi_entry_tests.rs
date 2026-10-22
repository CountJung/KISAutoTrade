//! Synthetic production replay contracts; no provider calls.
use super::*;
use serde_json::{json, Value};
fn bar(date: String, price: u64) -> Value {
    json!({"date":date,"open":price.to_string(),"high":(price+1).to_string(),"low":(price-1).to_string(),"close":price.to_string(),"volume":"1"})
}
fn request(params: Value, execution: &str, prices: &[u64], chunk: usize) -> Value {
    let bars = prices
        .iter()
        .enumerate()
        .map(|(i, p)| bar(format!("202610{:02}", i + 1), *p))
        .collect::<Vec<_>>();
    json!({"strategyId":"rsi_fixture","strategyName":"fixture","symbol":"TEST","isOverseas":false,"orderQuantity":2,"params":params,"historyCandles":[bar("20260901".into(),12000),bar("20260902".into(),11000),bar("20260903".into(),10000)],"chunks":bars.chunks(chunk).map(|c|json!({"candles":c})).collect::<Vec<_>>(),"interval":"1d","assumptions":{"initialCapitalKrw":1000000,"feeBps":0,"taxBps":0,"slippageBps":0,"executionModel":execution,"maxPositionRatio":1.0}})
}
fn params() -> Value {
    json!({"period":2,"oversold":30.0,"overbought":70.0})
}
fn continuous(value: Value) -> ContinuousStrategyPreviewView {
    preview_strategy_continuous_from_candles(serde_json::from_value(value).unwrap()).unwrap()
}
#[test]
fn rsi_optional_fields_are_strict_in_short_and_continuous_previews() {
    for extra in [
        json!({"long_trend_period":null}),
        json!({"long_trend_period":"2"}),
        json!({"long_trend_period":513}),
        json!({"long_trend_period":0.5}),
        json!({"max_hold_observations":false}),
        json!({"max_hold_observations":1000001}),
        json!({"max_hold_observations":0.5}),
        json!({"stop_loss_pct":null}),
        json!({"stop_loss_pct":100}),
        json!({"stop_loss_pct":-1}),
        json!({"stop_loss_pct":"3"}),
    ] {
        let mut p = params();
        p.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let mut value = request(p, "nextOpen", &[11000, 12000], 2);
        assert_eq!(
            preview_strategy_continuous_from_candles(
                serde_json::from_value(value.clone()).unwrap()
            )
            .unwrap_err()
            .code,
            "INVALID_STRATEGY_PARAMS"
        );
        let candles = value["chunks"][0]["candles"].take();
        value.as_object_mut().unwrap().remove("chunks");
        value["candles"] = candles;
        assert_eq!(
            preview_strategy_from_candles(serde_json::from_value(value).unwrap())
                .unwrap_err()
                .code,
            "INVALID_STRATEGY_PARAMS"
        );
    }
}
#[test]
fn rsi_partition_next_open_and_future_suffix_are_causal() {
    let prices = [11000, 12000, 12500, 11000, 10500, 11500, 12000, 12100];
    for execution in ["sameClose", "nextOpen"] {
        let batch = continuous(request(params(), execution, &prices, 8));
        let split = continuous(request(params(), execution, &prices, 1));
        assert_eq!(json!(batch.preview.signals), json!(split.preview.signals));
        assert_eq!(json!(batch.preview.backtest), json!(split.preview.backtest));
        assert!(batch.preview.backtest.summary.filled_order_count > 0);
        let mut changed = prices;
        changed[5..].copy_from_slice(&[100, 100000, 2]);
        let future = continuous(request(params(), execution, &changed, 2));
        let prefix = |values: Vec<Value>| {
            values
                .into_iter()
                .filter(|v| v["time"].as_str().unwrap() <= "20261005")
                .collect::<Vec<_>>()
        };
        assert_eq!(
            prefix(batch.preview.signals.iter().map(|s| json!(s)).collect()),
            prefix(future.preview.signals.iter().map(|s| json!(s)).collect())
        );
        assert_eq!(
            prefix(
                batch
                    .preview
                    .backtest
                    .trades
                    .iter()
                    .map(|s| json!(s))
                    .collect()
            ),
            prefix(
                future
                    .preview
                    .backtest
                    .trades
                    .iter()
                    .map(|s| json!(s))
                    .collect()
            )
        );
    }
}
#[test]
fn rsi_next_open_uses_real_fill_basis_and_stops_before_original_cross() {
    let mut p = params();
    p["stop_loss_pct"] = 3.into();
    let result = continuous(request(
        p,
        "nextOpen",
        &[11000, 12000, 11500, 11000, 10000, 10500],
        1,
    ));
    let trades = &result.preview.backtest.trades;
    assert!(trades
        .iter()
        .any(|t| t.side == "buy" && t.status == "filled" && t.fill_price == Some(12000.0)));
    assert!(result
        .preview
        .signals
        .iter()
        .any(|s| s.side == "sell" && s.reason.contains("손절") && s.time == "20261003"));
    assert!(trades
        .iter()
        .any(|t| t.side == "sell" && t.status == "filled" && t.fill_price == Some(11000.0)));
    assert!(result
        .preview
        .signals
        .iter()
        .filter(|s| s.side == "sell")
        .all(|s| s.quantity == 2));
}
#[test]
fn rsi_cap_counts_after_actual_next_open_fill_across_chunk_boundaries() {
    let mut p = params();
    p["max_hold_observations"] = 2.into();
    let result = continuous(request(
        p,
        "nextOpen",
        &[11000, 12000, 12100, 12200, 12300],
        1,
    ));
    let first = result
        .preview
        .signals
        .iter()
        .find(|s| s.side == "sell")
        .unwrap();
    assert_eq!(first.time, "20261003");
    assert!(first.reason.contains("2회"));
    let final_pending = continuous(request(params(), "nextOpen", &[11000], 1));
    assert_eq!(final_pending.preview.backtest.summary.filled_order_count, 0);
    assert_eq!(
        final_pending
            .preview
            .signals
            .iter()
            .filter(|s| s.side == "buy")
            .count(),
        1
    );
}
#[test]
fn rsi_filtered_cross_does_not_retry_and_default_has_no_flat_sell_or_held_buy() {
    let mut p = params();
    p["long_trend_period"] = 3.0.into();
    let rejected = continuous(request(p, "sameClose", &[11000, 12000, 13000], 1));
    assert!(rejected.preview.signals.is_empty()); // first prior mean11000 equality, crossing consumed
    let baseline = continuous(request(
        params(),
        "nextOpen",
        &[11000, 12000, 12500, 11000, 10500, 11500],
        1,
    ));
    let mut owned = false;
    for signal in &baseline.preview.signals {
        if signal.side == "buy" {
            assert!(!owned);
            owned = true;
        } else {
            assert!(owned);
            owned = false;
        }
    }
}

#[test]
fn rsi_cap_reoffers_after_actual_minimum_profit_guard_refusal() {
    let mut p = params();
    p["max_hold_observations"] = 1.into();
    let mut value = request(p, "nextOpen", &[11000, 12000, 12005, 12500, 13000], 1);
    // A small positive close return is insufficient after round-trip fees.
    value["chunks"][1]["candles"][0]["open"] = json!("11990");
    value["chunks"][1]["candles"][0]["low"] = json!("11989");
    value["assumptions"]["feeBps"] = json!(10);
    value["chunks"][3]["candles"][0]["open"] = json!("12010");
    value["chunks"][3]["candles"][0]["low"] = json!("12009");
    let result = continuous(value);
    let rejected = result
        .preview
        .backtest
        .trades
        .iter()
        .filter(|t| t.side == "sell" && t.status == "blocked")
        .collect::<Vec<_>>();
    assert!(rejected.len() >= 2, "{:?}", result.preview.backtest.trades);
    assert!(
        rejected[0]
            .blocked_reason
            .as_deref()
            .unwrap()
            .contains("기대 순익"),
        "{}",
        rejected[0].reason
    );
    assert_eq!(rejected[0].time, "20261003");
    assert_eq!(
        rejected[0].signal_available_at.as_ref().unwrap().bar_time,
        "20261002"
    );
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
    assert!(
        exits[0].reason.contains("1회")
            && exits[1].reason.contains("2회")
            && exits[2].reason.contains("3회")
    );
    assert!(exits.iter().all(|s| s.quantity == 2));
    assert!(result
        .preview
        .backtest
        .trades
        .iter()
        .any(|t| t.side == "sell"
            && t.status == "filled"
            && t.signal_available_at.as_ref().unwrap().bar_time == "20261004"
            && t.fill_price == Some(13000.0)));
}
