use super::*;
pub(super) struct Diagnostics {
    pub(super) cadence: &'static str,
    pub(super) live_unverified: bool,
    pub(super) evaluated: u64,
    pub(super) owned: u64,
    candidates: u64,
    unready: u64,
    range_rejected: u64,
    direction_rejected: u64,
    passed: u64,
}
impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            cadence: "unverified",
            live_unverified: false,
            evaluated: 0,
            owned: 0,
            candidates: 0,
            unready: 0,
            range_rejected: 0,
            direction_rejected: 0,
            passed: 0,
        }
    }
}
impl Diagnostics {
    pub(super) fn reset_counters(&mut self) {
        self.evaluated = 0;
        self.owned = 0;
        self.candidates = 0;
        self.unready = 0;
        self.range_rejected = 0;
        self.direction_rejected = 0;
        self.passed = 0;
    }
}
impl VolatilityExpansionState {
    pub(super) fn clear_day(&mut self) {
        self.day_open = None;
        self.day_high = 0;
        self.day_low = u64::MAX;
    }
    pub(super) fn observe_price(&mut self, price: u64) {
        self.day_high = self.day_high.max(price);
        self.day_low = self.day_low.min(price);
        if self.day_open.is_none() {
            self.day_open = Some(price);
        }
    }
    pub(super) fn refresh(&mut self, lookback: usize) {
        self.history_bars = self.completed_ranges.len();
        self.avg_range = (self.history_bars >= lookback).then(|| {
            let sum = self
                .completed_ranges
                .iter()
                .rev()
                .take(lookback)
                .map(|v| *v as u128)
                .sum::<u128>();
            sum as f64 / lookback as f64
        });
    }
    pub(super) fn invalidate(&mut self) {
        self.completed_ranges.clear();
        self.avg_range = None;
        self.history_bars = 0;
        self.verified = false;
    }
    pub(super) fn replace(&mut self, ranges: &[u64], lookback: usize) {
        self.invalidate();
        self.seen_history = true;
        self.completed_ranges = ranges[ranges.len().saturating_sub(512)..]
            .iter()
            .copied()
            .collect();
        self.verified = true;
        self.refresh(lookback);
    }
    pub(super) fn replace_bars(&mut self, bars: &[CompletedDailyBar], lookback: usize) {
        if !bars.iter().all(|b| valid_ohlc(&b.ohlc)) {
            self.invalidate();
            self.seen_history = true;
            return;
        }
        let ranges = bars[bars.len().saturating_sub(512)..]
            .iter()
            .map(|b| b.ohlc.high - b.ohlc.low)
            .collect::<Vec<_>>();
        self.replace(&ranges, lookback);
    }
    pub(super) fn preparation(
        &self,
        p: &VolatilityExpansionParams,
        valid: bool,
    ) -> VolatilityExpansionPreparation {
        let status = if !valid {
            "invalidParams"
        } else if self.diagnostics.live_unverified
            || !self.seen_history
            || self.diagnostics.cadence == "unverified"
        {
            "unsupportedObservation"
        } else if !self.verified || self.avg_range.is_none() {
            "insufficientHistory"
        } else {
            "ready"
        };
        VolatilityExpansionPreparation {
            ready: status == "ready",
            status: status.into(),
            cadence: self.diagnostics.cadence.into(),
            required_completed_bars: p.lookback_days,
            available_completed_bars: self.history_bars,
            evaluated_observations: self.diagnostics.evaluated,
            owned_observations: self.diagnostics.owned,
            candidate_observations: self.diagnostics.candidates,
            unready_candidates: self.diagnostics.unready,
            range_rejections: self.diagnostics.range_rejected,
            direction_rejections: self.diagnostics.direction_rejected,
            passed_candidates: self.diagnostics.passed,
        }
    }
    pub(super) fn accept(
        &mut self,
        p: &VolatilityExpansionParams,
        valid: bool,
        price: u64,
    ) -> bool {
        self.diagnostics.candidates = self.diagnostics.candidates.saturating_add(1);
        if !self.preparation(p, valid).ready || self.day_open.is_none() || self.day_low == u64::MAX
        {
            self.diagnostics.unready = self.diagnostics.unready.saturating_add(1);
            return false;
        }
        if self.day_high.saturating_sub(self.day_low) as f64
            <= self.avg_range.unwrap_or(0.0) * p.expansion_factor
        {
            self.diagnostics.range_rejected = self.diagnostics.range_rejected.saturating_add(1);
            return false;
        }
        if price <= self.day_open.unwrap_or(price) {
            self.diagnostics.direction_rejected =
                self.diagnostics.direction_rejected.saturating_add(1);
            return false;
        }
        self.diagnostics.passed = self.diagnostics.passed.saturating_add(1);
        true
    }
}
