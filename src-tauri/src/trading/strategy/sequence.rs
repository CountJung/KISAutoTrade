use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{
    state::{bounded_window, bounded_window_with_extra},
    HistoryReadiness, Signal, Strategy, StrategyConfig,
};

// ────────────────────────────────────────────────────────────────────
// 연속 상승/하락 전략 (Consecutive Move)
// - N일 연속 종가 상승 → 매수
// - M일 연속 종가 하락 → 매도
// ────────────────────────────────────────────────────────────────────

/// 연속 상승/하락 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsecutiveMoveParams {
    /// 매수 발동 연속 상승 횟수 (기본 3)
    pub buy_days: usize,
    /// 매도 발동 연속 하락 횟수 (기본 3)
    pub sell_days: usize,
}

impl Default for ConsecutiveMoveParams {
    fn default() -> Self {
        Self {
            buy_days: 3,
            sell_days: 3,
        }
    }
}

/// 종목별 연속상승/하락 상태
struct ConsecutiveMoveState {
    prices: VecDeque<u64>,
    in_position: bool,
}

pub struct ConsecutiveMoveStrategy {
    config: StrategyConfig,
    params: ConsecutiveMoveParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, ConsecutiveMoveState>,
}

impl ConsecutiveMoveStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: ConsecutiveMoveParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }

    fn is_consecutive_up(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n + 1 {
            return false;
        }
        let slice: Vec<u64> = prices.iter().rev().take(n + 1).cloned().collect();
        (0..n).all(|i| slice[i] > slice[i + 1])
    }

    fn is_consecutive_down(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n + 1 {
            return false;
        }
        let slice: Vec<u64> = prices.iter().rev().take(n + 1).cloned().collect();
        (0..n).all(|i| slice[i] < slice[i + 1])
    }
}

impl Strategy for ConsecutiveMoveStrategy {
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

    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let cap = bounded_window_with_extra(self.params.buy_days.max(self.params.sell_days), 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| ConsecutiveMoveState {
                prices: VecDeque::with_capacity(cap),
                in_position: false,
            });
        state.prices = prices[prices.len().saturating_sub(cap)..]
            .iter()
            .copied()
            .collect();
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars =
            bounded_window_with_extra(self.params.buy_days.max(self.params.sell_days), 1);
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars,
            available_bars: state.map_or(0, |state| state.prices.len()),
            ready: state.is_some_and(|state| state.prices.len() >= required_bars),
        })
    }

    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        let state = self.states.get(symbol);
        let required_moves = if state.is_some_and(|state| state.in_position) {
            self.params.sell_days
        } else {
            self.params.buy_days
        };
        let available_bars = state.map_or(0, |state| state.prices.len());
        // N회 연속 움직임은 이전 N가격과 현재 평가 가격으로 판정한다.
        Some(available_bars >= required_moves)
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let cap = bounded_window_with_extra(self.params.buy_days.max(self.params.sell_days), 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| ConsecutiveMoveState {
                prices: VecDeque::with_capacity(cap),
                in_position: false,
            });

        state.prices.push_back(price);
        if state.prices.len() > cap {
            state.prices.pop_front();
        }

        if state.in_position && Self::is_consecutive_down(&state.prices, self.params.sell_days) {
            state.in_position = false;
            return Signal::Sell {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("{}일 연속 하락 → 매도", self.params.sell_days),
            };
        }

        if !state.in_position && Self::is_consecutive_up(&state.prices, self.params.buy_days) {
            state.in_position = true;
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("{}일 연속 상승 → 매수", self.params.buy_days),
            };
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, _avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let cap = bounded_window_with_extra(self.params.buy_days.max(self.params.sell_days), 1);
        self.states
            .entry(symbol.to_string())
            .or_insert_with(|| ConsecutiveMoveState {
                prices: VecDeque::with_capacity(cap),
                in_position: false,
            })
            .in_position = quantity > 0;
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 06. 돌파 실패 전략 (FailedBreakoutStrategy)
// ────────────────────────────────────────────────────────────────────
// 동작:
//  1. 최근 lookback_days개 가격에서 전고점(prev_high) 계산
//  2. 현재가 ≥ prev_high × (1 + buffer_pct/100) → 전고점 돌파 → 매수
//  3. 매수 후 현재가 < 돌파 시점의 prev_high → 돌파 실패 → 매도
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedBreakoutParams {
    /// 전고점을 계산하기 위한 과거 기간 (기본 20)
    pub lookback_days: usize,
    /// 전고점 대비 돌파로 인정하는 버퍼 % (기본 0.5)
    pub buffer_pct: f64,
}

