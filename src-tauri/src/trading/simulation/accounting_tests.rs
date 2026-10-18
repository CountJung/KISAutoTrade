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
fn bar(day: u32, price: u64, signal: Option<Signal>) -> SimulationEvent {
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
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 0.011, "{a} != {b}");
}
fn reconciles(report: &BacktestReportView) {
    near(
        report.summary.final_equity_krw - report.summary.initial_capital_krw,
        report.accounting.realized_pnl_krw
            + report.accounting.residual_unrealized_pnl_krw
            + report.accounting.dividend_net_krw,
    );
}

#[test]
fn partial_exit_keeps_historical_basis_buy_fee_and_terminal_is_only_a_projection() {
    let a = SimulationAssumptions {
        fee_bps: 10.0,
        minimum_fee_native: 50.0,
        tax_bps: 100.0,
        ..assumptions()
    };
    let report = run_backtest(
        "test",
        "TEST",
        false,
        a,
        &[bar(1, 1_000, buy(10)), bar(2, 1_100, sell(5))],
    );
    let v = &report.accounting;
    assert_eq!(v.remaining_quantity, 5);
    near(v.remaining_cost_basis_krw, 5_000.0);
    near(v.remaining_buy_fee_krw, 25.0);
    near(v.realized_pnl_krw, 370.0);
    near(v.residual_unrealized_pnl_krw, 475.0);
    near(v.total_fee_krw, 100.0);
    near(v.total_tax_krw, 55.0);
    near(v.terminal_exit_fee_krw, 50.0);
    near(v.terminal_exit_tax_krw, 55.0);
    near(v.terminal_liquidation_equity_krw, 1_000_740.0);
    assert_eq!(report.summary.completed_trades, 1);
    assert_eq!(report.summary.filled_order_count, 2);
    assert_eq!(report.trades.len(), 2);
    reconciles(&report);
}

#[test]
fn no_position_has_no_hypothetical_minimum_fee_or_residual() {
    let report = run_backtest(
        "test",
        "TEST",
        false,
        SimulationAssumptions {
            minimum_fee_native: 50.0,
            ..assumptions()
        },
        &[bar(1, 1_000, None)],
    );
    near(report.accounting.terminal_exit_fee_krw, 0.0);
    near(
        report.accounting.terminal_liquidation_equity_krw,
        1_000_000.0,
    );
    assert_eq!(report.accounting.remaining_quantity, 0);
    assert!(report.accounting.warnings.iter().any(|w| w.contains("0건")));
    reconciles(&report);
}

#[test]
fn fx_changes_krw_mark_and_realized_basis_without_changing_native_strategy_entry() {
    let a = SimulationAssumptions {
        stop_loss_pct: 5.0,
        fx_scenario: vec![FxScenarioPoint {
            bar_time: "20261002".into(),
            rate_krw: 1_200.0,
        }],
        ..assumptions()
    };
    let mut session = ReplaySession::new("test", "TEST", true, a, &[], 3);
    let first = bar(1, 1_000, buy(10));
    session.begin_bar(&first);
    session.process_close(&first);
    assert_eq!(session.position_snapshot(), (10, 1_000));
    let second = bar(2, 1_000, sell(5));
    session.begin_bar(&second);
    // A 20% KRW change cannot move the native stop or entry basis.
    assert_eq!(session.position_snapshot(), (10, 1_000));
    session.process_close(&second);
    assert_eq!(session.position_snapshot(), (5, 1_000));
    let third = bar(3, 1_000, None);
    session.begin_bar(&third);
    session.process_close(&third);
    let report = session.finish();
    near(report.accounting.realized_pnl_krw, 10_000.0);
    near(report.accounting.realized_price_pnl_krw, 0.0);
    near(report.accounting.realized_fx_pnl_krw, 10_000.0);
    near(report.accounting.residual_fx_pnl_krw, 10_000.0);
    near(report.accounting.residual_price_pnl_krw, 0.0);
    near(report.summary.final_equity_krw, 1_020_000.0);
    reconciles(&report);
}

