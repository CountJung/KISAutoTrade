//! One-shot, scope-pinned holdings responses and atomic tracker publication.
use super::*;
use crate::trading::position::PositionSnapshotVersion;
use crate::{broker::BrokerId, config::AccountProfile};
use std::sync::Weak;

mod fetch;
mod parse;

pub(crate) fn normalize_overseas_order_exchange(code: &str) -> String {
    match code.trim().to_uppercase().as_str() {
        "NAS" | "NASD" | "NASDAQ" => "NASD".to_string(),
        "NYS" | "NYSE" => "NYSE".to_string(),
        "AMS" | "AMEX" => "AMEX".to_string(),
        other => other.to_string(),
    }
}

type DomesticHoldings = Vec<(String, String, u64, u64, u64)>;
type OverseasHoldings = Vec<(String, String, String, u64, u64, u64)>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HoldingsMarkets {
    Domestic,
    Overseas,
    Both,
    ReadOnly,
}

pub(crate) enum HoldingsUpdate {
    Domestic(DomesticHoldings),
    Overseas(OverseasHoldings),
    Both {
        domestic: DomesticHoldings,
        overseas: OverseasHoldings,
    },
    ReadOnly,
}

/// Contains credentials only for the provider call. Never Debug/serialize it.
pub(crate) struct HoldingsQuery {
    scope: BrokerScope,
    epoch: u64,
    client: Arc<KisRestClient>,
    profile: Option<AccountProfile>,
    configuration_edit: Option<Weak<crate::trading::configuration::ConfigurationEditReservation>>,
    domestic: Option<PositionSnapshotVersion>,
    overseas: Option<PositionSnapshotVersion>,
    markets: HoldingsMarkets,
}

impl HoldingsQuery {
    pub(crate) fn client(&self) -> Arc<KisRestClient> {
        self.client.clone()
    }
    pub(crate) fn profile(&self) -> Option<&AccountProfile> {
        self.profile.as_ref()
    }
}

struct ExpectedSource<'a> {
    scope: &'a BrokerScope,
    client: &'a Arc<KisRestClient>,
    profile: Option<&'a AccountProfile>,
}

fn same_profile(previous: Option<&AccountProfile>, current: Option<&AccountProfile>) -> bool {
    match (previous, current) {
        (Some(previous), Some(current)) => {
            previous.broker_id == current.broker_id
                && execution_run::profile_fingerprint(previous)
                    == execution_run::profile_fingerprint(current)
        }
        (None, None) => true,
        _ => false,
    }
}

fn same_edit(
    previous: &Option<Weak<crate::trading::configuration::ConfigurationEditReservation>>,
    current: &Option<Weak<crate::trading::configuration::ConfigurationEditReservation>>,
) -> bool {
    match (previous, current) {
        (Some(previous), Some(current)) => Weak::ptr_eq(previous, current),
        (None, None) => true,
        _ => false,
    }
}

impl OrderManager {
    pub(crate) async fn begin_holdings_query(
        &mut self,
        broker: BrokerId,
        markets: HoldingsMarkets,
    ) -> Result<HoldingsQuery> {
        self.capture_holdings_query(broker, markets, None).await
    }

    pub(crate) async fn begin_holdings_query_for_snapshot(
        &mut self,
        scope: &BrokerScope,
        client: &Arc<KisRestClient>,
        profile: Option<&AccountProfile>,
        markets: HoldingsMarkets,
    ) -> Result<HoldingsQuery> {
        self.capture_holdings_query(
            scope.broker_id,
            markets,
            Some(ExpectedSource {
                scope,
                client,
                profile,
            }),
        )
        .await
    }

