//! 실제 PostgreSQL 서버 대상 contract test.
//!
//! 접속 정보 환경 변수가 없으면 각 테스트는 skip(즉시 통과)한다. 실행 예시(macOS/zsh):
//!
//! ```bash
//! KISAT_PG_HOST=db.example.com KISAT_PG_PORT=5432 \
//! KISAT_PG_USER=user KISAT_PG_PASSWORD=secret \
//!   cargo test --lib storage::database_contract_tests -- --nocapture
//! ```
//!
//! - 사용자의 운영 database는 건드리지 않는다. 매 실행 고유한 `kisautotrade_ct_*`
//!   database를 미존재-자동생성 경로로 만들고 테스트 후 DROP한다.
//! - MariaDB contract test는 아직 없다 (todo.md P1 잔여 — 검증 서버 준비 후 추가).

use std::path::PathBuf;
use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

use super::database::{
    DatabaseManager, DatabaseProvider, DatabaseTlsMode, SaveDatabaseConfigInput, StorageBackend,
};

struct PgTestEnv {
    host: String,
    port: u16,
    username: String,
    password: String,
}

/// 접속 env가 모두 있어야 실행한다. 없으면 None → 테스트 skip.
fn pg_test_env() -> Option<PgTestEnv> {
    let host = std::env::var("KISAT_PG_HOST").ok()?;
    let port = std::env::var("KISAT_PG_PORT").ok()?.parse().ok()?;
    let username = std::env::var("KISAT_PG_USER").ok()?;
    let password = std::env::var("KISAT_PG_PASSWORD").ok()?;
    Some(PgTestEnv {
        host,
        port,
        username,
        password,
    })
}

fn unique_test_database() -> String {
    format!(
        "kisautotrade_ct_{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    )
}

fn temp_dirs() -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("kisat-db-contract-{}", uuid::Uuid::new_v4()));
    (base.join("database_config.json"), base.join("data"))
}

fn save_input(env: &PgTestEnv, database: &str) -> SaveDatabaseConfigInput {
    SaveDatabaseConfigInput {
        provider: DatabaseProvider::Postgresql,
        host: env.host.clone(),
        port: env.port,
        database: database.to_string(),
        username: env.username.clone(),
        password: Some(env.password.clone()),
        tls_mode: DatabaseTlsMode::Prefer,
        max_connections: 2,
    }
}

/// 테스트 database를 관리자 DB(postgres) 경유로 정리한다. 실패해도 테스트를 깨지 않는다.
async fn drop_test_database(env: &PgTestEnv, database: &str) {
    let options = PgConnectOptions::new()
        .host(&env.host)
        .port(env.port)
        .database("postgres")
        .username(&env.username)
        .password(&env.password)
        .ssl_mode(PgSslMode::Prefer);
    let pool = match PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("contract test DB 정리 스킵 (admin 접속 실패): {error}");
            return;
        }
    };
    // PostgreSQL 13+: WITH (FORCE)로 잔여 연결을 끊고 삭제. 미지원이면 일반 DROP 재시도.
    let force = format!("DROP DATABASE IF EXISTS \"{database}\" WITH (FORCE)");
    if sqlx::query(&force).execute(&pool).await.is_err() {
        let plain = format!("DROP DATABASE IF EXISTS \"{database}\"");
        if let Err(error) = sqlx::query(&plain).execute(&pool).await {
            eprintln!("contract test DB 정리 실패 — 수동 삭제 필요: {database} ({error})");
        }
    }
    pool.close().await;
}

async fn write_test_document(data_dir: &PathBuf, relative: &str, payload: &str) {
    let path = data_dir.join(relative);
    tokio::fs::create_dir_all(path.parent().expect("parent"))
        .await
        .expect("create doc dir");
    tokio::fs::write(&path, payload).await.expect("write doc");
}

