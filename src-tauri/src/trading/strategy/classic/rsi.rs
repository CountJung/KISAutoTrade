//! Simple RSI crossings with confirmed ownership and independent optional protections.
use super::super::observation::{
    advance_owned_age, begin_live_trend, decode_owned_observation, encode_owned_observation,
    observe_trend, sync_owned_age, trend_allows,
};
use super::super::state::bounded_window_with_extra;
use super::super::{HistoryReadiness, Signal, Strategy, StrategyConfig};
use std::collections::{HashMap, VecDeque};
mod params;
pub use params::{validate_rsi_params, RsiParams};

#[derive(Default)]
pub(crate) struct RsiState {
    pub(crate) prices: VecDeque<u64>,
    pub(crate) previous_rsi: Option<f64>,
    owned_quantity: u64,
    avg_price: u64,
    held_observations: Option<u64>,
    entry_candidate: bool,
    live_started: bool,
    trend_prices: VecDeque<u64>,
    trend_observations: u64,
}

pub struct RsiStrategy {
    pub(super) config: StrategyConfig,
    pub(super) params: RsiParams,
    entry_params_valid: bool,
    pub(crate) states: HashMap<String, RsiState>,
}
impl RsiStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params = params::parse(&config.params);
        let entry_params_valid = validate_rsi_params("rsi", &config.params).is_ok();
        Self {
            config,
            params,
            entry_params_valid,
            states: HashMap::new(),
        }
    }

    /// Preserve legacy arithmetic: simple means of the latest period changes, not Wilder.
    fn calc_rsi(prices: &VecDeque<u64>, period: usize) -> Option<f64> {
        if period == 0 || prices.len() < period.saturating_add(1) {
            return None;
        }
        let recent: Vec<u64> = prices.iter().rev().take(period + 1).copied().collect();
        let (mut gain_sum, mut loss_sum) = (0.0f64, 0.0f64);
        for i in 0..period {
            let diff = recent[i] as f64 - recent[i + 1] as f64;
            if diff > 0.0 {
                gain_sum += diff;
            } else {
                loss_sum += -diff;
            }
        }
        let avg_gain = gain_sum / period as f64;
        let avg_loss = loss_sum / period as f64;
        if avg_loss == 0.0 {
            return Some(100.0);
        }
        let rs = avg_gain / avg_loss;
        Some(100.0 - 100.0 / (1.0 + rs))
    }
}
impl Strategy for RsiStrategy {
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
    fn has_held_position(&self, symbol: &str) -> Option<bool> {
        Some(
            self.states
                .get(symbol)
                .is_some_and(|s| s.owned_quantity > 0),
        )
    }
    fn reconfigure_preserving_position(&mut self, config: StrategyConfig) -> bool {
        if !self.config.same_identity(&config)
            || self
                .states
                .iter()
                .any(|(symbol, s)| s.owned_quantity > 0 && !config.targets_symbol(symbol))
        {
            return false;
        }
        let mut replacement = Self::new(config);
        replacement.states = std::mem::take(&mut self.states);
        replacement
            .states
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        for state in replacement.states.values_mut() {
            while state.prices.len() > bounded_window_with_extra(replacement.params.period, 2) {
                state.prices.pop_front();
            }
            // Recompute only when the RSI calculation period changes. Ownership/pending age origins stay intact.
            if self.params.period != replacement.params.period {
                state.previous_rsi = Self::calc_rsi(&state.prices, replacement.params.period);
            }
        }
        *self = replacement;
        true
    }
    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self.states.entry(symbol.into()).or_default();
        let cap = bounded_window_with_extra(self.params.period, 2);
        state.prices = prices[prices.len().saturating_sub(cap)..]
            .iter()
            .copied()
            .collect();
        state.previous_rsi = Self::calc_rsi(&state.prices, self.params.period);
        if !state.live_started {
            state.trend_prices.clear();
            state.trend_observations = 0;
            for price in prices {
                observe_trend(
                    &mut state.trend_prices,
                    &mut state.trend_observations,
                    *price,
                );
            }
        }
    }
    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }
    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let state = self.states.get(symbol);
        let rsi_required = self.params.period.saturating_add(1);
        let required_bars = rsi_required.max(self.params.long_trend_period.saturating_add(1));
        Some(HistoryReadiness {
            required_bars,
            available_bars: state.map_or(0, |s| {
                if s.prices.len() < rsi_required || self.params.long_trend_period == 0 {
                    s.prices.len()
                } else {
                    (s.trend_observations as usize).min(required_bars)
                }
            }),
            ready: state.is_some_and(|s| {
                s.previous_rsi.is_some()
                    && (self.params.long_trend_period == 0
                        || s.trend_observations > self.params.long_trend_period as u64)
            }),
        })
    }
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        Some(self.states.get(symbol).is_some_and(|s| {
            if s.owned_quantity > 0
                && (self.params.stop_loss_pct > 0.0 || self.params.max_hold_observations > 0)
            {
                return true;
            }
            s.previous_rsi.is_some()
                && s.prices.len().saturating_add(1) >= self.params.period.saturating_add(1)
                && (s.owned_quantity > 0 || s.trend_prices.len() >= self.params.long_trend_period)
        }))
    }
    fn on_live_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let state = self.states.entry(symbol.into()).or_default();
        begin_live_trend(
            &mut state.live_started,
            &mut state.trend_prices,
            &mut state.trend_observations,
        );
        self.on_tick(symbol, price, volume)
    }
    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let s = self.states.entry(symbol.into()).or_default();
        advance_owned_age(s.owned_quantity, &mut s.held_observations);
        let trend_ok = trend_allows(&s.trend_prices, self.params.long_trend_period, price);
        observe_trend(&mut s.trend_prices, &mut s.trend_observations, price);
        s.prices.push_back(price);
        while s.prices.len() > bounded_window_with_extra(self.params.period, 2) {
            s.prices.pop_front();
        }
        let rsi = Self::calc_rsi(&s.prices, self.params.period);
        let buy_cross = matches!((s.previous_rsi, rsi), (Some(prev), Some(cur)) if prev <= self.params.oversold && cur > self.params.oversold);
        let sell_cross = !buy_cross
            && matches!((s.previous_rsi, rsi), (Some(prev), Some(cur)) if prev >= self.params.overbought && cur < self.params.overbought);
        // Every observation consumes the original crossing, even after a filter or guard refusal.
        s.previous_rsi = rsi;
        if s.owned_quantity > 0 {
            let reason = if self.params.stop_loss_pct > 0.0
                && s.avg_price > 0
                && price as f64 * 100.0 <= s.avg_price as f64 * (100.0 - self.params.stop_loss_pct)
            {
                Some(format!("RSI 손절 {:.1}%", self.params.stop_loss_pct))
            } else if self.params.max_hold_observations > 0
                && s.held_observations
                    .is_none_or(|n| n >= self.params.max_hold_observations)
            {
                Some(match s.held_observations {
                    Some(n) => format!("RSI 보유 관측 한도 {n}회"),
                    None => "RSI 보유 관측 나이 불명: 보호 매도 후보".into(),
                })
            } else if sell_cross {
                Some(format!("RSI 과매수 하락 {:.1}", rsi.unwrap_or_default()))
            } else {
                None
            };
            if let Some(reason) = reason {
                return Signal::Sell {
                    symbol: symbol.into(),
                    quantity: s.owned_quantity,
                    reason,
                };
            }
        } else if self.entry_params_valid && trend_ok && buy_cross {
            s.entry_candidate = true;
            return Signal::Buy {
                symbol: symbol.into(),
                quantity: self.config.order_quantity,
                reason: format!("RSI 과매도 반등 {:.1}", rsi.unwrap_or_default()),
            };
        }
        Signal::Hold
    }
    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        sync_owned_age(
            s.owned_quantity,
            quantity,
            &mut s.held_observations,
            &mut s.entry_candidate,
        );
        s.owned_quantity = quantity;
        s.avg_price = if quantity > 0 { avg_price } else { 0 };
    }
    fn owned_position_metadata(&self, symbol: &str) -> Option<serde_json::Value> {
        let s = self.states.get(symbol)?;
        encode_owned_observation(
            "rsi",
            s.owned_quantity,
            s.held_observations,
            s.entry_candidate,
        )
    }
    fn restore_owned_position_metadata(
        &mut self,
        symbol: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        let s = self
            .states
            .get_mut(symbol)
            .ok_or("Cannot restore RSI protection without a target position")?;
        let (age, candidate) = decode_owned_observation("rsi", s.owned_quantity, metadata)?;
        if s.owned_quantity > 0 {
            s.held_observations = age;
        }
        s.entry_candidate = candidate;
        Ok(())
    }
    fn reset(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod tests;
