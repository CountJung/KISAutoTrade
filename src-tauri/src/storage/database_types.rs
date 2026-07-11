use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseProvider {
    Postgresql,
    Mariadb,
}

impl DatabaseProvider {
    fn default_port(self) -> u16 {
        match self {
            Self::Postgresql => 5432,
            Self::Mariadb => 3306,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseTlsMode {
    Disable,
    Prefer,
    Require,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StorageBackend {
    Json,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DatabaseConfig {
    pub provider: DatabaseProvider,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub tls_mode: DatabaseTlsMode,
    pub max_connections: u32,
    pub active_backend: StorageBackend,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let provider = DatabaseProvider::Postgresql;
        Self {
            provider,
            host: "127.0.0.1".to_string(),
            port: provider.default_port(),
            database: "kisautotrade".to_string(),
            username: "kisautotrade".to_string(),
            password: String::new(),
            tls_mode: DatabaseTlsMode::Prefer,
            max_connections: 5,
            active_backend: StorageBackend::Json,
        }
    }
}

impl DatabaseConfig {
    pub(super) fn validate(&self) -> Result<()> {
        if self.host.trim().is_empty() {
            bail!("DB host를 입력하세요.");
        }
        if self.database.trim().is_empty() {
            bail!("DB 이름을 입력하세요.");
        }
        if self.username.trim().is_empty() {
            bail!("DB 사용자 이름을 입력하세요.");
        }
        if self.port == 0 {
            bail!("DB port는 1~65535 범위여야 합니다.");
        }
        if !(1..=20).contains(&self.max_connections) {
            bail!("DB 최대 연결 수는 1~20 범위여야 합니다.");
        }
        Ok(())
    }

    pub(super) fn fingerprint(&self) -> String {
        format!(
            "{:?}|{}|{}|{}|{}|{}|{:?}|{}",
            self.provider,
            self.host,
            self.port,
            self.database,
            self.username,
            self.password,
            self.tls_mode,
            self.max_connections
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDatabaseConfigInput {
    pub provider: DatabaseProvider,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub tls_mode: DatabaseTlsMode,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseConfigView {
    pub provider: DatabaseProvider,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password_configured: bool,
    pub tls_mode: DatabaseTlsMode,
    pub max_connections: u32,
    pub active_backend: StorageBackend,
    pub configured: bool,
}

impl From<&DatabaseConfig> for DatabaseConfigView {
    fn from(config: &DatabaseConfig) -> Self {
        Self {
            provider: config.provider,
            host: config.host.clone(),
            port: config.port,
            database: config.database.clone(),
            username: config.username.clone(),
            password_configured: !config.password.is_empty(),
            tls_mode: config.tls_mode,
            max_connections: config.max_connections,
            active_backend: config.active_backend,
            configured: !config.host.is_empty()
                && !config.database.is_empty()
                && !config.username.is_empty(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseTableStatus {
    pub name: String,
    pub purpose: String,
    pub exists: bool,
    pub row_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatusView {
    pub connected: bool,
    pub provider: DatabaseProvider,
    pub active_backend: StorageBackend,
    pub schema_version: Option<i32>,
    pub required_schema_version: i32,
    pub server_version: Option<String>,
    pub latency_ms: Option<u64>,
    pub checked_at: String,
    pub message: String,
    pub tables: Vec<DatabaseTableStatus>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonStorageCategoryView {
    pub category: String,
    pub file_count: usize,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonStorageInventoryView {
    pub file_count: usize,
    pub size_bytes: u64,
    pub categories: Vec<JsonStorageCategoryView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseTransferResult {
    pub operation: String,
    pub processed: usize,
    pub inserted_or_updated: usize,
    pub skipped: usize,
    pub size_bytes: u64,
    pub output_path: Option<String>,
    pub checksum: String,
    pub completed_at: String,
    pub message: String,
}
