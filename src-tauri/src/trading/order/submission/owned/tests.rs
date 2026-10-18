//! Actual localhost submission -> fill -> broker snapshot ownership boundaries.
use super::*;
use crate::trading::order::{
    holdings::{HoldingsMarkets, HoldingsUpdate},
    ownership::OwnershipFill,
    test_support::Fixture,
};
use tokio::time::{timeout, Duration};

type Requests = Arc<Mutex<Vec<serde_json::Value>>>;

async fn accept(
    axum::extract::State(requests): axum::extract::State<Requests>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let mut requests = requests.lock().await;
    requests.push(body);
    axum::Json(serde_json::json!({"rt_cd":"0","msg1":"synthetic accepted",
        "output":{"odno":format!("owned-order-{}", requests.len()),"ord_tmd":"120000"}}))
}

async fn publish(fixture: &Fixture, quantity: u64) {
    let mut manager = fixture.manager.lock().await;
    let query = manager
        .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Domestic)
        .await
        .unwrap();
    manager
        .publish_holdings(
            query,
            HoldingsUpdate::Domestic(if quantity > 0 {
                vec![(
                    "005930".into(),
                    "fixture".into(),
                    quantity,
                    100_000,
                    120_000,
                )]
            } else {
                vec![]
            }),
        )
        .await
        .unwrap();
}

async fn submit(
    f: &Fixture,
    run: &ExecutionRunContext,
    owner: &str,
    side: OrderSide,
    quantity: u64,
) -> SubmissionOutcome {
    let signal = match side {
        OrderSide::Buy => Signal::Buy {
            symbol: "005930".into(),
            quantity,
            reason: "synthetic owner buy".into(),
        },
        OrderSide::Sell => Signal::Sell {
            symbol: "005930".into(),
            quantity,
            reason: "synthetic owner sell".into(),
        },
    };
    OrderManager::submit_signal_shared(
        &f.manager,
        run.clone(),
        StrategySignal {
            strategy_id: owner.into(),
            signal,
        },
        "fixture".into(),
        100_000_000,
        None,
        120_000,
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn actual_submissions_sell_only_the_confirmed_owner_and_never_add_fill_to_account_snapshot() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let requests: Requests = Arc::default();
    let router = axum::Router::new()
        .route(
            "/uapi/domestic-stock/v1/trading/order-cash",
            axum::routing::post(accept),
        )
        .with_state(requests.clone());
    let f = Fixture::new(format!("http://{}", listener.local_addr().unwrap())).await;
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let run = f.begin(1).await;
    f.manager
        .lock()
        .await
        .update_auto_trading_budget(&run.broker_scope, 2_000_000, 0)
        .await
        .unwrap();
    assert!(matches!(
        submit(&f, &run, "owner-a", OrderSide::Buy, 2).await,
        SubmissionOutcome::Submitted { .. }
    ));
    let key = f.manager.lock().await.pending["owned-order-1"]
        .ownership_key
        .clone()
        .unwrap();
    assert!(key.owner == OwnershipOwner::Strategy("owner-a".into()));
    // A broker read already contains both this fill and ten unowned shares.
    publish(&f, 12).await;
    {
        let mut manager = f.manager.lock().await;
        manager.on_fill("owned-order-1", 2, 100_000).await.unwrap();
        assert_eq!(
            manager
                .position_tracker
                .lock()
                .await
                .get("005930")
                .unwrap()
                .quantity,
            12
        );
        assert_eq!(
            manager
                .load_ownership_ledger()
                .await
                .unwrap()
                .position(&key)
                .quantity,
            2
        );
    }
    // A confirmed fill requires another authoritative read before reservations.
    assert!(matches!(
        submit(&f, &run, "owner-a", OrderSide::Sell, 20).await,
        SubmissionOutcome::Skipped { .. }
    ));
    publish(&f, 12).await;
    assert!(matches!(
        submit(&f, &run, "owner-b", OrderSide::Sell, 20).await,
        SubmissionOutcome::Skipped {
            held_quantity: 0,
            ..
        }
    ));
    assert_eq!(requests.lock().await.len(), 1);
    assert!(matches!(
        submit(&f, &run, "owner-a", OrderSide::Sell, 20).await,
        SubmissionOutcome::Submitted { .. }
    ));
    assert_eq!(requests.lock().await[1]["ORD_QTY"], "2");
    {
        let mut manager = f.manager.lock().await;
        let pending = &manager.pending["owned-order-2"];
        assert_eq!(pending.record.quantity, 2);
        assert_eq!(pending.ownership_key.as_ref(), Some(&key));
        manager.on_fill("owned-order-2", 2, 120_000).await.unwrap();
        assert_eq!(
            manager
                .load_ownership_ledger()
                .await
                .unwrap()
                .position(&key)
                .quantity,
            0
        );
        assert_eq!(
            manager
                .position_tracker
                .lock()
                .await
                .get("005930")
                .unwrap()
                .quantity,
            12
        );
    }
    publish(&f, 10).await;
    assert_eq!(
        f.manager
            .lock()
            .await
            .position_tracker
            .lock()
            .await
            .get("005930")
            .unwrap()
            .quantity,
        10
    );
    server.abort();
}

#[tokio::test]
async fn ownership_recheck_after_preparation_prevents_a_stale_sell_reservation() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let run = f.begin(1).await;
    publish(&f, 12).await;
    let key = OwnershipKey {
        identity: OwnershipIdentity::capture(&run.broker_scope, Some(&f.profile), &f.client)
            .unwrap(),
        owner: OwnershipOwner::Strategy("owner-a".into()),
        market: BrokerMarket::Kr,
        symbol: "005930".into(),
    };
    let gate = Arc::new(PreparationTestGate {
        arrived: tokio::sync::Barrier::new(2),
        resume: tokio::sync::Barrier::new(2),
    });
    {
        let mut manager = f.manager.lock().await;
        let mut ledger = manager.load_ownership_ledger().await.unwrap();
        ledger
            .apply_cumulative(&OwnershipFill {
                order_record_id: "seed".into(),
                key: key.clone(),
                side: OrderSide::Buy,
                ordered_quantity: 2,
                cumulative_quantity: Some(2),
                cumulative_notional: Some(200_000),
            })
            .unwrap();
        manager.save_ownership_ledger(&ledger).await.unwrap();
        manager.preparation_test_gate = Some(gate.clone());
    }
    let task = async {
        timeout(Duration::from_secs(3), gate.arrived.wait())
            .await
            .unwrap();
        let mut manager = f.manager.lock().await;
        let mut ledger = manager.load_ownership_ledger().await.unwrap();
        ledger
            .apply_cumulative(&OwnershipFill {
                order_record_id: "concurrent-sale".into(),
                key: key.clone(),
                side: OrderSide::Sell,
                ordered_quantity: 1,
                cumulative_quantity: Some(1),
                cumulative_notional: Some(120_000),
            })
            .unwrap();
        manager.save_ownership_ledger(&ledger).await.unwrap();
        drop(manager);
        gate.resume.wait().await;
    };
    let (outcome, ()) = tokio::join!(submit(&f, &run, "owner-a", OrderSide::Sell, 20), task);
    assert!(matches!(
        outcome,
        SubmissionOutcome::Skipped {
            held_quantity: 1,
            ..
        }
    ));
    let manager = f.manager.lock().await;
    assert!(manager.submitting.is_empty());
    assert!(manager.pending.is_empty());
}

