use super::*;
use crate::{
    storage::order_store::OrderRecord,
    trading::order::{
        ownership::{OwnershipIdentity, OwnershipKey},
        test_support::{self, Fixture},
    },
};

#[tokio::test]
async fn ownership_fill_same_provider_id_across_paper_live_modes_has_distinct_receipts() {
    let (fixture, paper_key) = fixture(OrderSide::Buy, 2, 0, 0).await;
    let mut manager = fixture.manager.lock().await;
    let mut live_pending = manager.pending["fixture-owned-fill"].clone();
    manager.on_fill("fixture-owned-fill", 2, 100).await.unwrap();
    let mut live_profile = fixture.profile.clone();
    live_profile.is_paper_trading = false;
    live_profile.live_trading_consent = true;
    let live_client = test_support::client("http://127.0.0.1:1".into(), &live_profile).await;
    manager.end_execution_run();
    *manager.rest_client.write().await = Arc::clone(&live_client);
    *manager.profiles.write().await = crate::config::ProfilesConfig {
        active_id: Some(live_profile.id.clone()),
        profiles: vec![live_profile.clone()],
    };
    manager
        .begin_execution_run(
            test_support::scope(&live_profile),
            Some(live_profile.id.clone()),
            2,
            live_client,
            Some(live_profile),
        )
        .unwrap();
    let mut live_key = paper_key.clone();
    live_key.identity.is_paper = false;
    live_pending.ownership_key = Some(live_key.clone());
    live_pending.record.id = "fixture-owned-fill-live".into();
    live_pending.record.provider_tr_id = Some("TTTC0802U".into());
    manager.track_pending_order("fixture-owned-fill".into(), live_pending);
    manager.persist_pending_orders().await.unwrap();
    manager.on_fill("fixture-owned-fill", 2, 100).await.unwrap();
    let ledger = manager.load_ownership_ledger().await.unwrap();
    assert_eq!(ledger.position(&paper_key).quantity, 2);
    assert_eq!(ledger.position(&live_key).quantity, 2);
    let trades = manager
        .trade_store
        .get_by_date(chrono::Local::now().date_naive())
        .await
        .unwrap();
    assert_eq!(trades.len(), 2);
    assert_ne!(trades[0].id, trades[1].id);
    assert_eq!(
        manager
            .stats_store
            .get_by_date(chrono::Local::now().date_naive())
            .await
            .unwrap()
            .processed_fill_ids
            .len(),
        2
    );
}

