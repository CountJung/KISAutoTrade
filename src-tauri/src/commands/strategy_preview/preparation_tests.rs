use super::{preview_strategy_from_candles, StrategyPreviewInput};
use crate::api::rest::ChartCandle;
use crate::trading::simulation::SimulationAssumptions;

fn rows(start: &str, count: usize) -> Vec<ChartCandle> {
    let start = chrono::NaiveDate::parse_from_str(start, "%Y%m%d").unwrap();
    (0..count)
        .map(|index| ChartCandle {
            date: (start + chrono::Duration::days(index as i64))
                .format("%Y%m%d")
                .to_string(),
            open: "100".into(),
            high: "101".into(),
            low: "99".into(),
            close: "100".into(),
            volume: "1000".into(),
        })
        .collect()
}

fn input(id: &str) -> StrategyPreviewInput {
    StrategyPreviewInput {
        strategy_id: id.into(),
        strategy_name: id.into(),
        symbol: "SOXQ".into(),
        is_overseas: true,
        order_quantity: 10,
        params: serde_json::json!({}),
        candles: rows("20260101", 5),
        history_candles: Vec::new(),
        warmup_count: None,
        interval: Some("1d".into()),
        data_source: Some("fixture".into()),
        strategy_version: None,
        broker_id: None,
        broker_account_id: None,
        assumptions: SimulationAssumptions::default(),
    }
}

const IDS: [&str; 13] = [
    "ma_cross",
    "rsi",
    "momentum",
    "deviation",
    "fifty_two_week_high",
    "consecutive_move",
    "failed_breakout",
    "strong_close",
    "volatility_expansion",
    "mean_reversion",
    "trend_filter",
    "price_condition",
    "leveraged_trend_hold",
];

#[test]
fn all_strategies_keep_the_same_evaluation_window_without_implicit_warmup() {
    for id in IDS {
        let preview = preview_strategy_from_candles(input(id)).unwrap();
        assert_eq!(preview.replay.data_start, "20260101", "{id}");
        assert_eq!(preview.replay.data_end, "20260105", "{id}");
        assert_eq!(preview.replay.warmup_count, 0, "{id}");
        assert_eq!(preview.candles.len(), 5, "{id}");
        assert_eq!(preview.backtest.equity_curve.len(), 5, "{id}");
    }
}

#[test]
fn separate_history_matches_explicit_legacy_prefix_and_hashes_history() {
    let mut separate = input("trend_filter");
    separate.history_candles = rows("20250101", 252);
    let baseline = preview_strategy_from_candles(separate).unwrap();
    let mut legacy = input("trend_filter");
    legacy.candles = rows("20250101", 252)
        .into_iter()
        .chain(legacy.candles)
        .collect();
    legacy.warmup_count = Some(252);
    let combined = preview_strategy_from_candles(legacy).unwrap();
    assert_eq!(baseline.replay.input_hash, combined.replay.input_hash);
    assert_eq!(baseline.candles.len(), 5);
    assert_eq!(combined.candles[0].date, "20260101");
    assert_eq!(
        serde_json::to_value(&baseline.signals).unwrap(),
        serde_json::to_value(&combined.signals).unwrap()
    );
    let mut changed = input("trend_filter");
    changed.history_candles = rows("20250101", 252);
    changed.history_candles[0].high = "102".into();
    assert_ne!(
        baseline.replay.input_hash,
        preview_strategy_from_candles(changed)
            .unwrap()
            .replay
            .input_hash
    );
}

#[test]
fn complete_daily_history_initializes_supported_strategies_without_shortening_defaults() {
    for id in IDS.into_iter().filter(|id| *id != "leveraged_trend_hold") {
        let mut fixture = input(id);
        fixture.history_candles = rows("20250101", 252);
        let preview = preview_strategy_from_candles(fixture).unwrap();
        assert_eq!(preview.preparation.ready_at_start, Some(true), "{id}");
        assert_eq!(preview.preparation.unready_evaluation_bars, 0, "{id}");
        if id == "trend_filter" {
            assert_eq!(preview.preparation.required_history_bars, Some(200));
        }
        if id == "fifty_two_week_high" {
            assert_eq!(preview.preparation.required_history_bars, Some(252));
        }
    }
}

