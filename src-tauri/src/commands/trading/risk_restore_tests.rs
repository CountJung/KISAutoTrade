use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    broker::{BrokerAccountId, BrokerId, BrokerScope},
    storage::{
        order_store::{OrderRecord, OrderSide, OrderStatus, OrderStore},
        trade_store::{TradeRecord, TradeSide, TradeStore},
        RiskStore,
    },
    trading::risk::RiskManager,
};

#[tokio::test]
async fn restart_rebuilds_order_counts_and_consecutive_losses_from_ledgers() {
    let data_dir = std::env::temp_dir().join(format!("kis-risk-restart-{}", uuid::Uuid::new_v4()));
    let order_store = Arc::new(OrderStore::new(data_dir.clone()));
    let trade_store = Arc::new(TradeStore::new(data_dir.clone()));
    let risk_store = Arc::new(RiskStore::new(data_dir.clone()));
    let risk_manager = Arc::new(Mutex::new(RiskManager::new(-1_000_000, 0.5)));
    let scope = BrokerScope::new(BrokerId::Kis, Some(BrokerAccountId("12345678-01".into())));

    let mut order = OrderRecord::new(
        "005930".into(),
        "삼성전자".into(),
        OrderSide::Sell,
        3,
        70_000,
        "Market".into(),
    )
    .with_strategy_id(Some("restart-strategy".into()))
    .with_provider_trace("kis", Some("ORDER-1".into()), None, None)
    .with_broker_scope(&scope);
    order.status = OrderStatus::Filled;
    order_store.append(order).await.expect("order ledger");

    for index in 0..3 {
        let mut trade = TradeRecord::new(
            "005930".into(),
            "삼성전자".into(),
            TradeSide::Sell,
            1,
            69_000,
            0,
            format!("ORDER-1-{index}"),
            Some("restart-strategy".into()),
            "loss".into(),
        )
        .with_broker_scope(&scope);
        trade.realized_pnl_krw = Some(-10_000);
        trade_store.append(trade).await.expect("trade ledger");
    }

    super::restore_risk_from_today_trades(
        &order_store,
        &trade_store,
        &risk_manager,
        &risk_store,
        &scope,
    )
    .await
    .expect("restore");

    let state = risk_manager.lock().await.runtime_state();
    assert_eq!(
        state
            .daily_order_counts
            .iter()
            .map(|entry| entry.count)
            .sum::<u32>(),
        1
    );
    assert_eq!(
        state
            .consecutive_loss_counts
            .iter()
            .map(|entry| entry.count)
            .max(),
        Some(3)
    );
    assert_eq!(state.blocked_strategy_symbols.len(), 1);
    let persisted = risk_store
        .load_runtime()
        .await
        .expect("runtime read")
        .unwrap();
    assert_eq!(
        persisted.daily_order_counts.len(),
        state.daily_order_counts.len()
    );
    assert_eq!(
        persisted.blocked_strategy_symbols.len(),
        state.blocked_strategy_symbols.len()
    );
    assert_eq!(persisted.current_loss, state.current_loss);
    let _ = tokio::fs::remove_dir_all(data_dir).await;
}
