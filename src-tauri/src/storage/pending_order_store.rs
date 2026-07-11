use std::path::PathBuf;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::trading::order::PendingOrder;

use super::{read_json_or_default, write_json};

/// 재시작 복구용 미체결 주문 스냅샷 저장소.
pub struct PendingOrderStore {
    path: PathBuf,
    write_lock: Mutex<()>,
}

impl PendingOrderStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            path: data_dir.join("orders").join("pending_orders.json"),
            write_lock: Mutex::new(()),
        }
    }

    pub async fn load(&self) -> Result<Vec<PendingOrder>> {
        read_json_or_default(&self.path).await
    }

    pub async fn replace(&self, orders: &[PendingOrder]) -> Result<()> {
        let _write = self.write_lock.lock().await;
        write_json(&self.path, orders).await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        broker::{BrokerAccountId, BrokerId, BrokerScope},
        storage::order_store::{OrderRecord, OrderSide},
    };

    use super::*;

    #[tokio::test]
    async fn restart_snapshot_preserves_scope_client_id_and_fill_watermark() {
        let data_dir =
            std::env::temp_dir().join(format!("kis-pending-store-{}", uuid::Uuid::new_v4()));
        let store = PendingOrderStore::new(data_dir.clone());
        let mut record = OrderRecord::new(
            "AAPL".into(),
            "Apple".into(),
            OrderSide::Buy,
            10,
            20_000,
            "Limit".into(),
        );
        record.provider = Some("toss".into());
        record.provider_order_id = Some("provider-1".into());
        record.kis_order_id = Some("provider-1".into());
        let pending = PendingOrder {
            record,
            signal_reason: "test".into(),
            strategy_id: Some("strategy-1".into()),
            signal_price: 20_000,
            order_price: 20_000,
            exchange: Some("TOSS_US".into()),
            broker_scope: BrokerScope::new(
                BrokerId::Toss,
                Some(BrokerAccountId("account-1".into())),
            ),
            filled_quantity: 2,
            filled_notional: 40_000,
            confirmed_filled_quantity: 4,
            confirmed_avg_price: 20_000,
            application_started: true,
            application_pnl: Some(0),
            client_order_id: Some("client-1".into()),
            provider_status: Some("PARTIALLY_FILLED".into()),
        };
        store
            .replace(std::slice::from_ref(&pending))
            .await
            .expect("save");

        let restored = store.load().await.expect("load");
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].filled_quantity, 2);
        assert_eq!(restored[0].confirmed_filled_quantity, 4);
        assert!(restored[0].application_started);
        assert_eq!(restored[0].client_order_id.as_deref(), Some("client-1"));
        assert_eq!(
            restored[0]
                .broker_scope
                .account_id
                .as_ref()
                .map(|id| id.0.as_str()),
            Some("account-1")
        );
        let _ = tokio::fs::remove_dir_all(data_dir).await;
    }
}
