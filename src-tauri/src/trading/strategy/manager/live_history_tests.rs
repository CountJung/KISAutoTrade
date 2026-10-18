use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use chrono::Duration;

use super::*;
use crate::broker::BrokerAccountId;
use crate::trading::strategy::{build_strategy, Signal, Strategy};

fn now() -> DateTime<Utc> {
    "2026-10-19T13:30:00Z".parse().unwrap()
}

fn scope() -> BrokerScope {
    BrokerScope::new(
        BrokerId::Kis,
        Some(BrokerAccountId("fixture-account".into())),
    )
}

fn manager(symbols: Vec<String>) -> StrategyManager {
    let mut manager = StrategyManager::new();
    manager.add(build_strategy(
        StrategyConfig::new(
            "fifty_two_week_high",
            "fixture",
            true,
            symbols,
            1,
            serde_json::to_value(FiftyTwoWeekHighParams::default()).unwrap(),
        )
        .with_scope(BrokerId::Kis, Some("fixture-account".into())),
    ));
    manager.set_history_execution_identity(scope(), Some("fixture-profile".into()));
    manager
}

fn bars() -> Vec<OhlcCandle> {
    vec![
        OhlcCandle {
            open: 9_000,
            high: 10_000,
            low: 8_000,
            close: 9_000
        };
        252
    ]
}

fn prepare(manager: &mut StrategyManager, at: DateTime<Utc>) -> DailyHistoryTicket {
    manager
        .prepare_daily_history_targets(&scope(), "fixture-profile", at)
        .remove(0)
}

fn apply(
    manager: &mut StrategyManager,
    ticket: &DailyHistoryTicket,
    candles: &[OhlcCandle],
    at: DateTime<Utc>,
) -> bool {
    let last = ticket.market_date.pred_opt().unwrap().to_string();
    manager.apply_daily_history(ticket, candles, "fixture".into(), Some(last), at)
}

#[test]
fn completed_252_keeps_final_high_and_uses_high_instead_of_close() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let ticket = prepare(&mut manager, now());
    let mut candles = bars();
    candles[251].high = 20_000;
    assert!(apply(&mut manager, &ticket, &candles, now()));
    let status = manager.daily_history_status().remove(0);
    assert_eq!(
        (
            status.state.as_str(),
            status.available_bars,
            status.required_bars
        ),
        ("ready", 252, 252)
    );
    assert!(manager.on_tick("SOXQ", 12_000, 0).is_empty());
    assert!(matches!(
        manager.on_tick("SOXQ", 21_000, 0)[0].signal,
        Signal::Buy { .. }
    ));
}

#[test]
fn repeated_preparation_does_not_reset_crossing_or_refill_history() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let ticket = prepare(&mut manager, now());
    assert!(apply(&mut manager, &ticket, &bars(), now()));
    assert!(matches!(
        manager.on_tick("SOXQ", 11_000, 0)[0].signal,
        Signal::Buy { .. }
    ));
    manager.sync_position("SOXQ", 0, 0);
    assert_eq!(prepare(&mut manager, now() + Duration::seconds(10)), ticket);
    assert!(manager.on_tick("SOXQ", 12_000, 0).is_empty());
    assert_eq!(manager.daily_history_status()[0].available_bars, 252);
}

#[test]
fn next_market_day_expires_old_high_and_requires_new_completed_window() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let first = prepare(&mut manager, now());
    let mut candles = bars();
    candles[0].high = 30_000;
    assert!(apply(&mut manager, &first, &candles, now()));
    assert!(manager.on_tick("SOXQ", 12_000, 0).is_empty());
    let tomorrow = now() + Duration::days(1);
    let second = prepare(&mut manager, tomorrow);
    assert_ne!(first, second);
    assert_eq!(manager.daily_history_status()[0].available_bars, 0);
    assert!(manager.on_tick("SOXQ", 31_000, 0).is_empty());
    assert!(!apply(&mut manager, &first, &candles, tomorrow));
    assert!(apply(&mut manager, &second, &bars(), tomorrow));
    assert!(matches!(
        manager.on_tick("SOXQ", 12_000, 0)[0].signal,
        Signal::Buy { .. }
    ));
}

