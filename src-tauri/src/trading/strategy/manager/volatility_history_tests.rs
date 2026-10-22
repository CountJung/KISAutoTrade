//! Verified-day coordinator fixtures without provider or account access.
use super::*;
const ID: &str = "volatility_expansion_live_fixture";
fn volatility_manager(extra: serde_json::Value) -> StrategyManager {
    let mut p = serde_json::json!({"lookback_days":2,"expansion_factor":2,"stop_loss_pct":3});
    p.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let mut m = StrategyManager::new();
    m.add(build_strategy(
        StrategyConfig::new(ID, "fixture", true, vec!["SOXQ".into()], 100, p)
            .with_scope(BrokerId::Kis, Some("fixture-account".into())),
    ));
    m.set_history_execution_identity(scope(), Some("fixture-profile".into()));
    m
}
fn ranges() -> [OhlcCandle; 2] {
    [OhlcCandle {
        open: 100,
        high: 110,
        low: 100,
        close: 105,
    }; 2]
}
fn release(m: &mut StrategyManager) {
    m.complete_signal(ID, "SOXQ", m.history_generation());
}
#[test]
fn volatility_expansion_manager_same_day_refresh_failure_and_legacy_never_reset_range() {
    let mut m = volatility_manager(serde_json::json!({}));
    let t = prepare(&mut m, now());
    assert_eq!(t.required_bars, 2);
    assert!(m.on_tick("SOXQ", 1000, 0).is_empty());
    assert!(apply(&mut m, &t, &ranges(), now()));
    assert!(m.on_tick("SOXQ", 100, 0).is_empty());
    assert_eq!(m.on_tick("SOXQ", 125, 0).len(), 1);
    release(&mut m);
    assert!(apply(&mut m, &t, &ranges(), now()));
    assert_eq!(m.on_tick("SOXQ", 126, 0).len(), 1);
    release(&mut m);
    assert!(m.mark_daily_history_failure(&t, "synthetic", None, now()));
    assert!(m.on_tick("SOXQ", 999, 0).is_empty());
    m.initialize_warmup("SOXQ", &ranges(), &[]);
    assert!(m.on_tick("SOXQ", 998, 0).is_empty());
    let fresh = prepare(&mut m, now());
    assert!(apply(&mut m, &fresh, &ranges(), now()));
    assert_eq!(m.on_tick("SOXQ", 127, 0).len(), 1);
    release(&mut m);
    let tomorrow = now() + Duration::days(1);
    let next = prepare(&mut m, tomorrow);
    assert!(!apply(&mut m, &t, &ranges(), tomorrow));
    assert!(apply(&mut m, &next, &ranges(), tomorrow));
    assert!(m.on_tick("SOXQ", 200, 0).is_empty());
    assert_eq!(m.on_tick("SOXQ", 225, 0).len(), 1);
}
#[test]
fn volatility_expansion_manager_unknown_pending_never_gains_entry_age_from_suppressed_tick() {
    let mut m = volatility_manager(serde_json::json!({"max_hold_observations":3}));
    let t = prepare(&mut m, now());
    assert!(apply(&mut m, &t, &ranges(), now()));
    assert!(m.on_tick("SOXQ", 100, 0).is_empty());
    assert!(m.note_order_pending(ID, &scope(), BrokerMarket::Us, "SOXQ", true));
    assert!(m.on_tick("SOXQ", 125, 0).is_empty());
    assert!(m
        .owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ")
        .is_none());
    m.sync_position("SOXQ", 7, 125);
    assert!(m.note_order_pending(ID, &scope(), BrokerMarket::Us, "SOXQ", false));
    let p = m
        .owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ")
        .unwrap();
    assert_eq!(p["ageKnown"], false);
    assert!(matches!(
        m.on_tick("SOXQ", 125, 0)[0].signal,
        Signal::Sell { quantity: 7, .. }
    ));
}
#[test]
fn volatility_expansion_manager_pending_proof_survives_failure_and_actual_fill() {
    let mut m = volatility_manager(serde_json::json!({}));
    let t = prepare(&mut m, now());
    assert!(apply(&mut m, &t, &ranges(), now()));
    m.on_tick("SOXQ", 100, 0);
    assert_eq!(m.on_tick("SOXQ", 125, 0).len(), 1);
    assert!(m.note_order_pending(ID, &scope(), BrokerMarket::Us, "SOXQ", true));
    let p = m
        .owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ")
        .unwrap();
    assert_eq!(p["candidateEntry"], true);
    assert!(m.mark_daily_history_failure(&t, "synthetic", None, now()));
    m.initialize_warmup("SOXQ", &ranges(), &[]);
    let t = prepare(&mut m, now());
    assert!(apply(&mut m, &t, &ranges(), now()));
    assert!(m.on_tick("SOXQ", 126, 0).is_empty());
    assert_eq!(
        m.owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ"),
        Some(p)
    );
    m.sync_position("SOXQ", 7, 125);
    release(&mut m);
    assert!(m.note_order_pending(ID, &scope(), BrokerMarket::Us, "SOXQ", false));
    let p = m
        .owned_position_metadata_for_strategy(ID, &scope(), BrokerMarket::Us, "SOXQ")
        .unwrap();
    assert_eq!(p["ageKnown"], true);
    assert_eq!(p["heldObservations"], 0);
    let tomorrow = now() + Duration::days(1);
    let _ = prepare(&mut m, tomorrow);
    assert!(matches!(
        m.on_tick("SOXQ", 110, 0)[0].signal,
        Signal::Sell { quantity: 7, .. }
    ));
}
#[test]
fn volatility_expansion_manager_scope_generation_and_short_history_preserve_owned_exits() {
    let mut m = volatility_manager(serde_json::json!({}));
    let old = prepare(&mut m, now());
    assert!(apply(&mut m, &old, &ranges(), now()));
    m.sync_position("SOXQ", 7, 125);
    m.advance_history_generation();
    let t = prepare(&mut m, now());
    let mut profile = t.clone();
    profile.profile_id = "other".into();
    let mut account = t.clone();
    account.broker_account_id = Some("other".into());
    for bad in [&old, &profile, &account] {
        assert!(!apply(&mut m, bad, &ranges(), now()));
        assert!(!m.mark_daily_history_failure(bad, "late", None, now()));
    }
    assert!(apply(&mut m, &t, &ranges()[..1], now()));
    assert!(matches!(
        m.on_tick("SOXQ", 110, 0)[0].signal,
        Signal::Sell { quantity: 7, .. }
    ));
}
