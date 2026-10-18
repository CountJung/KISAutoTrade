//! A nonblocking configuration lease and current-generation signal bookkeeping.
use super::*;
use crate::trading::configuration::{
    effective_target_markets, ConfigurationEditReservation, ConfigurationUpdateError,
};
use std::sync::Weak;

impl StrategyManager {
    pub(crate) fn pause_configuration_edit(
        &mut self,
        reservation: Weak<ConfigurationEditReservation>,
    ) {
        self.configuration_edit = Some(reservation);
    }

    pub(crate) fn configuration_edit_active(&self) -> bool {
        self.configuration_edit
            .as_ref()
            .is_some_and(|edit| edit.strong_count() > 0)
    }

    pub(crate) fn has_unsettled_signals(&self) -> bool {
        self.unsettled_signals
            .values()
            .any(|revision| *revision == self.history_generation)
    }

    pub(crate) fn complete_signal(&mut self, strategy_id: &str, symbol: &str, revision: u64) {
        let key = (strategy_id.to_string(), symbol.to_string());
        if self.unsettled_signals.get(&key) == Some(&revision) {
            self.unsettled_signals.remove(&key);
        }
    }

    pub(crate) fn settle_unsubmitted_signal(
        &mut self,
        strategy_id: &str,
        symbol: &str,
        revision: u64,
        quantity: u64,
        avg_price: u64,
    ) {
        if self.history_generation != revision
            || self
                .unsettled_signals
                .get(&(strategy_id.to_string(), symbol.to_string()))
                != Some(&revision)
        {
            return;
        }
        if let Some(strategy) = self.strategies.iter_mut().find(|s| s.id() == strategy_id) {
            strategy.sync_position(symbol, quantity, avg_price);
        }
        self.complete_signal(strategy_id, symbol, revision);
    }

    pub(crate) fn validate_configuration_targets(
        &self,
        candidate: &StrategyConfig,
    ) -> Result<(), ConfigurationUpdateError> {
        let Some(strategy) = self.strategies.iter().find(|s| s.id() == candidate.id) else {
            return Ok(());
        };
        let previous = effective_target_markets(strategy.config());
        let next = effective_target_markets(candidate);
        for (symbol, market) in previous {
            if strategy.has_held_position(&symbol) != Some(true) {
                continue;
            }
            match next.get(&symbol) {
                None => return Err(ConfigurationUpdateError::new(
                    "HELD_TARGET_REMOVAL",
                    format!("보유 보호 중인 종목 {symbol}은 전략 대상에서 제거할 수 없습니다. 먼저 보유 상태를 확인하세요."),
                )),
                Some(next_market) if *next_market != market => return Err(ConfigurationUpdateError::new(
                    "HELD_MARKET_CHANGE",
                    format!("보유 보호 중인 종목 {symbol}의 국내·해외 구분은 변경할 수 없습니다."),
                )),
                Some(_) => {},
            }
        }
        Ok(())
    }
}
