//! Synthetic PC contract fixtures: these bars are not market efficacy evidence.
//! The production factory/driver/session run without provider or account access.
use super::*;
use crate::trading::simulation::{ExecutionModel, FxScenarioPoint};
use serde_json::{json, Value};

fn bar(time: &str, open: f64, close: f64) -> ChartCandle {
    ChartCandle {
        date: time.into(),
        open: open.to_string(),
        high: open.max(close).to_string(),
        low: open.min(close).to_string(),
        close: close.to_string(),
        volume: "1000".into(),
    }
}

fn input(bars: Vec<ChartCandle>, bps: f64) -> ContinuousStrategyPreviewInput {
    serde_json::from_value(json!({
        "strategyId":"price_condition_synthetic_contract", "strategyName":"synthetic PC contracts",
        "symbol":"TEST", "isOverseas":true, "orderQuantity":2,
        "params":{"symbols":[{"symbol":"TEST","quantity":2,"buy_trigger_price":90,
            "sell_trigger_price":0,"take_profit_pct":0,"stop_loss_pct":5,"is_overseas":true}]},
        "chunks":[{"candles":bars}],"interval":"1m","dataSource":"syntheticPcContractFixture",
        "brokerId":"kis","brokerAccountId":"synthetic-pc-contract",
        "assumptions":{"initialCapitalKrw":1000000,"executionModel":"nextOpen",
            "feeBps":bps,"taxBps":0,"slippageBps":bps,"exchangeRateKrw":1000,
            "maxPositionRatio":1,"dailyLossLimitKrw":1000000,"inSamplePercent":100}
    }))
    .unwrap()
}

fn run(input: ContinuousStrategyPreviewInput) -> ContinuousStrategyPreviewView {
    preview_strategy_continuous_from_candles(input).unwrap()
}

fn canonical(time: &str) -> String {
    generic::normalize_minute_time(time).unwrap()
}

fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 0.011, "{actual} != {expected}");
}

fn economics(view: &StrategyPreviewView) -> Value {
    json!({"signals":view.signals,"backtest":view.backtest,"preparation":view.preparation})
}

fn partition(
    mut input: ContinuousStrategyPreviewInput,
    sizes: &[usize],
) -> ContinuousStrategyPreviewInput {
    let bars = input.chunks.remove(0).candles;
    assert_eq!(sizes.iter().sum::<usize>(), bars.len());
    let mut offset = 0;
    input.chunks = sizes
        .iter()
        .map(|size| {
            let chunk = ContinuousStrategyPreviewChunk {
                candles: bars[offset..offset + size].to_vec(),
            };
            offset += size;
            chunk
        })
        .collect();
    input
}

fn sized_input(bars: Vec<ChartCandle>, bps: f64) -> ContinuousStrategyPreviewInput {
    let mut request = input(bars, bps);
    // Prior synthetic TR14 is $2; 200bps / ($2 * 2 * FX) buys 5 shares.
    // PC still requests its configured 2-share Sell, leaving a real remainder.
    request.history_candles = (0..15)
        .map(|i| {
            let mut row = bar(&format!("2026-10-01T12:{i:02}:00Z"), 90., 90.);
            row.high = "91".into();
            row.low = "89".into();
            row
        })
        .collect();
    request.assumptions.volatility_sizing_enabled = true;
    request.assumptions.risk_per_trade_bps = 200;
    request.assumptions.atr_stop_multiplier = 2.;
    request
}

fn partial_bars() -> Vec<ChartCandle> {
    vec![
        bar("2026-10-02T14:00:00Z", 90., 90.),
        bar("2026-10-02T14:01:00Z", 100., 90.),
        bar("2026-10-02T14:02:00Z", 90., 90.),
        bar("2026-10-02T14:03:00Z", 80., 80.),
    ]
}