#[test]
fn future_fx_point_cannot_revalue_earlier_mark_or_buy() {
    let a = SimulationAssumptions {
        fx_scenario: vec![FxScenarioPoint {
            bar_time: "20261003".into(),
            rate_krw: 1_200.0,
        }],
        ..assumptions()
    };
    let report = run_backtest(
        "test",
        "TEST",
        true,
        a,
        &[
            bar(1, 1_000, buy(10)),
            bar(2, 1_000, None),
            bar(3, 1_000, None),
        ],
    );
    near(report.equity_curve[0].equity_krw, 1_000_000.0);
    near(report.equity_curve[1].equity_krw, 1_000_000.0);
    near(report.equity_curve[2].equity_krw, 1_020_000.0);
    near(report.trades[0].exchange_rate_krw, 1_000.0);
    reconciles(&report);
}

#[test]
fn dividend_entitlement_precedes_pending_open_and_respects_partial_exit_quantity() {
    let a = SimulationAssumptions {
        execution_model: ExecutionModel::NextOpen,
        fx_scenario: vec![FxScenarioPoint {
            bar_time: "20261003".into(),
            rate_krw: 1_200.0,
        }],
        dividend_scenario: vec![
            DividendScenarioPoint {
                bar_time: "20261002".into(),
                amount_per_share: 99.0,
                withholding_tax_bps: 1_500.0,
            },
            DividendScenarioPoint {
                bar_time: "20261003".into(),
                amount_per_share: 2.0,
                withholding_tax_bps: 1_500.0,
            },
            DividendScenarioPoint {
                bar_time: "20261004".into(),
                amount_per_share: 1.0,
                withholding_tax_bps: 1_500.0,
            },
        ],
        ..assumptions()
    };
    let report = run_backtest(
        "test",
        "TEST",
        true,
        a,
        &[
            bar(1, 1_000, buy(10)),
            bar(2, 1_000, sell(5)),
            bar(3, 1_000, None),
            bar(4, 1_000, None),
        ],
    );
    near(report.accounting.dividend_gross_krw, 30_000.0);
    near(report.accounting.dividend_tax_krw, 4_500.0);
    near(report.accounting.dividend_net_krw, 25_500.0);
    assert_eq!(report.summary.completed_trades, 1);
    assert_eq!(report.summary.filled_order_count, 2);
    near(report.summary.final_equity_krw, 1_045_500.0);
    reconciles(&report);
}

#[test]
fn observed_investment_uses_zero_samples_and_is_distinct_from_holding_bar_fraction() {
    let report = run_backtest(
        "test",
        "TEST",
        false,
        assumptions(),
        &[
            bar(1, 1_000, None),
            bar(2, 1_000, buy(10)),
            bar(3, 1_000, None),
            bar(4, 1_000, sell(10)),
        ],
    );
    near(report.summary.exposure_pct, 50.0);
    near(report.accounting.average_invested_notional_krw, 5_000.0);
    near(report.accounting.average_invested_capital_pct, 0.5);
    near(report.accounting.average_invested_equity_pct, 0.5);
    near(report.accounting.peak_invested_capital_pct, 1.0);
    near(report.accounting.peak_invested_equity_pct, 1.0);
    reconciles(&report);
}

#[test]
fn residual_loss_can_reverse_realized_profit_sign_and_trigger_specific_warning() {
    let report = run_backtest(
        "test",
        "TEST",
        false,
        assumptions(),
        &[
            bar(1, 1_000, buy(10)),
            bar(2, 1_100, sell(1)),
            bar(3, 900, None),
        ],
    );
    near(report.accounting.realized_pnl_krw, 100.0);
    near(report.accounting.residual_unrealized_pnl_krw, -900.0);
    assert!(report.accounting.realized_equity_sign_mismatch);
    reconciles(&report);
}

#[test]
fn directional_fees_and_native_minimum_are_converted_with_the_current_fx() {
    let a = SimulationAssumptions {
        buy_fee_bps: Some(100.0),
        sell_fee_bps: Some(200.0),
        minimum_fee_native: 1.0,
        ..assumptions()
    };
    let report = run_backtest(
        "test",
        "TEST",
        true,
        a,
        &[bar(1, 1_000, buy(10)), bar(2, 1_100, sell(10))],
    );
    near(report.trades[0].fee_krw, 1_000.0);
    near(report.trades[1].fee_krw, 2_200.0);
    near(report.accounting.total_fee_krw, 3_200.0);
    near(report.accounting.realized_pnl_krw, 6_800.0);
    reconciles(&report);
}

