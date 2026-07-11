pub mod balance_store;
pub mod database;
mod database_io;
mod database_types;
pub mod order_store;
pub mod pending_order_store;
pub mod stats_store;
pub mod stock_store;
pub mod strategy_store;
pub mod trade_store;

pub use balance_store::BalanceStore;
pub use order_store::OrderStore;
pub use pending_order_store::PendingOrderStore;
pub use stats_store::StatsStore;
pub use stock_store::StockStore;
pub use strategy_store::StrategyStore;
pub use trade_store::TradeStore;

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::fs;
use tokio::sync::Mutex;

static JSON_PATH_LOCKS: OnceLock<std::sync::Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> =
    OnceLock::new();

fn json_path_lock(path: &Path) -> Arc<Mutex<()>> {
    let locks = JSON_PATH_LOCKS.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    let mut locks = locks
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Arc::clone(
        locks
            .entry(path.to_path_buf())
            .or_insert_with(|| Arc::new(Mutex::new(()))),
    )
}

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
    let path_lock = json_path_lock(path);
    let _read = path_lock.lock().await;
    let Some(content) = database::read_managed_json(path).await? else {
        return Ok(T::default());
    };
    match serde_json::from_str(&content) {
        Ok(value) => Ok(value),
        Err(primary_error) => {
            let Some(backup) = database::read_managed_json_backup(path).await? else {
                return Err(primary_error.into());
            };
            let recovered = serde_json::from_str(&backup).map_err(|backup_error| {
                anyhow::anyhow!(
                    "현재 JSON과 백업 JSON이 모두 손상되었습니다: current={primary_error}; backup={backup_error}"
                )
            })?;
            database::quarantine_corrupt_managed_json(path).await?;
            database::write_managed_json(path, &backup).await?;
            tracing::warn!(
                "손상 JSON을 격리하고 마지막 정상 백업에서 복구했습니다: {:?}",
                path
            );
            Ok(recovered)
        }
    }
}

/// JSON 파일 쓰기
pub async fn write_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize + ?Sized,
{
    let path_lock = json_path_lock(path);
    let _write = path_lock.lock().await;
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

    #[tokio::test]
    async fn corrupt_json_recovers_from_last_durable_backup() {
        let dir = std::env::temp_dir().join(format!("kis-storage-{}", uuid::Uuid::new_v4()));
        let path = dir.join("records.json");
        write_json(&path, &vec![1_u64]).await.expect("first write");
        write_json(&path, &vec![2_u64]).await.expect("second write");
        tokio::fs::write(&path, "{broken")
            .await
            .expect("corrupt current file");

        let recovered: Vec<u64> = read_json_or_default(&path).await.expect("backup recovery");
        assert_eq!(recovered, vec![1]);
        let restored: Vec<u64> = serde_json::from_str(
            &tokio::fs::read_to_string(&path)
                .await
                .expect("restored current file"),
        )
        .expect("valid restored json");
        assert_eq!(restored, vec![1]);
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn concurrent_writes_leave_one_complete_json_document() {
        let dir = std::env::temp_dir().join(format!("kis-storage-{}", uuid::Uuid::new_v4()));
        let path = dir.join("concurrent.json");
        let mut tasks = Vec::new();
        for value in 0_u64..20 {
            let path = path.clone();
            tasks.push(tokio::spawn(async move {
                write_json(&path, &vec![value; 32]).await
            }));
        }
        for task in tasks {
            task.await.expect("writer task").expect("atomic write");
        }

        let value: Vec<u64> = read_json_or_default(&path).await.expect("valid final json");
        assert_eq!(value.len(), 32);
        assert!(value.iter().all(|item| *item == value[0]));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }
}
