use super::*;
use crate::trading::order::test_support::{self, Fixture};

fn config(id: &str, symbols: &[&str]) -> StrategyConfig {
    StrategyConfig::new(
        id,
        "fixture",
        true,
        symbols.iter().map(|symbol| (*symbol).into()).collect(),
        1,
        serde_json::json!({}),
    )
}
fn reservation(
    f: &Fixture,
    previous: &StrategyConfig,
    next: &StrategyConfig,
) -> Arc<ConfigurationEditReservation> {
    Arc::new(ConfigurationEditReservation {
        strategy_id: previous.id.clone(),
        profile_id: f.profile.id.clone(),
        scope: test_support::scope(&f.profile),
        symbols: previous
            .target_symbols
            .iter()
            .chain(next.target_symbols.iter())
            .cloned()
            .collect(),
    })
}
async fn fixture() -> Fixture {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    f.manager
        .lock()
        .await
        .set_execution_scope(test_support::scope(&f.profile));
    f
}
fn pending(scope: BrokerScope, symbol: &str) -> PendingOrder {
    PendingOrder {
        ownership_key: None,
        ownership_generation: None,
        strategy_position_metadata: None,
        submitted_cost_basis: None,
        record: OrderRecord::new(
            symbol.into(),
            "fixture".into(),
            OrderSide::Buy,
            1,
            100,
            "Limit".into(),
        )
        .with_broker_scope(&scope),
        signal_reason: "fixture".into(),
        strategy_id: Some("fixture".into()),
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
    }
}

#[tokio::test]
async fn configuration_snapshots_include_real_held_and_flat_markets_and_revalidate_fresh_quantity()
{
    let f = fixture().await;
    let previous = config("moving_average_cross", &["005930", "AAPL", "000660"]);
    let mut next = previous.clone();
    next.order_quantity = 2;
    let edit = reservation(&f, &previous, &next);
    let mut manager = f.manager.lock().await;
    manager
        .position_tracker
        .lock()
        .await
        .on_buy("005930".into(), "fixture".into(), 3, 100);
    manager.overseas_position_tracker.lock().await.on_buy(
        "AAPL".into(),
        "fixture".into(),
        "NASD".into(),
        2,
        15000,
    );
    let initial = manager
        .reserve_configuration_edit(&edit, &previous, &next)
        .await
        .unwrap();
    assert_eq!(initial.len(), 3);
    assert_eq!(
        initial
            .iter()
            .find(|s| s.symbol == "005930")
            .unwrap()
            .quantity,
        3
    );
    let us = initial.iter().find(|s| s.symbol == "AAPL").unwrap();
    assert_eq!(
        (us.market, us.quantity, us.avg_price),
        (BrokerMarket::Us, 2, 15000)
    );
    assert_eq!(
        initial
            .iter()
            .find(|s| s.symbol == "000660")
            .unwrap()
            .quantity,
        0
    );
    manager.position_tracker.lock().await.on_sell("005930", 3);
    let current = manager
        .configuration_position_snapshots(&edit)
        .await
        .unwrap();
    assert_eq!(
        current
            .iter()
            .find(|s| s.symbol == "005930")
            .unwrap()
            .quantity,
        0
    );
    assert_eq!(Arc::strong_count(&edit), 1);
}

#[tokio::test]
async fn held_target_removal_and_explicit_pc_lth_market_changes_fail_closed() {
    for strategy in [
        "moving_average_cross",
        "price_condition",
        "leveraged_trend_hold",
    ] {
        let f = fixture().await;
        let mut previous = config(strategy, &["005930"]);
        let mut next = previous.clone();
        if strategy == "moving_average_cross" {
            next.target_symbols.clear();
        } else if strategy == "price_condition" {
            previous.params =
                serde_json::json!({"symbols":[{"symbol":"005930","is_overseas":false}]});
            next.params = serde_json::json!({"symbols":[{"symbol":"005930","is_overseas":true}]});
        } else {
            previous.params =
                serde_json::json!({"entries":[{"leveraged_symbol":"005930","is_overseas":false}]});
            next.params =
                serde_json::json!({"entries":[{"leveraged_symbol":"005930","is_overseas":true}]});
        }
        let edit = reservation(&f, &previous, &next);
        let mut manager = f.manager.lock().await;
        manager
            .position_tracker
            .lock()
            .await
            .on_buy("005930".into(), "fixture".into(), 3, 100);
        let error = manager
            .reserve_configuration_edit(&edit, &previous, &next)
            .await
            .unwrap_err();
        assert_eq!(
            error.code,
            if strategy == "moving_average_cross" {
                "HELD_TARGET_REMOVAL"
            } else {
                "HELD_MARKET_CHANGE"
            }
        );
        assert!(!manager.configuration_blocks_order(&edit.scope, "005930"));
    }
}

