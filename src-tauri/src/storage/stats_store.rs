use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{build_monthly_path, read_json_or_default, write_json};

/// 일별 거래 통계
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub gross_profit: i64,
    pub gross_loss: i64,
    pub net_profit: i64,
    pub fees_paid: u64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub starting_balance: i64,
    pub ending_balance: i64,
}

impl DailyStats {
    pub fn new(date: NaiveDate, starting_balance: i64) -> Self {
        Self {
            date: date.to_string(),
            starting_balance,
            ending_balance: starting_balance,
            ..Default::default()
        }
    }

    pub fn recalculate(&mut self) {
        self.net_profit = self.gross_profit + self.gross_loss - self.fees_paid as i64;
        self.ending_balance = self.starting_balance + self.net_profit;
        self.win_rate = if self.total_trades > 0 {
            self.winning_trades as f64 / self.total_trades as f64
        } else {
            0.0
        };
        self.profit_factor = if self.gross_loss != 0 {
            self.gross_profit as f64 / self.gross_loss.abs() as f64
        } else {
            0.0
        };
    }
}

/// 월별 일간 통계 저장소
/// 저장 경로: {data_dir}/stats/{YYYY}/{MM}/daily_stats.json
pub struct StatsStore {
    data_dir: PathBuf,
}

impl StatsStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn month_path(&self, year: i32, month: u32) -> PathBuf {
        build_monthly_path(&self.data_dir, "stats", year, month, "daily_stats.json")
    }

    /// 특정 날짜 통계 업데이트
    pub async fn upsert(&self, stats: DailyStats) -> Result<()> {
        let date = NaiveDate::parse_from_str(&stats.date, "%Y-%m-%d")?;
        let path = self.month_path(date.year(), date.month());

        let mut all_stats: Vec<DailyStats> = read_json_or_default(&path).await?;

        if let Some(existing) = all_stats.iter_mut().find(|s| s.date == stats.date) {
            *existing = stats;
        } else {
            all_stats.push(stats);
        }

        all_stats.sort_by(|a, b| a.date.cmp(&b.date));
        write_json(&path, &all_stats).await?;
        Ok(())
    }

    /// 오늘 통계 조회 (없으면 기본값)
    pub async fn get_today(&self) -> Result<DailyStats> {
        let today = Local::now().date_naive();
        self.get_by_date(today).await
    }

    /// 특정 날짜 통계 조회
    pub async fn get_by_date(&self, date: NaiveDate) -> Result<DailyStats> {
        let path = self.month_path(date.year(), date.month());
        let all_stats: Vec<DailyStats> = read_json_or_default(&path).await?;
        let date_str = date.to_string();
        Ok(all_stats
            .into_iter()
            .find(|s| s.date == date_str)
            .unwrap_or_else(|| DailyStats::new(date, 0)))
    }

    /// 날짜 범위 통계 조회
    pub async fn get_by_range(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<DailyStats>> {
        let mut result = Vec::new();
        let mut current = from;
        while current <= to {
            let stats = self.get_by_date(current).await?;
            result.push(stats);
            match current.succ_opt() {
                Some(next) => current = next,
                None => break,
            }
        }
        Ok(result)
    }
}
