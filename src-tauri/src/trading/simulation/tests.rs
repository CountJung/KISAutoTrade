use super::*;

fn event(time: &str, close: u64, signal: Option<Signal>) -> SimulationEvent {
    SimulationEvent {
        time: time.into(),
        chart_time: time.into(),
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
