use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::{state::bounded_window_with_extra, OhlcCandle, Signal, Strategy, StrategyConfig};

mod bollinger;
#[cfg(test)]
mod buffer_tests;
mod exits;
mod indicators;
mod live;
mod preview;
mod session;
#[cfg(test)]
mod tests;
use bollinger::BollingerParams;

fn lth_default_qty() -> u64 {
    1
}
fn lth_default_ema_short() -> usize {
    20
}
fn lth_default_ema_long() -> usize {
    60
}
fn lth_default_rsi_period() -> usize {
    14
}
fn lth_default_adx_period() -> usize {
    14
}
fn lth_default_buy_rsi() -> f64 {
    55.0
}
fn lth_default_sell_rsi() -> f64 {
    50.0
}
fn lth_default_buy_adx() -> f64 {
    20.0
}
fn lth_default_no_trade_adx() -> f64 {
    18.0
}
fn lth_default_trailing_stop() -> f64 {
    1.5
}
fn lth_default_trailing_activation_profit() -> f64 {
    1.0
}
fn lth_default_breakeven_buffer() -> f64 {
    0.2
}
fn lth_default_min_hold_observations() -> usize {
    2
}
fn lth_default_initial_stop_loss() -> f64 {
    1.0
}
fn lth_default_entry_failure_observations() -> usize {
    3
}
fn lth_default_entry_start() -> i64 {
    15
}
fn lth_default_entry_end() -> i64 {
    30
}
fn lth_default_exit_before_close() -> i64 {
    20
}
fn lth_default_gap_limit() -> f64 {
    4.0
}
fn lth_default_sensitivity() -> f64 {
    1.0
}
fn lth_default_toss_us_session() -> String {
    "auto".to_string()
}
fn lth_default_rebound_enabled() -> bool {
    false
}
fn lth_default_rebound_baseline_ticks() -> usize {
    8
}
fn lth_default_rebound_confirm_ticks() -> usize {
    3
}
fn lth_default_rebound_pullback() -> f64 {
    4.0
}
fn lth_default_rebound_buy_pressure() -> f64 {
    1.5
}
fn lth_default_rebound_rsi() -> f64 {
    30.0
}
fn lth_default_rapid_rebound_enabled() -> bool {
    false
}
fn lth_default_rapid_rebound_lookback_ticks() -> usize {
    8
}
fn lth_default_rapid_rebound_drop() -> f64 {
    2.0
}
fn lth_default_rapid_rebound_recovery() -> f64 {
    1.2
}
fn lth_default_rapid_rebound_max_low_age_ticks() -> usize {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeveragedTrendHoldEntry {
    pub leveraged_symbol: String,
    #[serde(default)]
    pub leveraged_symbol_name: String,
    #[serde(default)]
    pub inverse_leveraged_symbol: String,
    #[serde(default)]
    pub inverse_leveraged_symbol_name: String,
    #[serde(default)]
    pub base_symbols: Vec<String>,
    #[serde(default)]
    pub base_symbol_names: HashMap<String, String>,
    #[serde(default)]
    pub base_symbol_roles: HashMap<String, String>,
    #[serde(default = "lth_default_qty")]
    pub quantity: u64,
    #[serde(default = "lth_default_qty")]
    pub inverse_quantity: u64,
    #[serde(default)]
    pub is_overseas: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeveragedTrendHoldParams {
    #[serde(default)]
    pub bollinger: BollingerParams,
    #[serde(default)]
    pub entries: Vec<LeveragedTrendHoldEntry>,
    #[serde(default = "lth_default_ema_short")]
    pub ema_short_period: usize,
    #[serde(default = "lth_default_ema_long")]
    pub ema_long_period: usize,
    #[serde(default = "lth_default_rsi_period")]
    pub rsi_period: usize,
    #[serde(default = "lth_default_adx_period")]
    pub adx_period: usize,
    #[serde(default = "lth_default_buy_rsi")]
    pub entry_rsi_min: f64,
    #[serde(default = "lth_default_sensitivity")]
    pub upward_sensitivity: f64,
    #[serde(default = "lth_default_sensitivity")]
    pub downward_sensitivity: f64,
    #[serde(default = "lth_default_sell_rsi")]
    pub exit_rsi_below: f64,
    #[serde(default = "lth_default_buy_adx")]
    pub entry_adx_min: f64,
    #[serde(default = "lth_default_no_trade_adx")]
    pub no_trade_adx_below: f64,
    #[serde(default = "lth_default_trailing_stop")]
    pub trailing_stop_pct: f64,
    #[serde(default = "lth_default_trailing_activation_profit")]
    pub trailing_activation_profit_pct: f64,
    #[serde(default = "lth_default_breakeven_buffer")]
    pub breakeven_buffer_pct: f64,
    #[serde(default = "lth_default_min_hold_observations")]
    pub min_hold_observations: usize,
    #[serde(default = "lth_default_initial_stop_loss")]
    pub initial_stop_loss_pct: f64,
    #[serde(default = "lth_default_entry_failure_observations")]
    pub entry_failure_observations: usize,
    #[serde(default = "lth_default_entry_start")]
    pub entry_window_start_min: i64,
    #[serde(default = "lth_default_entry_end")]
    pub entry_window_end_min: i64,
    #[serde(default = "lth_default_exit_before_close")]
    pub exit_before_close_min: i64,
    #[serde(default = "lth_default_gap_limit")]
    pub max_gap_pct: f64,
    #[serde(default)]
    pub blackout_windows: Vec<String>,
    #[serde(default = "lth_default_toss_us_session")]
    pub toss_us_session: String,
    #[serde(default = "lth_default_rebound_enabled")]
    pub intraday_rebound_enabled: bool,
    #[serde(default = "lth_default_rebound_baseline_ticks")]
    pub rebound_baseline_ticks: usize,
    #[serde(default = "lth_default_rebound_confirm_ticks")]
    pub rebound_confirm_ticks: usize,
    #[serde(default = "lth_default_rebound_pullback")]
    pub rebound_pullback_pct: f64,
    #[serde(default = "lth_default_rebound_buy_pressure")]
    pub rebound_buy_pressure_pct: f64,
    #[serde(default = "lth_default_rebound_rsi")]
    pub rebound_rsi_min: f64,
    #[serde(default = "lth_default_rapid_rebound_enabled")]
    pub rapid_rebound_enabled: bool,
    #[serde(default = "lth_default_rapid_rebound_lookback_ticks")]
    pub rapid_rebound_lookback_ticks: usize,
    #[serde(default = "lth_default_rapid_rebound_drop")]
    pub rapid_rebound_drop_pct: f64,
    #[serde(default = "lth_default_rapid_rebound_recovery")]
    pub rapid_rebound_recovery_pct: f64,
    #[serde(default = "lth_default_rapid_rebound_max_low_age_ticks")]
    pub rapid_rebound_max_low_age_ticks: usize,
}

impl Default for LeveragedTrendHoldParams {
    fn default() -> Self {
        Self {
            bollinger: BollingerParams::default(),
            entries: Vec::new(),
            ema_short_period: lth_default_ema_short(),
            ema_long_period: lth_default_ema_long(),
            rsi_period: lth_default_rsi_period(),
            adx_period: lth_default_adx_period(),
            entry_rsi_min: lth_default_buy_rsi(),
            upward_sensitivity: lth_default_sensitivity(),
            downward_sensitivity: lth_default_sensitivity(),
            exit_rsi_below: lth_default_sell_rsi(),
            entry_adx_min: lth_default_buy_adx(),
            no_trade_adx_below: lth_default_no_trade_adx(),
            trailing_stop_pct: lth_default_trailing_stop(),
            trailing_activation_profit_pct: lth_default_trailing_activation_profit(),
            breakeven_buffer_pct: lth_default_breakeven_buffer(),
            min_hold_observations: lth_default_min_hold_observations(),
            initial_stop_loss_pct: lth_default_initial_stop_loss(),
            entry_failure_observations: lth_default_entry_failure_observations(),
            entry_window_start_min: lth_default_entry_start(),
            entry_window_end_min: lth_default_entry_end(),
            exit_before_close_min: lth_default_exit_before_close(),
            max_gap_pct: lth_default_gap_limit(),
            blackout_windows: Vec::new(),
            toss_us_session: lth_default_toss_us_session(),
            intraday_rebound_enabled: lth_default_rebound_enabled(),
            rebound_baseline_ticks: lth_default_rebound_baseline_ticks(),
            rebound_confirm_ticks: lth_default_rebound_confirm_ticks(),
            rebound_pullback_pct: lth_default_rebound_pullback(),
            rebound_buy_pressure_pct: lth_default_rebound_buy_pressure(),
            rebound_rsi_min: lth_default_rebound_rsi(),
            rapid_rebound_enabled: lth_default_rapid_rebound_enabled(),
            rapid_rebound_lookback_ticks: lth_default_rapid_rebound_lookback_ticks(),
            rapid_rebound_drop_pct: lth_default_rapid_rebound_drop(),
            rapid_rebound_recovery_pct: lth_default_rapid_rebound_recovery(),
            rapid_rebound_max_low_age_ticks: lth_default_rapid_rebound_max_low_age_ticks(),
        }
    }
}

struct LeveragedTrendHoldMarketState {
    intraday_candles: VecDeque<OhlcCandle>,
    rebound_prices: VecDeque<u64>,
    live_candle_minute: Option<i64>,
}

struct LeveragedTrendHoldPosition {
    in_position: bool,
    entry_price: Option<u64>,
    high_water: Option<u64>,
    held_observations: usize,
}

struct LeveragedTrendSnapshot {
    ema_short: f64,
    ema_long: f64,
    rsi: f64,
    adx: f64,
    bullish_count_3: usize,
}

struct LeveragedReboundSnapshot {
    rsi: Option<f64>,
    adx: Option<f64>,
    pullback_pct: f64,
    buy_pressure_pct: f64,
    rebound_from_low_pct: f64,
}

struct LeveragedRapidReboundSnapshot {
    rsi: Option<f64>,
    adx: Option<f64>,
    drop_pct: f64,
    recovery_pct: f64,
    low_age_ticks: usize,
}

#[derive(Debug, Clone)]
pub struct LeveragedTrendHoldTimedCandle {
    pub time: String,
    pub candle: OhlcCandle,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeveragedTrendHoldPreviewSignal {
    pub time: String,
    pub side: String,
    pub price_units: u64,
    pub quantity: u64,
    pub reason: String,
    pub ema_short: Option<f64>,
    pub ema_long: Option<f64>,
    pub rsi: Option<f64>,
    pub adx: Option<f64>,
}

pub struct LeveragedTrendHoldStrategy {
    daily_context: HashMap<String, VecDeque<OhlcCandle>>,
    bollinger_candles: HashMap<String, VecDeque<OhlcCandle>>,
    config: StrategyConfig,
    params: LeveragedTrendHoldParams,
    states: HashMap<String, LeveragedTrendHoldMarketState>,
    positions: HashMap<String, LeveragedTrendHoldPosition>,
    last_params: serde_json::Value,
}

impl LeveragedTrendHoldStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let mut config = config;
        let params: LeveragedTrendHoldParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        config.target_symbols = Self::target_symbols_for_params(&params);
        let last_params = config.params.clone();
        Self {
            daily_context: HashMap::new(),
            bollinger_candles: HashMap::new(),
            config,
            params,
            states: HashMap::new(),
            positions: HashMap::new(),
            last_params,
        }
    }

    fn sync_params(&mut self) {
        if self.config.params == self.last_params {
            return;
        }
        self.params = serde_json::from_value(self.config.params.clone()).unwrap_or_default();
        self.last_params = self.config.params.clone();
        self.config.target_symbols = Self::target_symbols_for_params(&self.params);
    }

    fn target_symbols_for_params(params: &LeveragedTrendHoldParams) -> Vec<String> {
        let mut symbols: Vec<String> = params
            .entries
            .iter()
            .map(|entry| entry.leveraged_symbol.clone())
            .filter(|symbol| !symbol.trim().is_empty())
            .collect();
        symbols.sort_unstable();
        symbols.dedup();
        symbols
    }

    fn entry_for_symbol(&self, symbol: &str) -> Option<LeveragedTrendHoldEntry> {
        self.params
            .entries
            .iter()
            .find(|entry| entry.leveraged_symbol == symbol)
            .cloned()
    }

    fn is_target_symbol(&self, symbol: &str) -> bool {
        self.params
            .entries
            .iter()
            .any(|entry| entry.leveraged_symbol == symbol)
    }

    fn window_cap(&self) -> usize {
        bounded_window_with_extra(
            self.params
                .ema_long_period
                .max(self.params.adx_period + 2)
                .max(80),
            5,
        )
    }

    fn rebound_price_cap(&self) -> usize {
        let staged_rebound = self
            .params
            .rebound_baseline_ticks
            .saturating_add(self.params.rebound_confirm_ticks);
        let rapid_rebound = self.params.rapid_rebound_lookback_ticks;
        bounded_window_with_extra(staged_rebound.max(rapid_rebound).max(4), 2)
    }

    #[cfg(test)]
    fn current_live_minute_key() -> i64 {
        use std::sync::atomic::{AtomicI64, Ordering};
        static NEXT_MINUTE: AtomicI64 = AtomicI64::new(0);
        NEXT_MINUTE.fetch_add(1, Ordering::Relaxed)
    }

    #[cfg(not(test))]
    fn current_live_minute_key() -> i64 {
        use chrono::{Datelike, Timelike};
        let now = chrono::Local::now();
        now.date_naive().num_days_from_ce() as i64 * 24 * 60
            + now.hour() as i64 * 60
            + now.minute() as i64
    }
}
