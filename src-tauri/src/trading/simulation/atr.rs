use std::collections::VecDeque;

use serde::Serialize;

use super::SimulationEvent;

const PERIOD: usize = 14;

/// Readiness of the replay's own bar interval, separate from strategy warmup.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtrPreparationView {
    pub period: usize,
    pub history_bars: usize,
    pub ready_at_start: bool,
    pub ready_at_end: bool,
    pub first_ready_time: Option<String>,
    pub unready_evaluation_bars: usize,
    pub range_policy: String,
}

/// Replay-specific strict 14-range window. The first close only establishes a
/// reference; 15 observed bars supply 14 complete true ranges. Missing bars are
/// never interpolated and adjacent observed closes retain overnight gaps.
#[derive(Default)]
pub(super) struct AtrWindow {
    previous_close: Option<u64>,
    ranges: VecDeque<u64>,
}

impl AtrWindow {
    pub fn observe(&mut self, event: &SimulationEvent) {
        if event.close_units == 0 {
            return;
        }
        if event.high_units >= event.low_units && event.low_units > 0 {
            if let Some(previous) = self.previous_close {
                let range = event
                    .high_units
                    .saturating_sub(event.low_units)
                    .max(event.high_units.abs_diff(previous))
                    .max(event.low_units.abs_diff(previous));
                self.ranges.push_back(range);
                if self.ranges.len() > PERIOD {
                    self.ranges.pop_front();
                }
            }
        }
        self.previous_close = Some(event.close_units);
    }

    pub fn ready(&self) -> bool {
        self.ranges.len() == PERIOD
    }

    pub fn value(&self) -> Option<u64> {
        self.ready().then(|| {
            // u128 avoids overflow even for adversarial positive u64 prices.
            (self.ranges.iter().map(|v| *v as u128).sum::<u128>() / PERIOD as u128) as u64
        })
    }

    pub fn available_ranges(&self) -> usize {
        self.ranges.len()
    }

    pub fn preparation(history_bars: usize, ready: bool) -> AtrPreparationView {
        AtrPreparationView {
            period: PERIOD,
            history_bars,
            ready_at_start: ready,
            ready_at_end: ready,
            first_ready_time: None,
            unready_evaluation_bars: 0,
            range_policy: "observedBarsWithPreviousCloseNoInterpolation".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(close: u64, high: u64, low: u64) -> SimulationEvent {
        SimulationEvent {
            time: "20261001".into(),
            chart_time: "20261001".into(),
            open_units: close,
            close_units: close,
            high_units: high,
            low_units: low,
            signal: None,
        }
    }

    #[test]
    fn full_previous_close_history_is_required_and_old_ranges_expire() {
        let mut atr = AtrWindow::default();
        for _ in 0..14 {
            atr.observe(&bar(100, 101, 99));
        }
        assert_eq!(atr.value(), None);
        atr.observe(&bar(100, 101, 99));
        assert_eq!(atr.value(), Some(2));
        atr.observe(&bar(200, 200, 200));
        assert_eq!(atr.value(), Some(9)); // (13 * 2 + gap 100) / 14
        for _ in 0..14 {
            atr.observe(&bar(200, 201, 199));
        }
        assert_eq!(atr.value(), Some(2));
        assert_eq!(atr.available_ranges(), 14);
    }

    #[test]
    fn downward_gap_on_flat_candle_is_not_a_zero_range() {
        let mut atr = AtrWindow::default();
        for _ in 0..15 {
            atr.observe(&bar(200, 201, 199));
        }
        atr.observe(&bar(100, 100, 100));
        assert_eq!(atr.value(), Some(9));
    }

    #[test]
    fn large_prices_do_not_overflow_the_range_sum() {
        let mut atr = AtrWindow::default();
        atr.observe(&bar(u64::MAX, u64::MAX, 1));
        for _ in 0..14 {
            atr.observe(&bar(1, u64::MAX, 1));
        }
        assert_eq!(atr.value(), Some(u64::MAX - 1));
    }
}
