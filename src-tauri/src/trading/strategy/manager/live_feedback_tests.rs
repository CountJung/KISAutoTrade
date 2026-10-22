use super::*;
use crate::broker::BrokerAccountId;

#[test]
fn live_feedback_manager_protection_metadata_requires_exact_owner_and_valid_checkpoint() {
    let config = StrategyConfig::new(
        "failed_breakout_fixture",
        "FB",
        true,
        vec!["SOXQ".into()],
        2,
        serde_json::json!({"lookback_days":3,"buffer_pct":0.5}),
    )
    .with_scope(BrokerId::Kis, Some("A".into()));
    let mut before = StrategyManager::new();
    before.add(build_strategy(config.clone()));
    before.initialize_historical("SOXQ", &[95, 100, 98]);
    before.set_history_execution_identity(scope("A"), None);
    assert_eq!(before.on_tick("SOXQ", 101, 0).len(), 1);
    let checkpoint = before
        .owned_position_metadata_for_strategy(
            "failed_breakout_fixture",
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
        )
        .unwrap();
    assert!(before
        .owned_position_metadata_for_strategy(
            "failed_breakout_fixture",
            &scope("B"),
            BrokerMarket::Us,
            "SOXQ"
        )
        .is_none());
    assert!(before
        .owned_position_metadata_for_strategy(
            "failed_breakout_fixture",
            &scope("A"),
            BrokerMarket::Kr,
            "SOXQ"
        )
        .is_none());
    let mut after = StrategyManager::new();
    after.add(build_strategy(config));
    after.set_history_execution_identity(scope("A"), None);
    assert!(after.sync_owned_position_for_strategy(
        "failed_breakout_fixture",
        &scope("A"),
        BrokerMarket::Us,
        "SOXQ",
        1,
        110
    ));
    assert!(!after
        .restore_owned_position_metadata_for_strategy(
            "failed_breakout_fixture",
            &scope("B"),
            BrokerMarket::Us,
            "SOXQ",
            &checkpoint
        )
        .unwrap());
    assert!(after
        .restore_owned_position_metadata_for_strategy(
            "failed_breakout_fixture",
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            &checkpoint
        )
        .unwrap());
    assert!(matches!(
        after.on_tick("SOXQ", 99, 0)[0].signal,
        Signal::Sell { .. }
    ));
    let mut malformed = checkpoint;
    malformed["version"] = serde_json::json!(3);
    assert!(after
        .restore_owned_position_metadata_for_strategy(
            "failed_breakout_fixture",
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            &malformed
        )
        .is_err());
}

fn config(id: &str, account: &str) -> StrategyConfig {
    StrategyConfig::new(
        id,
        "PC",
        true,
        vec![],
        2,
        serde_json::json!({"symbols":[{
        "symbol":"005930","symbol_name":"fixture","quantity":2,"buy_trigger_price":100.0,
        "take_profit_pct":5.0,"stop_loss_pct":3.0}]}),
    )
    .with_scope(BrokerId::Kis, Some(account.into()))
}
fn scope(account: &str) -> BrokerScope {
    BrokerScope::new(BrokerId::Kis, Some(BrokerAccountId(account.into())))
}

