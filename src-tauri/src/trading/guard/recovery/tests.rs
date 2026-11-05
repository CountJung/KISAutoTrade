use super::*;

fn time(value: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(value).unwrap()
}
fn sell(reason: &str) -> Signal {
    Signal::Sell {
        symbol: "FIXTURE".into(),
        quantity: 1,
        reason: reason.into(),
    }
}
fn buy() -> Signal {
    Signal::Buy {
        symbol: "FIXTURE".into(),
        quantity: 1,
        reason: "fixture buy".into(),
    }
}
fn evaluate(guard: &mut TradeGuard, now: DateTime<FixedOffset>) -> GuardDecision {
    guard.evaluate_for_scope_at(&BrokerScope::kis_legacy(), &buy(), 0, None, 100, false, now)
}
#[test]
fn recovered_native_stop_preserves_original_day_and_next_day_sell_cooldown() {
    let mut guard = TradeGuard::default();
    let scope = BrokerScope::kis_legacy();
    let submitted = time("2026-10-29T23:45:00+09:00");
    guard
        .restore_recovered_submission_at(
            "fixture",
            &scope,
            &sell("native 손절"),
            submitted,
            time("2026-10-29T23:50:00+09:00"),
        )
        .unwrap();
    assert!(
        matches!(evaluate(&mut guard, time("2026-10-29T23:59:00+09:00")), GuardDecision::Block{reason} if reason.contains("손절 후"))
    );
    guard.reset_day();
    guard
        .restore_recovered_submission_at(
            "fixture",
            &scope,
            &sell("native 손절"),
            submitted,
            time("2026-10-30T00:10:00+09:00"),
        )
        .unwrap();
    assert!(
        matches!(evaluate(&mut guard, time("2026-10-30T00:20:00+09:00")), GuardDecision::Block{reason} if reason.contains("매도 후"))
    );
    assert!(matches!(
        evaluate(&mut guard, time("2026-10-30T00:46:00+09:00")),
        GuardDecision::Allow
    ));
}
#[test]
fn recovery_retry_does_not_extend_time_or_duplicate_whipsaw_history() {
    let mut guard = TradeGuard::default();
    let scope = BrokerScope::kis_legacy();
    let submitted = time("2026-10-29T12:00:00+09:00");
    for minute in [1, 2, 10, 59] {
        guard
            .restore_recovered_submission_at(
                "fixture",
                &scope,
                &sell("ordinary sell"),
                submitted,
                submitted + Duration::minutes(minute),
            )
            .unwrap();
    }
    assert_eq!(guard.recovered_submissions.len(), 1);
    assert_eq!(
        guard.recent_sides[&(scope.clone(), "FIXTURE".into())].len(),
        1
    );
    assert_eq!(
        guard.last_order_at[&(scope, "FIXTURE".into(), GuardSide::Sell)],
        submitted
    );
    assert!(matches!(
        evaluate(&mut guard, submitted + Duration::minutes(61)),
        GuardDecision::Allow
    ));
}
#[test]
fn recovering_older_order_preserves_newer_last_side_date_and_recent_ordering() {
    let mut guard = TradeGuard::default();
    let scope = BrokerScope::kis_legacy();
    let newer = time("2026-10-30T00:03:00+09:00");
    let older = time("2026-10-29T23:59:00+09:00");
    guard.record_submitted_for_scope_at(&scope, &sell("newer 손절"), newer);
    guard
        .restore_recovered_submission_at("fixture", &scope, &sell("older 손절"), older, newer)
        .unwrap();
    assert_eq!(
        guard.last_order_at[&(scope.clone(), "FIXTURE".into(), GuardSide::Sell)],
        newer
    );
    assert_eq!(
        guard.stop_loss_block_date[&(scope.clone(), "FIXTURE".into())],
        newer.date_naive()
    );
    let recent = &guard.recent_sides[&(scope.clone(), "FIXTURE".into())];
    assert_eq!(
        recent.iter().map(|(at, _)| *at).collect::<Vec<_>>(),
        vec![older, newer]
    );
    guard
        .restore_recovered_submission_at("fixture", &scope, &sell("older 손절"), older, newer)
        .unwrap();
    assert_eq!(guard.recent_sides[&(scope, "FIXTURE".into())].len(), 2);
}
#[test]
fn live_recovery_uses_local_calendar_and_bounds_identity_receipts() {
    let mut guard = TradeGuard::default();
    let scope = BrokerScope::kis_legacy();
    let submitted = Local::now().fixed_offset();
    for index in 0..MAX_RECOVERED_SUBMISSIONS {
        guard
            .restore_recovered_submission(
                &format!("fixture-{index}"),
                &scope,
                &sell("손절"),
                submitted,
            )
            .unwrap();
    }
    assert!(guard
        .restore_recovered_submission("overflow", &scope, &sell("손절"), submitted)
        .is_err());
    assert_eq!(guard.recovered_submissions.len(), MAX_RECOVERED_SUBMISSIONS);
    assert_eq!(
        guard.stop_loss_block_date[&(scope.clone(), "FIXTURE".into())],
        submitted.with_timezone(&Local).date_naive()
    );
    assert!(guard
        .restore_recovered_submission("fixture-0", &scope, &buy(), submitted)
        .is_err());
}
