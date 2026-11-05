//! Actual exact-owner feedback and durable checkpoint, using synthetic fixture only.
use super::*;

#[tokio::test]
async fn rsi_feedback_pending_fill_partial_and_restart_preserve_exact_owner_age() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    fixture.begin(0).await;
    let id = "rsi_feedback";
    let cfg = StrategyConfig::new(
        id,
        "fixture",
        true,
        vec!["005930".into()],
        2,
        serde_json::json!({"period":2,"oversold":30.0,"overbought":70.0,"max_hold_observations":3}),
    )
    .with_scope(BrokerId::Kis, Some(fixture.profile.account_no.clone()));
    let strategies = manager(&fixture, vec![cfg.clone()]);
    strategies
        .lock()
        .await
        .initialize_historical("005930", &[100, 90, 80]);
    assert_eq!(strategies.lock().await.on_tick("005930", 100, 0).len(), 1);
    let key = key(&fixture, id, BrokerMarket::Kr, "005930");
    let checkpoint = strategies
        .lock()
        .await
        .owned_position_metadata_for_strategy(
            id,
            &scope(&fixture.profile),
            BrokerMarket::Kr,
            "005930",
        )
        .unwrap();
    assert_eq!(checkpoint["candidateEntry"], true);
    let mut ledger = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    let pending = PendingOrder {
        ownership_key: Some(key.clone()),
        ownership_generation: Some(ledger.generation().to_string()),
        strategy_position_metadata: Some(checkpoint.clone()),
        submitted_cost_basis: None,
        record: OrderRecord::new(
            "005930".into(),
            "fixture".into(),
            OrderSide::Buy,
            3,
            80,
            "Limit".into(),
        )
        .with_broker_scope(&scope(&fixture.profile)),
        signal_reason: "fixture".into(),
        strategy_id: Some(id.into()),
        signal_price: 80,
        order_price: 80,
        exchange: None,
        broker_scope: scope(&fixture.profile),
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
    fixture
        .manager
        .lock()
        .await
        .pending
        .insert("fixture-pending".into(), pending);
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    strategies
        .lock()
        .await
        .initialize_historical("005930", &[100, 90, 80]);
    let after_warmup = strategies
        .lock()
        .await
        .owned_position_metadata_for_strategy(
            id,
            &scope(&fixture.profile),
            BrokerMarket::Kr,
            "005930",
        )
        .unwrap();
    assert_eq!(after_warmup, checkpoint);
    // Actual owner publication is driven by the durable fill ledger, never
    // broker account totals, the raw Buy or the equal-zero pending publication.
    buy(&mut ledger, key.clone(), "buy-rsi", 3, 80);
    fixture
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    fixture
        .manager
        .lock()
        .await
        .pending
        .remove("fixture-pending");
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    let current = || scope(&fixture.profile);
    let metadata = strategies
        .lock()
        .await
        .owned_position_metadata_for_strategy(id, &current(), BrokerMarket::Kr, "005930")
        .unwrap();
    assert_eq!(metadata["ageKnown"], true);
    assert_eq!(metadata["heldObservations"], 0);
    assert_eq!(metadata["candidateEntry"], false);
    assert!(strategies.lock().await.on_tick("005930", 80, 0).is_empty());
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    let metadata = strategies
        .lock()
        .await
        .owned_position_metadata_for_strategy(id, &current(), BrokerMarket::Kr, "005930")
        .unwrap();
    assert_eq!(metadata["heldObservations"], 1);
    let mut ledger = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    ledger
        .apply_cumulative(&OwnershipFill {
            order_record_id: "partial-exit".into(),
            key: key.clone(),
            side: OrderSide::Sell,
            ordered_quantity: 2,
            cumulative_quantity: Some(1),
            cumulative_notional: Some(80),
        })
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
    assert!(strategies.lock().await.on_tick("005930", 80, 0).is_empty());
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    let cold = manager(&fixture, vec![cfg.clone()]);
    cold.lock()
        .await
        .initialize_historical("005930", &[80; 100]);
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &cold)
        .await
        .unwrap();
    let metadata = cold
        .lock()
        .await
        .owned_position_metadata_for_strategy(id, &current(), BrokerMarket::Kr, "005930")
        .unwrap();
    assert_eq!(metadata["heldObservations"], 2);
    let mut other = current();
    other.account_id.as_mut().unwrap().0 = "other-account".into();
    assert!(!cold
        .lock()
        .await
        .restore_owned_position_metadata_for_strategy(
            id,
            &other,
            BrokerMarket::Kr,
            "005930",
            &checkpoint
        )
        .unwrap());
    assert!(cold
        .lock()
        .await
        .owned_position_metadata_for_strategy(id, &current(), BrokerMarket::Us, "005930")
        .is_none());
    let signal = &cold.lock().await.on_tick("005930", 80, 0)[0].signal;
    assert!(matches!(signal,Signal::Sell{quantity:2,reason,..}if reason.contains("관측")));
    // A corrupt exact-owner checkpoint blocks restoration instead of resetting
    // the age or silently continuing with a fresh holding period.
    let mut ledger = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    let mut corrupt = metadata;
    corrupt["ageKnown"] = serde_json::Value::Null;
    ledger.set_position_metadata(&key, corrupt).unwrap();
    fixture
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    let cold = manager(&fixture, vec![cfg]);
    assert!(
        OrderManager::sync_strategy_ownership_shared(&fixture.manager, &cold)
            .await
            .is_err()
    );
    assert!(
        fixture
            .manager
            .lock()
            .await
            .health_status()
            .persistence_blocked
    );
}
