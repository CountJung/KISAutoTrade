//! One accounting engine for batch replay and strategy-feedback streaming replay.
use super::*;

struct PendingSignal {
    signal: Signal,
    trade_index: usize,
}

/// Causal bar lifecycle: begin_bar → position feedback → strategy close signal →
/// process_close → position feedback. No process-global state or provider access.
pub struct ReplaySession {
    strategy_id: String,
    symbol: String,
    is_overseas: bool,
    assumptions: SimulationAssumptions,
    risk: RiskManager,
    guard: TradeGuard,
    scope: BrokerScope,
    split_index: usize,
    total_bars: usize,
    index: usize,
    cash: f64,
    position: Option<PositionState>,
    pending: Option<PendingSignal>,
    trades: Vec<BacktestTradeView>,
    equity_curve: Vec<EquityPointView>,
    completed_pnls: Vec<(String, f64, String)>,
    turnover_krw: f64,
    exposed_events: usize,
    signal_count: usize,
    eligible_count: usize,
    filled_count: usize,
    ambiguous_bar_count: usize,
    atr_window: AtrWindow,
    atr_preparation: AtrPreparationView,
    last_close: u64,
    execution_warnings: Vec<String>,
}

impl ReplaySession {
    pub fn new(
        strategy_id: &str,
        symbol: &str,
        is_overseas: bool,
        assumptions: SimulationAssumptions,
        history_events: &[SimulationEvent],
        total_bars: usize,
    ) -> Self {
        let assumptions = assumptions.normalized(is_overseas);
        let mut risk = RiskManager::new(
            assumptions.daily_loss_limit_krw.round() as i64,
            assumptions.max_position_ratio,
        );
        risk.volatility_sizing_enabled = assumptions.volatility_sizing_enabled;
        risk.risk_per_trade_bps = assumptions.risk_per_trade_bps;
        risk.atr_stop_multiplier = assumptions.atr_stop_multiplier;
        risk.max_daily_sell_orders_per_symbol = 0;
        let rounded_bps = |value: f64| value.round() as i32;
        let guard = TradeGuard::new(TradeGuardConfig {
            domestic_slippage_bps: rounded_bps(
                assumptions.fee_bps * 2.0 + assumptions.tax_bps + assumptions.slippage_bps,
            ) - 22,
            overseas_fee_bps: rounded_bps(assumptions.fee_bps * 2.0 + assumptions.tax_bps),
            overseas_slippage_bps: rounded_bps(assumptions.slippage_bps),
            ..TradeGuardConfig::default()
        });
        let split_index = if assumptions.in_sample_percent >= 100 {
            total_bars
        } else {
            total_bars.saturating_mul(assumptions.in_sample_percent as usize) / 100
        };
        let mut atr_window = AtrWindow::default();
        for event in history_events {
            atr_window.observe(event);
        }
        let atr_preparation = AtrWindow::preparation(history_events.len(), atr_window.ready());
        let mut execution_warnings = vec![
            "OHLC 봉에는 체결 순서·호가·유동성이 없으며 슬리피지는 고정 bps 근사입니다. 봉 시각과 open/close/intrabar 단계는 실제 거래소 체결 시각이 아닙니다.".into(),
        ];
        if assumptions.execution_model == ExecutionModel::SameClose {
            execution_warnings.push("종가 신호를 같은 종가에 체결하는 가정은 신호 계산 후 실제 체결 가능성을 검증하지 않습니다.".into());
        } else {
            execution_warnings.push("종가에서 확정된 신호를 다음 관측 봉 시가에 체결합니다. 마지막 미체결 신호는 주문 거절·체결에 포함하지 않습니다.".into());
        }
        if assumptions.stop_loss_pct > 0.0 || assumptions.take_profit_pct > 0.0 {
            execution_warnings.push("손절·익절은 실제 체결 원가 기준의 별도 연구 시나리오이며 전략 자체의 손절 로직과 독립입니다. OHLC 장중 경로는 검증할 수 없습니다.".into());
        }
        Self {
            cash: assumptions.initial_capital_krw,
            strategy_id: strategy_id.into(),
            symbol: symbol.into(),
            is_overseas,
            assumptions,
            risk,
            guard,
            scope: BrokerScope::kis_legacy(),
            split_index,
            total_bars,
            index: 0,
            position: None,
            pending: None,
            trades: Vec::new(),
            equity_curve: Vec::with_capacity(total_bars),
            completed_pnls: Vec::new(),
            turnover_krw: 0.0,
            exposed_events: 0,
            signal_count: 0,
            eligible_count: 0,
            filled_count: 0,
            ambiguous_bar_count: 0,
            atr_window,
            atr_preparation,
            last_close: 0,
            execution_warnings,
        }
    }

