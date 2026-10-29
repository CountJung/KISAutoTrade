//! Inspect all retained primary history before binding an operator-supplied ID.
//! A conflicting ID is rejected across markets/modes when scope evidence is ambiguous.
use super::*;
use crate::broker::BrokerId;

fn possible_scope(broker: Option<BrokerId>, account: Option<&str>, scope: &BrokerScope) -> bool {
    broker.is_none_or(|broker| broker == scope.broker_id)
        && account.is_none_or(|account| scope.account_id.as_ref().is_some_and(|id| id.0 == account))
}
fn own_fill(id: &str, record_id: &str) -> bool {
    (id.starts_with("order-fill:") || id.starts_with("trade-fill:"))
        && id.ends_with(&format!(":{record_id}"))
}
impl OrderManager {
    pub(super) async fn validate_recovery_history(
        &self,
        pending: &PendingOrder,
        provider_id: &str,
        has_pending: bool,
    ) -> Result<()> {
        // A provider ID can be reused accidentally from any retained day (Toss
        // lookup is by ID). Ambiguous legacy account/mode provenance also blocks.
        {
            for record in self.order_store.recovery_records().await? {
                if !possible_scope(
                    record.broker_id,
                    record.broker_account_id.as_deref(),
                    &pending.broker_scope,
                ) || (record.provider_order_id.as_deref() != Some(provider_id)
                    && record.kis_order_id.as_deref() != Some(provider_id))
                {
                    continue;
                }
                let belongs =
                    record.id == pending.record.id || own_fill(&record.id, &pending.record.id);
                anyhow::ensure!(
                    belongs,
                    "이미 다른 로컬 주문 이력에 사용된 provider 주문번호입니다."
                );
                anyhow::ensure!(
                    has_pending || record.status == OrderStatus::Pending,
                    "이미 체결·종료 이력이 있으나 원래 pending intent가 없어 보류합니다."
                );
            }
            for record in self.trade_store.recovery_records().await? {
                if !possible_scope(
                    record.broker_id,
                    record.broker_account_id.as_deref(),
                    &pending.broker_scope,
                ) || (record.provider_order_id.as_deref() != Some(provider_id)
                    && record.order_id != provider_id)
                {
                    continue;
                }
                anyhow::ensure!(
                    own_fill(&record.id, &pending.record.id),
                    "이미 다른 로컬 체결 이력에 사용된 provider 주문번호입니다."
                );
                anyhow::ensure!(
                    has_pending,
                    "이미 체결 이력이 있으나 원래 pending intent가 없어 보류합니다."
                );
            }
        }
        Ok(())
    }
}
