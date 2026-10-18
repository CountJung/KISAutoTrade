use super::*;
use chrono::{Duration, NaiveDate};
use serde_json::{json, Value};

fn candle(time: &str, price: f64) -> ChartCandle {
    ChartCandle {
        date: time.into(),
        open: price.to_string(),
        high: price.to_string(),
        low: price.to_string(),
        close: price.to_string(),
        volume: "1000".into(),
    }
}
fn daily(prices: &[f64], start: &str) -> Vec<ChartCandle> {
    let start = NaiveDate::parse_from_str(start, "%Y%m%d").unwrap();
    prices
        .iter()
        .enumerate()
        .map(|(i, p)| {
            candle(
                &(start + Duration::days(i as i64))
                    .format("%Y%m%d")
                    .to_string(),
                *p,
            )
        })
        .collect()
}
fn request(chunks: Vec<Vec<ChartCandle>>, interval: &str) -> ContinuousStrategyPreviewInput {
    serde_json::from_value(json!({
        "strategyId":"price_condition_continuous", "strategyName":"continuous fixture",
        "symbol":"TEST", "isOverseas":false, "orderQuantity":1,
        "params":{"symbols":[{"symbol":"TEST","quantity":1,"buy_trigger_price":1000,"sell_trigger_price":1200,"take_profit_pct":0,"stop_loss_pct":3,"is_overseas":false}]},
        "chunks":chunks.into_iter().map(|candles|json!({"candles":candles})).collect::<Vec<_>>(),
        "interval":interval,"dataSource":"syntheticContinuousFixture", "assumptions":{
            "initialCapitalKrw":1000000,"feeBps":0,"taxBps":0,"slippageBps":0,"maxPositionRatio":1,"inSamplePercent":100
        }
    })).unwrap()
}
fn report_json(preview: &StrategyPreviewView) -> Value {
    json!({"signals":preview.signals,"backtest":preview.backtest,"preparation":preview.preparation})
}
fn flat_input(input: ContinuousStrategyPreviewInput) -> StrategyPreviewInput {
    StrategyPreviewInput {
        strategy_id: input.strategy_id,
        strategy_name: input.strategy_name,
        symbol: input.symbol,
        is_overseas: input.is_overseas,
        order_quantity: input.order_quantity,
        params: input.params,
        candles: input
            .chunks
            .into_iter()
            .flat_map(|chunk| chunk.candles)
            .collect(),
        history_candles: input.history_candles,
        warmup_count: None,
        interval: input.interval,
        data_source: input.data_source,
        strategy_version: input.strategy_version,
        broker_id: input.broker_id,
        broker_account_id: input.broker_account_id,
        assumptions: input.assumptions,
    }
}

#[test]
fn actual_strategy_single_and_chunk_supply_match_and_pending_orders_cross_both_boundaries() {
    let bars = daily(
        &[900., 1000., 1300., 1300., 900., 1000., 1300., 1300.],
        "20261001",
    );
    let mut one = request(vec![bars.clone()], "1d");
    one.assumptions.execution_model = crate::trading::simulation::ExecutionModel::NextOpen;
    let single = preview_strategy_continuous_from_candles(one).unwrap();
    let mut many = request(
        vec![bars[..1].to_vec(), bars[1..3].to_vec(), bars[3..].to_vec()],
        "1d",
    );
    many.assumptions.execution_model = crate::trading::simulation::ExecutionModel::NextOpen;
    let split = preview_strategy_continuous_from_candles(many).unwrap();
    assert_eq!(report_json(&single.preview), report_json(&split.preview));
    assert_eq!(
        single.preview.replay.input_hash,
        split.preview.replay.input_hash
    );
    assert!(split.continuity.chunks[0].pending_at_end);
    assert_eq!(split.continuity.chunks[0].end_quantity, 0);
    assert_eq!(split.continuity.chunks[1].start_quantity, 0);
    assert_eq!(split.continuity.chunks[1].end_quantity, 1);
    assert!(split.continuity.chunks[1].pending_at_end);
    assert_eq!(split.continuity.chunks[2].start_quantity, 1);
    assert!(split.preview.backtest.summary.completed_trades > 0);
    assert_eq!(
        split.preview.backtest.trades[0]
            .fill_at
            .as_ref()
            .unwrap()
            .bar_time,
        "20261002"
    );
    assert_eq!(
        split.preview.backtest.trades[1]
            .fill_at
            .as_ref()
            .unwrap()
            .bar_time,
        "20261004"
    );
}

