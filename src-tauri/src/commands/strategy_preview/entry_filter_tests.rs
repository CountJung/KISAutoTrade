use super::*;
use serde_json::json;

fn bar(date: &str, close: u64) -> ChartCandle {
    ChartCandle {
        date: date.into(),
        open: close.to_string(),
        high: (close + 1).to_string(),
        low: (close - 1).to_string(),
        close: close.to_string(),
        volume: "1".into(),
    }
}
fn input(params: serde_json::Value) -> StrategyPreviewInput {
    StrategyPreviewInput {
        strategy_id: "consecutive_move_entry".into(),
        strategy_name: "fixture".into(),
        symbol: "TEST".into(),
        is_overseas: false,
        order_quantity: 1,
        params,
        candles: vec![bar("20261001", 103), bar("20261002", 104)],
        history_candles: (1..=15)
            .map(|d| bar(&format!("202609{d:02}"), 100 + d))
            .collect(),
        warmup_count: None,
        interval: Some("1d".into()),
        data_source: Some("synthetic entry fixture".into()),
        strategy_version: None,
        broker_id: None,
        broker_account_id: None,
        assumptions: SimulationAssumptions {
            initial_capital_krw: 1_000_000.,
            fee_bps: 0.,
            tax_bps: 0.,
            slippage_bps: 0.,
            max_position_ratio: 1.,
            ..Default::default()
        },
    }
}
#[test]
fn invalid_filter_params_and_non_daily_ohlc_filter_requests_are_explicit_errors() {
    for (key, value) in [
        ("long_trend_period", json!(2.5)),
        ("long_trend_period", json!(0.5)),
        ("long_trend_period", json!(513)),
        ("min_move_atr_multiplier", json!(-1)),
        ("min_cumulative_move_pct", json!("1")),
    ] {
        let mut params = json!({"buy_days":2,"sell_days":4});
        params[key] = value;
        assert_eq!(
            preview_strategy_from_candles(input(params))
                .unwrap_err()
                .code,
            "INVALID_STRATEGY_PARAMS"
        );
    }
    for params in [
        json!({"buy_days":2,"sell_days":4,"min_move_atr_multiplier":0.5}),
        json!({"buy_days":2,"sell_days":4,"long_trend_period":200}),
    ] {
        let mut r = input(params);
        r.interval = Some("W".into());
        assert_eq!(
            preview_strategy_from_candles(r).unwrap_err().code,
            "UNSUPPORTED_ENTRY_FILTER_INTERVAL"
        );
    }
}
#[test]
fn integer_valued_json_float_periods_remain_enabled_in_production_preview() {
    for period in [200.0, 512.0] {
        let mut request = input(json!({"buy_days":2,"sell_days":4,"long_trend_period":period}));
        request.history_candles = vec![bar("20260929", 100), bar("20260930", 101)];
        let result = preview_strategy_from_candles(request).unwrap();
        let prep = result.entry_filter_preparation.unwrap();
        assert!(prep.enabled);
        assert_eq!(prep.required_completed_bars, period as usize);
        assert_eq!(prep.status, "insufficientHistory");
        assert!(prep.unready_candidates > 0);
    }
}
#[test]
fn default_diagnostic_is_omitted_while_threshold_and_preparation_failures_are_distinct() {
    let plain = preview_strategy_from_candles(input(json!({"buy_days":2,"sell_days":4}))).unwrap();
    assert!(serde_json::to_value(plain)
        .unwrap()
        .get("entryFilterPreparation")
        .is_none());
    let mut r = input(json!({"buy_days":2,"sell_days":4,"min_cumulative_move_pct":100}));
    r.history_candles = vec![bar("20260929", 100), bar("20260930", 101)];
    let filtered = preview_strategy_from_candles(r).unwrap();
    let p = filtered.entry_filter_preparation.unwrap();
    assert!(p.ready);
    assert!(p.candidates > 0);
    assert_eq!(p.candidates, p.min_move_rejections);
    assert_eq!(p.unready_candidates, 0);
    let mut r = input(json!({"buy_days":2,"sell_days":4,"long_trend_period":200}));
    r.history_candles = vec![bar("20260929", 100), bar("20260930", 101)];
    let unavailable = preview_strategy_from_candles(r).unwrap();
    let p = unavailable.entry_filter_preparation.unwrap();
    assert!(!p.ready);
    assert!(p.unready_candidates > 0);
    assert_eq!(p.min_move_rejections, 0);
    assert_eq!(unavailable.preparation.outcome, "notEvaluable");
}
#[test]
fn changed_candidate_ohlc_range_cannot_change_its_own_prior_atr_entry_filter() {
    let params = json!({"buy_days":2,"sell_days":4,"min_move_atr_multiplier":0.5});
    let mut left = input(params.clone());
    left.history_candles = vec![bar("20260930", 100); 15];
    // Distinct actual dates while keeping native prices identical.
    for (i, c) in left.history_candles.iter_mut().enumerate() {
        c.date = format!("202609{:02}", i + 1);
    }
    left.history_candles[14].close = "101".into();
    left.history_candles[14].open = "101".into();
    left.history_candles[14].high = "102".into();
    left.candles = vec![bar("20261001", 102)];
    let mut right = input(params);
    right.history_candles = left.history_candles.clone();
    right.candles = left.candles.clone();
    right.candles[0].high = "10000".into();
    right.candles[0].low = "1".into();
    let a = preview_strategy_from_candles(left).unwrap();
    let b = preview_strategy_from_candles(right).unwrap();
    assert_eq!(json!(a.signals), json!(b.signals));
    let a = a.entry_filter_preparation.unwrap();
    let b = b.entry_filter_preparation.unwrap();
    assert_eq!(
        (a.candidates, a.passed_candidates, a.atr_rejections),
        (b.candidates, b.passed_candidates, b.atr_rejections)
    );
}
#[test]
fn frozen_spy_default_cost_paths_preserve_predeclared_exit_dates() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let source: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("docs/research/strategy-review-20261018/input.json"))
            .unwrap(),
    )
    .unwrap();
    let rows = source["datasets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["symbol"] == "SPY")
        .unwrap()["candles"]
        .as_array()
        .unwrap();
    let history = rows[52..304].to_vec();
    let evaluation = rows[304..1056].to_vec();
    for (cost, first, second) in [(10, "20230804", "20250225"), (20, "20230807", "20250331")] {
        let request:ContinuousStrategyPreviewInput=serde_json::from_value(json!({"strategyId":"consecutive_move","strategyName":"fixture","symbol":"SPY","isOverseas":true,"orderQuantity":2,"params":{"buy_days":3,"sell_days":3},"historyCandles":history,"chunks":evaluation.chunks(500).map(|rows|json!({"candles":rows})).collect::<Vec<_>>(),"interval":"1d","assumptions":{"initialCapitalKrw":10000000,"exchangeRateKrw":1450,"executionModel":"nextOpen","feeBps":cost,"slippageBps":cost,"taxBps":0}})).unwrap();
        let result = preview_strategy_continuous_from_candles(request).unwrap();
        let sells = result
            .preview
            .backtest
            .trades
            .iter()
            .filter(|t| t.status == "filled" && t.side == "sell")
            .map(|t| t.time.as_str())
            .collect::<Vec<_>>();
        assert!(sells.contains(&first));
        assert!(sells.contains(&second));
    }
}
