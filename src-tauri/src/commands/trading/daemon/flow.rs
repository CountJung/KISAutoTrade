//! Configuration pauses and unsubmitted batches have distinct lifecycle outcomes.

use super::*;
use crate::trading::strategy::{StrategyManager, StrategySignal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MarketReadiness {
    ConfigurationPending,
    Open,
    Closed,
}

pub(super) fn market_readiness(
    manager: &StrategyManager,
    any_market_open: impl FnOnce(&StrategyManager) -> bool,
) -> MarketReadiness {
    if manager.configuration_edit_active() {
        MarketReadiness::ConfigurationPending
    } else if any_market_open(manager) {
        MarketReadiness::Open
    } else {
        MarketReadiness::Closed
    }
}

pub(super) async fn finish_closed_batch(
    fence: &RunFence<'_>,
    unsubmitted: Vec<StrategySignal>,
    exchange: Option<String>,
    tick_price: u64,
) -> TickCycleResult {
    if fence
        .settle_unsubmitted_signals(&unsubmitted, exchange, tick_price)
        .await
    {
        TickCycleResult::MarketClosed
    } else {
        TickCycleResult::Stopped
    }
}

#[cfg(test)]
mod tests;
