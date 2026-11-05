//! Entry-only diagnostics over verified prior completed DAILY candles.
use super::super::completed_daily::{atr14, mean_close, valid_ohlc};
use super::*;

#[derive(Default)]
pub(super) struct EntryContext {
    bars: VecDeque<OhlcCandle>,
    verified: bool,
    seen_ohlc: bool,
    pub(super) awaiting_completion: bool,
    counters: Counters,
}
#[derive(Default)]
struct Counters {
    candidates: u64,
    unready: u64,
    trend: u64,
    slope: u64,
    atr: u64,
    bandwidth: u64,
    passed: u64,
}
impl EntryContext {
    pub(super) fn reset_counters(&mut self) {
        self.counters = Counters::default();
        self.awaiting_completion = false;
        self.trim();
    }
    fn trim(&mut self) {
        while self.bars.len() > 512 {
            self.bars.pop_front();
        }
    }
    pub(super) fn available(&self) -> usize {
        self.bars.len()
    }
    pub(super) fn invalidate(&mut self) {
        self.bars.clear();
        self.verified = false;
        self.awaiting_completion = false;
    }
    pub(super) fn replace(&mut self, candles: &[OhlcCandle]) {
        self.invalidate();
        self.seen_ohlc = true;
        if !candles.iter().all(valid_ohlc) {
            return;
        }
        self.bars = candles[candles.len().saturating_sub(512)..]
            .iter()
            .copied()
            .collect();
        self.verified = true;
    }
    pub(super) fn complete(&mut self, candle: OhlcCandle) {
        if !self.awaiting_completion {
            return;
        }
        self.awaiting_completion = false;
        if !valid_ohlc(&candle) {
            self.invalidate();
            return;
        }
        self.seen_ohlc = true;
        self.verified = true;
        self.bars.push_back(candle);
        self.trim();
    }
    pub(super) fn preparation(&self, p: &MaCrossParams, valid: bool) -> EntryFilterPreparation {
        let required = p.required_completed();
        let status = if !valid {
            "invalidParams"
        } else if required > 0 && !self.seen_ohlc {
            "unsupportedObservation"
        } else if required > 0 && (!self.verified || self.bars.len() < required) {
            "insufficientHistory"
        } else {
            "ready"
        };
        EntryFilterPreparation {
            enabled: true,
            ready: status == "ready",
            status: status.into(),
            cadence: "priorCompletedDailyOhlc".into(),
            required_completed_bars: required,
            available_completed_bars: self.bars.len(),
            true_range_samples: self.bars.len().saturating_sub(1).min(14),
            candidates: self.counters.candidates,
            unready_candidates: self.counters.unready,
            min_move_rejections: 0,
            atr_rejections: self.counters.atr,
            trend_rejections: self.counters.trend,
            slope_rejections: Some(self.counters.slope),
            bandwidth_rejections: Some(self.counters.bandwidth),
            volume_rejections: None,
            breakout_basis: None,
            prior_volume_ratio: None,
            prior_atr: None,
            passed_candidates: self.counters.passed,
        }
    }
    fn slope(&self, period: usize) -> Option<f64> {
        if period == 0 || self.bars.len() < period.saturating_add(1) {
            return None;
        }
        let previous = self
            .bars
            .iter()
            .rev()
            .skip(1)
            .take(period)
            .map(|c| c.close as u128)
            .sum::<u128>() as f64
            / period as f64;
        mean_close(&self.bars, period).map(|current| (current / previous - 1.0) * 100.0)
    }
    fn bandwidth(&self) -> Option<f64> {
        let mean = mean_close(&self.bars, 20)?;
        let variance = self
            .bars
            .iter()
            .rev()
            .take(20)
            .map(|c| (c.close as f64 - mean).powi(2))
            .sum::<f64>()
            / 20.0;
        Some(4.0 * variance.sqrt() / mean * 100.0)
    }
    pub(super) fn accept_candidate(&mut self, p: &MaCrossParams, valid: bool, price: u64) -> bool {
        if valid && !p.enabled() {
            return true;
        }
        self.counters.candidates = self.counters.candidates.saturating_add(1);
        if !self.preparation(p, valid).ready {
            self.counters.unready = self.counters.unready.saturating_add(1);
            return false;
        }
        let trend = p.long_trend_period == 0
            || mean_close(&self.bars, p.long_trend_period).is_some_and(|ma| price as f64 > ma);
        let slope = p.min_long_ma_slope_pct == 0.0
            || self
                .slope(p.long_period)
                .is_some_and(|v| v >= p.min_long_ma_slope_pct);
        // Sufficient, validated history with zero ranges is ready; the positive threshold rejects it.
        let atr = p.min_atr_pct == 0.0
            || self.bars.back().is_some_and(|last| {
                atr14(&self.bars).unwrap_or(0.0) / last.close as f64 * 100.0 >= p.min_atr_pct
            });
        let bandwidth = p.min_bandwidth_pct == 0.0
            || self.bandwidth().is_some_and(|v| v >= p.min_bandwidth_pct);
        if !trend {
            self.counters.trend = self.counters.trend.saturating_add(1);
        }
        if !slope {
            self.counters.slope = self.counters.slope.saturating_add(1);
        }
        if !atr {
            self.counters.atr = self.counters.atr.saturating_add(1);
        }
        if !bandwidth {
            self.counters.bandwidth = self.counters.bandwidth.saturating_add(1);
        }
        if trend && slope && atr && bandwidth {
            self.counters.passed = self.counters.passed.saturating_add(1);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests;
