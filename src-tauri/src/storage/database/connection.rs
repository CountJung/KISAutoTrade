//! Connection pool creation and provider TLS options.

use super::*;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode},
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use std::time::Duration;

impl DatabaseManager {
    pub(in crate::storage) async fn pool(&self) -> Result<DatabasePool> {
        self.connection_snapshot().await.map(|(_, pool)| pool)
    }

    pub(super) async fn connection_snapshot(&self) -> Result<(DatabaseConfig, DatabasePool)> {
        self.pool_with_connector(|config| async move { connect_pool(&config).await })
            .await
    }

    async fn pool_with_connector<F, Fut>(
        &self,
        connect: F,
    ) -> Result<(DatabaseConfig, DatabasePool)>
    where
        F: FnOnce(DatabaseConfig) -> Fut,
        Fut: std::future::Future<Output = Result<DatabasePool>>,
    {
        // Separate creation from the cache mutex: a single bounded connection attempt is in
        // flight, while settings can still publish a new target without waiting on the network.
        let _creation = self.pool_creation_lock.lock().await;
        let (config, fingerprint) = {
            let config = self.config.read().await;
            self.ensure_config_recovered()?;
            config.validate()?;
            let fingerprint = config.fingerprint();
            if let Some((cached_fingerprint, pool)) = &*self.pool.lock().await {
                if cached_fingerprint == &fingerprint {
                    return Ok((config.clone(), pool.clone()));
                }
            }
            (config.clone(), fingerprint)
        };
        let pool = connect(config).await?;
        let current = self.config.read().await;
        if self.ensure_config_recovered().is_err() || current.fingerprint() != fingerprint {
            drop(current);
            match &pool {
                DatabasePool::Postgresql(pool) => pool.close().await,
                DatabasePool::Mariadb(pool) => pool.close().await,
            }
            bail!("DB 연결 중 설정이 변경되었거나 저장 복구가 필요합니다. 다시 시도하세요.");
        }
        *self.pool.lock().await = Some((fingerprint, pool.clone()));
        Ok((current.clone(), pool))
    }
}

async fn connect_pool(config: &DatabaseConfig) -> Result<DatabasePool> {
    let timeout = Duration::from_secs(10);
    match config.provider {
        DatabaseProvider::Postgresql => {
            let ssl_mode = match config.tls_mode {
                DatabaseTlsMode::Disable => PgSslMode::Disable,
                DatabaseTlsMode::Prefer => PgSslMode::Prefer,
                DatabaseTlsMode::Require => PgSslMode::VerifyFull,
            };
            let options = PgConnectOptions::new()
                .host(&config.host)
                .port(config.port)
                .database(&config.database)
                .username(&config.username)
                .password(&config.password)
                .ssl_mode(ssl_mode);
            match PgPoolOptions::new()
                .max_connections(config.max_connections)
                .acquire_timeout(timeout)
                .connect_with(options.clone())
                .await
            {
                Ok(pool) => Ok(DatabasePool::Postgresql(pool)),
                // 3D000 = invalid_catalog_name: 자격증명은 유효하지만 대상 DB가 없는 경우이므로
                // 관리자 DB(postgres)로 접속해 대상 DB를 생성한 뒤 다시 시도한다.
                Err(error) if is_missing_database_error(&error, "3D000") => {
                    create_missing_postgres_database(config, ssl_mode, timeout).await?;
                    Ok(DatabasePool::Postgresql(
                        PgPoolOptions::new()
                            .max_connections(config.max_connections)
                            .acquire_timeout(timeout)
                            .connect_with(options)
                            .await
                            .context("PostgreSQL 연결 실패")?,
                    ))
                }
                Err(error) => Err(error).context("PostgreSQL 연결 실패"),
            }
        }
        DatabaseProvider::Mariadb => {
            let ssl_mode = match config.tls_mode {
                DatabaseTlsMode::Disable => MySqlSslMode::Disabled,
                DatabaseTlsMode::Prefer => MySqlSslMode::Preferred,
                DatabaseTlsMode::Require => MySqlSslMode::VerifyIdentity,
            };
            let options = MySqlConnectOptions::new()
                .host(&config.host)
                .port(config.port)
                .database(&config.database)
                .username(&config.username)
                .password(&config.password)
                .ssl_mode(ssl_mode);
            match MySqlPoolOptions::new()
                .max_connections(config.max_connections)
                .acquire_timeout(timeout)
                .connect_with(options.clone())
                .await
            {
                Ok(pool) => Ok(DatabasePool::Mariadb(pool)),
                // 1049 = ER_BAD_DB_ERROR: 자격증명은 유효하지만 대상 DB가 없는 경우이므로
                // DB를 지정하지 않고 접속해 대상 DB를 생성한 뒤 다시 시도한다.
                Err(error) if is_missing_database_error(&error, "1049") => {
                    create_missing_mariadb_database(config, ssl_mode, timeout).await?;
                    Ok(DatabasePool::Mariadb(
                        MySqlPoolOptions::new()
                            .max_connections(config.max_connections)
                            .acquire_timeout(timeout)
                            .connect_with(options)
                            .await
                            .context("MariaDB 연결 실패")?,
                    ))
                }
                Err(error) => Err(error).context("MariaDB 연결 실패"),
            }
        }
    }
}

