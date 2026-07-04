use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::{state::bounded_window_with_extra, OhlcCandle, Signal, Strategy, StrategyConfig};

// ────────────────────────────────────────────────────────────────────
// 11. 레버리지 추세 보유 전략 (LeveragedTrendHoldStrategy)
// ────────────────────────────────────────────────────────────────────
// 기초 ETF(SOXX/SMH 등)의 추세 조건이 좋을 때 레버리지 ETF(SOXL 등)를 매수하고,
// 레버리지 가격의 고점 대비 하락 또는 기초 ETF 추세 훼손 시 청산한다.
// target_symbols에는 기초 종목과 레버리지 종목이 모두 들어간다.
// ────────────────────────────────────────────────────────────────────

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
fn lth_default_neutral_low() -> f64 {
    45.0
}
fn lth_default_neutral_high() -> f64 {
    55.0
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeveragedTrendHoldEntry {
    /// 매수/청산 대상 레버리지 종목 (예: SOXL)
    pub leveraged_symbol: String,
    /// UI 표시용 종목명
    #[serde(default)]
    pub leveraged_symbol_name: String,
    /// 하락 추세에서 매수할 역방향 레버리지 종목 (예: SOXS). 비어 있으면 비활성.
    #[serde(default)]
    pub inverse_leveraged_symbol: String,
    /// 역방향 레버리지 종목명 (UI 표시용)
    #[serde(default)]
    pub inverse_leveraged_symbol_name: String,
    /// 추세 판단에 사용할 기초 종목들 (예: SOXX, SMH). 하나라도 통과하면 진입 가능.
    #[serde(default)]
    pub base_symbols: Vec<String>,
    /// 기초 종목명 캐시 (UI 표시용)
    #[serde(default)]
    pub base_symbol_names: HashMap<String, String>,
    /// 기초 종목 역할. "underlying"은 직접 기초 ETF, "proxy"는 TECL -> VGT 같은 유사 기초 ETF.
    #[serde(default)]
    pub base_symbol_roles: HashMap<String, String>,
    /// 1회 주문 수량
    #[serde(default = "lth_default_qty")]
    pub quantity: u64,
    /// 역방향 레버리지 1회 주문 수량
    #[serde(default = "lth_default_qty")]
    pub inverse_quantity: u64,
    /// 해외 주식 여부. true이면 가격 단위 = USD, on_tick 내부 가격은 cents.
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
    /// 상승 추세 진입 민감도. 1.0은 기존 기준, 값이 높을수록 더 이른 상승 진입을 허용.
    #[serde(default = "lth_default_sensitivity")]
    pub upward_sensitivity: f64,
    /// 하락 추세 진입 민감도. 1.0은 기존 기준, 값이 높을수록 더 이른 하락 진입을 허용.
    #[serde(default = "lth_default_sensitivity")]
    pub downward_sensitivity: f64,
    #[serde(default = "lth_default_sell_rsi")]
    pub exit_rsi_below: f64,
    #[serde(default = "lth_default_buy_adx")]
    pub entry_adx_min: f64,
    #[serde(default = "lth_default_no_trade_adx")]
    pub no_trade_adx_below: f64,
    #[serde(default = "lth_default_neutral_low")]
    pub neutral_rsi_low: f64,
    #[serde(default = "lth_default_neutral_high")]
    pub neutral_rsi_high: f64,
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
    /// 주요 지표 발표 전후 등 수동 거래 금지 구간. 예: ["23:25-23:45", "02:55-03:10"]
    #[serde(default)]
    pub blackout_windows: Vec<String>,
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
            neutral_rsi_low: lth_default_neutral_low(),
            neutral_rsi_high: lth_default_neutral_high(),
            trailing_stop_pct: lth_default_trailing_stop(),
            entry_window_start_min: lth_default_entry_start(),
            entry_window_end_min: lth_default_entry_end(),
            exit_before_close_min: lth_default_exit_before_close(),
            max_gap_pct: lth_default_gap_limit(),
            blackout_windows: Vec::new(),
        }
    }
}

struct LeveragedTrendHoldMarketState {
    candles: VecDeque<OhlcCandle>,
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
    bearish_count_3: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeveragedTrendDirection {
    Long,
    Inverse,
}

pub struct LeveragedTrendHoldStrategy {
    config: StrategyConfig,
    params: LeveragedTrendHoldParams,
    base_states: HashMap<String, LeveragedTrendHoldMarketState>,
    positions: HashMap<String, LeveragedTrendHoldPosition>,
    last_params: serde_json::Value,
}

impl LeveragedTrendHoldStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: LeveragedTrendHoldParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        let last_params = config.params.clone();
        Self {
            config,
            params,
            base_states: HashMap::new(),
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
        let mut symbols = Vec::new();
        for entry in &self.params.entries {
            symbols.push(entry.leveraged_symbol.clone());
            if !entry.inverse_leveraged_symbol.trim().is_empty() {
                symbols.push(entry.inverse_leveraged_symbol.clone());
            }
            symbols.extend(entry.base_symbols.iter().cloned());
        }
        symbols.retain(|s| !s.trim().is_empty());
        symbols.sort_unstable();
        symbols.dedup();
        self.config.target_symbols = symbols;
    }

