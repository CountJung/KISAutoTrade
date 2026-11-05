//! Production gates and order coordinator with synthetic localhost fixtures only.

use super::*;
use crate::trading::{
    configuration::ConfigurationEditReservation,
    order::test_support::{self, Fixture},
    strategy::{PriceConditionStrategy, StrategyConfig},
};
use std::sync::atomic::{AtomicUsize, Ordering};

async fn strategies(fixture: &Fixture) -> (Arc<Mutex<StrategyManager>>, ExecutionRunContext) {
    let mut manager = StrategyManager::new();
    for suffix in ["first", "second", "already_submitted"] {
        manager.add(Box::new(PriceConditionStrategy::new(
            StrategyConfig::new(
                format!("price_condition_{suffix}"),
                suffix,
                true,
                vec!["005930".into()],
                1,
                serde_json::json!({"symbols":[{
                    "symbol":"005930", "quantity":1, "buy_trigger_price":100.0,
                    "take_profit_pct":0.0, "stop_loss_pct":5.0, "is_overseas":false
                }]}),
            )
            .with_scope(
                fixture.profile.broker_id,
                Some(fixture.profile.account_no.clone()),
            ),
        )));
    }
    manager.set_history_execution_identity(
        test_support::scope(&fixture.profile),
        Some(fixture.profile.id.clone()),
    );
    let context = fixture.begin(manager.history_generation()).await;
    (Arc::new(Mutex::new(manager)), context)
}

#[tokio::test]
async fn failed_configuration_pause_never_becomes_market_close_and_same_revision_resumes() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let (strategies, context) = strategies(&fixture).await;
    let running = Arc::new(Mutex::new(true));
    let reservation = Arc::new(ConfigurationEditReservation {
        strategy_id: "price_condition_first".into(),
        profile_id: fixture.profile.id.clone(),
        scope: context.broker_scope.clone(),
        symbols: ["005930".into()].into_iter().collect(),
    });
    strategies
        .lock()
        .await
        .pause_configuration_edit(Arc::downgrade(&reservation));
    let evaluated = AtomicUsize::new(0);
    {
        let manager = strategies.lock().await;
        assert_eq!(
            market_readiness(&manager, |_| {
                evaluated.fetch_add(1, Ordering::SeqCst);
                false
            }),
            MarketReadiness::ConfigurationPending
        );
    }
    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
    assert!(capture_cycle(&running, &strategies, &fixture.manager)
        .await
        .is_none());
    // Save failure drops the lease without publishing any configuration revision.
    drop(reservation);
    let cycle = capture_cycle(&running, &strategies, &fixture.manager)
        .await
        .unwrap();
    assert_eq!(cycle.context, context);
    let manager = strategies.lock().await;
    assert_eq!(market_readiness(&manager, |_| true), MarketReadiness::Open);
    assert_eq!(
        market_readiness(&manager, |_| false),
        MarketReadiness::Closed
    );
}

#[tokio::test]
async fn unsubmitted_feedback_never_assigns_an_unowned_account_holding_to_strategy() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let (strategies, context) = strategies(&fixture).await;
    let running = Arc::new(Mutex::new(true));
    let raw = strategies
        .lock()
        .await
        .on_tick_filtered("005930", 100, 0, |config| {
            config.id == "price_condition_first"
        });
    assert_eq!(raw.len(), 1);
    // Another owner acquired an account holding while preflight was awaited.
    // A rejected raw entry must not acquire that position through rollback.
    fixture.seed_holding("005930", 1, 100).await;
    let fence = RunFence {
        running: &running,
        strategies: &strategies,
        orders: &fixture.manager,
        expected: &context,
    };
    assert!(fence.settle_unsubmitted_signals(&raw, None, 100).await);
    let mut manager = strategies.lock().await;
    assert!(!manager.has_unsettled_signals());
    let signals = manager.on_tick_filtered("005930", 94, 0, |config| {
        config.id == "price_condition_first"
    });
    assert!(matches!(
        signals[0].signal,
        crate::trading::strategy::Signal::Buy { .. }
    ));
}

#[tokio::test]
async fn market_closed_first_order_settles_remaining_raw_batch_without_touching_submitted_state() {
    async fn closed_order() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
        (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({"msg1":"모의투자 장종료 입니다."})),
        )
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = axum::Router::new().route(
        "/uapi/domestic-stock/v1/trading/order-cash",
        axum::routing::post(closed_order),
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let fixture = Fixture::new(base).await;
    let (strategies, context) = strategies(&fixture).await;
    fixture
        .manager
        .lock()
        .await
        .update_auto_trading_budget(&context.broker_scope, 3000, 0)
        .await
        .unwrap();
    let mut batch = {
        let mut manager = strategies.lock().await;
        let batch = manager.on_tick("005930", 99, 0);
        assert_eq!(batch.len(), 3);
        assert!(manager.has_unsettled_signals());
        batch
    };
    // This strategy has an accepted order. It stays pending without inventing a
    // confirmed position and must not emit another outward signal in this batch.
    let submitted = batch.pop().unwrap();
    strategies.lock().await.note_order_pending(
        &submitted.strategy_id,
        &context.broker_scope,
        crate::broker::BrokerMarket::Kr,
        "005930",
        true,
    );
    strategies.lock().await.complete_signal(
        &submitted.strategy_id,
        "005930",
        context.configuration_revision,
    );
    let error = OrderManager::submit_signal_shared(
        &fixture.manager,
        context.clone(),
        batch[0].clone(),
        "synthetic fixture".into(),
        100_000,
        None,
        99,
    )
    .await
    .unwrap_err();
    assert!(is_market_closed_error(&error.to_string()));
    let running = Arc::new(Mutex::new(true));
    let fence = RunFence {
        running: &running,
        strategies: &strategies,
        orders: &fixture.manager,
        expected: &context,
    };
    assert_eq!(
        finish_closed_batch(&fence, batch, None, 99).await,
        TickCycleResult::MarketClosed
    );
    let mut manager = strategies.lock().await;
    assert!(!manager.has_unsettled_signals());
    let next = manager.on_tick("005930", 99, 0);
    assert_eq!(next.len(), 2);
    assert!(next
        .iter()
        .all(|signal| signal.strategy_id != submitted.strategy_id));
    server.abort();
}

#[tokio::test]
async fn old_market_closed_batch_cannot_restore_positions_into_restarted_run() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    let (strategies, old) = strategies(&fixture).await;
    let batch = strategies.lock().await.on_tick("005930", 99, 0);
    let running = Arc::new(Mutex::new(true));
    {
        let _running = running.lock().await;
        let mut manager = strategies.lock().await;
        let mut orders = fixture.manager.lock().await;
        manager.advance_history_generation();
        manager.sync_position("005930", 7, 100);
        orders
            .begin_execution_run(
                old.broker_scope.clone(),
                Some(fixture.profile.id.clone()),
                manager.history_generation(),
                fixture.client.clone(),
                Some(fixture.profile.clone()),
            )
            .unwrap();
    }
    let fence = RunFence {
        running: &running,
        strategies: &strategies,
        orders: &fixture.manager,
        expected: &old,
    };
    assert_eq!(
        finish_closed_batch(&fence, batch, None, 99).await,
        TickCycleResult::Stopped
    );
    let mut manager = strategies.lock().await;
    assert!(!manager.has_unsettled_signals());
    assert!(manager.on_tick("005930", 99, 0).is_empty());
}
