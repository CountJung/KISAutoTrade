use super::*;

impl LeveragedTrendHoldStrategy {
    pub(super) fn update_target_tick(&mut self, symbol: &str, price: u64) -> bool {
        let cap = self.window_cap();
        let rebound_cap = self.rebound_price_cap();
        let state = self.states.entry(symbol.to_string()).or_insert_with(|| {
            LeveragedTrendHoldMarketState {
                intraday_candles: VecDeque::with_capacity(cap),
                rebound_prices: VecDeque::with_capacity(rebound_cap),
                live_candle_minute: None,
            }
        });

        let live_minute = Self::current_live_minute_key();
        let same_minute = state.live_candle_minute == Some(live_minute);
        let mut new_observation = false;

        if !same_minute || state.intraday_candles.back().is_none() {
            state.intraday_candles.push_back(OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            });
            state.live_candle_minute = Some(live_minute);
            state.rebound_prices.push_back(price);
            new_observation = true;
        } else if let Some(last) = state.intraday_candles.back_mut() {
            last.high = last.high.max(price);
            last.low = last.low.min(price);
            last.close = price;
            if let Some(rebound_last) = state.rebound_prices.back_mut() {
                *rebound_last = price;
            } else {
                state.rebound_prices.push_back(price);
            }
        }

        while state.intraday_candles.len() > cap {
            state.intraday_candles.pop_front();
        }
        while state.rebound_prices.len() > rebound_cap {
            state.rebound_prices.pop_front();
        }

