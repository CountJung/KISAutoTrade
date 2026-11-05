use super::*;

pub(super) struct SaleProjection {
    pub(super) quantity: u64,
    pub(super) fill_krw: f64,
    pub(super) gross: f64,
    pub(super) fee: f64,
    pub(super) tax: f64,
    pub(super) buy_cost: f64,
    pub(super) pnl: f64,
}

impl ReplaySession {
    /// The same prospective settlement is used by the minimum-fee guard and the
    /// actual ledger, so partial quantities and historical entry FX cannot drift.
    pub(super) fn project_sale(
        &self,
        position: &PositionState,
        requested: u64,
        reference_krw: f64,
    ) -> SaleProjection {
        let quantity = requested.min(position.quantity);
        let fill_krw = apply_slippage(reference_krw, self.assumptions.slippage_bps, false);
        let gross = fill_krw * quantity as f64;
        let fee = self.assumptions.fee(gross, false, self.current_fx);
        let tax = bps_cost(gross, self.assumptions.tax_bps);
        let buy_cost = position.allocated_buy_cost_krw * quantity as f64 / position.quantity as f64;
        let pnl = (fill_krw - position.average_fill_krw) * quantity as f64 - fee - tax - buy_cost;
        SaleProjection {
            quantity,
            fill_krw,
            gross,
            fee,
            tax,
            buy_cost,
            pnl,
        }
    }

    pub(super) fn minimum_fee_profit_block_reason(
        &self,
        signal: &Signal,
        price_units: u64,
        protective: bool,
        projection: Option<&SaleProjection>,
    ) -> Option<String> {
        if self.assumptions.minimum_fee_native <= 0.0 || protective {
            return None;
        }
        let Signal::Sell { reason, .. } = signal else {
            return None;
        };
        if reason.contains("손절") || reason.contains("stop") || reason.contains("Stop") {
            return None;
        }
        let position = self.position.as_ref()?;
        // Preserve the native-price loss-exit exemption even when FX appreciation
        // makes the KRW result positive. A risk-reduction sell must remain possible.
        if price_units as f64 <= position.average_fill_units {
            return None;
        }
        // Replay has no separate minimum-profit input: TradeGuardConfig's default
        // min_expected_profit_bps is 0, so strictly positive net P&L is required.
        if projection.is_some_and(|sale| sale.pnl <= 0.0) {
            Some("최소 수수료·잔여 매수 비용·세금·슬리피지 차감 후 기대 순익이 0 이하".into())
        } else {
            None
        }
    }