#[tokio::test]
async fn actual_manual_sell_refresh_excludes_strategy_owned_shares() {
    async fn balance() -> axum::Json<serde_json::Value> {
        axum::Json(
            serde_json::json!({"rt_cd":"0","msg_cd":"0","msg1":"synthetic holdings",
            "output1":[{"pdno":"005930","prdt_name":"fixture","hldg_qty":"12","pchs_avg_pric":"100000","prpr":"120000"}],
            "output2":[]}),
        )
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let requests: Requests = Arc::default();
    let router = axum::Router::new()
        .route(
            "/uapi/domestic-stock/v1/trading/order-cash",
            axum::routing::post(accept),
        )
        .route(
            "/uapi/domestic-stock/v1/trading/inquire-balance",
            axum::routing::get(balance),
        )
        .with_state(requests.clone());
    let f = Fixture::new(format!("http://{}", listener.local_addr().unwrap())).await;
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let run = f.begin(1).await;
    let key = OwnershipKey {
        identity: OwnershipIdentity::capture(&run.broker_scope, Some(&f.profile), &f.client)
            .unwrap(),
        owner: OwnershipOwner::Strategy("owner-a".into()),
        market: BrokerMarket::Kr,
        symbol: "005930".into(),
    };
    {
        let mut manager = f.manager.lock().await;
        let mut ledger = manager.load_ownership_ledger().await.unwrap();
        ledger
            .apply_cumulative(&OwnershipFill {
                order_record_id: "synthetic-seed".into(),
                key: key.clone(),
                side: OrderSide::Buy,
                ordered_quantity: 2,
                cumulative_quantity: Some(2),
                cumulative_notional: Some(200_000),
            })
            .unwrap();
        manager.save_ownership_ledger(&ledger).await.unwrap();
    }
    let outcome = OrderManager::submit_manual_order_shared(
        &f.manager,
        "005930".into(),
        "fixture".into(),
        RestOrderSide::Sell,
        OrderType::Limit,
        20,
        120_000,
        120_000,
        100_000_000,
        None,
        run.broker_scope,
    )
    .await
    .unwrap();
    assert!(matches!(outcome, SubmissionOutcome::Submitted { .. }));
    assert_eq!(requests.lock().await[0]["ORD_QTY"], "10");
    let mut manager = f.manager.lock().await;
    assert!(manager.pending["owned-order-1"].ownership_key.is_none());
    assert_eq!(
        manager.pending["owned-order-1"].submitted_cost_basis,
        Some(100_000)
    );
    assert_eq!(
        manager
            .load_ownership_ledger()
            .await
            .unwrap()
            .position(&key)
            .quantity,
        2
    );
    server.abort();
}