    fn entries_for_symbol(
        &self,
        symbol: &str,
    ) -> Vec<(LeveragedTrendHoldEntry, LeveragedTrendDirection)> {
        self.params
            .entries
            .iter()
            .filter_map(|entry| {
                if entry.base_symbols.is_empty() {
                    return None;
                }
                if entry.leveraged_symbol == symbol {
                    Some((entry.clone(), LeveragedTrendDirection::Long))
                } else if entry.inverse_leveraged_symbol == symbol {
                    Some((entry.clone(), LeveragedTrendDirection::Inverse))
                } else {
                    None
                }
            })
            .collect()
    }

    fn is_base_symbol(&self, symbol: &str) -> bool {
        self.params
            .entries
            .iter()
            .any(|e| e.base_symbols.iter().any(|b| b == symbol))
    }

    fn update_base_tick(&mut self, symbol: &str, price: u64) {
        let cap = bounded_window_with_extra(
            self.params
                .ema_long_period
                .max(self.params.adx_period + 2)
                .max(80),
            5,
        );
        let state = self
            .base_states
            .entry(symbol.to_string())
            .or_insert_with(|| LeveragedTrendHoldMarketState {
                candles: VecDeque::with_capacity(cap),
                live_candle_started: false,
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

    fn bearish_count(candles: &VecDeque<OhlcCandle>, count: usize) -> usize {
        candles
            .iter()
            .rev()
            .take(count)
            .filter(|c| c.close < c.open)
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

    fn snapshot_for(&self, base_symbol: &str) -> Option<LeveragedTrendSnapshot> {
        let state = self.base_states.get(base_symbol)?;
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
            bearish_count_3: Self::bearish_count(&state.candles, 3),
        })
    }

    fn upward_entry_rsi_min(&self) -> f64 {
        let sensitivity = self.params.upward_sensitivity.clamp(1.0, 5.0);
        (self.params.entry_rsi_min - (sensitivity - 1.0) * 2.0).clamp(45.0, 70.0)
    }

    fn downward_entry_rsi_max(&self) -> f64 {
        let sensitivity = self.params.downward_sensitivity.clamp(1.0, 5.0);
        (self.params.neutral_rsi_low + (sensitivity - 1.0) * 2.0).clamp(30.0, 55.0)
    }

    fn base_entry_ok(
        &self,
        base_symbol: &str,
        direction: LeveragedTrendDirection,
    ) -> Option<LeveragedTrendSnapshot> {
        let state = self.base_states.get(base_symbol)?;
        let snap = self.snapshot_for(base_symbol)?;
        let close = state.candles.back()?.close as f64;
        let gap_ok = Self::gap_pct(&state.candles)
            .map(|g| g <= self.params.max_gap_pct)
            .unwrap_or(true);
        if !gap_ok || snap.adx < self.params.no_trade_adx_below {
            return None;
        }

        let trend_ok = match direction {
            LeveragedTrendDirection::Long => {
                close > snap.ema_short
                    && snap.ema_short > snap.ema_long
                    && snap.rsi >= self.upward_entry_rsi_min()
                    && snap.bullish_count_3 >= 2
            }
            LeveragedTrendDirection::Inverse => {
                close < snap.ema_short
                    && snap.ema_short < snap.ema_long
                    && snap.rsi <= self.downward_entry_rsi_max()
                    && snap.bearish_count_3 >= 2
            }
        };

        if trend_ok && snap.adx >= self.params.entry_adx_min {
            return Some(snap);
        }

        None
    }

    fn base_exit_reason(
        &self,
        base_symbol: &str,
        direction: LeveragedTrendDirection,
    ) -> Option<String> {
        let state = self.base_states.get(base_symbol)?;
        let snap = self.snapshot_for(base_symbol)?;
        let close = state.candles.back()?.close as f64;

        match direction {
            LeveragedTrendDirection::Long => {
                if close < snap.ema_short {
                    return Some(format!("{} EMA20 하향 이탈", base_symbol));
                }
                if snap.rsi < self.params.exit_rsi_below {
                    return Some(format!(
                        "{} RSI {:.1} < {:.1}",
                        base_symbol, snap.rsi, self.params.exit_rsi_below
                    ));
                }
            }
            LeveragedTrendDirection::Inverse => {
                if close > snap.ema_short {
                    return Some(format!("{} EMA20 상향 회복", base_symbol));
                }
                let inverse_exit_rsi = 100.0 - self.params.exit_rsi_below;
                if snap.rsi > inverse_exit_rsi {
                    return Some(format!(
                        "{} RSI {:.1} > {:.1}",
                        base_symbol, snap.rsi, inverse_exit_rsi
                    ));
                }
            }
        }

        None
    }

    fn session_minutes(is_overseas: bool) -> Option<(i64, i64)> {
        use chrono::Timelike;
        let now = chrono::Local::now();
        let mins = now.hour() as i64 * 60 + now.minute() as i64;
        if is_overseas {
            let open = 22 * 60 + 30;
            let close = 5 * 60;
            if mins >= open {
                Some((mins - open, (24 * 60 - mins) + close))
            } else if mins < close {
                Some(((24 * 60 - open) + mins, close - mins))
            } else {
                None
            }
        } else {
            let open = 9 * 60;
            let close = 15 * 60 + 30;
            if mins >= open && mins < close {
                Some((mins - open, close - mins))
            } else {
                None
            }
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

    fn base_role_label(entry: &LeveragedTrendHoldEntry, base_symbol: &str) -> &'static str {
        match entry.base_symbol_roles.get(base_symbol).map(String::as_str) {
            Some("proxy") => "유사기초",
            _ => "기초",
        }
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
        if !self.is_base_symbol(symbol) {
            return;
        }
        let cap = bounded_window_with_extra(
            self.params
                .ema_long_period
                .max(self.params.adx_period + 2)
                .max(80),
            5,
        );
        let mut state = LeveragedTrendHoldMarketState {
            candles: VecDeque::with_capacity(cap),
            live_candle_started: false,
        };
        let take = candles.len().min(cap);
        for candle in &candles[candles.len().saturating_sub(take)..] {
            state.candles.push_back(*candle);
        }
        self.base_states.insert(symbol.to_string(), state);
        tracing::info!(
            "레버리지 추세 보유 초기화 [{}]: OHLC {}봉 로드",
            symbol,
            take
        );
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        self.sync_params();

        if self.is_base_symbol(symbol) {
            self.update_base_tick(symbol, price);
            return Signal::Hold;
        }

        let entries = self.entries_for_symbol(symbol);
        if entries.is_empty() {
            return Signal::Hold;
        }

        for (entry, direction) in entries {
            let quantity = match direction {
                LeveragedTrendDirection::Long => entry.quantity,
                LeveragedTrendDirection::Inverse => entry.inverse_quantity,
            };
            let direction_label = match direction {
                LeveragedTrendDirection::Long => "정방향",
                LeveragedTrendDirection::Inverse => "역방향",
            };

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
                                "LeveragedTrendHold {} 추적손절: 고점 대비 -{:.2}% (기준 {:.2}%)",
                                direction_label, drawdown, self.params.trailing_stop_pct
                            ),
                        };
                    }
                }

                let base_exit_reason = entry
                    .base_symbols
                    .iter()
                    .filter_map(|base| self.base_exit_reason(base, direction))
                    .next();
                if let Some(reason) = base_exit_reason {
                    if let Some(pos) = self.positions.get_mut(symbol) {
                        pos.in_position = false;
                        pos.entry_price = None;
                        pos.high_water = None;
                    }
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity,
                        reason: format!(
                            "LeveragedTrendHold {} 추세 청산: {}",
                            direction_label, reason
                        ),
                    };
                }