#[test]
fn insufficient_history_and_failures_keep_held_stop_loss() {
    let mut manager = manager(vec!["SOXQ".into()]);
    manager.sync_position("SOXQ", 1, 10_000);
    let ticket = prepare(&mut manager, now());
    assert!(apply(&mut manager, &ticket, &bars()[..251], now()));
    assert_eq!(manager.daily_history_status()[0].state, "insufficient");
    assert!(manager.mark_daily_history_failure(
        &ticket,
        "fixture timeout",
        Some("2026-10-19T14:00:00Z".into()),
        now()
    ));
    assert!(matches!(
        manager.on_tick("SOXQ", 9_000, 0)[0].signal,
        Signal::Sell { .. }
    ));
    manager.sync_position("SOXQ", 0, 0);
    assert!(manager.on_tick("SOXQ", 40_000, 0).is_empty());
    assert_eq!(manager.daily_history_status()[0].state, "failed");
}

#[test]
fn run_generation_rejects_stop_start_same_scope_result() {
    let mut manager = manager(vec!["SOXQ".into()]);
    manager.sync_position("SOXQ", 1, 10_000);
    let old = prepare(&mut manager, now());
    manager.advance_history_generation();
    manager.clear_history_execution_identity();
    manager.advance_history_generation();
    manager.set_history_execution_identity(scope(), Some("fixture-profile".into()));
    let fresh = prepare(&mut manager, now());
    assert_ne!(fresh.generation, old.generation);
    assert!(!apply(&mut manager, &old, &bars(), now()));
    assert!(!manager.mark_daily_history_failure(&old, "late failure", None, now()));
    assert!(!manager.set_daily_history_retry(&old, Some("late".into())));
    assert_eq!(manager.daily_history_status()[0].state, "pending");
    assert!(matches!(
        manager.on_tick("SOXQ", 9_000, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[test]
fn config_aba_and_profile_scope_switch_cannot_revive_old_tickets() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let old = prepare(&mut manager, now());
    manager
        .update_config("fifty_two_week_high", |cfg| cfg.order_quantity = 2)
        .unwrap();
    manager
        .update_config("fifty_two_week_high", |cfg| cfg.order_quantity = 1)
        .unwrap();
    let fresh = prepare(&mut manager, now());
    assert_eq!(old.config_fingerprint, fresh.config_fingerprint);
    assert_ne!(old.generation, fresh.generation);
    assert!(!apply(&mut manager, &old, &bars(), now()));
    let current_generation = manager.history_generation();
    assert!(manager
        .prepare_daily_history_targets(&scope(), "another-profile", now())
        .is_empty());
    assert_eq!(manager.history_generation(), current_generation);
    assert!(apply(&mut manager, &fresh, &bars(), now()));
    manager
        .set_history_execution_identity(BrokerScope::kis_legacy(), Some("fixture-profile".into()));
    assert!(manager
        .prepare_daily_history_targets(&scope(), "fixture-profile", now())
        .is_empty());
}

#[test]
fn obsolete_scope_request_does_not_invalidate_new_run_history_or_held_stop() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let old_scope = scope();
    let old = prepare(&mut manager, now());
    let new_scope = BrokerScope::new(BrokerId::Kis, Some(BrokerAccountId("new-account".into())));
    let configs = manager
        .all_configs()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    manager.apply_saved_configs_for_scope(&configs, BrokerId::Kis, Some("new-account".into()));
    manager.advance_history_generation();
    manager.set_history_execution_identity(new_scope.clone(), Some("new-profile".into()));
    manager.sync_position("SOXQ", 3, 10_000);
    let current = manager
        .prepare_daily_history_targets(&new_scope, "new-profile", now())
        .remove(0);
    assert!(apply(&mut manager, &current, &bars(), now()));
    let generation = manager.history_generation();
    let before = serde_json::to_value(manager.daily_history_status()).unwrap();
    assert!(manager
        .prepare_daily_history_targets(&old_scope, "fixture-profile", now())
        .is_empty());
    assert_eq!(manager.history_generation(), generation);
    assert_eq!(
        serde_json::to_value(manager.daily_history_status()).unwrap(),
        before
    );
    assert!(!apply(&mut manager, &old, &bars(), now()));
    assert!(manager.is_daily_history_ready(&current));
    assert!(matches!(
        manager.on_tick("SOXQ", 9_000, 0)[0].signal,
        Signal::Sell { quantity: 1, .. }
    ));
}

#[test]
fn current_day_future_missing_and_stale_last_completed_dates_block_readiness() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let ticket = prepare(&mut manager, now());
    for date in [
        None,
        Some("bad-date"),
        Some("2026-10-19"),
        Some("2026-10-20"),
        Some("2026-10-08"),
    ] {
        assert!(manager.apply_daily_history(
            &ticket,
            &bars(),
            "fixture".into(),
            date.map(str::to_string),
            now()
        ));
        assert_eq!(manager.daily_history_status()[0].state, "insufficient");
        assert_eq!(manager.daily_history_status()[0].available_bars, 0);
        assert!(manager.on_tick("SOXQ", 40_000, 0).is_empty());
    }
    assert!(manager.apply_daily_history(
        &ticket,
        &bars(),
        "fixture".into(),
        Some("20261009".into()),
        now()
    ));
    assert_eq!(manager.daily_history_status()[0].state, "ready");
}

#[test]
fn midnight_during_quote_request_invalidates_history_before_tick_but_preserves_stop() {
    let before: DateTime<Utc> = "2026-10-20T03:59:59Z".parse().unwrap();
    let after = before + Duration::seconds(2);
    let mut manager = manager(vec!["SOXQ".into()]);
    let ticket = prepare(&mut manager, before);
    assert!(apply(&mut manager, &ticket, &bars(), before));
    manager.invalidate_stale_daily_history_for_symbol("SOXQ", after);
    assert_eq!(manager.daily_history_status()[0].state, "pending");
    assert!(manager.on_tick("SOXQ", 40_000, 0).is_empty());
    manager.sync_position("SOXQ", 1, 10_000);
    manager.invalidate_stale_daily_history_for_symbol("SOXQ", after);
    assert!(matches!(
        manager.on_tick("SOXQ", 9_000, 0)[0].signal,
        Signal::Sell { .. }
    ));
    assert!(!apply(&mut manager, &ticket, &bars(), after));
    let next = prepare(&mut manager, after);
    assert_eq!(next.market_date.to_string(), "2026-10-20");
}

#[test]
fn market_dates_follow_seoul_new_york_and_dst_boundaries() {
    let mut manager = manager(vec!["005930".into(), "SOXQ".into()]);
    let at: DateTime<Utc> = "2026-11-02T04:30:00Z".parse().unwrap();
    let tickets = manager.prepare_daily_history_targets(&scope(), "fixture-profile", at);
    assert_eq!(tickets[0].market_date.to_string(), "2026-11-02");
    assert_eq!(tickets[1].market_date.to_string(), "2026-11-01");
    let after = at + Duration::hours(1);
    let tickets = manager.prepare_daily_history_targets(&scope(), "fixture-profile", after);
    assert_eq!(tickets[1].market_date.to_string(), "2026-11-02");
}

struct UnrelatedProbe {
    config: StrategyConfig,
    initializations: Arc<AtomicUsize>,
}
impl Strategy for UnrelatedProbe {
    fn id(&self) -> &str {
        &self.config.id
    }
    fn name(&self) -> &str {
        &self.config.name
    }
    fn config(&self) -> &StrategyConfig {
        &self.config
    }
    fn config_mut(&mut self) -> &mut StrategyConfig {
        &mut self.config
    }
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }
    fn on_tick(&mut self, _: &str, _: u64, _: u64) -> Signal {
        Signal::Hold
    }
    fn initialize_ohlc(&mut self, _: &str, _: &[OhlcCandle]) {
        self.initializations.fetch_add(1, Ordering::SeqCst);
    }
    fn initialize_intraday_ohlc(&mut self, _: &str, _: &[OhlcCandle]) {
        self.initializations.fetch_add(1, Ordering::SeqCst);
    }
    fn reset(&mut self) {}
}

