use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use super::{build_daily_path, read_json_or_default, write_json};

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
    pub error_message: Option<String>,
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
            error_message: None,
        }
    }
}

/// 주문 이력 저장소
pub struct OrderStore {
    data_dir: PathBuf,
}

impl OrderStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn today_path(&self) -> PathBuf {
        build_daily_path(&self.data_dir, "orders", Local::now().date_naive(), "orders.json")
    }

    pub async fn append(&self, record: OrderRecord) -> Result<()> {
        let path = self.today_path();
        let mut records: Vec<OrderRecord> = read_json_or_default(&path).await?;
        records.push(record);
        write_json(&path, &records).await?;
        Ok(())
    }
}
