use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{
    state::bounded_window, HistoryReadiness, OhlcCandle, Signal, Strategy, StrategyConfig,
};

// ────────────────────────────────────────────────────────────────────
// 52주 신고가 전략 (52-Week High Breakout)
// ────────────────────────────────────────────────────────────────────

/// 52주 신고가 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiftyTwoWeekHighParams {
    /// 조회 기간 (거래일 수, 기본 252 ≈ 1년)
    pub lookback_days: usize,
    /// 손절 기준 (매수가 대비 하락 %, 기본 3.0)
    pub stop_loss_pct: f64,
}

impl Default for FiftyTwoWeekHighParams {
    fn default() -> Self {
        Self {
            lookback_days: 252,
            stop_loss_pct: 3.0,
        }
    }
}

/// 종목별 52주 신고가 상태
struct FiftyTwoWeekState {
    prev_price: Option<u64>,
    high_52w: Option<u64>,
    history_bars: usize,
    buy_price: Option<u64>,
}

pub struct FiftyTwoWeekHighStrategy {
    config: StrategyConfig,
    params: FiftyTwoWeekHighParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, FiftyTwoWeekState>,
}

impl FiftyTwoWeekHighStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: FiftyTwoWeekHighParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for FiftyTwoWeekHighStrategy {
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

    // 기존 단일 가격 API는 고가 배열을 받는다. 완료된 과거 봉 전체만 사용한다.
    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let required = bounded_window(self.params.lookback_days);
        let history = &prices[prices.len().saturating_sub(required)..];
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(FiftyTwoWeekState {
                prev_price: None,
                high_52w: None,
                history_bars: 0,
                buy_price: None,
            });
        state.history_bars = history.len();
        state.high_52w = if history.len() == required {
            history.iter().copied().max().filter(|high| *high > 0)
        } else {
            None
        };
        state.prev_price = history.last().copied();
    }

    fn initialize_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        let cap = bounded_window(self.params.lookback_days);
        let highs = candles[candles.len().saturating_sub(cap)..]
            .iter()
            .map(|candle| candle.high)
            .collect::<Vec<_>>();
        self.initialize_historical(symbol, &highs);
        if let Some(state) = self.states.get_mut(symbol) {
            state.prev_price = candles.last().map(|candle| candle.close);
        }
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars: bounded_window(self.params.lookback_days),
            available_bars: state.map_or(0, |state| state.history_bars),
            ready: state
                .is_some_and(|state| state.high_52w.is_some() && state.prev_price.is_some()),
        })
    }

    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        Some(self.states.get(symbol).is_some_and(|state| {
            state.buy_price.is_some() || (state.high_52w.is_some() && state.prev_price.is_some())
        }))
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(FiftyTwoWeekState {
                prev_price: None,
                high_52w: None,
                history_bars: 0,
                buy_price: None,
            });

        // ① 손절 체크
        if let Some(bp) = state.buy_price {
            let stop_price = (bp as f64 * (1.0 - self.params.stop_loss_pct / 100.0)) as u64;
            if price <= stop_price {
                state.buy_price = None;
                state.prev_price = Some(price);
                return Signal::Sell {
                    symbol: symbol.to_string(),
                    quantity: self.config.order_quantity,
                    reason: format!(
                        "52주 신고가 손절: -{}% ({:.0}원 → {:.0}원)",
                        self.params.stop_loss_pct, bp as f64, price as f64
                    ),
                };
            }
        }

        let signal = match state.high_52w {
            None => Signal::Hold,
            Some(high) => {
                // ② 52주 신고가 돌파 감지
                let crossed = state
                    .prev_price
                    .is_some_and(|prev| prev <= high && price > high);
                if crossed && state.buy_price.is_none() {
                    state.high_52w = Some(price);
                    state.buy_price = Some(price);
                    state.prev_price = Some(price);
                    return Signal::Buy {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!(
                            "52주 신고가 돌파: {:.0}원 (이전 고가 {:.0}원)",
                            price as f64, high as f64
                        ),
                    };
                }
                if price > high {
                    state.high_52w = Some(price);
                }
                Signal::Hold
            }
        };

        state.prev_price = Some(price);
        signal
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(FiftyTwoWeekState {
                prev_price: None,
                high_52w: None,
                history_bars: 0,
                buy_price: None,
            });
        state.buy_price = (quantity > 0).then_some(avg_price);
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 07. 강한 종가 전략 (StrongCloseStrategy)
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 자동매매 시작 시 `initialize_candles`로 일봉 (고가, 종가) 배열 전달
//  2. 전일 종가가 전일 고가 대비 threshold_pct% 이내이면 "강한 종가" → 다음날(당일) 매수 신호 대기
//  3. 당일 첫 틱 수신 시 매수 신호 발생 (1회 발생 후 pending 해제)
//  4. 매도 조건: 매수 후 현재가가 매수가 대비 stop_loss_pct% 이상 하락 시 손절
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrongCloseParams {
    /// 고가 대비 종가가 이 % 이내이면 강한 종가로 판단 (기본 3.0)
    pub threshold_pct: f64,
    /// 매수 후 손절 기준 % (기본 3.0)
    pub stop_loss_pct: f64,
}

