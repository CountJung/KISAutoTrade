//! Prior completed DAILY context only. Quote observations never manufacture OHLC.
use super::super::super::completed_daily::{atr14, mean_close, valid_ohlc};
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
    pct: u64,
    atr: u64,
    trend: u64,
    passed: u64,
}

impl ConsecutiveMoveParams {
    pub(super) fn enabled(&self) -> bool {
        self.min_cumulative_move_pct > 0.0
            || self.min_move_atr_multiplier > 0.0
            || self.long_trend_period > 0
    }
    pub(super) fn required_completed(&self) -> usize {
        (if self.min_move_atr_multiplier > 0.0 {
            15
        } else {
            0
        })
        .max(self.long_trend_period.min(512))
    }
}

pub(super) fn validate_entry_params(params: &serde_json::Value) -> Result<(), String> {
    for (key, max) in [
        ("min_cumulative_move_pct", 100.0),
        ("min_move_atr_multiplier", 20.0),
        ("long_trend_period", 512.0),
    ] {
        if let Some(raw) = params.get(key) {
            let value = raw.as_f64().filter(|n| {
                n.is_finite()
                    && *n >= 0.0
                    && *n <= max
                    && (key != "long_trend_period" || n.fract() == 0.0)
            });
            if value.is_none() {
                return Err(format!(
                    "{key}: 0~{max} 범위의 {}만 지원합니다.",
                    if key == "long_trend_period" {
                        "정수"
                    } else {
                        "유한수"
                    }
                ));
            }
        }
    }
    Ok(())
}

impl EntryContext {
    pub(super) fn reset_counters(&mut self) {
        self.counters = Counters::default();
    }

    pub(super) fn available(&self) -> usize {
        self.bars.len()
    }
    pub(super) fn trim(&mut self, required: usize) {
        while self.bars.len() > required {
            self.bars.pop_front();
        }
    }
    pub(super) fn invalidate(&mut self) {
        self.bars.clear();
        self.verified = false;
        self.awaiting_completion = false;
    }
    pub(super) fn replace(&mut self, candles: &[OhlcCandle], required: usize) {
        self.invalidate();
        self.seen_ohlc = true;
        if !candles.iter().all(valid_ohlc) {
            return;
        }
        self.bars = candles[candles.len().saturating_sub(required)..]
            .iter()
            .copied()
            .collect();
        self.verified = true;
    }
    pub(super) fn complete(&mut self, candle: OhlcCandle, required: usize) {
        if !self.awaiting_completion {
            return;
        }
        self.awaiting_completion = false;
        if required == 0 {
            return;
        }
        if !valid_ohlc(&candle) {
            self.invalidate();
            return;
        }
        self.seen_ohlc = true;
        self.verified = true;
        self.bars.push_back(candle);
        self.trim(required);
    }
    fn atr(&self) -> Option<f64> {
        atr14(&self.bars)
    }
    fn ma(&self, period: usize) -> Option<f64> {
        mean_close(&self.bars, period)
    }
    pub(super) fn preparation(
        &self,
        p: &ConsecutiveMoveParams,
        valid: bool,
    ) -> EntryFilterPreparation {
        let required = p.required_completed();
        let status = if !valid {
            "invalidParams"
        } else if required > 0 && !self.seen_ohlc {
            "unsupportedObservation"
        } else if required > 0 && (!self.verified || self.bars.len() < required) {
            "insufficientHistory"
        } else if p.min_move_atr_multiplier > 0.0 && self.atr().is_none() {
            "zeroAtr"
        } else {
            "ready"
        };
        EntryFilterPreparation {
            enabled: true,
            ready: status == "ready",
            status: status.into(),
            cadence: if required > 0 {
                "priorCompletedDailyOhlc"
            } else {
                "priceObservations"
            }
            .into(),
            required_completed_bars: required,
            available_completed_bars: self.bars.len(),
            true_range_samples: self.bars.len().saturating_sub(1).min(14),
            candidates: self.counters.candidates,
            unready_candidates: self.counters.unready,
            min_move_rejections: self.counters.pct,
            atr_rejections: self.counters.atr,
            trend_rejections: self.counters.trend,
            passed_candidates: self.counters.passed,
            slope_rejections: None,
            bandwidth_rejections: None,
            volume_rejections: None,
            breakout_basis: None,
            prior_volume_ratio: None,
            prior_atr: None,
        }
    }
    pub(super) fn accept_candidate(
        &mut self,
        p: &ConsecutiveMoveParams,
        valid: bool,
        prices: &VecDeque<u64>,
        price: u64,
    ) -> bool {
        if valid && !p.enabled() {
            return true;
        }
        self.counters.candidates = self.counters.candidates.saturating_add(1);
        let anchor = prices.iter().rev().nth(p.buy_days).copied().unwrap_or(0);
        if !self.preparation(p, valid).ready || anchor == 0 {
            self.counters.unready = self.counters.unready.saturating_add(1);
            return false;
        }
        let delta = price.saturating_sub(anchor) as f64;
        let pct_ok = p.min_cumulative_move_pct == 0.0
            || delta / anchor as f64 * 100.0 >= p.min_cumulative_move_pct;
        let atr_ok = p.min_move_atr_multiplier == 0.0
            || self
                .atr()
                .is_some_and(|atr| delta >= atr * p.min_move_atr_multiplier);
        let trend_ok = p.long_trend_period == 0
            || self
                .ma(p.long_trend_period)
                .is_some_and(|ma| price as f64 > ma);
        if !pct_ok {
            self.counters.pct = self.counters.pct.saturating_add(1);
        }
        if !atr_ok {
            self.counters.atr = self.counters.atr.saturating_add(1);
        }
        if !trend_ok {
            self.counters.trend = self.counters.trend.saturating_add(1);
        }
        if pct_ok && atr_ok && trend_ok {
            self.counters.passed = self.counters.passed.saturating_add(1);
            true
        } else {
            false
        }
    }
}