#[test]
fn scenario_validation_rejects_bad_dates_unmatched_duplicates_nonfinite_and_excess_points() {
    let allowed = vec!["20261001".into()];
    for (label, rate) in [
        ("20261002", 1_000.0),
        ("20260230", 1_000.0),
        ("20261001", f64::NAN),
        ("20261001", 99.0),
    ] {
        let a = SimulationAssumptions {
            fx_scenario: vec![FxScenarioPoint {
                bar_time: label.into(),
                rate_krw: rate,
            }],
            ..assumptions()
        };
        assert!(a.validate_scenarios(true, &allowed).is_err());
    }
    let point = FxScenarioPoint {
        bar_time: "20261001".into(),
        rate_krw: 1_000.0,
    };
    let a = SimulationAssumptions {
        fx_scenario: vec![point.clone(), point.clone()],
        ..assumptions()
    };
    assert!(a.validate_scenarios(true, &allowed).is_err());
    let a = SimulationAssumptions {
        fx_scenario: vec![point.clone(); 65],
        ..assumptions()
    };
    assert!(a.validate_scenarios(true, &allowed).is_err());
    let a = SimulationAssumptions {
        fx_scenario: vec![point],
        ..assumptions()
    };
    assert!(a.validate_scenarios(false, &allowed).is_err());
    let a = SimulationAssumptions {
        cost_source: CostSource::BrokerVerifiedManual,
        ..assumptions()
    };
    assert!(a.validate_scenarios(true, &allowed).is_err());
}

#[test]
fn invalid_batch_scenario_never_silently_executes_orders() {
    let a = SimulationAssumptions {
        fx_scenario: vec![FxScenarioPoint {
            bar_time: "20261002".into(),
            rate_krw: 1_000.0,
        }],
        ..assumptions()
    };
    let report = run_backtest("test", "TEST", true, a, &[bar(1, 1_000, buy(10))]);
    assert_eq!(report.summary.filled_order_count, 0);
    assert!(report
        .execution_warnings
        .iter()
        .any(|w| w.contains("평가 불가")));
    near(report.summary.final_equity_krw, 1_000_000.0);
}

#[test]
fn independently_derived_fx_partial_dividend_fixture_reconciles_all_cashflows() {
    let a = SimulationAssumptions {
        fee_bps: 100.0,
        fx_scenario: vec![FxScenarioPoint {
            bar_time: "20261002".into(),
            rate_krw: 1_200.0,
        }],
        dividend_scenario: vec![DividendScenarioPoint {
            bar_time: "20261002".into(),
            amount_per_share: 1.0,
            withholding_tax_bps: 1_500.0,
        }],
        dividend_price_basis: DividendPriceBasis::Unadjusted,
        ..assumptions()
    };
    let report = run_backtest(
        "test",
        "TEST",
        true,
        a,
        &[bar(1, 10_000, buy(2)), bar(2, 11_000, sell(1))],
    );
    near(report.accounting.realized_pnl_krw, 29_680.0);
    near(report.accounting.residual_unrealized_pnl_krw, 31_000.0);
    near(report.accounting.dividend_gross_krw, 2_400.0);
    near(report.accounting.dividend_tax_krw, 360.0);
    near(report.accounting.dividend_net_krw, 2_040.0);
    near(report.accounting.realized_price_pnl_krw, 10_000.0);
    near(report.accounting.realized_fx_pnl_krw, 22_000.0);
    near(report.accounting.residual_price_pnl_krw, 10_000.0);
    near(report.accounting.residual_fx_pnl_krw, 22_000.0);
    near(report.summary.final_equity_krw, 1_062_720.0);
    near(report.trades[0].execution_reference_price.unwrap(), 100.0);
    near(report.trades[1].execution_reference_price.unwrap(), 110.0);
    reconciles(&report);
}

#[test]
fn slippage_is_embedded_once_in_cost_basis_and_terminal_projection() {
    let a = SimulationAssumptions {
        slippage_bps: 100.0,
        fee_bps: 100.0,
        tax_bps: 100.0,
        ..assumptions()
    };
    let report = run_backtest(
        "test",
        "TEST",
        false,
        a,
        &[bar(1, 1_000, buy(10)), bar(2, 1_100, sell(5))],
    );
    near(report.trades[0].fill_price.unwrap(), 1_010.0);
    near(report.trades[0].slippage_krw, 100.0);
    near(report.trades[1].fill_price.unwrap(), 1_089.0);
    near(report.trades[1].slippage_krw, 55.0);
    near(report.accounting.total_slippage_krw, 155.0);
    near(report.accounting.remaining_buy_fee_krw, 50.5);
    near(report.accounting.realized_pnl_krw, 235.6);
    near(report.accounting.residual_unrealized_pnl_krw, 399.5);
    near(report.accounting.terminal_exit_slippage_krw, 55.0);
    near(report.accounting.terminal_exit_fee_krw, 54.45);
    near(report.accounting.terminal_exit_tax_krw, 54.45);
    near(
        report.accounting.terminal_liquidation_equity_krw,
        1_000_471.2,
    );
    reconciles(&report);
}