#[test]
fn live_feedback_manager_targets_exact_owner_scope_and_market() {
    let mut manager = StrategyManager::new();
    manager.add(Box::new(PriceConditionStrategy::new(config(
        "price_condition_a",
        "A",
    ))));
    manager.add(Box::new(PriceConditionStrategy::new(config(
        "price_condition_b",
        "A",
    ))));
    manager.set_history_execution_identity(scope("A"), None);
    assert!(manager.sync_owned_position_for_strategy(
        "price_condition_a",
        &scope("A"),
        BrokerMarket::Kr,
        "005930",
        2,
        100
    ));
    assert_eq!(
        manager.strategies[0].has_held_position("005930"),
        Some(true)
    );
    assert_eq!(
        manager.strategies[1].has_held_position("005930"),
        Some(false)
    );
    assert!(!manager.sync_owned_position_for_strategy(
        "price_condition_b",
        &scope("B"),
        BrokerMarket::Kr,
        "005930",
        5,
        100
    ));
    assert!(!manager.sync_owned_position_for_strategy(
        "price_condition_b",
        &scope("A"),
        BrokerMarket::Us,
        "005930",
        5,
        100
    ));
    assert!(!manager.sync_owned_position_for_strategy(
        "manual",
        &scope("A"),
        BrokerMarket::Kr,
        "005930",
        5,
        100
    ));
    let mut toss = scope("A");
    toss.broker_id = BrokerId::Toss;
    assert!(!manager.sync_owned_position_for_strategy(
        "price_condition_b",
        &toss,
        BrokerMarket::Kr,
        "005930",
        5,
        100
    ));
    let signals = manager.on_tick("005930", 90, 0);
    assert_eq!(signals.len(), 2);
    assert!(matches!(signals[0].signal, Signal::Sell { .. }));
    assert!(matches!(signals[1].signal, Signal::Buy { .. }));
    // Account holdings from another strategy/manual order cannot create ownership on a skip.
    let revision = manager.history_generation;
    manager.settle_unsubmitted_signal("price_condition_b", "005930", revision, 20, 200);
    assert_eq!(
        manager.strategies[1].has_held_position("005930"),
        Some(false)
    );
    assert_eq!(
        manager.strategies[0].has_held_position("005930"),
        Some(true)
    );
}

#[test]
fn live_feedback_manager_pending_and_unsettled_suppress_only_outward_signal_across_restart() {
    let mut manager = StrategyManager::new();
    manager.add(Box::new(PriceConditionStrategy::new(config(
        "price_condition_a",
        "A",
    ))));
    manager.set_history_execution_identity(scope("A"), None);
    assert_eq!(manager.on_tick("005930", 99, 0).len(), 1);
    assert!(manager.on_tick("005930", 98, 0).is_empty());
    let revision = manager.history_generation;
    manager.complete_signal("price_condition_a", "005930", revision);
    assert!(manager.note_order_pending(
        "price_condition_a",
        &scope("A"),
        BrokerMarket::Kr,
        "005930",
        true
    ));
    manager.clear_history_execution_identity();
    manager.set_history_execution_identity(scope("A"), None);
    assert!(manager.on_tick("005930", 97, 0).is_empty());
    assert_eq!(
        manager.strategies[0].has_held_position("005930"),
        Some(false)
    );
    assert!(manager.sync_owned_position_for_strategy(
        "price_condition_a",
        &scope("A"),
        BrokerMarket::Kr,
        "005930",
        1,
        98
    ));
    assert!(manager.note_order_pending(
        "price_condition_a",
        &scope("A"),
        BrokerMarket::Kr,
        "005930",
        false
    ));
    assert!(matches!(
        manager.on_tick("005930", 90, 0)[0].signal,
        Signal::Sell { .. }
    ));
    assert_eq!(
        manager.strategies[0].has_held_position("005930"),
        Some(true)
    );
}

#[test]
fn live_feedback_rsi_unfilled_and_ma_share_exact_replay_observations() {
    for (id, params) in [
        (
            "rsi",
            serde_json::json!({"period":2,"oversold":30.0,"overbought":70.0}),
        ),
        (
            "ma_cross",
            serde_json::json!({"short_period":1,"long_period":3}),
        ),
    ] {
        let config = StrategyConfig::new(id, id, true, vec!["SOXQ".into()], 1, params);
        let mut live = build_strategy(config.clone());
        let mut replay = build_strategy(config);
        live.initialize_historical("SOXQ", &[100, 110, 120, 110]);
        replay.initialize_historical("SOXQ", &[100, 110, 120, 110]);
        for price in [90, 80, 100, 130, 120] {
            assert_eq!(
                live.on_live_tick("SOXQ", price, 0),
                replay.on_tick("SOXQ", price, 0),
                "{id}"
            );
        }
        assert_eq!(
            live.has_held_position("SOXQ"),
            if id == "rsi" { Some(false) } else { None }
        );
    }
}

