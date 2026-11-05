//! Confirmed ownership and cost must not be inferred from a broker-wide snapshot.
use super::*;
use crate::broker::{BrokerAccountId, BrokerMarket};
use crate::trading::order::{
    ownership::{OwnershipFill, OwnershipLedger, OwnershipOwner},
    PendingOrder,
};

pub(super) struct PreparedOwnershipFill {
    pub candidate: Option<OwnershipLedger>,
    pub pnl: i64,
}

impl OrderManager {
    pub(in crate::trading::order) async fn prune_completed_ownership_receipt(
        &mut self,
        pending: &PendingOrder,
        cumulative_quantity: u64,
        cumulative_notional: u128,
    ) -> Result<()> {
        let Some(key) = &pending.ownership_key else {
            return Ok(());
        };
        let mut ledger = self.load_ownership_ledger().await?;
        if pending.ownership_generation.as_deref() != Some(ledger.generation().to_string().as_str())
        {
            self.block_for_persistence_failure(
                "완료 체결 영수증의 소유 원장 세대가 다릅니다.".into(),
            );
            anyhow::bail!("완료 체결 영수증의 소유 원장 세대가 다릅니다.");
        }
        let proof = OwnershipFill {
            order_record_id: pending.record.id.clone(),
            key: key.clone(),
            side: pending.record.side,
            ordered_quantity: pending.record.quantity,
            cumulative_quantity: Some(cumulative_quantity),
            cumulative_notional: Some(cumulative_notional),
        };
        match ledger.prune_terminal_order(&proof) {
            Ok(true) => self.save_ownership_ledger(&ledger).await?,
            Ok(false) => {}
            Err(error) => {
                self.block_for_persistence_failure(format!("완료 체결 영수증 정리 실패: {error}"));
                return Err(error);
            }
        }
        Ok(())
    }

    pub(super) async fn prepare_confirmed_ownership(
        &mut self,
        pending: &PendingOrder,
        cumulative_quantity: u64,
        cumulative_notional: u128,
        delta_quantity: u64,
        is_overseas: bool,
    ) -> Result<PreparedOwnershipFill> {
        let prepared: Result<PreparedOwnershipFill> = async {
            let Some(key) = &pending.ownership_key else {
                anyhow::ensure!(
                    pending.ownership_generation.is_none(),
                    "소유 원장 세대가 있는 체결 주문의 소유 key가 없습니다."
                );
                let pnl = if let Some(pnl) = pending.application_pnl {
                    pnl
                } else if matches!(pending.record.side, OrderSide::Buy) {
                    0
                } else {
                    let basis = pending
                        .submitted_cost_basis
                        .filter(|price| *price > 0)
                        .ok_or_else(|| {
                            anyhow::anyhow!("매도 체결의 제출 시 확정된 매입 원가가 없습니다.")
                        })?;
                    let proceeds = cumulative_notional
                        .checked_sub(pending.filled_notional)
                        .ok_or_else(|| anyhow::anyhow!("누적 매도 체결 금액이 감소했습니다."))?;
                    let cost = u128::from(basis) * u128::from(delta_quantity);
                    realized_pnl(proceeds, cost)?
                };
                return Ok(PreparedOwnershipFill {
                    candidate: None,
                    pnl,
                });
            };
            let client = self.rest_client.read().await.clone();
            let mut scope = pending.broker_scope.clone();
            if scope.account_id.is_none() && scope.broker_id == BrokerId::Kis {
                anyhow::ensure!(
                    !client.account_no().trim().is_empty(),
                    "체결 소유 계좌를 확인할 수 없습니다."
                );
                scope.account_id = Some(BrokerAccountId(client.account_no().to_string()));
            }
            let market = if is_overseas {
                BrokerMarket::Us
            } else {
                BrokerMarket::Kr
            };
            let owner_matches = match (&key.owner, pending.strategy_id.as_deref()) {
                (OwnershipOwner::Strategy(owner), Some(strategy)) => owner == strategy,
                (OwnershipOwner::Manual, None) => true,
                _ => false,
            };
            anyhow::ensure!(
                owner_matches
                    && key.symbol == pending.record.symbol
                    && key.market == market
                    && key.identity.scope == scope
                    && key.identity.is_paper == client.is_paper(),
                "체결 소유 전략·계좌·프로파일·시장·투자 모드가 저장 주문과 일치하지 않습니다."
            );
            let mut candidate = self.load_ownership_ledger().await?;
            anyhow::ensure!(
                pending.ownership_generation.as_deref()
                    == Some(candidate.generation().to_string().as_str()),
                "체결 소유 원장 세대가 저장 주문과 일치하지 않습니다."
            );
            let was_flat = candidate.position(key).quantity == 0;
            let applied = candidate.apply_cumulative(&OwnershipFill {
                order_record_id: pending.record.id.clone(),
                key: key.clone(),
                side: pending.record.side,
                ordered_quantity: pending.record.quantity,
                cumulative_quantity: Some(cumulative_quantity),
                cumulative_notional: Some(cumulative_notional),
            })?;
            anyhow::ensure!(
                candidate
                    .order_watermark(&pending.record.id)
                    .is_some_and(|receipt| {
                        receipt.key == *key
                            && receipt.side == pending.record.side
                            && receipt.ordered_quantity == pending.record.quantity
                            && receipt.cumulative_quantity == cumulative_quantity
                            && receipt.cumulative_notional == cumulative_notional
                    }),
                "체결 소유 영수증과 현재 누적 체결 intent가 일치하지 않습니다."
            );
            if was_flat && applied.position.quantity > 0 {
                if let Some(metadata) = pending.strategy_position_metadata.clone() {
                    candidate.set_position_metadata(key, metadata)?;
                }
            }
            let pnl = if let Some(pnl) = pending.application_pnl {
                pnl
            } else if applied.delta_quantity > 0 {
                i64::try_from(applied.realized_pnl)?
            } else if matches!(pending.record.side, OrderSide::Buy) {
                0
            } else {
                let previous = candidate
                    .last_application(&pending.record.id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("이미 적용한 매도 체결의 원가·손익 영수증이 없습니다.")
                    })?;
                i64::try_from(previous.realized_pnl)?
            };
            Ok(PreparedOwnershipFill {
                candidate: Some(candidate),
                pnl,
            })
        }
        .await;
        if let Err(error) = &prepared {
            self.block_for_persistence_failure(format!("체결 소유·원가 검증 실패: {error}"));
        }
        prepared
    }
}

fn realized_pnl(proceeds: u128, cost: u128) -> Result<i64> {
    let value = if proceeds >= cost {
        i128::try_from(proceeds - cost)?
    } else {
        -i128::try_from(cost - proceeds)?
    };
    Ok(i64::try_from(value)?)
}

#[cfg(test)]
mod tests;