#[test]
fn insufficient_history_is_not_reported_as_a_zero_trade_strategy() {
    for (id, required) in [("trend_filter", 200), ("fifty_two_week_high", 252)] {
        let mut fixture = input(id);
        fixture.history_candles = rows("20250101", 20);
        let preview = preview_strategy_from_candles(fixture).unwrap();
        let prep = preview.preparation;
        assert_eq!(prep.required_history_bars, Some(required));
        assert_eq!(prep.provided_history_bars, 20);
        assert_eq!(prep.ready_at_start, Some(false));
        assert_eq!(prep.ready_at_end, Some(false));
        assert_eq!(prep.unready_evaluation_bars, 5);
        assert_eq!(prep.history_status, "insufficient");
        assert_eq!(prep.indicator_status, "warmingUp");
        assert_eq!(prep.outcome, "notEvaluable");
    }
}

#[test]
fn first_ready_bar_is_observed_after_tick_without_discarding_evaluation_bars() {
    let mut fixture = input("mean_reversion");
    fixture.params = serde_json::to_value(crate::trading::strategy::MeanReversionParams {
        period: 3,
        ..Default::default()
    })
    .unwrap();
    let preview = preview_strategy_from_candles(fixture).unwrap();
    assert_eq!(preview.preparation.ready_at_start, Some(false));
    assert_eq!(preview.preparation.ready_at_end, Some(true));
    assert_eq!(
        preview.preparation.first_ready_time.as_deref(),
        Some("20260103")
    );
    assert_eq!(preview.preparation.unready_evaluation_bars, 2);
    assert_eq!(preview.preparation.outcome, "conditionsNotMet");
    assert_eq!(preview.preparation.evaluated_bars, Some(3));
    assert_eq!(preview.candles.len(), 5);
}

#[test]
fn unsupported_and_intraday_daily_indicators_do_not_claim_readiness() {
    let preview = preview_strategy_from_candles(input("leveraged_trend_hold")).unwrap();
    assert_eq!(preview.preparation.ready_at_start, None);
    assert_eq!(preview.preparation.history_status, "unsupported");
    assert_eq!(preview.preparation.outcome, "notEvaluable");
    assert!(!preview.replay.deterministic);
    assert_eq!(preview.preparation.evaluated_bars, None);
    assert_eq!(preview.preparation.unready_evaluation_bars, 0);
    for id in ["trend_filter", "fifty_two_week_high"] {
        let mut fixture = input(id);
        fixture.history_candles = rows("20250101", 252);
        fixture.interval = Some("1m".into());
        for row in fixture
            .candles
            .iter_mut()
            .chain(fixture.history_candles.iter_mut())
        {
            row.date.push_str("093000");
        }
        let preview = preview_strategy_from_candles(fixture).unwrap();
        assert_eq!(preview.preparation.provided_history_bars, 252);
        assert_eq!(preview.preparation.available_history_bars, Some(0), "{id}");
        assert_eq!(preview.preparation.ready_at_start, Some(false), "{id}");
    }
}