impl Default for StrongCloseParams {
    fn default() -> Self {
        Self {
            threshold_pct: 3.0,
            stop_loss_pct: 3.0,
        }
    }
}

/// 종목별 강한종가 상태
struct StrongCloseState {
    pending_buy: bool,
    history_bars: usize,
    in_position: bool,
    entry_price: Option<u64>,
}

pub struct StrongCloseStrategy {
    config: StrategyConfig,
    params: StrongCloseParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, StrongCloseState>,
}

impl StrongCloseStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: StrongCloseParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for StrongCloseStrategy {
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

    fn initialize_candles(&mut self, symbol: &str, candles: &[(u64, u64)]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(StrongCloseState {
                pending_buy: false,
                history_bars: 0,
                in_position: false,
                entry_price: None,
            });
        state.history_bars = usize::from(
            candles
                .last()
                .is_some_and(|(high, close)| *high > 0 && *close > 0 && close <= high),
        );
        state.pending_buy = !state.in_position
            && state.history_bars == 1
            && candles.last().is_some_and(|&(high, close)| {
                (high as f64 - close as f64) / high as f64 * 100.0 <= self.params.threshold_pct
            });
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let available_bars = self
            .states
            .get(symbol)
            .map_or(0, |state| state.history_bars);
        Some(HistoryReadiness {
            required_bars: 1,
            available_bars,
            ready: available_bars == 1,
        })
    }

    fn on_completed_candle(&mut self, symbol: &str, candle: &OhlcCandle) {
        // 완료봉은 체결 피드백 이후에 반영해 다음 거래일의 조건만 준비한다.
        self.initialize_candles(symbol, &[(candle.high, candle.close)]);
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(StrongCloseState {
                pending_buy: false,
                history_bars: 0,
                in_position: false,
                entry_price: None,
            });

        // ① 손절 우선
        if state.in_position {
            if let Some(ep) = state.entry_price {
                let loss_pct = (ep as f64 - price as f64) / ep as f64 * 100.0;
                if loss_pct >= self.params.stop_loss_pct {
                    state.in_position = false;
                    state.entry_price = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!(
                            "강한 종가 손절: 현재가 {} (매수가 {} 대비 -{:.2}%)",
                            price, ep, loss_pct
                        ),
                    };
                }
            }
            return Signal::Hold;
        }

        // ② 강한 종가 후 첫 틱 매수
        if state.pending_buy {
            state.pending_buy = false;
            state.in_position = true;
            state.entry_price = Some(price);
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!(
                    "강한 종가 후 매수: 현재가 {} (전일 종가가 고가 대비 {:.1}% 이내)",
                    price, self.params.threshold_pct
                ),
            };
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(StrongCloseState {
                pending_buy: false,
                history_bars: 0,
                in_position: false,
                entry_price: None,
            });
        state.in_position = quantity > 0;
        state.entry_price = (quantity > 0 && avg_price > 0).then_some(avg_price);
        if quantity > 0 {
            state.pending_buy = false;
        }
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 08. 변동성 확장 전략 (VolatilityExpansionStrategy)
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 자동매매 시작 시 `initialize_range_data`로 일봉 변동폭(고-저) 배열 전달 → 평균 변동폭 계산
//  2. 장중 첫 틱 = 시가(day_open), 이후 틱마다 당일 고/저 추적
//  3. 당일 변동폭 > 평균 변동폭 × expansion_factor AND 현재가 > day_open → 매수 (변동성 방향 확인)
//  4. 매수 후 stop_loss_pct% 하락 시 손절 매도
// ────────────────────────────────────────────────────────────────────

/// 변동성 확장 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityExpansionParams {
    /// 평균 변동폭 계산에 사용할 과거 기간 (기본 10거래일)
    pub lookback_days: usize,
    /// 평균 변동폭 대비 확장 배율 (기본 2.0배 이상이면 발동)
    pub expansion_factor: f64,
    /// 매수 후 손절 기준 % (기본 3.0)
    pub stop_loss_pct: f64,
}

impl Default for VolatilityExpansionParams {
    fn default() -> Self {
        Self {
            lookback_days: 10,
            expansion_factor: 2.0,
            stop_loss_pct: 3.0,
        }
    }
}

