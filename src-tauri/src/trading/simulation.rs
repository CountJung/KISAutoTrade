use std::collections::HashMap;

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::broker::BrokerScope;

use super::{
    guard::{GuardDecision, TradeGuard, TradeGuardConfig},
    risk::RiskManager,
    strategy::Signal,
};

pub const REPLAY_ENGINE_VERSION: &str = "strategy-replay-v2";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationAssumptions {
    #[serde(default = "default_initial_capital")]
    pub initial_capital_krw: f64,
    #[serde(default = "default_fee_bps")]
    pub fee_bps: f64,
    #[serde(default = "default_tax_bps")]
    pub tax_bps: f64,
    #[serde(default = "default_slippage_bps")]
    pub slippage_bps: f64,
    #[serde(default = "default_exchange_rate")]
    pub exchange_rate_krw: f64,
    #[serde(default = "default_max_position_ratio")]
    pub max_position_ratio: f64,
    #[serde(default)]
    pub volatility_sizing_enabled: bool,
    #[serde(default = "default_risk_per_trade_bps")]
    pub risk_per_trade_bps: u32,
    #[serde(default = "default_atr_stop_multiplier")]
    pub atr_stop_multiplier: f64,
    #[serde(default = "default_daily_loss_limit")]
    pub daily_loss_limit_krw: f64,
    #[serde(default = "default_in_sample_percent")]
    pub in_sample_percent: u8,
}

impl Default for SimulationAssumptions {
    fn default() -> Self {
        Self {
            initial_capital_krw: default_initial_capital(),
            fee_bps: default_fee_bps(),
            tax_bps: default_tax_bps(),
            slippage_bps: default_slippage_bps(),
            exchange_rate_krw: default_exchange_rate(),
            max_position_ratio: default_max_position_ratio(),
            volatility_sizing_enabled: false,
            risk_per_trade_bps: default_risk_per_trade_bps(),
            atr_stop_multiplier: default_atr_stop_multiplier(),
            daily_loss_limit_krw: default_daily_loss_limit(),
            in_sample_percent: default_in_sample_percent(),
        }
    }
}

impl SimulationAssumptions {
    fn normalized(mut self, is_overseas: bool) -> Self {
        self.initial_capital_krw = finite_clamp(
            self.initial_capital_krw,
            10_000.0,
            1_000_000_000_000.0,
            default_initial_capital(),
        );
        self.fee_bps = finite_clamp(
            self.fee_bps,
            0.0,
            1_000.0,
            if is_overseas { 10.0 } else { default_fee_bps() },
        );
        self.tax_bps = finite_clamp(
            self.tax_bps,
            0.0,
            1_000.0,
            if is_overseas { 0.0 } else { default_tax_bps() },
        );
        self.slippage_bps = finite_clamp(
            self.slippage_bps,
            0.0,
            1_000.0,
            if is_overseas {
                10.0
            } else {
                default_slippage_bps()
            },
        );
        self.exchange_rate_krw = finite_clamp(
            self.exchange_rate_krw,
            100.0,
            10_000.0,
            default_exchange_rate(),
        );
        self.max_position_ratio = finite_clamp(
            self.max_position_ratio,
            0.01,
            1.0,
            default_max_position_ratio(),
        );
        self.atr_stop_multiplier = finite_clamp(
            self.atr_stop_multiplier,
            0.1,
            20.0,
            default_atr_stop_multiplier(),
        );
        self.daily_loss_limit_krw = finite_clamp(
            self.daily_loss_limit_krw,
            0.0,
            1_000_000_000_000.0,
            default_daily_loss_limit(),
        );
        self.risk_per_trade_bps = self.risk_per_trade_bps.min(10_000);
        self.in_sample_percent = self.in_sample_percent.clamp(50, 100);
        self
    }
}

fn default_initial_capital() -> f64 {
    10_000_000.0
}
fn default_fee_bps() -> f64 {
    1.5
}
fn default_tax_bps() -> f64 {
    20.0
}
fn default_slippage_bps() -> f64 {
    5.0
}
fn default_exchange_rate() -> f64 {
    1_450.0
}
fn default_max_position_ratio() -> f64 {
    0.3
}
fn default_risk_per_trade_bps() -> u32 {
    100
}
fn default_atr_stop_multiplier() -> f64 {
    2.0
}
fn default_daily_loss_limit() -> f64 {
    1_000_000.0
}
fn default_in_sample_percent() -> u8 {
    70
}

