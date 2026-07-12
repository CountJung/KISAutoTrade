use anyhow::Result;
use sqlx::Row;

use super::database::DatabasePool;

#[derive(Debug, Clone)]
pub struct DatabaseArchiveStats {
    pub total_files: u64,
    pub size_bytes: u64,
    pub oldest_date: Option<String>,
    pub newest_date: Option<String>,
}

pub async fn stats(pool: &DatabasePool) -> Result<DatabaseArchiveStats> {
    let rows = trade_rows(pool).await?;
    let dates: Vec<_> = rows.iter().filter_map(|(key, _)| trade_date(key)).collect();
    Ok(DatabaseArchiveStats {
        total_files: rows.len() as u64,
        size_bytes: rows.iter().map(|(_, size)| *size).sum(),
        oldest_date: dates.iter().min().map(ToString::to_string),
        newest_date: dates.iter().max().map(ToString::to_string),
    })
}

pub async fn purge(pool: &DatabasePool, retention_days: u32, max_size_mb: u64) -> Result<()> {
    let cutoff = chrono::Local::now().date_naive() - chrono::Duration::days(retention_days as i64);
    let mut rows = trade_rows(pool).await?;
    rows.sort_by_key(|(key, _)| trade_date(key));
    let mut delete = Vec::new();
    let mut retained = Vec::new();
    for row in rows {
        if trade_date(&row.0).is_some_and(|date| date < cutoff) {
            delete.push(row.0);
        } else {
            retained.push(row);
        }
    }
    let max_bytes = max_size_mb.saturating_mul(1024 * 1024);
    let mut total: u64 = retained.iter().map(|(_, size)| *size).sum();
    for (key, size) in retained {
        if total <= max_bytes {
            break;
        }
        delete.push(key);
        total = total.saturating_sub(size);
    }
    match pool {
        DatabasePool::Postgresql(pool) => {
            let mut tx = pool.begin().await?;
            for key in delete {
                sqlx::query("DELETE FROM kisautotrade_fills WHERE source_key = $1")
                    .bind(&key)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DELETE FROM kisautotrade_documents WHERE document_key = $1")
                    .bind(key)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;
        }
        DatabasePool::Mariadb(pool) => {
            let mut tx = pool.begin().await?;
            for key in delete {
                sqlx::query("DELETE FROM kisautotrade_fills WHERE source_key = ?")
                    .bind(&key)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DELETE FROM kisautotrade_documents WHERE document_key = ?")
                    .bind(key)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;
        }
    }
    Ok(())
}

async fn trade_rows(pool: &DatabasePool) -> Result<Vec<(String, u64)>> {
    match pool {
        DatabasePool::Postgresql(pool) => Ok(sqlx::query("SELECT document_key, size_bytes FROM kisautotrade_documents WHERE document_key LIKE 'trades/%'")
            .fetch_all(pool).await?.into_iter().map(row_values).collect()),
        DatabasePool::Mariadb(pool) => Ok(sqlx::query("SELECT document_key, size_bytes FROM kisautotrade_documents WHERE document_key LIKE 'trades/%'")
            .fetch_all(pool).await?.into_iter().map(row_values).collect()),
    }
}

fn row_values<R: Row>(row: R) -> (String, u64)
where
    for<'c> String: sqlx::Decode<'c, R::Database> + sqlx::Type<R::Database>,
    for<'c> i64: sqlx::Decode<'c, R::Database> + sqlx::Type<R::Database>,
    usize: sqlx::ColumnIndex<R>,
{
    let key: String = row.get(0);
    let size: i64 = row.get(1);
    (key, size.max(0) as u64)
}

fn trade_date(key: &str) -> Option<chrono::NaiveDate> {
    let mut parts = key.split('/');
    (parts.next()? == "trades").then_some(())?;
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    chrono::NaiveDate::from_ymd_opt(year, month, day)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_trade_document_date() {
        assert_eq!(
            super::trade_date("trades/2026/07/12/trades.json")
                .unwrap()
                .to_string(),
            "2026-07-12"
        );
        assert!(super::trade_date("risk/runtime.json").is_none());
    }
}