#[test]
fn pc_next_open_native_average_drives_stop_even_when_fx_changes() {
    for bps in [10., 20.] {
        let bars = vec![
            bar("2026-10-02T14:00:00Z", 90., 90.),
            bar("2026-10-02T14:01:00Z", 100., 98.),
            bar("2026-10-02T14:02:00Z", 98., 96.),
            bar("2026-10-02T14:03:00Z", 96., 95.),
            bar("2026-10-02T14:04:00Z", 94., 94.),
        ];
        let fixed = run(input(bars.clone(), bps));
        let mut changed = input(bars, bps);
        changed.assumptions.fx_scenario = vec![FxScenarioPoint {
            bar_time: "2026-10-02T14:02:00Z".into(),
            rate_krw: 2000.,
        }];
        let changed = run(changed);
        assert_eq!(json!(fixed.preview.signals), json!(changed.preview.signals));
        assert_eq!(fixed.preview.signals.len(), 2);
        let stop = &changed.preview.signals[1];
        assert_eq!(stop.time, canonical("2026-10-02T14:03:00Z"));
        assert!(stop.reason.contains("가격조건 손절"));
        assert!(stop
            .reason
            .contains(&format!("{:.2}USD", 100. * (1. + bps / 10000.))));
        let trades = &changed.preview.backtest.trades;
        near(fixed.preview.backtest.trades[1].exchange_rate_krw, 1000.);
        near(trades[1].exchange_rate_krw, 2000.);
        assert_ne!(
            fixed.preview.backtest.summary.final_equity_krw,
            changed.preview.backtest.summary.final_equity_krw
        );
        assert_eq!(
            trades[0].signal_available_at.as_ref().unwrap().bar_time,
            canonical("2026-10-02T14:00:00Z")
        );
        assert_eq!(
            trades[0].fill_at.as_ref().unwrap().bar_time,
            canonical("2026-10-02T14:01:00Z")
        );
        near(trades[0].fill_price.unwrap(), 100. * (1. + bps / 10000.));
        assert_eq!(
            trades[1].fill_at.as_ref().unwrap().bar_time,
            canonical("2026-10-02T14:04:00Z")
        );
        assert_eq!(trades[1].fill_at.as_ref().unwrap().phase, "open");
        near(trades[1].execution_reference_price.unwrap(), 94.);
        assert_ne!(
            fixed.preview.replay.input_hash,
            changed.preview.replay.input_hash
        );
        assert_eq!(changed.preview.backtest.summary.completed_trades, 1);
        assert_eq!(changed.preview.backtest.accounting.remaining_quantity, 0);
    }
}

#[test]
fn pc_native_stop_threshold_is_inclusive_and_pending_stop_does_not_remove_owned_position() {
    let bars = vec![
        bar("2026-10-02T14:00:00Z", 90., 90.),
        bar("2026-10-02T14:01:00Z", 100., 95.01),
        bar("2026-10-02T14:02:00Z", 95.01, 95.),
    ];
    let pending = run(input(bars, 0.));
    assert_eq!(pending.preview.signals.len(), 2);
    assert_eq!(
        pending.preview.signals[1].time,
        canonical("2026-10-02T14:02:00Z")
    );
    assert_eq!(pending.preview.signals[1].quantity, 2);
    assert_eq!(pending.preview.backtest.trades[1].status, "unfilled");
    assert!(pending.preview.backtest.trades[1].fill_at.is_none());
    assert_eq!(pending.preview.backtest.summary.completed_trades, 0);
    assert_eq!(pending.preview.backtest.summary.blocked_order_count, 0);
    assert_eq!(pending.preview.backtest.accounting.remaining_quantity, 2);
    assert!(pending.continuity.chunks[0].pending_at_end);
    near(
        pending.preview.backtest.accounting.remaining_cost_basis_krw,
        200000.,
    );
}

#[test]
fn pc_stop_submission_blocks_same_market_day_and_next_day_until_exact_elapsed_boundary() {
    for bps in [10., 20.] {
        // Synthetic overnight observations exercise market dates, not exchange hours.
        let bars = vec![
            bar("2026-07-10T23:45:00-04:00", 90., 90.),
            bar("2026-07-10T23:46:00-04:00", 100., 90.),
            bar("2026-07-10T23:50:00-04:00", 90., 90.),
            bar("2026-07-10T23:59:00-04:00", 90., 90.),
            bar("2026-07-11T00:49:00-04:00", 90., 90.),
            bar("2026-07-11T00:50:00-04:00", 90., 90.),
        ];
        let view = run(input(bars, bps));
        let trades = &view.preview.backtest.trades;
        assert_eq!(trades.len(), 5);
        assert_eq!(trades[1].status, "filled");
        assert!(trades[2]
            .blocked_reason
            .as_ref()
            .unwrap()
            .contains("손절 후 당일 재진입"));
        assert!(trades[3]
            .blocked_reason
            .as_ref()
            .unwrap()
            .contains("매도 후 재매수 쿨다운"));
        assert_eq!(trades[4].status, "filled");
        assert_eq!(
            trades[4].fill_at.as_ref().unwrap().bar_time,
            canonical("2026-07-11T00:50:00-04:00")
        );
        assert_eq!(view.preview.backtest.summary.completed_trades, 1);
        assert_eq!(view.preview.backtest.summary.blocked_order_count, 2);
        assert_eq!(view.preview.backtest.accounting.remaining_quantity, 2);
    }
}

