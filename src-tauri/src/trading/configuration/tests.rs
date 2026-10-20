//! Real manager/store/coordinator; synthetic credentials and localhost-only clients.
use super::*;
use crate::trading::{
    order::test_support::{self, Fixture},
    strategy::{PriceConditionStrategy, Signal},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{
    sync::Notify,
    time::{timeout, Duration},
};

const ID: &str = "price_condition_fixture";
fn params(stop: f64, symbols: &[&str]) -> serde_json::Value {
    serde_json::json!({"symbols": symbols.iter().map(|symbol| serde_json::json!({
        "symbol":symbol,"quantity":1,"buy_trigger_price":100.0,
        "take_profit_pct":0.0,"stop_loss_pct":stop,"is_overseas":false
    })).collect::<Vec<_>>()})
}
async fn state(f: &Fixture, held: bool) -> ConfigurationUpdateState {
    let mut manager = StrategyManager::new();
    let config = StrategyConfig::new(
        ID,
        "fixture",
        true,
        vec!["005930".into()],
        1,
        params(5.0, &["005930"]),
    )
    .with_scope(f.profile.broker_id, Some(f.profile.account_no.clone()));
    manager.add(Box::new(PriceConditionStrategy::new(config)));
    manager.set_history_execution_identity(
        test_support::scope(&f.profile),
        Some(f.profile.id.clone()),
    );
    if held {
        manager.sync_position("005930", 1, 100);
        f.seed_holding("005930", 1, 100).await;
    }
    f.begin(manager.history_generation()).await;
    let store = Arc::new(StrategyStore::new(&f.path));
    let configs: Vec<_> = manager.all_configs().into_iter().cloned().collect();
    store.save(&f.profile.id, &configs).await.unwrap();
    ConfigurationUpdateState {
        update_lock: Arc::new(Mutex::new(())),
        strategies: Arc::new(Mutex::new(manager)),
        orders: f.manager.clone(),
        store,
    }
}
fn request(f: &Fixture) -> ConfigurationUpdateRequest {
    ConfigurationUpdateRequest {
        id: ID.into(),
        expected_profile_id: f.profile.id.clone(),
        expected_scope: test_support::scope(&f.profile),
        enabled: None,
        target_symbols: None,
        order_quantity: Some(2),
        params: Some(params(10.0, &["005930"])),
    }
}
async fn config(state: &ConfigurationUpdateState) -> StrategyConfig {
    state.strategies.lock().await.all_configs()[0].clone()
}
async fn unpaused(state: &ConfigurationUpdateState) {
    timeout(Duration::from_secs(3), async {
        loop {
            if !state.strategies.lock().await.configuration_edit_active() {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn real_store_failure_keeps_runtime_revision_and_held_stop_intact() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let mut s = state(&f, true).await;
    let before = f.manager.lock().await.execution_run_context();
    let blocker = f.path.join("not-a-directory");
    std::fs::write(&blocker, b"synthetic failure").unwrap();
    s.store = Arc::new(StrategyStore::new(&blocker));
    assert_eq!(
        update_strategy_configuration(s.clone(), request(&f))
            .await
            .unwrap_err()
            .code,
        "PERSISTENCE_ERROR"
    );
    assert_eq!(config(&s).await.order_quantity, 1);
    assert_eq!(f.manager.lock().await.execution_run_context(), before);
    assert!(!s.strategies.lock().await.configuration_edit_active());
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 94, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[tokio::test]
async fn save_that_replaces_disk_then_errors_restores_old_config_and_runtime() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let before = f.manager.lock().await.execution_run_context();
    let store = s.store.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    let error = run_update(s.clone(), request(&f), move |profile, configs| {
        let (store, c) = (store.clone(), c.clone());
        async move {
            store.save(&profile, &configs).await?;
            if c.fetch_add(1, Ordering::SeqCst) == 0 {
                anyhow::bail!("synthetic fsync failure after replace");
            }
            Ok(())
        }
    })
    .await
    .unwrap_err();
    assert_eq!(error.code, "PERSISTENCE_ERROR");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(config(&s).await.order_quantity, 1);
    assert_eq!(
        s.store.load(&f.profile.id).await.unwrap()[0].order_quantity,
        1
    );
    assert_eq!(f.manager.lock().await.execution_run_context(), before);
    assert!(!f.manager.lock().await.health_status().persistence_blocked);
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 94, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[tokio::test]
async fn uncertain_first_save_and_failed_restore_block_new_orders() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let store = s.store.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    let error = run_update(s.clone(), request(&f), move |profile, configs| {
        let (store, c) = (store.clone(), c.clone());
        async move {
            if c.fetch_add(1, Ordering::SeqCst) == 0 {
                store.save(&profile, &configs).await?;
            }
            anyhow::bail!("synthetic uncertain save/restore failure");
        }
    })
    .await
    .unwrap_err();
    assert_eq!(error.code, "PERSISTENCE_ERROR");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(s.orders.lock().await.health_status().persistence_blocked);
    assert_eq!(config(&s).await.order_quantity, 1);
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 94, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[tokio::test]
async fn successful_commit_preserves_held_state_applies_params_and_advances_revision() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let before = f.manager.lock().await.execution_run_context().unwrap();
    let updated = update_strategy_configuration(s.clone(), request(&f))
        .await
        .unwrap();
    assert_eq!(updated.order_quantity, 2);
    assert_eq!(
        s.store.load(&f.profile.id).await.unwrap()[0].params,
        updated.params
    );
    assert!(!f.manager.lock().await.execution_run_matches(&before));
    let mut manager = s.strategies.lock().await;
    assert!(manager.on_tick("005930", 94, 0).is_empty());
    assert!(matches!(
        manager.on_tick("005930", 89, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[tokio::test]
async fn delayed_save_pauses_ticks_without_locks_and_stop_can_complete() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let arrived = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());
    let (a, r, store) = (arrived.clone(), resume.clone(), s.store.clone());
    let job = spawn_update(s.clone(), request(&f), move |profile, configs| {
        let (a, r, store) = (a.clone(), r.clone(), store.clone());
        async move {
            a.notify_one();
            r.notified().await;
            store.save(&profile, &configs).await
        }
    });
    timeout(Duration::from_secs(3), arrived.notified())
        .await
        .unwrap();
    assert_eq!(config(&s).await.order_quantity, 1);
    {
        let mut manager = timeout(Duration::from_millis(300), s.strategies.lock())
            .await
            .unwrap();
        assert!(manager.active_symbols().is_empty());
        assert!(manager.on_tick("005930", 89, 0).is_empty());
        let mut orders = timeout(Duration::from_millis(300), s.orders.lock())
            .await
            .unwrap();
        manager.advance_history_generation();
        manager.clear_history_execution_identity();
        orders.end_execution_run();
    }
    resume.notify_one();
    assert_eq!(job.await.unwrap().unwrap().order_quantity, 2);
    assert!(f.manager.lock().await.execution_run_context().is_none());
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 89, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[tokio::test]
async fn cancelled_waiter_does_not_cancel_durable_save_and_publication() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let arrived = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());
    let (a, r, store) = (arrived.clone(), resume.clone(), s.store.clone());
    let job = spawn_update(s.clone(), request(&f), move |profile, configs| {
        let (a, r, store) = (a.clone(), r.clone(), store.clone());
        async move {
            a.notify_one();
            r.notified().await;
            store.save(&profile, &configs).await
        }
    });
    let waiter = tokio::spawn(job);
    timeout(Duration::from_secs(3), arrived.notified())
        .await
        .unwrap();
    waiter.abort();
    assert!(waiter.await.unwrap_err().is_cancelled());
    resume.notify_one();
    unpaused(&s).await;
    assert_eq!(config(&s).await.order_quantity, 2);
    assert_eq!(
        s.store.load(&f.profile.id).await.unwrap()[0].order_quantity,
        2
    );
}

#[tokio::test]
async fn held_effective_target_removal_is_rejected_before_persistence() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let mut req = request(&f);
    req.target_symbols = Some(vec!["005930".into()]);
    req.params = Some(params(10.0, &["000660"]));
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    let result = run_update(s.clone(), req, move |_, _| {
        c.fetch_add(1, Ordering::SeqCst);
        std::future::ready(Ok(()))
    })
    .await;
    assert_eq!(result.unwrap_err().code, "HELD_TARGET_REMOVAL");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(config(&s).await.target_symbols, ["005930"]);
}

#[tokio::test]
async fn empty_price_condition_entries_reject_held_removal_without_touching_disk() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let before = f.manager.lock().await.execution_run_context();
    let mut req = request(&f);
    req.target_symbols = Some(vec!["005930".into()]);
    req.params = Some(params(10.0, &[]));
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    let error = run_update(s.clone(), req, move |_, _| {
        c.fetch_add(1, Ordering::SeqCst);
        std::future::ready(Ok(()))
    })
    .await
    .unwrap_err();
    assert_eq!(error.code, "HELD_TARGET_REMOVAL");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        s.store.load(&f.profile.id).await.unwrap()[0].target_symbols,
        ["005930"]
    );
    assert_eq!(f.manager.lock().await.execution_run_context(), before);
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 94, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[tokio::test]
async fn zero_in_a_shared_tracker_neither_erases_protection_nor_permits_held_removal() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    f.seed_holding("005930", 0, 0).await;
    let mut removal = request(&f);
    removal.params = Some(params(10.0, &[]));
    assert_eq!(
        update_strategy_configuration(s.clone(), removal)
            .await
            .unwrap_err()
            .code,
        "HELD_TARGET_REMOVAL"
    );
    update_strategy_configuration(s.clone(), request(&f))
        .await
        .unwrap();
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 89, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[tokio::test]
async fn adding_a_target_does_not_transfer_other_strategy_or_manual_holdings() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    f.seed_holding("000660", 5, 100).await;
    let mut req = request(&f);
    let mut p = params(10.0, &["005930", "000660"]);
    p["symbols"][1]["buy_trigger_price"] = serde_json::json!(0.0);
    req.params = Some(p);
    update_strategy_configuration(s.clone(), req).await.unwrap();
    assert!(s
        .strategies
        .lock()
        .await
        .on_tick("000660", 50, 0)
        .is_empty());
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 89, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[tokio::test]
async fn outstanding_raw_signal_blocks_edit_until_current_run_feedback_settles() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, false).await;
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 99, 0)[0].signal,
        Signal::Buy { .. }
    ));
    assert_eq!(
        update_strategy_configuration(s.clone(), request(&f))
            .await
            .unwrap_err()
            .code,
        "CONFIGURATION_BUSY"
    );
    {
        let mut manager = s.strategies.lock().await;
        let generation = manager.history_generation();
        manager.sync_position("005930", 0, 0);
        manager.complete_signal(ID, "005930", generation);
    }
    update_strategy_configuration(s.clone(), request(&f))
        .await
        .unwrap();
}