fn is_missing_database_error(error: &sqlx::Error, code: &str) -> bool {
    matches!(error, sqlx::Error::Database(db_error) if db_error.code().as_deref() == Some(code))
}

/// 자동 생성 시 SQL 구문에 그대로 삽입되므로 영문/숫자/밑줄만 허용해 인젝션을 방지한다.
fn validate_database_identifier(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name.len() <= 63
        && name
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && name
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || value == '_');
    if !valid {
        bail!(
            "자동 생성하려는 DB 이름 '{name}'을(를) 사용할 수 없습니다. \
             영문/숫자/밑줄만 사용하고 첫 글자는 영문 또는 밑줄이어야 하며 63자 이하여야 합니다."
        );
    }
    Ok(())
}

async fn create_missing_postgres_database(
    config: &DatabaseConfig,
    ssl_mode: PgSslMode,
    timeout: Duration,
) -> Result<()> {
    validate_database_identifier(&config.database)?;
    let admin_options = PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .database("postgres")
        .username(&config.username)
        .password(&config.password)
        .ssl_mode(ssl_mode);
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(timeout)
        .connect_with(admin_options)
        .await
        .context(
            "지정한 DB가 없어 자동 생성을 시도했지만 관리자 DB(postgres)에 연결하지 못했습니다. \
             DB 관리자에게 요청해 데이터베이스를 미리 만들어주세요.",
        )?;
    sqlx::query(&format!("CREATE DATABASE \"{}\"", config.database))
        .execute(&admin_pool)
        .await
        .context(
            "지정한 DB가 없어 자동 생성을 시도했지만 실패했습니다. \
             DB 사용자에게 CREATEDB 권한이 있는지 확인하세요.",
        )?;
    admin_pool.close().await;
    Ok(())
}