                if let Some((_, minutes_to_close)) = Self::session_minutes(entry.is_overseas) {
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
                                "LeveragedTrendHold {} 장마감 청산: 마감 {}분 전",
                                direction_label, minutes_to_close
                            ),
                        };
                    }
                }

                continue;
            }

            let Some((elapsed, _)) = Self::session_minutes(entry.is_overseas) else {
                continue;
            };
            if elapsed < self.params.entry_window_start_min
                || elapsed > self.params.entry_window_end_min
                || Self::in_blackout_window(&self.params.blackout_windows)
            {
                continue;
            }

            let base_entry = entry
                .base_symbols
                .iter()
                .filter_map(|base| self.base_entry_ok(base, direction).map(|snap| (base, snap)))
                .next();

            if let Some((base, snap)) = base_entry {
                let base_role_label = Self::base_role_label(&entry, base);
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
                    reason: match direction {
                        LeveragedTrendDirection::Long => format!(
                            "LeveragedTrendHold 정방향 진입: {} {} EMA{} > EMA{}, RSI {:.1}, ADX {:.1}, 최근 3봉 양봉 {}개",
                            base_role_label,
                            base,
                            self.params.ema_short_period,
                            self.params.ema_long_period,
                            snap.rsi,
                            snap.adx,
                            snap.bullish_count_3
                        ),
                        LeveragedTrendDirection::Inverse => format!(
                            "LeveragedTrendHold 역방향 진입: {} {} EMA{} < EMA{}, RSI {:.1}, ADX {:.1}, 최근 3봉 음봉 {}개",
                            base_role_label,
                            base,
                            self.params.ema_short_period,
                            self.params.ema_long_period,
                            snap.rsi,
                            snap.adx,
                            snap.bearish_count_3
                        ),
                    },
                };
            }
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        self.sync_params();
        if quantity == 0
            || !self
                .params
                .entries
                .iter()
                .any(|e| e.leveraged_symbol == symbol || e.inverse_leveraged_symbol == symbol)
        {
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
            "레버리지 추세 보유 포지션 동기화: {} {}주 @ {}",
            symbol,
            quantity,
            avg_price
        );
    }

    fn reset(&mut self) {
        for state in self.base_states.values_mut() {
            state.live_candle_started = false;
        }
        for pos in self.positions.values_mut() {
            pos.in_position = false;
            pos.entry_price = None;
            pos.high_water = None;
        }
    }
}
