use super::*;
use crate::{
    broker::BrokerMarket,
    storage::{order_store::OrderSide, PendingOrderStore},
    trading::order::{
        ownership::{OwnershipFill, OwnershipIdentity, OwnershipKey, OwnershipOwner},
        test_support::{scope, Fixture},
    },
};
use std::sync::Arc;

fn fill(fixture: &Fixture) -> OwnershipFill {
    OwnershipFill {
        order_record_id: "synthetic-confirmed-order".into(),
        key: OwnershipKey {
            identity: OwnershipIdentity {
                scope: scope(&fixture.profile),
                profile_id: Some(fixture.profile.id.clone()),
                is_paper: true,
            },
            owner: OwnershipOwner::Strategy("synthetic-strategy".into()),
            market: BrokerMarket::Kr,
            symbol: "005930".into(),
        },
        side: OrderSide::Buy,
        ordered_quantity: 2,
        cumulative_quantity: Some(2),
        cumulative_notional: Some(230),
    }
}

#[tokio::test]
async fn missing_primary_starts_empty_and_real_file_restart_keeps_exact_idempotent_fill() {
    let first = Fixture::new("http://127.0.0.1:1".into()).await;
    let fill = fill(&first);
    let mut ledger = {
        let mut manager = first.manager.lock().await;
        let ledger = manager.load_ownership_ledger().await.unwrap();
        assert_eq!(ledger.positions().count(), 0);
        assert!(!manager.health_status().persistence_blocked);
        ledger
    };
    ledger.apply_cumulative(&fill).unwrap();
    first
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    let second = Fixture::new("http://127.0.0.1:1".into()).await;
    {
        let mut manager = second.manager.lock().await;
        manager.pending_order_store = Arc::new(PendingOrderStore::new(first.path.clone()));
        let mut restored = manager.load_ownership_ledger().await.unwrap();
        assert_eq!(restored.position(&fill.key).quantity, 2);
        assert_eq!(restored.position(&fill.key).cost_notional, 230);
        assert_eq!(restored.apply_cumulative(&fill).unwrap().delta_quantity, 0);
        manager.save_ownership_ledger(&restored).await.unwrap();
        assert_eq!(
            manager
                .load_ownership_ledger()
                .await
                .unwrap()
                .position(&fill.key)
                .quantity,
            2
        );
        assert!(!manager.health_status().persistence_blocked);
    }
}

#[tokio::test]
async fn primary_corruption_blocks_instead_of_recovering_an_old_valid_backup() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let mut fill = fill(&fixture);
    fill.cumulative_quantity = Some(1);
    fill.cumulative_notional = Some(100);
    let mut ledger = OwnershipLedger::default();
    ledger.apply_cumulative(&fill).unwrap();
    let mut manager = fixture.manager.lock().await;
    manager.save_ownership_ledger(&ledger).await.unwrap();
    fill.cumulative_quantity = Some(2);
    fill.cumulative_notional = Some(230);
    ledger.apply_cumulative(&fill).unwrap();
    manager.save_ownership_ledger(&ledger).await.unwrap();
    let path = manager.pending_order_store.ownership_path();
    assert!(path.with_file_name("position_ownership.json.bak").is_file());
    tokio::fs::write(&path, b"{ invalid synthetic ledger")
        .await
        .unwrap();
    assert!(manager.load_ownership_ledger().await.is_err());
    assert!(manager.health_status().persistence_blocked);
    assert_eq!(
        tokio::fs::read_to_string(&path).await.unwrap(),
        "{ invalid synthetic ledger"
    );
}

#[tokio::test]
async fn every_read_observes_current_primary_and_structural_errors_are_not_empty_ledgers() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let mut fill = fill(&fixture);
    fill.ordered_quantity = 3;
    let mut manager = fixture.manager.lock().await;
    let path = manager.pending_order_store.ownership_path();
    let mut ledger = manager.load_ownership_ledger().await.unwrap();
    ledger.apply_cumulative(&fill).unwrap();
    manager.save_ownership_ledger(&ledger).await.unwrap();
    // Another current publication in the same document generation must be read
    // fresh; replacing its generation would instead indicate lost ownership.
    let mut updated = ledger.clone();
    fill.cumulative_quantity = Some(3);
    fill.cumulative_notional = Some(360);
    updated.apply_cumulative(&fill).unwrap();
    crate::storage::write_json(&path, &updated).await.unwrap();
    let current = manager.load_ownership_ledger().await.unwrap();
    assert_eq!(current.generation(), ledger.generation());
    assert_eq!(current.position(&fill.key).quantity, 3);
    assert_eq!(current.position(&fill.key).cost_notional, 360);
    tokio::fs::write(&path, b"{}").await.unwrap();
    assert!(manager.load_ownership_ledger().await.is_err());
    assert!(manager.health_status().persistence_blocked);
}

#[tokio::test]
async fn inaccessible_parent_and_directory_primary_fail_closed_for_save_and_load() {
    let save_fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    tokio::fs::create_dir_all(&save_fixture.path).await.unwrap();
    tokio::fs::write(
        save_fixture.path.join("orders"),
        b"synthetic parent blocker",
    )
    .await
    .unwrap();
    let mut manager = save_fixture.manager.lock().await;
    assert!(manager
        .save_ownership_ledger(&OwnershipLedger::default())
        .await
        .is_err());
    assert!(manager.health_status().persistence_blocked);
    drop(manager);
    let load_fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let mut manager = load_fixture.manager.lock().await;
    tokio::fs::create_dir_all(manager.pending_order_store.ownership_path())
        .await
        .unwrap();
    assert!(manager.load_ownership_ledger().await.is_err());
    assert!(manager.health_status().persistence_blocked);
}

