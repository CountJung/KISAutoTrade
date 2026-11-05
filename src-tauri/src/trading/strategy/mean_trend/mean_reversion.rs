//! Current-inclusive Bollinger bands with confirmed ownership and optional exits.
use super::super::{
    observation::{
        advance_owned_age, begin_live_trend, decode_owned_observation, encode_owned_observation,
        sync_owned_age,
    },
    state::bounded_window,
    HistoryReadiness, Signal, Strategy, StrategyConfig,
};
use std::collections::{HashMap, VecDeque};
mod params;
mod state;
pub use params::{validate_mean_reversion_params, MeanReversionExitMode, MeanReversionParams};
use state::MeanReversionState;

pub struct MeanReversionStrategy {
    config: StrategyConfig,
    params: MeanReversionParams,
    entry_params_valid: bool,
    pub(super) states: HashMap<String, MeanReversionState>,
}

impl MeanReversionStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params = params::parse(&config.params);
        let entry_params_valid =
            validate_mean_reversion_params("mean_reversion", &config.params).is_ok();
        Self {
            config,
            params,
            entry_params_valid,
            states: HashMap::new(),
        }
    }

    pub(super) fn bollinger_bands(
        prices: &VecDeque<u64>,
        period: usize,
        std_dev_mult: f64,
    ) -> Option<(f64, f64, f64)> {
        if prices.len() < period {
            return None;
        }
        let slice: Vec<f64> = prices
            .iter()
            .rev()
            .take(period)
            .map(|&p| p as f64)
            .collect();
        let mean = slice.iter().sum::<f64>() / period as f64;
        let variance = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let std = variance.sqrt();
        Some((mean, mean + std_dev_mult * std, mean - std_dev_mult * std))
    }
}

