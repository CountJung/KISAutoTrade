use super::*;
use crate::broker::{BrokerAccountId, BrokerId};

fn scope(account: &str) -> BrokerScope {
    BrokerScope {
        broker_id: BrokerId::Kis,
        account_id: Some(BrokerAccountId(account.to_owned())),
    }
}

fn apply(risk: &mut RiskManager, event: &str, pnl: i64) -> anyhow::Result<bool> {
    risk.record_fill_pnl_once(
        chrono::Local::now().date_naive(),
        event,
        &scope("fixture-account"),
        "strategy",
        "005930",
        pnl,
    )
}

fn roundtrip(risk: &RiskManager) -> RiskManager {
    let json = serde_json::to_string(&risk.runtime_state()).unwrap();
    let mut restored = RiskManager::new(risk.daily_loss_limit, risk.max_position_ratio);
    restored.apply_config_state(&risk.config_state());
    restored.apply_runtime_state(serde_json::from_str(&json).unwrap());
    restored
}

#[test]
fn retry_and_partial_fills_preserve_both_loss_protections() {
    let mut risk = RiskManager::new(10_000, 0.2);
    risk.max_consecutive_losses_per_strategy_symbol = 2;
    assert!(apply(&mut risk, "partial-1", -6_000).unwrap());
    assert!(!apply(&mut risk, "partial-1", -6_000).unwrap());
    assert_eq!(risk.current_loss(), -6_000);
    assert_eq!(risk.blocked_strategy_symbol_count(), 0);
    assert!(apply(&mut risk, "partial-2", -5_000).unwrap());
    assert!(!apply(&mut risk, "partial-2", -5_000).unwrap());
    assert_eq!(risk.current_loss(), -11_000);
    assert!(risk.is_emergency_stop());
    assert_eq!(risk.blocked_strategy_symbol_count(), 1);
    assert!(apply(&mut risk, "partial-3", 3_000).unwrap());
    assert!(!apply(&mut risk, "partial-3", 3_000).unwrap());
    assert_eq!(risk.daily_profit(), 3_000);
    assert_eq!(risk.blocked_strategy_symbol_count(), 0);
    assert!(risk.is_emergency_stop());
}

#[test]
fn serialized_retries_and_intent_before_risk_are_exactly_once() {
    let before_risk = RiskManager::new(10_000, 0.2);
    let mut retry_intent = roundtrip(&before_risk);
    assert!(apply(&mut retry_intent, "intent", -2_000).unwrap());
    let mut after_save = roundtrip(&retry_intent);
    assert!(!apply(&mut after_save, "intent", -2_000).unwrap());
    assert_eq!(after_save.current_loss(), -2_000);
    assert_eq!(after_save.runtime_state().processed_fill_pnl.len(), 1);
    // A failed save after mutation leaves the same in-memory receipt. A retry
    // still saves this snapshot; it must not add PnL a second time.
    assert!(!apply(&mut retry_intent, "intent", -2_000).unwrap());
    assert_eq!(retry_intent.current_loss(), -2_000);
}

#[test]
fn rebuild_replays_durable_trades_then_missing_intent() {
    let mut risk = RiskManager::new(10_000, 0.2);
    risk.max_consecutive_losses_per_strategy_symbol = 2;
    apply(&mut risk, "durable-trade", -1_000).unwrap();
    apply(&mut risk, "intent-not-in-trades", -2_000).unwrap();
    let mut rebuilt = roundtrip(&risk);
    rebuilt.reset_daily();
    assert!(apply(&mut rebuilt, "durable-trade", -1_000).unwrap());
    assert!(!apply(&mut rebuilt, "durable-trade", -1_000).unwrap());
    assert!(apply(&mut rebuilt, "intent-not-in-trades", -2_000).unwrap());
    assert_eq!(rebuilt.current_loss(), -3_000);
    assert_eq!(rebuilt.blocked_strategy_symbol_count(), 1);
}

