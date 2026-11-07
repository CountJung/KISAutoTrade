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
    run_in_market(params, candles, false)
}

fn run_in_market(
    params: LeveragedTrendHoldParams,
    candles: &[LeveragedTrendHoldTimedCandle],
    is_overseas: bool,
) -> Value {
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
        is_overseas,
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

// Synthetic close-labelled US regular minutes, reusing the nonempty rebound prices.
// This exercises the production timed LTH entrypoint and replay, not provider I/O.
fn us_regular_timed(offset: chrono::FixedOffset) -> Vec<LeveragedTrendHoldTimedCandle> {
    use chrono::{Duration, TimeZone};
    let ny = chrono_tz::America::New_York;
    let warmup_start = ny.with_ymd_and_hms(2026, 7, 6, 12, 30, 0).unwrap();
    let evaluation_start = ny.with_ymd_and_hms(2026, 7, 7, 9, 31, 0).unwrap();
    timed()
        .into_iter()
        .enumerate()
        .map(|(i, mut row)| {
            let instant = if i < 90 {
                warmup_start + Duration::minutes(i as i64)
            } else if i < 94 {
                evaluation_start + Duration::minutes((i - 90) as i64)
            } else {
                ny.with_ymd_and_hms(2026, 7, 7, 15, 55, 0).unwrap()
            };
            row.time = instant.with_timezone(&offset).to_rfc3339();
            row
        })
        .collect()
}

#[test]
fn us_regular_replay_same_instants_have_identical_signals_and_fills() {
    use chrono::{DateTime, FixedOffset, SecondsFormat};
    let mut us_params = params();
    us_params.entries[0].is_overseas = true;
    us_params.toss_us_session = "regular".into();
    let kst = FixedOffset::east_opt(9 * 3600).unwrap();
    let baseline_candles = us_regular_timed(kst);
    let baseline = run_in_market(us_params.clone(), &baseline_candles, true);
    // A positive control must fail loudly before any empty==empty comparison.
    assert_eq!(baseline["signals"].as_array().unwrap().len(), 2);
    assert_eq!(baseline["signals"][0]["side"], "buy");
    assert_eq!(baseline["signals"][1]["side"], "sell");
    assert_eq!(baseline["backtest"]["summary"]["filledOrderCount"], 2);
    assert_eq!(baseline["backtest"]["summary"]["completedTrades"], 1);
    let semantic_signals = |result: &Value| {
        result["signals"]
            .as_array()
            .unwrap()
            .iter()
            .map(|signal| {
                let mut signal = signal.clone();
                signal["time"] = json!(DateTime::parse_from_rfc3339(
                    signal["time"].as_str().unwrap()
                )
                .unwrap()
                .to_utc()
                .to_rfc3339_opts(SecondsFormat::Secs, true));
                signal
            })
            .collect::<Vec<_>>()
    };
    let expected = semantic_signals(&baseline);
    let mut mismatches = Vec::new();
    for offset in [
        FixedOffset::east_opt(0).unwrap(),
        FixedOffset::west_opt(4 * 3600).unwrap(),
    ] {
        let mut candles = us_regular_timed(offset);
        for (reference, alternate) in baseline_candles.iter().zip(&candles) {
            assert_eq!(
                DateTime::parse_from_rfc3339(&reference.time).unwrap(),
                DateTime::parse_from_rfc3339(&alternate.time).unwrap()
            );
            assert_eq!(reference.candle.close, alternate.candle.close);
        }
        // Explicit Z spelling as well as the NY -04:00 and KST +09:00 spellings.
        if offset.local_minus_utc() == 0 {
            for row in &mut candles {
                row.time = DateTime::parse_from_rfc3339(&row.time)
                    .unwrap()
                    .to_utc()
                    .to_rfc3339_opts(SecondsFormat::Secs, true);
            }
        }
        let actual = run_in_market(us_params.clone(), &candles, true);
        if semantic_signals(&actual) != expected
            || actual["backtest"]["summary"] != baseline["backtest"]["summary"]
        {
            mismatches.push(json!({"offset": offset.to_string(),
                "expectedSignals": expected, "actualSignals": semantic_signals(&actual),
                "expectedFills": baseline["backtest"]["summary"]["filledOrderCount"],
                "actualFills": actual["backtest"]["summary"]["filledOrderCount"]}));
        }
    }
    assert!(
        mismatches.is_empty(),
        "same US instants changed production replay: {mismatches:#?}"
    );
}

#[test]
fn us_regular_timed_clock_winter_dst_and_kst_blackout_are_offset_invariant() {
    use chrono::{Duration, FixedOffset, TimeZone};
    let ny = chrono_tz::America::New_York;
    for (month, previous, day) in [
        (1, 6, 7),
        (3, 5, 6),
        (3, 6, 9),
        (7, 6, 7),
        (10, 29, 30),
        (11, 2, 3),
        (11, 25, 27),
    ] {
        let start = ny
            .with_ymd_and_hms(2026, month, previous, 12, 30, 0)
            .unwrap();
        let open = ny.with_ymd_and_hms(2026, month, day, 9, 31, 0).unwrap();
        let (hour, minute) = if month == 11 && day == 27 {
            (13, 0)
        } else {
            (15, 55)
        };
        let exit = ny
            .with_ymd_and_hms(2026, month, day, hour, minute, 0)
            .unwrap();
        let mut us = params();
        us.entries[0].is_overseas = true;
        us.toss_us_session = "regular".into();
        let mut baseline = None;
        for offset in [0, -4 * 3600, -5 * 3600, 9 * 3600] {
            let rows = timed()
                .into_iter()
                .enumerate()
                .map(|(i, mut row)| {
                    let instant = if i < 90 {
                        start + Duration::minutes(i as i64)
                    } else if i < 94 {
                        open + Duration::minutes((i - 90) as i64)
                    } else {
                        exit
                    };
                    row.time = instant
                        .with_timezone(&FixedOffset::east_opt(offset).unwrap())
                        .to_rfc3339();
                    row
                })
                .collect::<Vec<_>>();
            let result = run_in_market(us.clone(), &rows, true);
            assert_eq!(result["signals"].as_array().unwrap().len(), 2);
            assert_eq!(result["backtest"]["summary"]["filledOrderCount"], 2);
            let semantic = result["signals"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| {
                    let mut s = s.clone();
                    s["time"] = json!(chrono::DateTime::parse_from_rfc3339(
                        s["time"].as_str().unwrap()
                    )
                    .unwrap()
                    .to_utc()
                    .to_rfc3339());
                    s
                })
                .collect::<Vec<_>>();
            let current = (semantic, result["backtest"]["summary"].clone());
            if let Some(expected) = &baseline {
                assert_eq!(&current, expected);
            } else {
                baseline = Some(current);
            }
            let mut blocked = us.clone();
            let entry_kst = (open + Duration::minutes(3)).with_timezone(&chrono_tz::Asia::Seoul);
            blocked.blackout_windows = vec![format!(
                "{}-{}",
                entry_kst.format("%H:%M"),
                entry_kst.format("%H:%M")
            )];
            assert_eq!(run_in_market(blocked, &rows, true)["signals"], json!([]));
        }
    }
}