#[tokio::test]
async fn selecting_another_profile_resets_private_positions_even_for_the_same_account() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let cfg = config(&s).await;
    let mut manager = s.strategies.lock().await;
    manager.replace_saved_configs_for_scope(
        &[cfg],
        f.profile.broker_id,
        Some(f.profile.account_no.clone()),
    );
    assert!(manager.history_execution_identity().is_none());
    assert!(matches!(
        manager.on_tick("005930", 89, 0)[0].signal,
        Signal::Buy { .. }
    ));
}

#[tokio::test]
async fn empty_saved_profile_does_not_inherit_price_condition_or_lth_entries() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let mut manager = s.strategies.lock().await;
    let lth = StrategyConfig::new(
        "leveraged_trend_hold_fixture",
        "fixture",
        true,
        vec![],
        7,
        serde_json::json!({"entries":[{"leveraged_symbol":"AAPL","is_overseas":true}]}),
    )
    .with_scope(f.profile.broker_id, Some(f.profile.account_no.clone()));
    manager.add(build_strategy(lth));
    assert!(manager
        .all_configs()
        .iter()
        .all(|config| !config.target_symbols.is_empty()));
    manager.replace_saved_configs_for_scope(
        &[],
        f.profile.broker_id,
        Some(f.profile.account_no.clone()),
    );
    for config in manager.all_configs() {
        assert!(!config.enabled);
        assert!(config.target_symbols.is_empty());
        assert_eq!(config.order_quantity, 1);
        assert_eq!(config.params, serde_json::json!({}));
    }
    assert!(manager.history_execution_identity().is_none());
}

