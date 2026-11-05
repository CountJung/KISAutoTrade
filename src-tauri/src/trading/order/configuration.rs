//! 전략 설정 저장 중 영향 종목의 주문과 수동 전체 잔고 게시를 잠시 보류한다.

use super::execution_run::profile_fingerprint;
use super::*;
use crate::{
    broker::BrokerMarket,
    trading::{
        configuration::{
            effective_target_markets, ConfigurationEditReservation, ConfigurationUpdateError,
        },
        strategy::{BrokerPositionSnapshot, StrategyConfig},
    },
};
use std::{collections::HashMap, sync::Weak};

type EditResult<T> = std::result::Result<T, ConfigurationUpdateError>;

// 토큰 strong reference와 민감 원문은 저장하지 않는다.
pub(super) struct ConfigurationIdentity {
    client: Weak<KisRestClient>,
    profile_fingerprint: [u8; 32],
    previous_targets: HashMap<String, bool>,
    next_targets: HashMap<String, bool>,
}

impl OrderManager {
    pub(crate) async fn reserve_configuration_edit(
        &mut self,
        reservation: &Arc<ConfigurationEditReservation>,
        previous: &StrategyConfig,
        next: &StrategyConfig,
    ) -> EditResult<Vec<BrokerPositionSnapshot>> {
        if self
            .configuration_edit
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some()
        {
            return Err(ConfigurationUpdateError::new(
                "CONFIGURATION_BUSY",
                "다른 전략 설정을 저장하고 있습니다.",
            ));
        }
        if previous.id != reservation.strategy_id || next.id != reservation.strategy_id {
            return Err(ConfigurationUpdateError::new(
                "CONFIGURATION_MISMATCH",
                "전략 설정 예약이 변경할 전략과 다릅니다.",
            ));
        }
        let previous_targets = effective_target_markets(previous);
        let next_targets = effective_target_markets(next);
        let affected: std::collections::HashSet<_> = previous_targets
            .keys()
            .chain(next_targets.keys())
            .cloned()
            .collect();
        if affected != reservation.symbols {
            return Err(ConfigurationUpdateError::new(
                "CONFIGURATION_MISMATCH",
                "전략 설정의 영향 종목과 예약 범위가 다릅니다.",
            ));
        }
        let (client, profile_fingerprint) = self.configuration_scope_identity(reservation).await?;
        if self.submitting.keys().any(|(scope, symbol)| {
            scope == &reservation.scope && reservation.symbols.contains(symbol)
        }) || self.pending.values().any(|pending| {
            pending.broker_scope == reservation.scope
                && reservation.symbols.contains(&pending.record.symbol)
        }) || self.modifying.iter().any(|id| {
            self.pending.get(id).is_none_or(|pending| {
                pending.broker_scope == reservation.scope
                    && reservation.symbols.contains(&pending.record.symbol)
            })
        }) {
            return Err(ConfigurationUpdateError::new(
                "CONFIGURATION_BUSY",
                "변경할 종목의 제출·미체결·정정 주문이 있습니다. 처리가 끝난 후 다시 저장하세요.",
            ));
        }
        let identity = ConfigurationIdentity {
            client: Arc::downgrade(&client),
            profile_fingerprint,
            previous_targets,
            next_targets,
        };
        let snapshots = self.configuration_snapshots(reservation, &identity).await?;
        self.configuration_identity = Some(identity);
        self.configuration_edit = Some(Arc::downgrade(reservation));
        Ok(snapshots)
    }

    pub(crate) async fn configuration_position_snapshots(
        &self,
        reservation: &Arc<ConfigurationEditReservation>,
    ) -> EditResult<Vec<BrokerPositionSnapshot>> {
        let current = self
            .configuration_edit
            .as_ref()
            .and_then(Weak::upgrade)
            .filter(|current| Arc::ptr_eq(current, reservation))
            .ok_or_else(|| {
                ConfigurationUpdateError::new(
                    "CONFIGURATION_EXPIRED",
                    "전략 설정 저장 예약이 만료되었습니다.",
                )
            })?;
        let identity = self.configuration_identity.as_ref().ok_or_else(|| {
            ConfigurationUpdateError::new(
                "CONFIGURATION_EXPIRED",
                "전략 설정 저장 identity가 없습니다.",
            )
        })?;
        let (client, profile_fingerprint) = self.configuration_scope_identity(&current).await?;
        if identity
            .client
            .upgrade()
            .is_none_or(|previous| !Arc::ptr_eq(&previous, &client))
            || identity.profile_fingerprint != profile_fingerprint
        {
            return Err(ConfigurationUpdateError::new(
                "SCOPE_MISMATCH",
                "전략 설정 저장 중 계좌 인증 또는 모드가 변경되었습니다.",
            ));
        }
        self.configuration_snapshots(&current, identity).await
    }

