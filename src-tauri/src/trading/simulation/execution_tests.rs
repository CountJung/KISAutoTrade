use super::*;

fn buy(quantity: u64) -> Option<Signal> {
    Some(Signal::Buy {
        symbol: "005930".into(),
        quantity,
        reason: "entry".into(),
    })
}
fn sell(quantity: u64) -> Option<Signal> {
    Some(Signal::Sell {
        symbol: "005930".into(),
        quantity,
        reason: "손절 exit".into(),
    })
}
fn bar(
    time: &str,
    open: u64,
    high: u64,
    low: u64,
    close: u64,
    signal: Option<Signal>,
) -> SimulationEvent {
    SimulationEvent {
        time: time.into(),
        chart_time: time.into(),
        open_units: open,
        close_units: close,
        high_units: high,
        low_units: low,
        signal,
    }
}
fn assumptions() -> SimulationAssumptions {
    SimulationAssumptions {
        initial_capital_krw: 1_000_000.0,
        max_position_ratio: 1.0,
        fee_bps: 0.0,
        tax_bps: 0.0,
        slippage_bps: 0.0,
        execution_model: ExecutionModel::NextOpen,
        ..Default::default()
    }
}
fn replay(settings: SimulationAssumptions, bars: &[SimulationEvent]) -> BacktestReportView {
    run_backtest("test", "005930", false, settings, bars)
}

#[test]
fn delayed_fill_uses_next_observed_open_and_final_signal_remains_unfilled() {
    let bars = [
        bar("20261001", 90, 105, 85, 100, buy(10)),
        // Missing days are not interpolated.
        bar("20261005", 150, 205, 145, 200, sell(10)),
    ];
    let report = replay(assumptions(), &bars);
    assert_eq!(report.trades[0].signal_price, 100.0);
    assert_eq!(report.trades[0].fill_price, Some(150.0));
    assert_eq!(
        report.trades[0]
            .signal_available_at
            .as_ref()
            .unwrap()
            .bar_time,
        "20261001"
    );
    assert_eq!(
        report.trades[0].signal_available_at.as_ref().unwrap().phase,
        "close"
    );
    assert_eq!(
        report.trades[0].fill_at.as_ref().unwrap().bar_time,
        "20261005"
    );
    assert_eq!(report.trades[0].fill_at.as_ref().unwrap().phase, "open");
    assert_eq!(report.trades[1].status, "unfilled");
    assert!(report.trades[1].fill_at.is_none());
    assert!(report.trades[1].blocked_reason.is_none());
    assert!(report.trades[1]
        .reason
        .contains("다음 관측 봉이 없어 미체결"));
    assert_eq!(report.summary.unfilled_order_count, 1);
    assert_eq!(report.summary.blocked_order_count, 0);
    assert_eq!(report.summary.final_equity_krw, 1_000_500.0);
}

#[test]
fn gap_sizing_checks_cash_and_position_limits_at_execution_price() {
    let bars = [
        bar("20261001", 500, 500, 500, 500, buy(1_000)),
        bar("20261002", 2_000, 2_000, 2_000, 2_000, None),
    ];
    let report = replay(assumptions(), &bars);
    assert_eq!(report.trades[0].status, "blocked");
    assert!(report.trades[0]
        .blocked_reason
        .as_deref()
        .unwrap()
        .contains("비중"));
    assert_eq!(report.summary.filled_order_count, 0);
}

#[test]
fn next_open_atr_never_observes_future_high_low_or_close() {
    let history: Vec<_> = (1..=15)
        .map(|day| bar(&format!("202609{day:02}"), 500, 501, 499, 500, None))
        .collect();
    let settings = SimulationAssumptions {
        volatility_sizing_enabled: true,
        risk_per_trade_bps: 100,
        atr_stop_multiplier: 2.0,
        ..assumptions()
    };
    let signal = bar("20261001", 500, 501, 499, 500, buy(1_000));
    let stable = bar("20261002", 500, 501, 499, 500, None);
    let extreme = bar("20261002", 500, 1_000_000, 1, 900_000, None);
    let a = run_backtest_with_history(
        "test",
        "005930",
        false,
        settings.clone(),
        &history,
        &[signal.clone(), stable],
    );
    let b = run_backtest_with_history(
        "test",
        "005930",
        false,
        settings,
        &history,
        &[signal, extreme],
    );
    // Prior ATR=2: risk budget permits2,500; opening-price position cap permits2,000.
    assert_eq!(a.trades[0].quantity, 2_000);
    assert_eq!(a.trades[0].fill_price, Some(500.0));
    assert_eq!(
        serde_json::to_value(&a.trades).unwrap(),
        serde_json::to_value(&b.trades).unwrap()
    );
}

