use super::*;
use crate::trading::order::test_support::{self, Fixture};
use tokio::time::{timeout, Duration};

fn buy() -> Signal {
    Signal::Buy {
        symbol: "005930".into(),
        quantity: 1,
        reason: "synthetic run fixture".into(),
    }
}
async fn submit(
    manager: Arc<Mutex<OrderManager>>,
    context: ExecutionRunContext,
    balance: i64,
) -> Result<SubmissionOutcome> {
    OrderManager::submit_signal_shared(
        &manager,
        context,
        StrategySignal {
            strategy_id: "fixture-strategy".into(),
            signal: buy(),
        },
        "fixture".into(),
        balance,
        None,
        100,
    )
    .await
}

#[tokio::test]
async fn run_context_rejects_same_scope_restart_revision_aba_and_scope_setter() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let first = f.begin(10).await;
    let second = f.begin(10).await;
    assert_ne!(first.epoch, second.epoch);
    let mut manager = f.manager.lock().await;
    assert!(!manager.execution_run_matches(&first));
    assert!(manager.execution_run_matches(&second));
    manager.publish_strategy_revision(11);
    assert!(!manager.execution_run_matches(&second));
    let current = manager.execution_run_context().unwrap();
    manager.publish_strategy_revision(10);
    assert!(!manager.execution_run_matches(&current));
    assert!(manager.execution_run_context().is_none());
    drop(manager);
    let third = f.begin(12).await;
    let mut manager = f.manager.lock().await;
    manager.set_execution_scope(third.broker_scope.clone());
    assert!(!manager.execution_run_matches(&third));
}

#[tokio::test]
async fn begin_run_validates_profile_id_broker_account_credentials_and_mode() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    for mismatch in 0..6 {
        let mut p = f.profile.clone();
        match mismatch {
            0 => p.id.push_str("-other"),
            1 => p.broker_id = BrokerId::Toss,
            2 => p.account_no = "8765432101".into(),
            3 => p.app_key.push_str("-other"),
            4 => p.app_secret.push_str("-other"),
            _ => p.is_paper_trading = !p.is_paper_trading,
        }
        assert!(f
            .manager
            .lock()
            .await
            .begin_execution_run(
                test_support::scope(&f.profile),
                Some(f.profile.id.clone()),
                1,
                Arc::clone(&f.client),
                Some(p)
            )
            .is_err());
    }
    let mut manager = f.manager.lock().await;
    assert!(manager
        .begin_execution_run(
            BrokerScope::new(BrokerId::Toss, None),
            None,
            1,
            Arc::clone(&f.client),
            None
        )
        .is_err());
    assert!(manager
        .begin_execution_run(
            test_support::scope(&test_support::profile("8765432101")),
            None,
            1,
            Arc::clone(&f.client),
            None
        )
        .is_err());
    assert!(manager
        .begin_execution_run(
            test_support::scope(&f.profile),
            None,
            1,
            Arc::clone(&f.client),
            None
        )
        .is_ok());
}

#[tokio::test]
async fn automatic_missing_context_and_stale_reservation_leave_budget_and_pending_unchanged() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let mut submission =
        OrderSubmission::from_signal(None, buy(), "fixture".into(), 100_000, None, 100).unwrap();
    let outcome = OrderManager::submit_order_shared(&f.manager, submission.clone(), None)
        .await
        .unwrap();
    assert!(matches!(outcome, SubmissionOutcome::StaleRun { .. }));
    let context = f.begin(1).await;
    submission.broker_scope = context.broker_scope.clone();
    submission.expected_run = Some(context.clone());
    let mut manager = f.manager.lock().await;
    // Every public context dimension is part of the reservation gate, even on the same account.
    for mismatch in 0..5 {
        let mut changed = context.clone();
        match mismatch {
            0 => changed.epoch += 1,
            1 => changed.configuration_revision += 1,
            2 => changed.profile_id = Some("other-profile".into()),
            3 => changed.broker_scope.broker_id = BrokerId::Toss,
            _ => changed.broker_scope = test_support::scope(&test_support::profile("8765432101")),
        }
        submission.expected_run = Some(changed);
        assert!(matches!(
            manager.reserve_submission(&submission, 0, None),
            ReservationDecision::StaleRun(_)
        ));
        assert!(manager.submitting.is_empty());
        assert!(manager.pending.is_empty());
        assert!(manager.budget.is_none());
    }
    submission.expected_run = Some(context);
    manager.end_execution_run();
    assert!(matches!(
        manager.reserve_submission(&submission, 0, None),
        ReservationDecision::StaleRun(_)
    ));
    assert!(manager.pending.is_empty());
    assert!(manager.submitting.is_empty());
    assert!(manager.budget.is_none());
}

