use super::*;
use crate::broker::BrokerId;
use crate::trading::order::{
    ownership::{OwnershipFill, OwnershipLedger},
    test_support::{client, scope, Fixture},
};
use crate::trading::strategy::{build_strategy, Signal, StrategyConfig};

fn pc_config(id: &str, account: &str) -> StrategyConfig {
    StrategyConfig::new(
        id,
        "synthetic PC",
        true,
        vec![],
        2,
        serde_json::json!({"symbols":[{
            "symbol":"005930","symbol_name":"fixture","quantity":2,
            "buy_trigger_price":100.0,"take_profit_pct":5.0,"stop_loss_pct":3.0
        }]}),
    )
    .with_scope(BrokerId::Kis, Some(account.into()))
}

fn lth_config(account: &str) -> StrategyConfig {
    StrategyConfig::new(
        "leveraged_trend_hold_fixture",
        "synthetic LTH",
        true,
        vec![],
        2,
        serde_json::json!({"entries":[{
            "leveraged_symbol":"SOXQ","leveraged_symbol_name":"fixture",
            "base_symbols":[],"quantity":2,"is_overseas":true
        }]}),
    )
    .with_scope(BrokerId::Kis, Some(account.into()))
}

fn manager(fixture: &Fixture, configs: Vec<StrategyConfig>) -> Arc<Mutex<StrategyManager>> {
    let mut manager = StrategyManager::new();
    for config in configs {
        manager.add(build_strategy(config));
    }
    manager
        .set_history_execution_identity(scope(&fixture.profile), Some(fixture.profile.id.clone()));
    Arc::new(Mutex::new(manager))
}

fn key(fixture: &Fixture, strategy: &str, market: BrokerMarket, symbol: &str) -> OwnershipKey {
    OwnershipKey {
        identity: OwnershipIdentity {
            scope: scope(&fixture.profile),
            profile_id: Some(fixture.profile.id.clone()),
            is_paper: true,
        },
        owner: OwnershipOwner::Strategy(strategy.into()),
        market,
        symbol: symbol.into(),
    }
}

fn buy(ledger: &mut OwnershipLedger, key: OwnershipKey, id: &str, quantity: u64, price: u64) {
    ledger
        .apply_cumulative(&OwnershipFill {
            order_record_id: id.into(),
            key,
            side: OrderSide::Buy,
            ordered_quantity: quantity,
            cumulative_quantity: Some(quantity),
            cumulative_notional: Some(u128::from(quantity) * u128::from(price)),
        })
        .unwrap();
}

fn protection(high: u64, held: usize) -> serde_json::Value {
    serde_json::json!({"version":1,"kind":"leveragedTrendHold","entryPrice":100,
        "highWater":high,"heldObservations":held})
}

#[tokio::test]
async fn feedback_assigns_only_exact_owner_amid_manual_and_other_account_holdings() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    fixture.begin(0).await;
    let a = "price_condition_a";
    let b = "price_condition_b";
    let strategies = manager(
        &fixture,
        vec![
            pc_config(a, &fixture.profile.account_no),
            pc_config(b, &fixture.profile.account_no),
        ],
    );
    let mut ledger = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    let a_key = key(&fixture, a, BrokerMarket::Kr, "005930");
    buy(&mut ledger, a_key.clone(), "owner-a", 2, 100);
    let mut manual = a_key.clone();
    manual.owner = OwnershipOwner::Manual;
    buy(&mut ledger, manual, "manual", 7, 200);
    let mut other_account = a_key.clone();
    other_account.identity.scope.account_id.as_mut().unwrap().0 = "other-account".into();
    buy(&mut ledger, other_account, "other-account", 9, 300);
    fixture
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    fixture.seed_holding("005930", 50, 400).await;
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    let signals = strategies.lock().await.on_tick("005930", 90, 0);
    assert_eq!(signals.len(), 2);
    assert!(signals.iter().any(|signal| signal.strategy_id == a
        && matches!(signal.signal, Signal::Sell { quantity: 2, .. })));
    assert!(signals.iter().any(|signal| signal.strategy_id == b
        && matches!(signal.signal, Signal::Buy { quantity: 2, .. })));
    assert_eq!(
        fixture
            .manager
            .lock()
            .await
            .position_tracker
            .lock()
            .await
            .get("005930")
            .unwrap()
            .quantity,
        50
    );
}

