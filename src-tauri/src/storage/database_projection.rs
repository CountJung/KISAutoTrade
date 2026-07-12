use std::collections::BTreeSet;

use anyhow::{Context, Result};
use serde_json::Value;
use sqlx::{MySql, Postgres, Transaction};

use super::database::DatabasePool;

#[derive(Debug)]
struct RowData {
    id: String,
    source_key: String,
    broker: String,
    account: String,
    provider_order_id: Option<String>,
    symbol: String,
    status_or_time: String,
    payload: String,
}

pub async fn rebuild(pool: &DatabasePool, documents: &[(&str, &str)]) -> Result<()> {
    match pool {
        DatabasePool::Postgresql(pool) => {
            let mut tx = pool.begin().await?;
            rebuild_postgres(&mut tx, documents).await?;
            tx.commit().await?;
        }
        DatabasePool::Mariadb(pool) => {
            let mut tx = pool.begin().await?;
            rebuild_mariadb(&mut tx, documents).await?;
            tx.commit().await?;
        }
    }
    Ok(())
}

pub async fn rebuild_postgres(
    tx: &mut Transaction<'_, Postgres>,
    documents: &[(&str, &str)],
) -> Result<()> {
    clear_postgres(tx).await?;
    for (key, payload) in documents {
        project_postgres(tx, key, payload).await?;
    }
    Ok(())
}

pub async fn rebuild_mariadb(
    tx: &mut Transaction<'_, MySql>,
    documents: &[(&str, &str)],
) -> Result<()> {
    clear_mariadb(tx).await?;
    for (key, payload) in documents {
        project_mariadb(tx, key, payload).await?;
    }
    Ok(())
}

pub async fn project_postgres(
    tx: &mut Transaction<'_, Postgres>,
    key: &str,
    payload: &str,
) -> Result<()> {
    delete_source_postgres(tx, key).await?;
    let projection = projection(key, payload)?;
    for row in projection.orders {
        sqlx::query("INSERT INTO kisautotrade_orders (record_id, source_key, broker_id, account_id, provider_order_id, symbol, status, payload, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (source_key,record_id) DO UPDATE SET broker_id=EXCLUDED.broker_id, account_id=EXCLUDED.account_id, provider_order_id=EXCLUDED.provider_order_id, symbol=EXCLUDED.symbol, status=EXCLUDED.status, payload=EXCLUDED.payload, updated_at=EXCLUDED.updated_at")
            .bind(row.id).bind(row.source_key).bind(row.broker).bind(row.account).bind(row.provider_order_id).bind(row.symbol).bind(row.status_or_time).bind(row.payload).bind(chrono::Utc::now().to_rfc3339()).execute(&mut **tx).await?;
    }
    for row in projection.fills {
        sqlx::query("INSERT INTO kisautotrade_fills (fill_id, source_key, broker_id, account_id, provider_order_id, symbol, executed_at, payload) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (fill_id) DO UPDATE SET source_key=EXCLUDED.source_key, broker_id=EXCLUDED.broker_id, account_id=EXCLUDED.account_id, provider_order_id=EXCLUDED.provider_order_id, symbol=EXCLUDED.symbol, executed_at=EXCLUDED.executed_at, payload=EXCLUDED.payload")
            .bind(row.id).bind(row.source_key).bind(row.broker).bind(row.account).bind(row.provider_order_id).bind(row.symbol).bind(row.status_or_time).bind(row.payload).execute(&mut **tx).await?;
    }
    for row in projection.positions {
        sqlx::query("INSERT INTO kisautotrade_positions (broker_id, account_id, market, symbol, source_key, payload, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (broker_id,account_id,market,symbol) DO UPDATE SET source_key=EXCLUDED.source_key,payload=EXCLUDED.payload,updated_at=EXCLUDED.updated_at")
            .bind(row.broker).bind(row.account).bind(row.status_or_time).bind(row.symbol).bind(row.source_key).bind(row.payload).bind(chrono::Utc::now().to_rfc3339()).execute(&mut **tx).await?;
    }
    for row in projection.risk {
        sqlx::query("INSERT INTO kisautotrade_risk_runtime (broker_id, account_id, state_date, source_key, payload, updated_at) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (broker_id,account_id,state_date) DO UPDATE SET source_key=EXCLUDED.source_key,payload=EXCLUDED.payload,updated_at=EXCLUDED.updated_at")
            .bind(row.broker).bind(row.account).bind(row.status_or_time).bind(row.source_key).bind(row.payload).bind(chrono::Utc::now().to_rfc3339()).execute(&mut **tx).await?;
    }
    Ok(())
}

