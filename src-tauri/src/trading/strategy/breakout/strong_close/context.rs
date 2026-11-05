use super::super::super::completed_daily::{mean_close, valid_ohlc};
use super::*;
#[derive(Default)]
pub(super) struct EntryContext {
    bars: VecDeque<OhlcCandle>,
    pub(super) verified: bool,
    seen: bool,
    candidates: u64,
    unready: u64,
    rejected: u64,
    passed: u64,
}
impl EntryContext {
    pub(super) fn invalidate(&mut self) {
        self.bars.clear();
        self.verified = false;
    }
    pub(super) fn replace(&mut self, bars: &[CompletedDailyBar]) {
        self.invalidate();
        self.seen = true;
        if !bars.iter().all(|b| valid_ohlc(&b.ohlc)) {
            return;
        }
        self.bars = bars[bars.len().saturating_sub(512)..]
            .iter()
            .map(|b| b.ohlc)
            .collect();
        self.verified = true;
    }
    pub(super) fn complete(&mut self, candle: OhlcCandle) {
        if !valid_ohlc(&candle) {
            self.invalidate();
            return;
        }
        self.seen = true;
        self.verified = true;
        self.bars.push_back(candle);
        while self.bars.len() > 512 {
            self.bars.pop_front();
        }
    }
    pub(super) fn len(&self) -> usize {
        self.bars.len()
    }
    pub(super) fn preparation(&self, p: &StrongCloseParams, valid: bool) -> EntryFilterPreparation {
        let status = if !valid {
            "invalidParams"
        } else if !self.seen {
            "unsupportedObservation"
        } else if !self.verified || self.bars.len() < p.long_trend_period {
            "insufficientHistory"
        } else {
            "ready"
        };
        EntryFilterPreparation {
            enabled: true,
            ready: status == "ready",
            status: status.into(),
            cadence: "priorCompletedDailyOhlc".into(),
            required_completed_bars: p.long_trend_period,
            available_completed_bars: self.bars.len(),
            true_range_samples: 0,
            candidates: self.candidates,
            unready_candidates: self.unready,
            min_move_rejections: 0,
            atr_rejections: 0,
            trend_rejections: self.rejected,
            passed_candidates: self.passed,
            slope_rejections: None,
            bandwidth_rejections: None,
            volume_rejections: None,
            breakout_basis: None,
            prior_volume_ratio: None,
            prior_atr: None,
        }
    }
    pub(super) fn accept(&mut self, p: &StrongCloseParams, valid: bool, price: u64) -> bool {
        if p.long_trend_period == 0 && valid {
            return true;
        }
        self.candidates = self.candidates.saturating_add(1);
        if !self.preparation(p, valid).ready {
            self.unready = self.unready.saturating_add(1);
            return false;
        }
        if !mean_close(&self.bars, p.long_trend_period).is_some_and(|mean| price as f64 > mean) {
            self.rejected = self.rejected.saturating_add(1);
            return false;
        }
        self.passed = self.passed.saturating_add(1);
        true
    }
    pub(super) fn reset_counters(&mut self) {
        self.candidates = 0;
        self.unready = 0;
        self.rejected = 0;
        self.passed = 0;
    }
}