#[tokio::test]
async fn feedback_cold_lth_restores_metadata_and_partial_change_preserves_live_high() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    fixture.begin(0).await;
    let strategies = manager(&fixture, vec![lth_config(&fixture.profile.account_no)]);
    let key = key(
        &fixture,
        "leveraged_trend_hold_fixture",
        BrokerMarket::Us,
        "SOXQ",
    );
    let mut ledger = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    buy(&mut ledger, key.clone(), "buy-a", 2, 110);
    ledger
        .set_position_metadata(&key, protection(150, 73))
        .unwrap();
    fixture
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    let metadata = strategies
        .lock()
        .await
        .owned_position_metadata_for_strategy(
            "leveraged_trend_hold_fixture",
            &scope(&fixture.profile),
            BrokerMarket::Us,
            "SOXQ",
        )
        .unwrap();
    assert_eq!(metadata["entryPrice"], 110);
    assert_eq!(metadata["highWater"], 150);
    assert_eq!(metadata["heldObservations"], 73);
    // A current runtime's newer trailing observation wins over the old disk checkpoint.
    strategies
        .lock()
        .await
        .restore_owned_position_metadata_for_strategy(
            "leveraged_trend_hold_fixture",
            &scope(&fixture.profile),
            BrokerMarket::Us,
            "SOXQ",
            &protection(180, 81),
        )
        .unwrap();
    buy(&mut ledger, key.clone(), "buy-b", 1, 140);
    fixture
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    let metadata = strategies
        .lock()
        .await
        .owned_position_metadata_for_strategy(
            "leveraged_trend_hold_fixture",
            &scope(&fixture.profile),
            BrokerMarket::Us,
            "SOXQ",
        )
        .unwrap();
    assert_eq!(metadata["entryPrice"], 120);
    assert_eq!(metadata["highWater"], 180);
    assert_eq!(metadata["heldObservations"], 81);
    let disk = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    assert_eq!(disk.position(&key).quantity, 3);
    assert_eq!(disk.position_metadata(&key), Some(&metadata));
}

#[tokio::test]
async fn staged_profile_identity_error_is_retryable_without_persistence_latch() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    fixture.begin(0).await;
    let profile_b = crate::trading::order::test_support::profile("8765432101");
    let strategies = manager(
        &fixture,
        vec![pc_config("price_condition_b", &profile_b.account_no)],
    );
    {
        let orders = fixture.manager.lock().await;
        let mut profiles = orders.profiles.write().await;
        profiles.profiles.push(profile_b.clone());
        profiles.active_id = Some(profile_b.id.clone());
    }
    assert!(
        OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
            .await
            .is_err()
    );
    assert!(
        !fixture
            .manager
            .lock()
            .await
            .health_status()
            .persistence_blocked
    );
    let client_b = client("http://127.0.0.1:1".into(), &profile_b).await;
    {
        let mut orders = fixture.manager.lock().await;
        *orders.rest_client.write().await = Arc::clone(&client_b);
        orders
            .begin_execution_run(
                scope(&profile_b),
                Some(profile_b.id.clone()),
                0,
                client_b,
                Some(profile_b.clone()),
            )
            .unwrap();
    }
    strategies
        .lock()
        .await
        .set_history_execution_identity(scope(&profile_b), Some(profile_b.id.clone()));
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    assert!(
        !fixture
            .manager
            .lock()
            .await
            .health_status()
            .persistence_blocked
    );
    assert!(matches!(
        strategies.lock().await.on_tick("005930", 90, 0)[0].signal,
        Signal::Buy { .. }
    ));
}

#[tokio::test]
async fn feedback_never_transfers_same_account_quantity_across_profile_or_mode() {
    for different_mode in [false, true] {
        let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
        fixture.begin(0).await;
        let strategy_id = "price_condition_a";
        let mut ledger = fixture
            .manager
            .lock()
            .await
            .load_ownership_ledger()
            .await
            .unwrap();
        buy(
            &mut ledger,
            key(&fixture, strategy_id, BrokerMarket::Kr, "005930"),
            "old-owner",
            2,
            100,
        );
        fixture
            .manager
            .lock()
            .await
            .save_ownership_ledger(&ledger)
            .await
            .unwrap();
        let mut next = fixture.profile.clone();
        if different_mode {
            next.is_paper_trading = false;
            next.live_trading_consent = true;
        } else {
            next.id = "other-profile-same-account".into();
        }
        let next_client = client("http://127.0.0.1:1".into(), &next).await;
        {
            let mut orders = fixture.manager.lock().await;
            *orders.profiles.write().await = ProfilesConfig {
                active_id: Some(next.id.clone()),
                profiles: vec![next.clone()],
            };
            *orders.rest_client.write().await = Arc::clone(&next_client);
            orders
                .begin_execution_run(
                    scope(&next),
                    Some(next.id.clone()),
                    0,
                    next_client,
                    Some(next.clone()),
                )
                .unwrap();
        }
        let strategies = manager(&fixture, vec![pc_config(strategy_id, &next.account_no)]);
        strategies
            .lock()
            .await
            .set_history_execution_identity(scope(&next), Some(next.id.clone()));
        OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
            .await
            .unwrap();
        assert!(
            matches!(
                strategies.lock().await.on_tick("005930", 90, 0)[0].signal,
                Signal::Buy { .. }
            ),
            "mode change: {different_mode}"
        );
    }
}