    pub(super) fn finish_accounting(&self, final_equity: f64) -> ReplayAccountingView {
        let mut view = self.accounting.clone();
        view.realized_pnl_krw = self.completed_pnls.iter().map(|(_, pnl, _)| pnl).sum();
        for trade in self.trades.iter().filter(|trade| trade.status == "filled") {
            view.total_fee_krw += trade.fee_krw;
            view.total_tax_krw += trade.tax_krw;
            view.total_slippage_krw += trade.slippage_krw;
        }
        if let Some(position) = &self.position {
            let quantity = position.quantity as f64;
            let mark_krw = self.krw(self.last_close);
            let entry_native =
                position.average_fill_units / if self.is_overseas { 100.0 } else { 1.0 };
            let entry_fx = position.average_fill_krw / entry_native;
            let mark_native = mark_krw / self.current_fx;
            view.remaining_quantity = position.quantity;
            view.remaining_cost_basis_krw = position.average_fill_krw * quantity;
            view.remaining_buy_fee_krw = position.allocated_buy_cost_krw;
            view.residual_unrealized_pnl_krw =
                (mark_krw - position.average_fill_krw) * quantity - position.allocated_buy_cost_krw;
            view.residual_price_pnl_krw = (mark_native - entry_native) * entry_fx * quantity;
            view.residual_fx_pnl_krw = mark_native * (self.current_fx - entry_fx) * quantity;
            let liquidation_fill = apply_slippage(mark_krw, self.assumptions.slippage_bps, false);
            let gross = liquidation_fill * quantity;
            view.terminal_exit_slippage_krw = (mark_krw - liquidation_fill).max(0.0) * quantity;
            view.terminal_exit_fee_krw = self.assumptions.fee(gross, false, self.current_fx);
            view.terminal_exit_tax_krw = bps_cost(gross, self.assumptions.tax_bps);
        }
        view.terminal_liquidation_equity_krw = final_equity
            - view.terminal_exit_slippage_krw
            - view.terminal_exit_fee_krw
            - view.terminal_exit_tax_krw;
        view.terminal_liquidation_return_pct = return_pct(
            self.assumptions.initial_capital_krw,
            view.terminal_liquidation_equity_krw,
        );
        if self.index > 0 {
            view.average_invested_notional_krw = self.invested_notional_sum / self.index as f64;
            view.average_invested_capital_pct =
                view.average_invested_notional_krw / self.assumptions.initial_capital_krw * 100.0;
            view.average_invested_equity_pct = self.invested_equity_pct_sum / self.index as f64;
        }
        view.realized_equity_sign_mismatch =
            view.realized_pnl_krw * (final_equity - self.assumptions.initial_capital_krw) < 0.0;
        if view.realized_equity_sign_mismatch {
            view.warnings.push("실현 손익과 총자산 증감의 부호가 다릅니다. 잔여 평가 손익·매수 비용·배당을 함께 확인하세요.".into());
        }
        if self.completed_pnls.len() < 20 {
            view.warnings.push(format!("완료 청산 표본 {}건: 승률·profit factor의 전략 일반화 근거가 부족합니다. 부분 청산도 각 1건으로 셉니다.", self.completed_pnls.len()));
        }
        view.warnings.push("투자 비중은 각 관측 봉 종가의 평가금액 평균/최댓값입니다. 보유 봉 비율·시간 가중 노출·장중 최대 비중과 다릅니다.".into());
        view.warnings.push("종료 청산 값은 마지막 관측 종가의 추가 매도 비용을 차감한 가정입니다. 실제 주문·체결 수·승률·리스크 상태를 변경하지 않습니다.".into());
        if self.assumptions.minimum_fee_native > 0.0 {
            view.warnings.push("최소 수수료 시나리오의 일반 이익 매도는 실제 청산 수량·잔여 매수 비용·현재 환율의 세후 기대 순익이 양수일 때만 허용합니다. native 가격 손실 매도·명시 손절·연구 보호 청산은 이 순익 제한에서 제외하며 시간·리스크 검사는 유지합니다.".into());
        }
        if self.is_overseas {
            view.warnings.push("해외 시나리오는 거래·배당 시점마다 원화로 환산하는 KRW 현금 장부입니다. 유휴 현금을 달러로 보유·재평가하는 실제 브로커 장부가 아닙니다.".into());
            if self.assumptions.fx_scenario.is_empty() {
                view.warnings.push(
                    "환율 시나리오 없음: 입력한 단일 고정 환율을 전체 기간에 적용했습니다.".into(),
                );
            }
        }
        if !self.assumptions.dividend_scenario.is_empty() {
            view.warnings.push(if self.assumptions.dividend_price_basis == DividendPriceBasis::Unverified {
                "입력 가격의 배당 조정 여부가 미확인입니다. 현금 배당을 더하면 중복 계상할 위험이 있습니다.".into()
            } else {
                "배당 미조정 가격이라는 입력 자료의 선언을 사용했습니다. 프로그램이 원본 가격 조정 여부를 자동 검증하지 않았습니다.".into()
            });
        }
        view.warnings.push("가격/환율 손익 분해는 진입 환율로 가격 변화를 평가하고 최종 native 금액에 환율 변화를 적용합니다. 매수·매도 수수료는 순손익에 별도 반영되며 슬리피지는 체결 가격에 포함됩니다.".into());
        if self.assumptions.dividend_scenario.is_empty() {
            view.warnings
                .push("배당 시나리오 없음: 가격 기반 수익이며 배당 포함 총수익이 아닙니다.".into());
        } else {
            view.warnings.push("배당은 해당 봉 시작 직전 보유 수량에 귀속하며 같은 봉 대기 매수는 제외합니다. 시나리오 봉에서 즉시 세후 원화 현금으로 지급하는 가정이며 실제 배당락·지급일·통화를 검증하지 않습니다.".into());
        }
        view.warnings.push(if self.assumptions.cost_source == CostSource::BrokerVerifiedManual {
            "비용 출처는 사용자의 수동 확인 표시입니다. 프로그램이 브로커 요율·최소 수수료·환전·기타 부과금을 자동 검증하지 않았습니다.".into()
        } else {
            "비용은 추정치입니다. 실계좌 요율·방향별 수수료·최소 수수료·세금·환전 비용은 브로커 자료로 별도 확인해야 합니다.".into()
        });
        if self.applied_fx_points != self.assumptions.fx_scenario.len()
            || self.applied_dividend_points != self.assumptions.dividend_scenario.len()
        {
            view.warnings.push("시나리오 평가 불가: 관측 평가 봉에 적용되지 않은 환율/배당 시각이 있습니다. 이 결과를 유효한 시나리오 성과로 사용하지 마세요.".into());
        }
        // Round only at the presentation boundary; the ledger uses full precision.
        for value in [
            &mut view.realized_pnl_krw,
            &mut view.residual_unrealized_pnl_krw,
            &mut view.remaining_cost_basis_krw,
            &mut view.remaining_buy_fee_krw,
            &mut view.realized_price_pnl_krw,
            &mut view.realized_fx_pnl_krw,
            &mut view.residual_price_pnl_krw,
            &mut view.residual_fx_pnl_krw,
            &mut view.total_fee_krw,
            &mut view.total_tax_krw,
            &mut view.total_slippage_krw,
            &mut view.dividend_gross_krw,
            &mut view.dividend_tax_krw,
            &mut view.dividend_net_krw,
            &mut view.terminal_liquidation_equity_krw,
            &mut view.terminal_exit_fee_krw,
            &mut view.terminal_exit_tax_krw,
            &mut view.terminal_exit_slippage_krw,
            &mut view.average_invested_notional_krw,
        ] {
            *value = round_money(*value);
        }
        for value in [
            &mut view.terminal_liquidation_return_pct,
            &mut view.average_invested_capital_pct,
            &mut view.average_invested_equity_pct,
            &mut view.peak_invested_capital_pct,
            &mut view.peak_invested_equity_pct,
        ] {
            *value = round_metric(*value);
        }
        view
    }
}
