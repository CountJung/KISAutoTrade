use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::{state::bounded_window_with_extra, OhlcCandle, Signal, Strategy, StrategyConfig};

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
}

impl Default for LeveragedTrendHoldParams {
    fn default() -> Self {
        Self {
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
        }
    }
}

struct LeveragedTrendHoldMarketState {
    candles: VecDeque<OhlcCandle>,
    rebound_prices: VecDeque<u64>,
    live_candle_started: bool,
}

struct LeveragedTrendHoldPosition {
    in_position: bool,
    entry_price: Option<u64>,
    high_water: Option<u64>,
}

struct LeveragedTrendSnapshot {
    ema_short: f64,
    ema_long: f64,
    rsi: f64,
    adx: f64,
    bullish_count_3: usize,
}

struct LeveragedReboundSnapshot {
    trend: LeveragedTrendSnapshot,
    pullback_pct: f64,
    buy_pressure_pct: f64,
    rebound_from_low_pct: f64,
}

pub struct LeveragedTrendHoldStrategy {
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
        bounded_window_with_extra(
            self.params
                .rebound_baseline_ticks
                .saturating_add(self.params.rebound_confirm_ticks)
                .max(4),
            2,
        )
    }

    fn update_target_tick(&mut self, symbol: &str, price: u64) {
        let cap = self.window_cap();
        let rebound_cap = self.rebound_price_cap();
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| {
            LeveragedTrendHoldMarketState {
                candles: VecDeque::with_capacity(cap),
                rebound_prices: VecDeque::with_capacity(rebound_cap),
                live_candle_started: false,
            }
        });

        if !state.live_candle_started {
            state.candles.push_back(OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            });
            state.live_candle_started = true;
        } else if let Some(last) = state.candles.back_mut() {
            last.high = last.high.max(price);
            last.low = last.low.min(price);
            last.close = price;
        }

        while state.candles.len() > cap {
            state.candles.pop_front();
        }
        state.rebound_prices.push_back(price);
        while state.rebound_prices.len() > rebound_cap {
            state.rebound_prices.pop_front();
        }
    }

    fn closes(candles: &VecDeque<OhlcCandle>) -> Vec<f64> {
        candles.iter().map(|c| c.close as f64).collect()
    }

    fn ema(values: &[f64], period: usize) -> Option<f64> {
        if values.len() < period || period == 0 {
            return None;
        }
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = values[0];
        for value in &values[1..] {
            ema = value * alpha + ema * (1.0 - alpha);
        }
        Some(ema)
    }

    fn rsi(values: &[f64], period: usize) -> Option<f64> {
        if values.len() < period + 1 || period == 0 {
            return None;
        }
        let start = values.len() - period - 1;
        let mut gains = 0.0;
        let mut losses = 0.0;
        for pair in values[start..].windows(2) {
            let diff = pair[1] - pair[0];
            if diff >= 0.0 {
                gains += diff;
            } else {
                losses += -diff;
            }
        }
        if losses == 0.0 {
            return Some(100.0);
        }
        let rs = (gains / period as f64) / (losses / period as f64);
        Some(100.0 - 100.0 / (1.0 + rs))
    }

    fn adx(candles: &VecDeque<OhlcCandle>, period: usize) -> Option<f64> {
        if candles.len() < period + 1 || period == 0 {
            return None;
        }
        let start = candles.len() - period - 1;
        let slice: Vec<OhlcCandle> = candles.iter().skip(start).copied().collect();
        let mut tr_sum = 0.0;
        let mut plus_dm_sum = 0.0;
        let mut minus_dm_sum = 0.0;

        for pair in slice.windows(2) {
            let prev = pair[0];
            let cur = pair[1];
            let high_diff = cur.high as f64 - prev.high as f64;
            let low_diff = prev.low as f64 - cur.low as f64;
            let plus_dm = if high_diff > low_diff && high_diff > 0.0 {
                high_diff
            } else {
                0.0
            };
            let minus_dm = if low_diff > high_diff && low_diff > 0.0 {
                low_diff
            } else {
                0.0
            };
            let high_low = cur.high.saturating_sub(cur.low) as f64;
            let high_close = (cur.high as f64 - prev.close as f64).abs();
            let low_close = (cur.low as f64 - prev.close as f64).abs();
            tr_sum += high_low.max(high_close).max(low_close);
            plus_dm_sum += plus_dm;
            minus_dm_sum += minus_dm;
        }

        if tr_sum == 0.0 {
            return Some(0.0);
        }
        let plus_di = 100.0 * plus_dm_sum / tr_sum;
        let minus_di = 100.0 * minus_dm_sum / tr_sum;
        let denom = plus_di + minus_di;
        if denom == 0.0 {
            return Some(0.0);
        }
        Some(100.0 * (plus_di - minus_di).abs() / denom)
    }

    fn bullish_count(candles: &VecDeque<OhlcCandle>, count: usize) -> usize {
        candles
            .iter()
            .rev()
            .take(count)
            .filter(|c| c.close > c.open)
            .count()
    }

    fn gap_pct(candles: &VecDeque<OhlcCandle>) -> Option<f64> {
        if candles.len() < 2 {
            return None;
        }
        let cur = candles.back()?;
        let prev = candles.iter().rev().nth(1)?;
        if prev.close == 0 {
            return None;
        }
        Some((cur.open as f64 - prev.close as f64).abs() / prev.close as f64 * 100.0)
    }

    fn snapshot_for(&self, symbol: &str) -> Option<LeveragedTrendSnapshot> {
        let state = self.states.get(symbol)?;
        let closes = Self::closes(&state.candles);
        let ema_short = Self::ema(&closes, self.params.ema_short_period)?;
        let ema_long = Self::ema(&closes, self.params.ema_long_period)?;
        let rsi = Self::rsi(&closes, self.params.rsi_period)?;
        let adx = Self::adx(&state.candles, self.params.adx_period)?;
        Some(LeveragedTrendSnapshot {
            ema_short,
            ema_long,
            rsi,
            adx,
            bullish_count_3: Self::bullish_count(&state.candles, 3),
        })
    }

    fn upward_entry_rsi_min(&self) -> f64 {
        let sensitivity = self.params.upward_sensitivity.clamp(1.0, 5.0);
        (self.params.entry_rsi_min - (sensitivity - 1.0) * 2.0).clamp(45.0, 70.0)
    }

    fn entry_ok(&self, symbol: &str) -> Option<LeveragedTrendSnapshot> {
        let state = self.states.get(symbol)?;
        let snap = self.snapshot_for(symbol)?;
        let close = state.candles.back()?.close as f64;
        let gap_ok = Self::gap_pct(&state.candles)
            .map(|g| g <= self.params.max_gap_pct)
            .unwrap_or(true);
        if !gap_ok || snap.adx < self.params.no_trade_adx_below {
            return None;
        }

        let trend_ok = close > snap.ema_short
            && snap.ema_short > snap.ema_long
            && snap.rsi >= self.upward_entry_rsi_min()
            && snap.bullish_count_3 >= 2;
        if trend_ok && snap.adx >= self.params.entry_adx_min {
            Some(snap)
        } else {
            None
        }
    }

    fn rebound_entry_ok(&self, symbol: &str) -> Option<LeveragedReboundSnapshot> {
        if !self.params.intraday_rebound_enabled {
            return None;
        }
        let state = self.states.get(symbol)?;
        let baseline_len = self.params.rebound_baseline_ticks.clamp(2, 120);
        let confirm_len = self.params.rebound_confirm_ticks.clamp(2, 60);
        let required_len = baseline_len.saturating_add(confirm_len);
        if state.rebound_prices.len() < required_len {
            return None;
        }
        let snap = self.snapshot_for(symbol)?;
        if snap.adx < self.params.no_trade_adx_below || snap.rsi < self.params.rebound_rsi_min {
            return None;
        }

        let prices: Vec<u64> = state.rebound_prices.iter().copied().collect();
        let start = prices.len().saturating_sub(required_len);
        let window = &prices[start..];
        let (baseline, confirm) = window.split_at(baseline_len);
        let baseline_high = *baseline.iter().max()?;
        let baseline_low = *baseline.iter().min()?;
        let baseline_last = *baseline.last()?;
        let confirm_first = *confirm.first()?;
        let confirm_last = *confirm.last()?;
        if baseline_high == 0 || baseline_low == 0 || confirm_first == 0 {
            return None;
        }

        let pullback_pct =
            (baseline_high.saturating_sub(baseline_low) as f64 / baseline_high as f64) * 100.0;
        let buy_pressure_pct =
            (confirm_last.saturating_sub(confirm_first) as f64 / confirm_first as f64) * 100.0;
        let rebound_from_low_pct =
            (confirm_last.saturating_sub(baseline_low) as f64 / baseline_low as f64) * 100.0;

        if pullback_pct >= self.params.rebound_pullback_pct
            && buy_pressure_pct >= self.params.rebound_buy_pressure_pct
            && confirm_last > baseline_last
        {
            Some(LeveragedReboundSnapshot {
                trend: snap,
                pullback_pct,
                buy_pressure_pct,
                rebound_from_low_pct,
            })
        } else {
            None
        }
    }

    fn exit_reason(&self, symbol: &str) -> Option<String> {
        let state = self.states.get(symbol)?;
        let snap = self.snapshot_for(symbol)?;
        let close = state.candles.back()?.close as f64;

        if close < snap.ema_short {
            return Some(format!(
                "{} EMA{} 하향 이탈",
                symbol, self.params.ema_short_period
            ));
        }
        if snap.ema_short < snap.ema_long {
            return Some(format!(
                "{} EMA{} < EMA{}",
                symbol, self.params.ema_short_period, self.params.ema_long_period
            ));
        }
        if snap.rsi < self.params.exit_rsi_below {
            return Some(format!(
                "{} RSI {:.1} < {:.1}",
                symbol, snap.rsi, self.params.exit_rsi_below
            ));
        }
        None
    }

    #[cfg(test)]
    fn session_minutes(_is_overseas: bool, _toss_us_session: &str) -> Option<(i64, i64)> {
        Some((60, 10_000))
    }

    #[cfg(not(test))]
    fn session_minutes(is_overseas: bool, toss_us_session: &str) -> Option<(i64, i64)> {
        use chrono::Timelike;
        let now = chrono::Local::now();
        let mins = now.hour() as i64 * 60 + now.minute() as i64;
        let contains = |open: i64, close: i64| -> Option<(i64, i64)> {
            if open <= close {
                (mins >= open && mins < close).then_some((mins - open, close - mins))
            } else if mins >= open {
                Some((mins - open, (24 * 60 - mins) + close))
            } else if mins < close {
                Some(((24 * 60 - open) + mins, close - mins))
            } else {
                None
            }
        };
        if is_overseas {
            let day = (9 * 60, 16 * 60 + 50);
            let pre = (17 * 60, 22 * 60 + 30);
            let regular = (22 * 60 + 30, 5 * 60);
            let after = (5 * 60, 7 * 60);
            match crate::market_hours::UsTradingSessionPolicy::parse(Some(toss_us_session)) {
                crate::market_hours::UsTradingSessionPolicy::Auto => [day, pre, regular, after]
                    .into_iter()
                    .find_map(|(open, close)| contains(open, close)),
                crate::market_hours::UsTradingSessionPolicy::Day => contains(day.0, day.1),
                crate::market_hours::UsTradingSessionPolicy::Pre => contains(pre.0, pre.1),
                crate::market_hours::UsTradingSessionPolicy::Regular => {
                    contains(regular.0, regular.1)
                }
                crate::market_hours::UsTradingSessionPolicy::After => contains(after.0, after.1),
            }
        } else {
            let open = 9 * 60;
            let close = 15 * 60 + 30;
            contains(open, close)
        }
    }

    fn in_blackout_window(windows: &[String]) -> bool {
        use chrono::Timelike;
        let now = chrono::Local::now();
        let mins = now.hour() as i64 * 60 + now.minute() as i64;
        windows.iter().any(|w| {
            let Some((start, end)) = w.split_once('-') else {
                return false;
            };
            let Some(s) = parse_hhmm(start) else {
                return false;
            };
            let Some(e) = parse_hhmm(end) else {
                return false;
            };
            if s <= e {
                mins >= s && mins <= e
            } else {
                mins >= s || mins <= e
            }
        })
    }
}