#[test]
fn atr_that_only_becomes_ready_after_execution_cannot_size_the_open_order() {
    let history: Vec<_> = (1..=13)
        .map(|day| bar(&format!("202609{day:02}"), 500, 501, 499, 500, None))
        .collect();
    let settings = SimulationAssumptions {
        volatility_sizing_enabled: true,
        ..assumptions()
    };
    let bars = [
        bar("20261001", 500, 501, 499, 500, buy(1)),
        bar("20261002", 500, 501, 499, 500, None),
    ];
    let report = run_backtest_with_history("test", "005930", false, settings, &history, &bars);
    assert_eq!(report.trades[0].status, "blocked");
    assert!(report.trades[0]
        .blocked_reason
        .as_deref()
        .unwrap()
        .contains("ATR(TR,14)"));
    assert!(report.atr_preparation.ready_at_end);
}

#[test]
fn protective_stop_uses_actual_fill_cost_and_adverse_gap_open() {
    let bars = [
        bar("20261001", 100, 100, 100, 100, buy(10)),
        bar("20261002", 200, 205, 195, 200, None),
        bar("20261003", 140, 160, 135, 150, None),
    ];
    let report = replay(
        SimulationAssumptions {
            stop_loss_pct: 10.0,
            slippage_bps: 100.0,
            ..assumptions()
        },
        &bars,
    );
    assert_eq!(report.trades.len(), 2);
    assert_eq!(report.trades[0].fill_price, Some(202.0));
    // Threshold is 181.8 from actual 202 fill; gap must fill at 140 less slippage.
    assert_eq!(report.trades[1].fill_price, Some(138.6));
    assert_eq!(
        report.trades[1].intrabar_path_status.as_deref(),
        Some("gapAtOpen")
    );
    assert_eq!(report.trades[1].fill_at.as_ref().unwrap().phase, "open");
    assert_eq!(report.summary.completed_trades, 1);
}

#[test]
fn next_open_entry_can_stop_same_bar_but_same_close_entry_cannot_see_earlier_range() {
    let bars = [
        bar("20261001", 100, 120, 50, 100, buy(10)),
        bar("20261002", 100, 120, 50, 100, None),
    ];
    let next = replay(
        SimulationAssumptions {
            stop_loss_pct: 10.0,
            intrabar_exit_policy: IntrabarExitPolicy::StopFirst,
            ..assumptions()
        },
        &bars,
    );
    assert_eq!(next.trades.len(), 2);
    assert_eq!(next.trades[1].fill_price, Some(90.0));
    let same = replay(
        SimulationAssumptions {
            execution_model: ExecutionModel::SameClose,
            stop_loss_pct: 10.0,
            intrabar_exit_policy: IntrabarExitPolicy::StopFirst,
            ..assumptions()
        },
        &bars[..1],
    );
    assert_eq!(same.trades.len(), 1);
    assert_eq!(same.summary.completed_trades, 0);
}

#[test]
fn both_touch_uses_stop_first_only_when_selected_and_reports_ambiguity() {
    let bars = [
        bar("20261001", 100, 100, 100, 100, buy(10)),
        bar("20261002", 100, 120, 80, 100, None),
    ];
    let settings = SimulationAssumptions {
        stop_loss_pct: 10.0,
        take_profit_pct: 10.0,
        intrabar_exit_policy: IntrabarExitPolicy::StopFirst,
        ..assumptions()
    };
    let report = replay(settings.clone(), &bars);
    assert_eq!(report.summary.ambiguous_bar_count, 1);
    assert_eq!(report.trades[1].fill_price, Some(90.0));
    assert_eq!(
        report.trades[1].intrabar_path_status.as_deref(),
        Some("ambiguousStopFirst")
    );
    assert_eq!(report.trades[1].fill_at.as_ref().unwrap().phase, "intrabar");
    let unverified = replay(
        SimulationAssumptions {
            intrabar_exit_policy: IntrabarExitPolicy::Unverified,
            ..settings
        },
        &bars,
    );
    assert_eq!(unverified.trades.len(), 1);
    assert_eq!(unverified.summary.ambiguous_bar_count, 1);
    assert!(unverified
        .execution_warnings
        .iter()
        .any(|warning| warning.contains("unverified")));
}

#[test]
fn partial_sell_preserves_cost_basis_and_remaining_position_feedback() {
    let bars = [
        bar("20261001", 100, 100, 100, 100, buy(10)),
        bar("20261002", 100, 120, 100, 120, sell(4)),
        bar("20261003", 120, 120, 120, 120, None),
    ];
    let mut session = ReplaySession::new(
        "test",
        "005930",
        false,
        SimulationAssumptions {
            fee_bps: 100.0,
            ..assumptions()
        },
        &[],
        bars.len(),
    );
    for bar in &bars {
        session.begin_bar(bar);
        session.process_close(bar);
    }
    assert_eq!(session.position_snapshot(), (6, 100));
    let report = session.finish();
    assert_eq!(report.trades[1].quantity, 4);
    assert_eq!(report.trades[1].realized_pnl_krw, Some(71.2));
    assert_eq!(report.summary.final_equity_krw, 1_000_185.2);
    assert_eq!(report_position_snapshot(&report, false), (6, 100));
}