    async fn capture_holdings_query(
        &mut self,
        broker: BrokerId,
        markets: HoldingsMarkets,
        expected: Option<ExpectedSource<'_>>,
    ) -> Result<HoldingsQuery> {
        anyhow::ensure!(
            self.execution_scope.broker_id == broker,
            "잔고 조회 broker가 현재 계좌와 다릅니다. 다시 조회하세요."
        );
        anyhow::ensure!(
            !self.configuration_blocks_holdings(&self.execution_scope),
            "전략 설정 저장 중에는 잔고 갱신을 보류합니다."
        );
        let profile = self.profiles.read().await.get_active().cloned();
        let client = self.rest_client.read().await.clone();
        execution_run::validate_snapshot(
            &self.execution_scope,
            profile.as_ref().map(|p| p.id.as_str()),
            &client,
            profile.as_ref(),
        )?;
        if let Some(expected) = expected {
            anyhow::ensure!(
                &self.execution_scope == expected.scope
                    && Arc::ptr_eq(&client, expected.client)
                    && same_profile(expected.profile, profile.as_ref()),
                "잔고 조회를 준비하는 동안 계좌·인증이 변경되었습니다."
            );
        }
        // Invalid requests above cannot supersede a legitimate in-flight query.
        let domestic = if matches!(markets, HoldingsMarkets::Domestic | HoldingsMarkets::Both) {
            Some(self.position_tracker.lock().await.begin_snapshot())
        } else {
            None
        };
        let overseas = if matches!(markets, HoldingsMarkets::Overseas | HoldingsMarkets::Both) {
            Some(self.overseas_position_tracker.lock().await.begin_snapshot())
        } else {
            None
        };
        Ok(HoldingsQuery {
            scope: self.execution_scope.clone(),
            epoch: self.execution_epoch,
            client,
            profile,
            configuration_edit: self.configuration_edit.clone(),
            domestic,
            overseas,
            markets,
        })
    }

    /// The caller holds the order gate; provider I/O must already be complete.
    /// Validate both markets before replacing either, including a real empty list.
    pub(crate) async fn publish_holdings(
        &mut self,
        query: HoldingsQuery,
        update: HoldingsUpdate,
    ) -> Result<()> {
        anyhow::ensure!(
            query.scope == self.execution_scope && query.epoch == self.execution_epoch,
            "잔고 조회 중 계좌 또는 실행 상태가 변경되었습니다. 다시 조회하세요."
        );
        anyhow::ensure!(
            !self.configuration_blocks_holdings(&query.scope)
                && same_edit(&query.configuration_edit, &self.configuration_edit),
            "전략 설정 저장과 겹친 잔고 응답을 폐기했습니다. 다시 조회하세요."
        );
        let profiles = self.profiles.read().await;
        let current = self.rest_client.read().await;
        anyhow::ensure!(
            Arc::ptr_eq(&query.client, &current)
                && same_profile(query.profile.as_ref(), profiles.get_active()),
            "잔고 조회 중 프로파일·인증·모드가 변경되었습니다. 다시 조회하세요."
        );
        execution_run::validate_snapshot(
            &query.scope,
            query.profile.as_ref().map(|p| p.id.as_str()),
            &current,
            profiles.get_active(),
        )?;
        let (domestic, overseas) = match (query.markets, update) {
            (HoldingsMarkets::Domestic, HoldingsUpdate::Domestic(items)) => (Some(items), None),
            (HoldingsMarkets::Overseas, HoldingsUpdate::Overseas(items)) => (None, Some(items)),
            (HoldingsMarkets::Both, HoldingsUpdate::Both { domestic, overseas }) => {
                (Some(domestic), Some(overseas))
            }
            (HoldingsMarkets::ReadOnly, HoldingsUpdate::ReadOnly) => return Ok(()),
            _ => anyhow::bail!("잔고 응답의 시장 범위가 조회 예약과 다릅니다."),
        };
        let mut kr = self.position_tracker.lock().await;
        let mut us = self.overseas_position_tracker.lock().await;
        anyhow::ensure!(
            query
                .domestic
                .as_ref()
                .is_none_or(|version| kr.snapshot_is_current(version))
                && query
                    .overseas
                    .as_ref()
                    .is_none_or(|version| us.snapshot_is_current(version)),
            "새 잔고 조회 또는 체결·포지션 변경과 겹친 응답을 폐기했습니다. 다시 조회하세요."
        );
        if let Some(items) = domestic {
            kr.replace(items);
            self.holdings_refresh_required &= !1;
        }
        if let Some(items) = overseas {
            us.replace(items);
            self.holdings_refresh_required &= !2;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