/// 종목별 변동성 확장 상태
struct VolatilityExpansionState {
    /// 현재 거래일을 제외한 최근 완료 일봉 변동폭. 보합일 0도 포함한다.
    completed_ranges: VecDeque<u64>,
    avg_range: Option<f64>,
    history_bars: usize,
    day_open: Option<u64>,
    day_high: u64,
    day_low: u64,
    in_position: bool,
    entry_price: Option<u64>,
}

pub struct VolatilityExpansionStrategy {
    config: StrategyConfig,
    params: VolatilityExpansionParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, VolatilityExpansionState>,
}

impl VolatilityExpansionStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: VolatilityExpansionParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for VolatilityExpansionStrategy {
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

    fn initialize_range_data(&mut self, symbol: &str, ranges: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let lookback = bounded_window(self.params.lookback_days);
        let history = &ranges[ranges.len().saturating_sub(lookback)..];
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(VolatilityExpansionState {
                completed_ranges: VecDeque::with_capacity(lookback),
                avg_range: None,
                history_bars: 0,
                day_open: None,
                day_high: 0,
                day_low: u64::MAX,
                in_position: false,
                entry_price: None,
            });
        state.completed_ranges.clear();
        state.completed_ranges.extend(history.iter().copied());
        state.history_bars = history.len();
        state.avg_range = (history.len() == lookback)
            .then(|| history.iter().map(|range| *range as f64).sum::<f64>() / lookback as f64);
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars: bounded_window(self.params.lookback_days),
            available_bars: state.map_or(0, |state| state.history_bars),
            ready: state.is_some_and(|state| state.avg_range.is_some()),
        })
    }

    fn on_trading_day_start(&mut self, symbol: &str, open: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(VolatilityExpansionState {
                completed_ranges: VecDeque::new(),
                avg_range: None,
                history_bars: 0,
                day_open: None,
                day_high: 0,
                day_low: u64::MAX,
                in_position: false,
                entry_price: None,
            });
        // 날짜 경계에서 가격 범위만 초기화한다. 보유·평균 범위는 그대로 유지한다.
        state.day_open = Some(open);
        state.day_high = open;
        state.day_low = open;
    }

    fn on_daily_close_tick(&mut self, symbol: &str, candle: &OhlcCandle, volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        if !self.states.contains_key(symbol) {
            self.on_trading_day_start(symbol, candle.open);
        }
        if let Some(state) = self.states.get_mut(symbol) {
            // 완성 OHLC는 종가 시점에만 공개된다. 평균에는 아직 당일 범위를 넣지 않는다.
            state.day_open = Some(candle.open);
            state.day_high = candle.high;
            state.day_low = candle.low;
        }
        self.on_tick(symbol, candle.close, volume)
    }

    fn on_completed_candle(&mut self, symbol: &str, candle: &OhlcCandle) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        if !self.states.contains_key(symbol) {
            self.on_trading_day_start(symbol, candle.open);
        }
        let lookback = bounded_window(self.params.lookback_days);
        if let Some(state) = self.states.get_mut(symbol) {
            while state.completed_ranges.len() >= lookback {
                state.completed_ranges.pop_front();
            }
            state
                .completed_ranges
                .push_back(candle.high.saturating_sub(candle.low));
            state.history_bars = state.completed_ranges.len();
            state.avg_range = (state.history_bars == lookback).then(|| {
                state
                    .completed_ranges
                    .iter()
                    .map(|range| *range as f64)
                    .sum::<f64>()
                    / lookback as f64
            });
        }
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(VolatilityExpansionState {
                completed_ranges: VecDeque::new(),
                avg_range: None,
                history_bars: 0,
                day_open: None,
                day_high: 0,
                day_low: u64::MAX,
                in_position: false,
                entry_price: None,
            });

        if price > state.day_high {
            state.day_high = price;
        }
        if price < state.day_low {
            state.day_low = price;
        }
        if state.day_open.is_none() {
            state.day_open = Some(price);
        }

        // ① 손절 우선
        if state.in_position {
            if let Some(ep) = state.entry_price {
                let loss_pct = (ep as f64 - price as f64) / ep as f64 * 100.0;
                if loss_pct >= self.params.stop_loss_pct {
                    state.in_position = false;
                    state.entry_price = None;
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!(
                            "변동성 확장 손절: 현재가 {} (매수가 {} 대비 -{:.2}%)",
                            price, ep, loss_pct
                        ),
                    };
                }
            }
            return Signal::Hold;
        }

        // ② 매수 조건
        if let (Some(ar), Some(day_open)) = (state.avg_range, state.day_open) {
            if state.day_low == u64::MAX {
                return Signal::Hold;
            }
            let intraday_range = state.day_high.saturating_sub(state.day_low);
            let threshold = ar * self.params.expansion_factor;
            if intraday_range as f64 > threshold && price > day_open {
                state.in_position = true;
                state.entry_price = Some(price);
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity: self.config.order_quantity,
                    reason: format!(
                        "변동성 확장 매수: 당일 변동폭 {}원 > 평균 {:.0}원 × {:.1} (상승 방향)",
                        intraday_range, ar, self.params.expansion_factor
                    ),
                };
            }
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert(VolatilityExpansionState {
                completed_ranges: VecDeque::new(),
                avg_range: None,
                history_bars: 0,
                day_open: None,
                day_high: 0,
                day_low: u64::MAX,
                in_position: false,
                entry_price: None,
            });
        state.in_position = quantity > 0;
        state.entry_price = (quantity > 0 && avg_price > 0).then_some(avg_price);
    }

    fn reset(&mut self) {
        // 일 초기화: 당일 고/저/시가 리셋, avg_range는 유지
        for state in self.states.values_mut() {
            state.day_open = None;
            state.day_high = 0;
            state.day_low = u64::MAX;
            state.in_position = false;
            state.entry_price = None;
        }
    }
}