#[test]
fn dividend_adjusted_prices_reject_known_double_counting() {
    let a = SimulationAssumptions {
        dividend_price_basis: DividendPriceBasis::DividendAdjusted,
        dividend_scenario: vec![DividendScenarioPoint {
            bar_time: "20261001".into(),
            amount_per_share: 1.0,
            withholding_tax_bps: 0.0,
        }],
        ..assumptions()
    };
    assert!(a
        .validate_scenarios(false, &["20261001".into()])
        .unwrap_err()
        .contains("중복"));
}

fn ordinary_sell(quantity: u64) -> Option<Signal> {
    Some(Signal::Sell {
        symbol: "TEST".into(),
        quantity,
        reason: "ordinary exit".into(),
    })
}

#[test]
fn minimum_commission_blocks_small_native_profit_but_zero_minimum_keeps_legacy_path() {
    let bars = [bar(1, 10_000, buy(1)), bar(2, 10_100, ordinary_sell(1))];
    let report = run_backtest(
        "test",
        "TEST",
        false,
        SimulationAssumptions {
            minimum_fee_native: 1_000.0,
            ..assumptions()
        },
        &bars,
    );
    assert_eq!(report.trades[1].status, "blocked");
    assert!(report.trades[1]
        .blocked_reason
        .as_ref()
        .unwrap()
        .contains("최소 수수료"));
    assert_eq!(report.summary.completed_trades, 0);
    assert_eq!(report.accounting.remaining_quantity, 1);
    near(report.accounting.terminal_liquidation_equity_krw, 998_100.0);
    let legacy = run_backtest("test", "TEST", false, assumptions(), &bars);
    assert_eq!(legacy.trades[1].status, "filled");
    near(legacy.accounting.realized_pnl_krw, 100.0);
    reconciles(&report);
}

#[test]
fn minimum_fee_profit_gate_uses_requested_partial_quantity_and_remaining_buy_fee() {
    let report = run_backtest(
        "test",
        "TEST",
        false,
        SimulationAssumptions {
            minimum_fee_native: 1_000.0,
            ..assumptions()
        },
        &[
            bar(1, 10_000, buy(10)),
            bar(2, 10_300, ordinary_sell(1)),
            bar(3, 10_300, ordinary_sell(10)),
        ],
    );
    // One-share sale would earn 300 - 100 allocated entry fee - 1000 exit fee.
    assert_eq!(report.trades[1].status, "blocked");
    // The rejected partial exit has no effect on the ten-share cost allocation.
    assert_eq!(report.trades[2].status, "filled");
    assert_eq!(report.trades[2].quantity, 10);
    near(report.accounting.realized_pnl_krw, 1_000.0);
    assert_eq!(report.summary.completed_trades, 1);
    reconciles(&report);
}

#[test]
fn oversized_sell_request_is_clamped_before_minimum_fee_profit_projection() {
    let report = run_backtest(
        "test",
        "TEST",
        false,
        SimulationAssumptions {
            minimum_fee_native: 1_000.0,
            ..assumptions()
        },
        &[
            bar(1, 10_000, buy(2)),
            bar(2, 11_000, ordinary_sell(100)),
            bar(3, 11_001, ordinary_sell(100)),
        ],
    );
    assert_eq!(report.trades[1].status, "blocked"); // exactly zero after both fees
    assert_eq!(report.trades[2].status, "filled");
    assert_eq!(report.trades[2].quantity, 2);
    near(report.accounting.realized_pnl_krw, 2.0);
    reconciles(&report);
}

