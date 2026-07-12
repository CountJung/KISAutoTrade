//! PostgreSQL/MariaDB-backed JSON document storage and Settings management.
//!
//! The existing JSON relative paths remain the logical document keys. Exactly one backend is
//! active at runtime: local JSON files or the configured database. Database credentials stay in
//! the app-data directory and are never returned by IPC or included in logical exports.

use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context, Result};
use sha2::{Digest, Sha256};
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode},
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    MySqlPool, PgPool,
};
use tokio::{fs, sync::Mutex};

const DOCUMENTS_TABLE: &str = "kisautotrade_documents";
const METADATA_TABLE: &str = "kisautotrade_metadata";
use super::database_schema::{NORMALIZED_TABLES, SCHEMA_VERSION};
const MAX_JSON_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_IMPORT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_IMPORT_FILES: usize = 10_000;
const MAX_DOCUMENT_KEY_BYTES: usize = 512;

static DATABASE_MANAGER: OnceLock<Arc<DatabaseManager>> = OnceLock::new();
use super::database_io::{atomic_write, atomic_write_private};
use super::database_types::DatabaseConfig;
pub use super::database_types::{
    DatabaseConfigView, DatabaseProvider, DatabaseStatusView, DatabaseTableStatus, DatabaseTlsMode,
    DatabaseTransferResult, JsonStorageCategoryView, JsonStorageInventoryView,
    SaveDatabaseConfigInput, StorageBackend,
};

#[derive(Clone)]
pub(super) enum DatabasePool {
    Postgresql(PgPool),
    Mariadb(MySqlPool),
}

#[derive(Clone)]
struct LocalDocument {
    key: String,
    category: String,
    payload: String,
    size_bytes: u64,
}

pub struct DatabaseManager {
    config_path: PathBuf,
    data_dir: PathBuf,
    export_dir: PathBuf,
    config: tokio::sync::RwLock<DatabaseConfig>,
    pool: Mutex<Option<(String, DatabasePool)>>,
    operation_lock: Mutex<()>,
}

