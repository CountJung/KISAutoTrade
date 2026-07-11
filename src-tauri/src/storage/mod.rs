pub mod balance_store;
pub mod database;
mod database_io;
mod database_types;
pub mod order_store;
pub mod stats_store;
pub mod stock_store;
pub mod strategy_store;
pub mod trade_store;

pub use balance_store::BalanceStore;
pub use order_store::OrderStore;
pub use stats_store::StatsStore;
pub use stock_store::StockStore;
pub use strategy_store::StrategyStore;
pub use trade_store::TradeStore;

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use std::path::{Path, PathBuf};
use tokio::fs;

/// 날짜 기반 저장 경로 생성
/// {base}/{category}/{YYYY}/{MM}/{DD}/{filename}
pub fn build_daily_path(base: &Path, category: &str, date: NaiveDate, filename: &str) -> PathBuf {
    base.join(category)
        .join(format!("{:04}", date.year()))
        .join(format!("{:02}", date.month()))
        .join(format!("{:02}", date.day()))
        .join(filename)
}

/// 월별 경로 생성 (stat용: {base}/{category}/{YYYY}/{MM}/{filename})
pub fn build_monthly_path(
    base: &Path,
    category: &str,
    year: i32,
    month: u32,
    filename: &str,
) -> PathBuf {
    base.join(category)
        .join(format!("{:04}", year))
        .join(format!("{:02}", month))
        .join(filename)
}

/// 디렉토리 생성 (없으면)
pub async fn ensure_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

/// JSON 파일 읽기 (없으면 기본값 반환)
pub async fn read_json_or_default<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    let Some(content) = database::read_managed_json(path).await? else {
        return Ok(T::default());
    };
    let value = serde_json::from_str(&content)?;
    Ok(value)
}

/// JSON 파일 쓰기
pub async fn write_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let content = serde_json::to_string_pretty(value)?;
    database::write_managed_json(path, &content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_path_uses_date_components_without_string_separators() {
        let base = PathBuf::from("app data");
        let date = NaiveDate::from_ymd_opt(2026, 7, 1).expect("valid date");
        let path = build_daily_path(&base, "trades", date, "trades.json");

        assert_eq!(
            path.file_name().and_then(|v| v.to_str()),
            Some("trades.json")
        );
        let parts: Vec<String> = path
            .components()
            .map(|part| part.as_os_str().to_string_lossy().to_string())
            .collect();
        assert!(parts.ends_with(&[
            "trades".to_string(),
            "2026".to_string(),
            "07".to_string(),
            "01".to_string(),
            "trades.json".to_string(),
        ]));
    }

    #[test]
    fn monthly_path_zero_pads_month_for_stats() {
        let base = PathBuf::from("app data");
        let path = build_monthly_path(&base, "stats", 2026, 7, "daily_stats.json");

        let parts: Vec<String> = path
            .components()
            .map(|part| part.as_os_str().to_string_lossy().to_string())
            .collect();
        assert!(parts.ends_with(&[
            "stats".to_string(),
            "2026".to_string(),
            "07".to_string(),
            "daily_stats.json".to_string(),
        ]));
    }
}