#[tokio::test]
async fn checkpoint_rejects_old_run_and_configuration_generation_without_writing() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let expected = fixture.begin(0).await;
    let strategies = manager(&fixture, vec![lth_config(&fixture.profile.account_no)]);
    let key = key(
        &fixture,
        "leveraged_trend_hold_fixture",
        BrokerMarket::Us,
        "SOXQ",
    );
    let mut ledger = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    buy(&mut ledger, key.clone(), "confirmed", 2, 100);
    ledger
        .set_position_metadata(&key, protection(150, 73))
        .unwrap();
    fixture
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    strategies
        .lock()
        .await
        .restore_owned_position_metadata_for_strategy(
            "leveraged_trend_hold_fixture",
            &scope(&fixture.profile),
            BrokerMarket::Us,
            "SOXQ",
            &protection(180, 81),
        )
        .unwrap();
    let path = fixture
        .manager
        .lock()
        .await
        .pending_order_store
        .ownership_path();
    let before = tokio::fs::read(&path).await.unwrap();
    let mut wrong = expected.clone();
    wrong.epoch += 1;
    assert!(!OrderManager::checkpoint_strategy_positions_shared(
        &fixture.manager,
        &strategies,
        &wrong
    )
    .await
    .unwrap());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), before);
    strategies.lock().await.advance_history_generation();
    assert!(!OrderManager::checkpoint_strategy_positions_shared(
        &fixture.manager,
        &strategies,
        &expected
    )
    .await
    .unwrap());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), before);
    let current = fixture
        .begin(strategies.lock().await.history_generation())
        .await;
    assert!(OrderManager::checkpoint_strategy_positions_shared(
        &fixture.manager,
        &strategies,
        &current
    )
    .await
    .unwrap());
    let disk = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    assert_eq!(disk.position_metadata(&key).unwrap()["highWater"], 180);
}

#[tokio::test]
async fn stale_holdings_refresh_error_never_marks_the_new_account_unhealthy() {
    use axum::{routing::get, Json, Router};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Notify;
    let arrived = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let (a, r, count) = (
        Arc::clone(&arrived),
        Arc::clone(&resume),
        Arc::clone(&calls),
    );
    let router = Router::new().fallback(get(move || {
        let (a, r, count) = (Arc::clone(&a), Arc::clone(&r), Arc::clone(&count));
        async move {
            if count.fetch_add(1, Ordering::SeqCst) == 0 {
                a.notify_one();
                r.notified().await;
            }
            // Both domestic array and overseas object summary are optional.
            Json(serde_json::json!({"rt_cd":"0","msg1":"synthetic fixture", "output1":[],"output2":null}))
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let fixture = Fixture::new(server_url).await;
    fixture.begin(0).await;
    fixture.manager.lock().await.holdings_refresh_required = 3;
    let strategies = manager(
        &fixture,
        vec![pc_config("price_condition_a", &fixture.profile.account_no)],
    );
    let (orders, active_strategies) = (Arc::clone(&fixture.manager), Arc::clone(&strategies));
    let refresh = tokio::spawn(async move {
        OrderManager::reconcile_strategy_state_shared(&orders, &active_strategies).await
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), arrived.notified())
        .await
        .unwrap();
    let profile_b = crate::trading::order::test_support::profile("8765432101");
    let client_b = client("http://127.0.0.1:1".into(), &profile_b).await;
    let before = {
        let mut orders = fixture.manager.lock().await;
        *orders.profiles.write().await = ProfilesConfig {
            active_id: Some(profile_b.id.clone()),
            profiles: vec![profile_b.clone()],
        };
        *orders.rest_client.write().await = Arc::clone(&client_b);
        orders
            .begin_execution_run(
                scope(&profile_b),
                Some(profile_b.id.clone()),
                0,
                client_b,
                Some(profile_b),
            )
            .unwrap();
        orders.mark_holdings_sync(None);
        orders.health_status()
    };
    resume.notify_one();
    let error = tokio::time::timeout(std::time::Duration::from_secs(5), refresh)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(error.to_string().contains("계좌 또는 실행 상태"), "{error}");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let after = fixture.manager.lock().await.health_status();
    assert_eq!(
        after.holdings_consecutive_failures,
        before.holdings_consecutive_failures
    );
    assert_eq!(
        after.last_holdings_sync_error,
        before.last_holdings_sync_error
    );
    assert_eq!(
        after.last_holdings_sync_attempt_at,
        before.last_holdings_sync_attempt_at
    );
    assert_eq!(after.last_holdings_sync_at, before.last_holdings_sync_at);
    assert!(!after.persistence_blocked);
    server.abort();
}
