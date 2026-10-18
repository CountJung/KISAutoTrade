//! Integration tests compile the library without cfg(test)'s live-clock doubles.
use kis_auto_trade_lib::trading::{
    simulation::{replay_input_hash, run_backtest, SimulationAssumptions, SimulationEvent},
    strategy::{
        LeveragedTrendHoldParams, LeveragedTrendHoldStrategy, LeveragedTrendHoldTimedCandle,
        OhlcCandle, Signal,
    },
};
use serde_json::{json, Value};

fn params() -> LeveragedTrendHoldParams {
    let mut params: LeveragedTrendHoldParams = serde_json::from_value(json!({
        "entries": [{"leveraged_symbol":"KORU", "quantity":1, "is_overseas":false}]
    }))
    .unwrap();
    // Keep the fixture outside the trend entry window; use the existing rebound setup.
    params.intraday_rebound_enabled = true;
    params.rapid_rebound_enabled = false;
    params.rebound_baseline_ticks = 2;
    params.rebound_confirm_ticks = 2;
    params.rebound_pullback_pct = 4.0;
    params.rebound_buy_pressure_pct = 2.0;
    params.rebound_rsi_min = 20.0;
    params.no_trade_adx_below = 0.0;
    params
}

fn history() -> Vec<OhlcCandle> {
    (0..90)
        .map(|i| {
            let open = 100 + i;
            OhlcCandle {
                open,
                high: open + 3,
                low: open - 1,
                close: open + 2,
            }
        })
        .collect()
}

fn timed() -> Vec<LeveragedTrendHoldTimedCandle> {
    // These are legitimate timed minute observations, not daily indicator seeds.
    // The prior afternoon lies outside the configured trend entry window and
    // has no rebound; it prepares RSI/ADX before the next day's rebound.
    let warmup =
        history()
            .into_iter()
            .enumerate()
            .map(|(i, candle)| LeveragedTrendHoldTimedCandle {
                time: format!("20260706{:02}{:02}00", 13 + (30 + i) / 60, (30 + i) % 60),
                candle,
            });
    let evaluation = [190, 180, 181, 186, 186]
        .into_iter()
        .zip(["090100", "090200", "090300", "090400", "152500"])
        .map(|(price, time)| LeveragedTrendHoldTimedCandle {
            time: format!("20260707{time}"),
            candle: OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            },
        });
    warmup.chain(evaluation).collect()
}

fn run(params: LeveragedTrendHoldParams, candles: &[LeveragedTrendHoldTimedCandle]) -> Value {
    let signals = LeveragedTrendHoldStrategy::preview_signals_with_execution(
        "KORU",
        params,
        &history(),
        candles,
        |_, _| true,
    );
    let events: Vec<_> = candles
        .iter()
        .map(|c| SimulationEvent {
            time: c.time.clone(),
            chart_time: c.time.clone(),
            open_units: c.candle.open,
            close_units: c.candle.close,
            high_units: c.candle.high,
            low_units: c.candle.low,
            signal: signals.iter().find(|s| s.time == c.time).map(|s| {
                if s.side == "buy" {
                    Signal::Buy {
                        symbol: "KORU".into(),
                        quantity: s.quantity,
                        reason: s.reason.clone(),
                    }
                } else {
                    Signal::Sell {
                        symbol: "KORU".into(),
                        quantity: s.quantity,
                        reason: s.reason.clone(),
                    }
                }
            }),
        })
        .collect();
    let report = run_backtest(
        "leveraged_trend_hold",
        "KORU",
        false,
        SimulationAssumptions::default(),
        &events,
    );
    let fingerprint = serde_json::to_string(&signals).unwrap();
    json!({"signals": signals, "backtest": report, "signalHash": replay_input_hash(&[&fingerprint])})
}

#[test]
fn timed_replay_has_nonempty_repeatable_signals_and_execution_results() {
    let first = run(params(), &timed());
    assert_eq!(first, run(params(), &timed()));
    assert_eq!(first["signals"][0]["time"], "20260707090400");
    assert_eq!(first["signals"][0]["side"], "buy");
    assert_eq!(first["signals"][1]["time"], "20260707152500");
    assert_eq!(first["signals"][1]["side"], "sell");
    assert!(first["signals"][1]["reason"]
        .as_str()
        .unwrap()
        .contains("장마감"));
    assert_eq!(first["backtest"]["summary"]["filledOrderCount"], 2);
    assert_eq!(first["backtest"]["summary"]["completedTrades"], 1);
}

