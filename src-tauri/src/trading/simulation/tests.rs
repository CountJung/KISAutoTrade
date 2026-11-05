use super::*;

fn atr_history(count: usize) -> Vec<SimulationEvent> {
    (1..=count)
        .map(|day| SimulationEvent {
            time: format!("202609{day:02}"),
            chart_time: format!("202609{day:02}"),
            open_units: 500,
            close_units: 500,
            high_units: 501,
            low_units: 499,
            // Warmup signals must never become evaluation orders.
            signal: Some(Signal::Buy {
                symbol: "005930".into(),
                quantity: 1,
                reason: "ignored history".into(),
            }),
        })
        .collect()
}

fn atr_sizing_assumptions() -> SimulationAssumptions {
    SimulationAssumptions {
        initial_capital_krw: 1_000_000.0,
        max_position_ratio: 1.0,
        fee_bps: 0.0,
        tax_bps: 0.0,
        slippage_bps: 0.0,
        risk_per_trade_bps: 100,
        atr_stop_multiplier: 2.0,
        volatility_sizing_enabled: true,
        ..SimulationAssumptions::default()
    }
}

fn flat_gap_entry() -> SimulationEvent {
    SimulationEvent {
        time: "20261001".into(),
        chart_time: "20261001".into(),
        open_units: 1_000,
        close_units: 1_000,
        high_units: 1_000,
        low_units: 1_000,
        signal: Some(Signal::Buy {
            symbol: "005930".into(),
            quantity: 1_000,
            reason: "entry after gap".into(),
        }),
    }
}

#[test]
fn atr_risk_uses_previous_history_close_and_evaluation_gap_without_trading_history() {
    let report = run_backtest_with_history(
        "test",
        "005930",
        false,
        atr_sizing_assumptions(),
        &atr_history(15),
        &[flat_gap_entry()],
    );
    // (13 prior ranges * 2 + gap 500) / 14 = 37; 10,000 / (37 * 2) = 135.
    assert_eq!(report.trades.len(), 1);
    assert_eq!(report.trades[0].quantity, 135);
    assert_eq!(report.trades[0].status, "filled");
    assert_eq!(report.equity_curve.len(), 1);
    assert_eq!(report.atr_preparation.history_bars, 15);
    assert!(report.atr_preparation.ready_at_start);
    assert!(report.atr_preparation.ready_at_end);
    assert_eq!(report.atr_preparation.unready_evaluation_bars, 0);
}

#[test]
fn atr_sizing_never_silently_uses_requested_quantity_when_ranges_are_missing() {
    let report = run_backtest_with_history(
        "test",
        "005930",
        false,
        atr_sizing_assumptions(),
        &atr_history(13),
        &[flat_gap_entry()],
    );
    assert_eq!(report.trades[0].status, "blocked");
    assert!(report.trades[0]
        .blocked_reason
        .as_deref()
        .unwrap()
        .contains("ATR(TR,14)"));
    assert_eq!(report.summary.filled_order_count, 0);
    assert!(!report.atr_preparation.ready_at_end);
    assert_eq!(report.atr_preparation.unready_evaluation_bars, 1);
}

#[test]
fn zero_atr_risk_budget_cannot_fall_back_to_requested_buy_quantity() {
    let report = run_backtest_with_history(
        "test",
        "005930",
        false,
        SimulationAssumptions {
            risk_per_trade_bps: 0,
            ..atr_sizing_assumptions()
        },
        &atr_history(15),
        &[flat_gap_entry()],
    );
    assert_eq!(report.summary.filled_order_count, 0);
    assert_eq!(report.trades[0].status, "blocked");
    assert!(report.trades[0]
        .blocked_reason
        .as_ref()
        .unwrap()
        .contains("위험 예산 0bps"));
}

fn event(time: &str, close: u64, signal: Option<Signal>) -> SimulationEvent {
    SimulationEvent {
        time: time.into(),
        chart_time: time.into(),
        open_units: close,
        close_units: close,
        high_units: close + 100,
        low_units: close.saturating_sub(100),
        signal,
    }
}