#[tokio::test]
async fn shared_pipeline_discards_prepared_signals_on_stop_restart_account_and_config_changes() {
    for change in 0..5 {
        let f = Fixture::new("http://127.0.0.1:1".into()).await;
        let context = f.begin(40).await;
        let gate = Arc::new(PreparationTestGate {
            arrived: tokio::sync::Barrier::new(2),
            resume: tokio::sync::Barrier::new(2),
        });
        {
            let mut manager = f.manager.lock().await;
            manager
                .update_auto_trading_budget(&context.broker_scope, 2000, 0)
                .await
                .unwrap();
            manager.preparation_test_gate = Some(Arc::clone(&gate));
        }
        // The final case reaches the prepare Skip branch after a stop.
        let task = tokio::spawn(submit(
            Arc::clone(&f.manager),
            context.clone(),
            if change == 4 { 0 } else { 100_000 },
        ));
        timeout(Duration::from_secs(3), gate.arrived.wait())
            .await
            .unwrap();
        let before = {
            let mut manager = f.manager.lock().await;
            let before = serde_json::to_value(&manager.budget).unwrap();
            match change {
                0 | 4 => manager.end_execution_run(),
                1 => {
                    manager.end_execution_run();
                    manager
                        .begin_execution_run(
                            context.broker_scope.clone(),
                            Some(f.profile.id.clone()),
                            40,
                            Arc::clone(&f.client),
                            Some(f.profile.clone()),
                        )
                        .unwrap();
                }
                2 => {
                    manager.set_execution_scope(test_support::scope(&test_support::profile(
                        "8765432101",
                    )));
                }
                _ => {
                    manager.publish_strategy_revision(41);
                    manager.publish_strategy_revision(40);
                }
            }
            before
        };
        gate.resume.wait().await;
        let outcome = timeout(Duration::from_secs(3), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(matches!(outcome, SubmissionOutcome::StaleRun { .. }));
        let manager = f.manager.lock().await;
        assert_eq!(before, serde_json::to_value(&manager.budget).unwrap());
        assert!(manager.submitting.is_empty());
        assert!(manager.pending.is_empty());
    }
}

#[derive(Clone, Default)]
struct LocalProvider {
    reject_insufficient: bool,
    requests: Arc<Mutex<Vec<(axum::http::HeaderMap, serde_json::Value)>>>,
    arrived: Arc<tokio::sync::Notify>,
    resume: Arc<tokio::sync::Notify>,
}

async fn local_order(
    axum::extract::State(provider): axum::extract::State<LocalProvider>,
    headers: axum::http::HeaderMap,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    let index = {
        let mut requests = provider.requests.lock().await;
        let index = requests.len();
        requests.push((headers, body));
        index
    };
    if index == 0 {
        provider.arrived.notify_one();
        provider.resume.notified().await;
    }
    if provider.reject_insufficient {
        return (
            axum::http::StatusCode::OK,
            axum::Json(
                serde_json::json!({"rt_cd":"1", "msg1":"APBK0013 synthetic insufficient balance"}),
            ),
        );
    }
    if index < 2 {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({"msg1":"synthetic EGW00133 retry"})),
        )
    } else {
        (
            axum::http::StatusCode::OK,
            axum::Json(
                serde_json::json!({"rt_cd":"0","msg1":"synthetic accepted","output":{"odno":"fixture-order-id","ord_tmd":"120000"}}),
            ),
        )
    }
}