#[test]
fn daily_refresh_never_calls_unrelated_leveraged_intraday_initializers() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let initializations = Arc::new(AtomicUsize::new(0));
    manager.add(Box::new(UnrelatedProbe {
        config: StrategyConfig::new(
            "leveraged_trend_hold",
            "probe",
            true,
            vec!["SOXQ".into()],
            1,
            serde_json::json!({}),
        ),
        initializations: initializations.clone(),
    }));
    let ticket = prepare(&mut manager, now());
    assert!(apply(&mut manager, &ticket, &bars(), now()));
    manager.advance_history_generation();
    prepare(&mut manager, now() + Duration::days(1));
    assert_eq!(initializations.load(Ordering::SeqCst), 0);
}

#[test]
fn target_and_status_caps_are_explicit_and_removed_targets_are_pruned() {
    let symbols = (0..600).map(|i| format!("S{i}")).collect();
    let mut manager = manager(symbols);
    let targets = manager.prepare_daily_history_targets(&scope(), "fixture-profile", now());
    assert_eq!(targets.len(), 511);
    let status = manager.daily_history_status();
    assert_eq!(status.len(), 512);
    assert_eq!(
        status
            .iter()
            .filter(|s| s.state == "capacityExceeded")
            .count(),
        1
    );
    assert!(manager.on_tick("S599", 40_000, 0).is_empty());
    manager.update_config("fifty_two_week_high", |cfg| {
        cfg.target_symbols = vec!["SOXQ".into()]
    });
    prepare(&mut manager, now());
    assert_eq!(manager.daily_history_status().len(), 1);
}