#[tokio::test]
async fn duplicate_entries_use_the_same_first_market_as_the_live_strategy() {
    for strategy in ["price_condition", "leveraged_trend_hold"] {
        let f = fixture().await;
        let mut previous = config(strategy, &["005930"]);
        let (key, symbol_key) = if strategy == "price_condition" {
            ("symbols", "symbol")
        } else {
            ("entries", "leveraged_symbol")
        };
        previous.params = serde_json::json!({key:[
            {symbol_key:"005930","is_overseas":false},
            {symbol_key:"005930","is_overseas":true}
        ]});
        previous = crate::trading::strategy::build_strategy(previous)
            .config()
            .clone();
        let mut next = previous.clone();
        next.params[key][0]["is_overseas"] = true.into();
        next = crate::trading::strategy::build_strategy(next)
            .config()
            .clone();
        let edit = reservation(&f, &previous, &next);
        f.seed_holding("005930", 1, 100).await;
        assert_eq!(
            f.manager
                .lock()
                .await
                .reserve_configuration_edit(&edit, &previous, &next)
                .await
                .unwrap_err()
                .code,
            "HELD_MARKET_CHANGE"
        );
    }
}

#[tokio::test]
async fn legacy_inverse_field_cannot_override_another_lth_targets_market() {
    let f = fixture().await;
    let mut previous = config("leveraged_trend_hold", &["AAPL", "005930"]);
    previous.params = serde_json::json!({"entries":[
        {"leveraged_symbol":"AAPL","is_overseas":true},
        {"leveraged_symbol":"005930","inverse_leveraged_symbol":"AAPL","is_overseas":false}
    ]});
    previous = crate::trading::strategy::build_strategy(previous)
        .config()
        .clone();
    let mut next = previous.clone();
    next.params["entries"][0]["is_overseas"] = false.into();
    let edit = reservation(&f, &previous, &next);
    f.seed_holding("AAPL", 1, 100).await;
    assert_eq!(
        f.manager
            .lock()
            .await
            .reserve_configuration_edit(&edit, &previous, &next)
            .await
            .unwrap_err()
            .code,
        "HELD_MARKET_CHANGE"
    );
}

#[tokio::test]
async fn newly_observed_held_position_cannot_be_removed_at_commit() {
    let f = fixture().await;
    let previous = config("moving_average_cross", &["005930"]);
    let next = config("moving_average_cross", &[]);
    let edit = reservation(&f, &previous, &next);
    let mut manager = f.manager.lock().await;
    manager
        .reserve_configuration_edit(&edit, &previous, &next)
        .await
        .unwrap();
    manager
        .position_tracker
        .lock()
        .await
        .on_buy("005930".into(), "fixture".into(), 1, 100);
    assert_eq!(
        manager
            .configuration_position_snapshots(&edit)
            .await
            .unwrap_err()
            .code,
        "HELD_TARGET_REMOVAL"
    );
}

