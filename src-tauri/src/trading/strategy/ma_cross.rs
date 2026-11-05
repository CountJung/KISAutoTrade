use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::{
    state::bounded_window_with_extra, EntryFilterPreparation, HistoryReadiness, OhlcCandle, Signal,
    Strategy, StrategyConfig,
};

// ────────────────────────────────────────────────────────────────────
// 이동평균 교차 전략 (Golden Cross / Death Cross)
// ────────────────────────────────────────────────────────────────────

mod context;
mod params;
use context::EntryContext;
pub use params::{validate_ma_cross_entry_params, MaCrossParams};

/// 종목별 MA교차 상태
struct MaCrossState {
    prices: VecDeque<u64>,
    prev_short_ma: Option<f64>,
    prev_long_ma: Option<f64>,
    context: EntryContext,
}

pub struct MovingAverageCrossStrategy {
    config: StrategyConfig,
    params: MaCrossParams,
    entry_params_valid: bool,
    /// 종목코드 → 개별 상태 (다중 종목 지원)
    states: std::collections::HashMap<String, MaCrossState>,
}

impl MovingAverageCrossStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let (params, entry_params_valid) = MaCrossParams::parse(&config.params);
        Self {
            config,
            params,
            entry_params_valid,
            states: std::collections::HashMap::new(),
        }
    }

    fn moving_average(prices: &VecDeque<u64>, period: usize) -> Option<f64> {
        if period == 0 || prices.len() < period {
            return None;
        }
        let sum: u64 = prices.iter().rev().take(period).sum();
        Some(sum as f64 / period as f64)
    }
}

impl Strategy for MovingAverageCrossStrategy {
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

