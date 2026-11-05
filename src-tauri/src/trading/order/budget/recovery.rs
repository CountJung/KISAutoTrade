//! Operator-attested binding only. Never submits orders or accepts supplied fills.
use super::*;
use crate::trading::order::ownership::{OwnershipIdentity, OwnershipOwner};

mod history;

const MAX_RECEIPTS: usize = 256;
const PAGE_SIZE: usize = 100;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct RecoverySnapshot {
    revision: String,
    pending: PendingOrder,
}
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct RecoveryReceipt {
    snapshot: RecoverySnapshot,
    provider_order_id: String,
    evidence_reference: String,
    approved_at: String,
    pub(super) resolved: bool,
    pub(super) filled_quantity: u64,
    pub(super) filled_notional: u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApproveBudgetRecoveryInput {
    pub broker_id: crate::broker::BrokerId,
    pub broker_account_id: String,
    pub reservation_id: String,
    pub snapshot_revision: String,
    pub provider_order_id: String,
    pub client_order_id: Option<String>,
    pub evidence_reference: String,
    pub confirmed: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetRecoveryView {
    pub scope: BrokerScope,
    pub items: Vec<BudgetRecoveryItem>,
    pub total_count: usize,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetRecoveryItem {
    pub reservation_id: String,
    pub currency: &'static str,
    pub market: Option<crate::broker::BrokerMarket>,
    pub exchange: Option<String>,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: u64,
    pub order_price: u64,
    pub filled_quantity: u64,
    pub filled_notional: u64,
    pub reserved_amount: u64,
    pub submitted_at: Option<String>,
    pub profile_id: Option<String>,
    pub is_paper: Option<bool>,
    pub client_order_id: Option<String>,
    pub provider_order_id: Option<String>,
    pub snapshot_revision: Option<String>,
    pub evidence_reference: Option<String>,
    pub state: &'static str,
    pub blocked_reason: Option<String>,
}
// Keep memory bounded while unresolved items precede already linked/completed receipts.
fn push_bounded(items: &mut Vec<BudgetRecoveryItem>, item: BudgetRecoveryItem) {
    items.push(item);
    items.sort_by_key(|item| {
        (
            match item.state {
                "unresolved" => 0,
                "awaitingReconciliation" => 1,
                _ => 2,
            },
            item.reservation_id.clone(),
        )
    });
    items.truncate(PAGE_SIZE);
}
fn is_us(p: &PendingOrder) -> bool {
    p.exchange.is_some() || !crate::market_hours::is_domestic_symbol(&p.record.symbol)
}
fn bounded(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
impl RecoverySnapshot {
    fn validate(&self, id: &str, scope: &BrokerScope, r: &BudgetReservation) -> Result<()> {
        let p = &self.pending;
        let key = p
            .ownership_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("원래 주문 소유 식별자가 없습니다."))?;
        anyhow::ensure!(
            bounded(&self.revision, 64)
                && p.record.id == id
                && &p.broker_scope == scope
                && key.identity.scope == *scope
                && p.record.matches_scope(scope)
                && p.record.provider.as_deref()
                    == Some(match scope.broker_id {
                        crate::broker::BrokerId::Kis => "kis",
                        crate::broker::BrokerId::Toss => "toss",
                    })
                && key.symbol == r.symbol
                && p.record.symbol == r.symbol
                && p.record.quantity == r.quantity
                && p.signal_price == r.price
                && p.order_price == p.record.price
                && (p.record.side == OrderSide::Buy) == r.buy
                && p.strategy_id == p.record.strategy_id
                && matches!(&key.owner, OwnershipOwner::Strategy(s) if Some(s) == p.strategy_id.as_ref())
                && p.ownership_generation
                    .as_ref()
                    .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
                && p.filled_quantity == 0
                && p.filled_notional == 0
                && p.confirmed_filled_quantity == 0
                && p.confirmed_avg_price == 0
                && p.confirmed_filled_notional.is_none()
                && p.application_execution_date.is_none()
                && p.application_exchange_rate.is_none()
                && p.application_pnl.is_none()
                && p.record.status == OrderStatus::Pending
                && p.record.provider_request_id == p.client_order_id
                && !p.application_started
                && p.record.provider_order_id.is_none()
                && p.record.kis_order_id.is_none()
                && chrono::DateTime::parse_from_rfc3339(&p.record.timestamp).is_ok(),
            "복구 스냅샷과 예약의 원래 주문 필드가 일치하지 않습니다."
        );
        let expected_market = if is_us(p) {
            crate::broker::BrokerMarket::Us
        } else {
            crate::broker::BrokerMarket::Kr
        };
        anyhow::ensure!(
            key.market == expected_market,
            "복구 스냅샷 시장이 일치하지 않습니다."
        );
        if scope.broker_id == crate::broker::BrokerId::Kis {
            anyhow::ensure!(
                OrderManager::kis_order_is_paper(p.record.provider_tr_id.as_deref())?
                    == key.identity.is_paper,
                "복구 주문의 투자 모드가 일치하지 않습니다."
            );
        }
        Ok(())
    }
}
impl OrderManager {
    /// Reapply protection from durable receipts before any new submission guard.
    /// This is submission provenance only: no fill/risk/position counters change.
    pub(super) fn restore_budget_guards(&mut self, ledger: &BudgetLedger) -> Result<()> {
        anyhow::ensure!(
            ledger.recovery_receipts.len() <= MAX_RECEIPTS,
            "복구 승인 영수증 한도 초과"
        );
        for (id, receipt) in &ledger.recovery_receipts {
            let pending = &receipt.snapshot.pending;
            anyhow::ensure!(
                &pending.record.id == id,
                "복구 영수증의 원래 예약 ID가 다릅니다."
            );
            let signal = match pending.record.side {
                OrderSide::Buy => Signal::Buy {
                    symbol: pending.record.symbol.clone(),
                    quantity: pending.record.quantity,
                    reason: pending.signal_reason.clone(),
                },
                OrderSide::Sell => Signal::Sell {
                    symbol: pending.record.symbol.clone(),
                    quantity: pending.record.quantity,
                    reason: pending.signal_reason.clone(),
                },
            };
            let submitted = chrono::DateTime::parse_from_rfc3339(&pending.record.timestamp)?;
            self.trade_guard.restore_recovered_submission(
                id,
                &pending.broker_scope,
                &signal,
                submitted,
            )?;
        }
        Ok(())
    }
    pub(in crate::trading::order) async fn capture_budget_recovery(
        &mut self,
        pending: PendingOrder,
    ) -> Result<()> {
        self.load_budget().await?;
        let mut ledger = self.budget.clone().expect("loaded ledger");
        let r = ledger
            .currency(&pending.broker_scope, is_us(&pending))
            .reservations
            .get_mut(&pending.record.id)
            .ok_or_else(|| anyhow::anyhow!("복구 예약이 없습니다."))?;
        let snapshot = RecoverySnapshot {
            revision: uuid::Uuid::new_v4().to_string(),
            pending,
        };
        snapshot.validate(
            &snapshot.pending.record.id,
            &snapshot.pending.broker_scope,
            r,
        )?;
        anyhow::ensure!(
            serde_json::to_vec(&snapshot)?.len() <= 128 * 1024,
            "복구 스냅샷 크기 한도 초과"
        );
        anyhow::ensure!(
            r.recovery_snapshot.is_none(),
            "복구 스냅샷은 덮어쓸 수 없습니다."
        );
        r.recovery_snapshot = Some(snapshot);
        self.save_budget(ledger).await
    }
    async fn recovery_identity(&self, scope: &BrokerScope) -> Result<OwnershipIdentity> {
        anyhow::ensure!(
            &self.execution_scope == scope,
            "현재 실행 계좌와 복구 계좌가 다릅니다."
        );
        let profiles = self.profiles.read().await;
        let client = self.rest_client.read().await;
        OwnershipIdentity::capture(scope, profiles.get_active(), &client)
    }
    pub async fn budget_recovery(&mut self, scope: &BrokerScope) -> Result<BudgetRecoveryView> {
        self.load_budget().await?;
        let identity = self.recovery_identity(scope).await.ok();
        let ledger = self.budget.as_ref().expect("loaded ledger");
        let mut items = Vec::new();
        let mut total_count = 0;
        if let Some(a) = ledger.accounts.iter().find(|a| &a.scope == scope) {
            for (currency, c) in [("KRW", &a.krw), ("USD", &a.usd)] {
                for (id, r) in &c.reservations {
                    let receipt = ledger.recovery_receipts.get(id);
                    let pending = self.pending.values().find(|p| &p.record.id == id);
                    if receipt.is_none() && (pending.is_some() || r.terminal) {
                        continue;
                    }
                    total_count += 1;
                    let snapshot = r.recovery_snapshot.as_ref();
                    let p = snapshot.map(|s| &s.pending);
                    let resolved = r.terminal || receipt.is_some_and(|r| r.resolved);
                    let blocked_reason = if resolved || pending.is_some() {
                        None
                    } else if r.filled > 0 {
                        Some(
                            "일부 체결 후 pending 유실: 원래 체결 intent 증거가 부족합니다.".into(),
                        )
                    } else if let Some(s) = snapshot {
                        s.validate(id, scope, r)
                            .err()
                            .map(|e| e.to_string())
                            .or_else(|| {
                                (s.pending.ownership_key.as_ref().map(|k| &k.identity)
                                    != identity.as_ref())
                                .then(|| {
                                    "현재 계좌·프로파일·투자 모드와 원래 주문이 다릅니다.".into()
                                })
                            })
                    } else {
                        Some("기존 예약에 복구 스냅샷이 없어 보류합니다.".into())
                    };
                    push_bounded(
                        &mut items,
                        BudgetRecoveryItem {
                            reservation_id: id.clone(),
                            currency,
                            market: p
                                .and_then(|p| p.ownership_key.as_ref())
                                .map(|key| key.market),
                            exchange: p.and_then(|p| p.exchange.clone()),
                            symbol: r.symbol.clone(),
                            side: if r.buy {
                                OrderSide::Buy
                            } else {
                                OrderSide::Sell
                            },
                            quantity: r.quantity,
                            order_price: p.map_or(r.price, |p| p.order_price),
                            filled_quantity: r.filled,
                            filled_notional: r.notional,
                            reserved_amount: r.reserved,
                            submitted_at: p.map(|p| p.record.timestamp.clone()),
                            profile_id: p
                                .and_then(|p| p.ownership_key.as_ref())
                                .and_then(|k| k.identity.profile_id.clone()),
                            is_paper: p
                                .and_then(|p| p.ownership_key.as_ref())
                                .map(|k| k.identity.is_paper),
                            client_order_id: p.and_then(|p| p.client_order_id.clone()),
                            provider_order_id: receipt
                                .map(|r| r.provider_order_id.clone())
                                .or_else(|| {
                                    pending.and_then(|p| p.record.provider_order_id.clone())
                                }),
                            snapshot_revision: snapshot.map(|s| s.revision.clone()),
                            evidence_reference: receipt.map(|r| r.evidence_reference.clone()),
                            state: if resolved {
                                "resolved"
                            } else if pending.is_some() {
                                "awaitingReconciliation"
                            } else {
                                "unresolved"
                            },
                            blocked_reason,
                        },
                    );
                }
            }
        }
        for (id, receipt) in &ledger.recovery_receipts {
            let p = &receipt.snapshot.pending;
            if &p.broker_scope != scope
                || ledger.accounts.iter().any(|a| {
                    a.krw.reservations.contains_key(id) || a.usd.reservations.contains_key(id)
                })
            {
                continue;
            }
            total_count += 1;
            push_bounded(
                &mut items,
                BudgetRecoveryItem {
                    reservation_id: id.clone(),
                    currency: if is_us(p) { "USD" } else { "KRW" },
                    market: p.ownership_key.as_ref().map(|key| key.market),
                    exchange: p.exchange.clone(),
                    symbol: p.record.symbol.clone(),
                    side: p.record.side,
                    quantity: p.record.quantity,
                    order_price: p.order_price,
                    filled_quantity: receipt.filled_quantity,
                    filled_notional: receipt.filled_notional,
                    reserved_amount: 0,
                    submitted_at: Some(p.record.timestamp.clone()),
                    profile_id: p
                        .ownership_key
                        .as_ref()
                        .and_then(|k| k.identity.profile_id.clone()),
                    is_paper: p.ownership_key.as_ref().map(|k| k.identity.is_paper),
                    client_order_id: p.client_order_id.clone(),
                    provider_order_id: Some(receipt.provider_order_id.clone()),
                    snapshot_revision: Some(receipt.snapshot.revision.clone()),
                    evidence_reference: Some(receipt.evidence_reference.clone()),
                    state: if receipt.resolved {
                        "resolved"
                    } else {
                        "unresolved"
                    },
                    blocked_reason: (!receipt.resolved)
                        .then(|| "승인 영수증의 예산 예약이 유실되었습니다.".into()),
                },
            );
        }
        Ok(BudgetRecoveryView {
            scope: scope.clone(),
            items,
            total_count,
        })
    }
    pub async fn approve_budget_recovery(
        &mut self,
        scope: &BrokerScope,
        input: &ApproveBudgetRecoveryInput,
    ) -> Result<BudgetRecoveryView> {
        anyhow::ensure!(
            input.confirmed
                && bounded(&input.reservation_id, 100)
                && bounded(&input.snapshot_revision, 64)
                && bounded(&input.provider_order_id, 128)
                && bounded(&input.evidence_reference, 256),
            "대조 확인과 길이가 제한된 주문번호·증거 참조가 필요합니다."
        );
        anyhow::ensure!(
            scope.broker_id == input.broker_id
                && scope
                    .account_id
                    .as_ref()
                    .is_some_and(|a| a.0 == input.broker_account_id),
            "복구 요청 계좌가 다릅니다."
        );
        anyhow::ensure!(
            self.active_execution_run.is_none()
                && self.submitting.is_empty()
                && self.modifying.is_empty(),
            "자동매매·제출·정정이 진행 중이면 복구할 수 없습니다."
        );
        let identity = self.recovery_identity(scope).await?;
        self.load_budget().await?;
        let mut ledger = self.budget.clone().expect("loaded ledger");
        let receipt = ledger.recovery_receipts.get(&input.reservation_id).cloned();
        if let Some(receipt) = &receipt {
            anyhow::ensure!(
                receipt.snapshot.pending.broker_scope == *scope
                    && receipt.snapshot.revision == input.snapshot_revision
                    && receipt.provider_order_id == input.provider_order_id
                    && receipt.evidence_reference == input.evidence_reference
                    && receipt.snapshot.pending.client_order_id == input.client_order_id
                    && receipt
                        .snapshot
                        .pending
                        .ownership_key
                        .as_ref()
                        .map(|k| &k.identity)
                        == Some(&identity),
                "이전 승인과 다른 요청입니다. 승인된 주문번호는 변경할 수 없습니다."
            );
            if receipt.resolved {
                return self.budget_recovery(scope).await;
            }
        }
        let a = ledger.account(scope);
        let r = a
            .krw
            .reservations
            .get(&input.reservation_id)
            .or_else(|| a.usd.reservations.get(&input.reservation_id))
            .ok_or_else(|| anyhow::anyhow!("복구 예약을 찾을 수 없습니다."))?;
        let snapshot = r
            .recovery_snapshot
            .clone()
            .ok_or_else(|| anyhow::anyhow!("원래 주문 스냅샷이 없는 예약은 복구할 수 없습니다."))?;
        snapshot.validate(&input.reservation_id, scope, r)?;
        anyhow::ensure!(
            snapshot.revision == input.snapshot_revision,
            "오래된 복구 화면입니다. 새로고침하세요."
        );
        anyhow::ensure!(
            snapshot.pending.ownership_key.as_ref().map(|k| &k.identity) == Some(&identity),
            "원래 주문의 프로파일·모드와 현재 계좌가 다릅니다."
        );
        anyhow::ensure!(
            snapshot.pending.client_order_id == input.client_order_id,
            "원래 client 주문 ID와 일치하지 않습니다."
        );
        anyhow::ensure!(!r.terminal, "이미 종료된 예약은 복원하지 않습니다.");
        let existing = self
            .pending
            .iter()
            .find(|(_, p)| p.record.id == input.reservation_id)
            .map(|(id, _)| id.clone());
        if let Some(id) = &existing {
            anyhow::ensure!(
                id == &input.provider_order_id && receipt.is_some(),
                "이미 다른 미체결 주문에 연결된 예약입니다."
            );
        } else {
            anyhow::ensure!(
                r.filled == 0 && r.notional == 0,
                "일부 체결 후 pending이 유실된 예약은 증거가 부족합니다."
            );
            anyhow::ensure!(
                !self.pending.contains_key(&input.provider_order_id)
                    && !self.pending.values().any(|p| p.broker_scope == *scope
                        && p.record.symbol == snapshot.pending.record.symbol),
                "주문번호 또는 종목의 기존 미체결 주문과 충돌합니다."
            );
        }
        self.validate_recovery_history(
            &snapshot.pending,
            &input.provider_order_id,
            existing.is_some(),
        )
        .await?;
        let ownership = self.load_ownership_ledger().await?;
        anyhow::ensure!(
            existing.is_some() || ownership.order_watermark(&input.reservation_id).is_none(),
            "체결 소유 영수증이 있으나 원래 pending intent가 없어 보류합니다."
        );
        anyhow::ensure!(
            snapshot.pending.ownership_generation.as_deref()
                == Some(ownership.generation().to_string().as_str()),
            "소유 원장의 원래 세대가 다릅니다."
        );
        if receipt.is_none() {
            anyhow::ensure!(
                ledger.recovery_receipts.len() < MAX_RECEIPTS,
                "복구 승인 영수증 한도(256)에 도달하여 보류합니다."
            );
            anyhow::ensure!(
                !ledger
                    .recovery_receipts
                    .values()
                    .any(|r| r.snapshot.pending.broker_scope == *scope
                        && r.provider_order_id == input.provider_order_id),
                "이미 복구에 사용한 주문번호입니다."
            );
            ledger.recovery_receipts.insert(
                input.reservation_id.clone(),
                RecoveryReceipt {
                    snapshot: snapshot.clone(),
                    provider_order_id: input.provider_order_id.clone(),
                    evidence_reference: input.evidence_reference.clone(),
                    approved_at: chrono::Utc::now().to_rfc3339(),
                    resolved: false,
                    filled_quantity: 0,
                    filled_notional: 0,
                },
            );
            // Durable attestation intent precedes pending creation, including ambiguous write errors.
            self.save_budget(ledger).await?;
        }
        if existing.is_none() {
            let mut pending = snapshot.pending;
            pending.record.provider_order_id = Some(input.provider_order_id.clone());
            pending.record.kis_order_id = Some(input.provider_order_id.clone());
            pending.provider_status = Some("operator_attested_awaiting_reconciliation".into());
            self.track_pending_order(input.provider_order_id.clone(), pending);
        }
        // Always retry this write; an earlier failure may have left only the runtime pending.
        if let Err(error) = self.persist_pending_orders().await {
            self.block_for_persistence_failure(format!("복구 미체결 저장 실패: {error}"));
            return Err(error);
        }
        self.budget_recovery(scope).await
    }
}
#[cfg(test)]
mod tests;
