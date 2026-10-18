use super::*;
use crate::{
    broker::{BrokerAccountId, BrokerId},
    storage::trade_store::{TradeRecord, TradeSide},
    trading::order::test_support::Fixture,
};

fn trade(scope: &BrokerScope, event_id: &str, pnl: i64) -> TradeRecord {
    let mut trade = TradeRecord::new(
        "005930".into(),
        "synthetic fixture".into(),
        TradeSide::Sell,
        1,
        100,
        0,
        "synthetic-provider-order".into(),
        Some("fixture-strategy".into()),
        "synthetic confirmed fill".into(),
    )
    .with_broker_scope(scope)
    .with_provider_trace(
        Some("kis".into()),
        Some("synthetic-provider-order".into()),
        None,
        Some("TTTC0801U".into()),
    );
    trade.id = format!("trade-{event_id}");
    trade.realized_pnl_krw = Some(pnl);
    trade
}

#[tokio::test]
async fn risk_rebuild_seeds_trade_event_receipts_and_retries_missing_pending_intent_once() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let scope = crate::trading::order::test_support::scope(&fixture.profile);
    let today = chrono::Local::now().date_naive();
    let orders = Arc::new(OrderStore::new(fixture.path.clone()));
    let trades = Arc::new(TradeStore::new(fixture.path.clone()));
    let risk = Arc::new(Mutex::new(RiskManager::default()));
    let store = Arc::new(crate::storage::RiskStore::new(fixture.path.clone()));
    {
        let mut state = risk.lock().await;
        state.max_consecutive_losses_per_strategy_symbol = 2;
        assert!(state
            .record_fill_pnl_once(
                today,
                "durable-fill",
                &scope,
                "fixture-strategy",
                "005930",
                -1_000
            )
            .unwrap());
        assert!(state
            .record_fill_pnl_once(
                today,
                "intent-without-trade",
                &scope,
                "fixture-strategy",
                "005930",
                -2_000
            )
            .unwrap());
        store.save_runtime(&state.runtime_state()).await.unwrap();
    }
    trades
        .append(trade(&scope, "durable-fill", -1_000))
        .await
        .unwrap();
    // Another account and a different broker do not enter this scope's baseline.
    let mut other = scope.clone();
    other.account_id = Some(BrokerAccountId("other-account".into()));
    trades
        .append(trade(&other, "other-account-fill", -50_000))
        .await
        .unwrap();
    other = scope.clone();
    other.broker_id = BrokerId::Toss;
    trades
        .append(trade(&other, "other-broker-fill", -60_000))
        .await
        .unwrap();
    restore_risk_from_today_trades(&orders, &trades, &risk, &store, &scope, false)
        .await
        .unwrap();
    {
        let mut state = risk.lock().await;
        assert_eq!(state.current_loss(), -1_000);
        assert_eq!(state.runtime_state().processed_fill_pnl.len(), 1);
        assert!(!state
            .record_fill_pnl_once(
                today,
                "durable-fill",
                &scope,
                "fixture-strategy",
                "005930",
                -1_000
            )
            .unwrap());
        assert!(state
            .record_fill_pnl_once(
                today,
                "intent-without-trade",
                &scope,
                "fixture-strategy",
                "005930",
                -2_000
            )
            .unwrap());
        assert!(!state
            .record_fill_pnl_once(
                today,
                "intent-without-trade",
                &scope,
                "fixture-strategy",
                "005930",
                -2_000
            )
            .unwrap());
        assert_eq!(state.current_loss(), -3_000);
        assert_eq!(state.blocked_strategy_symbol_count(), 1);
    }
    trades
        .append(trade(&scope, "intent-without-trade", -2_000))
        .await
        .unwrap();
    // A subsequent restart takes the completed trade as its single source.
    restore_risk_from_today_trades(&orders, &trades, &risk, &store, &scope, false)
        .await
        .unwrap();
    let persisted = store.load_runtime().await.unwrap().unwrap();
    assert_eq!(persisted.current_loss, -3_000);
    assert_eq!(persisted.processed_fill_pnl.len(), 2);
    let mut restored = RiskManager::default();
    restored.apply_runtime_state(persisted);
    assert!(!restored
        .record_fill_pnl_once(
            today,
            "intent-without-trade",
            &scope,
            "fixture-strategy",
            "005930",
            -2_000
        )
        .unwrap());
    assert_eq!(restored.current_loss(), -3_000);
}