#[tokio::test]
async fn postgresql_full_roundtrip_contract() {
    let Some(env) = pg_test_env() else {
        eprintln!("KISAT_PG_* env 미설정 — PostgreSQL contract test skip");
        return;
    };
    super::database_keychain::use_mock_keychain_for_tests();
    let _keychain = super::database_keychain::keychain_test_lock();
    let database = unique_test_database();
    let (config_path, data_dir) = temp_dirs();
    tokio::fs::create_dir_all(&data_dir)
        .await
        .expect("data dir");

    let manager =
        DatabaseManager::load_sync(config_path.clone(), data_dir.clone()).expect("manager load");
    manager
        .save_config(save_input(&env, &database))
        .await
        .expect("save config");

    // ① 미존재 database 자동 생성 + 연결: 고유 이름이므로 항상 auto-create 경로를 지난다.
    let status = manager.status().await.expect("status after auto-create");
    assert!(status.connected);
    assert!(status.server_version.is_some());
    assert!(status.tables.iter().all(|table| !table.exists));

    // ② 앱 테이블 생성 + 스키마 버전
    let status = manager.create_tables().await.expect("create tables");
    assert!(status.tables.iter().all(|table| table.exists));
    assert_eq!(status.schema_version, Some(status.required_schema_version));

    // ③ JSON → DB 가져오기 (한글 payload 포함 — utf8 왕복 검증)
    let doc_a = r#"{"symbol":"005930","name":"삼성전자","qty":10}"#;
    let doc_b = r#"[{"date":"2026-07-12","pnl":-12000}]"#;
    write_test_document(&data_dir, "positions/current.json", doc_a).await;
    write_test_document(&data_dir, "trades/2026/07/12/trades.json", doc_b).await;
    let import = manager.import_json().await.expect("import json");
    assert_eq!(import.processed, 2);
    let status = manager.status().await.expect("status after import");
    let documents_table = status
        .tables
        .iter()
        .find(|table| table.name == "kisautotrade_documents")
        .expect("documents table");
    assert_eq!(documents_table.row_count, 2);
    assert_eq!(
        status
            .tables
            .iter()
            .find(|table| table.name == "kisautotrade_positions")
            .expect("positions projection")
            .row_count,
        1
    );
    assert_eq!(
        status
            .tables
            .iter()
            .find(|table| table.name == "kisautotrade_fills")
            .expect("fills projection")
            .row_count,
        1
    );

    // ④ DB → JSON 스냅샷 반출: 원본 payload와 checksum 왕복 일치
    let export = manager.export_json().await.expect("export json");
    assert_eq!(export.processed, 2);
    assert_eq!(export.checksum, import.checksum);
    let output = PathBuf::from(export.output_path.expect("output path"));
    let exported = tokio::fs::read_to_string(output.join("positions/current.json"))
        .await
        .expect("exported doc");
    assert_eq!(exported, doc_a);

    // ⑤ DB backend 전환 후 문서 read/write가 DB를 통해 왕복
    manager
        .set_backend(StorageBackend::Database)
        .await
        .expect("switch to database backend");
    let doc_path = data_dir.join("positions/current.json");
    let updated = r#"{"symbol":"005930","name":"삼성전자","qty":7}"#;
    manager
        .write_document(&doc_path, updated)
        .await
        .expect("write through db backend");
    let read_back = manager
        .read_document(&doc_path)
        .await
        .expect("read through db backend");
    assert_eq!(read_back.as_deref(), Some(updated));

    let order_path = data_dir.join("orders/2026/07/12/orders.json");
    let order_payload = r#"[{"id":"order-1","broker_id":"kis","broker_account_id":"12345678-01","provider_order_id":"provider-1","symbol":"005930","status":"partially_filled"}]"#;
    manager
        .write_document(&order_path, order_payload)
        .await
        .expect("normalized order projection");
    let status = manager.status().await.expect("projection status");
    assert_eq!(
        status
            .tables
            .iter()
            .find(|table| table.name == "kisautotrade_orders")
            .unwrap()
            .row_count,
        1
    );

    // document upsert와 projection은 같은 transaction이다. projection constraint 실패 시 문서도 없어야 한다.
    let invalid_path = data_dir.join("orders/2026/07/12/invalid-orders.json");
    let invalid_payload = format!(
        r#"[{{"id":"{}","symbol":"005930","status":"pending"}}]"#,
        "x".repeat(256)
    );
    assert!(manager
        .write_document(&invalid_path, &invalid_payload)
        .await
        .is_err());
    assert!(manager
        .read_document(&invalid_path)
        .await
        .expect("rollback read")
        .is_none());

    let old_trade_path = data_dir.join("trades/2020/01/01/trades.json");
    let old_trade_payload = r#"[{"id":"old-fill","broker_id":"kis","broker_account_id":"12345678-01","provider_order_id":"old-order","symbol":"005930","execution_date":"2020-01-01"}]"#;
    manager
        .write_document(&old_trade_path, old_trade_payload)
        .await
        .expect("old trade projection");
    assert!(
        manager
            .database_archive_stats()
            .await
            .expect("archive stats")
            .total_files
            >= 2
    );
    manager
        .purge_database_trades(90, 500)
        .await
        .expect("database retention");
    assert!(manager
        .read_document(&old_trade_path)
        .await
        .expect("purged trade read")
        .is_none());
    let status = manager.status().await.expect("retention projection status");
    assert_eq!(
        status
            .tables
            .iter()
            .find(|table| table.name == "kisautotrade_fills")
            .unwrap()
            .row_count,
        1
    );
    write_test_document(
        &data_dir,
        "trades/2020/01/01/trades.json",
        old_trade_payload,
    )
    .await;

    // DB backend 활성 상태로 manager를 재생성해 pending/partial/risk 문서 복원을 검증한다.
    let pending_path = data_dir.join("orders/pending.json");
    let risk_path = data_dir.join("risk/runtime.json");
    let pending_payload = r#"[{"providerStatus":"partially_filled","filledQuantity":2}]"#;
    let risk_payload = r#"{"date":"2026-07-12","emergencyStop":true}"#;
    manager
        .write_document(&pending_path, pending_payload)
        .await
        .expect("pending write");
    manager
        .write_document(&risk_path, risk_payload)
        .await
        .expect("risk write");
    let reloaded =
        DatabaseManager::load_sync(config_path, data_dir.clone()).expect("manager reload");
    assert_eq!(
        reloaded
            .read_document(&pending_path)
            .await
            .expect("pending reload")
            .as_deref(),
        Some(pending_payload)
    );
    assert_eq!(
        reloaded
            .read_document(&risk_path)
            .await
            .expect("risk reload")
            .as_deref(),
        Some(risk_payload)
    );

    // ⑥ JSON backend 복귀 시 DB 문서가 로컬 파일로 복원
    manager
        .set_backend(StorageBackend::Json)
        .await
        .expect("switch back to json backend");
    assert!(!old_trade_path.exists(), "DB에서 purge된 local trade가 부활하면 안 된다");
    let restored = tokio::fs::read_to_string(&doc_path)
        .await
        .expect("restored local doc");
    assert_eq!(restored, updated);

    // ⑦ 데이터 비우기 / 테이블 삭제 (확인 문구 필수)
    let status = manager
        .clear_tables("CLEAR KISAUTOTRADE DATA")
        .await
        .expect("clear tables");
    let documents_table = status
        .tables
        .iter()
        .find(|table| table.name == "kisautotrade_documents")
        .expect("documents table");
    assert_eq!(documents_table.row_count, 0);
    let status = manager
        .drop_tables("DROP KISAUTOTRADE TABLES")
        .await
        .expect("drop tables");
    assert!(status.tables.iter().all(|table| !table.exists));

    drop(manager);
    drop_test_database(&env, &database).await;
}