    fn reconfigure_preserving_position(&mut self, config: StrategyConfig) -> bool {
        if !self.config.same_identity(&config) {
            return false;
        }
        let mut replacement = Self::new(config);
        for (symbol, state) in &self.states {
            let prices: Vec<_> = state.prices.iter().copied().collect();
            replacement.initialize_historical(symbol, &prices);
        }
        for (symbol, old) in &mut self.states {
            if let Some(state) = replacement.states.get_mut(symbol) {
                state.context = std::mem::take(&mut old.context);
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
        let cap =
            bounded_window_with_extra(self.params.long_period.max(self.params.short_period), 1);
        let history = prices[prices.len().saturating_sub(cap)..]
            .iter()
            .copied()
            .collect();
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MaCrossState {
                prices: VecDeque::with_capacity(cap),
                prev_short_ma: None,
                prev_long_ma: None,
                context: EntryContext::default(),
            });
        state.context.invalidate();
        state.prices = history;
        state.prev_short_ma = Self::moving_average(&state.prices, self.params.short_period);
        state.prev_long_ma = Self::moving_average(&state.prices, self.params.long_period);
    }

    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        // Intraday seed affects native cross observations, never DAILY context.
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let context = self
            .states
            .get_mut(symbol)
            .map(|s| std::mem::take(&mut s.context));
        self.initialize_historical(symbol, prices);
        if let Some(context) = context {
            self.states.get_mut(symbol).unwrap().context = context;
        }
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
        let cap =
            bounded_window_with_extra(self.params.long_period.max(self.params.short_period), 1);
        self.states
            .entry(symbol.into())
            .or_insert_with(|| MaCrossState {
                prices: VecDeque::with_capacity(cap),
                prev_short_ma: None,
                prev_long_ma: None,
                context: EntryContext::default(),
            })
            .context
            .replace(candles);
    }
    fn invalidate_completed_daily_history(&mut self, symbol: &str) {
        if let Some(state) = self.states.get_mut(symbol) {
            state.context.invalidate();
        }
    }
    fn completed_daily_history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let context = self.states.get(symbol).map(|s| &s.context);
        Some(HistoryReadiness {
            required_bars: self.params.required_completed(),
            available_bars: context.map_or(0, EntryContext::available),
            ready: context.map_or_else(
                || {
                    EntryContext::default()
                        .preparation(&self.params, self.entry_params_valid)
                        .ready
                },
                |c| c.preparation(&self.params, self.entry_params_valid).ready,
            ),
        })
    }
    fn entry_filter_preparation(&self, symbol: &str) -> Option<EntryFilterPreparation> {
        if self.entry_params_valid && !self.params.enabled() {
            return None;
        }
        Some(self.states.get(symbol).map_or_else(
            || EntryContext::default().preparation(&self.params, self.entry_params_valid),
            |s| s.context.preparation(&self.params, self.entry_params_valid),
        ))
    }
    fn on_trading_day_start(&mut self, symbol: &str, _open: u64) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        if !self.states.contains_key(symbol) {
            self.apply_completed_daily_history(symbol, &[]);
        }
        self.states
            .get_mut(symbol)
            .unwrap()
            .context
            .awaiting_completion = true;
    }
    fn on_completed_candle(&mut self, symbol: &str, candle: &OhlcCandle) {
        if let Some(state) = self.states.get_mut(symbol) {
            state.context.complete(*candle);
        }
    }

    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let state = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars: self
                .params
                .short_period
                .max(self.params.long_period)
                .max(self.params.required_completed()),
            available_bars: state.map_or(0, |state| {
                state.prices.len().max(if self.params.enabled() {
                    state.context.available()
                } else {
                    0
                })
            }),
            ready: state
                .is_some_and(|state| state.prev_short_ma.is_some() && state.prev_long_ma.is_some())
                && self
                    .completed_daily_history_readiness(symbol)
                    .is_some_and(|r| r.ready),
        })
    }

    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled {
            return Signal::Hold;
        }
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }

        let cap =
            bounded_window_with_extra(self.params.long_period.max(self.params.short_period), 1);
        let state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| MaCrossState {
                prices: VecDeque::with_capacity(cap),
                prev_short_ma: None,
                prev_long_ma: None,
                context: EntryContext::default(),
            });

        state.prices.push_back(price);
        if state.prices.len() > cap {
            state.prices.pop_front();
        }

        let short_ma = Self::moving_average(&state.prices, self.params.short_period);
        let long_ma = Self::moving_average(&state.prices, self.params.long_period);

        let signal = match (state.prev_short_ma, state.prev_long_ma, short_ma, long_ma) {
            (Some(ps), Some(pl), Some(cs), Some(cl)) => {
                if ps <= pl && cs > cl {
                    if !state
                        .context
                        .accept_candidate(&self.params, self.entry_params_valid, price)
                    {
                        Signal::Hold
                    } else {
                        Signal::Buy {
                            symbol: symbol.to_string(),
                            quantity: self.config.order_quantity,
                            reason: format!("골든크로스 S{:.0} > L{:.0}", cs, cl),
                        }
                    }
                } else if ps >= pl && cs < cl {
                    Signal::Sell {
                        symbol: symbol.to_string(),
                        quantity: self.config.order_quantity,
                        reason: format!("데드크로스 S{:.0} < L{:.0}", cs, cl),
                    }
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        state.prev_short_ma = short_ma;
        state.prev_long_ma = long_ma;
        signal
    }

    fn reset(&mut self) {
        self.states.clear();
    }
}

// ────────────────────────────────────────────────────────────────────
// 전략 저장소 (런타임에 여러 전략을 관리)
// ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod history_tests {
    use super::*;

    fn config() -> StrategyConfig {
        StrategyConfig::new(
            "ma",
            "MA",
            true,
            vec!["SOXQ".into()],
            1,
            serde_json::json!({ "short_period": 2, "long_period": 3 }),
        )
    }

    #[test]
    fn initializer_restores_previous_averages_for_first_evaluation_cross() {
        let mut strategy = MovingAverageCrossStrategy::new(config());
        strategy.initialize_historical("SOXQ", &[100, 90, 80]);
        let state = &strategy.states["SOXQ"];
        assert_eq!(state.prev_short_ma, Some(85.0));
        assert_eq!(state.prev_long_ma, Some(90.0));
        assert!(strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(
            strategy.on_tick("SOXQ", 120, 0),
            Signal::Buy { .. }
        ));
    }

    #[test]
    fn insufficient_history_has_no_previous_long_average() {
        let mut strategy = MovingAverageCrossStrategy::new(config());
        strategy.initialize_historical("SOXQ", &[100, 90]);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(strategy.on_tick("SOXQ", 120, 0), Signal::Hold));
        strategy.initialize_intraday_prices("SOXQ", &[10, 11, 12]);
        assert_eq!(strategy.states["SOXQ"].prices.front(), Some(&10));
    }
}

#[cfg(test)]
mod reconfigure_tests;

#[cfg(test)]
mod entry_tests;
