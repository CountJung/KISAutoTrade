mod volatility_expansion;
pub use volatility_expansion::{
    validate_volatility_expansion_params, VolatilityExpansionParams, VolatilityExpansionStrategy,
};
mod strong_close;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
pub use strong_close::{validate_strong_close_params, StrongCloseParams, StrongCloseStrategy};

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
#[derive(Default)]
struct FiftyTwoWeekState {
    prev_price: Option<u64>,
    high_52w: Option<u64>,
    /// Only completed daily highs belong in this bounded window.
    completed_highs: VecDeque<u64>,
    /// One completion may follow each explicit daily boundary; ticks never age history.
    awaiting_daily_completion: bool,
    buy_price: Option<u64>,
}

impl FiftyTwoWeekState {
    fn refresh_high(&mut self, required: usize) {
        self.high_52w = (self.completed_highs.len() == required)
            .then(|| self.completed_highs.iter().copied().max())
            .flatten()
            .filter(|high| *high > 0);
    }
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
    fn has_held_position(&self, symbol: &str) -> Option<bool> {
        Some(
            self.states
                .get(symbol)
                .is_some_and(|state| state.buy_price.is_some()),
        )
    }
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn reconfigure_preserving_position(&mut self, config: StrategyConfig) -> bool {
        if !self.config.same_identity(&config) {
            return false;
        }
        self.params = serde_json::from_value(config.params.clone()).unwrap_or_default();
        // A removed target follows the existing explicit target-removal policy.
        // Retained targets keep buy_price, while every price/history observation is
        // invalidated so the new parameters cannot trade against the old window.
        self.states
            .retain(|symbol, _| config.targets_symbol(symbol));
        for state in self.states.values_mut() {
            state.prev_price = None;
            state.high_52w = None;
            state.completed_highs.clear();
            state.awaiting_daily_completion = false;
        }
        self.config = config;
        true
    }

    // 기존 단일 가격 API는 고가 배열을 받는다. 완료된 과거 봉 전체만 사용한다.
    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let required = bounded_window(self.params.lookback_days);
        let history = &prices[prices.len().saturating_sub(required)..];
        let state = self.states.entry(symbol.to_string()).or_default();
        state.completed_highs = history.iter().copied().collect();
        state.refresh_high(required);
        state.prev_price = history.last().copied();
        state.awaiting_daily_completion = false;
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

    fn required_completed_daily_bars(&self) -> Option<usize> {
        Some(bounded_window(self.params.lookback_days))
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars: bounded_window(self.params.lookback_days),
            available_bars: state.map_or(0, |state| state.completed_highs.len()),
            ready: state
                .is_some_and(|state| state.high_52w.is_some() && state.prev_price.is_some()),
        })
    }

    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        Some(self.states.get(symbol).is_some_and(|state| {
            state.buy_price.is_some() || (state.high_52w.is_some() && state.prev_price.is_some())
        }))
    }

    fn on_live_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal {
        let confirmed = self
            .states
            .get(symbol)
            .map(|state| state.buy_price)
            .unwrap_or(None);
        let signal = self.on_tick(symbol, price, volume);
        if let Some(state) = self.states.get_mut(symbol) {
            state.buy_price = confirmed;
        }
        signal
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let state = self.states.entry(symbol.to_string()).or_default();

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
                Signal::Hold
            }
        };

        state.prev_price = Some(price);
        signal
    }

    fn on_trading_day_start(&mut self, symbol: &str, _open: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        self.states
            .entry(symbol.to_string())
            .or_default()
            .awaiting_daily_completion = true;
    }

    fn on_completed_candle(&mut self, symbol: &str, candle: &OhlcCandle) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let Some(state) = self.states.get_mut(symbol) else {
            return;
        };
        if !state.awaiting_daily_completion {
            return;
        }
        state.awaiting_daily_completion = false;
        let required = bounded_window(self.params.lookback_days);
        if state.completed_highs.len() == required {
            state.completed_highs.pop_front();
        }
        state.completed_highs.push_back(candle.high);
        state.refresh_high(required);
        state.prev_price = Some(candle.close);
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self.states.entry(symbol.to_string()).or_default();
        state.buy_price = (quantity > 0).then_some(avg_price);
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
#[cfg(test)]
#[path = "breakout/day_event_tests.rs"]
mod day_event_tests;

#[cfg(test)]
#[path = "breakout/high_tests.rs"]
mod high_tests;

#[cfg(test)]
mod history_tests {
    use super::*;

    fn config(params: serde_json::Value) -> StrategyConfig {
        StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
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
        assert!(strategy.states["SOXQ"].owned_quantity > 0);
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
        strategy.sync_position("SOXQ", 1, 145);
        strategy.initialize_range_data("SOXQ", &[10, 20, 30]);
        assert_eq!(strategy.states["SOXQ"].avg_price, 145);
        assert!(matches!(
            strategy.on_tick("SOXQ", 100, 0),
            Signal::Sell { .. }
        ));
    }
}

#[cfg(test)]
mod reconfigure_tests;

#[cfg(test)]
#[path = "breakout/live_feedback_tests.rs"]
mod live_feedback_tests;
