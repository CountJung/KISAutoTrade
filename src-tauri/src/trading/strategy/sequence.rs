use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{
    state::{bounded_window, bounded_window_with_extra},
    HistoryReadiness, Signal, Strategy, StrategyConfig,
};

mod consecutive;
pub use consecutive::{
    validate_consecutive_entry_params, ConsecutiveMoveParams, ConsecutiveMoveStrategy,
};

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
    candidate_prev_high: Option<u64>,
    order_pending: bool,
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
    fn has_held_position(&self, symbol: &str) -> Option<bool> {
        Some(
            self.states
                .get(symbol)
                .is_some_and(|state| state.in_position),
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
        let mut replacement = Self::new(config);
        replacement.states = std::mem::take(&mut self.states);
        replacement
            .states
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        let histories: Vec<_> = replacement
            .states
            .iter()
            .map(|(symbol, state)| {
                (
                    symbol.clone(),
                    state.prices.iter().copied().collect::<Vec<_>>(),
                )
            })
            .collect();
        for (symbol, prices) in histories {
            replacement.initialize_historical(&symbol, &prices);
        }
        *self = replacement;
        true
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
                candidate_prev_high: None,
                order_pending: false,
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

    fn on_live_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal {
        let confirmed = self
            .states
            .get(symbol)
            .map(|state| (state.in_position, state.breakout_prev_high))
            .unwrap_or((false, None));
        let signal = self.on_tick(symbol, price, volume);
        if let Some(state) = self.states.get_mut(symbol) {
            if matches!(signal, Signal::Buy { .. }) && state.candidate_prev_high.is_none() {
                state.candidate_prev_high = state.breakout_prev_high;
            }
            state.in_position = confirmed.0;
            state.breakout_prev_high = confirmed.1;
        }
        signal
    }

    fn owned_position_metadata(&self, symbol: &str) -> Option<serde_json::Value> {
        let state = self.states.get(symbol)?;
        if state.breakout_prev_high.is_none() && state.candidate_prev_high.is_none() {
            return None;
        }
        Some(serde_json::json!({
            "version": 1,
            "kind": "failedBreakout",
            "confirmedAnchor": state.breakout_prev_high,
            "candidateAnchor": state.candidate_prev_high,
        }))
    }

    fn restore_owned_position_metadata(
        &mut self,
        symbol: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Protection {
            version: u64,
            kind: String,
            confirmed_anchor: Option<u64>,
            candidate_anchor: Option<u64>,
        }
        let protection: Protection = serde_json::from_value(metadata.clone())
            .map_err(|error| format!("Invalid breakout protection checkpoint: {error}"))?;
        if protection.version != 1
            || protection.kind != "failedBreakout"
            || protection.confirmed_anchor == Some(0)
            || protection.candidate_anchor == Some(0)
            || (protection.confirmed_anchor.is_none() && protection.candidate_anchor.is_none())
        {
            return Err("Invalid breakout protection checkpoint identity or anchor".into());
        }
        let state = self
            .states
            .get_mut(symbol)
            .ok_or("Cannot restore breakout protection without a target position")?;
        if state.in_position {
            state.breakout_prev_high = protection.confirmed_anchor.or(protection.candidate_anchor);
            state.candidate_prev_high = None;
        } else {
            if protection.confirmed_anchor.is_some() {
                return Err("Confirmed breakout anchor requires owned quantity".into());
            }
            state.candidate_prev_high = protection.candidate_anchor;
        }
        Ok(())
    }

    fn sync_owned_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        let was_held = self
            .states
            .get(symbol)
            .is_some_and(|state| state.in_position);
        self.sync_position(symbol, quantity, avg_price);
        if let Some(state) = self.states.get_mut(symbol) {
            if quantity > 0 && !was_held {
                state.breakout_prev_high = state.candidate_prev_high.take();
            } else if quantity == 0 {
                state.candidate_prev_high = None;
            }
        }
    }

    fn set_order_pending(&mut self, symbol: &str, pending: bool) {
        if let Some(state) = self.states.get_mut(symbol) {
            state.order_pending = pending;
            if !pending && !state.in_position {
                state.candidate_prev_high = None;
            }
        }
    }

    fn discard_unsubmitted_signal(&mut self, symbol: &str) {
        if let Some(state) = self.states.get_mut(symbol) {
            if !state.order_pending {
                state.candidate_prev_high = None;
            }
        }
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
                candidate_prev_high: None,
                order_pending: false,
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
                candidate_prev_high: None,
                order_pending: false,
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

#[cfg(test)]
mod reconfigure_tests;

#[cfg(test)]
#[path = "sequence/live_feedback_tests.rs"]
mod live_feedback_tests;