fn parse_hhmm(value: &str) -> Option<i64> {
    let (h, m) = value.trim().split_once(':')?;
    let h = h.parse::<i64>().ok()?;
    let m = m.parse::<i64>().ok()?;
    if (0..24).contains(&h) && (0..60).contains(&m) {
        Some(h * 60 + m)
    } else {
        None
    }
}

impl Strategy for LeveragedTrendHoldStrategy {
    fn id(&self) -> &str {
        &self.config.id
    }
    fn name(&self) -> &str {
        &self.config.name
    }
    fn config(&self) -> &StrategyConfig {
        &self.config
    }
    fn config_mut(&mut self) -> &mut StrategyConfig {
        &mut self.config
    }
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn initialize_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        self.sync_params();
        if !self.is_target_symbol(symbol) {
            return;
        }
        let cap = self.window_cap();
        let rebound_cap = self.rebound_price_cap();
        let mut state = LeveragedTrendHoldMarketState {
            candles: VecDeque::with_capacity(cap),
            rebound_prices: VecDeque::with_capacity(rebound_cap),
            live_candle_started: false,
        };
        let take = candles.len().min(cap);
        for candle in &candles[candles.len().saturating_sub(take)..] {
            state.candles.push_back(*candle);
        }
        self.states.insert(symbol.to_string(), state);
        tracing::info!(
            "레버리지 단일 티커 추세 초기화 [{}]: OHLC {}봉 로드",
            symbol,
            take
        );
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.sync_params();
        if !self.is_target_symbol(symbol) {
            return;
        }
        let cap = self.rebound_price_cap();
        let window_cap = self.window_cap();
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| {
            LeveragedTrendHoldMarketState {
                candles: VecDeque::with_capacity(window_cap),
                rebound_prices: VecDeque::with_capacity(cap),
                live_candle_started: false,
            }
        });
        state.rebound_prices.clear();
        let take = prices.len().min(cap);
        for price in prices[prices.len().saturating_sub(take)..]
            .iter()
            .copied()
            .filter(|price| *price > 0)
        {
            state.rebound_prices.push_back(price);
        }
        tracing::info!(
            "레버리지 단일 티커 장중 반동 초기화 [{}]: 가격 {}개 로드",
            symbol,
            state.rebound_prices.len()
        );
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        self.sync_params();
        let Some(entry) = self.entry_for_symbol(symbol) else {
            return Signal::Hold;
        };
        self.update_target_tick(symbol, price);

        let quantity = entry.quantity.max(1);
        let (in_position, high_water) = self
            .positions
            .get(symbol)
            .map(|p| (p.in_position, p.high_water))
            .unwrap_or((false, None));

        if in_position {
            let high = high_water.unwrap_or(price).max(price);
            if let Some(pos) = self.positions.get_mut(symbol) {
                pos.high_water = Some(high);
            }
            if high > 0 {
                let drawdown = (high as f64 - price as f64) / high as f64 * 100.0;
                if drawdown >= self.params.trailing_stop_pct {
                    if let Some(pos) = self.positions.get_mut(symbol) {
                        pos.in_position = false;
                        pos.entry_price = None;
                        pos.high_water = None;
                    }
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity,
                        reason: format!(
                            "LeveragedTrendHold 추적손절: 고점 대비 -{:.2}% (기준 {:.2}%)",
                            drawdown, self.params.trailing_stop_pct
                        ),
                    };
                }
            }

            if let Some(reason) = self.exit_reason(symbol) {
                if let Some(pos) = self.positions.get_mut(symbol) {
                    pos.in_position = false;
                    pos.entry_price = None;
                    pos.high_water = None;
                }
                return Signal::Sell {
                    symbol: symbol.to_string(),
                    quantity,
                    reason: format!("LeveragedTrendHold 추세 청산: {}", reason),
                };
            }

            if let Some((_, minutes_to_close)) =
                Self::session_minutes(entry.is_overseas, &self.params.toss_us_session)
            {
                if minutes_to_close <= self.params.exit_before_close_min {
                    if let Some(pos) = self.positions.get_mut(symbol) {
                        pos.in_position = false;
                        pos.entry_price = None;
                        pos.high_water = None;
                    }
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity,
                        reason: format!(
                            "LeveragedTrendHold 장마감 청산: 마감 {}분 전",
                            minutes_to_close
                        ),
                    };
                }
            }

            return Signal::Hold;
        }

        let Some((elapsed, _)) =
            Self::session_minutes(entry.is_overseas, &self.params.toss_us_session)
        else {
            return Signal::Hold;
        };
        let in_blackout = Self::in_blackout_window(&self.params.blackout_windows);
        let in_trend_window = elapsed >= self.params.entry_window_start_min
            && elapsed <= self.params.entry_window_end_min
            && !in_blackout;
        let can_check_rebound = self.params.intraday_rebound_enabled && !in_blackout;

        if in_trend_window {
            if let Some(snap) = self.entry_ok(symbol) {
                self.positions.insert(
                    symbol.to_string(),
                    LeveragedTrendHoldPosition {
                        in_position: true,
                        entry_price: Some(price),
                        high_water: Some(price),
                    },
                );
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity,
                    reason: format!(
                        "LeveragedTrendHold 상승 추세 진입: {} EMA{} > EMA{}, RSI {:.1}, ADX {:.1}, 최근 3봉 양봉 {}개",
                        symbol,
                        self.params.ema_short_period,
                        self.params.ema_long_period,
                        snap.rsi,
                        snap.adx,
                        snap.bullish_count_3
                    ),
                };
            }
        }

        if can_check_rebound {
            if let Some(snap) = self.rebound_entry_ok(symbol) {
                self.positions.insert(
                    symbol.to_string(),
                    LeveragedTrendHoldPosition {
                        in_position: true,
                        entry_price: Some(price),
                        high_water: Some(price),
                    },
                );
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity,
                    reason: format!(
                        "LeveragedTrendHold 장중 매수세 반동 진입: 확인 구간 +{:.2}% (기준 구간 하락 {:.2}%, 저점 대비 +{:.2}%), RSI {:.1}, ADX {:.1}",
                        snap.buy_pressure_pct,
                        snap.pullback_pct,
                        snap.rebound_from_low_pct,
                        snap.trend.rsi,
                        snap.trend.adx
                    ),
                };
            }
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        self.sync_params();
        if quantity == 0 || !self.is_target_symbol(symbol) {
            return;
        }
        self.positions.insert(
            symbol.to_string(),
            LeveragedTrendHoldPosition {
                in_position: true,
                entry_price: Some(avg_price),
                high_water: Some(avg_price),
            },
        );
        tracing::info!(
            "레버리지 단일 티커 추세 포지션 동기화: {} {}주 @ {}",
            symbol,
            quantity,
            avg_price
        );
    }

    fn reset(&mut self) {
        for state in self.states.values_mut() {
            state.live_candle_started = false;
            state.rebound_prices.clear();
        }
        for pos in self.positions.values_mut() {
            pos.in_position = false;
            pos.entry_price = None;
            pos.high_water = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upward_candles() -> Vec<OhlcCandle> {
        (0..90)
            .map(|i| {
                let open = 100 + i;
                OhlcCandle {
                    open,
                    high: open + 3,
                    low: open.saturating_sub(1),
                    close: open + 2,
                }
            })
            .collect()
    }

    fn downward_candles() -> Vec<OhlcCandle> {
        (0..90)
            .map(|i| {
                let open = 200_u64.saturating_sub(i);
                OhlcCandle {
                    open,
                    high: open + 1,
                    low: open.saturating_sub(3),
                    close: open.saturating_sub(2),
                }
            })
            .collect()
    }

    fn entry(symbol: &str) -> LeveragedTrendHoldEntry {
        LeveragedTrendHoldEntry {
            leveraged_symbol: symbol.into(),
            leveraged_symbol_name: symbol.into(),
            inverse_leveraged_symbol: "SOXS".into(),
            inverse_leveraged_symbol_name: "Legacy inverse".into(),
            base_symbols: vec!["SOXX".into()],
            base_symbol_names: HashMap::new(),
            base_symbol_roles: HashMap::new(),
            quantity: 3,
            inverse_quantity: 2,
            is_overseas: true,
        }
    }

    fn strategy_with_params(params: LeveragedTrendHoldParams) -> LeveragedTrendHoldStrategy {
        let config = StrategyConfig::new(
            "leveraged_trend_hold_test",
            "레버리지 단일 티커 추세 테스트",
            true,
            Vec::new(),
            1,
            serde_json::to_value(params).unwrap(),
        );
        LeveragedTrendHoldStrategy::new(config)
    }

    #[test]
    fn target_symbols_include_only_configured_tickers() {
        let params = LeveragedTrendHoldParams {
            entries: vec![entry("SOXL")],
            ..LeveragedTrendHoldParams::default()
        };
        let mut strategy = strategy_with_params(params);

        assert_eq!(strategy.config.target_symbols, vec!["SOXL".to_string()]);
        strategy.initialize_ohlc("SOXX", &upward_candles());
        assert_eq!(strategy.on_tick("SOXX", 190, 100), Signal::Hold);
        assert_eq!(strategy.on_tick("SOXS", 50, 100), Signal::Hold);
    }

    #[test]
    fn buys_any_target_ticker_when_itself_trends_up() {
        let params = LeveragedTrendHoldParams {
            entries: vec![entry("SOXS")],
            entry_adx_min: 0.0,
            no_trade_adx_below: 0.0,
            entry_window_end_min: 120,
            ..LeveragedTrendHoldParams::default()
        };
        let mut strategy = strategy_with_params(params);
        strategy.initialize_ohlc("SOXS", &upward_candles());

        let signal = strategy.on_tick("SOXS", 192, 100);

        assert!(
            matches!(signal, Signal::Buy { symbol, quantity, .. } if symbol == "SOXS" && quantity == 3)
        );
    }

    #[test]
    fn sells_target_when_its_own_trend_breaks() {
        let params = LeveragedTrendHoldParams {
            entries: vec![entry("SOXL")],
            entry_adx_min: 0.0,
            no_trade_adx_below: 0.0,
            trailing_stop_pct: 99.0,
            ..LeveragedTrendHoldParams::default()
        };
        let mut strategy = strategy_with_params(params);
        strategy.initialize_ohlc("SOXL", &downward_candles());
        strategy.sync_position("SOXL", 3, 300);

        let signal = strategy.on_tick("SOXL", 108, 100);

        assert!(
            matches!(signal, Signal::Sell { symbol, quantity, reason } if symbol == "SOXL" && quantity == 3 && reason.contains("추세 청산"))
        );
    }

    #[test]
    fn buys_intraday_rebound_when_next_window_shows_buy_pressure() {
        let params = LeveragedTrendHoldParams {
            entries: vec![entry("KORU")],
            intraday_rebound_enabled: true,
            rebound_baseline_ticks: 2,
            rebound_confirm_ticks: 2,
            rebound_pullback_pct: 4.0,
            rebound_buy_pressure_pct: 2.0,
            rebound_rsi_min: 20.0,
            no_trade_adx_below: 0.0,
            ..LeveragedTrendHoldParams::default()
        };
        let mut strategy = strategy_with_params(params);
        strategy.initialize_ohlc("KORU", &upward_candles());

        assert_eq!(strategy.on_tick("KORU", 190, 100), Signal::Hold);
        assert_eq!(strategy.on_tick("KORU", 180, 100), Signal::Hold);
        assert_eq!(strategy.on_tick("KORU", 181, 100), Signal::Hold);
        let signal = strategy.on_tick("KORU", 186, 100);

        assert!(
            matches!(signal, Signal::Buy { symbol, quantity, reason } if symbol == "KORU" && quantity == 3 && reason.contains("매수세 반동 진입"))
        );
    }

    #[test]
    fn ignores_intraday_rebound_by_default() {
        let params = LeveragedTrendHoldParams {
            entries: vec![entry("KORU")],
            rebound_baseline_ticks: 2,
            rebound_confirm_ticks: 2,
            no_trade_adx_below: 0.0,
            ..LeveragedTrendHoldParams::default()
        };
        let mut strategy = strategy_with_params(params);
        strategy.initialize_ohlc("KORU", &upward_candles());

        assert_eq!(strategy.on_tick("KORU", 190, 100), Signal::Hold);
        assert_eq!(strategy.on_tick("KORU", 180, 100), Signal::Hold);
        assert_eq!(strategy.on_tick("KORU", 181, 100), Signal::Hold);
        assert_eq!(strategy.on_tick("KORU", 186, 100), Signal::Hold);
    }
}