#[tokio::test]
async fn reserved_pipeline_keeps_original_client_for_every_retry_and_tracks_old_scope_after_restart(
) {
    for overseas in [false, true] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let provider = LocalProvider::default();
        let router = axum::Router::new()
            .route(
                "/uapi/domestic-stock/v1/trading/order-cash",
                axum::routing::post(local_order),
            )
            .route(
                "/uapi/overseas-stock/v1/trading/order",
                axum::routing::post(local_order),
            )
            .with_state(provider.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let f = Fixture::new(base_url).await;
        let context = f.begin(1).await;
        {
            let mut manager = f.manager.lock().await;
            manager
                .update_auto_trading_budget(&context.broker_scope, 2000, 2000)
                .await
                .unwrap();
        }
        let manager_for_task = Arc::clone(&f.manager);
        let expected = context.clone();
        let task = tokio::spawn(async move {
            OrderManager::submit_signal_shared(
                &manager_for_task,
                expected,
                StrategySignal {
                    strategy_id: "fixture-strategy".into(),
                    signal: Signal::Buy {
                        symbol: if overseas { "AAPL" } else { "005930" }.into(),
                        quantity: 1,
                        reason: "synthetic localhost request".into(),
                    },
                },
                "fixture".into(),
                100_000,
                overseas.then(|| "NAS".into()),
                100,
            )
            .await
        });
        timeout(Duration::from_secs(10), provider.arrived.notified())
            .await
            .unwrap();
        let new_profile = test_support::profile("8765432101");
        let new_client = test_support::client("http://127.0.0.1:1".into(), &new_profile).await;
        let new_context = {
            let mut manager = f.manager.lock().await;
            // Arrival at the actual HTTP endpoint proves the production reservation succeeded.
            assert_eq!(manager.submitting.len(), 1);
            let before = serde_json::to_value(&manager.budget).unwrap();
            manager.end_execution_run();
            manager.reset_day();
            assert_eq!(manager.submitting.len(), 1);
            assert!(manager.pending.is_empty());
            assert_eq!(before, serde_json::to_value(&manager.budget).unwrap());
            *manager.rest_client.write().await = Arc::clone(&new_client);
            *manager.profiles.write().await = ProfilesConfig {
                active_id: Some(new_profile.id.clone()),
                profiles: vec![new_profile.clone()],
            };
            let next = manager
                .begin_execution_run(
                    test_support::scope(&new_profile),
                    Some(new_profile.id.clone()),
                    2,
                    Arc::clone(&new_client),
                    Some(new_profile.clone()),
                )
                .unwrap();
            assert!(!manager.execution_run_matches(&context));
            assert!(manager.execution_run_snapshots(&context).is_none());
            assert!(Arc::ptr_eq(
                &manager.execution_run_snapshots(&next).unwrap().0,
                &new_client
            ));
            next
        };
        provider.resume.notify_one();
        let outcome = timeout(Duration::from_secs(20), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            matches!(outcome, SubmissionOutcome::Submitted { provider_order_id } if provider_order_id == "fixture-order-id")
        );
        {
            let requests = provider.requests.lock().await;
            assert_eq!(requests.len(), 3);
            for (headers, body) in requests.iter() {
                assert_eq!(headers["appkey"], f.profile.app_key.as_str());
                assert_eq!(headers["authorization"], "Bearer synthetic-localhost-token");
                assert_eq!(body["CANO"], "12345678");
                assert_eq!(body["ACNT_PRDT_CD"], "01");
                assert_eq!(body["ORD_QTY"], "1");
                assert_eq!(
                    headers["tr_id"],
                    if overseas { "VTTT1002U" } else { "VTTC0802U" }
                );
            }
        }
        {
            let manager = f.manager.lock().await;
            assert!(manager.execution_run_matches(&new_context));
            assert!(manager.submitting.is_empty());
            let pending = manager.pending.get("fixture-order-id").unwrap();
            assert_eq!(pending.broker_scope, context.broker_scope);
            assert_eq!(
                pending.record.broker_account_id.as_deref(),
                Some("1234567801")
            );
            assert!(pending.record.id.starts_with("auto-budget-"));
        }
        let stored = f
            .manager
            .lock()
            .await
            .pending_order_store
            .load()
            .await
            .unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].broker_scope, context.broker_scope);
        {
            let mut manager = f.manager.lock().await;
            let pending_before =
                serde_json::to_value(manager.pending.values().collect::<Vec<_>>()).unwrap();
            let budget_before = serde_json::to_value(&manager.budget).unwrap();
            manager.end_execution_run();
            assert_eq!(
                pending_before,
                serde_json::to_value(manager.pending.values().collect::<Vec<_>>()).unwrap()
            );
            assert_eq!(
                budget_before,
                serde_json::to_value(&manager.budget).unwrap()
            );
        }
        server.abort();
    }
}