#[test]
fn flat_api_preserves_report_and_bounds_but_hash_identifies_calculation_mode() {
    let bars = daily(&[900., 1300., 900., 1300.], "20261001");
    let flat =
        preview_strategy_from_candles(flat_input(request(vec![bars.clone()], "1d"))).unwrap();
    let continuous = preview_strategy_continuous_from_candles(request(vec![bars], "1d")).unwrap();
    assert_eq!(report_json(&flat), report_json(&continuous.preview));
    assert_ne!(flat.replay.input_hash, continuous.preview.replay.input_hash);
    let oversized = daily(&vec![900.; 501], "20200101");
    assert_eq!(
        preview_strategy_from_candles(flat_input(request(vec![oversized], "1d")))
            .unwrap_err()
            .code,
        "TOO_MANY_CANDLES"
    );
}

#[test]
fn actual_indicator_warmup_and_state_survive_more_than_500_bars_and_arbitrary_partitions() {
    let bars = daily(
        &(0..720)
            .map(|i| 1000. + (i % 20) as f64 * 20.)
            .collect::<Vec<_>>(),
        "20240101",
    );
    let history = daily(&vec![1000.; 30], "20231201");
    let prepare = |chunks| {
        let mut r = request(chunks, "1d");
        r.strategy_id = "momentum_continuous".into();
        r.params = json!({"lookback_period":3,"threshold_pct":1.0});
        r.history_candles = history.clone();
        r
    };
    let a = preview_strategy_continuous_from_candles(prepare(
        bars.chunks(500).map(<[_]>::to_vec).collect(),
    ))
    .unwrap();
    let b = preview_strategy_continuous_from_candles(prepare(
        bars.chunks(91).map(<[_]>::to_vec).collect(),
    ))
    .unwrap();
    // Independent single supply to the common private driver, without weakening API bounds.
    let input = flat_input(prepare(vec![]));
    let history_rows =
        super::super::generic::normalize_ordered_rows(history, false, false).unwrap();
    let rows = super::super::generic::normalize_ordered_rows(bars, false, false).unwrap();
    let (reference, _) = super::super::driver::run_prepared_preview(
        input,
        "TEST".into(),
        history_rows,
        vec![rows],
        "continuous",
    )
    .unwrap();
    assert_eq!(report_json(&a.preview), report_json(&b.preview));
    assert_eq!(report_json(&a.preview), report_json(&reference));
    assert_eq!(a.preview.replay.input_hash, b.preview.replay.input_hash);
    assert_eq!(a.preview.replay.input_hash, reference.replay.input_hash);
    assert_eq!(a.preview.preparation.provided_history_bars, 30);
    assert_eq!(a.preview.backtest.equity_curve.len(), 720);
    assert!(a.preview.backtest.summary.filled_order_count > 0);
}

#[test]
fn guard_cooldown_is_not_reset_at_chunk_boundary() {
    let bars = [
        candle("2026-07-01T00:00:00Z", 900.),
        candle("2026-07-01T00:01:00Z", 1300.),
        candle("2026-07-01T00:02:00Z", 900.),
        candle("2026-07-02T00:00:00Z", 900.),
    ];
    let report = preview_strategy_continuous_from_candles(request(
        vec![bars[..2].to_vec(), bars[2..].to_vec()],
        "1m",
    ))
    .unwrap();
    assert_eq!(report.preview.backtest.trades[2].status, "blocked");
    assert!(report.preview.backtest.trades[2]
        .blocked_reason
        .as_ref()
        .unwrap()
        .contains("쿨다운"));
    assert_eq!(report.preview.backtest.trades[3].status, "filled");
}