#[test]
fn minimum_fee_profit_gate_converts_exit_minimum_at_current_fx_and_preserves_entry_basis() {
    let report = run_backtest(
        "test",
        "TEST",
        true,
        SimulationAssumptions {
            minimum_fee_native: 1.0,
            fx_scenario: vec![
                FxScenarioPoint {
                    bar_time: "20261002".into(),
                    rate_krw: 1_200.0,
                },
                FxScenarioPoint {
                    bar_time: "20261003".into(),
                    rate_krw: 1_300.0,
                },
            ],
            ..assumptions()
        },
        &[
            bar(1, 1_000, buy(1)),
            bar(2, 1_010, ordinary_sell(1)),
            bar(3, 1_010, ordinary_sell(1)),
        ],
    );
    // 12,120 - 10,000 historical gross - 1,000 buy fee - 1,200 sell fee = -80.
    assert_eq!(report.trades[1].status, "blocked");
    assert_eq!(report.trades[2].status, "filled");
    near(report.trades[2].fee_krw, 1_300.0);
    near(report.accounting.realized_pnl_krw, 830.0);
    reconciles(&report);
}

#[test]
fn minimum_fee_mode_uses_krw_settlement_instead_of_native_only_bps_profit_estimate() {
    let report = run_backtest(
        "test",
        "TEST",
        true,
        SimulationAssumptions {
            minimum_fee_native: 1.0,
            fee_bps: 100.0,
            fx_scenario: vec![FxScenarioPoint {
                bar_time: "20261002".into(),
                rate_krw: 1_300.0,
            }],
            ..assumptions()
        },
        &[bar(1, 1_000, buy(1)), bar(2, 1_010, ordinary_sell(1))],
    );
    // Native +1% is below the static 2% roundtrip approximation, but actual
    // historical-KRW settlement earns +830 after the minimum commissions.
    assert_eq!(report.trades[1].status, "filled");
    near(report.accounting.realized_pnl_krw, 830.0);
    reconciles(&report);
}

#[test]
fn minimum_fee_gate_preserves_native_loss_exits_even_with_positive_fx_pnl() {
    for (fx, expected_pnl) in [(1_000.0, -3_000.0), (1_500.0, 1_000.0)] {
        let report = run_backtest(
            "test",
            "TEST",
            true,
            SimulationAssumptions {
                minimum_fee_native: 1.0,
                fx_scenario: vec![FxScenarioPoint {
                    bar_time: "20261002".into(),
                    rate_krw: fx,
                }],
                ..assumptions()
            },
            &[bar(1, 1_000, buy(1)), bar(2, 900, ordinary_sell(1))],
        );
        assert_eq!(report.trades[1].status, "filled");
        near(report.accounting.realized_pnl_krw, expected_pnl);
        reconciles(&report);
    }
}

#[test]
fn minimum_fee_gate_exempts_explicit_stop_and_research_protection() {
    let stopped = run_backtest(
        "test",
        "TEST",
        false,
        SimulationAssumptions {
            minimum_fee_native: 1_000.0,
            ..assumptions()
        },
        &[bar(1, 10_000, buy(1)), bar(2, 10_100, sell(1))],
    );
    assert_eq!(stopped.trades[1].status, "filled");
    near(stopped.accounting.realized_pnl_krw, -1_900.0);
    let protected = run_backtest(
        "test",
        "TEST",
        false,
        SimulationAssumptions {
            minimum_fee_native: 1_000.0,
            take_profit_pct: 1.0,
            ..assumptions()
        },
        &[bar(1, 10_000, buy(1)), bar(2, 10_100, None)],
    );
    assert_eq!(protected.trades[1].status, "filled");
    near(protected.accounting.realized_pnl_krw, -1_900.0);
    reconciles(&stopped);
    reconciles(&protected);
}

#[test]
fn minimum_fee_projection_counts_slippage_and_tax_before_accepting_profit_exit() {
    let report = run_backtest(
        "test",
        "TEST",
        false,
        SimulationAssumptions {
            minimum_fee_native: 1.0,
            slippage_bps: 100.0,
            tax_bps: 100.0,
            ..assumptions()
        },
        &[bar(1, 10_000, buy(1)), bar(2, 10_300, ordinary_sell(1))],
    );
    // Entry gross 10100; exit fill 10197, tax101.97 and both minimum fees.
    // Gross profit97 is insufficient after tax even though native price is up.
    assert_eq!(report.trades[1].status, "blocked");
    assert_eq!(report.summary.completed_trades, 0);
    reconciles(&report);
}