#[tokio::test]
async fn delayed_old_account_insufficient_balance_does_not_suspend_new_account() {
    for (change_account, change_mode) in [(true, false), (false, false), (false, true)] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let provider = LocalProvider {
            reject_insufficient: true,
            ..LocalProvider::default()
        };
        let router = axum::Router::new()
            .route(
                "/uapi/domestic-stock/v1/trading/order-cash",
                axum::routing::post(local_order),
            )
            .with_state(provider.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let f = Fixture::new(base_url).await;
        let original = f.begin(1).await;
        f.manager
            .lock()
            .await
            .update_auto_trading_budget(&original.broker_scope, 2000, 0)
            .await
            .unwrap();
        let task = tokio::spawn(submit(Arc::clone(&f.manager), original.clone(), 100_000));
        timeout(Duration::from_secs(10), provider.arrived.notified())
            .await
            .unwrap();
        let mut profile = if change_account {
            test_support::profile("8765432101")
        } else {
            f.profile.clone()
        };
        if change_mode {
            profile.is_paper_trading = !profile.is_paper_trading;
            profile.id.push_str("-live");
        }
        let client = test_support::client("http://127.0.0.1:1".into(), &profile).await;
        let (current, health_before) = {
            let mut manager = f.manager.lock().await;
            assert_eq!(manager.submitting.len(), 1);
            manager.end_execution_run();
            *manager.rest_client.write().await = Arc::clone(&client);
            *manager.profiles.write().await = ProfilesConfig {
                active_id: Some(profile.id.clone()),
                profiles: vec![profile.clone()],
            };
            let current = manager
                .begin_execution_run(
                    test_support::scope(&profile),
                    Some(profile.id.clone()),
                    2,
                    client,
                    Some(profile),
                )
                .unwrap();
            if change_account {
                manager
                    .update_auto_trading_budget(&current.broker_scope, 500, 0)
                    .await
                    .unwrap();
            }
            assert!(!manager.buy_suspended);
            assert!(manager.buy_suspended_reason.is_none());
            (current, serde_json::to_value(&manager.health).unwrap())
        };
        provider.resume.notify_one();
        let outcome = timeout(Duration::from_secs(10), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            matches!(outcome, SubmissionOutcome::Skipped { reason, .. } if reason.contains("APBK0013"))
        );
        {
            let mut manager = f.manager.lock().await;
            assert!(manager.execution_run_matches(&current));
            assert!(manager.submitting.is_empty());
            assert!(manager.pending.is_empty());
            assert_eq!(manager.buy_suspended, !change_account && !change_mode);
            assert_eq!(
                manager.buy_suspended_reason.is_some(),
                !change_account && !change_mode
            );
            assert_eq!(
                health_before,
                serde_json::to_value(&manager.health).unwrap()
            );
            let old_budget = manager
                .auto_trading_budget(&original.broker_scope)
                .await
                .unwrap();
            assert_eq!(old_budget.krw.available_amount, 2000);
            assert_eq!(old_budget.krw.reserved_amount, 0);
            if change_account {
                let current_budget = manager
                    .auto_trading_budget(&current.broker_scope)
                    .await
                    .unwrap();
                assert_eq!(current_budget.krw.available_amount, 500);
                assert_eq!(current_budget.krw.reserved_amount, 0);
            }
            let records = manager
                .order_store
                .get_by_date(chrono::Local::now().date_naive())
                .await
                .unwrap();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].status, OrderStatus::Failed);
            assert_eq!(records[0].broker_account_id.as_deref(), Some("1234567801"));
        }
        assert_eq!(provider.requests.lock().await.len(), 1);
        server.abort();
    }
}