    /// Execute only the previous close's queued signal using this bar's open and
    /// prior completed bars' ATR. Then evaluate the optional protective scenario.
    pub fn begin_bar(&mut self, event: &SimulationEvent) {
        let date = parse_replay_time(&event.time, self.is_overseas).date_naive();
        if self.risk.reset_for_date(date) && self.risk.is_emergency_stop() {
            self.risk.clear_emergency_stop();
        }
        self.publish_atr();
        if let Some(pending) = self.pending.take() {
            self.execute(
                pending.signal,
                pending.trade_index,
                event,
                event.open_units,
                "open",
                false,
            );
        }
        self.protect_position(event);
    }

    pub fn position_snapshot(&self) -> (u64, u64) {
        self.position.as_ref().map_or((0, 0), |position| {
            (
                position.quantity,
                krw_to_units(
                    position.average_fill_krw,
                    self.is_overseas,
                    self.assumptions.exchange_rate_krw,
                )
                .unwrap_or(0),
            )
        })
    }

    /// Observe the completed bar before sizing same-close orders. Next-open
    /// orders remain pending until begin_bar of the next observed bar.
    pub fn process_close(&mut self, event: &SimulationEvent) {
        self.atr_window.observe(event);
        if self.atr_window.ready() {
            self.atr_preparation
                .first_ready_time
                .get_or_insert_with(|| event.time.clone());
        } else {
            self.atr_preparation.unready_evaluation_bars += 1;
        }
        self.publish_atr();
        if let Some(signal) = &event.signal {
            self.signal_count += 1;
            let trade_index = self.push_trade(signal, event, event.close_units, "close");
            if self.assumptions.execution_model == ExecutionModel::NextOpen {
                self.trades[trade_index].status = "unfilled".into();
                self.pending = Some(PendingSignal {
                    signal: signal.clone(),
                    trade_index,
                });
            } else {
                self.execute(
                    signal.clone(),
                    trade_index,
                    event,
                    event.close_units,
                    "close",
                    false,
                );
            }
        }
        if self.position.is_some() {
            self.exposed_events += 1;
        }
        self.last_close = event.close_units;
        let equity = current_equity(
            self.cash,
            self.position.as_ref(),
            self.krw(event.close_units),
        );
        let peak = self
            .equity_curve
            .iter()
            .map(|p| p.equity_krw)
            .fold(self.assumptions.initial_capital_krw, f64::max);
        let drawdown = if peak > 0.0 {
            ((peak - equity) / peak * 100.0).max(0.0)
        } else {
            0.0
        };
        self.equity_curve.push(EquityPointView {
            time: event.chart_time.clone(),
            equity_krw: round_money(equity),
            drawdown_pct: round_metric(drawdown),
            phase: self.phase(),
        });
        self.index += 1;
    }

    fn publish_atr(&mut self) {
        if let Some(atr) = self.atr_window.value().filter(|value| *value > 0) {
            self.risk.set_symbol_atr(&self.symbol, atr);
        }
    }

    fn krw(&self, units: u64) -> f64 {
        units_to_krw(units, self.is_overseas, self.assumptions.exchange_rate_krw)
    }

    fn phase(&self) -> String {
        phase_for_index(self.index, self.split_index, self.total_bars).into()
    }

