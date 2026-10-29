use super::*;

fn assumptions() -> SimulationAssumptions {
    SimulationAssumptions {
        initial_capital_krw: 1_000_000.0,
        fee_bps: 0.0,
        tax_bps: 0.0,
        slippage_bps: 0.0,
        max_position_ratio: 1.0,
        exchange_rate_krw: 1_000.0,
        ..Default::default()
    }
}
fn buy(q: u64) -> Option<Signal> {
    Some(Signal::Buy {
        symbol: "TEST".into(),
        quantity: q,
        reason: "entry".into(),
    })
}
fn sell(q: u64) -> Option<Signal> {
    Some(Signal::Sell {
        symbol: "TEST".into(),
        quantity: q,
        reason: "손절 exit".into(),
    })
}
fn bar(day: usize, price: u64, signal: Option<Signal>) -> SimulationEvent {
    SimulationEvent {
        time: format!("202610{day:02}"),
        chart_time: format!("202610{day:02}"),
        open_units: price,
        close_units: price,
        high_units: price,
        low_units: price,
        signal,
    }
}
fn diagnostics(report: &BacktestReportView, index: usize) -> &PhaseDiagnosticsView {
    report.phases[index].diagnostics.as_ref().unwrap()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 0.011, "{a} != {b}");
}
fn reconciles(report: &BacktestReportView) {
    for phase in &report.phases {
        let d = phase.diagnostics.as_ref().unwrap();
        near(
            d.equity_change_krw,
            d.realized_pnl_krw + d.end_residual_unrealized_pnl_krw
                - d.start_residual_unrealized_pnl_krw
                + d.dividend_net_krw,
        );
    }
}

#[test]
fn unequal_70_30_monotonic_growth_is_not_an_overfit_verdict() {
    let bars = (0..10)
        .map(|i| {
            bar(
                i + 1,
                10_000 + i as u64 * 100,
                if i == 0 { buy(10) } else { None },
            )
        })
        .collect::<Vec<_>>();
    let r = run_backtest("test", "TEST", false, assumptions(), &bars);
    assert_eq!(
        (
            diagnostics(&r, 0).observed_bars,
            diagnostics(&r, 1).observed_bars
        ),
        (7, 3)
    );
    assert!(r.phases[1].return_pct > 0.0);
    assert!(r.phases[1].return_pct < r.phases[0].return_pct * 0.5);
    assert!(r.overfit_warning.is_none());
    assert_eq!(
        (diagnostics(&r, 0).held_bars, diagnostics(&r, 1).held_bars),
        (7, 3)
    );
    reconciles(&r);
}

#[test]
fn zero_and_negative_later_returns_have_distinct_observation_messages() {
    let run = |last| {
        run_backtest(
            "test",
            "TEST",
            false,
            SimulationAssumptions {
                in_sample_percent: 50,
                ..assumptions()
            },
            &[
                bar(1, 100, buy(10)),
                bar(2, 110, None),
                bar(3, 110, None),
                bar(4, last, None),
            ],
        )
    };
    let flat = run(110);
    let loss = run(100);
    assert_eq!(flat.phases[1].return_pct, 0.0);
    assert!(flat.overfit_warning.unwrap().contains("보합"));
    assert!(loss.phases[1].return_pct < 0.0);
    assert!(loss.overfit_warning.unwrap().contains("후반은 손실"));
}

#[test]
fn empty_single_and_all_in_sample_preserve_existing_phase_shape() {
    let empty = run_backtest("test", "TEST", false, assumptions(), &[]);
    assert!(empty.phases.is_empty());
    assert!(empty.overfit_warning.is_none());
    let one = run_backtest("test", "TEST", false, assumptions(), &[bar(1, 100, None)]);
    assert_eq!(one.phases[0].phase, "outOfSample"); // Preserve floor(1*70/100).
    assert_eq!(diagnostics(&one, 0).observed_bars, 1);
    assert_eq!(diagnostics(&one, 0).exposure_pct, 0.0);
    let all = run_backtest(
        "test",
        "TEST",
        false,
        SimulationAssumptions {
            in_sample_percent: 100,
            ..assumptions()
        },
        &[bar(1, 100, None), bar(2, 110, None)],
    );
    assert_eq!(all.phases.len(), 1);
    assert_eq!(all.phases[0].phase, "inSample");
    assert_eq!(diagnostics(&all, 0).observed_bars, 2);
    assert!(all.overfit_warning.is_none());
    reconciles(&one);
    reconciles(&all);
}

#[test]
fn no_trade_split_has_zero_exposure_and_all_observations() {
    let r = run_backtest(
        "test",
        "TEST",
        false,
        assumptions(),
        &(1..=10)
            .map(|i| bar(i, 100 + i as u64, None))
            .collect::<Vec<_>>(),
    );
    assert_eq!(r.summary.signal_count, 0);
    for d in r.phases.iter().map(|p| p.diagnostics.as_ref().unwrap()) {
        assert_eq!(d.held_bars, 0);
        assert_eq!(d.equity_change_krw, 0.0);
        assert_eq!(d.start_quantity, 0);
        assert_eq!(d.end_quantity, 0);
    }
    reconciles(&r);
}