#[test]
fn pc_repeated_native_stop_is_guarded_after_partial_exit_and_overlay_reduces_the_remainder() {
    for bps in [10., 20.] {
        let native = run(sized_input(partial_bars(), bps));
        let mut overlay = sized_input(partial_bars(), bps);
        overlay.assumptions.stop_loss_pct = 15.;
        let overlay = run(overlay);
        let native_trades = &native.preview.backtest.trades;
        assert_eq!(native_trades[0].quantity, 5);
        assert_eq!(native_trades[1].quantity, 2);
        assert_eq!(native_trades[1].status, "filled");
        assert_eq!(native_trades[2].quantity, 2);
        assert!(native_trades[2]
            .blocked_reason
            .as_ref()
            .unwrap()
            .contains("동일 방향 쿨다운"));
        assert_eq!(native.preview.backtest.accounting.remaining_quantity, 3);
        assert!(native
            .preview
            .signals
            .iter()
            .filter(|s| s.side == "sell")
            .all(|s| s.quantity == 2));
        let protective = overlay
            .preview
            .backtest
            .trades
            .iter()
            .find(|t| t.reason.starts_with("연구용 보호 시나리오"))
            .unwrap();
        assert_eq!(protective.status, "filled");
        assert_eq!(protective.quantity, 3);
        assert_eq!(protective.fill_at.as_ref().unwrap().phase, "open");
        assert_eq!(
            protective.intrabar_path_status.as_deref(),
            Some("gapAtOpen")
        );
        assert_eq!(overlay.preview.backtest.accounting.remaining_quantity, 0);
        // The pending native Sell remains an ordinary guarded strategy order.
        let native_after_overlay = overlay
            .preview
            .backtest
            .trades
            .iter()
            .find(|t| t.reason.contains("가격조건 손절") && t.status == "blocked")
            .unwrap();
        assert!(native_after_overlay
            .blocked_reason
            .as_ref()
            .unwrap()
            .contains("쿨다운"));
        assert_eq!(native.preview.backtest.summary.completed_trades, 1);
        assert_eq!(overlay.preview.backtest.summary.completed_trades, 2);
    }
}

#[test]
fn pc_native_sell_cooldown_uses_elapsed_instants_across_dst_repeated_hour() {
    let bars = vec![
        bar("2026-11-01T01:27:00-04:00", 90., 90.),
        bar("2026-11-01T01:28:00-04:00", 100., 90.),
        bar("2026-11-01T01:30:00-04:00", 90., 90.),
        bar("2026-11-01T01:59:00-04:00", 90., 90.),
        bar("2026-11-01T01:00:00-05:00", 90., 90.),
    ];
    let view = run(sized_input(bars, 10.));
    let trades = &view.preview.backtest.trades;
    assert_eq!(trades[1].status, "filled");
    assert!(trades[2]
        .blocked_reason
        .as_ref()
        .unwrap()
        .contains("동일 방향 쿨다운"));
    assert_eq!(trades[3].status, "filled");
    assert_eq!(
        trades[3].fill_at.as_ref().unwrap().bar_time,
        canonical("2026-11-01T01:00:00-05:00")
    );
    assert_eq!(view.preview.backtest.accounting.remaining_quantity, 1);
    assert_eq!(view.preview.backtest.summary.completed_trades, 2);
    assert_eq!(
        view.preview.backtest.trades.last().unwrap().status,
        "unfilled"
    );
}