#[tokio::test]
async fn postgresql_rejects_invalid_credentials() {
    let Some(env) = pg_test_env() else {
        eprintln!("KISAT_PG_* env 미설정 — PostgreSQL contract test skip");
        return;
    };
    super::database_keychain::use_mock_keychain_for_tests();
    let _keychain = super::database_keychain::keychain_test_lock();
    let (config_path, data_dir) = temp_dirs();
    tokio::fs::create_dir_all(&data_dir)
        .await
        .expect("data dir");
    let manager = DatabaseManager::load_sync(config_path, data_dir.clone()).expect("manager load");
    let mut input = save_input(&env, "kisautotrade_ct_badcred");
    input.password = Some("invalid-password-for-contract-test".to_string());
    manager.save_config(input).await.expect("save config");

    let error = manager
        .status()
        .await
        .expect_err("잘못된 자격증명으로 연결이 성공하면 안 된다");
    assert!(
        error.to_string().contains("PostgreSQL 연결 실패")
            || error.to_string().contains("자동 생성"),
        "예상치 못한 오류 유형: {error:#}"
    );
}

#[tokio::test]
async fn postgresql_destructive_operations_require_exact_confirmation() {
    let Some(env) = pg_test_env() else {
        eprintln!("KISAT_PG_* env 미설정 — PostgreSQL contract test skip");
        return;
    };
    super::database_keychain::use_mock_keychain_for_tests();
    let _keychain = super::database_keychain::keychain_test_lock();
    let (config_path, data_dir) = temp_dirs();
    tokio::fs::create_dir_all(&data_dir)
        .await
        .expect("data dir");
    let manager = DatabaseManager::load_sync(config_path, data_dir.clone()).expect("manager load");
    manager
        .save_config(save_input(&env, "kisautotrade_ct_confirm"))
        .await
        .expect("save config");

    // 확인 문구가 틀리면 서버 연결 전에 거부된다 (database는 생성되지 않는다).
    assert!(manager.clear_tables("wrong phrase").await.is_err());
    assert!(manager.drop_tables("DROP EVERYTHING").await.is_err());
}