#[tokio::test]
async fn fresh_install_creates_both_primary_and_generation_manifest_before_returning() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let mut manager = fixture.manager.lock().await;
    let primary = manager.pending_order_store.ownership_path();
    let manifest_path = manager.pending_order_store.ownership_manifest_path();
    assert!(!primary.exists());
    assert!(!manifest_path.exists());
    let ledger = manager.load_ownership_ledger().await.unwrap();
    assert!(primary.is_file());
    assert!(manifest_path.is_file());
    let manifest: OwnershipManifest =
        serde_json::from_str(&tokio::fs::read_to_string(manifest_path).await.unwrap()).unwrap();
    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.generation, ledger.generation().to_string());
    let persisted: OwnershipLedger =
        serde_json::from_str(&tokio::fs::read_to_string(primary).await.unwrap()).unwrap();
    assert_eq!(persisted.generation(), ledger.generation());
    assert!(!manager.health_status().persistence_blocked);
}

#[tokio::test]
async fn primary_loss_is_detected_after_restart_even_with_no_pending_orders_and_a_valid_backup() {
    let first = Fixture::new("http://127.0.0.1:1".into()).await;
    let fill = fill(&first);
    let primary = {
        let mut manager = first.manager.lock().await;
        let mut ledger = manager.load_ownership_ledger().await.unwrap();
        ledger.apply_cumulative(&fill).unwrap();
        manager.save_ownership_ledger(&ledger).await.unwrap();
        manager.pending_order_store.replace(&[]).await.unwrap();
        assert!(manager.pending_order_store.load().await.unwrap().is_empty());
        let primary = manager.pending_order_store.ownership_path();
        assert!(primary
            .with_file_name("position_ownership.json.bak")
            .is_file());
        primary
    };
    tokio::fs::remove_file(&primary).await.unwrap();
    let restarted = Fixture::new("http://127.0.0.1:1".into()).await;
    let mut manager = restarted.manager.lock().await;
    manager.pending_order_store = Arc::new(PendingOrderStore::new(first.path.clone()));
    assert!(manager.pending_order_store.load().await.unwrap().is_empty());
    assert!(manager.load_ownership_ledger().await.is_err());
    assert!(manager.health_status().persistence_blocked);
    assert!(!primary.exists());
}

#[tokio::test]
async fn another_valid_primary_generation_is_rejected_after_restart() {
    let first = Fixture::new("http://127.0.0.1:1".into()).await;
    let initial = first
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    let replacement = OwnershipLedger::default();
    assert_ne!(replacement.generation(), initial.generation());
    let path = first
        .manager
        .lock()
        .await
        .pending_order_store
        .ownership_path();
    crate::storage::write_json(&path, &replacement)
        .await
        .unwrap();
    let restarted = Fixture::new("http://127.0.0.1:1".into()).await;
    let mut manager = restarted.manager.lock().await;
    manager.pending_order_store = Arc::new(PendingOrderStore::new(first.path.clone()));
    assert!(manager.load_ownership_ledger().await.is_err());
    assert!(manager.health_status().persistence_blocked);
}

#[tokio::test]
async fn primary_only_creation_crash_gap_anchors_the_existing_generation_and_ownership() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let fill = fill(&fixture);
    let mut ledger = OwnershipLedger::default();
    ledger.apply_cumulative(&fill).unwrap();
    let mut manager = fixture.manager.lock().await;
    let manifest_path = manager.pending_order_store.ownership_manifest_path();
    crate::storage::write_json(&manager.pending_order_store.ownership_path(), &ledger)
        .await
        .unwrap();
    assert!(!manifest_path.exists());
    let restored = manager.load_ownership_ledger().await.unwrap();
    assert_eq!(restored.generation(), ledger.generation());
    assert_eq!(restored.position(&fill.key).quantity, 2);
    let manifest: OwnershipManifest =
        serde_json::from_str(&tokio::fs::read_to_string(manifest_path).await.unwrap()).unwrap();
    assert_eq!(manifest.generation, ledger.generation().to_string());
    assert!(!manager.health_status().persistence_blocked);
}

#[tokio::test]
async fn corrupt_invalid_and_mismatched_manifests_never_fallback_to_a_valid_backup() {
    for corruption in [
        "json",
        "version",
        "invalid-uuid",
        "nil",
        "different-generation",
    ] {
        let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
        let mut manager = fixture.manager.lock().await;
        let ledger = manager.load_ownership_ledger().await.unwrap();
        let path = manager.pending_order_store.ownership_manifest_path();
        let valid = OwnershipManifest {
            version: 1,
            generation: ledger.generation().to_string(),
        };
        crate::storage::write_json(&path, &valid).await.unwrap();
        assert!(path
            .with_file_name("position_ownership_manifest.json.bak")
            .is_file());
        let invalid = match corruption {
            "json" => "{ damaged synthetic manifest".to_string(),
            "version" => serde_json::json!({"version":2,"generation":valid.generation}).to_string(),
            "invalid-uuid" => serde_json::json!({"version":1,"generation":"invalid"}).to_string(),
            "nil" => serde_json::json!({"version":1,"generation":uuid::Uuid::nil().to_string()})
                .to_string(),
            _ => serde_json::json!({"version":1,"generation":uuid::Uuid::new_v4().to_string()})
                .to_string(),
        };
        tokio::fs::write(&path, &invalid).await.unwrap();
        assert!(
            manager.load_ownership_ledger().await.is_err(),
            "{corruption}"
        );
        assert!(manager.health_status().persistence_blocked);
        assert_eq!(tokio::fs::read_to_string(path).await.unwrap(), invalid);
    }
}