impl DatabaseManager {
    pub fn load_sync(config_path: PathBuf, data_dir: PathBuf) -> Result<Self> {
        let mut config: DatabaseConfig = match std::fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content)
                .with_context(|| format!("DB 설정 파일이 손상되었습니다: {config_path:?}"))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => DatabaseConfig::default(),
            Err(error) => return Err(error).context("DB 설정 파일을 읽지 못했습니다."),
        };
        if config.password.is_empty() {
            match super::database_keychain::load_database_password() {
                Ok(Some(password)) => config.password = password,
                Ok(None) => {}
                Err(error) => tracing::warn!(
                    "OS keychain에서 DB password를 읽지 못했습니다 — 미설정 상태로 시작합니다: {error:#}"
                ),
            }
        } else {
            // 파일에 남아 있는 password를 OS keychain으로 1회 이전한다. 실패하면 파일(0o600) 저장 유지.
            match super::database_keychain::store_database_password(&config.password) {
                Ok(()) => {
                    let mut file_config = config.clone();
                    file_config.password = String::new();
                    let content = serde_json::to_string_pretty(&file_config)?;
                    match super::database_io::atomic_write_private_sync(&config_path, &content) {
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
            pool: Mutex::new(None),
            operation_lock: Mutex::new(()),
        })
    }

    pub async fn config_view(&self) -> DatabaseConfigView {
        DatabaseConfigView::from(&*self.config.read().await)
    }

    pub async fn save_config(&self, input: SaveDatabaseConfigInput) -> Result<DatabaseConfigView> {
        let _operation = self.operation_lock.lock().await;
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
        *self.config.write().await = config.clone();
        *self.pool.lock().await = None;
        Ok(DatabaseConfigView::from(&config))
    }

    pub async fn status(&self) -> Result<DatabaseStatusView> {
        let config = self.config.read().await.clone();
        config.validate()?;
        let started = Instant::now();
        let pool = self.pool().await?;
        let server_version = self.server_version(&pool).await?;
        let documents_exists = self.table_exists(&pool, DOCUMENTS_TABLE).await?;
        let metadata_exists = self.table_exists(&pool, METADATA_TABLE).await?;
        let schema_version = if metadata_exists {
            self.schema_version(&pool).await?
        } else {
            None
        };
        let documents_count = if documents_exists {
            self.row_count(&pool, DOCUMENTS_TABLE).await?
        } else {
            0
        };
        let metadata_count = if metadata_exists {
            self.row_count(&pool, METADATA_TABLE).await?
        } else {
            0
        };
        let mut normalized_status = Vec::new();
        let mut normalized_ready = true;
        for (table, purpose) in NORMALIZED_TABLES {
            let exists = self.table_exists(&pool, table).await?;
            normalized_ready &= exists;
            normalized_status.push(DatabaseTableStatus {
                name: table.to_string(),
                purpose: purpose.to_string(),
                exists,
                row_count: if exists {
                    self.row_count(&pool, table).await?
                } else {
                    0
                },
            });
        }
        let ready = documents_exists
            && metadata_exists
            && normalized_ready
            && schema_version == Some(SCHEMA_VERSION);
        Ok(DatabaseStatusView {
            connected: true,
            provider: config.provider,
            active_backend: config.active_backend,
            schema_version,
            required_schema_version: SCHEMA_VERSION,
            server_version: Some(server_version),
            latency_ms: Some(started.elapsed().as_millis() as u64),
            checked_at: chrono::Utc::now().to_rfc3339(),
            message: if ready {
                "DB 연결과 KISAutoTrade 테이블이 정상입니다.".to_string()
            } else {
                "DB 연결은 정상이지만 앱 테이블 생성 또는 업그레이드가 필요합니다.".to_string()
            },
            tables: {
                let mut tables = vec![
                    DatabaseTableStatus {
                        name: DOCUMENTS_TABLE.to_string(),
                        purpose: "JSON 데이터 문서".to_string(),
                        exists: documents_exists,
                        row_count: documents_count,
                    },
                    DatabaseTableStatus {
                        name: METADATA_TABLE.to_string(),
                        purpose: "스키마 버전과 관리 메타데이터".to_string(),
                        exists: metadata_exists,
                        row_count: metadata_count,
                    },
                ];
                tables.extend(normalized_status);
                tables
            },
        })
    }

    pub async fn create_tables(&self) -> Result<DatabaseStatusView> {
        let _operation = self.operation_lock.lock().await;
        let pool = self.pool().await?;
        if self.table_exists(&pool, METADATA_TABLE).await? {
            if let Some(version) = self.schema_version(&pool).await? {
                if version > SCHEMA_VERSION {
                    bail!(
                        "DB schema version {version}은 이 앱이 지원하는 v{SCHEMA_VERSION}보다 새 버전입니다. 앱을 업그레이드하세요."
                    );
                }
            }
        }
        match &pool {
            DatabasePool::Postgresql(pool) => {
                sqlx::query(&format!(
                    "CREATE TABLE IF NOT EXISTS {DOCUMENTS_TABLE} (\
                     document_key VARCHAR(512) PRIMARY KEY, category VARCHAR(128) NOT NULL, \
                     payload TEXT NOT NULL, size_bytes BIGINT NOT NULL, updated_at VARCHAR(64) NOT NULL)"
                ))
                .execute(pool)
                .await?;
                sqlx::query(&format!(
                    "CREATE TABLE IF NOT EXISTS {METADATA_TABLE} (\
                     meta_key VARCHAR(128) PRIMARY KEY, meta_value TEXT NOT NULL, updated_at VARCHAR(64) NOT NULL)"
                ))
                .execute(pool)
                .await?;
            }
            DatabasePool::Mariadb(pool) => {
                sqlx::query(&format!(
                    "CREATE TABLE IF NOT EXISTS {DOCUMENTS_TABLE} (\
                     document_key VARCHAR(512) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin PRIMARY KEY, category VARCHAR(128) NOT NULL, \
                     payload LONGTEXT NOT NULL, size_bytes BIGINT NOT NULL, updated_at VARCHAR(64) NOT NULL) ENGINE=InnoDB"
                ))
                .execute(pool)
                .await?;
                sqlx::query(&format!(
                    "CREATE TABLE IF NOT EXISTS {METADATA_TABLE} (\
                     meta_key VARCHAR(128) PRIMARY KEY, meta_value LONGTEXT NOT NULL, updated_at VARCHAR(64) NOT NULL) ENGINE=InnoDB"
                ))
                .execute(pool)
                .await?;
            }
        }
        super::database_schema::create(&pool).await?;
        let documents = self.fetch_documents(&pool).await?;
        let projection_documents: Vec<_> = documents
            .iter()
            .map(|document| (document.key.as_str(), document.payload.as_str()))
            .collect();
        super::database_projection::rebuild(&pool, &projection_documents).await?;
        self.upsert_metadata(&pool, "schema_version", &SCHEMA_VERSION.to_string())
            .await?;
        drop(_operation);
        self.status().await
    }

    pub async fn clear_tables(&self, confirmation: &str) -> Result<DatabaseStatusView> {
        if confirmation != "CLEAR KISAUTOTRADE DATA" {
            bail!("확인 문구가 일치하지 않습니다: CLEAR KISAUTOTRADE DATA");
        }
        let _operation = self.operation_lock.lock().await;
        self.ensure_json_backend().await?;
        let pool = self.pool().await?;
        match &pool {
            DatabasePool::Postgresql(pool) => {
                let mut transaction = pool.begin().await?;
                for (table, _) in NORMALIZED_TABLES.iter().rev() {
                    sqlx::query(&format!("DELETE FROM {table}"))
                        .execute(&mut *transaction)
                        .await?;
                }
                sqlx::query(&format!("DELETE FROM {DOCUMENTS_TABLE}"))
                    .execute(&mut *transaction)
                    .await?;
                transaction.commit().await?;
            }
            DatabasePool::Mariadb(pool) => {
                let mut transaction = pool.begin().await?;
                for (table, _) in NORMALIZED_TABLES.iter().rev() {
                    sqlx::query(&format!("DELETE FROM {table}"))
                        .execute(&mut *transaction)
                        .await?;
                }
                sqlx::query(&format!("DELETE FROM {DOCUMENTS_TABLE}"))
                    .execute(&mut *transaction)
                    .await?;
                transaction.commit().await?;
            }
        }
        drop(_operation);
        self.status().await
    }

    pub async fn drop_tables(&self, confirmation: &str) -> Result<DatabaseStatusView> {
        if confirmation != "DROP KISAUTOTRADE TABLES" {
            bail!("확인 문구가 일치하지 않습니다: DROP KISAUTOTRADE TABLES");
        }
        let _operation = self.operation_lock.lock().await;
        self.ensure_json_backend().await?;
        let pool = self.pool().await?;
        super::database_schema::drop(&pool).await?;
        match &pool {
            DatabasePool::Postgresql(pool) => {
                sqlx::query(&format!("DROP TABLE IF EXISTS {DOCUMENTS_TABLE}"))
                    .execute(pool)
                    .await?;
                sqlx::query(&format!("DROP TABLE IF EXISTS {METADATA_TABLE}"))
                    .execute(pool)
                    .await?;
            }
            DatabasePool::Mariadb(pool) => {
                sqlx::query(&format!("DROP TABLE IF EXISTS {DOCUMENTS_TABLE}"))
                    .execute(pool)
                    .await?;
                sqlx::query(&format!("DROP TABLE IF EXISTS {METADATA_TABLE}"))
                    .execute(pool)
                    .await?;
            }
        }
        drop(_operation);
        self.status().await
    }

    pub async fn json_inventory(&self) -> Result<JsonStorageInventoryView> {
        let documents = scan_json_documents(self.data_dir.clone()).await?;
        Ok(inventory_from_documents(&documents))
    }

    pub async fn import_json(&self) -> Result<DatabaseTransferResult> {
        let _operation = self.operation_lock.lock().await;
        self.ensure_json_backend().await?;
        let documents = scan_json_documents(self.data_dir.clone()).await?;
        let pool = self.pool().await?;
        if !self.table_exists(&pool, DOCUMENTS_TABLE).await? {
            bail!("먼저 KISAutoTrade DB 테이블을 생성하세요.");
        }
        self.ensure_transactional_documents_table(&pool).await?;
        let mut checksum = Sha256::new();
        let mut size_bytes = 0_u64;
        for document in &documents {
            checksum.update(document.key.as_bytes());
            checksum.update(document.payload.as_bytes());
            size_bytes = size_bytes.saturating_add(document.size_bytes);
        }
        self.replace_documents_transaction(&pool, &documents)
            .await?;
        Ok(DatabaseTransferResult {
            operation: "jsonToDatabase".to_string(),
            processed: documents.len(),
            inserted_or_updated: documents.len(),
            skipped: 0,
            size_bytes,
            output_path: None,
            checksum: format!("{:x}", checksum.finalize()),
            completed_at: chrono::Utc::now().to_rfc3339(),
            message: format!("JSON {}개 문서를 DB에 저장했습니다.", documents.len()),
        })
    }

    pub async fn export_json(&self) -> Result<DatabaseTransferResult> {
        let _operation = self.operation_lock.lock().await;
        let pool = self.pool().await?;
        self.validate_document_set_size(&pool).await?;
        let documents = self.fetch_documents(&pool).await?;
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let suffix = &uuid::Uuid::new_v4().simple().to_string()[..8];
        let output_dir = self.export_dir.join(format!("db-export-{stamp}-{suffix}"));
        fs::create_dir_all(&output_dir).await?;
        let mut checksum = Sha256::new();
        let mut size_bytes = 0_u64;
        for document in &documents {
            let path = safe_export_path(&output_dir, &document.key)?;
            atomic_write(&path, &document.payload).await?;
            checksum.update(document.key.as_bytes());
            checksum.update(document.payload.as_bytes());
            size_bytes = size_bytes.saturating_add(document.size_bytes);
        }
        let checksum = format!("{:x}", checksum.finalize());
        let manifest = serde_json::json!({
            "formatVersion": 1,
            "generatedAt": chrono::Utc::now().to_rfc3339(),
            "documentCount": documents.len(),
            "sizeBytes": size_bytes,
            "checksumSha256": checksum,
            "excluded": ["profiles.json", "secure_config.json", ".env", "logs"]
        });
        atomic_write(
            &output_dir.join("manifest.json"),
            &serde_json::to_string_pretty(&manifest)?,
        )
        .await?;
        Ok(DatabaseTransferResult {
            operation: "databaseToJson".to_string(),
            processed: documents.len(),
            inserted_or_updated: documents.len(),
            skipped: 0,
            size_bytes,
            output_path: Some(output_dir.to_string_lossy().to_string()),
            checksum,
            completed_at: chrono::Utc::now().to_rfc3339(),
            message: format!(
                "DB {}개 문서를 JSON 스냅샷으로 반출했습니다.",
                documents.len()
            ),
        })
    }

    pub async fn set_backend(&self, backend: StorageBackend) -> Result<DatabaseStatusView> {
        let _operation = self.operation_lock.lock().await;
        let current_backend = self.config.read().await.active_backend;
        if backend == StorageBackend::Database {
            let pool = self.pool().await?;
            if !self.table_exists(&pool, DOCUMENTS_TABLE).await?
                || !self.table_exists(&pool, METADATA_TABLE).await?
                || self.schema_version(&pool).await? != Some(SCHEMA_VERSION)
            {
                bail!("DB 테이블 생성과 스키마 검증을 먼저 완료하세요.");
            }
            let inventory = scan_json_documents(self.data_dir.clone()).await?;
            for document in &inventory {
                match self.fetch_document(&pool, &document.key).await? {
                    None => bail!(
                        "JSON → DB 가져오기를 먼저 완료하세요. 누락 문서: {}",
                        document.key
                    ),
                    Some(payload) if payload != document.payload => bail!(
                        "JSON → DB 가져오기를 다시 실행하세요. 변경된 문서: {}",
                        document.key
                    ),
                    Some(_) => {}
                }
            }
        } else if current_backend == StorageBackend::Database {
            // DB가 source of truth인 동안 모든 문서를 local JSON으로 복원한 뒤에만 전환한다.
            let pool = self.pool().await?;
            self.validate_document_set_size(&pool).await?;
            let documents = self.fetch_documents(&pool).await?;
            let database_keys: std::collections::HashSet<_> =
                documents.iter().map(|document| document.key.as_str()).collect();
            for local in scan_json_documents(self.data_dir.clone()).await? {
                if !database_keys.contains(local.key.as_str()) {
                    let path = safe_export_path(&self.data_dir, &local.key)?;
                    match fs::remove_file(&path).await {
                        Ok(()) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error).context("DB에 없는 local JSON 제거 실패"),
                    }
                }
            }
            for document in &documents {
                let path = safe_export_path(&self.data_dir, &document.key)?;
                atomic_write(&path, &document.payload).await?;
            }
        }
        let mut config = self.config.read().await.clone();
        config.active_backend = backend;
        self.persist_config(&config).await?;
        *self.config.write().await = config;
        drop(_operation);
        self.status().await
    }

    pub async fn read_document(&self, path: &Path) -> Result<Option<String>> {
        let _operation = self.operation_lock.lock().await;
        let config = self.config.read().await.clone();
        if config.active_backend == StorageBackend::Json {
            return match fs::read_to_string(path).await {
                Ok(content) => Ok(Some(content)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error.into()),
            };
        }
        let key = document_key(&self.data_dir, path)?;
        let pool = self.pool().await?;
        self.fetch_document(&pool, &key).await
    }

    pub async fn write_document(&self, path: &Path, content: &str) -> Result<()> {
        let _operation = self.operation_lock.lock().await;
        serde_json::from_str::<serde_json::Value>(content)
            .context("저장할 데이터가 올바른 JSON이 아닙니다")?;
        if content.len() as u64 > MAX_JSON_FILE_BYTES {
            bail!("JSON 문서가 파일당 16MB 상한을 초과했습니다.");
        }
        let config = self.config.read().await.clone();
        if config.active_backend == StorageBackend::Json {
            return atomic_write(path, content).await;
        }
        let key = document_key(&self.data_dir, path)?;
        let document = LocalDocument {
            category: category_from_key(&key),
            key,
            payload: content.to_string(),
            size_bytes: content.len() as u64,
        };
        let pool = self.pool().await?;
        if !self.table_exists(&pool, DOCUMENTS_TABLE).await? {
            bail!("DB 저장 테이블이 없습니다. Settings에서 테이블을 생성하세요.");
        }
        self.upsert_document(&pool, &document).await
    }

    async fn ensure_json_backend(&self) -> Result<()> {
        if self.config.read().await.active_backend == StorageBackend::Database {
            bail!("DB 저장이 활성화된 동안에는 데이터를 비우거나 테이블을 삭제할 수 없습니다.");
        }
        Ok(())
    }

    async fn ensure_transactional_documents_table(&self, pool: &DatabasePool) -> Result<()> {
        if let DatabasePool::Mariadb(pool) = pool {
            let engine = sqlx::query_scalar::<_, Option<String>>(
                "SELECT ENGINE FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = ?",
            )
            .bind(DOCUMENTS_TABLE)
            .fetch_one(pool)
            .await?;
            if !engine.is_some_and(|value| value.eq_ignore_ascii_case("InnoDB")) {
                bail!("MariaDB 문서 테이블은 transaction rollback을 위해 InnoDB여야 합니다.");
            }
        }
        Ok(())
    }

    /// 설정을 저장한다. password는 OS keychain에 우선 저장하고 파일에서는 제거한다.
    /// keychain을 쓸 수 없는 환경에서만 기존 파일(0o600) 저장으로 fallback한다.
    async fn persist_config(&self, config: &DatabaseConfig) -> Result<()> {
        let mut file_config = config.clone();
        if file_config.password.is_empty() {
            // password 미설정: 남아 있을 수 있는 keychain 항목도 정리해 상태를 일치시킨다.
            let deleted =
                tokio::task::spawn_blocking(super::database_keychain::delete_database_password)
                    .await
                    .context("keychain 삭제 task 실패")?;
            if let Err(error) = deleted {
                tracing::warn!("OS keychain의 DB password 삭제 실패 (무시): {error:#}");
            }
        } else {
            let password = file_config.password.clone();
            let stored = tokio::task::spawn_blocking(move || {
                super::database_keychain::store_database_password(&password)
            })
            .await
            .context("keychain 저장 task 실패")?;
            match stored {
                Ok(()) => file_config.password = String::new(),
                Err(error) => tracing::warn!(
                    "OS keychain 저장 실패 — DB password를 파일(0o600)에 유지합니다: {error:#}"
                ),
            }
        }
        let content = serde_json::to_string_pretty(&file_config)?;
        atomic_write_private(&self.config_path, &content).await?;
        Ok(())
    }

    pub(super) async fn pool(&self) -> Result<DatabasePool> {
        let config = self.config.read().await.clone();
        config.validate()?;
        let fingerprint = config.fingerprint();
        if let Some((cached_fingerprint, pool)) = &*self.pool.lock().await {
            if cached_fingerprint == &fingerprint {
                return Ok(pool.clone());
            }
        }
        let pool = connect_pool(&config).await?;
        *self.pool.lock().await = Some((fingerprint, pool.clone()));
        Ok(pool)
    }

    pub async fn database_archive_stats(
        &self,
    ) -> Result<super::database_archive::DatabaseArchiveStats> {
        let _operation = self.operation_lock.lock().await;
        super::database_archive::stats(&self.pool().await?).await
    }

    pub async fn purge_database_trades(&self, retention_days: u32, max_size_mb: u64) -> Result<()> {
        let _operation = self.operation_lock.lock().await;
        super::database_archive::purge(&self.pool().await?, retention_days, max_size_mb).await
    }

    async fn server_version(&self, pool: &DatabasePool) -> Result<String> {
        Ok(match pool {
            DatabasePool::Postgresql(pool) => {
                sqlx::query_scalar::<_, String>("SHOW server_version")
                    .fetch_one(pool)
                    .await?
            }
            DatabasePool::Mariadb(pool) => {
                sqlx::query_scalar::<_, String>("SELECT VERSION()")
                    .fetch_one(pool)
                    .await?
            }
        })
    }

    async fn table_exists(&self, pool: &DatabasePool, table: &str) -> Result<bool> {
        Ok(match pool {
            DatabasePool::Postgresql(pool) => sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = current_schema() AND table_name = $1)",
            )
            .bind(table)
            .fetch_one(pool)
            .await?,
            DatabasePool::Mariadb(pool) => {
                let count = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = ?",
                )
                .bind(table)
                .fetch_one(pool)
                .await?;
                count > 0
            }
        })
    }

    async fn row_count(&self, pool: &DatabasePool, table: &str) -> Result<u64> {
        if table != DOCUMENTS_TABLE
            && table != METADATA_TABLE
            && !NORMALIZED_TABLES
                .iter()
                .any(|(allowed, _)| *allowed == table)
        {
            bail!("허용되지 않은 DB 테이블입니다.");
        }
        let sql = format!("SELECT COUNT(*) FROM {table}");
        let count = match pool {
            DatabasePool::Postgresql(pool) => {
                sqlx::query_scalar::<_, i64>(&sql).fetch_one(pool).await?
            }
            DatabasePool::Mariadb(pool) => {
                sqlx::query_scalar::<_, i64>(&sql).fetch_one(pool).await?
            }
        };
        Ok(count.max(0) as u64)
    }

    async fn schema_version(&self, pool: &DatabasePool) -> Result<Option<i32>> {
        let value = match pool {
            DatabasePool::Postgresql(pool) => {
                sqlx::query_scalar::<_, String>(&format!(
                    "SELECT meta_value FROM {METADATA_TABLE} WHERE meta_key = $1"
                ))
                .bind("schema_version")
                .fetch_optional(pool)
                .await?
            }
            DatabasePool::Mariadb(pool) => {
                sqlx::query_scalar::<_, String>(&format!(
                    "SELECT meta_value FROM {METADATA_TABLE} WHERE meta_key = ?"
                ))
                .bind("schema_version")
                .fetch_optional(pool)
                .await?
            }
        };
        Ok(value.and_then(|value| value.parse().ok()))
    }

    async fn upsert_metadata(&self, pool: &DatabasePool, key: &str, value: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        match pool {
            DatabasePool::Postgresql(pool) => {
                sqlx::query(&format!(
                    "INSERT INTO {METADATA_TABLE} (meta_key, meta_value, updated_at) VALUES ($1, $2, $3) \
                     ON CONFLICT (meta_key) DO UPDATE SET meta_value = EXCLUDED.meta_value, updated_at = EXCLUDED.updated_at"
                ))
                .bind(key)
                .bind(value)
                .bind(now)
                .execute(pool)
                .await?;
            }
            DatabasePool::Mariadb(pool) => {
                sqlx::query(&format!(
                    "INSERT INTO {METADATA_TABLE} (meta_key, meta_value, updated_at) VALUES (?, ?, ?) \
                     ON DUPLICATE KEY UPDATE meta_value = VALUES(meta_value), updated_at = VALUES(updated_at)"
                ))
                .bind(key)
                .bind(value)
                .bind(now)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    async fn upsert_document(&self, pool: &DatabasePool, document: &LocalDocument) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        match pool {
            DatabasePool::Postgresql(pool) => {
                let mut transaction = pool.begin().await?;
                sqlx::query(&format!(
                    "INSERT INTO {DOCUMENTS_TABLE} (document_key, category, payload, size_bytes, updated_at) \
                     VALUES ($1, $2, $3, $4, $5) ON CONFLICT (document_key) DO UPDATE SET \
                     category = EXCLUDED.category, payload = EXCLUDED.payload, \
                     size_bytes = EXCLUDED.size_bytes, updated_at = EXCLUDED.updated_at"
                ))
                .bind(&document.key)
                .bind(&document.category)
                .bind(&document.payload)
                .bind(document.size_bytes as i64)
                .bind(now)
                .execute(&mut *transaction)
                .await?;
                super::database_projection::project_postgres(
                    &mut transaction,
                    &document.key,
                    &document.payload,
                )
                .await?;
                transaction.commit().await?;
            }
            DatabasePool::Mariadb(pool) => {
                let mut transaction = pool.begin().await?;
                sqlx::query(&format!(
                    "INSERT INTO {DOCUMENTS_TABLE} (document_key, category, payload, size_bytes, updated_at) \
                     VALUES (?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE category = VALUES(category), \
                     payload = VALUES(payload), size_bytes = VALUES(size_bytes), updated_at = VALUES(updated_at)"
                ))
                .bind(&document.key)
                .bind(&document.category)
                .bind(&document.payload)
                .bind(document.size_bytes as i64)
                .bind(now)
                .execute(&mut *transaction)
                .await?;
                super::database_projection::project_mariadb(
                    &mut transaction,
                    &document.key,
                    &document.payload,
                )
                .await?;
                transaction.commit().await?;
            }
        }
        Ok(())
    }

    async fn replace_documents_transaction(
        &self,
        pool: &DatabasePool,
        documents: &[LocalDocument],
    ) -> Result<()> {
        match pool {
            DatabasePool::Postgresql(pool) => {
                let mut transaction = pool.begin().await?;
                sqlx::query(&format!("DELETE FROM {DOCUMENTS_TABLE}"))
                    .execute(&mut *transaction)
                    .await?;
                for document in documents {
                    sqlx::query(&format!(
                        "INSERT INTO {DOCUMENTS_TABLE} (document_key, category, payload, size_bytes, updated_at) \
                         VALUES ($1, $2, $3, $4, $5) ON CONFLICT (document_key) DO UPDATE SET \
                         category = EXCLUDED.category, payload = EXCLUDED.payload, \
                         size_bytes = EXCLUDED.size_bytes, updated_at = EXCLUDED.updated_at"
                    ))
                    .bind(&document.key)
                    .bind(&document.category)
                    .bind(&document.payload)
                    .bind(document.size_bytes as i64)
                    .bind(chrono::Utc::now().to_rfc3339())
                    .execute(&mut *transaction)
                    .await?;
                }
                let projection_documents: Vec<_> = documents
                    .iter()
                    .map(|document| (document.key.as_str(), document.payload.as_str()))
                    .collect();
                super::database_projection::rebuild_postgres(
                    &mut transaction,
                    &projection_documents,
                )
                .await?;
                transaction.commit().await?;
            }
            DatabasePool::Mariadb(pool) => {
                let mut transaction = pool.begin().await?;
                sqlx::query(&format!("DELETE FROM {DOCUMENTS_TABLE}"))
                    .execute(&mut *transaction)
                    .await?;
                for document in documents {
                    sqlx::query(&format!(
                        "INSERT INTO {DOCUMENTS_TABLE} (document_key, category, payload, size_bytes, updated_at) \
                         VALUES (?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE category = VALUES(category), \
                         payload = VALUES(payload), size_bytes = VALUES(size_bytes), updated_at = VALUES(updated_at)"
                    ))
                    .bind(&document.key)
                    .bind(&document.category)
                    .bind(&document.payload)
                    .bind(document.size_bytes as i64)
                    .bind(chrono::Utc::now().to_rfc3339())
                    .execute(&mut *transaction)
                    .await?;
                }
                let projection_documents: Vec<_> = documents
                    .iter()
                    .map(|document| (document.key.as_str(), document.payload.as_str()))
                    .collect();
                super::database_projection::rebuild_mariadb(
                    &mut transaction,
                    &projection_documents,
                )
                .await?;
                transaction.commit().await?;
            }
        }
        Ok(())
    }

    async fn validate_document_set_size(&self, pool: &DatabasePool) -> Result<()> {
        let (count, bytes) = match pool {
            DatabasePool::Postgresql(pool) => {
                let count = sqlx::query_scalar::<_, i64>(&format!(
                    "SELECT COUNT(*) FROM {DOCUMENTS_TABLE}"
                ))
                .fetch_one(pool)
                .await?;
                let bytes = sqlx::query_scalar::<_, i64>(&format!(
                    "SELECT CAST(COALESCE(SUM(OCTET_LENGTH(payload)), 0) AS BIGINT) FROM {DOCUMENTS_TABLE}"
                ))
                .fetch_one(pool)
                .await?;
                (count, bytes)
            }
            DatabasePool::Mariadb(pool) => {
                let count = sqlx::query_scalar::<_, i64>(&format!(
                    "SELECT COUNT(*) FROM {DOCUMENTS_TABLE}"
                ))
                .fetch_one(pool)
                .await?;
                let bytes = sqlx::query_scalar::<_, i64>(&format!(
                    "SELECT CAST(COALESCE(SUM(OCTET_LENGTH(payload)), 0) AS SIGNED) FROM {DOCUMENTS_TABLE}"
                ))
                .fetch_one(pool)
                .await?;
                (count, bytes)
            }
        };
        if count < 0 || count as usize > MAX_IMPORT_FILES {
            bail!("DB 문서 수가 10,000개 상한을 초과했습니다.");
        }
        if bytes < 0 || bytes as u64 > MAX_IMPORT_BYTES {
            bail!("DB 문서 전체 크기가 512MB 상한을 초과했습니다.");
        }
        Ok(())
    }

    async fn fetch_document(&self, pool: &DatabasePool, key: &str) -> Result<Option<String>> {
        Ok(match pool {
            DatabasePool::Postgresql(pool) => {
                sqlx::query_scalar::<_, String>(&format!(
                    "SELECT payload FROM {DOCUMENTS_TABLE} WHERE document_key = $1"
                ))
                .bind(key)
                .fetch_optional(pool)
                .await?
            }
            DatabasePool::Mariadb(pool) => {
                sqlx::query_scalar::<_, String>(&format!(
                    "SELECT payload FROM {DOCUMENTS_TABLE} WHERE document_key = ?"
                ))
                .bind(key)
                .fetch_optional(pool)
                .await?
            }
        })
    }

    async fn fetch_documents(&self, pool: &DatabasePool) -> Result<Vec<LocalDocument>> {
        let rows: Vec<(String, String, String, i64)> = match pool {
            DatabasePool::Postgresql(pool) => sqlx::query_as(&format!(
                "SELECT document_key, category, payload, size_bytes FROM {DOCUMENTS_TABLE} ORDER BY document_key"
            ))
            .fetch_all(pool)
            .await?,
            DatabasePool::Mariadb(pool) => sqlx::query_as(&format!(
                "SELECT document_key, category, payload, size_bytes FROM {DOCUMENTS_TABLE} ORDER BY document_key"
            ))
            .fetch_all(pool)
            .await?,
        };
        Ok(rows
            .into_iter()
            .map(|(key, category, payload, size_bytes)| LocalDocument {
                key,
                category,
                payload,
                size_bytes: size_bytes.max(0) as u64,
            })
            .collect())
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

pub fn install_database_manager(manager: Arc<DatabaseManager>) -> Result<()> {
    DATABASE_MANAGER
        .set(manager)
        .map_err(|_| anyhow!("DatabaseManager가 이미 초기화되었습니다."))
}

pub async fn read_managed_json(path: &Path) -> Result<Option<String>> {
    if let Some(manager) = DATABASE_MANAGER.get() {
        manager.read_document(path).await
    } else {
        match fs::read_to_string(path).await {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

pub async fn write_managed_json(path: &Path, content: &str) -> Result<()> {
    if let Some(manager) = DATABASE_MANAGER.get() {
        manager.write_document(path, content).await
    } else {
        atomic_write(path, content).await
    }
}

pub async fn read_managed_json_backup(path: &Path) -> Result<Option<String>> {
    if let Some(manager) = DATABASE_MANAGER.get() {
        if manager.config.read().await.active_backend == StorageBackend::Database {
            return Ok(None);
        }
    }
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.json");
    let backup = path.with_file_name(format!("{file_name}.bak"));
    match fs::read_to_string(backup).await {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub async fn quarantine_corrupt_managed_json(path: &Path) -> Result<()> {
    if let Some(manager) = DATABASE_MANAGER.get() {
        if manager.config.read().await.active_backend == StorageBackend::Database {
            return Ok(());
        }
    }
    if !fs::try_exists(path).await? {
        return Ok(());
    }
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.json");
    let quarantine = path.with_file_name(format!(
        "{file_name}.corrupt-{}",
        chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ")
    ));
    fs::rename(path, quarantine).await?;
    Ok(())
}

fn document_key(data_dir: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(data_dir).with_context(|| {
        format!("관리 데이터 경로 밖의 JSON은 DB에 저장할 수 없습니다: {path:?}")
    })?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            _ => bail!("JSON document key에 허용되지 않은 경로가 포함되어 있습니다."),
        }
    }
    if parts.is_empty() || !parts.last().is_some_and(|name| name.ends_with(".json")) {
        bail!("DB 저장 대상은 data 디렉토리 아래 JSON 파일이어야 합니다.");
    }
    let key = parts.join("/");
    if key.len() > MAX_DOCUMENT_KEY_BYTES {
        bail!("JSON document key가 512 byte 상한을 초과했습니다.");
    }
    Ok(key)
}

fn category_from_key(key: &str) -> String {
    key.split('/').next().unwrap_or("root").to_string()
}

fn safe_export_path(root: &Path, key: &str) -> Result<PathBuf> {
    if key.len() > MAX_DOCUMENT_KEY_BYTES {
        bail!("DB document key가 512 byte 상한을 초과했습니다.");
    }
    let mut path = root.to_path_buf();
    for part in key.split('/') {
        if part.is_empty() || part == "." || part == ".." || part.contains(['\\', ':']) {
            bail!("DB document key에 안전하지 않은 경로가 포함되어 있습니다.");
        }
        path.push(part);
    }
    if path.extension().and_then(|value| value.to_str()) != Some("json") {
        bail!("DB export 대상은 JSON 문서만 허용됩니다.");
    }
    Ok(path)
}

async fn scan_json_documents(data_dir: PathBuf) -> Result<Vec<LocalDocument>> {
    tokio::task::spawn_blocking(move || scan_json_documents_sync(&data_dir))
        .await
        .context("JSON inventory task 실패")?
}

fn scan_json_documents_sync(data_dir: &Path) -> Result<Vec<LocalDocument>> {
    if !data_dir.exists() {
        return Ok(Vec::new());
    }
    let mut stack = vec![data_dir.to_path_buf()];
    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let size = entry.metadata()?.len();
            if size > MAX_JSON_FILE_BYTES {
                bail!("JSON 파일당 16MB 상한 초과: {path:?}");
            }
            total_bytes = total_bytes.saturating_add(size);
            if total_bytes > MAX_IMPORT_BYTES {
                bail!("JSON 가져오기 전체 512MB 상한을 초과했습니다.");
            }
            if documents.len() >= MAX_IMPORT_FILES {
                bail!("JSON 가져오기 파일 수 10,000개 상한을 초과했습니다.");
            }
            let payload = std::fs::read_to_string(&path)?;
            serde_json::from_str::<serde_json::Value>(&payload)
                .with_context(|| format!("올바르지 않은 JSON 파일: {path:?}"))?;
            let key = document_key(data_dir, &path)?;
            documents.push(LocalDocument {
                category: category_from_key(&key),
                key,
                payload,
                size_bytes: size,
            });
        }
    }
    documents.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(documents)
}

fn inventory_from_documents(documents: &[LocalDocument]) -> JsonStorageInventoryView {
    let mut categories: BTreeMap<String, (usize, u64)> = BTreeMap::new();
    for document in documents {
        let entry = categories.entry(document.category.clone()).or_default();
        entry.0 += 1;
        entry.1 = entry.1.saturating_add(document.size_bytes);
    }
    JsonStorageInventoryView {
        file_count: documents.len(),
        size_bytes: documents.iter().map(|document| document.size_bytes).sum(),
        categories: categories
            .into_iter()
            .map(
                |(category, (file_count, size_bytes))| JsonStorageCategoryView {
                    category,
                    file_count,
                    size_bytes,
                },
            )
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_key_rejects_paths_outside_data_dir() {
        let root = PathBuf::from("/tmp/kis-data");
        assert!(document_key(&root, Path::new("/tmp/secret.json")).is_err());
    }

    #[test]
    fn safe_export_path_rejects_parent_traversal() {
        assert!(safe_export_path(Path::new("/tmp/export"), "../profiles.json").is_err());
        assert!(safe_export_path(Path::new("/tmp/export"), "trades/ok.json").is_ok());
    }

    #[test]
    fn config_view_never_contains_password() {
        let config = DatabaseConfig {
            password: "secret-value".to_string(),
            ..DatabaseConfig::default()
        };
        let json = serde_json::to_string(&DatabaseConfigView::from(&config)).unwrap();
        assert!(!json.contains("secret-value"));
        assert!(json.contains("passwordConfigured"));
    }

    #[test]
    fn inventory_groups_documents_by_top_level_category() {
        let documents = vec![
            LocalDocument {
                key: "trades/2026/07/11/trades.json".to_string(),
                category: "trades".to_string(),
                payload: "[]".to_string(),
                size_bytes: 2,
            },
            LocalDocument {
                key: "orders/2026/07/11/orders.json".to_string(),
                category: "orders".to_string(),
                payload: "[]".to_string(),
                size_bytes: 2,
            },
        ];
        let inventory = inventory_from_documents(&documents);
        assert_eq!(inventory.file_count, 2);
        assert_eq!(inventory.categories.len(), 2);
    }

    fn keychain_test_dirs() -> (PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!("kisat-db-keychain-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(base.join("data")).expect("data dir");
        (base.join("database_config.json"), base.join("data"))
    }

    fn save_input_with_password(password: &str) -> SaveDatabaseConfigInput {
        SaveDatabaseConfigInput {
            provider: DatabaseProvider::Postgresql,
            host: "127.0.0.1".to_string(),
            port: 5432,
            database: "kisautotrade".to_string(),
            username: "kisautotrade".to_string(),
            password: Some(password.to_string()),
            tls_mode: DatabaseTlsMode::Prefer,
            max_connections: 5,
        }
    }

    #[tokio::test]
    async fn save_config_stores_password_in_keychain_not_in_file() {
        super::super::database_keychain::use_mock_keychain_for_tests();
        let _keychain = super::super::database_keychain::keychain_test_lock();
        let (config_path, data_dir) = keychain_test_dirs();
        let manager =
            DatabaseManager::load_sync(config_path.clone(), data_dir.clone()).expect("load");
        let view = manager
            .save_config(save_input_with_password("keychain-secret-1"))
            .await
            .expect("save config");
        assert!(view.password_configured);

        let file_content = std::fs::read_to_string(&config_path).expect("config file");
        assert!(
            !file_content.contains("keychain-secret-1"),
            "password가 설정 파일에 남아 있으면 안 된다"
        );

        // 재시작 시뮬레이션: 새 인스턴스가 keychain에서 password를 복원한다.
        let reloaded = DatabaseManager::load_sync(config_path, data_dir).expect("reload");
        let view = reloaded.config_view().await;
        assert!(view.password_configured);
    }

    #[tokio::test]
    async fn legacy_file_password_migrates_to_keychain_on_load() {
        super::super::database_keychain::use_mock_keychain_for_tests();
        let _keychain = super::super::database_keychain::keychain_test_lock();
        let (config_path, data_dir) = keychain_test_dirs();
        let legacy = DatabaseConfig {
            password: "legacy-file-password".to_string(),
            ..DatabaseConfig::default()
        };
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&legacy).expect("legacy json"),
        )
        .expect("write legacy config");

        let manager = DatabaseManager::load_sync(config_path.clone(), data_dir).expect("load");
        let view = manager.config_view().await;
        assert!(
            view.password_configured,
            "이전 후에도 password는 유효해야 한다"
        );

        let file_content = std::fs::read_to_string(&config_path).expect("config file");
        assert!(
            !file_content.contains("legacy-file-password"),
            "migration 후 설정 파일에 password가 남아 있으면 안 된다"
        );
        assert_eq!(
            super::super::database_keychain::load_database_password()
                .expect("keychain read")
                .as_deref(),
            Some("legacy-file-password")
        );
    }
}