#[test]
fn deterministic_replay_applies_costs_and_separates_signals_from_fills() {
    let events = vec![
        event(
            "20260701090000",
            10_000,
            Some(Signal::Buy {
                symbol: "005930".into(),
                quantity: 10,
                reason: "entry".into(),
            }),
        ),
        event(
            "20260701100000",
            11_000,
            Some(Signal::Buy {
                symbol: "005930".into(),
                quantity: 10,
                reason: "duplicate".into(),
            }),
        ),
        event(
            "20260702150000",
            12_000,
            Some(Signal::Sell {
                symbol: "005930".into(),
                quantity: 10,
                reason: "exit".into(),
            }),
        ),
    ];
    let report = run_backtest(
        "test",
        "005930",
        false,
        SimulationAssumptions {
            max_position_ratio: 1.0,
            ..SimulationAssumptions::default()
        },
        &events,
    );
    assert_eq!(report.summary.signal_count, 3);
    assert_eq!(report.summary.filled_order_count, 2);
    assert_eq!(report.summary.blocked_order_count, 1);
    assert_eq!(report.summary.completed_trades, 1);
    assert!(report.summary.cumulative_return_pct > 0.0);
    assert!(report.trades[2].cost_krw > 0.0);
}

#[test]
fn identical_fixture_has_identical_report_and_hash() {
    let events = vec![event("20260701090000", 10_000, None)];
    let assumptions = SimulationAssumptions::default();
    let left = run_backtest("test", "005930", false, assumptions.clone(), &events);
    let right = run_backtest("test", "005930", false, assumptions, &events);
    assert_eq!(
        left.summary.final_equity_krw,
        right.summary.final_equity_krw
    );
    assert_eq!(
        replay_input_hash(&["a", "b"]),
        replay_input_hash(&["a", "b"])
    );
}

#[test]
fn daily_loss_and_consecutive_loss_state_roll_over_on_event_date() {
    let buy = |time: &str| {
        event(
            time,
            10_000,
            Some(Signal::Buy {
                symbol: "005930".into(),
                quantity: 10,
                reason: "entry".into(),
            }),
        )
    };
    let events = vec![
        buy("20260701090000"),
        event(
            "20260701100000",
            8_000,
            Some(Signal::Sell {
                symbol: "005930".into(),
                quantity: 10,
                reason: "loss".into(),
            }),
        ),
        buy("20260701110000"),
        buy("20260702090000"),
    ];
    let report = run_backtest(
        "test",
        "005930",
        false,
        SimulationAssumptions {
            initial_capital_krw: 1_000_000.0,
            daily_loss_limit_krw: 1_000.0,
            max_position_ratio: 1.0,
            ..SimulationAssumptions::default()
        },
        &events,
    );

    assert_eq!(report.trades[2].status, "blocked");
    assert_eq!(
        report.trades[3].status, "filled",
        "{:?}",
        report.trades[3].blocked_reason
    );
}

#[test]
fn sell_guard_uses_the_same_round_trip_cost_assumptions_as_settlement() {
    let events = vec![
        event(
            "20260701090000",
            10_000,
            Some(Signal::Buy {
                symbol: "005930".into(),
                quantity: 10,
                reason: "entry".into(),
            }),
        ),
        event(
            "20260701100000",
            10_100,
            Some(Signal::Sell {
                symbol: "005930".into(),
                quantity: 10,
                reason: "small gross profit".into(),
            }),
        ),
    ];
    let report = run_backtest(
        "test",
        "005930",
        false,
        SimulationAssumptions {
            fee_bps: 100.0,
            tax_bps: 0.0,
            slippage_bps: 0.0,
            max_position_ratio: 1.0,
            ..SimulationAssumptions::default()
        },
        &events,
    );

    assert_eq!(report.trades[1].status, "blocked");
    assert_eq!(report.summary.completed_trades, 0);
}
