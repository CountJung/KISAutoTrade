//! Price breakout with confirmed ownership and separate completed DAILY entry context.
use super::super::observation::{advance_owned_age, sync_owned_age};
use super::super::state::bounded_window;
use super::super::{
    CompletedDailyBar, EntryFilterPreparation, HistoryReadiness, OhlcCandle, Signal, Strategy,
    StrategyConfig,
};
use std::collections::{HashMap, VecDeque};
mod context;
mod params;
mod protection;
use context::EntryContext;
pub use params::{validate_failed_breakout_params, FailedBreakoutBasis, FailedBreakoutParams};
#[derive(Default)]
pub(super) struct FailedBreakoutState {
    pub(super) prices: VecDeque<u64>,
    pub(super) owned_quantity: u64,
    avg_price: u64,
    pub(super) breakout_prev_high: Option<u64>,
    pub(super) candidate_prev_high: Option<u64>,
    order_pending: bool,
    held_observations: Option<u64>,
    entry_candidate: bool,
    observed_peak: Option<u64>,
    profit_armed: bool,
    context: EntryContext,
}
pub struct FailedBreakoutStrategy {
    config: StrategyConfig,
    params: FailedBreakoutParams,
    entry_params_valid: bool,
    pub(super) states: HashMap<String, FailedBreakoutState>,
}
impl FailedBreakoutStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params = params::parse(&config.params);
        let entry_params_valid =
            validate_failed_breakout_params("failed_breakout", &config.params).is_ok();
        Self {
            config,
            params,
            entry_params_valid,
            states: HashMap::new(),
        }
    }
    fn optional_enabled(&self) -> bool {
        self.params.required_completed() > 0
            || self.params.profit_activation_pct > 0.0
            || self.params.profit_trailing_pct > 0.0
            || self.params.max_hold_observations > 0
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
    fn set_enabled(&mut self, v: bool) {
        self.config.enabled = v;
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
        for s in replacement.states.values_mut() {
            while s.prices.len() > bounded_window(replacement.params.lookback_days) {
                s.prices.pop_front();
            }
            s.context.reset_counters();
        }
        *self = replacement;
        true
    }
    fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let cap = bounded_window(self.params.lookback_days);
        self.states.entry(symbol.into()).or_default().prices = prices
            [prices.len().saturating_sub(cap)..]
            .iter()
            .copied()
            .collect();
    }
    fn initialize_intraday_prices(&mut self, symbol: &str, prices: &[u64]) {
        self.initialize_historical(symbol, prices);
    }
    fn initialize_ohlc(&mut self, symbol: &str, bars: &[OhlcCandle]) {
        self.initialize_daily_bars(
            symbol,
            &bars
                .iter()
                .map(|ohlc| CompletedDailyBar {
                    ohlc: *ohlc,
                    volume: None,
                })
                .collect::<Vec<_>>(),
        );
    }
    fn initialize_daily_bars(&mut self, symbol: &str, bars: &[CompletedDailyBar]) {
        self.initialize_historical(
            symbol,
            &bars.iter().map(|b| b.ohlc.close).collect::<Vec<_>>(),
        );
        self.apply_completed_daily_bars(symbol, bars);
    }
    fn required_completed_daily_bars(&self) -> Option<usize> {
        (self.entry_params_valid && self.params.required_completed() > 0)
            .then(|| self.params.required_completed())
    }
    fn legacy_warmup_requires_daily_verification(&self) -> bool {
        self.params.required_completed() > 0
    }
    fn apply_completed_daily_history(&mut self, symbol: &str, bars: &[OhlcCandle]) {
        self.apply_completed_daily_bars(
            symbol,
            &bars
                .iter()
                .map(|ohlc| CompletedDailyBar {
                    ohlc: *ohlc,
                    volume: None,
                })
                .collect::<Vec<_>>(),
        );
    }
    fn apply_completed_daily_bars(&mut self, symbol: &str, bars: &[CompletedDailyBar]) {
        if self.config.targets_symbol(symbol) {
            self.states
                .entry(symbol.into())
                .or_default()
                .context
                .replace(bars);
        }
    }
    fn invalidate_completed_daily_history(&mut self, symbol: &str) {
        if let Some(s) = self.states.get_mut(symbol) {
            s.context.invalidate();
        }
    }
    fn completed_daily_history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let p = self.states.get(symbol).map_or_else(
            || EntryContext::default().preparation(&self.params, self.entry_params_valid),
            |s| s.context.preparation(&self.params, self.entry_params_valid),
        );
        Some(HistoryReadiness {
            required_bars: p.required_completed_bars,
            available_bars: p.available_completed_bars,
            ready: p.ready,
        })
    }
    fn entry_filter_preparation(&self, symbol: &str) -> Option<EntryFilterPreparation> {
        if self.entry_params_valid && !self.optional_enabled() {
            return None;
        }
        Some(self.states.get(symbol).map_or_else(
            || EntryContext::default().preparation(&self.params, self.entry_params_valid),
            |s| s.context.preparation(&self.params, self.entry_params_valid),
        ))
    }
    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let lookback = bounded_window(self.params.lookback_days);
        let s = self.states.get(symbol);
        let daily = self.completed_daily_history_readiness(symbol).unwrap();
        let high_basis = self.params.breakout_basis == FailedBreakoutBasis::CompletedHigh;
        let required_bars = lookback.max(daily.required_bars);
        let price_ready = high_basis || s.is_some_and(|s| s.prices.len() >= lookback);
        let available_bars = if !price_ready {
            s.map_or(0, |s| s.prices.len())
        } else if daily.required_bars > 0 {
            daily.available_bars.min(required_bars)
        } else {
            s.map_or(0, |s| s.prices.len())
        };
        Some(HistoryReadiness {
            required_bars,
            available_bars,
            ready: self.entry_params_valid && price_ready && daily.ready,
        })
    }
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        if self
            .states
            .get(symbol)
            .is_some_and(|s| s.owned_quantity > 0)
        {
            Some(true)
        } else {
            self.history_readiness(symbol).map(|s| s.ready)
        }
    }
    fn on_trading_day_start(&mut self, symbol: &str, _open: u64) {
        if self.config.targets_symbol(symbol) {
            self.states
                .entry(symbol.into())
                .or_default()
                .context
                .awaiting_completion = true;
        }
    }
    fn on_completed_candle(&mut self, symbol: &str, candle: &OhlcCandle) {
        self.on_completed_daily_bar(
            symbol,
            &CompletedDailyBar {
                ohlc: *candle,
                volume: None,
            },
        );
    }
    fn on_completed_daily_bar(&mut self, symbol: &str, bar: &CompletedDailyBar) {
        if let Some(s) = self.states.get_mut(symbol) {
            s.context.complete(*bar);
        }
    }
    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let s = self.states.entry(symbol.into()).or_default();
        let lookback = bounded_window(self.params.lookback_days);
        let prior = s.prices.iter().copied().max().unwrap_or(0);
        let ready = s.prices.len() >= lookback
            || (self.params.breakout_basis == FailedBreakoutBasis::CompletedHigh
                && s.context.high(lookback).is_some());
        // All observation state advances exactly once, without inferring an execution.
        advance_owned_age(s.owned_quantity, &mut s.held_observations);
        s.prices.push_back(price);
        while s.prices.len() > lookback {
            s.prices.pop_front();
        }
        if s.owned_quantity > 0 {
            s.observed_peak = Some(s.observed_peak.unwrap_or(price).max(price));
            if self.params.profit_activation_pct > 0.0
                && self.params.profit_trailing_pct > 0.0
                && s.avg_price > 0
                && price as f64
                    >= s.avg_price as f64 * (1.0 + self.params.profit_activation_pct / 100.0)
            {
                s.profit_armed = true;
            }
            let reason = if s.breakout_prev_high.is_some_and(|anchor| price < anchor) {
                Some(format!(
                    "돌파 실패: 현재가 {} < 전고점 {} → 매도",
                    price,
                    s.breakout_prev_high.unwrap()
                ))
            } else if self.params.profit_activation_pct > 0.0
                && self.params.profit_trailing_pct > 0.0
                && s.profit_armed
                && s.observed_peak.is_some_and(|peak| {
                    price as f64 <= peak as f64 * (1.0 - self.params.profit_trailing_pct / 100.0)
                })
            {
                Some(format!(
                    "돌파 수익 보호: 관측 고점 대비 {:.1}% 되밀림",
                    self.params.profit_trailing_pct
                ))
            } else if self.params.max_hold_observations > 0
                && s.held_observations
                    .is_none_or(|n| n >= self.params.max_hold_observations)
            {
                Some(match s.held_observations {
                    Some(n) => format!("돌파 보유 관측 한도 {n}회"),
                    None => "돌파 보유 관측 나이 불명: 보호 매도 후보".into(),
                })
            } else {
                None
            };
            return reason.map_or(Signal::Hold, |reason| Signal::Sell {
                symbol: symbol.into(),
                quantity: s.owned_quantity,
                reason,
            });
        }
        let anchor = match self.params.breakout_basis {
            FailedBreakoutBasis::PriceObservations => prior,
            FailedBreakoutBasis::CompletedHigh => s.context.high(lookback).unwrap_or(prior),
        };
        if ready && anchor > 0 {
            let threshold = (anchor as f64 * (1.0 + self.params.buffer_pct / 100.0)) as u64;
            if price >= threshold
                && s.context
                    .accept(&self.params, self.entry_params_valid, anchor, price)
            {
                // The first accepted pending order owns its original signal anchor.
                if s.candidate_prev_high.is_none() && !s.order_pending {
                    s.candidate_prev_high = Some(anchor);
                    s.entry_candidate = true;
                }
                return Signal::Buy {
                    symbol: symbol.into(),
                    quantity: self.config.order_quantity,
                    reason: format!(
                        "전고점 돌파 매수: {} ≥ {} (전고점 {} + {:.1}% 버퍼)",
                        price, threshold, anchor, self.params.buffer_pct
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
        let s = self.states.entry(symbol.into()).or_default();
        let previous = s.owned_quantity;
        sync_owned_age(
            previous,
            quantity,
            &mut s.held_observations,
            &mut s.entry_candidate,
        );
        if quantity > 0 && previous == 0 {
            s.breakout_prev_high = s.candidate_prev_high.take();
            s.observed_peak = None;
            s.profit_armed = false;
        } else if quantity == 0 && previous > 0 {
            s.breakout_prev_high = None;
            s.candidate_prev_high = None;
            s.observed_peak = None;
            s.profit_armed = false;
        }
        s.owned_quantity = quantity;
        s.avg_price = if quantity > 0 { avg_price } else { 0 };
    }
    fn set_order_pending(&mut self, symbol: &str, pending: bool) {
        if let Some(s) = self.states.get_mut(symbol) {
            s.order_pending = pending;
            if !pending && s.owned_quantity == 0 {
                s.candidate_prev_high = None;
                s.entry_candidate = false;
            }
        }
    }
    fn discard_unsubmitted_signal(&mut self, symbol: &str) {
        if let Some(s) = self.states.get_mut(symbol) {
            if !s.order_pending && s.owned_quantity == 0 {
                s.candidate_prev_high = None;
                s.entry_candidate = false;
            }
        }
    }
    fn owned_position_metadata(&self, symbol: &str) -> Option<serde_json::Value> {
        self.states.get(symbol).and_then(protection::encode)
    }
    fn restore_owned_position_metadata(
        &mut self,
        symbol: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        let s = self
            .states
            .get_mut(symbol)
            .ok_or("Cannot restore breakout protection without a target position")?;
        protection::restore(s, metadata)
    }
    fn reset(&mut self) {
        self.states.clear();
    }
}
#[cfg(test)]
mod tests;