#[test]
fn disabled_targets_and_config_mutation_remove_ready_history() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let ticket = prepare(&mut manager, now());
    assert!(apply(&mut manager, &ticket, &bars(), now()));
    manager
        .get_config_mut("fifty_two_week_high")
        .unwrap()
        .enabled = false;
    assert!(manager
        .prepare_daily_history_targets(&scope(), "fixture-profile", now())
        .is_empty());
    assert!(manager.daily_history_status().is_empty());
    assert!(!apply(&mut manager, &ticket, &bars(), now()));
}

#[test]
fn configured_512_window_and_invalid_ohlc_are_not_mislabeled_ready() {
    let mut manager = manager(vec!["SOXQ".into()]);
    manager
        .update_config("fifty_two_week_high", |config| {
            config.params["lookback_days"] = serde_json::json!(512);
        })
        .unwrap();
    let ticket = prepare(&mut manager, now());
    assert_eq!(ticket.required_bars, 512);
    let mut candles = vec![bars()[0]; 512];
    candles[511].low = 20_000;
    assert!(apply(&mut manager, &ticket, &candles, now()));
    assert_eq!(manager.daily_history_status()[0].state, "insufficient");
    assert_eq!(manager.daily_history_status()[0].available_bars, 511);
    assert!(manager.on_tick("SOXQ", 40_000, 0).is_empty());
    candles[511].low = 8_000;
    assert!(apply(&mut manager, &ticket, &candles, now()));
    assert_eq!(manager.daily_history_status()[0].state, "ready");
    assert_eq!(manager.daily_history_status()[0].available_bars, 512);
}

#[test]
fn retry_transitions_block_entries_while_retaining_last_fetch_diagnostics() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let ticket = prepare(&mut manager, now());
    assert!(apply(&mut manager, &ticket, &bars()[..251], now()));
    assert!(manager.set_daily_history_collection_reason(&ticket, "providerExhausted", now()));
    assert!(manager.set_daily_history_retry(&ticket, Some("2026-10-19T14:00:00Z".into())));
    assert!(manager.daily_history_status()[0]
        .last_error
        .as_deref()
        .unwrap()
        .contains("providerExhausted"));
    assert!(manager.mark_daily_history_pending(&ticket, now()));
    assert!(manager.daily_history_status()[0].next_retry_at.is_none());
    assert!(manager.on_tick("SOXQ", 40_000, 0).is_empty());
    assert!(apply(&mut manager, &ticket, &bars(), now()));
    assert!(manager.mark_daily_history_pending(&ticket, now()));
    assert!(manager.on_tick("SOXQ", 40_000, 0).is_empty());
}

#[test]
fn legacy_missing_profile_gets_visible_failure_without_accepting_another_profile() {
    let mut manager = manager(vec!["SOXQ".into()]);
    manager.set_history_execution_identity(scope(), None);
    let ticket = manager
        .prepare_daily_history_targets(&scope(), "", now())
        .remove(0);
    assert_eq!(ticket.profile_id, "");
    assert!(manager.mark_daily_history_failure(
        &ticket,
        "실행 프로파일을 찾을 수 없습니다.",
        None,
        now()
    ));
    assert_eq!(manager.daily_history_status()[0].state, "failed");
    // Empty-profile tickets remain diagnostic even without an unrelated request.
    assert!(!apply(&mut manager, &ticket, &bars(), now()));
    assert_eq!(manager.daily_history_status()[0].state, "failed");
    assert!(manager
        .prepare_daily_history_targets(&scope(), "unrelated-profile", now())
        .is_empty());
    assert!(!apply(&mut manager, &ticket, &bars(), now()));
}