impl Default for FailedBreakoutParams {
    fn default() -> Self {
        Self {
            lookback_days: 20,
            buffer_pct: 0.5,
        }
    }
}

/// 종목별 돌파실패 상태
struct FailedBreakoutState {
    prices: VecDeque<u64>,
    in_position: bool,
    breakout_prev_high: Option<u64>,
}

pub struct FailedBreakoutStrategy {
    config: StrategyConfig,
    params: FailedBreakoutParams,
    /// 종목코드 → 개별 상태
    states: std::collections::HashMap<String, FailedBreakoutState>,
}

impl FailedBreakoutStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params: FailedBreakoutParams =
            serde_json::from_value(config.params.clone()).unwrap_or_default();
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
        }
    }
}

impl Strategy for FailedBreakoutStrategy {
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

    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let cap = bounded_window(self.params.lookback_days);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| FailedBreakoutState {
                prices: VecDeque::with_capacity(cap),
                in_position: false,
                breakout_prev_high: None,
            });
        state.prices = prices[prices.len().saturating_sub(cap)..]
            .iter()
            .copied()
            .collect();
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars = bounded_window(self.params.lookback_days);
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars,
            available_bars: state.map_or(0, |state| state.prices.len()),
            ready: state.is_some_and(|state| state.prices.len() >= required_bars),
        })
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let lookback = bounded_window(self.params.lookback_days);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| FailedBreakoutState {
                prices: VecDeque::with_capacity(lookback),
                in_position: false,
                breakout_prev_high: None,
            });

        let prev_high = state.prices.iter().copied().max().unwrap_or(0);

        // ① 매도 우선: 돌파 실패
        if state.in_position {
            if let Some(ref_high) = state.breakout_prev_high {
                if price < ref_high {
                    state.in_position = false;
                    state.breakout_prev_high = None;
                    state.prices.push_back(price);
                    if state.prices.len() > lookback {
                        state.prices.pop_front();
                    }
                    return Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("돌파 실패: 현재가 {} < 전고점 {} → 매도", price, ref_high),
                    };
                }
            }
        }

        // ② 매수: 전고점 돌파
        if !state.in_position && state.prices.len() >= lookback && prev_high > 0 {
            let breakout_threshold =
                (prev_high as f64 * (1.0 + self.params.buffer_pct / 100.0)) as u64;
            if price >= breakout_threshold {
                state.in_position = true;
                state.breakout_prev_high = Some(prev_high);
                state.prices.push_back(price);
                if state.prices.len() > lookback {
                    state.prices.pop_front();
                }
                return Signal::Buy {
                    symbol: symbol.to_string(),
                    quantity: self.config.order_quantity,
                    reason: format!(
                        "전고점 돌파 매수: {} ≥ {} (전고점 {} + {:.1}% 버퍼)",
                        price, breakout_threshold, prev_high, self.params.buffer_pct
                    ),
                };
            }
        }

        state.prices.push_back(price);
        if state.prices.len() > lookback {
            state.prices.pop_front();
        }

        Signal::Hold
    }

    fn sync_position(&mut self, symbol: &str, quantity: u64, _avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let lookback = bounded_window(self.params.lookback_days);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| FailedBreakoutState {
                prices: VecDeque::with_capacity(bounded_window_with_extra(lookback, 1)),
                in_position: false,
                breakout_prev_high: None,
            });
        state.in_position = quantity > 0;
        if quantity == 0 {
            state.breakout_prev_high = None;
        }
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;

    fn config(params: serde_json::Value) -> StrategyConfig {
        StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
    }

    #[test]
    fn consecutive_history_preserves_flat_and_existing_positions() {
        let mut strategy = ConsecutiveMoveStrategy::new(config(serde_json::json!({
            "buy_days": 3, "sell_days": 3,
        })));
        strategy.initialize_historical("SOXQ", &[100, 101, 102, 103]);
        assert!(!strategy.states["SOXQ"].in_position);
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(
            strategy.on_tick("SOXQ", 104, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_historical("SOXQ", &[104, 103, 102, 101]);
        assert!(strategy.states["SOXQ"].in_position);
        assert!(matches!(
            strategy.on_tick("SOXQ", 100, 0),
            Signal::Sell { .. }
        ));
        strategy.initialize_historical("SOXQ", &[100, 101]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    }

    #[test]
    fn failed_breakout_history_retains_actual_breakout_reference() {
        let mut strategy = FailedBreakoutStrategy::new(config(serde_json::json!({
            "lookback_days": 3, "buffer_pct": 0.5,
        })));
        strategy.initialize_historical("SOXQ", &[100, 100]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
        strategy.initialize_historical("SOXQ", &[100, 100, 100]);
        assert!(!strategy.states["SOXQ"].in_position);
        assert!(matches!(
            strategy.on_tick("SOXQ", 120, 0),
            Signal::Buy { .. }
        ));
        strategy.initialize_historical("SOXQ", &[110, 110, 110]);
        assert_eq!(strategy.states["SOXQ"].breakout_prev_high, Some(100));
        assert!(matches!(
            strategy.on_tick("SOXQ", 90, 0),
            Signal::Sell { .. }
        ));
        strategy.initialize_intraday_prices("SOXQ", &[20, 20, 20]);
        assert_eq!(strategy.states["SOXQ"].prices.front(), Some(&20));
    }
    #[test]
    fn consecutive_counts_current_price_and_uses_active_position_branch() {
        let mut strategy = ConsecutiveMoveStrategy::new(config(serde_json::json!({
            "buy_days": 2, "sell_days": 4,
        })));
        let mut evaluated = 0;
        for (index, price) in [100, 101, 102].into_iter().enumerate() {
            let can_evaluate = strategy.can_evaluate_next_tick("SOXQ").unwrap();
            assert_eq!(can_evaluate, index == 2);
            evaluated += usize::from(can_evaluate);
            let signal = strategy.on_tick("SOXQ", price, 0);
            assert_eq!(matches!(signal, Signal::Buy { .. }), index == 2);
        }
        assert_eq!(evaluated, 1);
        assert_eq!(strategy.can_evaluate_next_tick("SOXQ"), Some(false));
        assert!(matches!(strategy.on_tick("SOXQ", 101, 0), Signal::Hold));
        assert_eq!(strategy.can_evaluate_next_tick("SOXQ"), Some(true));
        strategy.initialize_historical("SOXQ", &[104, 103, 102, 101]);
        assert!(matches!(
            strategy.on_tick("SOXQ", 100, 0),
            Signal::Sell { .. }
        ));
    }

    #[test]
    fn failed_breakout_excludes_last_seed_tick_before_first_breakout_evaluation() {
        let mut strategy = FailedBreakoutStrategy::new(config(serde_json::json!({
            "lookback_days": 3, "buffer_pct": 0.5,
        })));
        let mut evaluated = 0;
        for (index, price) in [100, 101, 102, 110].into_iter().enumerate() {
            let can_evaluate = strategy.can_evaluate_next_tick("SOXQ").unwrap();
            assert_eq!(can_evaluate, index == 3);
            evaluated += usize::from(can_evaluate);
            let signal = strategy.on_tick("SOXQ", price, 0);
            assert_eq!(matches!(signal, Signal::Buy { .. }), index == 3);
        }
        assert_eq!(evaluated, 1);
    }
}
