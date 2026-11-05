use super::*;
use crate::trading::order::{
    test_support::{self, Fixture},
    PendingOrder,
};
use crate::{config::ProfilesConfig, storage::order_store::OrderRecord};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::{
    sync::Notify,
    time::{timeout, Duration},
};

#[derive(Clone)]
struct Provider {
    arrived: Arc<Notify>,
    resume: Arc<Notify>,
    requests: Arc<AtomicUsize>,
    fail: Arc<AtomicBool>,
}

#[tokio::test]
async fn real_fill_invalidates_an_earlier_whole_holdings_response() {
    use crate::trading::order::{HoldingsMarkets, HoldingsUpdate};
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    seed_pending(&fixture).await;
    let scope = test_support::scope(&fixture.profile);
    let mut manager = fixture.manager.lock().await;
    let query = manager
        .begin_holdings_query(scope.broker_id, HoldingsMarkets::Domestic)
        .await
        .unwrap();
    // Actual order-ID accounting invalidates the in-flight snapshot without double-applying account quantity.
    manager.on_fill("fixture-fill", 1, 100).await.unwrap();
    assert!(manager
        .publish_holdings(query, HoldingsUpdate::Domestic(vec![]))
        .await
        .is_err());
    assert!(manager
        .position_tracker
        .lock()
        .await
        .get("005930")
        .is_none());
    assert_eq!(manager.pending["fixture-fill"].filled_quantity, 1);
}

