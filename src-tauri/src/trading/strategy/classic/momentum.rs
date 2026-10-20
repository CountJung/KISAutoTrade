//! Momentum observations advance independently of confirmed position feedback.
use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use super::super::{
    state::bounded_window_with_extra, HistoryReadiness, Signal, Strategy, StrategyConfig,
};

/// Level retains the native threshold test; crossing gates new entries only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MomentumEntryMode {
    #[default]
    Level,
    Crossing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MomentumParams {
    pub lookback_period: usize,
    pub threshold_pct: f64,
    #[serde(default)]
    pub entry_mode: MomentumEntryMode,
    /// Percentage points below threshold required to rearm a crossing entry.
    #[serde(default)]
    pub entry_hysteresis_pct: f64,
}

impl Default for MomentumParams {
    fn default() -> Self {
        Self {
            lookback_period: 20,
            threshold_pct: 5.0,
            entry_mode: MomentumEntryMode::Level,
            entry_hysteresis_pct: 0.0,
        }
    }
}

fn legacy_params(value: &serde_json::Value) -> MomentumParams {
    // A malformed optional entry field must not erase a custom legacy exit.
    let mut legacy = value.clone();
    if let Some(object) = legacy.as_object_mut() {
        object.remove("entry_mode");
        object.remove("entry_hysteresis_pct");
    }
    serde_json::from_value(legacy).unwrap_or_default()
}

pub fn validate_momentum_entry_params(id: &str, value: &serde_json::Value) -> Result<(), String> {
    if !id.starts_with("momentum") {
        return Ok(());
    }
    let mode = match value.get("entry_mode") {
        None => MomentumEntryMode::Level,
        Some(v) => match v.as_str() {
            Some("level") => MomentumEntryMode::Level,
            Some("crossing") => MomentumEntryMode::Crossing,
            _ => return Err("entry_mode는 level 또는 crossing이어야 합니다.".into()),
        },
    };
    let hysteresis = match value.get("entry_hysteresis_pct") {
        None => 0.0,
        Some(v) => v
            .as_f64()
            .filter(|n| n.is_finite() && (0.0..=100.0).contains(n))
            .ok_or_else(|| {
                "entry_hysteresis_pct는 0..100의 유한한 %p 값이어야 합니다.".to_string()
            })?,
    };
    if mode == MomentumEntryMode::Crossing && hysteresis > legacy_params(value).threshold_pct {
        return Err("crossing의 entry_hysteresis_pct는 threshold_pct 이하여야 합니다.".into());
    }
    Ok(())
}

#[derive(Default)]
pub(super) struct MomentumState {
    pub(super) prices: VecDeque<u64>,
    pub(super) owned_quantity: u64,
    previous_momentum: Option<f64>,
    armed: bool,
    retry_candidate: bool,
}

impl MomentumState {
    fn observe(&mut self, price: u64, p: &MomentumParams, create_candidate: bool) -> Option<f64> {
        let cap = bounded_window_with_extra(p.lookback_period, 1);
        self.prices.push_back(price);
        while self.prices.len() > cap {
            self.prices.pop_front();
        }
        let past = self.prices.front().copied().unwrap_or(0);
        if self.prices.len() < cap || past == 0 {
            self.previous_momentum = None;
            self.armed = false;
            self.retry_candidate = false;
            return None;
        }
        let current = (price as f64 - past as f64) / past as f64 * 100.0;
        if current < p.threshold_pct {
            // An unfilled crossing expires when its condition no longer holds.
            self.retry_candidate = false;
        }
        if current <= p.threshold_pct - p.entry_hysteresis_pct {
            self.armed = true;
        }
        if self.armed
            && self
                .previous_momentum
                .is_some_and(|previous| previous < p.threshold_pct)
            && current >= p.threshold_pct
        {
            // Capture a crossing once, retaining retries until a real fill or
            // threshold cancellation. Historical crossings seed disarmed state
            // without inventing a pending order or successful ownership.
            self.armed = false;
            self.retry_candidate = create_candidate && self.owned_quantity == 0;
        }
        self.previous_momentum = Some(current);
        Some(current)
    }
}

pub struct MomentumStrategy {
    config: StrategyConfig,
    params: MomentumParams,
    entry_params_valid: bool,
    pub(super) states: HashMap<String, MomentumState>,
}

impl MomentumStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let entry_params_valid = validate_momentum_entry_params("momentum", &config.params).is_ok();
        let mut params = legacy_params(&config.params);
        if entry_params_valid {
            params.entry_mode = if config
                .params
                .get("entry_mode")
                .and_then(serde_json::Value::as_str)
                == Some("crossing")
            {
                MomentumEntryMode::Crossing
            } else {
                MomentumEntryMode::Level
            };
            params.entry_hysteresis_pct = config
                .params
                .get("entry_hysteresis_pct")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
        }
        Self {
            config,
            params,
            entry_params_valid,
            states: HashMap::new(),
        }
    }
}

impl Strategy for MomentumStrategy {
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
        let owned_quantity = self.states.get(symbol).map_or(0, |s| s.owned_quantity);
        let mut state = MomentumState {
            owned_quantity,
            ..Default::default()
        };
        // Strict prior history seeds observations/arming, never orders or a
        // retry candidate. Process bounded windows even for oversized callers.
        for price in prices {
            state.observe(*price, &self.params, false);
        }
        self.states.insert(symbol.into(), state);
    }
    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }
    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars: bounded_window_with_extra(self.params.lookback_period, 1),
            available_bars: state.map_or(0, |s| s.prices.len()),
            ready: state.is_some_and(|s| s.previous_momentum.is_some()),
        })
    }
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        Some(
            self.states
                .get(symbol)
                .map_or(0, |s| s.prices.len())
                .saturating_add(1)
                >= bounded_window_with_extra(self.params.lookback_period, 1),
        )
    }
    fn on_live_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal {
        // No speculative ownership/filled anchor is written by on_tick. Only
        // observations advance, including while the manager suppresses pending.
        self.on_tick(symbol, price, volume)
    }
    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let state = self.states.entry(symbol.into()).or_default();
        let Some(momentum) = state.observe(price, &self.params, self.entry_params_valid) else {
            return Signal::Hold;
        };
        if state.owned_quantity > 0 {
            // Crossing and malformed optional entry fields never block exits.
            if momentum <= -self.params.threshold_pct {
                return Signal::Sell {
                    symbol: symbol.into(),
                    quantity: self.config.order_quantity,
                    reason: format!("하락 모멘텀 {:.1}%", momentum),
                };
            }
        } else if self.entry_params_valid
            && match self.params.entry_mode {
                MomentumEntryMode::Level => momentum >= self.params.threshold_pct,
                MomentumEntryMode::Crossing => {
                    state.retry_candidate && momentum >= self.params.threshold_pct
                }
            }
        {
            return Signal::Buy {
                symbol: symbol.into(),
                quantity: self.config.order_quantity,
                reason: format!("상승 모멘텀 +{:.1}%", momentum),
            };
        }
        Signal::Hold
    }
    fn sync_position(&mut self, symbol: &str, quantity: u64, _avg_price: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let state = self.states.entry(symbol.into()).or_default();
        if quantity > 0 {
            if state.owned_quantity == 0 || state.retry_candidate {
                state.armed = false;
            }
            state.retry_candidate = false;
        }
        // Equal quantity (especially zero after nextOpen enqueue) is neither a
        // successful fill nor a rejection; retain the unfilled entry candidate.
        state.owned_quantity = quantity;
    }
    fn reset(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod tests;