async fn local_holdings(
    axum::extract::State(provider): axum::extract::State<LocalProvider>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> axum::Json<serde_json::Value> {
    provider.requests.lock().await.push((
        headers,
        serde_json::json!({"stage":"holdings", "CANO":query["CANO"]}),
    ));
    provider.arrived.notify_one();
    provider.resume.notified().await;
    axum::Json(
        serde_json::json!({"rt_cd":"0","msg1":"synthetic holdings", "output1":[{"pdno":"005930","prdt_name":"old account fixture","hldg_qty":"9","pchs_avg_pric":"100","prpr":"100"}], "output2":[]}),
    )
}

async fn local_single_order(
    axum::extract::State(provider): axum::extract::State<LocalProvider>,
    headers: axum::http::HeaderMap,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    provider.requests.lock().await.push((headers, body));
    axum::Json(
        serde_json::json!({"rt_cd":"0","msg1":"synthetic accepted","output":{"odno":"manual-fixture-id","ord_tmd":"120000"}}),
    )
}

async fn manual_submit(
    manager: Arc<Mutex<OrderManager>>,
    scope: BrokerScope,
) -> Result<SubmissionOutcome> {
    OrderManager::submit_manual_order_shared(
        &manager,
        "005930".into(),
        "fixture".into(),
        RestOrderSide::Buy,
        OrderType::Limit,
        1,
        100,
        100,
        100_000,
        None,
        scope,
    )
    .await
}

#[tokio::test]
async fn manual_delayed_holdings_cannot_replace_changed_account_or_profile_tracker() {
    for change_account in [true, false] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let provider = LocalProvider::default();
        let router = axum::Router::new()
            .route(
                "/uapi/domestic-stock/v1/trading/inquire-balance",
                axum::routing::get(local_holdings),
            )
            .route(
                "/uapi/domestic-stock/v1/trading/order-cash",
                axum::routing::post(local_single_order),
            )
            .with_state(provider.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let f = Fixture::new(base_url).await;
        let original = test_support::scope(&f.profile);
        f.manager.lock().await.set_execution_scope(original.clone());
        let task = tokio::spawn(manual_submit(Arc::clone(&f.manager), original));
        timeout(Duration::from_secs(10), provider.arrived.notified())
            .await
            .unwrap();
        let mut profile = if change_account {
            test_support::profile("8765432101")
        } else {
            f.profile.clone()
        };
        profile.id.push_str("-new-active");
        let client = test_support::client("http://127.0.0.1:1".into(), &profile).await;
        {
            let mut manager = f.manager.lock().await;
            manager.set_execution_scope(test_support::scope(&profile));
            *manager.rest_client.write().await = client;
            *manager.profiles.write().await = ProfilesConfig {
                active_id: Some(profile.id.clone()),
                profiles: vec![profile],
            };
            manager.position_tracker.lock().await.replace(vec![(
                "000660".into(),
                "new scope holding".into(),
                7,
                120,
                130,
            )]);
        }
        provider.resume.notify_one();
        assert!(timeout(Duration::from_secs(10), task)
            .await
            .unwrap()
            .unwrap()
            .is_err());
        let manager = f.manager.lock().await;
        let positions = manager.position_tracker.lock().await;
        assert!(positions.get("005930").is_none());
        assert_eq!(positions.get("000660").unwrap().quantity, 7);
        assert!(manager.pending.is_empty());
        assert!(manager.submitting.is_empty());
        assert!(manager.budget.is_none());
        assert_eq!(provider.requests.lock().await.len(), 1);
        server.abort();
    }
}

#[tokio::test]
async fn manual_same_account_uses_exact_active_profile_instead_of_first_inactive_mode() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let provider = LocalProvider::default();
    let router = axum::Router::new()
        .route(
            "/uapi/domestic-stock/v1/trading/inquire-balance",
            axum::routing::get(local_holdings),
        )
        .route(
            "/uapi/domestic-stock/v1/trading/order-cash",
            axum::routing::post(local_single_order),
        )
        .with_state(provider.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let f = Fixture::new(base_url).await;
    let scope = test_support::scope(&f.profile);
    let mut inactive = f.profile.clone();
    inactive.id.push_str("-inactive-first");
    inactive.app_key.push_str("-other-key");
    inactive.is_paper_trading = false;
    {
        let mut manager = f.manager.lock().await;
        manager.set_execution_scope(scope.clone());
        *manager.profiles.write().await = ProfilesConfig {
            active_id: Some(f.profile.id.clone()),
            profiles: vec![inactive, f.profile.clone()],
        };
    }
    let task = tokio::spawn(manual_submit(Arc::clone(&f.manager), scope));
    timeout(Duration::from_secs(10), provider.arrived.notified())
        .await
        .unwrap();
    provider.resume.notify_one();
    let outcome = timeout(Duration::from_secs(10), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(
        matches!(outcome, SubmissionOutcome::Submitted { provider_order_id } if provider_order_id == "manual-fixture-id")
    );
    let requests = provider.requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].0["tr_id"], "VTTC8434R");
    assert_eq!(requests[1].0["tr_id"], "VTTC0802U");
    for (headers, _) in requests.iter() {
        assert_eq!(headers["appkey"], f.profile.app_key.as_str());
    }
    server.abort();
}