#[cfg(test)]
#[path = "breakout/day_event_tests.rs"]
mod day_event_tests;

#[cfg(test)]
mod history_tests {
    use super::*;

    fn config(params: serde_json::Value) -> StrategyConfig {
        StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
    }

    #[test]
    fn fifty_two_week_requires_full_window_and_uses_last_completed_high_and_close() {
        let mut strategy = FiftyTwoWeekHighStrategy::new(config(serde_json::json!({})));
        let candle = OhlcCandle {
            open: 100,
            high: 200,
            low: 90,
            close: 100,
        };
        strategy.initialize_ohlc("SOXQ", &vec![candle; 251]);
        let readiness = strategy.history_readiness("SOXQ").unwrap();
        assert_eq!(readiness.required_bars, 252);
        assert_eq!(readiness.available_bars, 251);
        assert!(!readiness.ready);
        assert!(matches!(strategy.on_tick("SOXQ", 300, 0), Signal::Hold));
        let mut candles = vec![candle; 252];
        candles[251].high = 300;
        strategy.initialize_ohlc("SOXQ", &candles);
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert_eq!(strategy.states["SOXQ"].high_52w, Some(300));
        assert_eq!(strategy.states["SOXQ"].prev_price, Some(100));
        assert!(matches!(
            strategy.on_tick("SOXQ", 301, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_ohlc("SOXQ", &candles);
        assert_eq!(strategy.states["SOXQ"].buy_price, Some(301));
    }

    #[test]
    fn strong_close_ready_is_distinct_from_condition_and_preserves_positions() {
        let mut strategy = StrongCloseStrategy::new(config(serde_json::json!({})));
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
        strategy.initialize_candles("SOXQ", &[(100, 99)]);
        assert!(strategy.states["SOXQ"].pending_buy);
        strategy.initialize_candles("SOXQ", &[(100, 90)]);
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(!strategy.states["SOXQ"].pending_buy);
        assert!(matches!(strategy.on_tick("SOXQ", 100, 0), Signal::Hold));
        strategy.sync_position("SOXQ", 1, 100);
        strategy.initialize_candles("SOXQ", &[(100, 99)]);
        assert!(strategy.states["SOXQ"].in_position);
        assert!(!strategy.states["SOXQ"].pending_buy);
        assert!(matches!(
            strategy.on_tick("SOXQ", 95, 0),
            Signal::Sell { .. }
        ));
        strategy.initialize_candles("SOXQ", &[]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    }

    #[test]
    fn volatility_requires_full_range_window_and_preserves_position() {
        let mut strategy = VolatilityExpansionStrategy::new(config(serde_json::json!({
            "lookback_days": 3, "expansion_factor": 2.0, "stop_loss_pct": 3.0,
        })));
        strategy.initialize_range_data("SOXQ", &[10, 20]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
        assert_eq!(strategy.states["SOXQ"].avg_range, None);
        strategy.initialize_range_data("SOXQ", &[10, 20, 30]);
        assert_eq!(strategy.states["SOXQ"].avg_range, Some(20.0));
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(strategy.on_tick("SOXQ", 100, 0), Signal::Hold));
        assert!(matches!(
            strategy.on_tick("SOXQ", 145, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_range_data("SOXQ", &[10, 20, 30]);
        assert_eq!(strategy.states["SOXQ"].entry_price, Some(145));
        assert!(matches!(
            strategy.on_tick("SOXQ", 100, 0),
            Signal::Sell { .. }
        ));
    }
}
