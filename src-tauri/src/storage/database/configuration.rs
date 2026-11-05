//! Configuration publication and durable credential persistence.

use super::super::{database_io, database_keychain};
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

impl DatabaseManager {
    pub fn load_sync(config_path: PathBuf, data_dir: PathBuf) -> Result<Self> {
        ensure_recovered(&config_path)?;
        let mut config: DatabaseConfig = match std::fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content)
                .with_context(|| format!("DB 설정 파일이 손상되었습니다: {config_path:?}"))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => DatabaseConfig::default(),
            Err(error) => return Err(error).context("DB 설정 파일을 읽지 못했습니다."),
        };
        if config.password.is_empty() {
            match database_keychain::load_database_password() {
                Ok(Some(password)) => config.password = password,
                Ok(None) => {}
                Err(error) => tracing::warn!(
                    "OS keychain에서 DB password를 읽지 못했습니다 — 미설정 상태로 시작합니다: {error:#}"
                ),
            }
        } else {
            // 파일에 남아 있는 password를 OS keychain으로 1회 이전한다. 실패하면 파일(0o600) 저장 유지.
            match database_keychain::store_database_password(&config.password) {
                Ok(()) => {
                    let mut file_config = config.clone();
                    file_config.password = String::new();
                    let content = serde_json::to_string_pretty(&file_config)?;
                    match database_io::atomic_write_private_sync(&config_path, &content) {
                        Ok(()) => tracing::info!("DB password를 OS keychain으로 이전했습니다."),
                        Err(error) => tracing::warn!(
                            "keychain 이전 후 설정 파일 재작성 실패 — 다음 시작에서 재시도합니다: {error:#}"
                        ),
                    }
                }
                Err(error) => tracing::warn!(
                    "DB password를 OS keychain으로 이전하지 못해 파일(0o600) 저장을 유지합니다: {error:#}"
                ),
            }
        }
        let export_dir = config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("database_exports");
        Ok(Self {
            config_path,
            data_dir,
            export_dir,
            config: tokio::sync::RwLock::new(config),
            recovery_required: Arc::new(AtomicBool::new(false)),
            pool: Mutex::new(None),
            pool_creation_lock: Mutex::new(()),
            operation_lock: Mutex::new(()),
        })
    }

    pub async fn config_view(&self) -> DatabaseConfigView {
        DatabaseConfigView::from(&*self.config.read().await)
    }

    pub async fn save_config(&self, input: SaveDatabaseConfigInput) -> Result<DatabaseConfigView> {
        let _operation = self.operation_lock.lock().await;
        self.ensure_config_recovered()?;
        let existing = self.config.read().await.clone();
        if existing.active_backend == StorageBackend::Database {
            bail!("DB 연결 설정을 바꾸기 전에 JSON 저장으로 전환해 최신 데이터를 복구하세요.");
        }
        let password = input
            .password
            .filter(|value| !value.is_empty())
            .unwrap_or(existing.password);
        let config = DatabaseConfig {
            provider: input.provider,
            host: input.host.trim().to_string(),
            port: input.port,
            database: input.database.trim().to_string(),
            username: input.username.trim().to_string(),
            password,
            tls_mode: input.tls_mode,
            max_connections: input.max_connections,
            // Connection changes always return to JSON until the new target is tested/imported.
            active_backend: StorageBackend::Json,
        };
        config.validate()?;
        self.persist_config(&config).await?;
        self.publish_config(config.clone(), true).await?;
        Ok(DatabaseConfigView::from(&config))
    }

    /// Keep the recovery marker until the caller publishes the committed settings in memory.
    /// A cancelled caller cannot expose a new keychain secret alongside the previous endpoint.
    pub(super) async fn persist_config(&self, config: &DatabaseConfig) -> Result<()> {
        self.ensure_config_recovered()?;
        let path = self.config_path.clone();
        let config = config.clone();
        let recovery_required = self.recovery_required.clone();
        recovery_required.store(true, Ordering::SeqCst);
        create_recovery_marker(&path)?;
        tokio::task::spawn_blocking(move || {
            persist_using_writer_and_recovery(
                &path,
                &config,
                database_io::atomic_write_private_sync,
                || recovery_required.store(false, Ordering::SeqCst),
            )
        })
        .await
        .context("DB 설정 저장 task 실패 — 복구 확인이 필요합니다")?
    }

    pub(super) async fn publish_config(
        &self,
        config: DatabaseConfig,
        invalidate_pool: bool,
    ) -> Result<()> {
        self.publish_config_using_remover(config, invalidate_pool, database_io::durable_remove_file)
            .await
    }

    async fn publish_config_using_remover(
        &self,
        config: DatabaseConfig,
        invalidate_pool: bool,
        remove: impl FnOnce(&Path) -> Result<()>,
    ) -> Result<()> {
        let mut current = self.config.write().await;
        let mut pool = self.pool.lock().await;
        *current = config;
        if invalidate_pool {
            *pool = None;
        }
        // There is no await between publication and marker removal. Pool readers hold the
        // config read guard while checking the marker and looking up the cached pool.
        clear_recovery_marker(&self.config_path, remove)
            .context("DB 설정 게시 후 복구 표시를 제거하지 못했습니다")?;
        self.recovery_required.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub(super) fn ensure_config_recovered(&self) -> Result<()> {
        anyhow::ensure!(
            !self.recovery_required.load(Ordering::SeqCst),
            "DB 설정 저장이 완료되지 않았습니다. 파일과 OS keychain 복구를 확인하세요."
        );
        ensure_recovered(&self.config_path)
    }
}