#[tokio::test]
async fn legacy_configured_account_without_profile_allows_manual_but_fences_client_replacement() {
    for replace_client in [false, true] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let provider = LocalProvider::default();
        let router = axum::Router::new()
            .route(
                "/uapi/domestic-stock/v1/trading/inquire-balance",
                axum::routing::get(local_holdings),
            )
            .route(
                "/uapi/domestic-stock/v1/trading/order-cash",
                axum::routing::post(local_single_order),
            )
            .with_state(provider.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let f = Fixture::new(base_url).await;
        let scope = test_support::scope(&f.profile);
        {
            let mut manager = f.manager.lock().await;
            manager.set_execution_scope(scope.clone());
            *manager.profiles.write().await = ProfilesConfig::default();
        }
        let task = tokio::spawn(manual_submit(Arc::clone(&f.manager), scope));
        timeout(Duration::from_secs(10), provider.arrived.notified())
            .await
            .unwrap();
        if replace_client {
            // Even identical account/credentials on another Arc invalidate an unpublished holdings response.
            let replacement = test_support::client("http://127.0.0.1:1".into(), &f.profile).await;
            let manager = f.manager.lock().await;
            *manager.rest_client.write().await = replacement;
            manager.position_tracker.lock().await.replace(vec![(
                "000660".into(),
                "replacement scope".into(),
                7,
                120,
                130,
            )]);
        }
        provider.resume.notify_one();
        let result = timeout(Duration::from_secs(10), task)
            .await
            .unwrap()
            .unwrap();
        let manager = f.manager.lock().await;
        if replace_client {
            assert!(result.is_err());
            assert!(manager.pending.is_empty());
            let positions = manager.position_tracker.lock().await;
            assert!(positions.get("005930").is_none());
            assert_eq!(positions.get("000660").unwrap().quantity, 7);
            assert_eq!(provider.requests.lock().await.len(), 1);
        } else {
            assert!(matches!(
                result.unwrap(),
                SubmissionOutcome::Submitted { .. }
            ));
            assert_eq!(manager.pending.len(), 1);
            assert_eq!(provider.requests.lock().await.len(), 2);
        }
        server.abort();
    }
}
