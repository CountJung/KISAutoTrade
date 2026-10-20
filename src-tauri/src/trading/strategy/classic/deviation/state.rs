use super::super::super::state::bounded_window;
use super::DeviationParams;
use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct DeviationState {
    pub(crate) prices: VecDeque<u64>,
    pub(super) owned_quantity: u64,
    pub(super) avg_price: u64,
    pub(super) held_observations: Option<u64>,
    pub(super) previous_deviation: Option<f64>,
    pub(super) retry_candidate: bool,
    pub(super) entry_candidate: bool,
    pub(super) live_started: bool,
    pub(super) trend_prices: VecDeque<u64>,
    pub(super) trend_observations: u64,
}
pub(super) fn capacity(p: &DeviationParams) -> usize {
    bounded_window(p.ma_period).max(p.long_trend_period.saturating_add(1))
}
impl DeviationState {
    pub(super) fn observe(
        &mut self,
        price: u64,
        p: &DeviationParams,
        create_candidate: bool,
        advance_trend: bool,
    ) -> Option<(f64, f64, bool)> {
        let trend_ok = if p.long_trend_period == 0 {
            true
        } else if self.trend_prices.len() < p.long_trend_period {
            false
        } else {
            let mean = self
                .trend_prices
                .iter()
                .rev()
                .take(p.long_trend_period)
                .map(|v| *v as f64)
                .sum::<f64>()
                / p.long_trend_period as f64;
            mean > 0.0 && price as f64 > mean
        };
        if advance_trend {
            self.trend_prices.push_back(price);
            while self.trend_prices.len() > 512 {
                self.trend_prices.pop_front();
            }
            self.trend_observations = self.trend_observations.saturating_add(1);
        }
        self.prices.push_back(price);
        while self.prices.len() > capacity(p) {
            self.prices.pop_front();
        }
        let period = bounded_window(p.ma_period);
        if self.prices.len() < period {
            self.previous_deviation = None;
            self.retry_candidate = false;
            return None;
        }
        // Convert before summation to avoid u64 accumulator overflow.
        let ma = self
            .prices
            .iter()
            .rev()
            .take(period)
            .map(|v| *v as f64)
            .sum::<f64>()
            / period as f64;
        if ma == 0.0 {
            self.previous_deviation = None;
            self.retry_candidate = false;
            return None;
        }
        let deviation = (price as f64 / ma - 1.0) * 100.0;
        if deviation > p.buy_threshold_pct {
            self.retry_candidate = false;
        }
        if self
            .previous_deviation
            .is_some_and(|prev| prev > p.buy_threshold_pct)
            && deviation <= p.buy_threshold_pct
        {
            self.retry_candidate = create_candidate && self.owned_quantity == 0;
        }
        self.previous_deviation = Some(deviation);
        Some((deviation, ma, trend_ok))
    }
}
