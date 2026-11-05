//! Prior completed high/close entry with confirmed owner quantity and protections.
use super::super::observation::{advance_owned_age, sync_owned_age};
use super::super::{
    CompletedDailyBar, EntryFilterPreparation, HistoryReadiness, OhlcCandle, Signal, Strategy,
    StrategyConfig,
};
use std::collections::{HashMap, VecDeque};
mod context;
mod params;
mod protection;
use context::EntryContext;
pub use params::{validate_strong_close_params, StrongCloseParams};
#[derive(Default)]
pub(super) struct StrongCloseState {
    pub(super) pending_buy: bool,
    history_bars: usize,
    pub(super) owned_quantity: u64,
    pub(super) avg_price: u64,
    held_observations: Option<u64>,
    entry_candidate: bool,
    order_pending: bool,
    observed_peak: Option<u64>,
    profit_armed: bool,
    context: EntryContext,
    market_day: Option<String>,
}
impl StrongCloseState {
    fn seed(&mut self, pair: Option<(u64, u64)>, threshold: f64) {
        self.history_bars = usize::from(pair.is_some_and(|(h, c)| h > 0 && c > 0 && c <= h));
        self.pending_buy = self.owned_quantity == 0
            && self.history_bars == 1
            && pair.is_some_and(|(h, c)| (h as f64 - c as f64) / h as f64 * 100.0 <= threshold);
    }
}
pub struct StrongCloseStrategy {
    config: StrategyConfig,
    params: StrongCloseParams,
    entry_params_valid: bool,
    pub(super) states: HashMap<String, StrongCloseState>,
}
impl StrongCloseStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        let params = params::parse(&config.params);
        let entry_params_valid =
            validate_strong_close_params("strong_close", &config.params).is_ok();
        Self {
            config,
            params,
            entry_params_valid,
            states: HashMap::new(),
        }
    }
}
impl Strategy for StrongCloseStrategy {
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
            s.pending_buy = false;
            if !s.order_pending && s.owned_quantity == 0 {
                s.entry_candidate = false;
            }
            s.context.reset_counters();
        }
        *self = replacement;
        true
    }
    fn initialize_candles(&mut self, symbol: &str, candles: &[(u64, u64)]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        if s.market_day.is_some() {
            return;
        }
        s.seed(candles.last().copied(), self.params.threshold_pct);
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
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        if s.market_day.is_some() {
            return;
        }
        s.context.replace(bars);
        s.seed(
            bars.last()
                .filter(|_| s.context.verified)
                .map(|b| (b.ohlc.high, b.ohlc.close)),
            self.params.threshold_pct,
        );
    }
    fn required_completed_daily_bars(&self) -> Option<usize> {
        Some(self.params.long_trend_period.max(1))
    }
    fn legacy_warmup_requires_daily_verification(&self) -> bool {
        true
    }
    fn apply_completed_daily_history(&mut self, symbol: &str, bars: &[OhlcCandle]) {
        self.initialize_ohlc(symbol, bars);
    }
    fn apply_completed_daily_bars(&mut self, symbol: &str, bars: &[CompletedDailyBar]) {
        self.initialize_daily_bars(symbol, bars);
    }
    fn apply_completed_daily_bars_for_market_day(
        &mut self,
        symbol: &str,
        market_day: &str,
        bars: &[CompletedDailyBar],
    ) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        s.context.replace(bars);
        s.history_bars = usize::from(s.context.verified && !bars.is_empty());
        if s.market_day.as_deref() != Some(market_day) && s.history_bars > 0 {
            s.seed(
                bars.last().map(|b| (b.ohlc.high, b.ohlc.close)),
                self.params.threshold_pct,
            );
            s.market_day = Some(market_day.into());
        }
    }
    fn invalidate_completed_daily_history(&mut self, symbol: &str) {
        if let Some(s) = self.states.get_mut(symbol) {
            s.context.invalidate();
            s.history_bars = 0;
        }
    }
    fn completed_daily_history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        self.history_readiness(symbol)
    }
    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let required = self.params.long_trend_period.max(1);
        let s = self.states.get(symbol);
        let available = s.map_or(0, |s| {
            if self.params.long_trend_period > 0 {
                s.context.len()
            } else {
                s.history_bars
            }
        });
        let trend_ready = self.params.long_trend_period == 0
            || s.is_some_and(|s| s.context.verified && s.context.len() >= required);
        Some(HistoryReadiness {
            required_bars: required,
            available_bars: available,
            ready: self.entry_params_valid && s.is_some_and(|s| s.history_bars > 0) && trend_ready,
        })
    }
    fn entry_filter_preparation(&self, symbol: &str) -> Option<EntryFilterPreparation> {
        if self.entry_params_valid && self.params.long_trend_period == 0 {
            return None;
        }
        Some(self.states.get(symbol).map_or_else(
            || EntryContext::default().preparation(&self.params, self.entry_params_valid),
            |s| s.context.preparation(&self.params, self.entry_params_valid),
        ))
    }
    fn can_evaluate_next_tick(&self, symbol: &str) -> Option<bool> {
        if self
            .states
            .get(symbol)
            .is_some_and(|s| s.owned_quantity > 0)
        {
            Some(true)
        } else {
            self.history_readiness(symbol).map(|r| r.ready)
        }
    }
    fn on_completed_candle(&mut self, symbol: &str, candle: &OhlcCandle) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        s.context.complete(*candle);
        s.seed(
            s.context.verified.then_some((candle.high, candle.close)),
            self.params.threshold_pct,
        );
    }
    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.targets_symbol(symbol) {
            return Signal::Hold;
        }
        let s = self.states.entry(symbol.into()).or_default();
        advance_owned_age(s.owned_quantity, &mut s.held_observations);
        if s.owned_quantity > 0 {
            if price > 0 {
                s.observed_peak = Some(s.observed_peak.unwrap_or(price).max(price));
            }
            if self.params.profit_activation_pct > 0.0
                && self.params.profit_trailing_pct > 0.0
                && s.avg_price > 0
                && price as f64
                    >= s.avg_price as f64 * (1.0 + self.params.profit_activation_pct / 100.0)
            {
                s.profit_armed = true;
            }
        }
        if !self.config.enabled {
            return Signal::Hold;
        }
        if s.owned_quantity > 0 {
            let loss_pct = if s.avg_price > 0 {
                (s.avg_price as f64 - price as f64) / s.avg_price as f64 * 100.0
            } else {
                f64::NEG_INFINITY
            };
            let reason = if loss_pct >= self.params.stop_loss_pct {
                Some(format!(
                    "강한 종가 손절: 현재가 {} (매수가 {} 대비 -{:.2}%)",
                    price, s.avg_price, loss_pct
                ))
            } else if s.profit_armed
                && self.params.profit_trailing_pct > 0.0
                && s.observed_peak.is_some_and(|peak| {
                    price as f64 <= peak as f64 * (1.0 - self.params.profit_trailing_pct / 100.0)
                })
            {
                Some("강한 종가 수익 보호 되밀림".into())
            } else if self.params.max_hold_observations > 0
                && s.held_observations
                    .is_none_or(|age| age >= self.params.max_hold_observations)
            {
                Some("강한 종가 최대 보유 관측 제한".into())
            } else {
                None
            };
            return reason.map_or(Signal::Hold, |reason| Signal::Sell {
                symbol: symbol.into(),
                quantity: s.owned_quantity,
                reason,
            });
        }
        if s.pending_buy
            && s.history_bars > 0
            && s.context
                .accept(&self.params, self.entry_params_valid, price)
        {
            if !s.order_pending {
                s.entry_candidate = true;
            }
            return Signal::Buy {
                symbol: symbol.into(),
                quantity: self.config.order_quantity,
                reason: format!(
                    "강한 종가 후 매수: 현재가 {} (전일 종가가 고가 대비 {:.1}% 이내)",
                    price, self.params.threshold_pct
                ),
            };
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
        if quantity > 0 {
            s.pending_buy = false;
        }
        if (quantity > 0 && previous == 0) || (quantity == 0 && previous > 0) {
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
                s.entry_candidate = false;
            }
        }
    }
    fn discard_unsubmitted_signal(&mut self, symbol: &str) {
        if let Some(s) = self.states.get_mut(symbol) {
            if !s.order_pending && s.owned_quantity == 0 {
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
            .ok_or("Cannot restore strong close protection without target state")?;
        protection::restore(s, metadata)
    }
    fn reset(&mut self) {
        self.states.clear();
    }
}
#[cfg(test)]
mod tests;