#[test]
fn session_and_blackout_use_input_minutes() {
    let mut off_session = timed();
    for (i, c) in off_session.iter_mut().enumerate() {
        c.time = format!("2026070708{:02}00", i % 60);
    }
    assert_eq!(run(params(), &off_session)["signals"], json!([]));
    let mut blocked = params();
    blocked.blackout_windows = vec!["09:00-09:10".into()];
    assert_eq!(run(blocked, &timed()[..94])["signals"], json!([]));
    assert_eq!(
        run(params(), &timed()[..94])["signals"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn future_bar_does_not_change_prior_signal() {
    let baseline = run(params(), &timed());
    let mut changed = timed();
    changed[94].candle = OhlcCandle {
        open: 120,
        high: 120,
        low: 120,
        close: 120,
    };
    let future = run(params(), &changed);
    assert_eq!(baseline["signals"][0], future["signals"][0]);
    assert_eq!(
        baseline["backtest"]["equityCurve"].as_array().unwrap()[..94],
        future["backtest"]["equityCurve"].as_array().unwrap()[..94]
    );
}

#[test]
fn delayed_open_fill_reaches_lth_before_close_and_last_exit_remains_unfilled() {
    use kis_auto_trade_lib::trading::simulation::{ExecutionModel, ReplaySession};
    use kis_auto_trade_lib::trading::strategy::LeveragedTrendHoldPreviewStep;
    let mut candles = timed();
    candles.insert(
        candles.len() - 1,
        LeveragedTrendHoldTimedCandle {
            time: "20260707090500".into(),
            candle: OhlcCandle {
                open: 187,
                high: 187,
                low: 187,
                close: 187,
            },
        },
    );
    let events = candles
        .iter()
        .map(|c| SimulationEvent {
            time: c.time.clone(),
            chart_time: c.time.clone(),
            open_units: c.candle.open,
            close_units: c.candle.close,
            high_units: c.candle.high,
            low_units: c.candle.low,
            signal: None,
        })
        .collect::<Vec<_>>();
    let mut session = ReplaySession::new(
        "leveraged_trend_hold",
        "KORU",
        false,
        SimulationAssumptions {
            execution_model: ExecutionModel::NextOpen,
            fee_bps: 0.0,
            tax_bps: 0.0,
            slippage_bps: 0.0,
            ..SimulationAssumptions::default()
        },
        &[],
        events.len(),
    );
    let mut index = 0;
    let mut closed = false;
    let signals = LeveragedTrendHoldStrategy::preview_signals_with_driver(
        "KORU",
        params(),
        &history(),
        &candles,
        |step| {
            let event = &events[index];
            match step {
                LeveragedTrendHoldPreviewStep::Open(_) => {
                    session.begin_bar(event);
                    closed = false;
                }
                LeveragedTrendHoldPreviewStep::Signal(signal, _) => {
                    let mut event = event.clone();
                    event.signal = Some(if signal.side == "buy" {
                        Signal::Buy {
                            symbol: "KORU".into(),
                            quantity: signal.quantity,
                            reason: signal.reason.clone(),
                        }
                    } else {
                        Signal::Sell {
                            symbol: "KORU".into(),
                            quantity: signal.quantity,
                            reason: signal.reason.clone(),
                        }
                    });
                    session.process_close(&event);
                    closed = true;
                }
                LeveragedTrendHoldPreviewStep::Close(_) => {
                    if !closed {
                        session.process_close(event);
                    }
                    index += 1;
                }
            }
            session.position_snapshot()
        },
    );
    assert_eq!(
        index,
        candles.len(),
        "every observation must close exactly once"
    );
    assert_eq!(session.position_snapshot(), (1, 187));
    let report = session.finish();
    assert_eq!(signals.len(), 2);
    assert_eq!(report.summary.filled_order_count, 1);
    assert_eq!(report.summary.unfilled_order_count, 1);
    assert_eq!(report.trades[0].fill_price, Some(187.0));
    assert_eq!(
        report.trades[0]
            .signal_available_at
            .as_ref()
            .unwrap()
            .bar_time,
        "20260707090400"
    );
    assert_eq!(
        report.trades[0].fill_at.as_ref().unwrap().bar_time,
        "20260707090500"
    );
    assert_eq!(report.trades[1].status, "unfilled");
    assert!(report.trades[1].fill_at.is_none());
}