#[tokio::test]
async fn scoped_pending_submitting_and_modifying_orders_block_reservation_but_unrelated_orders_continue(
) {
    for state in 0..3 {
        let f = fixture().await;
        let previous = config("moving_average_cross", &["005930"]);
        let edit = reservation(&f, &previous, &previous);
        let mut manager = f.manager.lock().await;
        if state == 0 {
            manager
                .submitting
                .insert((edit.scope.clone(), "005930".into()), OrderSide::Buy);
        } else {
            manager.track_pending_order("pending-id".into(), pending(edit.scope.clone(), "005930"));
            if state == 2 {
                manager.begin_pending_modification("pending-id").unwrap();
            }
        }
        assert_eq!(
            manager
                .reserve_configuration_edit(&edit, &previous, &previous)
                .await
                .unwrap_err()
                .code,
            "CONFIGURATION_BUSY"
        );
    }
    let f = fixture().await;
    let previous = config("moving_average_cross", &["005930"]);
    let edit = reservation(&f, &previous, &previous);
    let mut manager = f.manager.lock().await;
    manager
        .submitting
        .insert((edit.scope.clone(), "000660".into()), OrderSide::Buy);
    manager.track_pending_order(
        "different-scope".into(),
        pending(
            test_support::scope(&test_support::profile("8765432101")),
            "005930",
        ),
    );
    manager
        .reserve_configuration_edit(&edit, &previous, &previous)
        .await
        .unwrap();
    assert!(manager.configuration_blocks_order(&edit.scope, "005930"));
    assert!(!manager.configuration_blocks_order(&edit.scope, "000660"));
    assert!(!manager.configuration_blocks_order(
        &test_support::scope(&test_support::profile("8765432101")),
        "005930"
    ));
    assert!(manager.configuration_blocks_holdings(&edit.scope));
    drop(edit);
    assert!(!manager.configuration_blocks_order(&test_support::scope(&f.profile), "005930"));
    assert!(!manager.configuration_blocks_holdings(&test_support::scope(&f.profile)));
}

#[tokio::test]
async fn commit_rejects_scope_profile_client_and_credential_changes() {
    for change in 0..4 {
        let f = fixture().await;
        let previous = config("moving_average_cross", &["005930"]);
        let edit = reservation(&f, &previous, &previous);
        let mut manager = f.manager.lock().await;
        manager
            .reserve_configuration_edit(&edit, &previous, &previous)
            .await
            .unwrap();
        match change {
            0 => manager
                .set_execution_scope(test_support::scope(&test_support::profile("8765432101"))),
            1 => manager.profiles.write().await.active_id = None,
            2 => {
                *manager.rest_client.write().await =
                    test_support::client("http://127.0.0.1:1".into(), &f.profile).await
            }
            _ => manager.profiles.write().await.profiles[0]
                .app_secret
                .push_str("-changed"),
        }
        assert_eq!(
            manager
                .configuration_position_snapshots(&edit)
                .await
                .unwrap_err()
                .code,
            "SCOPE_MISMATCH"
        );
    }
}

#[tokio::test]
async fn configuration_fence_blocks_shared_automatic_and_manual_before_any_http_or_budget_change() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let requests = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let counter = Arc::clone(&requests);
    let router = axum::Router::new().fallback(move || {
        let counter = Arc::clone(&counter);
        async move {
            counter.fetch_add(1, Ordering::SeqCst);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    });
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let f = Fixture::new(base_url).await;
    let run = f.begin(1).await;
    let previous = config("moving_average_cross", &["005930"]);
    let edit = reservation(&f, &previous, &previous);
    f.manager
        .lock()
        .await
        .reserve_configuration_edit(&edit, &previous, &previous)
        .await
        .unwrap();
    let auto = OrderManager::submit_signal_shared(
        &f.manager,
        run,
        crate::trading::strategy::StrategySignal {
            strategy_id: previous.id,
            signal: Signal::Buy {
                symbol: "005930".into(),
                quantity: 1,
                reason: "fixture".into(),
            },
        },
        "fixture".into(),
        100_000,
        None,
        100,
    )
    .await
    .unwrap();
    assert!(matches!(auto, SubmissionOutcome::Skipped { reason,.. } if reason.contains("설정")));
    // A different symbol still refreshes the entire account, so manual submission is scope-blocked.
    let manual = OrderManager::submit_manual_order_shared(
        &f.manager,
        "000660".into(),
        "fixture".into(),
        RestOrderSide::Buy,
        OrderType::Limit,
        1,
        100,
        100,
        100_000,
        None,
        edit.scope.clone(),
    )
    .await
    .unwrap();
    assert!(matches!(manual, SubmissionOutcome::Skipped { reason,.. } if reason.contains("설정")));
    let manager = f.manager.lock().await;
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    assert!(manager.budget.is_none());
    assert!(manager.pending.is_empty());
    assert!(manager.submitting.is_empty());
    assert_eq!(Arc::strong_count(&edit), 1);
    server.abort();
}
