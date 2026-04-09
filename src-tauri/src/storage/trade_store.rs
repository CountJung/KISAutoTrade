use anyhow::Result;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use super::{build_daily_path, read_json_or_default, write_json};

/// 체결 방향
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

/// 체결 상태
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradeStatus {
    Filled,
    PartiallyFilled,
    Cancelled,
}

/// 체결 기록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: String,
    pub timestamp: String,
    pub symbol: String,
    pub symbol_name: String,
    pub side: TradeSide,
    pub quantity: u64,
    pub price: u64,
    pub total_amount: u64,
    pub fee: u64,
    pub strategy_id: Option<String>,
    pub order_id: String,
    pub status: TradeStatus,
    /// 체결 원인 — 어떤 전략 신호에 의해 매매됐는지 기록
    /// (기존 JSON 파일과의 하위 호환을 위해 default 적용)
    #[serde(default)]
    pub signal_reason: String,
}

impl TradeRecord {
    pub fn new(
        symbol: String,
        symbol_name: String,
        side: TradeSide,
        quantity: u64,
        price: u64,
        fee: u64,
        order_id: String,
        strategy_id: Option<String>,
        signal_reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now().to_rfc3339(),
            symbol,
            symbol_name,
            side,
            quantity,
            price,
            total_amount: price * quantity,
            fee,
            strategy_id,
            order_id,
            status: TradeStatus::Filled,
            signal_reason,
        }
    }
}

/// 체결 기록 저장소
pub struct TradeStore {
    data_dir: PathBuf,
}

impl TradeStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// 오늘의 체결 기록 파일 경로
    fn today_path(&self) -> PathBuf {
        build_daily_path(&self.data_dir, "trades", Local::now().date_naive(), "trades.json")
    }

    /// 체결 기록 저장
    pub async fn append(&self, record: TradeRecord) -> Result<()> {
        let path = self.today_path();
        let mut records: Vec<TradeRecord> = read_json_or_default(&path).await?;
        records.push(record);
        write_json(&path, &records).await?;
        Ok(())
    }

    /// 특정 날짜의 체결 기록 조회
    pub async fn get_by_date(&self, date: NaiveDate) -> Result<Vec<TradeRecord>> {
        let path = build_daily_path(&self.data_dir, "trades", date, "trades.json");
        read_json_or_default(&path).await
    }

    /// 날짜 범위 체결 기록 조회
    pub async fn get_by_range(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<TradeRecord>> {
        let mut all = Vec::new();
        let mut current = from;
        while current <= to {
            let mut records = self.get_by_date(current).await?;
            all.append(&mut records);
            current = current.succ_opt().unwrap_or(current);
            if current == from {
                break; // 무한루프 방지
            }
        }
        Ok(all)
    }
}
