use anyhow::Result;

use super::database::DatabasePool;

pub const SCHEMA_VERSION: i32 = 2;
pub const NORMALIZED_TABLES: [(&str, &str); 4] = [
    ("kisautotrade_orders", "broker/account 범위 주문 상태"),
    ("kisautotrade_fills", "provider 체결 ledger"),
    (
        "kisautotrade_positions",
        "broker/account/market 포지션 스냅샷",
    ),
    ("kisautotrade_risk_runtime", "scope별 일별 리스크 runtime"),
];

pub async fn create(pool: &DatabasePool) -> Result<()> {
    match pool {
        DatabasePool::Postgresql(pool) => {
            for ddl in postgres_ddl() {
                sqlx::query(ddl).execute(pool).await?;
            }
        }
        DatabasePool::Mariadb(pool) => {
            for ddl in mariadb_ddl() {
                sqlx::query(ddl).execute(pool).await?;
            }
        }
    }
    Ok(())
}

pub async fn drop(pool: &DatabasePool) -> Result<()> {
    for (table, _) in NORMALIZED_TABLES.iter().rev() {
        let query = format!("DROP TABLE IF EXISTS {table}");
        match pool {
            DatabasePool::Postgresql(pool) => {
                sqlx::query(&query).execute(pool).await?;
            }
            DatabasePool::Mariadb(pool) => {
                sqlx::query(&query).execute(pool).await?;
            }
        }
    }
    Ok(())
}

fn postgres_ddl() -> [&'static str; 4] {
    [
        "CREATE TABLE IF NOT EXISTS kisautotrade_orders (record_id VARCHAR(128) NOT NULL, source_key VARCHAR(512) NOT NULL, broker_id VARCHAR(16) NOT NULL, account_id VARCHAR(128) NOT NULL DEFAULT '', provider_order_id VARCHAR(256), symbol VARCHAR(64) NOT NULL, status VARCHAR(32) NOT NULL, payload TEXT NOT NULL, updated_at VARCHAR(64) NOT NULL, PRIMARY KEY (source_key, record_id))",
        "CREATE TABLE IF NOT EXISTS kisautotrade_fills (fill_id VARCHAR(256) PRIMARY KEY, source_key VARCHAR(512) NOT NULL, broker_id VARCHAR(16) NOT NULL, account_id VARCHAR(128) NOT NULL DEFAULT '', provider_order_id VARCHAR(256), symbol VARCHAR(64) NOT NULL, executed_at VARCHAR(64) NOT NULL, payload TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS kisautotrade_positions (broker_id VARCHAR(16) NOT NULL, account_id VARCHAR(128) NOT NULL DEFAULT '', market VARCHAR(16) NOT NULL, symbol VARCHAR(64) NOT NULL, source_key VARCHAR(512) NOT NULL, payload TEXT NOT NULL, updated_at VARCHAR(64) NOT NULL, PRIMARY KEY (broker_id, account_id, market, symbol))",
        "CREATE TABLE IF NOT EXISTS kisautotrade_risk_runtime (broker_id VARCHAR(16) NOT NULL, account_id VARCHAR(128) NOT NULL DEFAULT '', state_date VARCHAR(16) NOT NULL, source_key VARCHAR(512) NOT NULL, payload TEXT NOT NULL, updated_at VARCHAR(64) NOT NULL, PRIMARY KEY (broker_id, account_id, state_date))",
    ]
}

fn mariadb_ddl() -> [&'static str; 4] {
    [
        "CREATE TABLE IF NOT EXISTS kisautotrade_orders (record_id VARCHAR(128) NOT NULL, source_key VARCHAR(512) NOT NULL, broker_id VARCHAR(16) NOT NULL, account_id VARCHAR(128) NOT NULL DEFAULT '', provider_order_id VARCHAR(256), symbol VARCHAR(64) NOT NULL, status VARCHAR(32) NOT NULL, payload LONGTEXT NOT NULL, updated_at VARCHAR(64) NOT NULL, PRIMARY KEY (source_key, record_id)) ENGINE=InnoDB",
        "CREATE TABLE IF NOT EXISTS kisautotrade_fills (fill_id VARCHAR(256) PRIMARY KEY, source_key VARCHAR(512) NOT NULL, broker_id VARCHAR(16) NOT NULL, account_id VARCHAR(128) NOT NULL DEFAULT '', provider_order_id VARCHAR(256), symbol VARCHAR(64) NOT NULL, executed_at VARCHAR(64) NOT NULL, payload LONGTEXT NOT NULL) ENGINE=InnoDB",
        "CREATE TABLE IF NOT EXISTS kisautotrade_positions (broker_id VARCHAR(16) NOT NULL, account_id VARCHAR(128) NOT NULL DEFAULT '', market VARCHAR(16) NOT NULL, symbol VARCHAR(64) NOT NULL, source_key VARCHAR(512) NOT NULL, payload LONGTEXT NOT NULL, updated_at VARCHAR(64) NOT NULL, PRIMARY KEY (broker_id, account_id, market, symbol)) ENGINE=InnoDB",
        "CREATE TABLE IF NOT EXISTS kisautotrade_risk_runtime (broker_id VARCHAR(16) NOT NULL, account_id VARCHAR(128) NOT NULL DEFAULT '', state_date VARCHAR(16) NOT NULL, source_key VARCHAR(512) NOT NULL, payload LONGTEXT NOT NULL, updated_at VARCHAR(64) NOT NULL, PRIMARY KEY (broker_id, account_id, state_date)) ENGINE=InnoDB",
    ]
}
