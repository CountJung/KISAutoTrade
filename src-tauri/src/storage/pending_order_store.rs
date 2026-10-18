use std::path::PathBuf;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::trading::order::PendingOrder;

use super::write_json;

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

    pub(crate) fn budget_path(&self) -> PathBuf {
        self.path.with_file_name("auto_trading_budget.json")
    }

    pub(crate) fn ownership_path(&self) -> PathBuf {
        self.path.with_file_name("position_ownership.json")
    }

    pub(crate) fn ownership_manifest_path(&self) -> PathBuf {
        self.path.with_file_name("position_ownership_manifest.json")
    }

    pub async fn load(&self) -> Result<Vec<PendingOrder>> {
        // A backup can predate an applied fill/terminal removal. Never resurrect
        // those orders after ownership receipts have been compacted.
        let _read = self.write_lock.lock().await;
        match super::database::read_managed_json(&self.path).await? {
            Some(content) => Ok(serde_json::from_str(&content)?),
            None => Ok(Vec::new()),
        }
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

    fn snapshot() -> PendingOrder {
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
        PendingOrder {
            ownership_key: None,
            ownership_generation: None,
            strategy_position_metadata: None,
            submitted_cost_basis: None,
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
            confirmed_filled_notional: None,
            application_started: true,
            application_execution_date: None,
            application_exchange_rate: None,
            application_pnl: Some(0),
            client_order_id: Some("client-1".into()),
            provider_status: Some("PARTIALLY_FILLED".into()),
        }
    }

    #[tokio::test]
    async fn restart_snapshot_preserves_scope_client_id_and_fill_watermark() {
        let data_dir =
            std::env::temp_dir().join(format!("kis-pending-store-{}", uuid::Uuid::new_v4()));
        let store = PendingOrderStore::new(data_dir.clone());
        let pending = snapshot();
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

    #[tokio::test]
    async fn corrupted_primary_never_resurrects_orders_from_a_valid_backup() {
        let data_dir =
            std::env::temp_dir().join(format!("kis-pending-corrupt-{}", uuid::Uuid::new_v4()));
        let store = PendingOrderStore::new(data_dir.clone());
        let pending = snapshot();
        store.replace(std::slice::from_ref(&pending)).await.unwrap();
        store.replace(&[]).await.unwrap();
        let backup = store.path.with_file_name("pending_orders.json.bak");
        let old: Vec<PendingOrder> =
            serde_json::from_str(&tokio::fs::read_to_string(&backup).await.unwrap()).unwrap();
        assert_eq!(old.len(), 1);
        tokio::fs::write(&store.path, b"{ corrupt primary")
            .await
            .unwrap();
        assert!(store.load().await.is_err());
        assert_eq!(
            tokio::fs::read_to_string(&store.path).await.unwrap(),
            "{ corrupt primary"
        );
        assert!(backup.is_file());
        let _ = tokio::fs::remove_dir_all(data_dir).await;
    }
}