fn finite_clamp(value: f64, min: f64, max: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

#[derive(Debug, Clone)]
pub struct SimulationEvent {
    pub time: String,
    pub chart_time: String,
    pub close_units: u64,
    pub high_units: u64,
    pub low_units: u64,
    pub signal: Option<Signal>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayMetadataView {
    pub engine_version: String,
    pub strategy_version: String,
    pub source_interval: String,
    pub replay_cadence: String,
    pub live_cadence_seconds: u64,
    pub warmup_count: usize,
    pub data_start: String,
    pub data_end: String,
    pub data_source: String,
    pub deterministic: bool,
    pub look_ahead_safe: bool,
    pub input_hash: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestTradeView {
    pub time: String,
    pub side: String,
    pub signal_price: f64,
    pub fill_price: Option<f64>,
    pub quantity: u64,
    pub gross_amount_krw: f64,
    pub cost_krw: f64,
    pub realized_pnl_krw: Option<f64>,
    pub status: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquityPointView {
    pub time: String,
    pub equity_krw: f64,
    pub drawdown_pct: f64,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestPhaseView {
    pub phase: String,
    pub start: String,
    pub end: String,
    pub return_pct: f64,
    pub mdd_pct: f64,
    pub completed_trades: usize,
    pub win_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestSummaryView {
    pub initial_capital_krw: f64,
    pub final_equity_krw: f64,
    pub cumulative_return_pct: f64,
    pub mdd_pct: f64,
    pub completed_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: f64,
    pub profit_factor: Option<f64>,
    pub turnover_pct: f64,
    pub exposure_pct: f64,
    pub signal_count: usize,
    pub order_eligible_count: usize,
    pub filled_order_count: usize,
    pub blocked_order_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestReportView {
    pub assumptions: SimulationAssumptions,
    pub summary: BacktestSummaryView,
    pub phases: Vec<BacktestPhaseView>,
    pub trades: Vec<BacktestTradeView>,
    pub equity_curve: Vec<EquityPointView>,
    pub overfit_warning: Option<String>,
}

#[derive(Debug, Clone)]
struct PositionState {
    quantity: u64,
    average_fill_krw: f64,
    allocated_buy_cost_krw: f64,
}

pub fn run_backtest(
    strategy_id: &str,
    symbol: &str,
    is_overseas: bool,
    assumptions: SimulationAssumptions,
    events: &[SimulationEvent],
) -> BacktestReportView {
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
    let guard_config = TradeGuardConfig {
        domestic_slippage_bps: rounded_bps(
            assumptions.fee_bps * 2.0 + assumptions.tax_bps + assumptions.slippage_bps,
        ) - 22,
        overseas_fee_bps: rounded_bps(assumptions.fee_bps * 2.0 + assumptions.tax_bps),
        overseas_slippage_bps: rounded_bps(assumptions.slippage_bps),
        ..TradeGuardConfig::default()
    };
    let mut guard = TradeGuard::new(guard_config);
    let scope = BrokerScope::kis_legacy();
    let split_index = if assumptions.in_sample_percent >= 100 {
        events.len()
    } else {
        events
            .len()
            .saturating_mul(assumptions.in_sample_percent as usize)
            / 100
    };
    let mut cash = assumptions.initial_capital_krw;
    let mut position: Option<PositionState> = None;
    let mut trades = Vec::new();
    let mut equity_curve = Vec::with_capacity(events.len());
    let mut completed_pnls = Vec::new();
    let mut turnover_krw = 0.0;
    let mut exposed_events = 0usize;
    let mut signal_count = 0usize;
    let mut eligible_count = 0usize;
    let mut filled_count = 0usize;
    let mut atr_window = Vec::with_capacity(14);

    for (index, event) in events.iter().enumerate() {
        let event_time = parse_replay_time(&event.time);
        if risk.reset_for_date(event_time.date_naive()) && risk.is_emergency_stop() {
            // replay에는 사용자 수동 비상정지 입력이 없으므로 전 거래일의 자동 손실한도
            // 비상정지는 다음 거래일 시작에 해제한다.
            risk.clear_emergency_stop();
        }
        if event.high_units > 0 && event.low_units > 0 {
            atr_window.push(event.high_units.saturating_sub(event.low_units));
            if atr_window.len() > 14 {
                atr_window.remove(0);
            }
            let atr = atr_window.iter().sum::<u64>() / atr_window.len() as u64;
            if atr > 0 {
                risk.set_symbol_atr(symbol, atr);
            }
        }

        let phase = phase_for_index(index, split_index, events.len()).to_string();
        if let Some(signal) = &event.signal {
            signal_count += 1;
            let now = event_time;
            let held = position.as_ref().map(|p| p.quantity).unwrap_or(0);
            let average_units = position.as_ref().and_then(|p| {
                krw_to_units(
                    p.average_fill_krw,
                    is_overseas,
                    assumptions.exchange_rate_krw,
                )
            });
            let mut blocked_reason = match guard.evaluate_for_scope_at(
                &scope,
                signal,
                held,
                average_units,
                event.close_units,
                is_overseas,
                now,
            ) {
                GuardDecision::Allow => None,
                GuardDecision::Block { reason } => Some(reason),
            };

            if blocked_reason.is_none() {
                blocked_reason =
                    execution_block_reason(&risk, strategy_id, symbol, signal, position.as_ref());
            }

            let (side, requested_quantity, reason) = signal_parts(signal);
            let signal_price_krw = units_to_krw(
                event.close_units,
                is_overseas,
                assumptions.exchange_rate_krw,
            );
            let mut trade = BacktestTradeView {
                time: event.time.clone(),
                side: side.to_string(),
                signal_price: units_to_display(event.close_units, is_overseas),
                fill_price: None,
                quantity: requested_quantity,
                gross_amount_krw: 0.0,
                cost_krw: 0.0,
                realized_pnl_krw: None,
                status: "blocked".into(),
                reason: reason.to_string(),
                blocked_reason,
                phase: phase.clone(),
            };

            if trade.blocked_reason.is_none() {
                match signal {
                    Signal::Buy { quantity, .. } => {
                        let total_equity =
                            current_equity(cash, position.as_ref(), signal_price_krw);
                        let adjusted_quantity = risk.volatility_adjusted_quantity(
                            symbol,
                            *quantity,
                            event.close_units,
                            total_equity.round() as i64,
                            is_overseas,
                            assumptions.exchange_rate_krw,
                        );
                        let fill_krw =
                            apply_slippage(signal_price_krw, assumptions.slippage_bps, true);
                        let gross = fill_krw * adjusted_quantity as f64;
                        let fee = bps_cost(gross, assumptions.fee_bps);
                        let amount = gross + fee;
                        if adjusted_quantity == 0 {
                            trade.blocked_reason = Some("변동성 수량 산정 결과가 0주".into());
                        } else if !risk
                            .check_position_size(gross.round() as i64, total_equity.round() as i64)
                        {
                            trade.blocked_reason = Some("단일 종목 최대 비중 초과".into());
                        } else if amount > cash {
                            trade.blocked_reason = Some("초기자본/현금 부족".into());
                        } else {
                            cash -= amount;
                            position = Some(PositionState {
                                quantity: adjusted_quantity,
                                average_fill_krw: fill_krw,
                                allocated_buy_cost_krw: fee,
                            });
                            trade.fill_price = Some(krw_to_display(
                                fill_krw,
                                is_overseas,
                                assumptions.exchange_rate_krw,
                            ));
                            trade.quantity = adjusted_quantity;
                            trade.gross_amount_krw = gross;
                            trade.cost_krw = fee
                                + (fill_krw - signal_price_krw).max(0.0) * adjusted_quantity as f64;
                            trade.status = "filled".into();
                            turnover_krw += gross;
                        }
                    }
                    Signal::Sell { quantity, .. } => {
                        if let Some(open) = position.take() {
                            let sell_quantity = (*quantity).min(open.quantity);
                            let fill_krw =
                                apply_slippage(signal_price_krw, assumptions.slippage_bps, false);
                            let gross = fill_krw * sell_quantity as f64;
                            let fee = bps_cost(gross, assumptions.fee_bps);
                            let tax = bps_cost(gross, assumptions.tax_bps);
                            let buy_cost = open.allocated_buy_cost_krw * sell_quantity as f64
                                / open.quantity as f64;
                            let pnl = (fill_krw - open.average_fill_krw) * sell_quantity as f64
                                - fee
                                - tax
                                - buy_cost;
                            cash += gross - fee - tax;
                            let remaining = open.quantity - sell_quantity;
                            if remaining > 0 {
                                position = Some(PositionState {
                                    quantity: remaining,
                                    average_fill_krw: open.average_fill_krw,
                                    allocated_buy_cost_krw: open.allocated_buy_cost_krw - buy_cost,
                                });
                            }
                            risk.record_pnl(pnl.round() as i64);
                            risk.record_strategy_symbol_pnl_for_scope(
                                &scope,
                                strategy_id,
                                symbol,
                                pnl.round() as i64,
                            );
                            completed_pnls.push((event.time.clone(), pnl, phase.clone()));
                            trade.fill_price = Some(krw_to_display(
                                fill_krw,
                                is_overseas,
                                assumptions.exchange_rate_krw,
                            ));
                            trade.quantity = sell_quantity;
                            trade.gross_amount_krw = gross;
                            trade.cost_krw = fee
                                + tax
                                + (signal_price_krw - fill_krw).max(0.0) * sell_quantity as f64;
                            trade.realized_pnl_krw = Some(round_money(pnl));
                            trade.status = "filled".into();
                            turnover_krw += gross;
                        }
                    }
                    Signal::Hold => {}
                }

                if trade.status == "filled" {
                    eligible_count += 1;
                    filled_count += 1;
                    guard.record_submitted_for_scope_at(&scope, signal, now);
                }
            }
            trades.push(trade);
        }

        if position.is_some() {
            exposed_events += 1;
        }
        let mark_price = units_to_krw(
            event.close_units,
            is_overseas,
            assumptions.exchange_rate_krw,
        );
        let equity = current_equity(cash, position.as_ref(), mark_price);
        let peak = equity_curve
            .iter()
            .map(|p: &EquityPointView| p.equity_krw)
            .fold(assumptions.initial_capital_krw, f64::max);
        let drawdown = if peak > 0.0 {
            ((peak - equity) / peak * 100.0).max(0.0)
        } else {
            0.0
        };
        equity_curve.push(EquityPointView {
            time: event.chart_time.clone(),
            equity_krw: round_money(equity),
            drawdown_pct: round_metric(drawdown),
            phase: phase_for_index(index, split_index, events.len()).to_string(),
        });
    }

    let final_price_krw = events
        .last()
        .map(|e| units_to_krw(e.close_units, is_overseas, assumptions.exchange_rate_krw))
        .unwrap_or(0.0);
    let final_equity = current_equity(cash, position.as_ref(), final_price_krw);
    let winning = completed_pnls
        .iter()
        .filter(|(_, pnl, _)| *pnl > 0.0)
        .count();
    let losing = completed_pnls
        .iter()
        .filter(|(_, pnl, _)| *pnl < 0.0)
        .count();
    let gross_profit: f64 = completed_pnls.iter().map(|(_, pnl, _)| pnl.max(0.0)).sum();
    let gross_loss: f64 = completed_pnls
        .iter()
        .map(|(_, pnl, _)| pnl.min(0.0).abs())
        .sum();
    let phases = phase_summaries(
        &equity_curve,
        &completed_pnls,
        assumptions.initial_capital_krw,
    );
    let overfit_warning = overfit_warning(&phases);

    BacktestReportView {
        assumptions: assumptions.clone(),
        summary: BacktestSummaryView {
            initial_capital_krw: round_money(assumptions.initial_capital_krw),
            final_equity_krw: round_money(final_equity),
            cumulative_return_pct: round_metric(return_pct(
                assumptions.initial_capital_krw,
                final_equity,
            )),
            mdd_pct: equity_curve
                .iter()
                .map(|p| p.drawdown_pct)
                .fold(0.0, f64::max),
            completed_trades: completed_pnls.len(),
            winning_trades: winning,
            losing_trades: losing,
            win_rate_pct: round_metric(percent(winning, completed_pnls.len())),
            profit_factor: (gross_loss > 0.0).then(|| round_metric(gross_profit / gross_loss)),
            turnover_pct: round_metric(turnover_krw / assumptions.initial_capital_krw * 100.0),
            exposure_pct: round_metric(percent(exposed_events, events.len())),
            signal_count,
            order_eligible_count: eligible_count,
            filled_order_count: filled_count,
            blocked_order_count: trades
                .iter()
                .filter(|trade| trade.status == "blocked")
                .count(),
        },
        phases,
        trades,
        equity_curve,
        overfit_warning,
    }
}

/// replay의 마지막 실제 체결 포지션을 전략 내부 상태 동기화 단위로 반환한다.
/// 해외 가격은 주문/전략과 동일하게 cents, 국내 가격은 원 단위다.
pub fn report_position_snapshot(report: &BacktestReportView, is_overseas: bool) -> (u64, u64) {
    let mut quantity = 0u64;
    let mut average_units = 0f64;
    for trade in report
        .trades
        .iter()
        .filter(|trade| trade.status == "filled")
    {
        let fill_units = trade.fill_price.unwrap_or(0.0) * if is_overseas { 100.0 } else { 1.0 };
        match trade.side.as_str() {
            "buy" => {
                let next_quantity = quantity.saturating_add(trade.quantity);
                if next_quantity > 0 {
                    average_units = (average_units * quantity as f64
                        + fill_units * trade.quantity as f64)
                        / next_quantity as f64;
                }
                quantity = next_quantity;
            }
            "sell" => {
                quantity = quantity.saturating_sub(trade.quantity);
                if quantity == 0 {
                    average_units = 0.0;
                }
            }
            _ => {}
        }
    }
    (quantity, average_units.round() as u64)
}

fn execution_block_reason(
    risk: &RiskManager,
    strategy_id: &str,
    symbol: &str,
    signal: &Signal,
    position: Option<&PositionState>,
) -> Option<String> {
    match signal {
        Signal::Buy { .. } if position.is_some() => Some("이미 보유 중인 포지션".into()),
        Signal::Buy { .. } if !risk.can_trade() => Some("리스크 손실 한도 또는 비상정지".into()),
        Signal::Buy { .. } => risk.consecutive_loss_block_reason(strategy_id, symbol),
        Signal::Sell { .. } if position.is_none() => Some("매도 가능한 보유 수량 없음".into()),
        _ => None,
    }
}

fn signal_parts(signal: &Signal) -> (&str, u64, &str) {
    match signal {
        Signal::Buy {
            quantity, reason, ..
        } => ("buy", *quantity, reason),
        Signal::Sell {
            quantity, reason, ..
        } => ("sell", *quantity, reason),
        Signal::Hold => ("hold", 0, "hold"),
    }
}

fn parse_replay_time(value: &str) -> DateTime<Local> {
    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
    let parsed = match digits.len() {
        14.. => NaiveDateTime::parse_from_str(&digits[..14], "%Y%m%d%H%M%S").ok(),
        8.. => NaiveDate::parse_from_str(&digits[..8], "%Y%m%d")
            .ok()
            .and_then(|date| date.and_hms_opt(15, 30, 0)),
        _ => None,
    };
    parsed
        .and_then(|value| Local.from_local_datetime(&value).earliest())
        .unwrap_or_else(|| {
            Local
                .timestamp_opt(0, 0)
                .earliest()
                .expect("local timezone must represent unix epoch")
        })
}

fn units_to_krw(units: u64, is_overseas: bool, exchange_rate: f64) -> f64 {
    if is_overseas {
        units as f64 / 100.0 * exchange_rate
    } else {
        units as f64
    }
}

fn units_to_display(units: u64, is_overseas: bool) -> f64 {
    if is_overseas {
        units as f64 / 100.0
    } else {
        units as f64
    }
}

fn krw_to_units(krw: f64, is_overseas: bool, exchange_rate: f64) -> Option<u64> {
    if krw <= 0.0 {
        return None;
    }
    Some(if is_overseas {
        (krw / exchange_rate * 100.0).round() as u64
    } else {
        krw.round() as u64
    })
}

fn krw_to_display(krw: f64, is_overseas: bool, exchange_rate: f64) -> f64 {
    if is_overseas {
        krw / exchange_rate
    } else {
        krw
    }
}

fn apply_slippage(price: f64, bps: f64, is_buy: bool) -> f64 {
    let ratio = bps / 10_000.0;
    if is_buy {
        price * (1.0 + ratio)
    } else {
        price * (1.0 - ratio)
    }
}

fn bps_cost(amount: f64, bps: f64) -> f64 {
    amount * bps / 10_000.0
}

fn current_equity(cash: f64, position: Option<&PositionState>, mark_price: f64) -> f64 {
    cash + position
        .map(|p| p.quantity as f64 * mark_price)
        .unwrap_or(0.0)
}

fn phase_for_index(index: usize, split_index: usize, len: usize) -> &'static str {
    if split_index >= len || index < split_index {
        "inSample"
    } else {
        "outOfSample"
    }
}

fn phase_summaries(
    curve: &[EquityPointView],
    completed_pnls: &[(String, f64, String)],
    initial_capital_krw: f64,
) -> Vec<BacktestPhaseView> {
    let mut grouped: HashMap<&str, Vec<&EquityPointView>> = HashMap::new();
    for point in curve {
        grouped.entry(&point.phase).or_default().push(point);
    }
    ["inSample", "outOfSample"]
        .into_iter()
        .filter_map(|phase| {
            let points = grouped.get(phase)?;
            let first = points.first()?;
            let last = points.last()?;
            let phase_trades = completed_pnls
                .iter()
                .filter(|(_, _, trade_phase)| trade_phase == phase)
                .collect::<Vec<_>>();
            let wins = phase_trades.iter().filter(|(_, pnl, _)| *pnl > 0.0).count();
            let phase_start_equity = if phase == "inSample" {
                initial_capital_krw
            } else {
                curve
                    .iter()
                    .position(|point| point.phase == phase)
                    .and_then(|index| index.checked_sub(1))
                    .and_then(|index| curve.get(index))
                    .map(|point| point.equity_krw)
                    .unwrap_or(first.equity_krw)
            };
            let mut peak = phase_start_equity;
            let mut phase_mdd = 0.0f64;
            for point in points {
                peak = peak.max(point.equity_krw);
                if peak > 0.0 {
                    phase_mdd = phase_mdd.max((peak - point.equity_krw) / peak * 100.0);
                }
            }
            Some(BacktestPhaseView {
                phase: phase.to_string(),
                start: first.time.clone(),
                end: last.time.clone(),
                return_pct: round_metric(return_pct(phase_start_equity, last.equity_krw)),
                mdd_pct: round_metric(phase_mdd),
                completed_trades: phase_trades.len(),
                win_rate_pct: round_metric(percent(wins, phase_trades.len())),
            })
        })
        .collect()
}

fn overfit_warning(phases: &[BacktestPhaseView]) -> Option<String> {
    let in_sample = phases.iter().find(|phase| phase.phase == "inSample")?;
    let out_of_sample = phases.iter().find(|phase| phase.phase == "outOfSample")?;
    if in_sample.return_pct > 0.0 && out_of_sample.return_pct <= 0.0 {
        Some(
            "학습 구간은 수익이지만 검증 구간은 손실입니다. 파라미터 과최적화 가능성을 확인하세요."
                .into(),
        )
    } else if in_sample.return_pct > 0.0 && out_of_sample.return_pct < in_sample.return_pct * 0.5 {
        Some("검증 구간 수익률이 학습 구간의 절반 미만입니다. out-of-sample 안정성이 낮을 수 있습니다.".into())
    } else {
        None
    }
}

fn return_pct(start: f64, end: f64) -> f64 {
    if start > 0.0 {
        (end - start) / start * 100.0
    } else {
        0.0
    }
}

fn percent(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 / total as f64 * 100.0
    }
}

fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
fn round_metric(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

pub fn replay_input_hash(parts: &[&str]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in parts
        .iter()
        .flat_map(|part| part.as_bytes().iter().copied())
    {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
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
}