#[test]
fn invalid_or_ambiguous_history_is_rejected_without_swallowing_evaluation() {
    let assert_error = |fixture, code| {
        assert_eq!(
            preview_strategy_from_candles(fixture).unwrap_err().code,
            code
        );
    };
    let mut invalid = input("trend_filter");
    invalid.history_candles = rows("20250101", 2);
    invalid.history_candles[0].low = "102".into();
    assert_error(invalid, "INVALID_CANDLE");
    let mut bad_date = input("trend_filter");
    bad_date.candles[0].date = "20260230".into();
    assert_error(bad_date, "INVALID_CANDLE");
    let mut duplicate = input("trend_filter");
    duplicate.history_candles = vec![rows("20250101", 1)[0].clone(); 2];
    assert_error(duplicate, "DUPLICATE_CANDLE");
    let mut overlap = input("trend_filter");
    overlap.history_candles = rows("20260101", 1);
    assert_error(overlap, "HISTORY_OVERLAP");
    let mut ambiguous = input("trend_filter");
    ambiguous.history_candles = rows("20250101", 1);
    ambiguous.warmup_count = Some(1);
    assert_error(ambiguous, "AMBIGUOUS_HISTORY");
    let mut no_evaluation = input("trend_filter");
    no_evaluation.warmup_count = Some(5);
    assert_error(no_evaluation, "NO_EVALUATION_CANDLES");
    let mut too_many = input("trend_filter");
    too_many.history_candles = rows("20200101", 496);
    assert_error(too_many, "TOO_MANY_CANDLES");
    let mut mixed = input("trend_filter");
    mixed.interval = Some("1m".into());
    mixed.candles[0].date = "20260101093000".into();
    mixed.history_candles = rows("20260101", 1);
    assert_error(mixed, "INVALID_CANDLE");
}

#[test]
fn cross_and_breakout_seed_only_final_bars_are_not_conditions_not_met() {
    use crate::trading::strategy::{FailedBreakoutParams, MaCrossParams, RsiParams};
    for (id, params, length) in [
        (
            "ma_cross",
            serde_json::to_value(MaCrossParams {
                short_period: 2,
                long_period: 3,
            })
            .unwrap(),
            3,
        ),
        (
            "rsi",
            serde_json::to_value(RsiParams {
                period: 3,
                ..Default::default()
            })
            .unwrap(),
            4,
        ),
        (
            "failed_breakout",
            serde_json::to_value(FailedBreakoutParams {
                lookback_days: 3,
                ..Default::default()
            })
            .unwrap(),
            3,
        ),
    ] {
        let mut fixture = input(id);
        fixture.params = params;
        fixture.candles = rows("20260101", length);
        let result = preview_strategy_from_candles(fixture).unwrap();
        assert_eq!(result.preparation.ready_at_end, Some(true), "{id}");
        assert_eq!(result.preparation.evaluated_bars, Some(0), "{id}");
        assert_eq!(result.preparation.outcome, "notEvaluable", "{id}");
    }
}

#[test]
fn omitted_history_remains_compatible_and_preparation_uses_camel_case() {
    let fixture: StrategyPreviewInput = serde_json::from_value(serde_json::json!({
        "strategyId":"price_condition", "strategyName":"fixture", "symbol":"SOXQ",
        "isOverseas":true, "orderQuantity":10, "params":{}, "candles":rows("20260101",1)
    }))
    .unwrap();
    assert!(fixture.history_candles.is_empty());
    let result = serde_json::to_value(preview_strategy_from_candles(fixture).unwrap()).unwrap();
    assert_eq!(result["preparation"]["historyStatus"], "notRequired");
    assert_eq!(result["preparation"]["readyAtStart"], true);
    assert!(result["preparation"].get("history_status").is_none());
}

#[test]
fn unordered_trend_periods_use_the_largest_actual_window() {
    let mut fixture = input("trend_filter");
    fixture.params = serde_json::to_value(crate::trading::strategy::TrendFilterParams {
        long_period: 50,
        mid_period: 52,
        short_period: 5,
    })
    .unwrap();
    fixture.candles = rows("20260101", 60);
    let result = preview_strategy_from_candles(fixture).unwrap();
    assert_eq!(result.preparation.required_history_bars, Some(52));
    assert_eq!(result.preparation.ready_at_end, Some(true));
    assert_eq!(result.preparation.evaluated_bars, Some(9));
    assert_eq!(result.preparation.outcome, "conditionsNotMet");
}