        let candle = *state.intraday_candles.back().expect("tick inserted candle");
        self.update_bollinger_candle(symbol, candle, new_observation);
        new_observation
    }

    pub(super) fn closes(candles: &VecDeque<OhlcCandle>) -> Vec<f64> {
        candles.iter().map(|c| c.close as f64).collect()
    }

    pub(super) fn ema(values: &[f64], period: usize) -> Option<f64> {
        if values.len() < period || period == 0 {
            return None;
        }
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = values[0];
        for value in &values[1..] {
            ema = value * alpha + ema * (1.0 - alpha);
        }
        Some(ema)
    }

    pub(super) fn rsi(values: &[f64], period: usize) -> Option<f64> {
        if values.len() < period + 1 || period == 0 {
            return None;
        }
        let start = values.len() - period - 1;
        let mut gains = 0.0;
        let mut losses = 0.0;
        for pair in values[start..].windows(2) {
            let diff = pair[1] - pair[0];
            if diff >= 0.0 {
                gains += diff;
            } else {
                losses += -diff;
            }
        }
        if losses == 0.0 {
            return Some(100.0);
        }
        let rs = (gains / period as f64) / (losses / period as f64);
        Some(100.0 - 100.0 / (1.0 + rs))
    }

    pub(super) fn adx(candles: &VecDeque<OhlcCandle>, period: usize) -> Option<f64> {
        if candles.len() < period + 1 || period == 0 {
            return None;
        }
        let start = candles.len() - period - 1;
        let slice: Vec<OhlcCandle> = candles.iter().skip(start).copied().collect();
        let mut tr_sum = 0.0;
        let mut plus_dm_sum = 0.0;
        let mut minus_dm_sum = 0.0;

        for pair in slice.windows(2) {
            let prev = pair[0];
            let cur = pair[1];
            let high_diff = cur.high as f64 - prev.high as f64;
            let low_diff = prev.low as f64 - cur.low as f64;
            let plus_dm = if high_diff > low_diff && high_diff > 0.0 {
                high_diff
            } else {
                0.0
            };
            let minus_dm = if low_diff > high_diff && low_diff > 0.0 {
                low_diff
            } else {
                0.0
            };
            let high_low = cur.high.saturating_sub(cur.low) as f64;
            let high_close = (cur.high as f64 - prev.close as f64).abs();
            let low_close = (cur.low as f64 - prev.close as f64).abs();
            tr_sum += high_low.max(high_close).max(low_close);
            plus_dm_sum += plus_dm;
            minus_dm_sum += minus_dm;
        }

        if tr_sum == 0.0 {
            return Some(0.0);
        }
        let plus_di = 100.0 * plus_dm_sum / tr_sum;
        let minus_di = 100.0 * minus_dm_sum / tr_sum;
        let denom = plus_di + minus_di;
        if denom == 0.0 {
            return Some(0.0);
        }
        Some(100.0 * (plus_di - minus_di).abs() / denom)
    }

    pub(super) fn bullish_count(candles: &VecDeque<OhlcCandle>, count: usize) -> usize {
        candles
            .iter()
            .rev()
            .take(count)
            .filter(|c| c.close > c.open)
            .count()
    }

    pub(super) fn gap_pct(candles: &VecDeque<OhlcCandle>) -> Option<f64> {
        if candles.len() < 2 {
            return None;
        }
        let cur = candles.back()?;
        let prev = candles.iter().rev().nth(1)?;
        if prev.close == 0 {
            return None;
        }
        Some((cur.open as f64 - prev.close as f64).abs() / prev.close as f64 * 100.0)
    }

    pub(super) fn snapshot_for(&self, symbol: &str) -> Option<LeveragedTrendSnapshot> {
        let state = self.states.get(symbol)?;
        let closes = Self::closes(&state.intraday_candles);
        let ema_short = Self::ema(&closes, self.params.ema_short_period)?;
        let ema_long = Self::ema(&closes, self.params.ema_long_period)?;
        let rsi = Self::rsi(&closes, self.params.rsi_period)?;
        let adx = Self::adx(&state.intraday_candles, self.params.adx_period)?;
        Some(LeveragedTrendSnapshot {
            ema_short,
            ema_long,
            rsi,
            adx,
            bullish_count_3: Self::bullish_count(&state.intraday_candles, 3),
        })
    }

    pub(super) fn upward_entry_rsi_min(&self) -> f64 {
        let sensitivity = self.params.upward_sensitivity.clamp(1.0, 5.0);
        (self.params.entry_rsi_min - (sensitivity - 1.0) * 2.0).clamp(45.0, 70.0)
    }

    pub(super) fn entry_ok(&self, symbol: &str) -> Option<LeveragedTrendSnapshot> {
        if !self.bollinger_entry_allowed(symbol) {
            return None;
        }
        let state = self.states.get(symbol)?;
        let snap = self.snapshot_for(symbol)?;
        let close = state.intraday_candles.back()?.close as f64;
        let gap_ok = Self::gap_pct(&state.intraday_candles)
            .map(|g| g <= self.params.max_gap_pct)
            .unwrap_or(true);
        if !gap_ok || snap.adx < self.params.no_trade_adx_below {
            return None;
        }

        let trend_ok = close > snap.ema_short
            && snap.ema_short > snap.ema_long
            && snap.rsi >= self.upward_entry_rsi_min()
            && snap.bullish_count_3 >= 2;
        if trend_ok && snap.adx >= self.params.entry_adx_min {
            Some(snap)
        } else {
            None
        }
    }

    pub(super) fn rebound_entry_ok(&self, symbol: &str) -> Option<LeveragedReboundSnapshot> {
        if !self.params.intraday_rebound_enabled || !self.bollinger_entry_allowed(symbol) {
            return None;
        }
        let state = self.states.get(symbol)?;
        let baseline_len = self.params.rebound_baseline_ticks.clamp(2, 120);
        let confirm_len = self.params.rebound_confirm_ticks.clamp(2, 60);
        let required_len = baseline_len.saturating_add(confirm_len);
        if state.rebound_prices.len() < required_len {
            return None;
        }

        let prices: Vec<u64> = state.rebound_prices.iter().copied().collect();
        let start = prices.len().saturating_sub(required_len);
        let window = &prices[start..];
        let (baseline, confirm) = window.split_at(baseline_len);
        let baseline_high = *baseline.iter().max()?;
        let baseline_low = *baseline.iter().min()?;
        let baseline_last = *baseline.last()?;
        let confirm_first = *confirm.first()?;
        let confirm_last = *confirm.last()?;
        if baseline_high == 0 || baseline_low == 0 || confirm_first == 0 {
            return None;
        }

        let pullback_pct =
            (baseline_high.saturating_sub(baseline_low) as f64 / baseline_high as f64) * 100.0;
        let buy_pressure_pct =
            (confirm_last.saturating_sub(confirm_first) as f64 / confirm_first as f64) * 100.0;
        let rebound_from_low_pct =
            (confirm_last.saturating_sub(baseline_low) as f64 / baseline_low as f64) * 100.0;

        let closes = Self::closes(&state.intraday_candles);
        let rsi = Self::rsi(&closes, self.params.rsi_period);
        if rsi
            .map(|value| value < self.params.rebound_rsi_min)
            .unwrap_or(false)
        {
            return None;
        }
        let adx = Self::adx(&state.intraday_candles, self.params.adx_period);

        if pullback_pct >= self.params.rebound_pullback_pct
            && buy_pressure_pct >= self.params.rebound_buy_pressure_pct
            && confirm_last > baseline_last
        {
            Some(LeveragedReboundSnapshot {
                rsi,
                adx,
                pullback_pct,
                buy_pressure_pct,
                rebound_from_low_pct,
            })
        } else {
            None
        }
    }

    pub(super) fn rapid_rebound_entry_ok(
        &self,
        symbol: &str,
    ) -> Option<LeveragedRapidReboundSnapshot> {
        if !self.params.rapid_rebound_enabled || !self.bollinger_entry_allowed(symbol) {
            return None;
        }
        let state = self.states.get(symbol)?;
        let lookback = self.params.rapid_rebound_lookback_ticks.clamp(3, 120);
        if state.rebound_prices.len() < lookback {
            return None;
        }

        let prices: Vec<u64> = state.rebound_prices.iter().copied().collect();
        let start = prices.len().saturating_sub(lookback);
        let window = &prices[start..];
        let current = *window.last()?;
        if current == 0 {
            return None;
        }

        let (low_idx, &low) = window.iter().enumerate().min_by_key(|(_, price)| **price)?;
        if low_idx == 0 || low == 0 || low_idx + 1 >= window.len() {
            return None;
        }
        let prior_high = *window[..low_idx].iter().max()?;
        let previous = window[window.len().saturating_sub(2)];
        if prior_high == 0 || previous == 0 || current <= previous {
            return None;
        }

        let low_age_ticks = window.len().saturating_sub(1).saturating_sub(low_idx);
        let max_low_age = self.params.rapid_rebound_max_low_age_ticks.clamp(1, 30);
        if low_age_ticks == 0 || low_age_ticks > max_low_age {
            return None;
        }

        let drop_pct = (prior_high.saturating_sub(low) as f64 / prior_high as f64) * 100.0;
        let recovery_pct = (current.saturating_sub(low) as f64 / low as f64) * 100.0;
        if drop_pct < self.params.rapid_rebound_drop_pct
            || recovery_pct < self.params.rapid_rebound_recovery_pct
        {
            return None;
        }

        let closes = Self::closes(&state.intraday_candles);
        Some(LeveragedRapidReboundSnapshot {
            rsi: Self::rsi(&closes, self.params.rsi_period),
            adx: Self::adx(&state.intraday_candles, self.params.adx_period),
            drop_pct,
            recovery_pct,
            low_age_ticks,
        })
    }
}