#[test]
fn scope_is_part_of_event_identity_and_payload_conflicts_fail_closed() {
    let mut risk = RiskManager::default();
    apply(&mut risk, "same-order", -1_000).unwrap();
    assert!(apply(&mut risk, "same-order", -2_000).is_err());
    let today = chrono::Local::now().date_naive();
    assert!(risk
        .record_fill_pnl_once(
            today,
            "same-order",
            &scope("other-account"),
            "strategy",
            "005930",
            -2_000
        )
        .unwrap());
    let mut other_broker = scope("fixture-account");
    other_broker.broker_id = BrokerId::Toss;
    assert!(risk
        .record_fill_pnl_once(
            today,
            "same-order",
            &other_broker,
            "strategy",
            "005930",
            -3_000
        )
        .unwrap());
    assert!(risk
        .record_fill_pnl_once(
            today,
            "same-order",
            &scope("fixture-account"),
            "other-strategy",
            "005930",
            -1_000
        )
        .is_err());
    assert_eq!(risk.current_loss(), -6_000);
}

#[test]
fn date_reset_retires_receipts_without_replaying_historical_fills() {
    let today = chrono::Local::now().date_naive();
    let yesterday = today.pred_opt().unwrap();
    let mut risk = RiskManager::new(100, 0.2);
    assert!(risk
        .record_fill_pnl_once(
            today,
            "yesterday",
            &scope("fixture-account"),
            "strategy",
            "005930",
            -100
        )
        .unwrap());
    assert!(risk.is_emergency_stop());
    // Simulate a live process crossing midnight before its daemon reset. The
    // first fill must itself retire the prior totals and apply today's loss.
    risk.last_reset_date = Some(yesterday);
    assert!(apply(&mut risk, "today", -10).unwrap());
    assert_eq!(risk.current_loss(), -10);
    assert_eq!(risk.runtime_state().date, today);
    assert_eq!(risk.runtime_state().processed_fill_pnl.len(), 1);
    assert!(!apply(&mut risk, "today", -10).unwrap());
    assert!(!risk
        .record_fill_pnl_once(
            yesterday,
            "yesterday",
            &scope("fixture-account"),
            "strategy",
            "005930",
            -100
        )
        .unwrap());
    assert_eq!(risk.current_loss(), -10);
    assert!(risk.is_emergency_stop());
}

#[test]
fn stale_runtime_and_legacy_runtime_have_no_current_receipts() {
    let mut old = RiskManager::default();
    apply(&mut old, "old", -50).unwrap();
    old.trigger_emergency_stop();
    let mut state = old.runtime_state();
    state.date = state.date.pred_opt().unwrap();
    let mut restored = RiskManager::default();
    apply(&mut restored, "discard-me", -10).unwrap();
    restored.apply_runtime_state(state);
    assert_eq!(restored.current_loss(), 0);
    assert!(restored.runtime_state().processed_fill_pnl.is_empty());
    assert!(restored.is_emergency_stop());

    let mut legacy = serde_json::to_value(old.runtime_state()).unwrap();
    legacy.as_object_mut().unwrap().remove("processedFillPnl");
    restored.apply_runtime_state(serde_json::from_value(legacy).unwrap());
    assert!(restored.runtime_state().processed_fill_pnl.is_empty());
    assert_eq!(restored.current_loss(), -50);
}

#[test]
fn bounded_receipts_and_numeric_overflow_reject_without_partial_mutation() {
    let mut risk = RiskManager::default();
    assert!(apply(&mut risk, "invalid", i64::MIN).is_err());
    assert_eq!(risk.current_loss(), 0);
    assert!(risk.runtime_state().processed_fill_pnl.is_empty());
    apply(&mut risk, "first", -1).unwrap();
    let (key, value) = risk.processed_fill_pnl.first_key_value().unwrap();
    assert_eq!(key.len(), 64);
    assert_eq!(value.len(), 64);
    for index in 1..MAX_DAILY_FILL_EVENTS {
        risk.processed_fill_pnl
            .insert(format!("{index:064x}"), "0".repeat(64));
    }
    assert_eq!(risk.processed_fill_pnl.len(), MAX_DAILY_FILL_EVENTS);
    assert!(apply(&mut risk, "over-limit", -10).is_err());
    assert!(!apply(&mut risk, "first", -1).unwrap());
    assert_eq!(risk.current_loss(), -1);
    risk.reset_daily();
    assert!(apply(&mut risk, "after-reset", -10).unwrap());
}

#[cfg(test)]
#[path = "dependency_contract_tests.rs"]
mod dependency_contract_tests;
