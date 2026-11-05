//! Runs only in the disposable, labelled localhost Docker harness.
use super::super::database::DatabasePool;
use super::*;
use std::sync::Arc;
use tokio::time::timeout;

struct Fixture {
    env: PgTestEnv,
    run_id: String,
    container: String,
    plain_port: u16,
}

impl Fixture {
    fn from_env() -> Option<Self> {
        let Ok(run_id) = std::env::var("KISAT_PG_TLS_FIXTURE") else {
            eprintln!(
                "Disposable TLS fixture not requested; run npm run test:database-tls-contract"
            );
            return None;
        };
        assert_eq!(uuid::Uuid::parse_str(&run_id).unwrap().to_string(), run_id);
        assert!(
            std::env::args().any(|arg| arg == "--exact"),
            "TLS fixture must run in an isolated exact-test process"
        );
        let env = pg_test_env().expect("complete synthetic fixture environment");
        assert_eq!(env.host, "localhost");
        assert_eq!(env.username, "kisat_fixture");
        assert_eq!(env.password, format!("synthetic-{run_id}"));
        let container = std::env::var("KISAT_PG_TLS_CONTAINER").unwrap();
        assert_eq!(container, format!("kisat-pg-tls-{run_id}-verified"));
        let plain_port = std::env::var("KISAT_PG_PLAIN_PORT")
            .unwrap()
            .parse()
            .unwrap();
        Some(Self {
            env,
            run_id,
            container,
            plain_port,
        })
    }

    async fn docker(&self, args: &[&str]) -> String {
        let output = timeout(
            Duration::from_secs(30),
            tokio::process::Command::new("docker")
                .args(args)
                .kill_on_drop(true)
                .output(),
        )
        .await
        .expect("Docker command deadline")
        .expect("Docker CLI");
        assert!(
            output.status.success(),
            "Docker fixture command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    async fn verify_owned_container(&self) {
        let value = self
            .docker(&[
                "inspect",
                "--format",
                "{{ index .Config.Labels \"com.kisautotrade.database-tls-fixture\" }}",
                &self.container,
            ])
            .await;
        assert_eq!(
            value.trim(),
            self.run_id,
            "Never control a container outside this test's fixture"
        );
    }

    async fn stop(&self) {
        self.verify_owned_container().await;
        self.docker(&["kill", "--signal=SIGKILL", &self.container])
            .await;
        assert_eq!(
            self.docker(&["inspect", "--format", "{{.State.Running}}", &self.container])
                .await
                .trim(),
            "false"
        );
    }

    async fn restart(&self) {
        self.verify_owned_container().await;
        self.docker(&["start", &self.container]).await;
        timeout(Duration::from_secs(25), async {
            for _ in 0..80 {
                let output = tokio::process::Command::new("docker")
                    .args([
                        "exec",
                        &self.container,
                        "pg_isready",
                        "-h",
                        "127.0.0.1",
                        "-U",
                        &self.env.username,
                        "-d",
                        "postgres",
                    ])
                    .kill_on_drop(true)
                    .output()
                    .await
                    .unwrap();
                if output.status.success() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            panic!("fixture did not restart within the bounded readiness window");
        })
        .await
        .expect("Docker restart readiness deadline");
    }

    // Plaintext is used only by this synthetic observer to prove that a failed
    // application TLS attempt neither created a DB nor silently downgraded.
    async fn control(&self, port: u16) -> sqlx::PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(
                PgConnectOptions::new()
                    .host("localhost")
                    .port(port)
                    .database("postgres")
                    .username(&self.env.username)
                    .password(&self.env.password)
                    .ssl_mode(PgSslMode::Disable),
            )
            .await
            .unwrap()
    }
}

fn new_manager() -> (PathBuf, PathBuf, DatabaseManager) {
    let (config, data) = temp_dirs();
    std::fs::create_dir_all(&data).unwrap();
    let manager = DatabaseManager::load_sync(config.clone(), data.clone()).unwrap();
    (config, data, manager)
}

fn required_input(fixture: &Fixture, database: &str) -> SaveDatabaseConfigInput {
    let mut input = save_input(&fixture.env, database);
    input.tls_mode = DatabaseTlsMode::Require;
    input
}

async fn assert_missing(pool: &sqlx::PgPool, database: &str) {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname=$1)")
            .bind(database)
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(
        !exists,
        "Rejected TLS/authentication must not create a target database"
    );
}

