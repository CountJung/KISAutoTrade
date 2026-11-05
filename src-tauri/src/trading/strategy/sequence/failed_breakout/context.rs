//! Bounded completed DAILY bars, never reconstructed from quotes or candidate OHLC.
use super::super::super::completed_daily::{mean_true_range14, valid_ohlc};
use super::*;
#[derive(Default)]
pub(super) struct EntryContext {
    bars: VecDeque<CompletedDailyBar>,
    verified: bool,
    seen: bool,
    pub(super) awaiting_completion: bool,
    candidates: u64,
    unready: u64,
    atr_rejections: u64,
    volume_rejections: u64,
    passed: u64,
}
impl EntryContext {
    pub(super) fn invalidate(&mut self) {
        self.bars.clear();
        self.verified = false;
        self.awaiting_completion = false;
    }
    pub(super) fn replace(&mut self, bars: &[CompletedDailyBar]) {
        self.invalidate();
        self.seen = true;
        if !bars.iter().all(|b| valid_ohlc(&b.ohlc)) {
            return;
        }
        self.bars = bars[bars.len().saturating_sub(512)..]
            .iter()
            .copied()
            .collect();
        self.verified = true;
    }
    pub(super) fn complete(&mut self, bar: CompletedDailyBar) {
        if !self.awaiting_completion {
            return;
        }
        self.awaiting_completion = false;
        if !valid_ohlc(&bar.ohlc) {
            self.invalidate();
            return;
        }
        self.seen = true;
        self.verified = true;
        self.bars.push_back(bar);
        while self.bars.len() > 512 {
            self.bars.pop_front();
        }
    }
    pub(super) fn high(&self, n: usize) -> Option<u64> {
        (self.verified && self.bars.len() >= n)
            .then(|| self.bars.iter().rev().take(n).map(|b| b.ohlc.high).max())
            .flatten()
    }
    fn atr(&self) -> Option<f64> {
        if !self.verified {
            return None;
        }
        mean_true_range14(&self.bars.iter().map(|b| b.ohlc).collect())
    }
    fn ratio(&self) -> Result<f64, &'static str> {
        if self.bars.len() < 21 {
            return Err("insufficientHistory");
        }
        let mut bars = self.bars.iter().rev();
        let latest = bars
            .next()
            .and_then(|b| b.volume)
            .ok_or("unverifiedVolume")?;
        let mut sum = 0u128;
        for b in bars.take(20) {
            sum += b.volume.ok_or("unverifiedVolume")? as u128;
        }
        if sum == 0 {
            return Err("zeroReferenceVolume");
        }
        Ok(latest as f64 / (sum as f64 / 20.0))
    }
    pub(super) fn preparation(
        &self,
        p: &FailedBreakoutParams,
        valid: bool,
    ) -> EntryFilterPreparation {
        let required = p.required_completed();
        let status = if !valid {
            "invalidParams"
        } else if required > 0 && !self.seen {
            "unsupportedObservation"
        } else if required > 0 && (!self.verified || self.bars.len() < required) {
            "insufficientHistory"
        } else if p.min_volume_ratio > 0.0 {
            self.ratio().err().unwrap_or("ready")
        } else {
            "ready"
        };
        EntryFilterPreparation {
            enabled: true,
            ready: status == "ready",
            status: status.into(),
            cadence: if required > 0 {
                "priorCompletedDailyOhlcv"
            } else {
                "priceObservations"
            }
            .into(),
            required_completed_bars: required,
            available_completed_bars: self.bars.len(),
            true_range_samples: self.bars.len().saturating_sub(1).min(14),
            candidates: self.candidates,
            unready_candidates: self.unready,
            min_move_rejections: 0,
            atr_rejections: self.atr_rejections,
            trend_rejections: 0,
            passed_candidates: self.passed,
            slope_rejections: None,
            bandwidth_rejections: None,
            volume_rejections: Some(self.volume_rejections),
            breakout_basis: Some(
                match p.breakout_basis {
                    FailedBreakoutBasis::PriceObservations => "priceObservations",
                    FailedBreakoutBasis::CompletedHigh => "completedHigh",
                }
                .into(),
            ),
            prior_volume_ratio: (p.min_volume_ratio > 0.0)
                .then(|| self.ratio().ok())
                .flatten(),
            prior_atr: (p.atr_buffer_multiplier > 0.0)
                .then(|| self.atr())
                .flatten(),
        }
    }
    /// Count flat baseline-% candidates. With no completed-high anchor, use the prior
    /// price anchor only to record an unready candidate; it can never authorize a Buy.
    pub(super) fn accept(
        &mut self,
        p: &FailedBreakoutParams,
        valid: bool,
        anchor: u64,
        price: u64,
    ) -> bool {
        self.candidates = self.candidates.saturating_add(1);
        if !self.preparation(p, valid).ready {
            self.unready = self.unready.saturating_add(1);
            return false;
        }
        let legacy = (anchor as f64 * (1.0 + p.buffer_pct / 100.0)) as u64;
        let atr_ok = p.atr_buffer_multiplier == 0.0
            || self.atr().is_some_and(|atr| {
                price as f64 >= (legacy as f64).max(anchor as f64 + p.atr_buffer_multiplier * atr)
            });
        let volume_ok = p.min_volume_ratio == 0.0
            || self.ratio().is_ok_and(|ratio| ratio >= p.min_volume_ratio);
        if !atr_ok {
            self.atr_rejections = self.atr_rejections.saturating_add(1);
        }
        if !volume_ok {
            self.volume_rejections = self.volume_rejections.saturating_add(1);
        }
        if atr_ok && volume_ok {
            self.passed = self.passed.saturating_add(1);
            true
        } else {
            false
        }
    }
    pub(super) fn reset_counters(&mut self) {
        self.candidates = 0;
        self.unready = 0;
        self.atr_rejections = 0;
        self.volume_rejections = 0;
        self.passed = 0;
    }
}