#[test]
fn parameter_update_keeps_held_stop_with_new_parameters_and_invalidates_old_window() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let ticket = prepare(&mut manager, now());
    assert!(apply(&mut manager, &ticket, &bars(), now()));
    manager.sync_position("SOXQ", 1, 10_000);
    manager
        .update_config("fifty_two_week_high", |config| {
            config.params["lookback_days"] = serde_json::json!(100);
            config.params["stop_loss_pct"] = serde_json::json!(5.0);
        })
        .unwrap();
    assert!(!apply(&mut manager, &ticket, &bars(), now()));
    let changed = prepare(&mut manager, now());
    assert_eq!(changed.required_bars, 100);
    assert_eq!(manager.daily_history_status()[0].available_bars, 0);
    assert!(manager.on_tick("SOXQ", 9_600, 0).is_empty());
    assert!(matches!(
        manager.on_tick("SOXQ", 9_400, 0)[0].signal,
        Signal::Sell { .. }
    ));
    manager.sync_position("SOXQ", 0, 0);
    assert!(manager.on_tick("SOXQ", 40_000, 0).is_empty());
}

#[test]
fn same_scope_saved_config_rollback_keeps_held_stop_and_rejects_old_fetch() {
    let mut manager = manager(vec!["SOXQ".into()]);
    let original = manager.all_configs()[0].clone();
    let ticket = prepare(&mut manager, now());
    assert!(apply(&mut manager, &ticket, &bars(), now()));
    manager.sync_position("SOXQ", 1, 10_000);
    manager
        .update_config("fifty_two_week_high", |config| {
            config.params["stop_loss_pct"] = serde_json::json!(10.0);
        })
        .unwrap();
    manager.apply_saved_configs_for_scope(
        &[original],
        BrokerId::Kis,
        Some("fixture-account".into()),
    );
    let fresh = prepare(&mut manager, now());
    assert_ne!(ticket.generation, fresh.generation);
    assert!(!apply(&mut manager, &ticket, &bars(), now()));
    assert_eq!(manager.daily_history_status()[0].available_bars, 0);
    assert!(matches!(
        manager.on_tick("SOXQ", 9_500, 0)[0].signal,
        Signal::Sell { .. }
    ));
}

#[test]
fn scope_change_never_transfers_held_stop_to_another_account() {
    let mut manager = manager(vec!["SOXQ".into()]);
    manager.sync_position("SOXQ", 1, 10_000);
    manager
        .update_config("fifty_two_week_high", |config| {
            config.set_scope(BrokerId::Kis, Some("other-account".into()));
        })
        .unwrap();
    assert!(manager.on_tick("SOXQ", 9_000, 0).is_empty());

    let mut manager = self::manager(vec!["SOXQ".into()]);
    let original = manager.all_configs()[0].clone();
    manager.sync_position("SOXQ", 1, 10_000);
    manager.apply_saved_configs_for_scope(
        &[original],
        BrokerId::Toss,
        Some("other-account".into()),
    );
    assert!(manager.on_tick("SOXQ", 9_000, 0).is_empty());
}

#[test]
fn removed_target_does_not_reappear_after_preserving_retained_target_stop() {
    let mut manager = manager(vec!["SOXQ".into(), "QQQ".into()]);
    manager.sync_position("SOXQ", 1, 10_000);
    manager.sync_position("QQQ", 1, 10_000);
    manager
        .update_config("fifty_two_week_high", |config| {
            config.target_symbols = vec!["SOXQ".into()];
        })
        .unwrap();
    assert!(manager.on_tick("QQQ", 9_000, 0).is_empty());
    assert!(matches!(
        manager.on_tick("SOXQ", 9_000, 0)[0].signal,
        Signal::Sell { .. }
    ));
    assert_eq!(manager.active_symbols(), vec!["SOXQ"]);
}