#[tokio::test]
async fn post_save_scope_drift_restores_disk_without_publishing_runtime() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let (store, orders) = (s.store.clone(), s.orders.clone());
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    let error = run_update(s.clone(), request(&f), move |profile, configs| {
        let (store, orders, c) = (store.clone(), orders.clone(), c.clone());
        async move {
            store.save(&profile, &configs).await?;
            if c.fetch_add(1, Ordering::SeqCst) == 0 {
                orders
                    .lock()
                    .await
                    .set_execution_scope(crate::broker::BrokerScope::kis_legacy());
            }
            Ok(())
        }
    })
    .await
    .unwrap_err();
    assert_eq!(error.code, "SCOPE_MISMATCH");
    assert_eq!(count.load(Ordering::SeqCst), 2);
    assert_eq!(config(&s).await.order_quantity, 1);
    assert_eq!(
        s.store.load(&f.profile.id).await.unwrap()[0].order_quantity,
        1
    );
}

#[tokio::test]
async fn failed_post_save_restore_blocks_all_new_orders_and_reports_persistence_error() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, true).await;
    let (store, orders) = (s.store.clone(), s.orders.clone());
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    let error = run_update(s.clone(), request(&f), move |profile, configs| {
        let (store, orders, c) = (store.clone(), orders.clone(), c.clone());
        async move {
            if c.fetch_add(1, Ordering::SeqCst) > 0 {
                anyhow::bail!("synthetic restore failure");
            }
            store.save(&profile, &configs).await?;
            orders
                .lock()
                .await
                .set_execution_scope(BrokerScope::kis_legacy());
            Ok(())
        }
    })
    .await
    .unwrap_err();
    assert_eq!(error.code, "PERSISTENCE_ERROR");
    assert!(s.orders.lock().await.health_status().persistence_blocked);
    assert_eq!(config(&s).await.order_quantity, 1);
}