#[test]
fn daily_loss_state_crosses_chunks_then_resets_on_the_next_market_date() {
    let bars = [
        candle("2026-07-01T13:00:00Z", 9.),
        candle("2026-07-01T13:01:00Z", 13.),
        candle("2026-07-01T14:31:00Z", 9.),
        candle("2026-07-02T13:00:00Z", 9.),
    ];
    let mut r = request(vec![bars[..2].to_vec(), bars[2..].to_vec()], "1m");
    r.is_overseas = true;
    r.params = json!({"symbols":[{"symbol":"TEST","quantity":1,"buy_trigger_price":10,"sell_trigger_price":12,"take_profit_pct":0,"stop_loss_pct":0,"is_overseas":true}]});
    r.assumptions.exchange_rate_krw = 1000.;
    r.assumptions.daily_loss_limit_krw = 1000.;
    r.assumptions.fx_scenario = vec![crate::trading::simulation::FxScenarioPoint {
        bar_time: "2026-07-01T13:01:00Z".into(),
        rate_krw: 500.,
    }];
    let report = preview_strategy_continuous_from_candles(r).unwrap();
    let trades = &report.preview.backtest.trades;
    assert_eq!(trades[1].realized_pnl_krw, Some(-2500.));
    assert_eq!(trades[2].status, "blocked");
    assert!(trades[2]
        .blocked_reason
        .as_ref()
        .unwrap()
        .contains("리스크"));
    assert_eq!(trades[3].status, "filled");
}

#[test]
fn changing_future_chunk_prices_preserves_all_earlier_strategy_fills_and_equity() {
    use crate::trading::simulation::{report_position_snapshot, ExecutionModel};
    let bars = daily(&[900., 1300., 900., 1300., 900., 1300.], "20261001");
    for model in [ExecutionModel::SameClose, ExecutionModel::NextOpen] {
        let prepare = |tail| {
            let mut input = request(vec![bars[..3].to_vec(), tail], "1d");
            input.assumptions.execution_model = model;
            input
        };
        let original =
            preview_strategy_continuous_from_candles(prepare(bars[3..].to_vec())).unwrap();
        let changed = preview_strategy_continuous_from_candles(prepare(daily(
            &[50000., 20., 80000.],
            "20261004",
        )))
        .unwrap();
        assert_eq!(
            json!(original.preview.backtest.equity_curve[..3]),
            json!(changed.preview.backtest.equity_curve[..3])
        );
        let prefix = |view: &StrategyPreviewView| {
            json!(view
                .backtest
                .trades
                .iter()
                .filter(|t| t.time.as_str() <= "20261003")
                .collect::<Vec<_>>())
        };
        assert_eq!(prefix(&original.preview), prefix(&changed.preview));
        let signals = |view: &StrategyPreviewView| {
            json!(view
                .signals
                .iter()
                .filter(|t| t.time.as_str() <= "20261003")
                .collect::<Vec<_>>())
        };
        assert_eq!(signals(&original.preview), signals(&changed.preview));
        let position = |view: &StrategyPreviewView| {
            let mut report = view.backtest.clone();
            report
                .trades
                .retain(|trade| trade.time.as_str() <= "20261003");
            report_position_snapshot(&report, false)
        };
        assert_eq!(position(&original.preview), position(&changed.preview));
        let left = &original.continuity.chunks[0];
        let right = &changed.continuity.chunks[0];
        assert_eq!(left.end_quantity, right.end_quantity);
        assert_eq!(left.end_quantity, position(&original.preview).0);
        assert_eq!(left.pending_at_end, right.pending_at_end);
        // This prefix ends with an actual queued buy in next-open mode. Its
        // later fill price may change, but its earlier presence/position cannot.
        assert_eq!(left.pending_at_end, model == ExecutionModel::NextOpen);
    }
}

#[test]
fn strict_order_rejects_inside_and_between_chunk_overlap_and_future_history() {
    let bars = daily(&[900., 1000., 1300.], "20261001");
    let reversed = vec![bars[1].clone(), bars[0].clone()];
    assert_eq!(
        preview_strategy_continuous_from_candles(request(vec![reversed], "1d"))
            .unwrap_err()
            .code,
        "NON_MONOTONIC_CANDLES"
    );
    for chunks in [
        vec![bars[..2].to_vec(), bars[1..].to_vec()],
        vec![bars[2..].to_vec(), bars[..2].to_vec()],
    ] {
        assert_eq!(
            preview_strategy_continuous_from_candles(request(chunks, "1d"))
                .unwrap_err()
                .code,
            "CONTINUOUS_TIME_OVERLAP"
        );
    }
    let mut r = request(vec![bars.clone()], "1d");
    r.history_candles = vec![bars[0].clone()];
    assert_eq!(
        preview_strategy_continuous_from_candles(r)
            .unwrap_err()
            .code,
        "CONTINUOUS_TIME_OVERLAP"
    );
    let duplicate = vec![
        candle("2026-07-01T13:00:00Z", 900.),
        candle("2026-07-01T22:00:00+09:00", 900.),
    ];
    assert_eq!(
        preview_strategy_continuous_from_candles(request(vec![duplicate], "1m"))
            .unwrap_err()
            .code,
        "NON_MONOTONIC_CANDLES"
    );
}