#[tokio::test]
async fn ownership_fill_retry_keeps_saved_execution_date_across_midnight() {
    let (fixture, _key) = fixture(OrderSide::Sell, 2, 2, 200).await;
    let today = chrono::Local::now().date_naive();
    let yesterday = today.pred_opt().unwrap();
    let blocked = fixture.path.join("risk/runtime.json");
    tokio::fs::create_dir_all(&blocked).await.unwrap();
    let mut manager = fixture.manager.lock().await;
    assert!(manager
        .on_fill_on("fixture-owned-fill", 2, 120, yesterday)
        .await
        .is_err());
    assert_eq!(
        manager.pending["fixture-owned-fill"].application_execution_date,
        Some(yesterday.to_string())
    );
    tokio::fs::remove_dir(&blocked).await.unwrap();
    manager
        .on_fill_on("fixture-owned-fill", 2, 120, today)
        .await
        .unwrap();
    assert_eq!(
        manager
            .trade_store
            .get_by_date(yesterday)
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(manager
        .trade_store
        .get_by_date(today)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        manager
            .risk_manager
            .lock()
            .await
            .runtime_state()
            .daily_profit,
        0
    );
}

#[tokio::test]
async fn ownership_fill_profile_switch_same_account_records_only_original_owner_identity() {
    let (fixture, key) = fixture(OrderSide::Buy, 2, 0, 0).await;
    let mut manager = fixture.manager.lock().await;
    let mut switched = fixture.profile.clone();
    switched.id = "another-profile-same-account".into();
    *manager.profiles.write().await = crate::config::ProfilesConfig {
        active_id: Some(switched.id.clone()),
        profiles: vec![switched],
    };
    manager.on_fill("fixture-owned-fill", 2, 100).await.unwrap();
    let ledger = manager.load_ownership_ledger().await.unwrap();
    assert_eq!(ledger.position(&key).quantity, 2);
    assert_eq!(ledger.positions().count(), 1);
    assert_eq!(key.identity.profile_id, Some(fixture.profile.id.clone()));
}

#[tokio::test]
async fn ownership_fill_restart_loads_partial_cost_and_risk_receipts_from_primary_documents() {
    let (fixture, key) = fixture(OrderSide::Sell, 2, 2, 200).await;
    let mut manager = fixture.manager.lock().await;
    manager.on_fill("fixture-owned-fill", 1, 120).await.unwrap();
    let restored = manager.pending_order_store.load().await.unwrap();
    let risk = Arc::new(Mutex::new(crate::trading::risk::RiskManager::default()));
    risk.lock()
        .await
        .apply_runtime_state(manager.risk_store.load_runtime().await.unwrap().unwrap());
    let mut restarted = OrderManager::new(
        Arc::clone(&manager.rest_client),
        Arc::clone(&manager.profiles),
        Arc::clone(&manager.order_store),
        Arc::clone(&manager.pending_order_store),
        Arc::clone(&manager.trade_store),
        Arc::new(Mutex::new(crate::trading::position::PositionTracker::new())),
        Arc::new(Mutex::new(
            crate::trading::position::OverseasPositionTracker::new(),
        )),
        Arc::clone(&manager.stats_store),
        Arc::clone(&manager.exchange_rate_krw),
        risk,
        Arc::clone(&manager.risk_store),
        None,
    );
    restarted.restore_pending_orders(restored);
    restarted
        .begin_execution_run(
            test_support::scope(&fixture.profile),
            Some(fixture.profile.id.clone()),
            2,
            Arc::clone(&fixture.client),
            Some(fixture.profile.clone()),
        )
        .unwrap();
    *manager = restarted;
    manager.on_fill("fixture-owned-fill", 2, 115).await.unwrap();
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
            .risk_manager
            .lock()
            .await
            .runtime_state()
            .daily_profit,
        30
    );
    assert!(manager
        .position_tracker
        .lock()
        .await
        .get("005930")
        .is_none());
    assert_eq!(
        manager
            .trade_store
            .get_by_date(chrono::Local::now().date_naive())
            .await
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn ownership_fill_overseas_empty_snapshot_uses_cents_cost_before_fx_conversion() {
    let (fixture, mut key) = fixture(OrderSide::Sell, 2, 0, 0).await;
    key.market = BrokerMarket::Us;
    key.symbol = "SOXQ".into();
    let mut manager = fixture.manager.lock().await;
    let mut ledger = manager.load_ownership_ledger().await.unwrap();
    ledger
        .apply_cumulative(&OwnershipFill {
            order_record_id: "overseas-seed".into(),
            key: key.clone(),
            side: OrderSide::Buy,
            ordered_quantity: 2,
            cumulative_quantity: Some(2),
            cumulative_notional: Some(20_000),
        })
        .unwrap();
    manager.save_ownership_ledger(&ledger).await.unwrap();
    let pending = manager.pending.get_mut("fixture-owned-fill").unwrap();
    pending.ownership_key = Some(key.clone());
    pending.record.symbol = "SOXQ".into();
    pending.record.provider_tr_id = Some("VTTT1006U".into());
    pending.exchange = Some("NASD".into());
    let blocked = fixture.path.join("risk/runtime.json");
    tokio::fs::create_dir_all(&blocked).await.unwrap();
    assert!(manager
        .on_fill("fixture-owned-fill", 2, 11_000)
        .await
        .is_err());
    assert_eq!(
        manager.pending["fixture-owned-fill"].application_exchange_rate,
        Some(1400.0)
    );
    *manager.exchange_rate_krw.write().await = 1500.0;
    tokio::fs::remove_dir(&blocked).await.unwrap();
    manager
        .on_fill("fixture-owned-fill", 2, 11_000)
        .await
        .unwrap();
    assert_eq!(
        manager
            .load_ownership_ledger()
            .await
            .unwrap()
            .position(&key)
            .quantity,
        0
    );
    assert!(manager
        .overseas_position_tracker
        .lock()
        .await
        .get("SOXQ")
        .is_none());
    let trades = manager
        .trade_store
        .get_by_date(chrono::Local::now().date_naive())
        .await
        .unwrap();
    assert_eq!(trades[0].realized_pnl_krw, Some(28_000));
    assert_eq!(
        manager
            .risk_manager
            .lock()
            .await
            .runtime_state()
            .daily_profit,
        28_000
    );
}

async fn fixture(
    side: OrderSide,
    quantity: u64,
    seed_quantity: u64,
    seed_cost: u128,
) -> (Fixture, OwnershipKey) {
    fixture_with_base(
        side,
        quantity,
        seed_quantity,
        seed_cost,
        "http://127.0.0.1:1".into(),
    )
    .await
}

async fn fixture_with_base(
    side: OrderSide,
    quantity: u64,
    seed_quantity: u64,
    seed_cost: u128,
    base: String,
) -> (Fixture, OwnershipKey) {
    let fixture = Fixture::new(base).await;
    fixture.begin(1).await;
    let scope = test_support::scope(&fixture.profile);
    let key = OwnershipKey {
        identity: OwnershipIdentity {
            scope: scope.clone(),
            profile_id: Some(fixture.profile.id.clone()),
            is_paper: true,
        },
        owner: OwnershipOwner::Strategy("fixture-owner".into()),
        market: BrokerMarket::Kr,
        symbol: "005930".into(),
    };
    let mut manager = fixture.manager.lock().await;
    let mut ledger = manager.load_ownership_ledger().await.unwrap();
    if seed_quantity > 0 {
        ledger
            .apply_cumulative(&OwnershipFill {
                order_record_id: "fixture-seed".into(),
                key: key.clone(),
                side: OrderSide::Buy,
                ordered_quantity: seed_quantity,
                cumulative_quantity: Some(seed_quantity),
                cumulative_notional: Some(seed_cost),
            })
            .unwrap();
    }
    manager.save_ownership_ledger(&ledger).await.unwrap();
    let mut record = OrderRecord::new(
        "005930".into(),
        "fixture".into(),
        side,
        quantity,
        100,
        "Limit".into(),
    )
    .with_broker_scope(&scope);
    record.id = "fixture-owned-fill".into();
    record.kis_order_id = Some("fixture-owned-fill".into());
    record.provider_tr_id = Some(
        if matches!(side, OrderSide::Buy) {
            "VTTC0802U"
        } else {
            "VTTC0801U"
        }
        .into(),
    );
    let pending = PendingOrder {
        record,
        broker_scope: scope,
        ownership_key: Some(key.clone()),
        ownership_generation: Some(ledger.generation().to_string()),
        strategy_position_metadata: None,
        submitted_cost_basis: None,
        strategy_id: Some("fixture-owner".into()),
        signal_reason: "fixture".into(),
        signal_price: 100,
        order_price: 100,
        exchange: None,
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
    manager.track_pending_order("fixture-owned-fill".into(), pending);
    manager.persist_pending_orders().await.unwrap();
    drop(manager);
    (fixture, key)
}

#[tokio::test]
async fn ownership_fill_whole_snapshot_already_includes_buy_without_double_accounting() {
    let (fixture, key) = fixture(OrderSide::Buy, 2, 0, 0).await;
    fixture.seed_holding("005930", 2, 100).await;
    let mut manager = fixture.manager.lock().await;
    manager.on_fill("fixture-owned-fill", 2, 100).await.unwrap();
    assert_eq!(
        manager
            .position_tracker
            .lock()
            .await
            .get("005930")
            .unwrap()
            .quantity,
        2
    );
    let ledger = manager.load_ownership_ledger().await.unwrap();
    assert_eq!(ledger.position(&key).quantity, 2);
    assert_eq!(ledger.position(&key).cost_notional, 200);
    manager.on_fill("fixture-owned-fill", 2, 100).await.unwrap();
    assert_eq!(
        manager
            .load_ownership_ledger()
            .await
            .unwrap()
            .position(&key)
            .quantity,
        2
    );
    assert_eq!(
        manager
            .trade_store
            .get_by_date(chrono::Local::now().date_naive())
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn ownership_fill_empty_sell_snapshot_uses_exact_owner_cost_across_partial_and_repeat() {
    let (fixture, key) = fixture(OrderSide::Sell, 2, 2, 200).await;
    let mut manager = fixture.manager.lock().await;
    assert!(manager
        .position_tracker
        .lock()
        .await
        .get("005930")
        .is_none());
    manager.on_fill("fixture-owned-fill", 1, 120).await.unwrap();
    let ledger = manager.load_ownership_ledger().await.unwrap();
    assert_eq!(ledger.position(&key).quantity, 1);
    assert_eq!(ledger.position(&key).cost_notional, 100);
    manager.on_fill("fixture-owned-fill", 1, 120).await.unwrap();
    manager.on_fill("fixture-owned-fill", 2, 115).await.unwrap();
    assert_eq!(
        manager
            .load_ownership_ledger()
            .await
            .unwrap()
            .position(&key)
            .quantity,
        0
    );
    assert!(manager
        .position_tracker
        .lock()
        .await
        .get("005930")
        .is_none());
    let trades = manager
        .trade_store
        .get_by_date(chrono::Local::now().date_naive())
        .await
        .unwrap();
    assert_eq!(trades.len(), 2);
    assert_eq!(
        trades
            .iter()
            .filter_map(|trade| trade.realized_pnl_krw)
            .sum::<i64>(),
        30
    );
    assert_eq!(
        manager
            .risk_manager
            .lock()
            .await
            .runtime_state()
            .daily_profit,
        30
    );
}

#[tokio::test]
async fn ownership_fill_unknown_cost_and_generation_never_fabricate_sell_profit() {
    for failure in ["unknown-owner-cost", "wrong-generation", "legacy-empty"] {
        let (fixture, _key) = fixture(OrderSide::Sell, 2, 0, 0).await;
        let mut manager = fixture.manager.lock().await;
        let pending = manager.pending.get_mut("fixture-owned-fill").unwrap();
        match failure {
            "wrong-generation" => {
                pending.ownership_generation = Some(uuid::Uuid::new_v4().to_string())
            }
            "legacy-empty" => {
                pending.ownership_key = None;
                pending.ownership_generation = None;
            }
            _ => {}
        }
        assert!(
            manager.on_fill("fixture-owned-fill", 2, 120).await.is_err(),
            "{failure}"
        );
        assert_eq!(manager.pending["fixture-owned-fill"].filled_quantity, 0);
        assert!(manager.persistence_blocked);
        assert!(manager
            .trade_store
            .get_by_date(chrono::Local::now().date_naive())
            .await
            .unwrap()
            .is_empty());
    }
}

#[tokio::test]
async fn ownership_fill_manual_saved_cost_survives_empty_account_snapshot() {
    let (fixture, _key) = fixture(OrderSide::Sell, 2, 0, 0).await;
    let mut manager = fixture.manager.lock().await;
    let pending = manager.pending.get_mut("fixture-owned-fill").unwrap();
    pending.ownership_key = None;
    pending.ownership_generation = None;
    pending.strategy_id = None;
    pending.submitted_cost_basis = Some(100);
    manager.on_fill("fixture-owned-fill", 2, 120).await.unwrap();
    let trades = manager
        .trade_store
        .get_by_date(chrono::Local::now().date_naive())
        .await
        .unwrap();
    assert_eq!(trades[0].realized_pnl_krw, Some(40));
    assert!(manager
        .position_tracker
        .lock()
        .await
        .get("005930")
        .is_none());
}

#[tokio::test]
async fn ownership_fill_risk_save_retry_freezes_intent_and_records_each_delta_once() {
    let (fixture, key) = fixture(OrderSide::Sell, 4, 4, 400).await;
    let blocked = fixture.path.join("risk/runtime.json");
    tokio::fs::create_dir_all(&blocked).await.unwrap();
    let mut manager = fixture.manager.lock().await;
    assert!(manager.on_fill("fixture-owned-fill", 2, 90).await.is_err());
    assert_eq!(manager.pending["fixture-owned-fill"].filled_quantity, 0);
    assert_eq!(
        manager.pending["fixture-owned-fill"].confirmed_filled_quantity,
        2
    );
    assert_eq!(
        manager.pending["fixture-owned-fill"].application_pnl,
        Some(-20)
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
    tokio::fs::remove_dir(&blocked).await.unwrap();
    // Complete exactly the saved two-share intent even if the provider has advanced.
    manager.on_fill("fixture-owned-fill", 4, 95).await.unwrap();
    assert_eq!(manager.pending["fixture-owned-fill"].filled_quantity, 2);
    assert_eq!(
        manager
            .load_ownership_ledger()
            .await
            .unwrap()
            .position(&key)
            .quantity,
        2
    );
    assert_eq!(
        manager
            .risk_manager
            .lock()
            .await
            .runtime_state()
            .current_loss,
        -20
    );
    manager.on_fill("fixture-owned-fill", 4, 95).await.unwrap();
    assert!(manager.pending.is_empty());
    assert_eq!(
        manager
            .load_ownership_ledger()
            .await
            .unwrap()
            .position(&key)
            .quantity,
        0
    );
    let risk = manager.risk_manager.lock().await.runtime_state();
    assert_eq!(risk.current_loss, -20);
    assert_eq!(risk.daily_profit, 0);
    let trades = manager
        .trade_store
        .get_by_date(chrono::Local::now().date_naive())
        .await
        .unwrap();
    assert_eq!(trades.len(), 2);
    assert_eq!(
        trades
            .iter()
            .filter_map(|trade| trade.realized_pnl_krw)
            .sum::<i64>(),
        -20
    );
}

#[tokio::test]
async fn ownership_fill_legacy_applying_intent_requires_durable_date_fx_and_notional() {
    let today = chrono::Local::now().date_naive();
    for failure in ["date-today", "date-yesterday", "overseas-fx", "notional"] {
        let (fixture, _key) = fixture(OrderSide::Sell, 2, 2, 200).await;
        let mut manager = fixture.manager.lock().await;
        let pending = manager.pending.get_mut("fixture-owned-fill").unwrap();
        pending.application_started = true;
        pending.confirmed_filled_quantity = 1;
        pending.confirmed_avg_price = 120;
        pending.confirmed_filled_notional = Some(120);
        pending.application_pnl = Some(20);
        pending.application_execution_date = Some(today.to_string());
        pending.application_exchange_rate = Some(1400.0);
        match failure {
            "overseas-fx" => {
                pending.exchange = Some("NASD".into());
                pending.record.symbol = "SOXQ".into();
                pending.record.provider_tr_id = Some("VTTT1006U".into());
                pending.application_exchange_rate = None;
            }
            "notional" => pending.confirmed_filled_notional = None,
            _ => pending.application_execution_date = None,
        }
        let before = serde_json::to_value(&manager.pending).unwrap();
        let date = if failure == "date-yesterday" {
            today.pred_opt().unwrap()
        } else {
            today
        };
        let error = manager
            .on_fill_confirmed("fixture-owned-fill", 2, 120, Some(240), date)
            .await
            .unwrap_err();
        let expected = match failure {
            "overseas-fx" => "환율",
            "notional" => "누적 금액",
            _ => "실행일",
        };
        assert!(error.to_string().contains(expected), "{failure}: {error}");
        assert_eq!(serde_json::to_value(&manager.pending).unwrap(), before);
        assert!(manager.persistence_blocked);
        assert!(manager
            .trade_store
            .get_by_date(today)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            manager
                .risk_manager
                .lock()
                .await
                .runtime_state()
                .daily_profit,
            0
        );
    }
}

#[tokio::test]
async fn ownership_fill_legacy_sell_does_not_adopt_other_owners_account_average() {
    let (fixture, _key) = fixture(OrderSide::Sell, 2, 0, 0).await;
    fixture.seed_holding("005930", 50, 100).await;
    let mut manager = fixture.manager.lock().await;
    let pending = manager.pending.get_mut("fixture-owned-fill").unwrap();
    pending.ownership_key = None;
    pending.ownership_generation = None;
    let before = serde_json::to_value(&manager.pending).unwrap();
    let error = manager
        .on_fill("fixture-owned-fill", 2, 120)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("제출 시 확정된 매입 원가"));
    assert_eq!(serde_json::to_value(&manager.pending).unwrap(), before);
    assert_eq!(
        manager
            .position_tracker
            .lock()
            .await
            .get("005930")
            .unwrap()
            .quantity,
        50
    );
    assert!(manager
        .trade_store
        .get_by_date(chrono::Local::now().date_naive())
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn ownership_fill_exact_native_notional_preserves_301_total_and_partial_remainder() {
    for overseas in [false, true] {
        let (fixture, mut key) = fixture(OrderSide::Buy, 3, 0, 0).await;
        let mut manager = fixture.manager.lock().await;
        if overseas {
            key.market = BrokerMarket::Us;
            key.symbol = "SOXQ".into();
            let pending = manager.pending.get_mut("fixture-owned-fill").unwrap();
            pending.ownership_key = Some(key.clone());
            pending.exchange = Some("NASD".into());
            pending.record.symbol = "SOXQ".into();
            pending.record.provider_tr_id = Some("VTTT1002U".into());
        }
        let today = chrono::Local::now().date_naive();
        manager
            .on_fill_confirmed("fixture-owned-fill", 2, 100, Some(201), today)
            .await
            .unwrap();
        manager
            .on_fill_confirmed("fixture-owned-fill", 3, 100, Some(301), today)
            .await
            .unwrap();
        let ledger = manager.load_ownership_ledger().await.unwrap();
        assert_eq!(ledger.position(&key).quantity, 3);
        assert_eq!(ledger.position(&key).cost_notional, 301);
        let trades = manager.trade_store.get_by_date(today).await.unwrap();
        assert_eq!(
            trades.iter().map(|trade| trade.total_amount).sum::<u64>(),
            301
        );
        if overseas {
            assert!(
                (trades
                    .iter()
                    .map(|trade| trade.total_amount_usd.unwrap())
                    .sum::<f64>()
                    - 3.01)
                    .abs()
                    < 1e-10
            );
        }
    }
}

#[tokio::test]
async fn ownership_fill_larger_terminal_cumulative_waits_for_earlier_intent_then_applies_every_share(
) {
    let date = chrono::Local::now().date_naive();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let body = serde_json::json!({"rt_cd":"0","msg1":"synthetic terminal", "output1":[{
        "pdno":"005930","odno":"fixture-owned-fill","ord_dt":date.format("%Y%m%d").to_string(),
        "ord_qty":"4","tot_ccld_qty":"3","tot_ccld_amt":"300","cnc_cfrm_qty":"1",
        "rmn_qty":"0","cncl_yn":"Y"
    }]});
    let router = axum::Router::new().route(
        "/uapi/domestic-stock/v1/trading/inquire-daily-ccld",
        axum::routing::get(move || {
            let body = body.clone();
            async move { axum::Json(body) }
        }),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let (fixture, key) = fixture_with_base(OrderSide::Sell, 4, 4, 400, base).await;
    let blocked = fixture.path.join("risk/runtime.json");
    tokio::fs::create_dir_all(&blocked).await.unwrap();
    {
        let mut manager = fixture.manager.lock().await;
        assert!(manager
            .on_fill_confirmed("fixture-owned-fill", 1, 90, Some(90), date)
            .await
            .is_err());
    }
    tokio::fs::remove_dir(&blocked).await.unwrap();
    OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
        .await
        .unwrap();
    {
        let mut manager = fixture.manager.lock().await;
        assert_eq!(manager.pending["fixture-owned-fill"].filled_quantity, 1);
        assert_eq!(
            manager
                .load_ownership_ledger()
                .await
                .unwrap()
                .position(&key)
                .quantity,
            3
        );
    }
    OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
        .await
        .unwrap();
    let mut manager = fixture.manager.lock().await;
    assert!(manager.pending.is_empty());
    let ledger = manager.load_ownership_ledger().await.unwrap();
    assert_eq!(ledger.position(&key).quantity, 1);
    assert_eq!(ledger.position(&key).cost_notional, 100);
    let trades = manager.trade_store.get_by_date(date).await.unwrap();
    assert_eq!(trades.len(), 2);
    assert_eq!(trades.iter().map(|trade| trade.quantity).sum::<u64>(), 3);
    assert_eq!(
        trades
            .iter()
            .filter_map(|trade| trade.realized_pnl_krw)
            .sum::<i64>(),
        0
    );
    assert_eq!(
        trades.iter().map(|trade| trade.total_amount).sum::<u64>(),
        300
    );
    server.abort();
}

#[tokio::test]
async fn ownership_fill_zero_terminal_retries_confirmed_intent_and_waits_for_consistent_quantity() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let date = chrono::Local::now().date_naive();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(AtomicUsize::new(0));
    let router = axum::Router::new().route("/uapi/domestic-stock/v1/trading/inquire-daily-ccld",
        axum::routing::get(move || {
            let qty = if requests.fetch_add(1,Ordering::SeqCst) < 2 {0} else {2};
            async move {axum::Json(serde_json::json!({"rt_cd":"0","msg1":"synthetic zero terminal","output1":[{
                "pdno":"005930","odno":"fixture-owned-fill","ord_dt":date.format("%Y%m%d").to_string(),
                "ord_qty":"4","tot_ccld_qty":qty.to_string(),"tot_ccld_amt":(qty*90).to_string(),
                "cnc_cfrm_qty":(4-qty).to_string(),"rmn_qty":"0","cncl_yn":"Y"
            }]}))}
        }));
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let (fixture, key) = fixture_with_base(OrderSide::Sell, 4, 4, 400, base).await;
    let blocked = fixture.path.join("risk/runtime.json");
    tokio::fs::create_dir_all(&blocked).await.unwrap();
    {
        let mut manager = fixture.manager.lock().await;
        assert!(manager
            .on_fill_confirmed("fixture-owned-fill", 2, 90, Some(180), date)
            .await
            .is_err());
        assert_eq!(manager.pending["fixture-owned-fill"].filled_quantity, 0);
        assert_eq!(
            manager.pending["fixture-owned-fill"].confirmed_filled_quantity,
            2
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
    tokio::fs::remove_dir(&blocked).await.unwrap();
    for _ in 0..2 {
        OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
            .await
            .unwrap();
        let manager = fixture.manager.lock().await;
        assert_eq!(manager.pending["fixture-owned-fill"].filled_quantity, 2);
        assert!(!manager.pending["fixture-owned-fill"].application_started);
        assert_eq!(
            manager.trade_store.get_by_date(date).await.unwrap().len(),
            1
        );
        assert_eq!(
            manager
                .risk_manager
                .lock()
                .await
                .runtime_state()
                .current_loss,
            -20
        );
    }
    OrderManager::confirm_pending_fills_from_broker_shared(&fixture.manager)
        .await
        .unwrap();
    let mut manager = fixture.manager.lock().await;
    assert!(manager.pending.is_empty());
    assert_eq!(
        manager
            .load_ownership_ledger()
            .await
            .unwrap()
            .position(&key)
            .quantity,
        2
    );
    let trades = manager.trade_store.get_by_date(date).await.unwrap();
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, 2);
    assert_eq!(trades[0].realized_pnl_krw, Some(-20));
    server.abort();
}