#[tokio::test]
async fn invalid_consecutive_optional_params_are_rejected_before_persistence_or_runtime_publication(
) {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, false).await;
    let id = "consecutive_move_configuration";
    let original = StrategyConfig::new(
        id,
        "fixture",
        true,
        vec!["005930".into()],
        1,
        serde_json::json!({"buy_days":2,"sell_days":4}),
    )
    .with_scope(f.profile.broker_id, Some(f.profile.account_no.clone()));
    {
        let mut manager = s.strategies.lock().await;
        *manager = StrategyManager::new();
        manager.add(build_strategy(original.clone()));
        manager.set_history_execution_identity(
            test_support::scope(&f.profile),
            Some(f.profile.id.clone()),
        );
    }
    let calls = Arc::new(AtomicUsize::new(0));
    for period in [2.5, 0.5] {
        let mut update = request(&f);
        update.id = id.into();
        update.params =
            Some(serde_json::json!({"buy_days":2,"sell_days":4,"long_trend_period":period}));
        let c = calls.clone();
        let error = run_update(s.clone(), update, move |_, _| {
            c.fetch_add(1, Ordering::SeqCst);
            async { Ok(()) }
        })
        .await
        .unwrap_err();
        assert_eq!(error.code, "INVALID_STRATEGY_PARAMS");
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(config(&s).await.params, original.params);
    assert!(!s.strategies.lock().await.configuration_edit_active());

    // JSON numeric integers such as 200.0 are accepted consistently by the
    // validator, durable save and the production constructor; never disabled.
    for period in [200.0, 512.0] {
        let mut update = request(&f);
        update.id = id.into();
        update.params =
            Some(serde_json::json!({"buy_days":2,"sell_days":4,"long_trend_period":period}));
        let published = update_strategy_configuration(s.clone(), update)
            .await
            .unwrap();
        assert_eq!(
            published.params["long_trend_period"],
            serde_json::json!(period)
        );
        let loaded = s.store.load(&f.profile.id).await.unwrap();
        assert_eq!(loaded[0].params, published.params);
        let strategy = build_strategy(loaded[0].clone());
        assert_eq!(
            strategy.required_completed_daily_bars(),
            Some(period as usize)
        );
        assert_eq!(published.params["buy_days"], 2);
        assert_eq!(published.params["sell_days"], 4);
    }
}