fn redacted(error: &anyhow::Error, fixture: &Fixture) {
    assert!(
        !format!("{error:#}").contains(&fixture.env.password),
        "Database error exposed a credential"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn verified_tls_and_restart_contract() {
    let Some(fixture) = Fixture::from_env() else {
        return;
    };
    super::super::database_keychain::use_mock_keychain_for_tests();
    let _keychain = super::super::database_keychain::keychain_test_lock().await;
    fixture.verify_owned_container().await;
    let control = fixture.control(fixture.env.port).await;
    // The certificate has DNS:localhost only: the very same trusted server
    // rejects the IP name, proving hostname validation rather than encryption.
    for failure in ["hostname", "plaintext", "password"] {
        let (_, _, manager) = new_manager();
        let database = unique_test_database();
        let mut input = required_input(&fixture, &database);
        match failure {
            "hostname" => input.host = "127.0.0.1".into(),
            "plaintext" => input.port = fixture.plain_port,
            _ => input.password = Some("synthetic-invalid-password".into()),
        }
        manager.save_config(input).await.unwrap();
        let error = timeout(Duration::from_secs(15), manager.status())
            .await
            .expect("failed connect deadline")
            .expect_err(failure);
        redacted(&error, &fixture);
        let observer = if failure == "plaintext" {
            fixture.control(fixture.plain_port).await
        } else {
            control.clone()
        };
        assert_missing(&observer, &database).await;
        eprintln!("TLS Require rejected {failure} without creating a database");
    }

    let (config_path, data_dir, manager) = new_manager();
    let manager = Arc::new(manager);
    let database = unique_test_database();
    manager
        .save_config(required_input(&fixture, &database))
        .await
        .unwrap();
    let mut tasks = tokio::task::JoinSet::new();
    let barrier = Arc::new(tokio::sync::Barrier::new(17));
    for _ in 0..16 {
        let manager = manager.clone();
        let barrier = barrier.clone();
        tasks.spawn(async move {
            barrier.wait().await;
            manager.pool().await.unwrap()
        });
    }
    barrier.wait().await;
    // Retain all returned pool handles. A racy implementation cannot hide
    // excess pools by dropping the earlier callers' connections before counting.
    let mut handles = Vec::new();
    while let Some(pool) = tasks.join_next().await {
        handles.push(pool.unwrap());
    }
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname=$1 AND usename=$2")
            .bind(&database)
            .bind(&fixture.env.username)
            .fetch_one(&control)
            .await
            .unwrap();
    assert!(
        (1..=2).contains(&count),
        "concurrent cold callers exceeded max_connections: {count}"
    );
    let DatabasePool::Postgresql(cached_pool) = manager.pool().await.unwrap() else {
        unreachable!()
    };
    let encrypted: bool =
        sqlx::query_scalar("SELECT ssl FROM pg_stat_ssl WHERE pid=pg_backend_pid()")
            .fetch_one(&cached_pool)
            .await
            .unwrap();
    assert!(encrypted, "Require must never fall back to plaintext");
    let first_start: String = sqlx::query_scalar("SELECT pg_postmaster_start_time()::text")
        .fetch_one(&cached_pool)
        .await
        .unwrap();
    manager.create_tables().await.unwrap();
    let keys = [
        "orders/pending_orders.json",
        "orders/position_ownership.json",
        "risk/runtime.json",
    ];
    let local = [
        "[]",
        "{\"fixture\":\"local-owned\"}",
        "{\"fixture\":\"local-risk\"}",
    ];
    let committed = [
        "[]",
        "{\"fixture\":\"database-owned\"}",
        "{\"fixture\":\"database-risk\"}",
    ];
    for (index, key) in keys.iter().enumerate() {
        write_test_document(&data_dir, key, local[index]).await;
        write_test_document(&data_dir, &format!("{key}.bak"), "synthetic local backup").await;
    }
    manager.import_json().await.unwrap();
    manager.set_backend(StorageBackend::Database).await.unwrap();
    // Selected with --exact in its own child process; exercise production store
    // routing without installing a manager in the other unit-test processes.
    super::super::database::install_database_manager(manager.clone()).unwrap();
    for (index, key) in keys.iter().enumerate() {
        manager
            .write_document(&data_dir.join(key), committed[index])
            .await
            .unwrap();
    }
    let persisted_config = tokio::fs::read(&config_path).await.unwrap();

    // A killed backend socket is repaired by SQLx acquisition, with the same
    // application manager and cached pool; no write is replayed automatically.
    let mut connection = cached_pool.acquire().await.unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *connection)
        .await
        .unwrap();
    let terminated: bool = sqlx::query_scalar("SELECT pg_terminate_backend($1)")
        .bind(pid)
        .fetch_one(&control)
        .await
        .unwrap();
    assert!(terminated);
    assert!(sqlx::query("SELECT 1")
        .execute(&mut *connection)
        .await
        .is_err());
    drop(connection);
    assert_eq!(
        manager
            .read_document(&data_dir.join(keys[1]))
            .await
            .unwrap()
            .as_deref(),
        Some(committed[1])
    );

    fixture.stop().await;
    let error = timeout(Duration::from_secs(15), manager.status())
        .await
        .expect("outage status deadline")
        .expect_err("outage status");
    redacted(&error, &fixture);
    eprintln!("Stopped fixture: status failed within its acquisition deadline");
    let pending = super::super::PendingOrderStore::new(data_dir.clone());
    let error = timeout(Duration::from_secs(15), pending.load())
        .await
        .expect("pending outage deadline")
        .expect_err("pending DB failure must not load local empty pending or backup");
    redacted(&error, &fixture);
    let error = timeout(
        Duration::from_secs(15),
        super::super::database::read_managed_json(&data_dir.join(keys[1])),
    )
    .await
    .expect("ownership outage deadline")
    .expect_err("ownership DB failure must not become missing JSON");
    redacted(&error, &fixture);
    let error = timeout(
        Duration::from_secs(15),
        super::super::read_json_or_default::<serde_json::Value>(&data_dir.join(keys[2])),
    )
    .await
    .expect("risk document outage deadline")
    .expect_err("risk DB failure must not load JSON/default/backup");
    redacted(&error, &fixture);
    eprintln!("Stopped fixture: pending/ownership/risk read routes failed without JSON fallback");
    assert!(timeout(
        Duration::from_secs(15),
        manager.write_document(&data_dir.join(keys[1]), "{\"fixture\":\"uncommitted\"}")
    )
    .await
    .expect("outage write deadline")
    .is_err());
    assert!(timeout(
        Duration::from_secs(15),
        manager.set_backend(StorageBackend::Json)
    )
    .await
    .expect("outage backend switch deadline")
    .is_err());
    assert_eq!(
        manager.config_view().await.active_backend,
        StorageBackend::Database
    );
    assert_eq!(
        tokio::fs::read(&config_path).await.unwrap(),
        persisted_config
    );
    for (index, key) in keys.iter().enumerate() {
        assert_eq!(
            tokio::fs::read_to_string(data_dir.join(key)).await.unwrap(),
            local[index]
        );
        assert_eq!(
            tokio::fs::read_to_string(data_dir.join(format!("{key}.bak")))
                .await
                .unwrap(),
            "synthetic local backup"
        );
    }
    fixture.restart().await;
    let second_start: String = sqlx::query_scalar("SELECT pg_postmaster_start_time()::text")
        .fetch_one(&cached_pool)
        .await
        .unwrap();
    assert_ne!(
        first_start, second_start,
        "the same server process was not restarted"
    );
    for (index, key) in keys.iter().enumerate() {
        assert_eq!(
            manager
                .read_document(&data_dir.join(key))
                .await
                .unwrap()
                .as_deref(),
            Some(committed[index])
        );
    }
    manager
        .write_document(&data_dir.join(keys[1]), "{\"fixture\":\"reconnected\"}")
        .await
        .unwrap();
    assert_eq!(
        manager
            .read_document(&data_dir.join(keys[1]))
            .await
            .unwrap()
            .as_deref(),
        Some("{\"fixture\":\"reconnected\"}")
    );
    assert!(manager.status().await.unwrap().connected);
    cached_pool.close().await;
    control.close().await;
    drop(handles);
    drop(manager);
    eprintln!("TLS_REQUIRE_AND_RESTART_CONTRACT_PASSED");
}

#[tokio::test]
async fn rejects_untrusted_ca_contract() {
    let Some(fixture) = Fixture::from_env() else {
        return;
    };
    assert_eq!(std::env::var("KISAT_PG_TLS_UNTRUSTED").as_deref(), Ok("1"));
    super::super::database_keychain::use_mock_keychain_for_tests();
    let _keychain = super::super::database_keychain::keychain_test_lock().await;
    fixture.verify_owned_container().await;
    let (_, _, manager) = new_manager();
    let database = unique_test_database();
    manager
        .save_config(required_input(&fixture, &database))
        .await
        .unwrap();
    let error = timeout(Duration::from_secs(15), manager.status())
        .await
        .expect("untrusted CA deadline")
        .expect_err("untrusted CA");
    redacted(&error, &fixture);
    let control = fixture.control(fixture.env.port).await;
    assert_missing(&control, &database).await;
    control.close().await;
    eprintln!("TLS_UNTRUSTED_CA_CONTRACT_PASSED");
}
