//! 미체결 주문 복원·영속화·취소와 수정 예약.

use super::*;

impl OrderManager {
    /// 주문 취소 이벤트 처리
    pub async fn on_cancel(&mut self, odno: &str) -> Result<()> {
        self.finalize_pending_order(odno, OrderStatus::Cancelled, "cancelled", None)
            .await
    }

    pub(in crate::trading) async fn finalize_pending_order(
        &mut self,
        odno: &str,
        status: OrderStatus,
        provider_status: &str,
        error_message: Option<String>,
    ) -> Result<()> {
        let Some(pending) = self.pending.get(odno).cloned() else {
            return Ok(());
        };

        let pending_for_restore = pending.clone();
        let mut record = pending.record.clone();
        record.status = status;
        record.error_message = error_message.or_else(|| {
            matches!(
                status,
                OrderStatus::Rejected | OrderStatus::Expired | OrderStatus::Failed
            )
            .then(|| format!("provider status: {provider_status}"))
        });
        let order_date = chrono::DateTime::parse_from_rfc3339(&record.timestamp)
            .map(|timestamp| timestamp.date_naive())
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        if let Err(e) = self.order_store.upsert_on(order_date, record.clone()).await {
            tracing::error!("주문 terminal 기록 저장 실패 ({provider_status}): {e}");
            self.track_pending_order(odno.to_string(), pending_for_restore);
            let _ = self.persist_pending_orders().await;
            self.block_for_persistence_failure(format!(
                "주문 terminal 상태 영속화 실패 ({provider_status}): {e}"
            ));
            return Err(e);
        }

        self.release_budget(&pending).await?;
        self.pending.remove(odno);
        self.symbol_to_odno.remove(&pending.record.symbol);
        if let Err(error) = self.persist_pending_orders().await {
            self.track_pending_order(odno.to_string(), pending.clone());
            self.block_for_persistence_failure(format!("주문 완료 저장 실패: {error}"));
            return Err(error);
        }

        tracing::info!(
            "주문 terminal 처리: {} (odno: {}, status: {})",
            record.symbol,
            odno,
            provider_status
        );
        Ok(())
    }

    /// 미체결 주문 건수
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// 특정 종목의 미체결 odno 조회
    pub fn get_pending_odno(&self, symbol: &str) -> Option<&str> {
        self.symbol_to_odno.get(symbol).map(String::as_str)
    }

    /// 미체결 주문 목록 (UI 표시용)
    pub fn pending_orders(&self) -> Vec<&PendingOrder> {
        self.pending.values().collect()
    }

    pub fn restore_pending_orders(&mut self, orders: Vec<PendingOrder>) {
        self.pending.clear();
        self.symbol_to_odno.clear();
        for pending in orders {
            let Some(key) = pending
                .record
                .provider_order_id
                .clone()
                .or_else(|| pending.record.kis_order_id.clone())
                .filter(|value| !value.trim().is_empty())
            else {
                tracing::error!(
                    "미체결 주문 복원 제외: provider 주문번호 없음 ({})",
                    pending.record.id
                );
                continue;
            };
            self.track_pending_order(key, pending);
        }
    }

    pub async fn persist_pending_orders(&self) -> Result<()> {
        let snapshot: Vec<PendingOrder> = self.pending.values().cloned().collect();
        self.pending_order_store.replace(&snapshot).await
    }

    /// 외부 주문 경로(수동 주문 등)에서 받은 provider 주문을 미체결 풀에 편입한다.
    pub fn track_pending_order(&mut self, key: String, pending: PendingOrder) {
        self.symbol_to_odno
            .insert(pending.record.symbol.clone(), key.clone());
        self.pending.insert(key, pending);
    }

    /// provider 정정 성공 후 로컬 pending snapshot과 주문번호 key를 갱신한다.
    pub fn update_pending_order_snapshot(
        &mut self,
        order_id: &str,
        new_order_id: Option<&str>,
        quantity: Option<u64>,
        price: Option<u64>,
        order_type: Option<String>,
    ) -> bool {
        let Some(mut pending) = self.pending.remove(order_id) else {
            return false;
        };
        if let Some(quantity) = quantity {
            pending.record.quantity = quantity;
        }
        if let Some(price) = price {
            pending.record.price = price;
            pending.order_price = price;
        }
        if let Some(order_type) = order_type {
            pending.record.order_type = order_type;
        }
        let key = new_order_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(order_id)
            .to_string();
        pending.record.kis_order_id = Some(key.clone());
        pending.record.provider_order_id = Some(key.clone());
        self.symbol_to_odno
            .insert(pending.record.symbol.clone(), key.clone());
        self.pending.insert(key, pending);
        true
    }

    pub fn begin_pending_modification(&mut self, order_id: &str) -> Result<bool> {
        if !self.pending.contains_key(order_id) {
            return Ok(false);
        }
        if self
            .pending
            .get(order_id)
            .is_some_and(|p| p.record.id.starts_with("auto-budget-"))
        {
            anyhow::bail!(
                "자동매매 전용 예산 주문은 정정할 수 없습니다. 취소 후 새 주문을 사용하세요."
            );
        }
        if !self.modifying.insert(order_id.to_string()) {
            anyhow::bail!("이미 정정 또는 취소 처리 중인 주문입니다: {order_id}");
        }
        Ok(true)
    }

    pub fn finish_pending_modification(&mut self, order_id: &str) {
        self.modifying.remove(order_id);
    }

    pub(crate) fn pending_conflict_reason_for_scope(
        &self,
        broker_scope: &BrokerScope,
        symbol: &str,
        requested_side: &OrderSide,
    ) -> Option<String> {
        pending_conflict_reason_for_scope(
            self.pending.values(),
            broker_scope,
            symbol,
            requested_side,
        )
    }
}
