//! Synthetic production driver tests; actual accounts and provider calls are absent.
use super::*;
use serde_json::{json, Value};
fn bar(date: String, price: u64) -> Value {
    json!({"date":date,"open":price.to_string(),"high":(price+1).to_string(),"low":(price-1).to_string(),"close":price.to_string(),"volume":"100"})
}
fn request(extra: Value, execution: &str, prices: &[u64], chunk: usize) -> Value {
    let mut bars = prices
        .iter()
        .enumerate()
        .map(|(i, p)| bar(format!("202610{:02}", i + 1), *p))
        .collect::<Vec<_>>();
    for b in &mut bars {
        b["high"] = json!((b["open"].as_str().unwrap().parse::<u64>().unwrap() + 100).to_string());
        b["low"] = json!((b["open"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            .saturating_sub(100)
            .max(1))
        .to_string());
        b["open"] = json!((b["close"].as_str().unwrap().parse::<u64>().unwrap() - 1).to_string());
    }
    let mut params = json!({"lookback_days":2,"expansion_factor":2,"stop_loss_pct":3});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    json!({"strategyId":"volatility_expansion_fixture","strategyName":"fixture","symbol":"TEST","isOverseas":false,"orderQuantity":2,"params":params,"historyCandles":(1..=21).map(|i|bar(format!("202609{i:02}"),10000)).collect::<Vec<_>>(),"chunks":bars.chunks(chunk).map(|c|json!({"candles":c})).collect::<Vec<_>>(),"interval":"1d","assumptions":{"initialCapitalKrw":1000000,"feeBps":0,"taxBps":0,"slippageBps":0,"executionModel":execution,"maxPositionRatio":1.0}})
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
fn volatility_expansion_cap_reoffers_after_actual_guard_refusal_and_keeps_actual_average() {
    let mut value = request(
        json!({"max_hold_observations":1}),
        "nextOpen",
        &[11000, 12000, 12005, 12500, 13000],
        1,
    );
    value["chunks"][1]["candles"][0]["open"] = json!("11990");
    value["chunks"][1]["candles"][0]["low"] = json!("11989");
    value["assumptions"]["feeBps"] = json!(10);
    value["chunks"][4]["candles"][0]["open"] = json!("13000");
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
fn volatility_expansion_profit_arms_on_actual_avg_and_reoffers_after_actual_guard_refusal() {
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
fn volatility_expansion_partition_and_future_suffix_are_causal_for_all_options() {
    let extras = [
        json!({}),
        json!({"lookback_days":20}),
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
            assert_eq!(
                json!(a.preview.volatility_expansion_preparation),
                json!(b.preview.volatility_expansion_preparation)
            );
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
fn volatility_expansion_optional_validation_both_previews_and_daily_only_preserved() {
    for extra in [
        json!({"lookback_days":null}),
        json!({"lookback_days":0}),
        json!({"lookback_days":false}),
        json!({"lookback_days":0.5}),
        json!({"lookback_days":513}),
        json!({"expansion_factor":"2"}),
        json!({"expansion_factor":1001}),
        json!({"stop_loss_pct":null}),
        json!({"stop_loss_pct":100}),
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
    assert!(
        preview_strategy_continuous_from_candles(serde_json::from_value(v.clone()).unwrap())
            .is_err()
    );
    assert!(preview_strategy_from_candles(short(v)).is_err());
}
#[test]
fn volatility_expansion_common_atr_sizing_remains_and_sells_confirmed_adjusted_quantity() {
    let mut v = request(json!({}), "nextOpen", &[11000, 11000, 9999, 9800], 1);
    v["orderQuantity"] = json!(100);
    v["assumptions"]["volatilitySizingEnabled"] = json!(true);
    v["assumptions"]["riskPerTradeBps"] = json!(10);
    v["assumptions"]["atrStopMultiplier"] = json!(2);
    // Actual daily range gives a meaningful risk-limited quantity below the request.
    for b in v["historyCandles"].as_array_mut().unwrap() {
        b["high"] = json!("10010");
        b["low"] = json!("9990");
    }
    let a = run(v);
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
}

#[test]
fn volatility_expansion_diagnostic_serialization_and_other_strategy_omission() {
    let v = request(json!({}), "nextOpen", &[11000, 12000, 13000], 1);
    let a = run(v.clone());
    let p = a.preview.volatility_expansion_preparation.as_ref().unwrap();
    assert!(p.ready);
    assert_eq!(p.status, "ready");
    assert_eq!(p.cadence, "dailyCloseRange");
    assert_eq!(p.required_completed_bars, 2);
    assert_eq!(p.evaluated_observations, 3);
    assert_eq!(
        p.evaluated_observations,
        p.owned_observations + p.candidate_observations
    );
    assert_eq!(
        p.candidate_observations,
        p.unready_candidates + p.range_rejections + p.direction_rejections + p.passed_candidates
    );
    assert!(p.passed_candidates > 0);
    assert!(a.preview.entry_filter_preparation.is_none());
    let value = json!(a.preview);
    assert!(value["volatilityExpansionPreparation"].is_object());
    assert!(value.get("volatility_expansion_preparation").is_none());
    let mut other = v;
    other["strategyId"] = json!("ma_cross_fixture");
    other["params"] = json!({"short_period":1,"long_period":2});
    assert!(json!(run(other).preview)
        .get("volatilityExpansionPreparation")
        .is_none());
}

#[test]
fn volatility_expansion_daily_full_range_is_current_input_while_prior_mean_excludes_it() {
    let mut small = request(json!({}), "sameClose", &[11000], 1);
    small["chunks"][0]["candles"][0]["open"] = json!("10999");
    small["chunks"][0]["candles"][0]["low"] = json!("10999");
    small["chunks"][0]["candles"][0]["high"] = json!("11003");
    let mut expanded = small.clone();
    expanded["chunks"][0]["candles"][0]["high"] = json!("11004");
    let a = run(small);
    let b = run(expanded);
    assert!(a.preview.signals.is_empty());
    assert_eq!(b.preview.signals.len(), 1);
    assert_eq!(
        a.preview
            .volatility_expansion_preparation
            .unwrap()
            .range_rejections,
        1
    );
    assert_eq!(
        b.preview
            .volatility_expansion_preparation
            .unwrap()
            .passed_candidates,
        1
    );
}
