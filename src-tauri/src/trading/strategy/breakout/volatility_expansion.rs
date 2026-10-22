//! Prior completed daily ranges; signal proposals never establish ownership.
use super::super::completed_daily::valid_ohlc;
use super::super::observation::{advance_owned_age, sync_owned_age};
use super::super::{
    CompletedDailyBar, HistoryReadiness, OhlcCandle, Signal, Strategy, StrategyConfig,
    VolatilityExpansionPreparation,
};
use std::collections::{HashMap, VecDeque};
mod context;
mod params;
mod protection;
pub use params::{validate_volatility_expansion_params, VolatilityExpansionParams};

pub(super) struct VolatilityExpansionState {
    pub(super) completed_ranges: VecDeque<u64>,
    pub(super) avg_range: Option<f64>,
    pub(super) history_bars: usize,
    pub(super) day_open: Option<u64>,
    pub(super) day_high: u64,
    pub(super) day_low: u64,
    pub(super) owned_quantity: u64,
    pub(super) avg_price: u64,
    held_observations: Option<u64>,
    entry_candidate: bool,
    order_pending: bool,
    observed_peak: Option<u64>,
    profit_armed: bool,
    verified: bool,
    seen_history: bool,
    market_day: Option<String>,
    diagnostics: context::Diagnostics,
}
impl Default for VolatilityExpansionState {
    fn default() -> Self {
        Self {
            completed_ranges: VecDeque::new(),
            avg_range: None,
            history_bars: 0,
            day_open: None,
            day_high: 0,
            day_low: u64::MAX,
            owned_quantity: 0,
            avg_price: 0,
            held_observations: None,
            entry_candidate: false,
            order_pending: false,
            observed_peak: None,
            profit_armed: false,
            verified: false,
            seen_history: false,
            market_day: None,
            diagnostics: context::Diagnostics::default(),
        }
    }
}
pub struct VolatilityExpansionStrategy {
    config: StrategyConfig,
    params: VolatilityExpansionParams,
    entry_params_valid: bool,
    pub(super) states: HashMap<String, VolatilityExpansionState>,
}
impl VolatilityExpansionStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        Self {
            params: params::parse(&config.params),
            entry_params_valid: validate_volatility_expansion_params(
                "volatility_expansion",
                &config.params,
            )
            .is_ok(),
            config,
            states: HashMap::new(),
        }
    }
}
impl Strategy for VolatilityExpansionStrategy {
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
        for s in replacement.states.values_mut() {
            s.refresh(replacement.params.lookback_days);
            s.diagnostics.reset_counters();
            if !s.order_pending && s.owned_quantity == 0 {
                s.entry_candidate = false;
            }
        }
        *self = replacement;
        true
    }
    fn initialize_range_data(&mut self, symbol: &str, ranges: &[u64]) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        // A late legacy warmup must not replace verified live-day context.
        if s.market_day.is_some() {
            return;
        }
        s.replace(ranges, self.params.lookback_days);
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
        s.replace_bars(bars, self.params.lookback_days);
    }
    fn required_completed_daily_bars(&self) -> Option<usize> {
        Some(self.params.lookback_days)
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
        s.replace_bars(bars, self.params.lookback_days);
        if s.verified
            && s.history_bars >= self.params.lookback_days
            && s.market_day.as_deref() != Some(market_day)
        {
            s.clear_day();
            s.market_day = Some(market_day.into());
        }
    }
    fn invalidate_completed_daily_history(&mut self, symbol: &str) {
        if let Some(s) = self.states.get_mut(symbol) {
            s.invalidate();
        }
    }
    fn completed_daily_history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        self.history_readiness(symbol)
    }
    fn history_readiness(&self, symbol: &str) -> Option<HistoryReadiness> {
        let s = self.states.get(symbol);
        Some(HistoryReadiness {
            required_bars: self.params.lookback_days,
            available_bars: s.map_or(0, |s| s.history_bars),
            ready: self.entry_params_valid
                && s.is_some_and(|s| s.verified && s.avg_range.is_some()),
        })
    }
    fn volatility_expansion_preparation(
        &self,
        symbol: &str,
    ) -> Option<VolatilityExpansionPreparation> {
        Some(self.states.get(symbol).map_or_else(
            || {
                VolatilityExpansionState::default()
                    .preparation(&self.params, self.entry_params_valid)
            },
            |s| s.preparation(&self.params, self.entry_params_valid),
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
    fn on_trading_day_start(&mut self, symbol: &str, open: u64) {
        if !self.config.targets_symbol(symbol) || open == 0 {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        s.clear_day();
        s.observe_price(open);
        s.diagnostics.cadence = "dailyCloseRange";
        s.diagnostics.live_unverified = false;
    }
    fn on_daily_close_tick(&mut self, symbol: &str, candle: &OhlcCandle, volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) || !valid_ohlc(candle) {
            return Signal::Hold;
        }
        let s = self.states.entry(symbol.into()).or_default();
        s.day_open = Some(candle.open);
        s.day_high = candle.high;
        s.day_low = candle.low;
        s.diagnostics.cadence = "dailyCloseRange";
        s.diagnostics.live_unverified = false;
        self.on_tick(symbol, candle.close, volume)
    }
    fn on_completed_candle(&mut self, symbol: &str, candle: &OhlcCandle) {
        if !self.config.targets_symbol(symbol) {
            return;
        }
        let s = self.states.entry(symbol.into()).or_default();
        if !valid_ohlc(candle) {
            s.invalidate();
            return;
        }
        s.seen_history = true;
        s.verified = true;
        s.completed_ranges.push_back(candle.high - candle.low);
        while s.completed_ranges.len() > 512 {
            s.completed_ranges.pop_front();
        }
        s.refresh(self.params.lookback_days);
    }
    fn on_live_tick(&mut self, symbol: &str, price: u64, volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) || price == 0 {
            return Signal::Hold;
        }
        let s = self.states.entry(symbol.into()).or_default();
        s.diagnostics.cadence = "observedLiveQuotes";
        // Only quotes after verified daily context establish the session-open proxy.
        let live_ready = s.market_day.is_some() && s.verified;
        s.diagnostics.live_unverified = !live_ready;
        self.on_tick(symbol, price, volume)
    }
    fn on_tick(&mut self, symbol: &str, price: u64, _volume: u64) -> Signal {
        if !self.config.enabled || !self.config.targets_symbol(symbol) || price == 0 {
            return Signal::Hold;
        }
        let s = self.states.entry(symbol.into()).or_default();
        if s.diagnostics.cadence == "unverified" {
            s.diagnostics.cadence = "observedLiveQuotes";
        }
        if !s.diagnostics.live_unverified {
            s.observe_price(price);
        }
        s.diagnostics.evaluated = s.diagnostics.evaluated.saturating_add(1);
        advance_owned_age(s.owned_quantity, &mut s.held_observations);
        if s.owned_quantity > 0 {
            s.diagnostics.owned = s.diagnostics.owned.saturating_add(1);
            s.observed_peak = Some(s.observed_peak.unwrap_or(price).max(price));
            if self.params.profit_activation_pct > 0.0
                && self.params.profit_trailing_pct > 0.0
                && s.avg_price > 0
                && price as f64
                    >= s.avg_price as f64 * (1.0 + self.params.profit_activation_pct / 100.0)
            {
                s.profit_armed = true;
            }
            let loss = if s.avg_price > 0 {
                (s.avg_price as f64 - price as f64) / s.avg_price as f64 * 100.0
            } else {
                f64::NEG_INFINITY
            };
            let reason = if loss >= self.params.stop_loss_pct {
                Some(format!(
                    "변동성 확장 손절: 현재가 {} (매수가 {} 대비 -{:.2}%)",
                    price, s.avg_price, loss
                ))
            } else if s.profit_armed
                && self.params.profit_trailing_pct > 0.0
                && s.observed_peak.is_some_and(|peak| {
                    price as f64 <= peak as f64 * (1.0 - self.params.profit_trailing_pct / 100.0)
                })
            {
                Some("변동성 확장 수익 보호 되밀림".into())
            } else if self.params.max_hold_observations > 0
                && s.held_observations
                    .is_none_or(|age| age >= self.params.max_hold_observations)
            {
                Some("변동성 확장 최대 보유 관측 제한".into())
            } else {
                None
            };
            return reason.map_or(Signal::Hold, |reason| Signal::Sell {
                symbol: symbol.into(),
                quantity: s.owned_quantity,
                reason,
            });
        }
        if s.accept(&self.params, self.entry_params_valid, price) {
            if !s.order_pending {
                s.entry_candidate = true;
            }
            return Signal::Buy {
                symbol: symbol.into(),
                quantity: self.config.order_quantity,
                reason: format!(
                    "변동성 확장 매수: 당일 변동폭 {}원 > 평균 {:.0}원 × {:.1} (상승 방향)",
                    s.day_high.saturating_sub(s.day_low),
                    s.avg_range.unwrap_or(0.0),
                    self.params.expansion_factor
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
            .ok_or("Cannot restore volatility expansion protection without target state")?;
        protection::restore(s, metadata)
    }
    fn reset(&mut self) {
        // Preserve prior completed ranges, as the legacy explicit reset did.
        for s in self.states.values_mut() {
            s.clear_day();
            s.owned_quantity = 0;
            s.avg_price = 0;
            s.held_observations = None;
            s.entry_candidate = false;
            s.order_pending = false;
            s.observed_peak = None;
            s.profit_armed = false;
            s.market_day = None;
            s.diagnostics = context::Diagnostics::default();
        }
    }
}
#[cfg(test)]
mod tests;
