//! 주문번호 기반 provider 조회와 체결 반영 조정.

use super::{collector::*, provider_mode::validate_pending_kis_mode, *};

type PendingCandidate = (BrokerId, String, String, String);

impl OrderManager {
    /// 주문번호 기반 체결 확인.
    ///
    /// KIS 당일 체결 내역에서 pending 주문번호를 찾아 실제 체결수량/체결금액으로 반영한다.
    /// 국내는 원화 정수, 해외는 USD × 100(cents) 단위로 `on_fill()`에 전달한다.
    pub async fn confirm_pending_fills_from_broker(&mut self) -> Result<()> {
        let snapshot = ReconciliationSnapshot::capture(self).await?;
        let (pending, blocked) = self.pending_fill_candidates(snapshot.client.is_paper());
        if pending.is_empty() {
            if let Some(reason) = blocked {
                self.mark_reconciliation(Some(reason.clone()));
                return Err(anyhow::anyhow!(reason));
            }
            return Ok(());
        }
        let fills = snapshot.collect(&pending).await?;
        self.apply_pending_fills(fills).await?;
        if let Some(reason) = blocked {
            self.mark_reconciliation(Some(reason.clone()));
            return Err(anyhow::anyhow!(reason));
        }
        Ok(())
    }

    /// Snapshot the actual provider dependencies before releasing the order gate.
    /// A completed result may cross an epoch only while its account and credentials
    /// remain identical; actual fills are independent of the strategy generation.
    pub async fn confirm_pending_fills_from_broker_shared(
        order_manager: &Arc<Mutex<OrderManager>>,
    ) -> Result<()> {
        let (snapshot, pending, blocked) = {
            let mut manager = order_manager.lock().await;
            let snapshot = match ReconciliationSnapshot::capture(&manager).await {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    manager.mark_reconciliation(Some(error.to_string()));
                    return Err(error);
                }
            };
            let (pending, blocked) = manager.pending_fill_candidates(snapshot.client.is_paper());
            if pending.is_empty() {
                manager.mark_reconciliation(blocked.clone());
                return blocked.map_or(Ok(()), |reason| Err(anyhow::anyhow!(reason)));
            }
            (snapshot, pending, blocked)
        };
        let result = snapshot.collect(&pending).await;
        let mut manager = order_manager.lock().await;
        if !snapshot.matches_current(&manager).await {
            tracing::debug!("계좌·provider 변경으로 이전 체결 조회 결과를 보류합니다.");
            return Ok(());
        }
        let result = match result {
            Ok(fills) => manager.apply_pending_fills(fills).await,
            Err(error) => Err(error),
        };
        let result =
            result.and_then(|()| blocked.map_or(Ok(()), |reason| Err(anyhow::anyhow!(reason))));
        manager.mark_reconciliation(result.as_ref().err().map(ToString::to_string));
        result
    }

    fn pending_fill_candidates(&self, is_paper: bool) -> (Vec<PendingCandidate>, Option<String>) {
        let mut blocked = None;
        let pending = self
            .pending
            .iter()
            .filter(|(odno, order)| {
                order.broker_scope == self.execution_scope && !self.modifying.contains(*odno)
            })
            .filter_map(|(odno, order)| {
                if let Err(error) = validate_pending_kis_mode(order, is_paper) {
                    if blocked.is_none() {
                        blocked = Some(error.to_string());
                    }
                    return None;
                }
                let is_overseas =
                    order.exchange.is_some() || !is_domestic_symbol(&order.record.symbol);
                Some((
                    pending_order_provider(order),
                    odno.clone(),
                    order.record.symbol.clone(),
                    chrono::DateTime::parse_from_rfc3339(&order.record.timestamp)
                        .map(|timestamp| {
                            if is_overseas {
                                timestamp
                                    .with_timezone(&chrono_tz::America::New_York)
                                    .format("%Y%m%d")
                                    .to_string()
                            } else {
                                timestamp.format("%Y%m%d").to_string()
                            }
                        })
                        .unwrap_or_else(|_| chrono::Local::now().format("%Y%m%d").to_string()),
                ))
            })
            .collect();
        (pending, blocked)
    }

    async fn apply_pending_fills(&mut self, fills: Vec<PendingFill>) -> Result<()> {
        for fill in fills {
            if !self
                .pending
                .get(&fill.odno)
                .is_some_and(|pending| pending.broker_scope == self.execution_scope)
            {
                // Includes zero-fill terminal notifications: never release another
                // account's budget or rewrite its pending lifecycle here.
                continue;
            }
            if let Some(pending) = self.pending.get(&fill.odno) {
                if let Err(error) = self.validate_pending_fill_mode(pending).await {
                    self.mark_reconciliation(Some(error.to_string()));
                    return Err(error);
                }
            }
            if fill.filled_qty > 0 {
                self.on_fill_on(
                    &fill.odno,
                    fill.filled_qty,
                    fill.avg_price,
                    fill.execution_date,
                )
                .await?;
            }
            if let Some(status) = fill.terminal_status {
                self.finalize_pending_order(&fill.odno, status, &fill.provider_status, None)
                    .await?;
            }
        }
        Ok(())
    }
}

