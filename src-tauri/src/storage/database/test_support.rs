//! Isolated JSON-only fixtures: no config load, keychain or global installation.
use super::*;

impl DatabaseManager {
    pub(crate) fn isolated_json_for_tests(base: &Path) -> Self {
        Self {
            config_path: base.join("fixture-database.json"),
            data_dir: base.join("data"),
            export_dir: base.join("exports"),
            config: tokio::sync::RwLock::new(DatabaseConfig::default()),
            recovery_required: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pool: Mutex::new(None),
            pool_creation_lock: Mutex::new(()),
            operation_lock: Mutex::new(()),
        }
    }
}
