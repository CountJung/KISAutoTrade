use super::super::super::observation;
use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct MeanReversionState {
    pub(crate) prices: VecDeque<u64>,
    pub(crate) owned_quantity: u64,
    pub(crate) entry_price: Option<u64>,
    pub(super) held_observations: Option<u64>,
    pub(super) entry_candidate: bool,
    pub(super) live_started: bool,
    pub(super) trend_prices: VecDeque<u64>,
    pub(super) trend_observations: u64,
}

impl MeanReversionState {
    pub(super) fn trend_ready(&self, period: usize) -> bool {
        self.trend_prices.len() >= period
    }

    pub(super) fn trend_allows(&self, price: u64, period: usize) -> bool {
        observation::trend_allows(&self.trend_prices, period, price)
    }

    pub(super) fn observe_trend(&mut self, price: u64) {
        observation::observe_trend(&mut self.trend_prices, &mut self.trend_observations, price);
    }
}