impl Strategy for MeanReversionStrategy {
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
                .is_some_and(|s| s.owned_quantity > 0),
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
        let changed = replacement.params != self.params
            || replacement.entry_params_valid != self.entry_params_valid;
        replacement.states = std::mem::take(&mut self.states);
        replacement
            .states
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        if changed {
            let period = bounded_window(replacement.params.period as usize);
            for state in replacement.states.values_mut() {
                // Band reseeding never rewinds confirmed ownership, age or live cadence.
                while state.prices.len() > period {
                    state.prices.pop_front();
                }
                state.entry_candidate = false;
            }
        }
        *self = replacement;
        true
    }
    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let period = bounded_window(self.params.period as usize);
        let state = self.states.entry(symbol.into()).or_default();
        state.prices = prices.iter().rev().take(period).rev().copied().collect();
        if !state.live_started {
            state.trend_prices.clear();
            state.trend_observations = 0;
            for price in prices {
                state.observe_trend(*price);
            }
        }
    }
    // Keep the legacy intraday-price warmup hook as the trait's no-op.
    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let period = bounded_window(self.params.period as usize);
        let required_bars = period.max(self.params.long_trend_period.saturating_add(1));
        let state = self.states.get(symbol);
        let available_bars = state.map_or(0, |s| {
            if s.prices.len() < period || self.params.long_trend_period == 0 {
                s.prices.len()
            } else if s.trend_observations <= self.params.long_trend_period as u64 {
                s.trend_observations as usize
            } else {
                required_bars
            }
        });
        Some(HistoryReadiness {
            required_bars,
            available_bars,
            ready: state.is_some_and(|s| {
                s.prices.len() >= period
                    && (self.params.long_trend_period == 0
                        || s.trend_observations > self.params.long_trend_period as u64)
            }),
        })
    }
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        let period = bounded_window(self.params.period as usize);
        Some(self.states.get(symbol).map_or(
            period == 1 && self.params.long_trend_period == 0,
            |s| {
                if s.owned_quantity > 0 {
                    s.entry_price.is_some()
                        || self.params.max_hold_observations > 0
                        || s.prices.len().saturating_add(1) >= period
                } else {
                    s.prices.len().saturating_add(1) >= period
                        && s.trend_ready(self.params.long_trend_period)
                }
            },
        ))
    }
    fn on_live_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let state = self.states.entry(symbol.into()).or_default();
        // Daily/minute seed is not an actual live-quote cadence history.
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
        let period = bounded_window(self.params.period as usize);
        let qty = self.config.order_quantity;
        let state = self.states.entry(symbol.into()).or_default();
        let trend_ok = state.trend_allows(price, self.params.long_trend_period);
        state.observe_trend(price);
        state.prices.push_back(price);
        while state.prices.len() > period + 1 {
            state.prices.pop_front();
        }
        if state.owned_quantity > 0 {
            advance_owned_age(state.owned_quantity, &mut state.held_observations);
            if let Some(ep) = state.entry_price {
                // Preserve the original expression, >= boundary, and configured quantity.
                let loss_pct = (ep as f64 - price as f64) / ep as f64 * 100.0;
                if loss_pct >= self.params.stop_loss_pct {
                    return Signal::Sell {
                        symbol: symbol.into(),
                        quantity: qty,
                        reason: format!(
                            "평균회귀 손절: 현재가 {} (매수가 {} 대비 -{:.2}%)",
                            price, ep, loss_pct
                        ),
                    };
                }
            }
            if self.params.max_hold_observations > 0
                && state
                    .held_observations
                    .is_none_or(|n| n >= self.params.max_hold_observations)
            {
                return Signal::Sell {
                    symbol: symbol.into(),
                    quantity: state.owned_quantity,
                    reason: match state.held_observations {
                        Some(n) => format!("평균회귀 보유 관측 한도 {n}회"),
                        None => "평균회귀 보유 관측 나이 불명: 보호 매도 후보".into(),
                    },
                };
            }
        }
        let (mean, upper, lower) =
            match Self::bollinger_bands(&state.prices, period, self.params.std_dev) {
                Some(b) => b,
                None => return Signal::Hold,
            };
        if state.owned_quantity > 0 {
            let (target, name) = match self.params.exit_mode {
                MeanReversionExitMode::UpperBand => (upper, "상단밴드"),
                MeanReversionExitMode::MiddleBand => (mean, "중심밴드"),
            };
            if price as f64 > target {
                return Signal::Sell {
                    symbol: symbol.into(),
                    quantity: qty,
                    reason: format!(
                        "평균회귀 익절: 현재가 {} > {} {:.0} (mean={:.0})",
                        price, name, target, mean
                    ),
                };
            }
        } else if self.entry_params_valid && trend_ok && (price as f64) < lower {
            state.entry_candidate = true;
            return Signal::Buy {
                symbol: symbol.into(),
                quantity: qty,
                reason: format!(
                    "평균회귀 매수: 현재가 {} < 하단밴드 {:.0} (mean={:.0})",
                    price, lower, mean
                ),
            };
        }
        Signal::Hold
    }
    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self.states.entry(symbol.into()).or_default();
        sync_owned_age(
            state.owned_quantity,
            quantity,
            &mut state.held_observations,
            &mut state.entry_candidate,
        );
        state.owned_quantity = quantity;
        state.entry_price = (quantity > 0 && avg_price > 0).then_some(avg_price);
    }
    fn owned_position_metadata(&self, symbol: &str) -> Option<serde_json::Value> {
        let s = self.states.get(symbol)?;
        encode_owned_observation(
            "mean_reversion",
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
            .ok_or("Cannot restore mean reversion protection without a target position")?;
        let (age, candidate) =
            decode_owned_observation("mean_reversion", s.owned_quantity, metadata)?;
        if s.owned_quantity > 0 {
            s.held_observations = age;
        }
        s.entry_candidate = candidate;
        Ok(())
    }
    fn reset(&mut self) {
        // Preserve the original price-buffer contract and independent live context.
        for state in self.states.values_mut() {
            state.owned_quantity = 0;
            state.entry_price = None;
            state.held_observations = None;
            state.entry_candidate = false;
        }
    }
}

#[cfg(test)]
mod tests;