#[test]
fn bounded_request_rejects_empty_or_oversized_chunks_history_and_total() {
    for chunks in [
        vec![],
        vec![vec![]],
        vec![daily(&vec![900.; 501], "20200101")],
        vec![vec![candle("20260101", 900.)]; 17],
    ] {
        assert_eq!(
            preview_strategy_continuous_from_candles(request(chunks, "1d"))
                .unwrap_err()
                .code,
            "INVALID_CONTINUOUS_BOUNDS"
        );
    }
    let bars = daily(&vec![900.; 4097], "20000101");
    assert_eq!(
        preview_strategy_continuous_from_candles(request(
            bars.chunks(500).map(<[_]>::to_vec).collect(),
            "1d"
        ))
        .unwrap_err()
        .code,
        "TOO_MANY_CONTINUOUS_CANDLES"
    );
    let mut r = request(vec![vec![candle("20261001", 900.)]], "1d");
    r.history_candles = daily(&vec![900.; 513], "20200101");
    assert_eq!(
        preview_strategy_continuous_from_candles(r)
            .unwrap_err()
            .code,
        "INVALID_CONTINUOUS_BOUNDS"
    );
}

#[test]
fn max_evaluation_and_history_are_accepted_once_and_serialization_is_flattened() {
    let bars = daily(&vec![900.; 4096], "20100101");
    let mut r = request(bars.chunks(500).map(<[_]>::to_vec).collect(), "1d");
    r.history_candles = daily(&vec![900.; 512], "20000101");
    let started = std::time::Instant::now();
    let view = preview_strategy_continuous_from_candles(r).unwrap();
    eprintln!(
        "4096-bar continuous production strategy completed in {:?}",
        started.elapsed()
    );
    assert_eq!(view.preview.backtest.equity_curve.len(), 4096);
    assert_eq!(view.preview.replay.warmup_count, 512);
    assert_eq!(view.continuity.evaluation_bars, 4096);
    assert_eq!(view.continuity.chunks.len(), 9);
    let value = serde_json::to_value(view).unwrap();
    assert!(value.get("backtest").is_some());
    assert!(value.get("preview").is_none());
    assert_eq!(value["continuity"]["mode"], "continuous");
}

#[test]
fn final_chunk_validation_and_accounting_are_completed_before_any_execution() {
    let mut r = request(
        vec![vec![candle("20261001", 900.)], vec![candle("bad", 1300.)]],
        "1d",
    );
    assert_eq!(
        preview_strategy_continuous_from_candles(r)
            .unwrap_err()
            .code,
        "INVALID_CANDLE"
    );
    r = request(vec![vec![candle("20261001", 900.)]], "1d");
    r.assumptions.fx_scenario = vec![crate::trading::simulation::FxScenarioPoint {
        bar_time: "20261002".into(),
        rate_krw: 1000.,
    }];
    assert_eq!(
        preview_strategy_continuous_from_candles(r)
            .unwrap_err()
            .code,
        "INVALID_ACCOUNTING_SCENARIO"
    );
    let mut unsupported = request(vec![vec![candle("20261001", 900.)]], "1d");
    unsupported.strategy_id = "leveraged_trend_hold_x".into();
    assert_eq!(
        preview_strategy_continuous_from_candles(unsupported)
            .unwrap_err()
            .code,
        "UNSUPPORTED_GENERIC_REPLAY"
    );
}