    fn push_trade(
        &mut self,
        signal: &Signal,
        event: &SimulationEvent,
        price: u64,
        clock_phase: &str,
    ) -> usize {
        let (side, quantity, reason) = signal_parts(signal);
        let index = self.trades.len();
        self.trades.push(BacktestTradeView {
            signal_available_at: Some(ReplayEventTime {
                bar_time: event.time.clone(),
                phase: clock_phase.into(),
            }),
            fill_at: None,
            intrabar_path_status: None,
            time: event.time.clone(),
            side: side.into(),
            signal_price: units_to_display(price, self.is_overseas),
            fill_price: None,
            quantity,
            gross_amount_krw: 0.0,
            cost_krw: 0.0,
            realized_pnl_krw: None,
            status: "blocked".into(),
            reason: reason.into(),
            blocked_reason: None,
            phase: self.phase(),
        });
        index
    }

    fn execute(
        &mut self,
        signal: Signal,
        trade_index: usize,
        event: &SimulationEvent,
        price_units: u64,
        clock_phase: &str,
        protective: bool,
    ) {
        let now = parse_replay_time(&event.time, self.is_overseas);
        let held = self
            .position
            .as_ref()
            .map_or(0, |position| position.quantity);
        let average_units = self.position.as_ref().and_then(|position| {
            krw_to_units(
                position.average_fill_krw,
                self.is_overseas,
                self.assumptions.exchange_rate_krw,
            )
        });
        let mut blocked_reason = if price_units == 0 {
            Some("체결 단계 가격이 0: 체결 불가".into())
        } else if protective {
            // Research protection is a forced risk-reduction scenario. Profit and
            // strategy reentry guards must not suppress an already touched stop.
            None
        } else {
            match self.guard.evaluate_for_scope_at(
                &self.scope,
                &signal,
                held,
                average_units,
                price_units,
                self.is_overseas,
                now,
            ) {
                GuardDecision::Allow => None,
                GuardDecision::Block { reason } => Some(reason),
            }
        };
        if blocked_reason.is_none() {
            blocked_reason = execution_block_reason(
                &self.risk,
                &self.strategy_id,
                &self.symbol,
                &signal,
                self.position.as_ref(),
            );
        }
        if blocked_reason.is_none()
            && self.assumptions.volatility_sizing_enabled
            && matches!(signal, Signal::Buy { .. })
        {
            if self.assumptions.risk_per_trade_bps == 0 {
                blocked_reason = Some("ATR 수량 산정 위험 예산 0bps: 신규 매수 불가".into());
            } else if !self.atr_window.value().is_some_and(|value| value > 0) {
                blocked_reason = Some(format!(
                    "ATR(TR,14) 수량 산정 자료 부족 또는 변동폭 0: {} / 14개 true range",
                    self.atr_window.available_ranges()
                ));
            }
        }
        let phase = self.phase();
        let signal_price_krw = self.krw(price_units);
        let trade = &mut self.trades[trade_index];
        trade.time = event.time.clone();
        trade.phase = phase.clone();
        trade.status = "blocked".into();
        trade.blocked_reason = blocked_reason;
        if trade.blocked_reason.is_some() {
            return;
        }
        match &signal {
            Signal::Buy { quantity, .. } => {
                let total_equity =
                    current_equity(self.cash, self.position.as_ref(), signal_price_krw);
                let adjusted_quantity = self.risk.volatility_adjusted_quantity(
                    &self.symbol,
                    *quantity,
                    price_units,
                    total_equity.round() as i64,
                    self.is_overseas,
                    self.assumptions.exchange_rate_krw,
                );
                let fill_krw =
                    apply_slippage(signal_price_krw, self.assumptions.slippage_bps, true);
                let gross = fill_krw * adjusted_quantity as f64;
                let fee = bps_cost(gross, self.assumptions.fee_bps);
                let amount = gross + fee;
                if adjusted_quantity == 0 {
                    trade.blocked_reason = Some("변동성 수량 산정 결과가 0주".into());
                } else if !self
                    .risk
                    .check_position_size(gross.round() as i64, total_equity.round() as i64)
                {
                    trade.blocked_reason = Some("단일 종목 최대 비중 초과".into());
                } else if amount > self.cash {
                    trade.blocked_reason = Some("초기자본/현금 부족".into());
                } else {
                    self.cash -= amount;
                    self.position = Some(PositionState {
                        quantity: adjusted_quantity,
                        average_fill_krw: fill_krw,
                        allocated_buy_cost_krw: fee,
                    });
                    trade.fill_price = Some(krw_to_display(
                        fill_krw,
                        self.is_overseas,
                        self.assumptions.exchange_rate_krw,
                    ));
                    trade.quantity = adjusted_quantity;
                    trade.gross_amount_krw = gross;
                    trade.cost_krw =
                        fee + (fill_krw - signal_price_krw).max(0.0) * adjusted_quantity as f64;
                    trade.status = "filled".into();
                    self.turnover_krw += gross;
                }
            }
            Signal::Sell { quantity, .. } => {
                if let Some(open) = self.position.take() {
                    let sell_quantity = (*quantity).min(open.quantity);
                    let fill_krw =
                        apply_slippage(signal_price_krw, self.assumptions.slippage_bps, false);
                    let gross = fill_krw * sell_quantity as f64;
                    let fee = bps_cost(gross, self.assumptions.fee_bps);
                    let tax = bps_cost(gross, self.assumptions.tax_bps);
                    let buy_cost =
                        open.allocated_buy_cost_krw * sell_quantity as f64 / open.quantity as f64;
                    let pnl = (fill_krw - open.average_fill_krw) * sell_quantity as f64
                        - fee
                        - tax
                        - buy_cost;
                    self.cash += gross - fee - tax;
                    let remaining = open.quantity - sell_quantity;
                    if remaining > 0 {
                        self.position = Some(PositionState {
                            quantity: remaining,
                            average_fill_krw: open.average_fill_krw,
                            allocated_buy_cost_krw: open.allocated_buy_cost_krw - buy_cost,
                        });
                    }
                    self.risk.record_pnl(pnl.round() as i64);
                    self.risk.record_strategy_symbol_pnl_for_scope(
                        &self.scope,
                        &self.strategy_id,
                        &self.symbol,
                        pnl.round() as i64,
                    );
                    self.completed_pnls.push((event.time.clone(), pnl, phase));
                    trade.fill_price = Some(krw_to_display(
                        fill_krw,
                        self.is_overseas,
                        self.assumptions.exchange_rate_krw,
                    ));
                    trade.quantity = sell_quantity;
                    trade.gross_amount_krw = gross;
                    trade.cost_krw =
                        fee + tax + (signal_price_krw - fill_krw).max(0.0) * sell_quantity as f64;
                    trade.realized_pnl_krw = Some(round_money(pnl));
                    trade.status = "filled".into();
                    self.turnover_krw += gross;
                }
            }
            Signal::Hold => {}
        }
        if trade.status == "filled" {
            trade.fill_at = Some(ReplayEventTime {
                bar_time: event.time.clone(),
                phase: clock_phase.into(),
            });
            self.eligible_count += 1;
            self.filled_count += 1;
            self.guard
                .record_submitted_for_scope_at(&self.scope, &signal, now);
        }
    }