    async fn configuration_scope_identity(
        &self,
        reservation: &ConfigurationEditReservation,
    ) -> EditResult<(Arc<KisRestClient>, [u8; 32])> {
        if self.execution_scope != reservation.scope {
            return Err(ConfigurationUpdateError::new(
                "SCOPE_MISMATCH",
                "전략 설정을 변경할 계좌가 현재 계좌와 다릅니다.",
            ));
        }
        let profiles = self.profiles.read().await;
        let profile = profiles
            .get_active()
            .filter(|profile| profile.id == reservation.profile_id)
            .ok_or_else(|| {
                ConfigurationUpdateError::new("SCOPE_MISMATCH", "활성 프로파일이 변경되었습니다.")
            })?;
        let client = self.rest_client.read().await.clone();
        execution_run::validate_snapshot(
            &reservation.scope,
            Some(&reservation.profile_id),
            &client,
            Some(profile),
        )
        .map_err(|_| {
            ConfigurationUpdateError::new(
                "SCOPE_MISMATCH",
                "전략 설정 계좌·인증·모드가 현재 client와 다릅니다.",
            )
        })?;
        Ok((client, profile_fingerprint(profile)))
    }

    async fn configuration_snapshots(
        &self,
        reservation: &ConfigurationEditReservation,
        identity: &ConfigurationIdentity,
    ) -> EditResult<Vec<BrokerPositionSnapshot>> {
        let domestic = self.position_tracker.lock().await;
        let overseas = self.overseas_position_tracker.lock().await;
        let mut symbols: Vec<_> = reservation.symbols.iter().collect();
        symbols.sort();
        let mut snapshots = Vec::with_capacity(symbols.len());
        for symbol in symbols {
            let kr = domestic.get(symbol);
            let us = overseas.get(symbol);
            let held = kr.is_some_and(|p| p.quantity > 0) || us.is_some_and(|p| p.quantity > 0);
            if held
                && identity.previous_targets.contains_key(symbol)
                && !identity.next_targets.contains_key(symbol)
            {
                return Err(ConfigurationUpdateError::new("HELD_TARGET_REMOVAL", format!("보유 종목 {symbol}은 전략 대상에서 제거할 수 없습니다. 먼저 보유 포지션을 정리하세요.")));
            }
            if held
                && identity
                    .previous_targets
                    .get(symbol)
                    .zip(identity.next_targets.get(symbol))
                    .is_some_and(|(old, new)| old != new)
            {
                return Err(ConfigurationUpdateError::new(
                    "HELD_MARKET_CHANGE",
                    format!("보유 종목 {symbol}의 국내·해외 구분은 변경할 수 없습니다."),
                ));
            }
            let is_us = identity
                .previous_targets
                .get(symbol)
                .or_else(|| identity.next_targets.get(symbol))
                .copied()
                .unwrap_or_else(|| !crate::market_hours::is_domestic_symbol(symbol));
            let (quantity, avg_price) = if is_us {
                us.map(|p| (p.quantity, p.avg_price_cents.round() as u64))
                    .unwrap_or((0, 0))
            } else {
                kr.map(|p| (p.quantity, p.avg_price.round() as u64))
                    .unwrap_or((0, 0))
            };
            snapshots.push(BrokerPositionSnapshot {
                broker_id: reservation.scope.broker_id,
                market: if is_us {
                    BrokerMarket::Us
                } else {
                    BrokerMarket::Kr
                },
                symbol: symbol.clone(),
                quantity,
                avg_price,
            });
        }
        Ok(snapshots)
    }

    pub(super) fn configuration_blocks_order(&self, scope: &BrokerScope, symbol: &str) -> bool {
        self.configuration_edit
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|reservation| {
                &reservation.scope == scope && reservation.symbols.contains(symbol)
            })
    }

    pub(super) fn configuration_blocks_holdings(&self, scope: &BrokerScope) -> bool {
        self.configuration_edit
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|reservation| &reservation.scope == scope)
    }
}

#[cfg(test)]
mod tests;
