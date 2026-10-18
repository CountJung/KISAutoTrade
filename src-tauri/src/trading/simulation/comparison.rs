//! Comparators disclose their quantity/exposure policy; none is an independent strategy test.
use super::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyHoldComparisonView {
    pub requested_quantity: u64,
    pub actual_entry_quantity: u64,
    pub remaining_quantity: u64,
    pub final_equity_krw: f64,
    pub cumulative_return_pct: f64,
    pub terminal_liquidation_return_pct: f64,
    pub realized_pnl_krw: f64,
    pub residual_unrealized_pnl_krw: f64,
    pub dividend_net_krw: f64,
    pub mdd_pct: f64,
    pub average_invested_capital_pct: f64,
    pub average_invested_equity_pct: f64,
    pub peak_invested_capital_pct: f64,
    pub peak_invested_equity_pct: f64,
    pub holding_bar_pct: f64,
    pub filled_order_count: usize,
    pub blocked_order_count: usize,
    pub unfilled_order_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostRestoredReferenceView {
    pub restored_cost_krw: f64,
    pub final_equity_krw: f64,
    pub cumulative_return_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayComparisonView {
    pub same_input_quantity_buy_hold: BuyHoldComparisonView,
    pub same_execution_path_cost_reference: CostRestoredReferenceView,
    pub capital_exposure_difference_pct: f64,
    pub holding_bar_difference_pct: f64,
    pub limitations: Vec<String>,
}

/// Execute a first-evaluation-close buy instruction with the same engine, capital,
/// requested quantity, costs, FX/dividends, ATR history and risk constraints.
/// No protective overlay or strategy exit is applied to this buy-and-hold reference.
pub fn attach_comparison(
    report: &mut BacktestReportView,
    strategy_id: &str,
    symbol: &str,
    is_overseas: bool,
    requested_quantity: u64,
    history: &[SimulationEvent],
    events: &[SimulationEvent],
) {
    if events.is_empty() {
        return;
    }
    let mut assumptions = report.assumptions.clone();
    assumptions.stop_loss_pct = 0.0;
    assumptions.take_profit_pct = 0.0;
    let mut benchmark_events = events.to_vec();
    for event in &mut benchmark_events {
        event.signal = None;
    }
    benchmark_events[0].signal = Some(Signal::Buy {
        symbol: symbol.into(),
        quantity: requested_quantity,
        reason: "비교 기준: 첫 평가 종가 매수 지시 후 보유".into(),
    });
    let benchmark = run_backtest_with_history(
        strategy_id,
        symbol,
        is_overseas,
        assumptions,
        history,
        &benchmark_events,
    );
    let restored_cost = report.accounting.total_fee_krw
        + report.accounting.total_tax_krw
        + report.accounting.total_slippage_krw
        + report.accounting.dividend_tax_krw;
    let restored_equity = report.summary.final_equity_krw + restored_cost;
    let exposure_difference = report.accounting.average_invested_capital_pct
        - benchmark.accounting.average_invested_capital_pct;
    let holding_difference = report.summary.exposure_pct - benchmark.summary.exposure_pct;
    report.comparison = Some(ReplayComparisonView {
        same_input_quantity_buy_hold: BuyHoldComparisonView {
            requested_quantity,
            actual_entry_quantity: benchmark.trades.iter()
                .filter(|trade| trade.status == "filled" && trade.side == "buy")
                .map(|trade| trade.quantity).sum(),
            remaining_quantity: benchmark.accounting.remaining_quantity,
            final_equity_krw: benchmark.summary.final_equity_krw,
            cumulative_return_pct: benchmark.summary.cumulative_return_pct,
            terminal_liquidation_return_pct: benchmark.accounting.terminal_liquidation_return_pct,
            realized_pnl_krw: benchmark.accounting.realized_pnl_krw,
            residual_unrealized_pnl_krw: benchmark.accounting.residual_unrealized_pnl_krw,
            dividend_net_krw: benchmark.accounting.dividend_net_krw,
            mdd_pct: benchmark.summary.mdd_pct,
            average_invested_capital_pct: benchmark.accounting.average_invested_capital_pct,
            average_invested_equity_pct: benchmark.accounting.average_invested_equity_pct,
            peak_invested_capital_pct: benchmark.accounting.peak_invested_capital_pct,
            peak_invested_equity_pct: benchmark.accounting.peak_invested_equity_pct,
            holding_bar_pct: benchmark.summary.exposure_pct,
            filled_order_count: benchmark.summary.filled_order_count,
            blocked_order_count: benchmark.summary.blocked_order_count,
            unfilled_order_count: benchmark.summary.unfilled_order_count,
        },
        same_execution_path_cost_reference: CostRestoredReferenceView {
            restored_cost_krw: round_money(restored_cost),
            final_equity_krw: round_money(restored_equity),
            cumulative_return_pct: round_metric(return_pct(
                report.summary.initial_capital_krw, restored_equity,
            )),
        },
        capital_exposure_difference_pct: round_metric(exposure_difference),
        holding_bar_difference_pct: round_metric(holding_difference),
        limitations: vec![
            "단순 보유는 같은 평가 구간·자본·요청 수량·체결/비용/FX/배당·ATR/리스크 조건의 첫 종가 지시입니다. 실제 체결 수량·보유기간·평균 투자비중은 전략과 다를 수 있고 연구 보호·전략 청산은 적용하지 않습니다.".into(),
            "동일 체결경로 비용 복원은 실제 수량·시점·노출을 고정하고 발생 당시 기록된 원화 비용만 환급한 사후 산술 참조입니다. 원가/환율 귀속을 재산정하거나 비용을 바꿔 전략을 재실행한 무비용 성과가 아닙니다.".into(),
            "평균 투자비중은 관측 봉말 기준이며 누락 구간·봉 내부의 시간 가중 투자비중이 아닙니다. 부분 분기와 겹치는 구간은 독립 표본으로 합산하지 않습니다.".into(),
        ],
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparator_preserves_actual_trades_and_discloses_final_pending() {
        let events = [SimulationEvent {
            time: "20261001".into(),
            chart_time: "20261001".into(),
            open_units: 100,
            close_units: 100,
            high_units: 100,
            low_units: 100,
            signal: None,
        }];
        let mut report = run_backtest(
            "test",
            "TEST",
            false,
            SimulationAssumptions {
                execution_model: ExecutionModel::NextOpen,
                ..SimulationAssumptions::default()
            },
            &events,
        );
        let before = serde_json::to_value(&report.trades).unwrap();
        attach_comparison(&mut report, "test", "TEST", false, 1, &[], &events);
        assert_eq!(serde_json::to_value(&report.trades).unwrap(), before);
        let comparison = report.comparison.unwrap();
        assert_eq!(
            comparison.same_input_quantity_buy_hold.unfilled_order_count,
            1
        );
        assert_eq!(
            comparison
                .same_input_quantity_buy_hold
                .actual_entry_quantity,
            0
        );
        assert_eq!(
            comparison
                .same_execution_path_cost_reference
                .restored_cost_krw,
            0.0
        );
    }
    #[test]
    fn same_quantity_reference_discloses_exposure_and_restores_only_recorded_costs() {
        let events = [
            (
                100,
                Some(Signal::Buy {
                    symbol: "TEST".into(),
                    quantity: 2,
                    reason: "entry".into(),
                }),
            ),
            (
                90,
                Some(Signal::Sell {
                    symbol: "TEST".into(),
                    quantity: 1,
                    reason: "손절 fixture".into(),
                }),
            ),
            (120, None),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (price, signal))| SimulationEvent {
            time: format!("202610{:02}", 1 + index),
            chart_time: format!("202610{:02}", 1 + index),
            open_units: price,
            close_units: price,
            high_units: price,
            low_units: price,
            signal,
        })
        .collect::<Vec<_>>();
        let mut report = run_backtest(
            "fixture",
            "TEST",
            false,
            SimulationAssumptions {
                initial_capital_krw: 10000.0,
                fee_bps: 100.0,
                tax_bps: 0.0,
                slippage_bps: 0.0,
                ..SimulationAssumptions::default()
            },
            &events,
        );
        assert_eq!(report.summary.final_equity_krw, 10007.1);
        assert_eq!(report.accounting.realized_pnl_krw, -11.9);
        assert_eq!(report.accounting.residual_unrealized_pnl_krw, 19.0);
        assert!(report.accounting.realized_equity_sign_mismatch);
        let before = serde_json::to_value(&report.trades).unwrap();
        attach_comparison(&mut report, "fixture", "TEST", false, 2, &[], &events);
        assert_eq!(serde_json::to_value(&report.trades).unwrap(), before);
        let comparison = report.comparison.unwrap();
        assert_eq!(
            comparison.same_input_quantity_buy_hold.requested_quantity,
            2
        );
        assert_eq!(
            comparison
                .same_input_quantity_buy_hold
                .actual_entry_quantity,
            2
        );
        assert_eq!(
            comparison.same_input_quantity_buy_hold.remaining_quantity,
            2
        );
        assert!(comparison.capital_exposure_difference_pct < 0.0);
        assert_eq!(comparison.holding_bar_difference_pct, 0.0);
        assert_eq!(
            comparison
                .same_execution_path_cost_reference
                .restored_cost_krw,
            2.9
        );
        assert_eq!(
            comparison
                .same_execution_path_cost_reference
                .final_equity_krw,
            10010.0
        );
        assert!(comparison
            .limitations
            .iter()
            .any(|value| value.contains("재실행")));
    }
}