#[test]
fn pc_two_cost_partial_stop_ledgers_reconcile_independent_cash_costs_and_turnover() {
    for bps in [10., 20.] {
        let view = run(sized_input(partial_bars(), bps));
        let report = &view.preview.backtest;
        let rate = bps / 10000.;
        let buy = 100. * (1. + rate) * 1000.;
        let sell = 90. * (1. - rate) * 1000.;
        let buy_fee = buy * 5. * rate;
        let sell_fee = sell * 2. * rate;
        let realized = (sell - buy) * 2. - buy_fee * 2. / 5. - sell_fee;
        let residual = (80000. - buy) * 3. - buy_fee * 3. / 5.;
        let cash = 1000000. - buy * 5. - buy_fee + sell * 2. - sell_fee;
        assert_eq!(report.summary.signal_count, 4);
        assert_eq!(report.summary.filled_order_count, 2);
        assert_eq!(report.summary.blocked_order_count, 1);
        assert_eq!(report.summary.unfilled_order_count, 1);
        assert_eq!(report.accounting.remaining_quantity, 3);
        near(report.accounting.remaining_cost_basis_krw, buy * 3.);
        near(report.accounting.remaining_buy_fee_krw, buy_fee * 3. / 5.);
        near(report.accounting.realized_pnl_krw, realized);
        near(report.accounting.residual_unrealized_pnl_krw, residual);
        near(report.accounting.total_fee_krw, buy_fee + sell_fee);
        near(
            report.accounting.total_slippage_krw,
            100000. * rate * 5. + 90000. * rate * 2.,
        );
        near(report.accounting.total_tax_krw, 0.);
        near(report.summary.final_equity_krw, cash + 80000. * 3.);
        near(
            report.summary.final_equity_krw - 1000000.,
            realized + residual,
        );
        near(
            report.summary.turnover_pct,
            (buy * 5. + sell * 2.) / 1000000. * 100.,
        );
        assert_eq!(report.summary.completed_trades, 1);
    }
}

#[test]
fn pc_native_and_overlay_reports_signals_and_hash_are_invariant_under_continuous_partitions() {
    for bps in [10., 20.] {
        for overlay in [0., 15.] {
            let prepare = || {
                let mut request = sized_input(partial_bars(), bps);
                request.assumptions.stop_loss_pct = overlay;
                request
            };
            let one = run(prepare());
            for sizes in [vec![1, 1, 1, 1], vec![1, 2, 1], vec![2, 2]] {
                let split = run(partition(prepare(), &sizes));
                assert_eq!(economics(&one.preview), economics(&split.preview));
                assert_eq!(
                    one.preview.replay.input_hash,
                    split.preview.replay.input_hash
                );
                assert_eq!(
                    split.preview.replay.data_source,
                    "syntheticPcContractFixture"
                );
                assert_eq!(
                    split.preview.backtest.assumptions.execution_model,
                    ExecutionModel::NextOpen
                );
            }
        }
    }
}

#[test]
fn pc_stop_reentry_uses_us_market_day_after_utc_midnight_and_elapsed_cooldown_expiry() {
    for bps in [10., 20.] {
        let bars = vec![
            bar("2026-07-10T18:58:00-04:00", 90., 90.),
            bar("2026-07-10T18:59:00-04:00", 100., 90.),
            bar("2026-07-10T19:00:00-04:00", 90., 90.),
            bar("2026-07-11T00:01:00Z", 90., 90.),
            bar("2026-07-11T04:00:00Z", 90., 90.),
        ];
        let view = run(input(bars, bps));
        let trades = &view.preview.backtest.trades;
        assert_eq!(trades[1].status, "filled");
        assert_eq!(trades[2].time, canonical("2026-07-11T00:01:00Z"));
        assert!(trades[2]
            .blocked_reason
            .as_ref()
            .unwrap()
            .contains("손절 후 당일 재진입"));
        assert_eq!(trades[3].status, "filled");
        assert_eq!(
            trades[3].fill_at.as_ref().unwrap().bar_time,
            canonical("2026-07-11T04:00:00Z")
        );
        assert_eq!(view.preview.backtest.summary.completed_trades, 1);
        assert_eq!(view.preview.backtest.accounting.remaining_quantity, 2);
    }
}

#[test]
fn pc_configured_sell_request_is_preserved_when_actual_fill_is_smaller() {
    for bps in [10., 20.] {
        let mut request = sized_input(partial_bars(), bps);
        request.assumptions.risk_per_trade_bps = 50;
        let view = run(request);
        let trades = &view.preview.backtest.trades;
        assert_eq!(view.preview.signals[0].quantity, 2);
        assert_eq!(trades[0].quantity, 1);
        assert_eq!(view.preview.signals[1].quantity, 2);
        assert_eq!(trades[1].quantity, 1);
        assert_eq!(trades[1].status, "filled");
        assert_eq!(view.preview.backtest.accounting.remaining_quantity, 0);
        assert_eq!(view.preview.backtest.summary.completed_trades, 1);
        assert!(trades[2]
            .blocked_reason
            .as_ref()
            .unwrap()
            .contains("손절 후 당일 재진입"));
    }
}
