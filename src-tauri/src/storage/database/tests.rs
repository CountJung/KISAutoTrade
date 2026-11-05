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
    let _keychain = super::super::database_keychain::keychain_test_lock().await;
    let (config_path, data_dir) = keychain_test_dirs();
    let manager = DatabaseManager::load_sync(config_path.clone(), data_dir.clone()).expect("load");
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
    let _keychain = super::super::database_keychain::keychain_test_lock().await;
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

#[cfg(test)]
#[path = "checksum_dependency_contract_tests.rs"]
mod dependency_contract_tests;
