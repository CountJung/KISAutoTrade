//! StrongClose verified day fixture: coordinator guards, no provider/account calls.
use super::*;
const ID: &str = "strong_close_live_fixture";
fn strong_manager() -> StrategyManager {
    let mut m = StrategyManager::new();
    m.add(build_strategy(
        StrategyConfig::new(
            ID,
            "fixture",
            true,
            vec!["SOXQ".into()],
            100,
            serde_json::json!({}),
        )
        .with_scope(BrokerId::Kis, Some("fixture-account".into())),
    ));
    m.set_history_execution_identity(scope(), Some("fixture-profile".into()));
    m
}
fn strong_bar(price: u64) -> [OhlcCandle; 1] {
    [OhlcCandle {
        open: price,
        high: price,
        low: price,
        close: price,
    }]
}
fn release(m: &mut StrategyManager) {
    m.complete_signal(ID, "SOXQ", m.history_generation());
}
#[test]
fn strong_close_manager_new_day_reseeds_same_day_refresh_never_rearms_consumed() {
    let mut m = strong_manager();
    let t = prepare(&mut m, now());
    assert_eq!(t.required_bars, 1);
    assert!(apply(&mut m, &t, &strong_bar(100), now()));
    assert!(matches!(
        m.on_tick("SOXQ", 101, 0)[0].signal,
        Signal::Buy { .. }
    ));
    m.sync_position("SOXQ", 7, 101);
    release(&mut m);
    m.sync_position("SOXQ", 0, 0);
    assert!(apply(&mut m, &t, &strong_bar(102), now()));
    assert!(m.on_tick("SOXQ", 103, 0).is_empty());
    let tomorrow = now() + Duration::days(1);
    let t2 = prepare(&mut m, tomorrow);
    assert!(m.on_tick("SOXQ", 103, 0).is_empty());
    assert!(!apply(&mut m, &t, &strong_bar(100), tomorrow));
    assert!(apply(&mut m, &t2, &strong_bar(102), tomorrow));
    assert_eq!(m.on_tick("SOXQ", 103, 0).len(), 1);
}
#[test]
fn strong_close_manager_unfilled_pending_failure_and_legacy_warmup_preserve_original_proof() {
    let mut m = strong_manager();
    let t = prepare(&mut m, now());
    assert!(apply(&mut m, &t, &strong_bar(100), now()));
    assert_eq!(m.on_tick("SOXQ", 101, 0).len(), 1);
    assert!(m.note_order_pending(ID, &scope(), BrokerMarket::Us, "SOXQ", true));
    let proof = m
        .owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ")
        .unwrap();
    assert!(m.mark_daily_history_failure(&t, "synthetic", None, now()));
    assert!(m.on_tick("SOXQ", 102, 0).is_empty());
    m.initialize_warmup("SOXQ", &strong_bar(1), &[]);
    let t2 = prepare(&mut m, now());
    assert!(apply(&mut m, &t2, &strong_bar(103), now()));
    assert!(m.on_tick("SOXQ", 104, 0).is_empty());
    assert_eq!(
        m.owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ"),
        Some(proof)
    );
    m.sync_position("SOXQ", 7, 104);
    release(&mut m);
    assert!(m.note_order_pending(ID, &scope(), BrokerMarket::Us, "SOXQ", false));
    let metadata = m
        .owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ")
        .unwrap();
    assert_eq!(metadata["heldObservations"], 0);
    assert_eq!(metadata["ageKnown"], true);
    assert!(matches!(
        m.on_tick("SOXQ", 99, 0)[0].signal,
        Signal::Sell { quantity: 7, .. }
    ));
}
#[test]
fn strong_close_manager_consumption_survives_late_legacy_invalidation_and_failure() {
    let mut m = strong_manager();
    let t = prepare(&mut m, now());
    assert!(apply(&mut m, &t, &strong_bar(100), now()));
    m.on_tick("SOXQ", 101, 0);
    m.sync_position("SOXQ", 2, 101);
    release(&mut m);
    m.sync_position("SOXQ", 0, 0);
    assert!(m.mark_daily_history_failure(&t, "synthetic", None, now()));
    m.initialize_warmup("SOXQ", &strong_bar(9999), &[]);
    assert!(m.on_tick("SOXQ", 101, 0).is_empty());
    let t2 = prepare(&mut m, now());
    assert!(apply(&mut m, &t2, &strong_bar(100), now()));
    assert!(m.on_tick("SOXQ", 101, 0).is_empty());
}
#[test]
fn strong_close_manager_late_profile_scope_generation_do_not_publish_context_or_erase_owned() {
    let mut m = strong_manager();
    let old = prepare(&mut m, now());
    assert!(apply(&mut m, &old, &strong_bar(100), now()));
    m.on_tick("SOXQ", 101, 0);
    m.sync_position("SOXQ", 7, 101);
    release(&mut m);
    m.advance_history_generation();
    let t = prepare(&mut m, now());
    let before = m.owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ");
    let mut profile = t.clone();
    profile.profile_id = "other".into();
    let mut account = t.clone();
    account.broker_account_id = Some("other-account".into());
    for stale in [&old, &profile, &account] {
        assert!(!apply(&mut m, stale, &strong_bar(9999), now()));
        assert!(!m.mark_daily_history_failure(stale, "late", None, now()));
    }
    assert_eq!(
        m.owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ"),
        before
    );
    assert!(matches!(
        m.on_tick("SOXQ", 95, 0)[0].signal,
        Signal::Sell { quantity: 7, .. }
    ));
}
#[test]
fn strong_close_manager_trend200_context_and_held_exit_during_missing_history() {
    let mut m = strong_manager();
    m.update_config(ID, |c| {
        c.params = serde_json::json!({"long_trend_period":200})
    })
    .unwrap();
    let t = prepare(&mut m, now());
    assert_eq!(t.required_bars, 200);
    assert!(apply(&mut m, &t, &[strong_bar(100)[0]; 199], now()));
    assert!(m.on_tick("SOXQ", 101, 0).is_empty());
    m.sync_position("SOXQ", 4, 100);
    assert!(matches!(
        m.on_tick("SOXQ", 96, 0)[0].signal,
        Signal::Sell { quantity: 4, .. }
    ));
}
#[test]
fn dated_default_hook_for_other_strategy_forwards_exactly_once() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut p = UnrelatedProbe {
        config: StrategyConfig::new(
            "unrelated",
            "fixture",
            true,
            vec!["SOXQ".into()],
            1,
            serde_json::json!({}),
        ),
        initializations: count.clone(),
    };
    p.apply_completed_daily_bars_for_market_day("SOXQ", "20261022", &[]);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
#[test]
fn strong_close_manager_pending_without_checkpoint_keeps_age_unknown_after_suppressed_tick() {
    let mut m = strong_manager();
    m.update_config(ID, |c| {
        c.params = serde_json::json!({"max_hold_observations":63})
    })
    .unwrap();
    let t = prepare(&mut m, now());
    assert!(apply(&mut m, &t, &strong_bar(100), now()));
    assert!(m.note_order_pending(ID, &scope(), BrokerMarket::Us, "SOXQ", true));
    assert!(m.on_tick("SOXQ", 101, 0).is_empty());
    assert_eq!(
        m.owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ"),
        None
    );
    m.sync_position("SOXQ", 7, 100);
    assert!(m.note_order_pending(ID, &scope(), BrokerMarket::Us, "SOXQ", false));
    let p = m
        .owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ")
        .unwrap();
    assert_eq!(p["ageKnown"], false);
    assert!(
        matches!(m.on_tick("SOXQ",101,0)[0].signal,Signal::Sell{quantity:7,ref reason,..}if reason.contains("관측"))
    );
}