pub async fn project_mariadb(
    tx: &mut Transaction<'_, MySql>,
    key: &str,
    payload: &str,
) -> Result<()> {
    delete_source_mariadb(tx, key).await?;
    let projection = projection(key, payload)?;
    for row in projection.orders {
        sqlx::query("INSERT INTO kisautotrade_orders (record_id, source_key, broker_id, account_id, provider_order_id, symbol, status, payload, updated_at) VALUES (?,?,?,?,?,?,?,?,?) ON DUPLICATE KEY UPDATE source_key=VALUES(source_key),broker_id=VALUES(broker_id),account_id=VALUES(account_id),provider_order_id=VALUES(provider_order_id),symbol=VALUES(symbol),status=VALUES(status),payload=VALUES(payload),updated_at=VALUES(updated_at)")
            .bind(row.id).bind(row.source_key).bind(row.broker).bind(row.account).bind(row.provider_order_id).bind(row.symbol).bind(row.status_or_time).bind(row.payload).bind(chrono::Utc::now().to_rfc3339()).execute(&mut **tx).await?;
    }
    for row in projection.fills {
        sqlx::query("INSERT INTO kisautotrade_fills (fill_id, source_key, broker_id, account_id, provider_order_id, symbol, executed_at, payload) VALUES (?,?,?,?,?,?,?,?) ON DUPLICATE KEY UPDATE source_key=VALUES(source_key),broker_id=VALUES(broker_id),account_id=VALUES(account_id),provider_order_id=VALUES(provider_order_id),symbol=VALUES(symbol),executed_at=VALUES(executed_at),payload=VALUES(payload)")
            .bind(row.id).bind(row.source_key).bind(row.broker).bind(row.account).bind(row.provider_order_id).bind(row.symbol).bind(row.status_or_time).bind(row.payload).execute(&mut **tx).await?;
    }
    for row in projection.positions {
        sqlx::query("INSERT INTO kisautotrade_positions (broker_id, account_id, market, symbol, source_key, payload, updated_at) VALUES (?,?,?,?,?,?,?) ON DUPLICATE KEY UPDATE source_key=VALUES(source_key),payload=VALUES(payload),updated_at=VALUES(updated_at)")
            .bind(row.broker).bind(row.account).bind(row.status_or_time).bind(row.symbol).bind(row.source_key).bind(row.payload).bind(chrono::Utc::now().to_rfc3339()).execute(&mut **tx).await?;
    }
    for row in projection.risk {
        sqlx::query("INSERT INTO kisautotrade_risk_runtime (broker_id, account_id, state_date, source_key, payload, updated_at) VALUES (?,?,?,?,?,?) ON DUPLICATE KEY UPDATE source_key=VALUES(source_key),payload=VALUES(payload),updated_at=VALUES(updated_at)")
            .bind(row.broker).bind(row.account).bind(row.status_or_time).bind(row.source_key).bind(row.payload).bind(chrono::Utc::now().to_rfc3339()).execute(&mut **tx).await?;
    }
    Ok(())
}

#[derive(Default)]
struct Projection {
    orders: Vec<RowData>,
    fills: Vec<RowData>,
    positions: Vec<RowData>,
    risk: Vec<RowData>,
}

fn projection(key: &str, payload: &str) -> Result<Projection> {
    let value: Value = serde_json::from_str(payload)
        .with_context(|| format!("projection JSON 파싱 실패: {key}"))?;
    let mut result = Projection::default();
    if key.starts_with("orders/") && (key.ends_with("orders.json") || key.ends_with("pending.json"))
    {
        for item in value.as_array().into_iter().flatten() {
            let record = item.get("record").unwrap_or(item);
            result.orders.push(row(record, key, "pending"));
        }
    } else if key.starts_with("trades/") {
        for item in value.as_array().into_iter().flatten() {
            let mut data = row(item, key, "");
            data.status_or_time = text(item, &["executionDate", "execution_date", "timestamp"])
                .unwrap_or_default();
            result.fills.push(data);
        }
    } else if key.starts_with("positions/") || key.starts_with("balance/") {
        let position_value = if key.starts_with("balance/") {
            value.get("holdings").unwrap_or(&Value::Null)
        } else {
            &value
        };
        let items: Vec<&Value> = match position_value.as_array() {
            Some(items) => items.iter().collect(),
            None if position_value.is_object() => vec![position_value],
            None => Vec::new(),
        };
        for item in items {
            let mut data = row(item, key, "domestic");
            data.status_or_time = text(item, &["market"]).unwrap_or_else(|| "domestic".into());
            result.positions.push(data);
        }
    } else if key == "risk/runtime.json" {
        let state_date = text(&value, &["date"])
            .unwrap_or_else(|| chrono::Local::now().date_naive().to_string());
        let mut scopes = BTreeSet::new();
        for field in [
            "daily_order_counts",
            "dailyOrderCounts",
            "consecutive_loss_counts",
            "consecutiveLossCounts",
            "blocked_strategy_symbols",
            "blockedStrategySymbols",
        ] {
            for entry in value
                .get(field)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(scope) = entry.get("scope") {
                    scopes.insert(scope_values(scope));
                }
            }
        }
        if scopes.is_empty() {
            scopes.insert(("global".into(), String::new()));
        }
        for (broker, account) in scopes {
            result.risk.push(RowData {
                id: String::new(),
                source_key: key.into(),
                broker,
                account,
                provider_order_id: None,
                symbol: String::new(),
                status_or_time: state_date.clone(),
                payload: payload.into(),
            });
        }
    }
    Ok(result)
}

