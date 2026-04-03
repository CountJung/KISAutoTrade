use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{build_daily_path, write_json};

/// 보유 종목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingItem {
    pub symbol: String,
    pub symbol_name: String,
    pub quantity: u64,
    pub avg_price: u64,
    pub current_price: u64,
    pub profit_loss: i64,
    pub profit_rate: f64,
}

/// 잔고 스냅샷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub timestamp: String,
    pub total_balance: i64,
    pub available_cash: i64,
    pub stock_value: i64,
    pub total_profit_loss: i64,
    pub total_profit_rate: f64,
    pub holdings: Vec<HoldingItem>,
}

/// 잔고 스냅샷 저장소
pub struct BalanceStore {
    data_dir: PathBuf,
}

impl BalanceStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// 잔고 스냅샷 저장 (덮어쓰기)
    pub async fn save_snapshot(&self, snapshot: BalanceSnapshot) -> Result<()> {
        let path = build_daily_path(
            &self.data_dir,
            "balance",
            Local::now().date_naive(),
            "snapshot.json",
        );
        write_json(&path, &snapshot).await?;
        Ok(())
    }
}