// Credential-bearing snapshots intentionally have no Debug implementation.
struct ReconciliationSnapshot {
    scope: crate::broker::BrokerScope,
    client: Arc<KisRestClient>,
    profile: Option<AccountProfile>,
}

impl ReconciliationSnapshot {
    async fn capture(manager: &OrderManager) -> Result<Self> {
        let scope = manager.execution_scope.clone();
        let (client, profile) = if let Some(context) = manager.execution_run_context() {
            anyhow::ensure!(
                context.broker_scope == scope,
                "체결 조회 실행 계좌가 변경되었습니다."
            );
            manager
                .execution_run_snapshots(&context)
                .ok_or_else(|| anyhow::anyhow!("체결 조회 실행 snapshot을 확인할 수 없습니다."))?
        } else {
            // Stopped runs still reconcile the selected account, using its exact
            // active profile rather than the first profile sharing the account.
            let client = manager.rest_client.read().await.clone();
            let profile = manager.profiles.read().await.get_active().cloned();
            (client, profile)
        };
        super::super::execution_run::validate_snapshot(
            &scope,
            profile.as_ref().map(|profile| profile.id.as_str()),
            &client,
            profile.as_ref(),
        )?;
        Ok(Self {
            scope,
            client,
            profile,
        })
    }

    async fn matches_current(&self, manager: &OrderManager) -> bool {
        if manager.execution_scope != self.scope {
            return false;
        }
        let Ok(current) = Self::capture(manager).await else {
            return false;
        };
        if !same_profile(self.profile.as_ref(), current.profile.as_ref()) {
            return false;
        }
        if self.scope.broker_id != BrokerId::Kis {
            return true;
        }
        if Arc::ptr_eq(&self.client, &current.client) {
            return true;
        }
        // Recreating an identical KIS client is safe. A credential/mode change
        // defers this response and obtains a fresh result on the next poll.
        let Some(account) = self.scope.account_id.as_ref() else {
            return false;
        };
        current.client.matches_account(
            self.client.app_key(),
            self.client.app_secret(),
            &account.0,
            self.client.is_paper(),
        )
    }

    async fn collect(
        &self,
        pending: &[(BrokerId, String, String, String)],
    ) -> Result<Vec<PendingFill>> {
        let mut fills =
            collect_kis_pending_fills(Arc::clone(&self.client), &kis_pending_from(pending)).await?;
        fills.extend(
            collect_toss_pending_fills(self.profile.clone(), &toss_pending_from(pending)).await?,
        );
        Ok(fills)
    }
}

fn same_profile(left: Option<&AccountProfile>, right: Option<&AccountProfile>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            left.id == right.id
                && left.broker_id == right.broker_id
                && left.account_no == right.account_no
                && left.app_key == right.app_key
                && left.app_secret == right.app_secret
                && left.is_paper_trading == right.is_paper_trading
                && left.live_trading_consent == right.live_trading_consent
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