#[test]
fn live_feedback_deviation_manager_observations_match_replay_and_ownership_requires_feedback() {
    let id = "deviation_manager";
    let cfg = StrategyConfig::new(
        id,
        "fixture",
        true,
        vec!["SOXQ".into()],
        1,
        serde_json::json!({"ma_period":3,"buy_threshold_pct":-5.0,"sell_threshold_pct":5.0}),
    )
    .with_scope(BrokerId::Kis, Some("A".into()));
    let mut live = StrategyManager::new();
    live.add(build_strategy(cfg.clone()));
    live.set_history_execution_identity(scope("A"), None);
    let mut replay = StrategyManager::new();
    replay.add(build_strategy(cfg));
    live.initialize_historical("SOXQ", &[100, 110, 120, 110]);
    replay.initialize_historical("SOXQ", &[100, 110, 120, 110]);
    let mut entries = 0;
    for price in [90, 80, 100, 130, 120] {
        let a = live
            .on_tick("SOXQ", price, 0)
            .into_iter()
            .map(|s| s.signal)
            .collect::<Vec<_>>();
        let b = replay
            .on_tick("SOXQ", price, 0)
            .into_iter()
            .map(|s| s.signal)
            .collect::<Vec<_>>();
        assert_eq!(a, b, "flat observation {price}");
        assert!(
            a.iter().all(|s| matches!(s, Signal::Buy { .. })),
            "flat owner cannot Sell"
        );
        entries += a.len();
        assert_eq!(live.strategies[0].has_held_position("SOXQ"), Some(false));
        assert_eq!(replay.strategies[0].has_held_position("SOXQ"), Some(false));
        // Settle each raw signal so the live coordinator's pending fence does
        // not hide the next observation when comparing production semantics.
        live.complete_signal(id, "SOXQ", live.history_generation());
    }
    assert!(entries > 0);
    assert!(!live.sync_owned_position_for_strategy(
        id,
        &scope("B"),
        BrokerMarket::Us,
        "SOXQ",
        9,
        999
    ));
    assert_eq!(live.strategies[0].has_held_position("SOXQ"), Some(false));
    for manager in [&mut live, &mut replay] {
        assert!(manager.sync_owned_position_for_strategy(
            id,
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            5,
            110
        ));
        manager.initialize_historical("SOXQ", &[100, 100, 100]);
        assert_eq!(manager.strategies[0].has_held_position("SOXQ"), Some(true));
    }
    for price in [80, 60] {
        assert!(
            live.on_tick("SOXQ", price, 0).is_empty(),
            "held owner cannot Buy"
        );
        assert!(replay.on_tick("SOXQ", price, 0).is_empty());
    }
    let a = live.on_tick("SOXQ", 120, 0);
    let b = replay.on_tick("SOXQ", 120, 0);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(a[0].signal, b[0].signal);
    assert!(matches!(a[0].signal, Signal::Sell { quantity: 1, .. }));
    for manager in [&mut live, &mut replay] {
        assert_eq!(
            manager.strategies[0].has_held_position("SOXQ"),
            Some(true),
            "raw Sell is not a fill"
        );
        let before = manager
            .owned_position_metadata_for_strategy(id, &scope("A"), BrokerMarket::Us, "SOXQ")
            .unwrap();
        assert_eq!(before["heldObservations"], 3);
        assert!(manager.sync_owned_position_for_strategy(
            id,
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            2,
            105
        ));
        assert!(manager.sync_owned_position_for_strategy(
            id,
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            2,
            105
        ));
        let after = manager
            .owned_position_metadata_for_strategy(id, &scope("A"), BrokerMarket::Us, "SOXQ")
            .unwrap();
        assert_eq!(after, before, "same/partial feedback preserves held age");
    }
    live.complete_signal(id, "SOXQ", live.history_generation());
    let a = live.on_tick("SOXQ", 160, 0);
    let b = replay.on_tick("SOXQ", 160, 0);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(a[0].signal, b[0].signal);
    assert!(matches!(a[0].signal, Signal::Sell { quantity: 1, .. }));
    live.complete_signal(id, "SOXQ", live.history_generation());
    for manager in [&mut live, &mut replay] {
        assert!(manager.sync_owned_position_for_strategy(
            id,
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            0,
            0
        ));
        assert_eq!(manager.strategies[0].has_held_position("SOXQ"), Some(false));
        assert!(
            manager.on_tick("SOXQ", 200, 0).is_empty(),
            "confirmed flat owner cannot Sell"
        );
    }
}

