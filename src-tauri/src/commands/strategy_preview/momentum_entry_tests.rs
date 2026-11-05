//! Production driver coverage, using synthetic prices without provider access.
use super::*;
use serde_json::{json, Value};

fn bar(date: String, price: u64) -> Value {
    json!({"date":date,"open":price.to_string(),"high":(price+1).to_string(),"low":(price-1).to_string(),"close":price.to_string(),"volume":"1"})
}
fn request(params: Value, execution: &str, prices: &[u64], chunk_size: usize) -> Value {
    let bars = prices
        .iter()
        .enumerate()
        .map(|(i, p)| bar(format!("202610{:02}", i + 1), *p))
        .collect::<Vec<_>>();
    json!({"strategyId":"momentum_fixture","strategyName":"fixture","symbol":"TEST","isOverseas":false,"orderQuantity":2,"params":params,"historyCandles":(1..=4).map(|i|bar(format!("202609{i:02}"),10000)).collect::<Vec<_>>(),"chunks":bars.chunks(chunk_size).map(|chunk|json!({"candles":chunk})).collect::<Vec<_>>(),"interval":"1d","assumptions":{"initialCapitalKrw":1000000,"feeBps":0,"taxBps":0,"slippageBps":0,"executionModel":execution,"maxPositionRatio":1.0}})
}
fn params() -> Value {
    json!({"lookback_period":3,"threshold_pct":5.0,"entry_mode":"crossing","entry_hysteresis_pct":1.0})
}
fn continuous(value: Value) -> ContinuousStrategyPreviewView {
    preview_strategy_continuous_from_candles(serde_json::from_value(value).unwrap()).unwrap()
}

#[test]
fn momentum_new_fields_are_strict_in_both_production_preview_paths() {
    for extra in [
        json!({"entry_mode":"unknown"}),
        json!({"entry_mode":null}),
        json!({"entry_mode":1}),
        json!({"entry_hysteresis_pct":"1"}),
        json!({"entry_hysteresis_pct":-1}),
        json!({"entry_hysteresis_pct":101}),
        json!({"entry_hysteresis_pct":6}),
    ] {
        let mut params = params();
        params
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let mut value = request(params, "nextOpen", &[10500, 10600], 2);
        let input = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(
            preview_strategy_continuous_from_candles(input)
                .unwrap_err()
                .code,
            "INVALID_STRATEGY_PARAMS"
        );
        let candles = value["chunks"][0]["candles"].take();
        value.as_object_mut().unwrap().remove("chunks");
        value["candles"] = candles;
        let input = serde_json::from_value(value).unwrap();
        assert_eq!(
            preview_strategy_from_candles(input).unwrap_err().code,
            "INVALID_STRATEGY_PARAMS"
        );
    }
}

#[test]
fn momentum_actual_strategy_batch_chunk_reports_match_and_future_suffix_is_causal() {
    let prices = [10500, 10600, 10700, 9000, 8900, 8800, 10000, 11000];
    for model in ["sameClose", "nextOpen"] {
        let batch = continuous(request(params(), model, &prices, 8));
        let chunked = continuous(request(params(), model, &prices, 1));
        assert_eq!(json!(batch.preview.signals), json!(chunked.preview.signals));
        assert_eq!(
            json!(batch.preview.backtest),
            json!(chunked.preview.backtest)
        );
        assert!(batch
            .preview
            .backtest
            .trades
            .iter()
            .any(|t| t.status == "filled"));
        let mut changed = prices;
        changed[5..].copy_from_slice(&[100, 100000, 2]);
        let altered = continuous(request(params(), model, &changed, 2));
        let prefix = |values: Vec<Value>| {
            values
                .into_iter()
                .filter(|v| v["time"].as_str().unwrap() <= "20261005")
                .collect::<Vec<_>>()
        };
        assert_eq!(
            prefix(batch.preview.signals.iter().map(|s| json!(s)).collect()),
            prefix(altered.preview.signals.iter().map(|s| json!(s)).collect())
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
                altered
                    .preview
                    .backtest
                    .trades
                    .iter()
                    .map(|s| json!(s))
                    .collect()
            )
        );
        assert_eq!(
            json!(&batch.preview.backtest.equity_curve[..5]),
            json!(&altered.preview.backtest.equity_curve[..5])
        );
    }
}

#[test]
fn momentum_guard_block_and_next_open_equal_zero_feedback_do_not_consume_retry() {
    for model in ["sameClose", "nextOpen"] {
        let mut value = request(params(), model, &[10500, 10600, 10700], 1);
        value["assumptions"]["initialCapitalKrw"] = json!(1);
        let result = continuous(value);
        assert_eq!(result.preview.signals.len(), 3);
        assert!(result.preview.signals.iter().all(|s| s.side == "buy"));
        assert!(!result
            .preview
            .backtest
            .trades
            .iter()
            .any(|t| t.status == "filled"));
    }
}