#[test]
fn execution_day_rollover_releases_prior_day_automatic_loss_stop() {
    let bars = [
        bar("20261001090000", 10_000, 10_000, 10_000, 10_000, buy(10)),
        bar("20261001100000", 10_000, 10_000, 10_000, 10_000, sell(10)),
        bar("20261001110000", 8_000, 8_000, 8_000, 8_000, buy(10)),
        bar("20261002090000", 10_000, 10_000, 10_000, 10_000, None),
    ];
    let report = replay(
        SimulationAssumptions {
            daily_loss_limit_krw: 1_000.0,
            ..assumptions()
        },
        &bars,
    );
    assert_eq!(
        report
            .trades
            .iter()
            .map(|trade| trade.status.as_str())
            .collect::<Vec<_>>(),
        vec!["filled", "filled", "filled"]
    );
}

#[test]
fn streamed_chunks_and_batch_share_identical_accounting_and_causal_pending_state() {
    let bars = [
        bar("20261001", 100, 101, 99, 100, buy(10)),
        bar("20261002", 110, 130, 100, 120, sell(4)),
        bar("20261003", 130, 140, 125, 135, None),
    ];
    let expected = replay(assumptions(), &bars);
    let mut session = ReplaySession::new("test", "005930", false, assumptions(), &[], bars.len());
    for chunk in bars.chunks(1) {
        for event in chunk {
            session.begin_bar(event);
            session.process_close(event);
        }
    }
    assert_eq!(
        serde_json::to_value(expected).unwrap(),
        serde_json::to_value(session.finish()).unwrap()
    );
}

#[test]
fn default_same_close_legacy_numeric_fixture_is_preserved() {
    let bars = [
        bar("20260701090000", 10_000, 10_100, 9_900, 10_000, buy(10)),
        bar("20260701100000", 11_000, 11_100, 10_900, 11_000, buy(10)),
        bar("20260702150000", 12_000, 12_100, 11_900, 12_000, sell(10)),
    ];
    let report = replay(
        SimulationAssumptions {
            max_position_ratio: 1.0,
            ..Default::default()
        },
        &bars,
    );
    assert_eq!(
        report.assumptions.execution_model,
        ExecutionModel::SameClose
    );
    assert_eq!(report.summary.final_equity_krw, 10_019_617.12);
    assert_eq!(report.trades[0].fill_price, Some(10_005.0));
    assert_eq!(report.trades[2].fill_price, Some(11_994.0));
    assert_eq!(report.trades[2].realized_pnl_krw, Some(19_617.12));
    assert_eq!(report.summary.filled_order_count, 2);
    assert_eq!(report.summary.blocked_order_count, 1);
}

#[test]
fn assumptions_normalize_nonfinite_protection_and_expose_selected_model() {
    let settings = SimulationAssumptions {
        stop_loss_pct: f64::NAN,
        take_profit_pct: f64::INFINITY,
        ..assumptions()
    };
    let report = replay(settings, &[]);
    assert_eq!(report.assumptions.stop_loss_pct, 0.0);
    assert_eq!(report.assumptions.take_profit_pct, 0.0);
    let report = replay(
        SimulationAssumptions {
            stop_loss_pct: 100.0,
            take_profit_pct: 2_000.0,
            ..assumptions()
        },
        &[],
    );
    assert_eq!(report.assumptions.stop_loss_pct, 99.9);
    assert_eq!(report.assumptions.take_profit_pct, 1_000.0);
}

#[test]
fn single_take_profit_touch_is_labeled_unverified_and_floors_fractional_threshold() {
    let bars = [
        bar("20261001", 100, 100, 100, 100, buy(10)),
        bar("20261002", 100, 105, 100, 100, None),
        bar("20261003", 105, 112, 104, 110, None),
    ];
    let report = replay(
        SimulationAssumptions {
            take_profit_pct: 10.0,
            intrabar_exit_policy: IntrabarExitPolicy::StopFirst,
            slippage_bps: 100.0,
            ..assumptions()
        },
        &bars,
    );
    assert_eq!(report.trades[0].fill_price, Some(101.0));
    // 101 * 1.1 = 111.1: the sell price is conservatively floored to whole units.
    assert_eq!(report.trades[1].fill_price, Some(109.89));
    assert_eq!(
        report.trades[1].intrabar_path_status.as_deref(),
        Some("ohlcUnverified")
    );
    assert_eq!(report.summary.ambiguous_bar_count, 0);
}
