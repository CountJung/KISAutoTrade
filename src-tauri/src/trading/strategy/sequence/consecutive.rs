use super::super::{
    state::bounded_window_with_extra, EntryFilterPreparation, HistoryReadiness, OhlcCandle, Signal,
    Strategy, StrategyConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
mod context;
use context::{validate_entry_params, EntryContext};

pub fn validate_consecutive_entry_params(
    id: &str,
    params: &serde_json::Value,
) -> Result<(), String> {
    if id.starts_with("consecutive_move") {
        validate_entry_params(params)
    } else {
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────
// 연속 상승/하락 전략 (Consecutive Move)
// - N회 연속 가격 관측 상승 → 매수
// - M회 연속 가격 관측 하락 → 매도
// ────────────────────────────────────────────────────────────────────

/// 연속 상승/하락 전략 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsecutiveMoveParams {
    /// 매수 발동 연속 상승 횟수 (기본 3)
    pub buy_days: usize,
    /// 매도 발동 연속 하락 횟수 (기본 3)
    pub sell_days: usize,
    #[serde(default)]
    pub min_cumulative_move_pct: f64,
    #[serde(default)]
    pub min_move_atr_multiplier: f64,
    #[serde(default)]
    pub long_trend_period: usize,
}

impl Default for ConsecutiveMoveParams {
    fn default() -> Self {
        Self {
            buy_days: 3,
            sell_days: 3,
            min_cumulative_move_pct: 0.0,
            min_move_atr_multiplier: 0.0,
            long_trend_period: 0,
        }
    }
}

/// 종목별 연속상승/하락 상태
pub(super) struct ConsecutiveMoveState {
    pub(super) prices: VecDeque<u64>,
    pub(super) in_position: bool,
    context: EntryContext,
}

pub struct ConsecutiveMoveStrategy {
    config: StrategyConfig,
    params: ConsecutiveMoveParams,
    /// 종목코드 → 개별 상태
    pub(super) states: std::collections::HashMap<String, ConsecutiveMoveState>,
    entry_params_valid: bool,
}

impl ConsecutiveMoveStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let entry_params_valid = validate_entry_params(&config.params).is_ok();
        // Parse legacy fields independently: a malformed optional filter must not
        // silently disable itself or replace a custom held-position exit period.
        let mut legacy = config.params.clone();
        if let Some(object) = legacy.as_object_mut() {
            for key in [
                "min_cumulative_move_pct",
                "min_move_atr_multiplier",
                "long_trend_period",
            ] {
                object.remove(key);
            }
        }
        let mut params: ConsecutiveMoveParams = serde_json::from_value(legacy).unwrap_or_default();
        if entry_params_valid {
            params.min_cumulative_move_pct = config
                .params
                .get("min_cumulative_move_pct")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            params.min_move_atr_multiplier = config
                .params
                .get("min_move_atr_multiplier")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            params.long_trend_period = config
                .params
                .get("long_trend_period")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0) as usize;
        }
        Self {
            config,
            params,
            states: std::collections::HashMap::new(),
            entry_params_valid,
        }
    }

    fn entry_ready(&self, symbol: &str) -> bool {
        self.entry_params_valid
            && (self.params.required_completed() == 0
                || self
                    .states
                    .get(symbol)
                    .is_some_and(|s| s.context.preparation(&self.params, true).ready))
    }

    fn is_consecutive_up(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n.saturating_add(1) {
            return false;
        }
        let slice: Vec<u64> = prices
            .iter()
            .rev()
            .take(n.saturating_add(1))
            .cloned()
            .collect();
        (0..n).all(|i| slice[i] > slice[i + 1])
    }

    fn is_consecutive_down(prices: &VecDeque<u64>, n: usize) -> bool {
        if prices.len() < n.saturating_add(1) {
            return false;
        }
        let slice: Vec<u64> = prices
            .iter()
            .rev()
            .take(n.saturating_add(1))
            .cloned()
            .collect();
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
        for (symbol, _) in histories {
            if let Some(state) = replacement.states.get_mut(&symbol) {
                let cap = bounded_window_with_extra(
                    replacement
                        .params
                        .buy_days
                        .max(replacement.params.sell_days),
                    1,
                );
                while state.prices.len() > cap {
                    state.prices.pop_front();
                }
                state.context.trim(replacement.params.required_completed());
                state.context.awaiting_completion = false;
                state.context.reset_counters();
            }
        }
        *self = replacement;
        true
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
                context: EntryContext::default(),
            });
        state.context.invalidate();
        state.prices = prices[prices.len().saturating_sub(cap)..]
            .iter()
            .copied()
            .collect();
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }

    fn initialize_ohlc(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        self.initialize_historical(symbol, &candles.iter().map(|c| c.close).collect::<Vec<_>>());
        self.apply_completed_daily_history(symbol, candles);
    }

    fn required_completed_daily_bars(&self) -> Option<usize> {
        (self.entry_params_valid && self.params.required_completed() > 0)
            .then(|| self.params.required_completed())
    }

    fn legacy_warmup_requires_daily_verification(&self) -> bool {
        self.params.required_completed() > 0
    }

    fn apply_completed_daily_history(&mut self, symbol: &str, candles: &[OhlcCandle]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let cap = bounded_window_with_extra(self.params.buy_days.max(self.params.sell_days), 1);
        self.states
            .entry(symbol.into())
            .or_insert_with(|| ConsecutiveMoveState {
                prices: VecDeque::with_capacity(cap),
                in_position: false,
                context: EntryContext::default(),
            })
            .context
            .replace(candles, self.params.required_completed());
    }

    fn invalidate_completed_daily_history(&mut self, symbol: &str) {
        if let Some(state) = self.states.get_mut(symbol) {
            state.context.invalidate();
        }
    }

    fn completed_daily_history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        Some(HistoryReadiness {
            required_bars: self.params.required_completed(),
            available_bars: self.states.get(symbol).map_or(0, |s| s.context.available()),
            ready: self.entry_ready(symbol),
        })
    }

    fn on_trading_day_start(&mut self, symbol: &str, _open: u64) {
        if let Some(state) = self.states.get_mut(symbol) {
            state.context.awaiting_completion = true;
        } else if self.config.targets_symbol(symbol) {
            self.sync_position(symbol, 0, 0);
            self.states
                .get_mut(symbol)
                .unwrap()
                .context
                .awaiting_completion = true;
        }
    }

    fn on_completed_candle(&mut self, symbol: &str, candle: &OhlcCandle) {
        if let Some(state) = self.states.get_mut(symbol) {
            state
                .context
                .complete(*candle, self.params.required_completed());
        }
    }

    fn entry_filter_preparation(&self, symbol: &str) -> Option<EntryFilterPreparation> {
        if !self.params.enabled() && self.entry_params_valid {
            return None;
        }
        Some(
            self.states
                .get(symbol)
                .map(|state| {
                    state
                        .context
                        .preparation(&self.params, self.entry_params_valid)
                })
                .unwrap_or_else(|| {
                    EntryContext::default().preparation(&self.params, self.entry_params_valid)
                }),
        )
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars =
            bounded_window_with_extra(self.params.buy_days.max(self.params.sell_days), 1);
        let state = self.states.get(symbol);
        let available_bars = state.map_or(0, |state| state.prices.len());
        Some(HistoryReadiness {
            required_bars: required_bars.max(self.params.required_completed()),
            available_bars: if self.params.required_completed() > 0 {
                available_bars.max(state.map_or(0, |state| state.context.available()))
            } else {
                available_bars
            },
            ready: available_bars >= required_bars && self.entry_ready(symbol),
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
        Some(
            available_bars >= required_moves
                && (state.is_some_and(|state| state.in_position) || self.entry_ready(symbol)),
        )
    }

    fn on_live_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal {
        let confirmed = self
            .states
            .get(symbol)
            .map(|state| state.in_position)
            .unwrap_or(false);
        let signal = self.on_tick(symbol, price, volume);
        if let Some(state) = self.states.get_mut(symbol) {
            state.in_position = confirmed;
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

        let cap = bounded_window_with_extra(self.params.buy_days.max(self.params.sell_days), 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| ConsecutiveMoveState {
                prices: VecDeque::with_capacity(cap),
                in_position: false,
                context: EntryContext::default(),
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
                reason: format!("{}회 연속 가격 하락 → 매도", self.params.sell_days),
            };
        }

        if !state.in_position && Self::is_consecutive_up(&state.prices, self.params.buy_days) {
            if !state.context.accept_candidate(
                &self.params,
                self.entry_params_valid,
                &state.prices,
                price,
            ) {
                return Signal::Hold;
            }
            state.in_position = true;
            return Signal::Buy {
                symbol: symbol.to_string(),
                quantity: self.config.order_quantity,
                reason: format!("{}회 연속 가격 상승 → 매수", self.params.buy_days),
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
                context: EntryContext::default(),
            })
            .in_position = quantity > 0;
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod tests;
