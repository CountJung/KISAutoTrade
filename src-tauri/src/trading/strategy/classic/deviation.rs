//! Confirmed ownership and protective age advance independently of observations.
use super::super::{HistoryReadiness, Signal, Strategy, StrategyConfig};
use std::collections::HashMap;
mod params;
mod state;
pub use params::{validate_deviation_entry_params, DeviationEntryMode, DeviationParams};
use state::{capacity, DeviationState};

pub struct DeviationStrategy {
    config: StrategyConfig,
    params: DeviationParams,
    entry_params_valid: bool,
    pub(crate) states: HashMap<String, DeviationState>,
}
impl DeviationStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params = params::parse(&config.params);
        let entry_params_valid =
            validate_deviation_entry_params("deviation", &config.params).is_ok();
        Self {
            config,
            params,
            entry_params_valid,
            states: HashMap::new(),
        }
    }
}
impl Strategy for DeviationStrategy {
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
        if !self.config.same_identity(&config) {
            return false;
        }
        let mut replacement = Self::new(config);
        let reseed = replacement.params != self.params
            || replacement.entry_params_valid != self.entry_params_valid;
        replacement.states = std::mem::take(&mut self.states);
        replacement
            .states
            .retain(|symbol, _| replacement.config.targets_symbol(symbol));
        if reseed {
            let histories = replacement
                .states
                .iter()
                .map(|(symbol, s)| (symbol.clone(), s.prices.iter().copied().collect::<Vec<_>>()))
                .collect::<Vec<_>>();
            for (symbol, prices) in histories {
                if let Some(state) = replacement.states.get_mut(&symbol) {
                    state.entry_candidate = false;
                }
                replacement.initialize_historical(&symbol, &prices);
            }
        }
        *self = replacement;
        true
    }
    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let mut state = self.states.remove(symbol).unwrap_or_default();
        state.prices.clear();
        state.previous_deviation = None;
        state.retry_candidate = false;
        // Preserve a signal-time age origin during pending-order warmup.
        // Reconfiguration explicitly clears it only when parameters change.
        let seed_trend = !state.live_started;
        if seed_trend {
            state.trend_prices.clear();
            state.trend_observations = 0;
        }
        for price in prices {
            state.observe(*price, &self.params, false, seed_trend);
        }
        self.states.insert(symbol.into(), state);
    }
    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }
    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required_bars = capacity(&self.params);
        let state = self.states.get(symbol);
        let ma_required = super::super::state::bounded_window(self.params.ma_period);
        let available_bars = state.map_or(0, |s| {
            if s.prices.len() < ma_required || self.params.long_trend_period == 0 {
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
                s.prices.len() >= ma_required
                    && s.previous_deviation.is_some()
                    && (self.params.long_trend_period == 0
                        || s.trend_observations > self.params.long_trend_period as u64)
            }),
        })
    }
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        let ma_required = super::super::state::bounded_window(self.params.ma_period);
        Some(self.states.get(symbol).map_or(
            ma_required == 1 && self.params.long_trend_period == 0,
            |s| {
                if s.owned_quantity > 0 {
                    self.params.stop_loss_pct > 0.0
                        || self.params.max_hold_observations > 0
                        || s.prices.len().saturating_add(1) >= ma_required
                } else {
                    s.prices.len().saturating_add(1) >= ma_required
                        && s.trend_prices.len() >= self.params.long_trend_period
                }
            },
        ))
    }
    fn on_live_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let state = self.states.entry(symbol.into()).or_default();
        if !state.live_started {
            // Daily/minute warmup is not a live-tick cadence history.
            state.live_started = true;
            state.trend_prices.clear();
            state.trend_observations = 0;
        }
        self.on_tick(symbol, price, volume)
    }
    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let state = self.states.entry(symbol.into()).or_default();
        if state.owned_quantity > 0 {
            state.held_observations = state.held_observations.map(|n| n.saturating_add(1));
        }
        let observation = state.observe(price, &self.params, self.entry_params_valid, true);
        if state.owned_quantity > 0 {
            let reason = if self.params.stop_loss_pct > 0.0
                && state.avg_price > 0
                && price as f64 * 100.0
                    <= state.avg_price as f64 * (100.0 - self.params.stop_loss_pct)
            {
                Some(format!("이격도 손절 {:.1}%", self.params.stop_loss_pct))
            } else if self.params.max_hold_observations > 0
                && state
                    .held_observations
                    .is_none_or(|n| n >= self.params.max_hold_observations)
            {
                Some(match state.held_observations {
                    Some(n) => format!("이격도 보유 관측 한도 {n}회"),
                    None => "이격도 보유 관측 나이 불명: 보호 매도 후보".into(),
                })
            } else {
                None
            };
            if let Some(reason) = reason {
                return Signal::Sell {
                    symbol: symbol.into(),
                    quantity: state.owned_quantity,
                    reason,
                };
            }
            if let Some((pct, ma, _)) = observation {
                if pct >= self.params.sell_threshold_pct {
                    return Signal::Sell {
                        symbol: symbol.into(),
                        quantity: self.config.order_quantity,
                        reason: format!("이격도 고점 +{:.1}% (MA {:.0}원)", pct, ma),
                    };
                }
            }
        } else if let Some((pct, ma, trend_ok)) = observation {
            if self.entry_params_valid
                && trend_ok
                && pct <= self.params.buy_threshold_pct
                && match self.params.entry_mode {
                    DeviationEntryMode::Level => true,
                    DeviationEntryMode::Crossing => state.retry_candidate,
                }
            {
                state.entry_candidate = true;
                return Signal::Buy {
                    symbol: symbol.into(),
                    quantity: self.config.order_quantity,
                    reason: format!("이격도 저점 {:.1}% (MA {:.0}원)", pct, ma),
                };
            }
        }
        Signal::Hold
    }
    fn sync_position(&mut self, symbol: &str, quantity: u64, avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self.states.entry(symbol.into()).or_default();
        if quantity > 0 {
            if state.owned_quantity == 0 {
                state.held_observations = state.entry_candidate.then_some(0);
            }
            state.retry_candidate = false;
            state.entry_candidate = false;
        } else if state.owned_quantity > 0 {
            state.held_observations = None;
            state.entry_candidate = false;
        }
        state.owned_quantity = quantity;
        state.avg_price = if quantity > 0 { avg_price } else { 0 };
    }
    fn owned_position_metadata(&self, symbol: &str) -> Option<serde_json::Value> {
        let s = self.states.get(symbol)?;
        if s.owned_quantity == 0 && !s.entry_candidate {
            return None;
        }
        Some(
            serde_json::json!({"version":1,"kind":"deviation","ageKnown":s.held_observations.is_some(),"heldObservations":s.held_observations.unwrap_or(0),"candidateEntry":s.entry_candidate}),
        )
    }
    fn restore_owned_position_metadata(
        &mut self,
        symbol: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Protection {
            version: u64,
            kind: String,
            age_known: bool,
            held_observations: u64,
            candidate_entry: bool,
        }
        let p: Protection = serde_json::from_value(metadata.clone())
            .map_err(|e| format!("Invalid deviation protection checkpoint: {e}"))?;
        if p.version != 1
            || p.kind != "deviation"
            || (!p.age_known && p.held_observations != 0)
            || (p.age_known && p.candidate_entry)
        {
            return Err("Invalid deviation protection checkpoint identity or age".into());
        }
        let s = self
            .states
            .get_mut(symbol)
            .ok_or("Cannot restore deviation protection without a target position")?;
        if s.owned_quantity > 0 {
            s.held_observations = if p.candidate_entry {
                Some(0)
            } else {
                p.age_known.then_some(p.held_observations)
            };
            s.entry_candidate = false;
        } else {
            if p.age_known || !p.candidate_entry {
                return Err("Confirmed deviation age requires owned quantity".into());
            }
            s.entry_candidate = true;
        }
        Ok(())
    }
    fn reset(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod tests;