#[test]
fn fx_native_cost_minimum_fee_and_atr_survive_partition_boundaries() {
    let bars = daily(&[9., 10., 13., 13., 9., 10., 13., 13.], "20261001");
    let prepare = |chunks| {
        let mut r = request(chunks, "1d");
        r.is_overseas = true;
        r.params = json!({"symbols":[{"symbol":"TEST","quantity":1,"buy_trigger_price":10,"sell_trigger_price":12,"take_profit_pct":0,"stop_loss_pct":3,"is_overseas":true}]});
        r.assumptions.exchange_rate_krw = 1000.;
        r.assumptions.minimum_fee_native = 0.5;
        r.assumptions.execution_model = crate::trading::simulation::ExecutionModel::NextOpen;
        r.assumptions.fx_scenario = vec![crate::trading::simulation::FxScenarioPoint {
            bar_time: "20261003".into(),
            rate_krw: 1200.,
        }];
        r.history_candles = (1..=15)
            .map(|day| {
                let mut b = candle(&format!("202609{day:02}"), 9.);
                b.high = "9.1".into();
                b.low = "8.9".into();
                b
            })
            .collect();
        r.assumptions.volatility_sizing_enabled = true;
        r.assumptions.risk_per_trade_bps = 10;
        r
    };
    let one = preview_strategy_continuous_from_candles(prepare(vec![bars.clone()])).unwrap();
    let split = preview_strategy_continuous_from_candles(prepare(
        bars.chunks(2).map(<[_]>::to_vec).collect(),
    ))
    .unwrap();
    assert_eq!(report_json(&one.preview), report_json(&split.preview));
    assert!(split.preview.backtest.atr_preparation.ready_at_start);
    assert!(split.preview.backtest.summary.filled_order_count > 0);
    assert!(split.preview.backtest.accounting.total_fee_krw > 0.);
}

#[test]
fn continuous_phase_warning_discloses_one_ledger_instead_of_independent_training() {
    let bars = daily(&[900., 1300., 900., 500.], "20261001");
    let mut r = request(vec![bars[..2].to_vec(), bars[2..].to_vec()], "1d");
    r.assumptions.in_sample_percent = 50;
    let report = preview_strategy_continuous_from_candles(r).unwrap();
    let warning = report.preview.backtest.overfit_warning.unwrap();
    assert!(warning.contains("연속 장부"));
    assert!(warning.contains("독립적인 학습/검증 표본으로 해석하지"));
}

#[test]
fn consecutive_realized_losses_preserve_the_strategy_symbol_block_across_chunks() {
    let bars = [
        candle("2026-07-01T13:00:00Z", 9.),
        candle("2026-07-01T13:01:00Z", 13.),
        candle("2026-07-01T14:31:00Z", 9.),
        candle("2026-07-01T14:32:00Z", 13.),
        candle("2026-07-01T16:02:00Z", 9.),
        candle("2026-07-01T16:03:00Z", 13.),
        candle("2026-07-01T17:33:00Z", 9.),
    ];
    let prepare = |chunks| {
        let mut r = request(chunks, "1m");
        r.is_overseas = true;
        r.params = json!({"symbols":[{"symbol":"TEST","quantity":1,"buy_trigger_price":10,"sell_trigger_price":12,"take_profit_pct":0,"stop_loss_pct":0,"is_overseas":true}]});
        r.assumptions.exchange_rate_krw = 1000.;
        r.assumptions.daily_loss_limit_krw = 100_000.;
        r.assumptions.fx_scenario = [
            ("2026-07-01T13:01:00Z", 500.),
            ("2026-07-01T14:32:00Z", 200.),
            ("2026-07-01T16:03:00Z", 100.),
        ]
        .into_iter()
        .map(|(time, rate)| crate::trading::simulation::FxScenarioPoint {
            bar_time: time.into(),
            rate_krw: rate,
        })
        .collect();
        r
    };
    let reference = preview_strategy_continuous_from_candles(prepare(vec![bars.to_vec()])).unwrap();
    let split = preview_strategy_continuous_from_candles(prepare(vec![
        bars[..6].to_vec(),
        bars[6..].to_vec(),
    ]))
    .unwrap();
    assert_eq!(report_json(&reference.preview), report_json(&split.preview));
    assert_eq!(split.preview.backtest.summary.completed_trades, 3);
    let last = split.preview.backtest.trades.last().unwrap();
    assert_eq!(last.status, "blocked");
    assert!(last.blocked_reason.as_ref().unwrap().contains("연속 손실"));
}