#[tokio::test]
async fn momentum_entry_fields_are_rejected_before_save_and_valid_modes_roundtrip() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, false).await;
    let id = "momentum_configuration";
    let original = StrategyConfig::new(
        id,
        "fixture",
        true,
        vec!["005930".into()],
        1,
        serde_json::json!({"lookback_period":3,"threshold_pct":5.0}),
    )
    .with_scope(f.profile.broker_id, Some(f.profile.account_no.clone()));
    {
        let mut manager = s.strategies.lock().await;
        *manager = StrategyManager::new();
        manager.add(build_strategy(original.clone()));
        manager.set_history_execution_identity(
            test_support::scope(&f.profile),
            Some(f.profile.id.clone()),
        );
    }
    let calls = Arc::new(AtomicUsize::new(0));
    for extra in [
        serde_json::json!({"entry_mode":"unknown"}),
        serde_json::json!({"entry_mode":null}),
        serde_json::json!({"entry_hysteresis_pct":"1"}),
        serde_json::json!({"entry_hysteresis_pct":101}),
        serde_json::json!({"entry_mode":"crossing","entry_hysteresis_pct":6}),
    ] {
        let mut params = original.params.clone();
        params
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let mut update = request(&f);
        update.id = id.into();
        update.params = Some(params);
        let c = calls.clone();
        let error = run_update(s.clone(), update, move |_, _| {
            c.fetch_add(1, Ordering::SeqCst);
            async { Ok(()) }
        })
        .await
        .unwrap_err();
        assert_eq!(error.code, "INVALID_STRATEGY_PARAMS");
        assert_eq!(config(&s).await.params, original.params);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    for (mode, hysteresis) in [("level", 100.0), ("crossing", 1.0)] {
        let mut params = original.params.clone();
        params["entry_mode"] = serde_json::json!(mode);
        params["entry_hysteresis_pct"] = serde_json::json!(hysteresis);
        let mut update = request(&f);
        update.id = id.into();
        update.params = Some(params.clone());
        let published = update_strategy_configuration(s.clone(), update)
            .await
            .unwrap();
        assert_eq!(published.params, params);
        assert_eq!(s.store.load(&f.profile.id).await.unwrap()[0].params, params);
    }
    assert!(!s.strategies.lock().await.configuration_edit_active());
}