#[test]
fn fx_dividend_partial_exit_preserves_original_entry_cost_and_reconciles_split() {
    let a = SimulationAssumptions {
        fee_bps: 100.0,
        fx_scenario: vec![FxScenarioPoint {
            bar_time: "20261002".into(),
            rate_krw: 1200.0,
        }],
        dividend_scenario: vec![DividendScenarioPoint {
            bar_time: "20261002".into(),
            amount_per_share: 1.0,
            withholding_tax_bps: 1500.0,
        }],
        dividend_price_basis: DividendPriceBasis::Unadjusted,
        ..assumptions()
    };
    for quantity in [1, 2] {
        let r = run_backtest(
            "test",
            "TEST",
            true,
            a.clone(),
            &[bar(1, 10_000, buy(2)), bar(2, 11_000, sell(quantity))],
        );
        let first = diagnostics(&r, 0);
        let second = diagnostics(&r, 1);
        near(first.equity_change_krw, -2000.0);
        near(first.end_residual_unrealized_pnl_krw, -2000.0);
        near(second.start_residual_unrealized_pnl_krw, -2000.0);
        near(second.dividend_net_krw, 2040.0);
        near(second.realized_pnl_krw, 29680.0 * quantity as f64);
        assert_eq!(
            (second.start_quantity, second.end_quantity),
            (2, 2 - quantity)
        );
        assert_eq!(r.phases[0].completed_trades, 0);
        assert_eq!(r.phases[1].completed_trades, 1);
        assert_eq!(
            r.trades[1].realized_pnl_krw,
            Some(29680.0 * quantity as f64)
        );
        near(
            second.equity_change_krw,
            if quantity == 1 { 64720.0 } else { 63400.0 },
        );
        assert_eq!(r.phases[0].return_pct, -0.2);
        near(
            r.phases[1].return_pct,
            if quantity == 1 { 6.485 } else { 6.3527 },
        );
        reconciles(&r);
    }
}

#[test]
fn pending_buy_at_split_and_terminal_pending_sell_are_observed_without_consumption() {
    let a = SimulationAssumptions {
        in_sample_percent: 50,
        execution_model: ExecutionModel::NextOpen,
        ..assumptions()
    };
    let bars = [
        bar(1, 100, None),
        bar(2, 100, buy(2)),
        bar(3, 110, None),
        bar(4, 120, sell(2)),
    ];
    let r = run_backtest("test", "TEST", false, a, &bars);
    let first = diagnostics(&r, 0);
    let second = diagnostics(&r, 1);
    assert!(first.pending_at_end);
    assert!(second.pending_at_start);
    assert!(second.pending_at_end);
    assert_eq!((second.start_quantity, second.end_quantity), (0, 2));
    assert_eq!(second.held_bars, 2);
    assert_eq!(r.summary.unfilled_order_count, 1);
    assert_eq!(r.summary.completed_trades, 0);
    assert_eq!(r.trades[0].fill_at.as_ref().unwrap().bar_time, "20261003");
    reconciles(&r);
}

#[test]
fn pending_sale_crosses_split_with_full_cost_basis_and_prior_close_snapshot() {
    let a = SimulationAssumptions {
        in_sample_percent: 50,
        execution_model: ExecutionModel::NextOpen,
        ..assumptions()
    };
    let mut second = bar(2, 110, sell(2));
    second.open_units = 100;
    let r = run_backtest(
        "test",
        "TEST",
        false,
        a,
        &[
            bar(1, 100, buy(2)),
            second,
            bar(3, 120, None),
            bar(4, 120, None),
        ],
    );
    assert!(diagnostics(&r, 1).pending_at_start);
    assert!(!diagnostics(&r, 1).pending_at_end);
    near(diagnostics(&r, 0).end_residual_unrealized_pnl_krw, 20.0);
    near(diagnostics(&r, 1).realized_pnl_krw, 40.0);
    near(diagnostics(&r, 1).equity_change_krw, 20.0);
    assert_eq!(r.trades[1].phase, "outOfSample");
    assert_eq!(r.phases[1].completed_trades, 1);
    reconciles(&r);
}

#[test]
fn diagnostics_are_optional_camel_case_and_do_not_rewrite_legacy_phase_fields() {
    let r = run_backtest(
        "test",
        "TEST",
        false,
        assumptions(),
        &[bar(1, 100, None), bar(2, 100, None)],
    );
    let mut phase = r.phases[0].clone();
    let original = serde_json::to_value(&phase).unwrap();
    assert_eq!(original["diagnostics"]["version"], 1);
    assert_eq!(
        original["diagnostics"]["policy"],
        "sameLedgerObservationSplit"
    );
    assert_eq!(original["diagnostics"]["observedBars"], 1);
    assert!(original["diagnostics"].get("observed_bars").is_none());
    phase.diagnostics = None;
    let legacy = serde_json::to_value(phase).unwrap();
    assert!(legacy.get("diagnostics").is_none());
    for key in [
        "phase",
        "start",
        "end",
        "returnPct",
        "mddPct",
        "completedTrades",
        "winRatePct",
    ] {
        assert_eq!(legacy[key], original[key]);
    }
}
