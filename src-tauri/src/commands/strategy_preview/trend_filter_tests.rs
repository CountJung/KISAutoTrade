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
    let mut params = json!({"long_period":4,"short_period":1,"mid_period":2});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    json!({"strategyId":"trend_filter_fixture","strategyName":"fixture","symbol":"TEST","isOverseas":false,"orderQuantity":2,"params":params,"historyCandles":(1..=21).map(|i|bar(format!("202609{i:02}"),10000)).collect::<Vec<_>>(),"chunks":bars.chunks(chunk).map(|c|json!({"candles":c})).collect::<Vec<_>>(),"interval":"1d","assumptions":{"initialCapitalKrw":1000000,"feeBps":0,"taxBps":0,"slippageBps":0,"executionModel":execution,"maxPositionRatio":1.0}})
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
fn trend_filter_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average() {
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
fn trend_filter_profit_arms_on_actual_avg_and_reoffers_after_actual_guard_refusal() {
    let mut value = request(
        json!({"long_period":20,"profit_activation_pct":5,"profit_trailing_pct":3}),
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
    assert!(sells[..2].iter().all(|s| s.reason.contains("수익 보호")));
    assert!(sells.iter().all(|s| s.quantity == 2));
    assert!(result
        .preview
        .backtest
        .trades
        .iter()
        .any(|t| t.side == "sell" && t.status == "blocked" && t.time == "20261005"));
}

#[test]
fn trend_filter_partition_and_future_suffix_are_causal_for_all_options() {
    let extras = [
        json!({}),
        json!({"exit_confirm_observations":2}),
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
fn trend_filter_strict_validation_in_both_previews_and_minute_support() {
    for extra in [
        json!({"long_period":null}),
        json!({"short_period":"5"}),
        json!({"mid_period":0}),
        json!({"exit_confirm_observations":0}),
        json!({"exit_confirm_observations":0.5}),
        json!({"profit_activation_pct":5}),
        json!({"profit_trailing_pct":"3"}),
        json!({"max_hold_observations":false}),
        json!({"max_hold_observations":1000001}),
    ] {
        let v = request(extra, "nextOpen", &[11000, 12000], 2);
        assert_eq!(
            preview_strategy_continuous_from_candles(serde_json::from_value(v.clone()).unwrap())
                .unwrap_err()
                .code,
            "INVALID_STRATEGY_PARAMS"
        );
        assert_eq!(
            preview_strategy_from_candles(short(v)).unwrap_err().code,
            "INVALID_STRATEGY_PARAMS"
        );
    }
    let mut v = request(json!({}), "nextOpen", &[11000, 12000], 2);
    v["interval"] = json!("1m");
    for b in v["historyCandles"].as_array_mut().unwrap() {
        b["date"] = json!(format!("{}090000", b["date"].as_str().unwrap()));
    }
    for chunk in v["chunks"].as_array_mut().unwrap() {
        for b in chunk["candles"].as_array_mut().unwrap() {
            b["date"] = json!(format!("{}090000", b["date"].as_str().unwrap()));
        }
    }

    // 단일 관측을 분봉으로 제공해도 지원한다. 완료 일봉 전용 옵션을 추가하지 않았다.
    assert!(
        preview_strategy_continuous_from_candles(serde_json::from_value(v.clone()).unwrap())
            .is_ok()
    );
    assert!(preview_strategy_from_candles(short(v)).is_ok());
}
#[test]
fn trend_filter_confirmation_resets_on_recovery_and_chunk_preserves_consecutive_exits() {
    let prices = [11000, 12000, 9000, 13000, 9000, 8000, 7000];
    let a = run(request(
        json!({"exit_confirm_observations":2}),
        "nextOpen",
        &prices,
        1,
    ));
    let b = run(request(
        json!({"exit_confirm_observations":2}),
        "nextOpen",
        &prices,
        7,
    ));
    assert_eq!(json!(a.preview.backtest), json!(b.preview.backtest));
    let sells: Vec<_> = a
        .preview
        .signals
        .iter()
        .filter(|s| s.side == "sell")
        .collect();
    assert_eq!(sells[0].time, "20261006");
    assert!(sells[0].reason.contains("2연속"));
    assert!(a.continuity.chunks.iter().any(|c| c.pending_at_end));
}