#[tokio::test]
async fn risk_rebuild_uses_chronological_profits_and_keeps_manual_emergency_stop() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let scope = crate::trading::order::test_support::scope(&fixture.profile);
    let orders = Arc::new(OrderStore::new(fixture.path.clone()));
    let trades = Arc::new(TradeStore::new(fixture.path.clone()));
    let risk = Arc::new(Mutex::new(RiskManager::default()));
    let store = Arc::new(crate::storage::RiskStore::new(fixture.path.clone()));
    {
        let mut state = risk.lock().await;
        state.max_consecutive_losses_per_strategy_symbol = 1;
        state.trigger_emergency_stop();
    }
    let start = chrono::Local::now();
    let mut profit = trade(&scope, "profit", 300);
    profit.timestamp = start.to_rfc3339();
    let mut loss = trade(&scope, "loss", -200);
    loss.timestamp = (start - chrono::Duration::seconds(1)).to_rfc3339();
    // Store insertion order intentionally opposes chronological fill order.
    trades.append(profit).await.unwrap();
    trades.append(loss).await.unwrap();
    for _ in 0..2 {
        restore_risk_from_today_trades(&orders, &trades, &risk, &store, &scope, false)
            .await
            .unwrap();
        let state = risk.lock().await;
        assert_eq!(state.current_loss(), -200);
        assert_eq!(state.daily_profit(), 300);
        assert_eq!(state.blocked_strategy_symbol_count(), 0);
        assert_eq!(state.runtime_state().processed_fill_pnl.len(), 2);
        assert!(state.is_emergency_stop());
    }
}

#[tokio::test]
async fn risk_rebuild_isolates_paper_profit_from_live_losses_for_the_same_account() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let scope = crate::trading::order::test_support::scope(&fixture.profile);
    let orders = Arc::new(OrderStore::new(fixture.path.clone()));
    let trades = Arc::new(TradeStore::new(fixture.path.clone()));
    let risk = Arc::new(Mutex::new(RiskManager::default()));
    let store = Arc::new(crate::storage::RiskStore::new(fixture.path.clone()));
    let mut paper = trade(&scope, "paper-profit", 100_000);
    paper.provider_tr_id = Some("VTTC0801U".into());
    trades.append(paper).await.unwrap();
    trades
        .append(trade(&scope, "live-loss", -50_000))
        .await
        .unwrap();
    for is_paper in [false, true, false] {
        restore_risk_from_today_trades(&orders, &trades, &risk, &store, &scope, is_paper)
            .await
            .unwrap();
        let state = risk.lock().await;
        assert_eq!(state.current_loss(), if is_paper { 0 } else { -50_000 });
        assert_eq!(state.daily_profit(), if is_paper { 100_000 } else { 0 });
        assert_eq!(state.net_loss(), if is_paper { 0 } else { 50_000 });
        assert_eq!(state.runtime_state().processed_fill_pnl.len(), 1);
    }
}

#[tokio::test]
async fn unknown_mode_or_runtime_save_failure_never_publishes_a_partial_rebuild() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let scope = crate::trading::order::test_support::scope(&fixture.profile);
    let orders = Arc::new(OrderStore::new(fixture.path.clone()));
    let trades = Arc::new(TradeStore::new(fixture.path.clone()));
    let risk = Arc::new(Mutex::new(RiskManager::default()));
    let store = Arc::new(crate::storage::RiskStore::new(fixture.path.clone()));
    {
        let mut state = risk.lock().await;
        state.record_pnl(-700);
        state.trigger_emergency_stop();
        store.save_runtime(&state.runtime_state()).await.unwrap();
    }
    let before = serde_json::to_value(risk.lock().await.runtime_state()).unwrap();
    let mut unknown = trade(&scope, "legacy-unknown-mode", -500);
    unknown.provider_tr_id = None;
    trades.append(unknown).await.unwrap();
    let error = restore_risk_from_today_trades(&orders, &trades, &risk, &store, &scope, false)
        .await
        .unwrap_err();
    assert_eq!(error.code, "RISK_MODE_UNCONFIRMED");
    assert_eq!(
        serde_json::to_value(risk.lock().await.runtime_state()).unwrap(),
        before
    );
    assert_eq!(
        serde_json::to_value(store.load_runtime().await.unwrap().unwrap()).unwrap(),
        before
    );

    let empty_trades = Arc::new(TradeStore::new(fixture.path.join("empty-trades")));
    let blocked_path = fixture.path.join("not-a-directory");
    tokio::fs::write(&blocked_path, b"synthetic fixture")
        .await
        .unwrap();
    let blocked_store = Arc::new(crate::storage::RiskStore::new(blocked_path));
    let error = restore_risk_from_today_trades(
        &orders,
        &empty_trades,
        &risk,
        &blocked_store,
        &scope,
        false,
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "RISK_PERSIST_FAILED");
    assert_eq!(
        serde_json::to_value(risk.lock().await.runtime_state()).unwrap(),
        before
    );
}