async fn create_missing_mariadb_database(
    config: &DatabaseConfig,
    ssl_mode: MySqlSslMode,
    timeout: Duration,
) -> Result<()> {
    validate_database_identifier(&config.database)?;
    let admin_options = MySqlConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .ssl_mode(ssl_mode);
    let admin_pool = MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(timeout)
        .connect_with(admin_options)
        .await
        .context(
            "지정한 DB가 없어 자동 생성을 시도했지만 MariaDB 서버에 연결하지 못했습니다. \
             DB 관리자에게 요청해 데이터베이스를 미리 만들어주세요.",
        )?;
    sqlx::query(&format!(
        "CREATE DATABASE IF NOT EXISTS `{}` CHARACTER SET utf8mb4 COLLATE utf8mb4_bin",
        config.database
    ))
    .execute(&admin_pool)
    .await
    .context(
        "지정한 DB가 없어 자동 생성을 시도했지만 실패했습니다. \
         DB 사용자에게 CREATE 권한이 있는지 확인하세요.",
    )?;
    admin_pool.close().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn manager() -> DatabaseManager {
        let root = std::env::temp_dir().join(format!("kis-pool-{}", uuid::Uuid::new_v4()));
        DatabaseManager {
            config_path: root.join("database_config.json"),
            data_dir: root.join("data"),
            export_dir: root.join("exports"),
            config: tokio::sync::RwLock::new(DatabaseConfig::default()),
            recovery_required: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pool: Mutex::new(None),
            pool_creation_lock: Mutex::new(()),
            operation_lock: Mutex::new(()),
        }
    }

    #[tokio::test]
    async fn cached_connection_snapshot_reports_current_matching_settings() {
        let manager = manager();
        let (initial, _) = manager
            .pool_with_connector(|config| async move {
                assert_eq!(config.host, "127.0.0.1");
                Ok(DatabasePool::Postgresql(
                    PgPoolOptions::new().connect_lazy_with(PgConnectOptions::new()),
                ))
            })
            .await
            .unwrap();
        assert_eq!(initial.active_backend, StorageBackend::Json);
        // Backend-only publication preserves the same connection fingerprint.
        manager.config.write().await.active_backend = StorageBackend::Database;
        let (snapshot, pool) = manager
            .pool_with_connector(|_| async { panic!("matching pool must be cached") })
            .await
            .unwrap();
        assert_eq!(snapshot.host, initial.host);
        assert_eq!(snapshot.active_backend, StorageBackend::Database);
        assert!(matches!(pool, DatabasePool::Postgresql(_)));
    }

    #[tokio::test]
    async fn concurrent_first_connections_create_one_pool() {
        let manager = Arc::new(manager());
        let calls = Arc::new(AtomicUsize::new(0));
        let start = Arc::new(tokio::sync::Barrier::new(24));
        let mut tasks = tokio::task::JoinSet::new();
        for _ in 0..24 {
            let manager = manager.clone();
            let calls = calls.clone();
            let start = start.clone();
            tasks.spawn(async move {
                start.wait().await;
                manager
                    .pool_with_connector(|config| async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        tokio::task::yield_now().await;
                        Ok(DatabasePool::Postgresql(
                            PgPoolOptions::new()
                                .max_connections(config.max_connections)
                                .connect_lazy_with(PgConnectOptions::new()),
                        ))
                    })
                    .await
                    .unwrap()
            });
        }
        tokio::time::timeout(Duration::from_secs(3), async {
            while let Some(result) = tasks.join_next().await {
                result.expect("connection task");
            }
        })
        .await
        .expect("bounded concurrent connection test");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn connecting_old_target_is_closed_and_not_cached_after_config_save() {
        super::super::super::database_keychain::use_mock_keychain_for_tests();
        let _keychain = super::super::super::database_keychain::keychain_test_lock().await;
        let manager = Arc::new(manager());
        let candidate = PgPoolOptions::new().connect_lazy_with(PgConnectOptions::new());
        let observed = candidate.clone();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (resume_tx, resume_rx) = tokio::sync::oneshot::channel();
        let task_manager = manager.clone();
        let connecting = tokio::spawn(async move {
            task_manager
                .pool_with_connector(|_| async move {
                    started_tx.send(()).unwrap();
                    resume_rx.await.unwrap();
                    Ok(DatabasePool::Postgresql(candidate))
                })
                .await
        });
        started_rx.await.unwrap();
        manager
            .save_config(SaveDatabaseConfigInput {
                provider: DatabaseProvider::Postgresql,
                host: "new.synthetic.invalid".into(),
                port: 5432,
                database: "synthetic_database".into(),
                username: "synthetic_user".into(),
                password: Some("synthetic_secret".into()),
                tls_mode: DatabaseTlsMode::Require,
                max_connections: 1,
            })
            .await
            .unwrap();
        resume_tx.send(()).unwrap();
        assert!(tokio::time::timeout(Duration::from_secs(3), connecting)
            .await
            .unwrap()
            .unwrap()
            .is_err());
        assert!(observed.is_closed());
        assert!(manager.pool.lock().await.is_none());
        std::fs::remove_dir_all(manager.config_path.parent().unwrap()).unwrap();
    }
}
