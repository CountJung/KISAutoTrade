use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::{build_daily_path, read_json_or_default, write_json};
use crate::broker::{BrokerId, BrokerScope};

/// 주문 방향
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 주문 상태
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Failed,
}

/// 주문 기록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRecord {
    pub id: String,
    pub timestamp: String,
    pub symbol: String,
    pub symbol_name: String,
    pub side: OrderSide,
    pub quantity: u64,
    pub price: u64,
    pub order_type: String,
    pub status: OrderStatus,
    pub kis_order_id: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub provider_order_id: Option<String>,
    #[serde(default)]
    pub provider_request_id: Option<String>,
    #[serde(default)]
    pub provider_tr_id: Option<String>,
    pub error_message: Option<String>,
    #[serde(default)]
    pub execution_date: Option<String>,
    /// 주문이 발생한 broker. 기존 레코드는 None(scope 도입 전 KIS)으로 간주한다.
    #[serde(default)]
    pub broker_id: Option<BrokerId>,
    /// 주문이 발생한 broker 계좌 ID (KIS CANO-상품코드, Toss accountSeq)
    #[serde(default)]
    pub broker_account_id: Option<String>,
}

impl OrderRecord {
    pub fn new(
        symbol: String,
        symbol_name: String,
        side: OrderSide,
        quantity: u64,
        price: u64,
        order_type: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now().to_rfc3339(),
            symbol,
            symbol_name,
            side,
            quantity,
            price,
            order_type,
            status: OrderStatus::Pending,
            kis_order_id: None,
            provider: None,
            provider_order_id: None,
            provider_request_id: None,
            provider_tr_id: None,
            error_message: None,
            execution_date: None,
            broker_id: None,
            broker_account_id: None,
        }
    }

    pub fn with_provider_trace(
        mut self,
        provider: impl Into<String>,
        order_id: Option<String>,
        request_id: Option<String>,
        tr_id: Option<String>,
    ) -> Self {
        self.provider = Some(provider.into());
        self.provider_order_id = order_id;
        self.provider_request_id = request_id;
        self.provider_tr_id = tr_id;
        self
    }

    /// 주문이 발생한 broker/account scope를 기록한다.
    pub fn with_broker_scope(mut self, scope: &BrokerScope) -> Self {
        self.broker_id = Some(scope.broker_id);
        self.broker_account_id = scope.account_id.as_ref().map(|account| account.0.clone());
        self
    }
}

/// 주문 이력 저장소
pub struct OrderStore {
    data_dir: PathBuf,
    write_lock: Mutex<()>,
}

impl OrderStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            write_lock: Mutex::new(()),
        }
    }

    fn path_for(&self, date: chrono::NaiveDate) -> PathBuf {
        build_daily_path(&self.data_dir, "orders", date, "orders.json")
    }

    pub async fn append(&self, record: OrderRecord) -> Result<()> {
        self.append_on(Local::now().date_naive(), record).await
    }

    pub async fn append_on(&self, date: chrono::NaiveDate, record: OrderRecord) -> Result<()> {
        let _write = self.write_lock.lock().await;
        let path = self.path_for(date);
        let mut records: Vec<OrderRecord> = read_json_or_default(&path).await?;
        if records.iter().any(|existing| existing.id == record.id) {
            return Ok(());
        }
        records.push(record);
        write_json(&path, &records).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn parallel_appends_do_not_lose_order_records() {
        let data_dir =
            std::env::temp_dir().join(format!("kis-order-store-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(OrderStore::new(data_dir.clone()));
        let mut tasks = Vec::new();
        for index in 0..32_u64 {
            let store = Arc::clone(&store);
            tasks.push(tokio::spawn(async move {
                store
                    .append(OrderRecord::new(
                        format!("SYM{index}"),
                        format!("종목 {index}"),
                        OrderSide::Buy,
                        1,
                        1_000 + index,
                        "Limit".to_string(),
                    ))
                    .await
            }));
        }
        for task in tasks {
            task.await.expect("append task").expect("append succeeds");
        }

        let records: Vec<OrderRecord> =
            read_json_or_default(&store.path_for(Local::now().date_naive()))
                .await
                .expect("read records");
        assert_eq!(records.len(), 32);
        let _ = tokio::fs::remove_dir_all(data_dir).await;
    }
}