fn row(value: &Value, key: &str, fallback_status: &str) -> RowData {
    let (broker, account) = scope_values(value);
    RowData {
        id: text(value, &["id", "record_id", "order_id"])
            .unwrap_or_else(|| format!("{key}:{}", uuid::Uuid::new_v4())),
        source_key: key.into(),
        broker,
        account,
        provider_order_id: text(value, &["provider_order_id", "kis_order_id", "order_id"]),
        symbol: text(value, &["symbol", "pdno"]).unwrap_or_else(|| "unknown".into()),
        status_or_time: text(value, &["status", "execution_date", "timestamp"])
            .unwrap_or_else(|| fallback_status.into()),
        payload: value.to_string(),
    }
}

fn scope_values(value: &Value) -> (String, String) {
    let scope = value.get("scope").unwrap_or(value);
    let broker = text(scope, &["broker_id", "brokerId"]).unwrap_or_else(|| "kis".into());
    let account = text(
        scope,
        &[
            "account_id",
            "broker_account_id",
            "accountId",
            "brokerAccountId",
        ],
    )
    .unwrap_or_default();
    (broker, account)
}

fn text(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key))
        .and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            Value::Object(map) => map.get("0").and_then(Value::as_str).map(str::to_string),
            _ => None,
        })
}

async fn clear_postgres(tx: &mut Transaction<'_, Postgres>) -> Result<()> {
    for table in [
        "kisautotrade_orders",
        "kisautotrade_fills",
        "kisautotrade_positions",
        "kisautotrade_risk_runtime",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
async fn clear_mariadb(tx: &mut Transaction<'_, MySql>) -> Result<()> {
    for table in [
        "kisautotrade_orders",
        "kisautotrade_fills",
        "kisautotrade_positions",
        "kisautotrade_risk_runtime",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
async fn delete_source_postgres(tx: &mut Transaction<'_, Postgres>, key: &str) -> Result<()> {
    for table in [
        "kisautotrade_orders",
        "kisautotrade_fills",
        "kisautotrade_positions",
        "kisautotrade_risk_runtime",
    ] {
        sqlx::query(&format!("DELETE FROM {table} WHERE source_key=$1"))
            .bind(key)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
async fn delete_source_mariadb(tx: &mut Transaction<'_, MySql>, key: &str) -> Result<()> {
    for table in [
        "kisautotrade_orders",
        "kisautotrade_fills",
        "kisautotrade_positions",
        "kisautotrade_risk_runtime",
    ] {
        sqlx::query(&format!("DELETE FROM {table} WHERE source_key=?"))
            .bind(key)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn extracts_scoped_order_projection() {
        let payload = r#"[{"id":"o1","broker_id":"toss","broker_account_id":"7","provider_order_id":"p1","symbol":"AAPL","status":"filled"}]"#;
        let projection = super::projection("orders/2026/07/12/orders.json", payload).unwrap();
        assert_eq!(projection.orders[0].broker, "toss");
        assert_eq!(projection.orders[0].account, "7");
        assert_eq!(projection.orders[0].status_or_time, "filled");
    }

    #[test]
    fn fill_projection_uses_execution_time_not_status() {
        let payload = r#"[{"id":"f1","status":"filled","executionDate":"2026-07-12","symbol":"005930"}]"#;
        let projection = super::projection("trades/2026/07/12/trades.json", payload).unwrap();
        assert_eq!(projection.fills[0].status_or_time, "2026-07-12");
    }

    #[test]
    fn risk_projection_reads_camel_case_scopes() {
        let payload = r#"{"date":"2026-07-12","dailyOrderCounts":[{"scope":{"brokerId":"toss","accountId":"7"}}]}"#;
        let projection = super::projection("risk/runtime.json", payload).unwrap();
        assert_eq!(projection.risk[0].broker, "toss");
        assert_eq!(projection.risk[0].account, "7");
    }

    #[test]
    fn balance_snapshot_projects_nested_holdings() {
        let payload = r#"{"holdings":[{"symbol":"005930","quantity":3}]}"#;
        let projection = super::projection("balance/2026/07/12/snapshot.json", payload).unwrap();
        assert_eq!(projection.positions.len(), 1);
        assert_eq!(projection.positions[0].symbol, "005930");
    }
}