#[test]
fn live_feedback_mean_reversion_manager_observations_match_replay_and_ownership_requires_feedback()
{
    let id = "mean_reversion_manager";
    let cfg = StrategyConfig::new(
        id,
        "fixture",
        true,
        vec!["SOXQ".into()],
        1,
        serde_json::json!({"period":3,"std_dev":1.0,"stop_loss_pct":50.0}),
    )
    .with_scope(BrokerId::Kis, Some("A".into()));
    let mut live = StrategyManager::new();
    live.add(build_strategy(cfg.clone()));
    live.set_history_execution_identity(scope("A"), None);
    let mut replay = StrategyManager::new();
    replay.add(build_strategy(cfg));
    live.initialize_historical("SOXQ", &[100, 110, 120, 110]);
    replay.initialize_historical("SOXQ", &[100, 110, 120, 110]);
    let mut entries = 0;
    for price in [90, 80, 100, 130, 120] {
        let a = live
            .on_tick("SOXQ", price, 0)
            .into_iter()
            .map(|s| s.signal)
            .collect::<Vec<_>>();
        let b = replay
            .on_tick("SOXQ", price, 0)
            .into_iter()
            .map(|s| s.signal)
            .collect::<Vec<_>>();
        assert_eq!(a, b, "flat observation {price}");
        assert!(
            a.iter().all(|s| matches!(s, Signal::Buy { .. })),
            "flat owner cannot Sell"
        );
        entries += a.len();
        assert_eq!(live.strategies[0].has_held_position("SOXQ"), Some(false));
        assert_eq!(replay.strategies[0].has_held_position("SOXQ"), Some(false));
        // Settle each raw signal so the live coordinator's pending fence does
        // not hide the next observation when comparing production semantics.
        live.complete_signal(id, "SOXQ", live.history_generation());
    }
    assert!(entries > 0);
    assert!(!live.sync_owned_position_for_strategy(
        id,
        &scope("B"),
        BrokerMarket::Us,
        "SOXQ",
        9,
        999
    ));
    assert_eq!(live.strategies[0].has_held_position("SOXQ"), Some(false));
    for manager in [&mut live, &mut replay] {
        assert!(manager.sync_owned_position_for_strategy(
            id,
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            5,
            110
        ));
        manager.initialize_historical("SOXQ", &[100, 100, 100]);
        assert_eq!(manager.strategies[0].has_held_position("SOXQ"), Some(true));
    }
    for price in [80, 60] {
        assert!(
            live.on_tick("SOXQ", price, 0).is_empty(),
            "held owner cannot Buy"
        );
        assert!(replay.on_tick("SOXQ", price, 0).is_empty());
    }
    let a = live.on_tick("SOXQ", 120, 0);
    let b = replay.on_tick("SOXQ", 120, 0);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(a[0].signal, b[0].signal);
    assert!(matches!(a[0].signal, Signal::Sell { quantity: 1, .. }));
    for manager in [&mut live, &mut replay] {
        assert_eq!(
            manager.strategies[0].has_held_position("SOXQ"),
            Some(true),
            "raw Sell is not a fill"
        );
        let before = manager
            .owned_position_metadata_for_strategy(id, &scope("A"), BrokerMarket::Us, "SOXQ")
            .unwrap();
        assert_eq!(before["heldObservations"], 3);
        assert!(manager.sync_owned_position_for_strategy(
            id,
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            2,
            105
        ));
        assert!(manager.sync_owned_position_for_strategy(
            id,
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            2,
            105
        ));
        let after = manager
            .owned_position_metadata_for_strategy(id, &scope("A"), BrokerMarket::Us, "SOXQ")
            .unwrap();
        assert_eq!(after, before, "same/partial feedback preserves held age");
    }
    live.complete_signal(id, "SOXQ", live.history_generation());
    let a = live.on_tick("SOXQ", 160, 0);
    let b = replay.on_tick("SOXQ", 160, 0);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(a[0].signal, b[0].signal);
    assert!(matches!(a[0].signal, Signal::Sell { quantity: 1, .. }));
    live.complete_signal(id, "SOXQ", live.history_generation());
    for manager in [&mut live, &mut replay] {
        assert!(manager.sync_owned_position_for_strategy(
            id,
            &scope("A"),
            BrokerMarket::Us,
            "SOXQ",
            0,
            0
        ));
        assert_eq!(manager.strategies[0].has_held_position("SOXQ"), Some(false));
        assert!(
            manager.on_tick("SOXQ", 200, 0).is_empty(),
            "confirmed flat owner cannot Sell"
        );
    }
}