fn recovery_path(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!("{name}.recovery"))
}

fn ensure_recovered(path: &Path) -> Result<()> {
    if recovery_path(path).try_exists()? {
        bail!("DB 설정 저장이 완료되지 않았습니다. 설정 파일과 OS keychain 복구를 확인한 후 다시 시작하세요.");
    }
    Ok(())
}

fn create_recovery_marker(path: &Path) -> Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let marker = recovery_path(path);
    let mut file = options.open(&marker)?;
    file.write_all(b"Database settings transaction requires completion or recovery.\n")?;
    file.sync_all()?;
    drop(file);
    #[cfg(unix)]
    if let Some(parent) = marker.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    // File flush alone does not commit a Windows directory entry. Reuse the existing
    // private writer's MoveFileExW WRITE_THROUGH replacement before changing credentials.
    #[cfg(windows)]
    database_io::atomic_write_private_sync(
        &marker,
        "Database settings transaction requires completion or recovery.\n",
    )?;
    Ok(())
}

fn read_optional_file(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn restore_keychain(password: Option<&str>) -> Result<()> {
    match password {
        Some(password) => database_keychain::store_database_password(password)?,
        None => database_keychain::delete_database_password()?,
    }
    anyhow::ensure!(
        database_keychain::load_database_password()?.as_deref() == password,
        "OS keychain 복원 검증 실패"
    );
    Ok(())
}

fn clear_recovery_marker(path: &Path, remove: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    if let Err(error) = remove(&recovery_path(path)) {
        // unlink may already have succeeded before the parent sync failed. Restore a durable
        // marker before returning; the manager latch also blocks use if recreation fails.
        if !recovery_path(path).try_exists()? {
            create_recovery_marker(path).context("DB 복구 표시 재생성 실패")?;
        }
        return Err(error).context("DB 복구 표시의 영구 삭제 확인 실패");
    }
    Ok(())
}

#[cfg(test)]
fn persist_using_writer(
    path: &Path,
    config: &DatabaseConfig,
    write: impl FnOnce(&Path, &str) -> Result<()>,
) -> Result<()> {
    persist_using_writer_and_recovery(path, config, write, || {})
}

fn persist_using_writer_and_recovery(
    path: &Path,
    config: &DatabaseConfig,
    write: impl FnOnce(&Path, &str) -> Result<()>,
    recovered: impl FnOnce(),
) -> Result<()> {
    // If keychain is unavailable, do not mutate it. A nonempty inline password stays
    // authoritative, preserving the owner-only config fallback on headless systems.
    let old_file = read_optional_file(path)?;
    let old_password = match database_keychain::load_database_password() {
        Ok(password) => Some(password),
        Err(error) => {
            tracing::warn!("OS keychain snapshot 실패 — 파일(0o600) 저장을 유지합니다: {error:#}");
            None
        }
    };
    let result = (|| -> Result<()> {
        let mut file_config = config.clone();
        if let Some(old_password) = &old_password {
            let stored = if config.password.is_empty() {
                database_keychain::delete_database_password()
            } else {
                database_keychain::store_database_password(&config.password)
            };
            match stored {
                Ok(()) => file_config.password.clear(),
                Err(error) => {
                    // An error may follow a mutation. Verify restoration before file fallback.
                    restore_keychain(old_password.as_deref())?;
                    if config.password.is_empty() && old_password.is_some() {
                        return Err(error)
                            .context("빈 DB password 저장을 위해 keychain 삭제가 필요합니다");
                    }
                    tracing::warn!(
                        "OS keychain 저장 실패 — DB password를 파일(0o600)에 유지합니다: {error:#}"
                    );
                }
            }
        } else {
            anyhow::ensure!(
                !config.password.is_empty(),
                "keychain 상태를 확인하지 못해 빈 DB password를 저장할 수 없습니다"
            );
        }
        write(path, &serde_json::to_string_pretty(&file_config)?)
    })();
    if let Err(error) = result {
        let recovery = (|| -> Result<()> {
            // atomic write can fail after rename (directory fsync). Restore the actual file,
            // not just the keychain, before clearing the recovery marker.
            if read_optional_file(path)? != old_file {
                match &old_file {
                    Some(content) => database_io::atomic_write_private_sync(path, content)?,
                    None => database_io::durable_remove_file(path)?,
                }
            }
            anyhow::ensure!(
                read_optional_file(path)? == old_file,
                "DB 설정 파일 복원 검증 실패"
            );
            if let Some(old_password) = &old_password {
                restore_keychain(old_password.as_deref())?;
            }
            clear_recovery_marker(path, database_io::durable_remove_file)?;
            recovered();
            Ok(())
        })();
        return match recovery {
            Ok(()) => Err(error),
            Err(recovery) => Err(error).context(format!(
                "DB 설정 복구 실패 — 연결이 차단됩니다: {recovery:#}"
            )),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn first_save_failure_after_rename_durably_restores_missing_config() {
        database_keychain::use_mock_keychain_for_tests();
        let _keychain = database_keychain::keychain_test_lock().await;
        let path = test_path();
        database_keychain::delete_database_password().unwrap();
        let replacement = DatabaseConfig {
            host: "new.synthetic.invalid".into(),
            password: "new-secret".into(),
            ..DatabaseConfig::default()
        };
        create_recovery_marker(&path).unwrap();
        assert!(persist_using_writer(&path, &replacement, |path, content| {
            database_io::atomic_write_private_sync(path, content)?;
            bail!("synthetic failure after first rename")
        })
        .is_err());
        assert!(!path.exists());
        assert!(!recovery_path(&path).exists());
        assert_eq!(database_keychain::load_database_password().unwrap(), None);
        let reloaded = DatabaseManager::load_sync(path.clone(), PathBuf::new()).unwrap();
        assert_eq!(reloaded.config.read().await.host, "127.0.0.1");
        assert_eq!(reloaded.config.read().await.password, "");
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn marker_sync_failure_recreates_marker_and_keeps_runtime_latched() {
        database_keychain::use_mock_keychain_for_tests();
        let _keychain = database_keychain::keychain_test_lock().await;
        let path = test_path();
        let manager = DatabaseManager::load_sync(path.clone(), PathBuf::new()).unwrap();
        let replacement = DatabaseConfig {
            host: "new.synthetic.invalid".into(),
            password: "new-secret".into(),
            ..DatabaseConfig::default()
        };
        manager.persist_config(&replacement).await.unwrap();
        assert!(manager
            .publish_config_using_remover(replacement, true, |marker| {
                std::fs::remove_file(marker)?;
                bail!("synthetic marker parent sync failure")
            })
            .await
            .is_err());
        assert!(
            recovery_path(&path).exists(),
            "recreate marker after unlink + failed sync"
        );
        assert!(DatabaseManager::load_sync(path.clone(), PathBuf::new()).is_err());
        assert!(manager.pool().await.is_err());
        // The in-memory latch still blocks use even if the marker cannot remain on disk.
        std::fs::remove_file(recovery_path(&path)).unwrap();
        assert!(manager.ensure_config_recovered().is_err());
        assert!(manager.pool().await.is_err());
        assert!(manager
            .save_config(save_input("third.synthetic.invalid", "third-secret"))
            .await
            .is_err());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    fn save_input(host: &str, password: &str) -> SaveDatabaseConfigInput {
        SaveDatabaseConfigInput {
            provider: DatabaseProvider::Postgresql,
            host: host.into(),
            port: 5432,
            database: "synthetic_database".into(),
            username: "synthetic_user".into(),
            password: Some(password.into()),
            tls_mode: DatabaseTlsMode::Require,
            max_connections: 1,
        }
    }

    #[tokio::test]
    async fn failed_save_keeps_live_and_restarted_endpoint_credential_pair() {
        database_keychain::use_mock_keychain_for_tests();
        let _keychain = database_keychain::keychain_test_lock().await;
        let path = test_path();
        let manager = DatabaseManager::load_sync(path.clone(), PathBuf::new()).unwrap();
        manager
            .save_config(save_input("old.synthetic.invalid", "old-secret"))
            .await
            .unwrap();
        // An actual atomic writer backup failure occurs after the new secret was stored.
        std::fs::create_dir(path.with_file_name("database_config.json.bak")).unwrap();
        assert!(manager
            .save_config(save_input("new.synthetic.invalid", "new-secret"))
            .await
            .is_err());
        assert_eq!(manager.config.read().await.password, "old-secret");
        let restarted = DatabaseManager::load_sync(path.clone(), PathBuf::new()).unwrap();
        assert_eq!(restarted.config.read().await.host, "old.synthetic.invalid");
        assert_eq!(restarted.config.read().await.password, "old-secret");
        std::fs::remove_dir(path.with_file_name("database_config.json.bak")).unwrap();
        manager
            .save_config(save_input("retry.synthetic.invalid", "retry-secret"))
            .await
            .expect("verified rollback must release the runtime recovery latch");
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn cancelled_save_before_memory_publication_blocks_further_connections() {
        database_keychain::use_mock_keychain_for_tests();
        let _keychain = database_keychain::keychain_test_lock().await;
        let path = test_path();
        let manager = Arc::new(DatabaseManager::load_sync(path.clone(), PathBuf::new()).unwrap());
        // Allow snapshot reads but prevent the caller from publishing after the blocking commit.
        let config_read = manager.config.read().await;
        let task_manager = manager.clone();
        let task = tokio::spawn(async move {
            task_manager
                .save_config(save_input("new.synthetic.invalid", "new-secret"))
                .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                if read_optional_file(&path)
                    .unwrap()
                    .is_some_and(|content| content.contains("new.synthetic.invalid"))
                {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("blocking persistence committed");
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(config_read.host, "127.0.0.1");
        drop(config_read);
        assert!(manager
            .save_config(save_input("third.synthetic.invalid", "third-secret"))
            .await
            .is_err());
        assert!(manager.pool().await.is_err());
        assert!(DatabaseManager::load_sync(path.clone(), PathBuf::new()).is_err());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn unavailable_keychain_keeps_private_file_fallback_without_mutation() {
        database_keychain::use_mock_keychain_for_tests();
        let _keychain = database_keychain::keychain_test_lock().await;
        let path = test_path();
        database_keychain::store_database_password("previous-keychain-secret").unwrap();
        let manager = DatabaseManager::load_sync(path.clone(), PathBuf::new()).unwrap();
        let replacement = DatabaseConfig {
            password: "inline-fallback-secret".into(),
            ..DatabaseConfig::default()
        };
        database_keychain::fail_next_mock_operation_for_tests();
        manager.persist_config(&replacement).await.unwrap();
        manager.publish_config(replacement, true).await.unwrap();
        assert_eq!(
            database_keychain::load_database_password()
                .unwrap()
                .as_deref(),
            Some("previous-keychain-secret")
        );
        assert!(std::fs::read_to_string(&path)
            .unwrap()
            .contains("inline-fallback-secret"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    fn test_path() -> PathBuf {
        let root = std::env::temp_dir().join(format!("kis-config-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        root.join("database_config.json")
    }

    async fn assert_failed_save_preserves_pair(write_then_fail: bool) {
        database_keychain::use_mock_keychain_for_tests();
        let _keychain = database_keychain::keychain_test_lock().await;
        let path = test_path();
        let old = DatabaseConfig {
            host: "old.synthetic.invalid".into(),
            password: String::new(),
            ..DatabaseConfig::default()
        };
        database_keychain::store_database_password("old-secret").unwrap();
        database_io::atomic_write_private_sync(&path, &serde_json::to_string(&old).unwrap())
            .unwrap();
        let replacement = DatabaseConfig {
            host: "new.synthetic.invalid".into(),
            password: "new-secret".into(),
            ..old
        };
        create_recovery_marker(&path).unwrap();
        let result = persist_using_writer(&path, &replacement, |path, content| {
            if write_then_fail {
                database_io::atomic_write_private_sync(path, content)?;
            }
            bail!("synthetic atomic write failure")
        });
        assert!(result.is_err());
        let reloaded =
            DatabaseManager::load_sync(path.clone(), path.parent().unwrap().join("data")).unwrap();
        let config = reloaded.config.read().await;
        assert_eq!(config.host, "old.synthetic.invalid");
        assert_eq!(config.password, "old-secret");
        assert!(!recovery_path(&path).exists());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn write_failure_restores_original_endpoint_and_keychain() {
        assert_failed_save_preserves_pair(false).await;
    }

    #[tokio::test]
    async fn failure_after_atomic_rename_restores_original_endpoint_and_keychain() {
        assert_failed_save_preserves_pair(true).await;
    }

    #[tokio::test]
    async fn uncertain_keychain_rollback_blocks_reload_save_and_pool() {
        database_keychain::use_mock_keychain_for_tests();
        let _keychain = database_keychain::keychain_test_lock().await;
        let path = test_path();
        database_keychain::store_database_password("old-secret").unwrap();
        let manager =
            DatabaseManager::load_sync(path.clone(), path.parent().unwrap().join("data")).unwrap();
        let replacement = DatabaseConfig {
            password: "new-secret".into(),
            ..DatabaseConfig::default()
        };
        create_recovery_marker(&path).unwrap();
        assert!(persist_using_writer(&path, &replacement, |_, _| {
            database_keychain::fail_next_mock_operation_for_tests();
            bail!("synthetic write failure")
        })
        .is_err());
        assert!(DatabaseManager::load_sync(path.clone(), PathBuf::new()).is_err());
        assert!(manager.persist_config(&replacement).await.is_err());
        assert!(manager.pool().await.is_err());
        assert!(!std::fs::read_to_string(recovery_path(&path))
            .unwrap()
            .contains("secret"));
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn committed_file_without_memory_publication_stays_fail_closed() {
        database_keychain::use_mock_keychain_for_tests();
        let _keychain = database_keychain::keychain_test_lock().await;
        let path = test_path();
        let manager =
            DatabaseManager::load_sync(path.clone(), path.parent().unwrap().join("data")).unwrap();
        let replacement = DatabaseConfig {
            host: "new.synthetic.invalid".into(),
            password: "new-secret".into(),
            ..DatabaseConfig::default()
        };
        // Simulate cancellation after the blocking commit, before the caller can publish it.
        manager.persist_config(&replacement).await.unwrap();
        assert!(DatabaseManager::load_sync(path.clone(), PathBuf::new()).is_err());
        assert!(manager.persist_config(&replacement).await.is_err());
        assert!(manager.pool().await.is_err());
        manager.publish_config(replacement, true).await.unwrap();
        assert_eq!(
            DatabaseManager::load_sync(path.clone(), PathBuf::new())
                .unwrap()
                .config_view()
                .await
                .host,
            "new.synthetic.invalid"
        );
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