async fn executed(
    axum::extract::State(provider): axum::extract::State<Provider>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> axum::Json<serde_json::Value> {
    assert_eq!(headers["authorization"], "Bearer synthetic-localhost-token");
    assert_eq!(query["CANO"], "12345678");
    if provider.requests.fetch_add(1, Ordering::SeqCst) == 0 {
        provider.arrived.notify_one();
        provider.resume.notified().await;
    }
    axum::Json(if provider.fail.load(Ordering::SeqCst) {
        serde_json::json!({"rt_cd":"1","msg1":"synthetic stale execution error"})
    } else {
        serde_json::json!({"rt_cd":"0","msg1":"synthetic fill","output1":[{
            "pdno":"005930","odno":"fixture-fill","ord_dt":chrono::Local::now().format("%Y%m%d").to_string(),
            "ord_qty":"2","tot_ccld_qty":"2","tot_ccld_amt":"200","rmn_qty":"0"
        }]})
    })
}

async fn provider(fail: bool) -> (String, Provider, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let state = Provider {
        arrived: Arc::new(Notify::new()),
        resume: Arc::new(Notify::new()),
        requests: Arc::new(AtomicUsize::new(0)),
        fail: Arc::new(AtomicBool::new(fail)),
    };
    let router = axum::Router::new()
        .route(
            "/uapi/domestic-stock/v1/trading/inquire-daily-ccld",
            axum::routing::get(executed),
        )
        .with_state(state.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (base, state, server)
}

async fn seed_pending(fixture: &Fixture) {
    fixture.begin(1).await;
    let scope = test_support::scope(&fixture.profile);
    let mut manager = fixture.manager.lock().await;
    manager
        .update_auto_trading_budget(&scope, 2000, 0)
        .await
        .unwrap();
    let id = manager
        .reserve_budget(&scope, "005930", false, true, 2, 100)
        .await
        .unwrap();
    let mut record = OrderRecord::new(
        "005930".into(),
        "fixture".into(),
        OrderSide::Buy,
        2,
        100,
        "Limit".into(),
    )
    .with_broker_scope(&scope);
    record.id = id;
    record.kis_order_id = Some("fixture-fill".into());
    record.provider_tr_id = Some(
        if fixture.client.is_paper() {
            "VTTC0802U"
        } else {
            "TTTC0802U"
        }
        .into(),
    );
    let pending = PendingOrder {
        ownership_key: None,
        ownership_generation: None,
        strategy_position_metadata: None,
        submitted_cost_basis: None,
        record,
        signal_reason: "synthetic fill".into(),
        strategy_id: Some("fixture-strategy".into()),
        signal_price: 100,
        order_price: 100,
        exchange: None,
        broker_scope: scope,
        filled_quantity: 0,
        filled_notional: 0,
        confirmed_filled_quantity: 0,
        confirmed_avg_price: 0,
        confirmed_filled_notional: None,
        application_started: false,
        application_execution_date: None,
        application_exchange_rate: None,
        application_pnl: None,
        client_order_id: None,
        provider_status: None,
    };
    manager.accept_budget(&pending).await.unwrap();
    manager.track_pending_order("fixture-fill".into(), pending);
    manager.persist_pending_orders().await.unwrap();
}

async fn switch_to_other_scope(fixture: &Fixture) {
    let profile = test_support::profile("8765432101");
    let client = test_support::client("http://127.0.0.1:1".into(), &profile).await;
    let mut manager = fixture.manager.lock().await;
    manager.end_execution_run();
    *manager.rest_client.write().await = Arc::clone(&client);
    *manager.profiles.write().await = ProfilesConfig {
        active_id: Some(profile.id.clone()),
        profiles: vec![profile.clone()],
    };
    manager
        .position_tracker
        .lock()
        .await
        .on_buy("005930".into(), "new account".into(), 5, 1000);
    manager.risk_manager.lock().await.record_pnl(123);
    manager
        .begin_execution_run(
            test_support::scope(&profile),
            Some(profile.id.clone()),
            2,
            client,
            Some(profile),
        )
        .unwrap();
    manager.mark_reconciliation(Some("new scope diagnostic".into()));
}

async fn snapshot(fixture: &Fixture) -> serde_json::Value {
    let manager = fixture.manager.lock().await;
    let positions = manager.position_tracker.lock().await;
    let position = positions
        .get("005930")
        .map(|position| (position.quantity, position.avg_price));
    let risk = manager.risk_manager.lock().await.runtime_state();
    serde_json::json!({"pending":manager.pending,"budget":manager.budget,
        "position":position,"risk":risk,"health":manager.health_status()})
}

#[tokio::test]
async fn delayed_reconciliation_cannot_mutate_new_scope_and_applies_when_original_is_reselected() {
    for fail in [false, true] {
        let (base, provider, server) = provider(fail).await;
        let fixture = Fixture::new(base).await;
        seed_pending(&fixture).await;
        let task_manager = Arc::clone(&fixture.manager);
        let task = tokio::spawn(async move {
            OrderManager::confirm_pending_fills_from_broker_shared(&task_manager).await
        });
        timeout(Duration::from_secs(5), provider.arrived.notified())
            .await
            .unwrap();
        switch_to_other_scope(&fixture).await;
        let before = snapshot(&fixture).await;
        provider.resume.notify_one();
        timeout(Duration::from_secs(5), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(snapshot(&fixture).await, before);
        provider.fail.store(false, Ordering::SeqCst);
        {
            let mut manager = fixture.manager.lock().await;
            *manager.rest_client.write().await = Arc::clone(&fixture.client);
            *manager.profiles.write().await = ProfilesConfig {
                active_id: Some(fixture.profile.id.clone()),
                profiles: vec![fixture.profile.clone()],
            };
            manager.position_tracker.lock().await.clear();
            manager
                .begin_execution_run(
                    test_support::scope(&fixture.profile),
                    Some(fixture.profile.id.clone()),
                    3,
                    Arc::clone(&fixture.client),
                    Some(fixture.profile.clone()),
                )
                .unwrap();
        }
        OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
            .await
            .unwrap();
        {
            let mut manager = fixture.manager.lock().await;
            assert!(manager.pending.is_empty());
            assert!(manager
                .position_tracker
                .lock()
                .await
                .get("005930")
                .is_none());
            assert_eq!(
                manager
                    .auto_trading_budget(&test_support::scope(&fixture.profile))
                    .await
                    .unwrap()
                    .krw
                    .reserved_amount,
                0
            );
        }
        let trades_before = fixture
            .manager
            .lock()
            .await
            .trade_store
            .get_by_date(chrono::Local::now().date_naive())
            .await
            .unwrap()
            .len();
        OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
            .await
            .unwrap();
        assert_eq!(
            fixture
                .manager
                .lock()
                .await
                .trade_store
                .get_by_date(chrono::Local::now().date_naive())
                .await
                .unwrap()
                .len(),
            trades_before
        );
        assert_eq!(trades_before, 1);
        server.abort();
    }
}

#[tokio::test]
async fn actual_fill_crosses_same_scope_restart_with_identical_provider_snapshot() {
    let (base, provider, server) = provider(false).await;
    let fixture = Fixture::new(base).await;
    seed_pending(&fixture).await;
    let task_manager = Arc::clone(&fixture.manager);
    let task = tokio::spawn(async move {
        OrderManager::confirm_pending_fills_from_broker_shared(&task_manager).await
    });
    timeout(Duration::from_secs(5), provider.arrived.notified())
        .await
        .unwrap();
    fixture.begin(2).await;
    provider.resume.notify_one();
    timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let manager = fixture.manager.lock().await;
    assert!(manager.pending.is_empty());
    assert!(manager
        .position_tracker
        .lock()
        .await
        .get("005930")
        .is_none());
    server.abort();
}

#[tokio::test]
async fn direct_fill_and_zero_fill_terminal_cannot_mutate_another_execution_scope() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    seed_pending(&fixture).await;
    switch_to_other_scope(&fixture).await;
    let before = snapshot(&fixture).await;
    let mut manager = fixture.manager.lock().await;
    manager.on_fill("fixture-fill", 2, 100).await.unwrap();
    manager
        .apply_pending_fills(vec![PendingFill {
            cumulative_notional: None,
            odno: "fixture-fill".into(),
            filled_qty: 0,
            avg_price: 0,
            terminal_status: Some(OrderStatus::Cancelled),
            provider_status: "CANCELLED".into(),
            execution_date: chrono::Local::now().date_naive(),
        }])
        .await
        .unwrap();
    drop(manager);
    assert_eq!(snapshot(&fixture).await, before);
}

async fn select_mode(fixture: &Fixture, base: String, paper: bool) {
    let mut profile = fixture.profile.clone();
    profile.is_paper_trading = paper;
    let client = test_support::client(base, &profile).await;
    let mut manager = fixture.manager.lock().await;
    manager.end_execution_run();
    *manager.rest_client.write().await = Arc::clone(&client);
    *manager.profiles.write().await = ProfilesConfig {
        active_id: Some(profile.id.clone()),
        profiles: vec![profile.clone()],
    };
    manager
        .begin_execution_run(
            test_support::scope(&profile),
            Some(profile.id.clone()),
            3,
            client,
            Some(profile),
        )
        .unwrap();
}

#[tokio::test]
async fn persisted_kis_pending_never_queries_or_applies_in_the_opposite_mode() {
    for original_paper in [true, false] {
        let (base, provider, server) = provider(false).await;
        // The server only delays its first real query, which occurs after mode restoration.
        provider.resume.notify_one();
        let mut fixture = Fixture::new(base.clone()).await;
        if !original_paper {
            fixture.profile.is_paper_trading = false;
            fixture.client = test_support::client(base.clone(), &fixture.profile).await;
            let manager = fixture.manager.lock().await;
            *manager.rest_client.write().await = Arc::clone(&fixture.client);
            *manager.profiles.write().await = ProfilesConfig {
                active_id: Some(fixture.profile.id.clone()),
                profiles: vec![fixture.profile.clone()],
            };
        }
        seed_pending(&fixture).await;
        select_mode(&fixture, base.clone(), !original_paper).await;
        let before = snapshot(&fixture).await;
        assert!(
            OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
                .await
                .is_err()
        );
        {
            let mut manager = fixture.manager.lock().await;
            assert!(manager.on_fill("fixture-fill", 2, 100).await.is_err());
            assert!(manager
                .apply_pending_fills(vec![PendingFill {
                    cumulative_notional: None,
                    odno: "fixture-fill".into(),
                    filled_qty: 0,
                    avg_price: 0,
                    terminal_status: Some(OrderStatus::Cancelled),
                    provider_status: "CANCELLED".into(),
                    execution_date: chrono::Local::now().date_naive(),
                }])
                .await
                .is_err());
            assert!(manager
                .health_status()
                .last_reconciliation_error
                .unwrap()
                .contains("투자 모드"));
        }
        let after = snapshot(&fixture).await;
        for key in ["pending", "budget", "position", "risk"] {
            assert_eq!(before[key], after[key]);
        }
        assert_eq!(provider.requests.load(Ordering::SeqCst), 0);
        select_mode(&fixture, base, original_paper).await;
        OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
            .await
            .unwrap();
        assert_eq!(provider.requests.load(Ordering::SeqCst), 1);
        let manager = fixture.manager.lock().await;
        assert!(manager.pending.is_empty());
        assert!(manager
            .position_tracker
            .lock()
            .await
            .get("005930")
            .is_none());
        server.abort();
    }
}

#[tokio::test]
async fn absent_unknown_and_similarly_prefixed_kis_tr_ids_preserve_pending_without_querying() {
    for tr_id in [
        None,
        Some("unknown"),
        Some("VTTC0802U-extra"),
        Some("TTTC8001R"),
    ] {
        let (base, provider, server) = provider(false).await;
        let fixture = Fixture::new(base).await;
        seed_pending(&fixture).await;
        fixture
            .manager
            .lock()
            .await
            .pending
            .get_mut("fixture-fill")
            .unwrap()
            .record
            .provider_tr_id = tr_id.map(str::to_owned);
        let before = snapshot(&fixture).await;
        assert!(timeout(
            Duration::from_secs(3),
            OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
        )
        .await
        .expect("unknown TR-ID must not await a provider response")
        .is_err());
        {
            let mut manager = fixture.manager.lock().await;
            assert!(manager.on_fill("fixture-fill", 2, 100).await.is_err());
            assert!(manager
                .health_status()
                .last_reconciliation_error
                .unwrap()
                .contains("TR-ID"));
        }
        let after = snapshot(&fixture).await;
        for key in ["pending", "budget", "position", "risk"] {
            assert_eq!(before[key], after[key]);
        }
        assert_eq!(provider.requests.load(Ordering::SeqCst), 0);
        server.abort();
    }
}

#[tokio::test]
async fn exact_eight_order_tr_ids_select_mode_and_toss_bypasses_kis_mode_classification() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    seed_pending(&fixture).await;
    let mut manager = fixture.manager.lock().await;
    let pending = manager.pending.get_mut("fixture-fill").unwrap();
    for (tr_id, paper) in [
        ("VTTC0802U", true),
        ("VTTC0801U", true),
        ("VTTT1002U", true),
        ("VTTT1006U", true),
        ("TTTC0802U", false),
        ("TTTC0801U", false),
        ("TTTT1002U", false),
        ("TTTT1006U", false),
    ] {
        pending.record.provider_tr_id = Some(tr_id.into());
        assert!(validate_pending_kis_mode(pending, paper).is_ok());
        assert!(validate_pending_kis_mode(pending, !paper).is_err());
    }
    pending.broker_scope.broker_id = BrokerId::Toss;
    pending.record.broker_id = Some(BrokerId::Toss);
    pending.record.provider = Some("toss".into());
    pending.record.provider_tr_id = None;
    assert!(validate_pending_kis_mode(pending, true).is_ok());
    assert!(validate_pending_kis_mode(pending, false).is_ok());
}
