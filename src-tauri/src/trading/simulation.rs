use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::broker::BrokerScope;

use super::{
    guard::{GuardDecision, TradeGuard, TradeGuardConfig},
    risk::RiskManager,
    strategy::Signal,
};

mod comparison;
pub use comparison::{attach_comparison, ReplayComparisonView};
mod accounting;
pub use accounting::{
    CostSource, DividendPriceBasis, DividendScenarioPoint, FxScenarioPoint, ReplayAccountingView,
};
mod session;
pub use session::ReplaySession;
mod assessment;
pub use assessment::ReplayAssessmentView;
mod atr;
pub use atr::AtrPreparationView;
use atr::AtrWindow;

pub const REPLAY_ENGINE_VERSION: &str = "strategy-replay-v15";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionModel {
    #[default]
    SameClose,
    NextOpen,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IntrabarExitPolicy {
    #[default]
    Unverified,
    StopFirst,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationAssumptions {
    #[serde(default = "default_initial_capital")]
    pub initial_capital_krw: f64,
    #[serde(default)]
    pub execution_model: ExecutionModel,
    #[serde(default)]
    pub intrabar_exit_policy: IntrabarExitPolicy,
    /// Research-only protection, independent from strategy stop logic; zero disables.
    #[serde(default)]
    pub stop_loss_pct: f64,
    #[serde(default)]
    pub take_profit_pct: f64,
    #[serde(default = "default_fee_bps")]
    pub fee_bps: f64,
    #[serde(default)]
    pub buy_fee_bps: Option<f64>,
    #[serde(default)]
    pub sell_fee_bps: Option<f64>,
    #[serde(default)]
    pub minimum_fee_native: f64,
    #[serde(default)]
    pub cost_source: CostSource,
    #[serde(default)]
    pub cost_note: String,
    #[serde(default)]
    pub fx_scenario: Vec<FxScenarioPoint>,
    #[serde(default)]
    pub dividend_scenario: Vec<DividendScenarioPoint>,
    #[serde(default)]
    pub dividend_price_basis: DividendPriceBasis,
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
            execution_model: ExecutionModel::default(),
            intrabar_exit_policy: IntrabarExitPolicy::default(),
            stop_loss_pct: 0.0,
            take_profit_pct: 0.0,
            fee_bps: default_fee_bps(),
            buy_fee_bps: None,
            sell_fee_bps: None,
            minimum_fee_native: 0.0,
            cost_source: CostSource::default(),
            cost_note: String::new(),
            fx_scenario: Vec::new(),
            dividend_scenario: Vec::new(),
            dividend_price_basis: DividendPriceBasis::default(),
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
        self.stop_loss_pct = finite_clamp(self.stop_loss_pct, 0.0, 99.9, 0.0);
        self.take_profit_pct = finite_clamp(self.take_profit_pct, 0.0, 1_000.0, 0.0);
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
    pub open_units: u64,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment: Option<ReplayAssessmentView>,
}

/// Bar label and causal phase, without inventing an exchange execution timestamp.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayEventTime {
    pub bar_time: String,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestTradeView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_available_at: Option<ReplayEventTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_at: Option<ReplayEventTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrabar_path_status: Option<String>,
    pub time: String,
    pub side: String,
    pub signal_price: f64,
    pub fill_price: Option<f64>,
    pub execution_reference_price: Option<f64>,
    pub exchange_rate_krw: f64,
    pub fee_krw: f64,
    pub tax_krw: f64,
    pub slippage_krw: f64,
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
    pub unfilled_order_count: usize,
    pub ambiguous_bar_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestReportView {
    pub execution_warnings: Vec<String>,
    pub assumptions: SimulationAssumptions,
    pub accounting: ReplayAccountingView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparison: Option<ReplayComparisonView>,
    pub summary: BacktestSummaryView,
    pub phases: Vec<BacktestPhaseView>,
    pub trades: Vec<BacktestTradeView>,
    pub equity_curve: Vec<EquityPointView>,
    pub overfit_warning: Option<String>,
    pub atr_preparation: AtrPreparationView,
}

#[derive(Debug, Clone)]
struct PositionState {
    quantity: u64,
    average_fill_krw: f64,
    average_fill_units: f64,
    allocated_buy_cost_krw: f64,
}

pub fn run_backtest(
    strategy_id: &str,
    symbol: &str,
    is_overseas: bool,
    assumptions: SimulationAssumptions,
    events: &[SimulationEvent],
) -> BacktestReportView {
    run_backtest_with_history(strategy_id, symbol, is_overseas, assumptions, &[], events)
}

/// Seed risk indicators with the caller's validated, same-interval history.
/// History contributes no orders, equity samples or evaluation-period returns.
pub fn run_backtest_with_history(
    strategy_id: &str,
    symbol: &str,
    is_overseas: bool,
    assumptions: SimulationAssumptions,
    history_events: &[SimulationEvent],
    events: &[SimulationEvent],
) -> BacktestReportView {
    let scenario_error = assumptions
        .validate_scenarios(
            is_overseas,
            &events
                .iter()
                .map(|event| event.time.clone())
                .collect::<Vec<_>>(),
        )
        .err();
    let mut session = ReplaySession::new(
        strategy_id,
        symbol,
        is_overseas,
        assumptions,
        history_events,
        events.len(),
    );
    if let Some(error) = scenario_error {
        session.reject_scenarios(error);
        return session.finish();
    }
    for event in events {
        session.begin_bar(event);
        session.process_close(event);
    }
    session.finish()
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

fn parse_replay_time(value: &str, is_overseas: bool) -> DateTime<FixedOffset> {
    if let Ok(instant) = DateTime::parse_from_rfc3339(value) {
        // Replay risk reset and same-day reentry rules follow the market calendar,
        // independent of the machine timezone. Fix each event's actual DST offset.
        return if is_overseas {
            instant
                .with_timezone(&chrono_tz::America::New_York)
                .fixed_offset()
        } else {
            instant
                .with_timezone(&chrono_tz::Asia::Seoul)
                .fixed_offset()
        };
    }
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
        // Preserve legacy compact/daily replay interpretation and numeric results.
        .fixed_offset()
}

#[test]
fn replay_time_preserves_rfc3339_instant_and_subseconds() {
    let utc = parse_replay_time("2026-07-11T13:30:00.125Z", true);
    let new_york = parse_replay_time("2026-07-11T09:30:00.125-04:00", true);
    let seoul = parse_replay_time("2026-07-11T22:30:00.125+09:00", true);
    assert_eq!(utc, new_york);
    assert_eq!(utc, seoul);
    assert_eq!(utc.timestamp_millis(), 1_783_776_600_125);
    assert_eq!(utc.offset().local_minus_utc(), -4 * 60 * 60);
    let winter = parse_replay_time("2026-11-01T06:30:00Z", true);
    assert_eq!(winter.offset().local_minus_utc(), -5 * 60 * 60);
    assert_eq!(
        parse_replay_time("2026-07-11T23:30:00Z", true)
            .date_naive()
            .to_string(),
        "2026-07-11"
    );
    assert_eq!(
        parse_replay_time("2026-07-11T23:30:00Z", false)
            .date_naive()
            .to_string(),
        "2026-07-12"
    );
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
#[path = "simulation/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "simulation/execution_tests.rs"]
mod execution_tests;

#[cfg(test)]
#[path = "simulation/accounting_tests.rs"]
mod accounting_tests;