    fn protect_position(&mut self, event: &SimulationEvent) {
        let Some(position) = &self.position else {
            return;
        };
        if event.open_units == 0 {
            return;
        }
        let entry_units = if self.is_overseas {
            position.average_fill_krw / self.assumptions.exchange_rate_krw * 100.0
        } else {
            position.average_fill_krw
        };
        let stop = (self.assumptions.stop_loss_pct > 0.0)
            .then(|| entry_units * (1.0 - self.assumptions.stop_loss_pct / 100.0));
        let take = (self.assumptions.take_profit_pct > 0.0)
            .then(|| entry_units * (1.0 + self.assumptions.take_profit_pct / 100.0));
        let gap_stop = stop.is_some_and(|threshold| event.open_units as f64 <= threshold);
        let gap_take = take.is_some_and(|threshold| event.open_units as f64 >= threshold);
        let hit_stop = stop
            .is_some_and(|threshold| event.low_units > 0 && event.low_units as f64 <= threshold);
        let hit_take = take.is_some_and(|threshold| event.high_units as f64 >= threshold);
        let ambiguous = hit_stop && hit_take && !gap_stop && !gap_take;
        if ambiguous {
            self.ambiguous_bar_count += 1;
        }
        let (price, clock, path, label) = if gap_stop {
            (event.open_units, "open", "gapAtOpen", "손절 시가 갭")
        } else if gap_take {
            (event.open_units, "open", "gapAtOpen", "익절 시가 갭")
        } else if !(hit_stop || hit_take) {
            return;
        } else if self.assumptions.intrabar_exit_policy == IntrabarExitPolicy::Unverified {
            let warning =
                "OHLC 보호 가격 접촉은 장중 경로 미검증으로 체결하지 않았습니다(unverified).";
            if !self.execution_warnings.iter().any(|entry| entry == warning) {
                self.execution_warnings.push(warning.into());
            }
            return;
        } else if hit_stop {
            (
                stop.unwrap_or(entry_units).floor().max(1.0) as u64,
                "intrabar",
                if ambiguous {
                    "ambiguousStopFirst"
                } else {
                    "ohlcUnverified"
                },
                "손절 우선",
            )
        } else {
            (
                take.unwrap_or(entry_units).floor().max(1.0) as u64,
                "intrabar",
                "ohlcUnverified",
                "익절 접촉",
            )
        };
        let signal = Signal::Sell {
            symbol: self.symbol.clone(),
            quantity: position.quantity,
            reason: format!("연구용 보호 시나리오: {label}"),
        };
        let trade_index = self.push_trade(&signal, event, price, clock);
        self.trades[trade_index].intrabar_path_status = Some(path.into());
        self.execute(signal, trade_index, event, price, clock, true);
    }

