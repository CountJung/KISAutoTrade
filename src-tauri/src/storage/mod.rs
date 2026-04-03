pub mod balance_store;
pub mod order_store;
pub mod stats_store;
pub mod trade_store;

pub use trade_store::TradeStore;
pub use order_store::OrderStore;
pub use stats_store::StatsStore;
pub use balance_store::BalanceStore;

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use std::path::{Path, PathBuf};
use tokio::fs;

/// 날짜 기반 저장 경로 생성
/// {base}/{category}/{YYYY}/{MM}/{DD}/{filename}
pub fn build_daily_path(
    base: &Path,
    category: &str,
    date: NaiveDate,
    filename: &str,
) -> PathBuf {
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
    if !path.exists() {
        return Ok(T::default());
    }
    let content = fs::read_to_string(path).await?;
    let value = serde_json::from_str(&content)?;
    Ok(value)
}

/// JSON 파일 쓰기
pub async fn write_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize,
{
    ensure_dir(path).await?;
    let content = serde_json::to_string_pretty(value)?;
    fs::write(path, content).await?;
    Ok(())
}