#[tokio::test]
async fn deviation_options_reject_before_save_and_integer_float_values_roundtrip() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, false).await;
    let id = "deviation_configuration";
    let original = StrategyConfig::new(
        id,
        "fixture",
        true,
        vec!["005930".into()],
        1,
        serde_json::json!({"ma_period":3,"buy_threshold_pct":-5.0,"sell_threshold_pct":5.0}),
    )
    .with_scope(f.profile.broker_id, Some(f.profile.account_no.clone()));
    {
        let mut manager = s.strategies.lock().await;
        *manager = StrategyManager::new();
        manager.add(build_strategy(original.clone()));
        manager.set_history_execution_identity(
            test_support::scope(&f.profile),
            Some(f.profile.id.clone()),
        );
    }
    let calls = Arc::new(AtomicUsize::new(0));
    for extra in [
        serde_json::json!({"entry_mode":"bad"}),
        serde_json::json!({"entry_mode":null}),
        serde_json::json!({"long_trend_period":2.5}),
        serde_json::json!({"long_trend_period":513}),
        serde_json::json!({"max_hold_observations":1_000_001}),
        serde_json::json!({"max_hold_observations":0.5}),
        serde_json::json!({"stop_loss_pct":-1}),
        serde_json::json!({"stop_loss_pct":"10"}),
    ] {
        let mut params = original.params.clone();
        params
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let mut update = request(&f);
        update.id = id.into();
        update.params = Some(params);
        let c = calls.clone();
        let error = run_update(s.clone(), update, move |_, _| {
            c.fetch_add(1, Ordering::SeqCst);
            async { Ok(()) }
        })
        .await
        .unwrap_err();
        assert_eq!(error.code, "INVALID_STRATEGY_PARAMS");
        assert_eq!(config(&s).await.params, original.params);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    for mode in ["level", "crossing"] {
        let mut params = original.params.clone();
        params["entry_mode"] = serde_json::json!(mode);
        params["long_trend_period"] = serde_json::json!(200.0);
        params["max_hold_observations"] = serde_json::json!(63.0);
        params["stop_loss_pct"] = serde_json::json!(10.0);
        let mut update = request(&f);
        update.id = id.into();
        update.params = Some(params.clone());
        let published = update_strategy_configuration(s.clone(), update)
            .await
            .unwrap();
        assert_eq!(published.params, params);
        assert_eq!(s.store.load(&f.profile.id).await.unwrap()[0].params, params);
        let mut strategy = build_strategy(published);
        strategy.sync_position("005930", 3, 100);
        assert!(matches!(
            strategy.on_tick("005930", 90, 0),
            Signal::Sell { quantity: 3, .. }
        ));
    }
    assert!(!s.strategies.lock().await.configuration_edit_active());
}

#[tokio::test]
async fn mean_reversion_options_reject_before_save_and_integer_float_values_roundtrip() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, false).await;
    let id = "mean_reversion_configuration";
    let original = StrategyConfig::new(
        id,
        "fixture",
        true,
        vec!["005930".into()],
        1,
        serde_json::json!({"period":3,"std_dev":1.0,"stop_loss_pct":3.0}),
    )
    .with_scope(f.profile.broker_id, Some(f.profile.account_no.clone()));
    {
        let mut manager = s.strategies.lock().await;
        *manager = StrategyManager::new();
        manager.add(build_strategy(original.clone()));
        manager.set_history_execution_identity(
            test_support::scope(&f.profile),
            Some(f.profile.id.clone()),
        );
    }
    let calls = Arc::new(AtomicUsize::new(0));
    for extra in [
        serde_json::json!({"exit_mode":"bad"}),
        serde_json::json!({"exit_mode":null}),
        serde_json::json!({"long_trend_period":2.5}),
        serde_json::json!({"long_trend_period":513}),
        serde_json::json!({"max_hold_observations":1_000_001}),
        serde_json::json!({"max_hold_observations":0.5}),
    ] {
        let mut params = original.params.clone();
        params
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let mut update = request(&f);
        update.id = id.into();
        update.params = Some(params);
        let c = calls.clone();
        let error = run_update(s.clone(), update, move |_, _| {
            c.fetch_add(1, Ordering::SeqCst);
            async { Ok(()) }
        })
        .await
        .unwrap_err();
        assert_eq!(error.code, "INVALID_STRATEGY_PARAMS");
        assert_eq!(config(&s).await.params, original.params);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    for mode in ["upperBand", "middleBand"] {
        let mut params = original.params.clone();
        params["exit_mode"] = serde_json::json!(mode);
        params["long_trend_period"] = serde_json::json!(200.0);
        params["max_hold_observations"] = serde_json::json!(63.0);
        let mut update = request(&f);
        update.id = id.into();
        update.params = Some(params.clone());
        let published = update_strategy_configuration(s.clone(), update)
            .await
            .unwrap();
        assert_eq!(published.params, params);
        assert_eq!(s.store.load(&f.profile.id).await.unwrap()[0].params, params);
        let mut strategy = build_strategy(published);
        strategy.sync_position("005930", 3, 100);
        assert!(matches!(
            strategy.on_tick("005930", 90, 0),
            Signal::Sell { quantity: 2, .. }
        ));
    }
    assert!(!s.strategies.lock().await.configuration_edit_active());
}