    pub fn finish(mut self) -> BacktestReportView {
        if let Some(pending) = self.pending.take() {
            self.trades[pending.trade_index]
                .reason
                .push_str(" · 평가 종료: 다음 관측 봉이 없어 미체결");
        }
        let final_equity =
            current_equity(self.cash, self.position.as_ref(), self.krw(self.last_close));
        let winning = self
            .completed_pnls
            .iter()
            .filter(|(_, pnl, _)| *pnl > 0.0)
            .count();
        let losing = self
            .completed_pnls
            .iter()
            .filter(|(_, pnl, _)| *pnl < 0.0)
            .count();
        let gross_profit: f64 = self
            .completed_pnls
            .iter()
            .map(|(_, pnl, _)| pnl.max(0.0))
            .sum();
        let gross_loss: f64 = self
            .completed_pnls
            .iter()
            .map(|(_, pnl, _)| pnl.min(0.0).abs())
            .sum();
        let phases = phase_summaries(
            &self.equity_curve,
            &self.completed_pnls,
            self.assumptions.initial_capital_krw,
        );
        self.atr_preparation.ready_at_end = self.atr_window.ready();
        BacktestReportView {
            execution_warnings: self.execution_warnings,
            assumptions: self.assumptions.clone(),
            summary: BacktestSummaryView {
                initial_capital_krw: round_money(self.assumptions.initial_capital_krw),
                final_equity_krw: round_money(final_equity),
                cumulative_return_pct: round_metric(return_pct(
                    self.assumptions.initial_capital_krw,
                    final_equity,
                )),
                mdd_pct: self
                    .equity_curve
                    .iter()
                    .map(|p| p.drawdown_pct)
                    .fold(0.0, f64::max),
                completed_trades: self.completed_pnls.len(),
                winning_trades: winning,
                losing_trades: losing,
                win_rate_pct: round_metric(percent(winning, self.completed_pnls.len())),
                profit_factor: (gross_loss > 0.0).then(|| round_metric(gross_profit / gross_loss)),
                turnover_pct: round_metric(
                    self.turnover_krw / self.assumptions.initial_capital_krw * 100.0,
                ),
                exposure_pct: round_metric(percent(self.exposed_events, self.index)),
                signal_count: self.signal_count,
                order_eligible_count: self.eligible_count,
                filled_order_count: self.filled_count,
                blocked_order_count: self
                    .trades
                    .iter()
                    .filter(|trade| trade.status == "blocked")
                    .count(),
                unfilled_order_count: self
                    .trades
                    .iter()
                    .filter(|trade| trade.status == "unfilled")
                    .count(),
                ambiguous_bar_count: self.ambiguous_bar_count,
            },
            overfit_warning: overfit_warning(&phases),
            phases,
            trades: self.trades,
            equity_curve: self.equity_curve,
            atr_preparation: self.atr_preparation,
        }
    }
}
