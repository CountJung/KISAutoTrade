use super::*;
use crate::trading::order::{
    ownership::{OwnershipKey, OwnershipOwner},
    test_support::{self, Fixture},
};

async fn setup() -> (Fixture, BrokerScope, ApproveBudgetRecoveryInput) {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let scope = test_support::scope(&f.profile);
    let mut m = f.manager.lock().await;
    m.set_execution_scope(scope.clone());
    m.update_auto_trading_budget(&scope, 2020, 0).await.unwrap();
    let id = m
        .reserve_budget(&scope, "005930", false, true, 10, 100)
        .await
        .unwrap();
    let ownership = m.load_ownership_ledger().await.unwrap();
    let mut record = OrderRecord::new(
        "005930".into(),
        "fixture".into(),
        OrderSide::Buy,
        10,
        100,
        "Limit".into(),
    )
    .with_strategy_id(Some("fixture-strategy".into()))
    .with_broker_scope(&scope)
    .with_provider_trace("kis", None, None, Some("VTTC0802U".into()));
    record.id = id.clone();
    let pending = PendingOrder {
        record,
        signal_reason: "fixture".into(),
        strategy_id: Some("fixture-strategy".into()),
        signal_price: 100,
        order_price: 100,
        exchange: None,
        broker_scope: scope.clone(),
        ownership_key: Some(OwnershipKey {
            identity: m.recovery_identity(&scope).await.unwrap(),
            owner: OwnershipOwner::Strategy("fixture-strategy".into()),
            market: crate::broker::BrokerMarket::Kr,
            symbol: "005930".into(),
        }),
        ownership_generation: Some(ownership.generation().to_string()),
        strategy_position_metadata: None,
        submitted_cost_basis: None,
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
    };
    m.capture_budget_recovery(pending).await.unwrap();
    let view = m.budget_recovery(&scope).await.unwrap();
    let input = ApproveBudgetRecoveryInput {
        broker_id: scope.broker_id,
        broker_account_id: scope.account_id.as_ref().unwrap().0.clone(),
        reservation_id: id,
        snapshot_revision: view.items[0].snapshot_revision.clone().unwrap(),
        provider_order_id: "fixture-provider-order".into(),
        client_order_id: None,
        evidence_reference: "operator reviewed synthetic provider statement row 7".into(),
        confirmed: true,
    };
    drop(m);
    (f, scope, input)
}
async fn restarted(m: &OrderManager, scope: &BrokerScope) -> OrderManager {
    let mut n = OrderManager::new(
        m.rest_client.clone(),
        m.profiles.clone(),
        m.order_store.clone(),
        m.pending_order_store.clone(),
        m.trade_store.clone(),
        m.position_tracker.clone(),
        m.overseas_position_tracker.clone(),
        m.stats_store.clone(),
        m.exchange_rate_krw.clone(),
        m.risk_manager.clone(),
        m.risk_store.clone(),
        None,
    );
    n.set_execution_scope(scope.clone());
    n.restore_pending_orders(n.pending_order_store.load().await.unwrap());
    n
}
#[tokio::test]
async fn recovery_binding_restart_partial_reverse_terminal_and_reapproval_are_idempotent() {
    let (f, scope, mut input) = setup().await;
    let mut m = f.manager.lock().await;
    let before = m.auto_trading_budget(&scope).await.unwrap();
    assert_eq!(before.krw.reserved_amount, 1010);
    let view = m.approve_budget_recovery(&scope, &input).await.unwrap();
    assert_eq!(view.items[0].state, "awaitingReconciliation");
    assert_eq!(
        view.items[0].provider_order_id.as_deref(),
        Some(input.provider_order_id.as_str())
    );
    let original_owner = m.pending[&input.provider_order_id].ownership_key.clone();
    let original_generation = m.pending[&input.provider_order_id]
        .ownership_generation
        .clone();
    assert_eq!(
        m.pending[&input.provider_order_id].record.id,
        input.reservation_id
    );
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    assert_eq!(m.pending.len(), 1);
    input.provider_order_id.push_str("-different");
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    input.provider_order_id = "fixture-provider-order".into();
    let mut m = restarted(&m, &scope).await;
    assert_eq!(
        m.pending[&input.provider_order_id].ownership_key,
        original_owner
    );
    assert_eq!(
        m.pending[&input.provider_order_id].ownership_generation,
        original_generation
    );
    let date = chrono::NaiveDate::from_ymd_opt(2026, 10, 29).unwrap();
    for (qty, amount) in [(4, 400), (2, 200), (4, 400)] {
        m.on_fill_confirmed(&input.provider_order_id, qty, 100, Some(amount), date)
            .await
            .unwrap();
    }
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    assert_eq!(m.pending[&input.provider_order_id].filled_quantity, 4);
    assert_eq!(m.budget_owned(&scope, "005930", false).await.unwrap(), 4);
    assert_eq!(
        m.auto_trading_budget(&scope).await.unwrap().krw.cash_amount,
        1616
    );
    m.finalize_pending_order(
        &input.provider_order_id,
        OrderStatus::Cancelled,
        "cancelled",
        None,
    )
    .await
    .unwrap();
    assert!(m.pending.is_empty());
    let mut m = restarted(&m, &scope).await;
    let view = m.approve_budget_recovery(&scope, &input).await.unwrap();
    assert_eq!(view.items[0].state, "resolved");
    assert_eq!(view.items[0].filled_quantity, 4);
    assert!(m.pending.is_empty());
    let budget = m.auto_trading_budget(&scope).await.unwrap();
    assert_eq!(budget.krw.cash_amount, 1616);
    assert_eq!(budget.krw.reserved_amount, 0);
}
#[tokio::test]
async fn recovery_intent_survives_pending_write_failure_and_does_not_clear_block() {
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    let path = f.path.join("orders/pending_orders.json");
    tokio::fs::create_dir(&path).await.unwrap();
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    assert!(m.persistence_blocked);
    let ledger: BudgetLedger = serde_json::from_slice(
        &tokio::fs::read(m.pending_order_store.budget_path())
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(ledger.recovery_receipts.contains_key(&input.reservation_id));
    tokio::fs::remove_dir(&path).await.unwrap();
    // Crash after intent: no pending exists on disk, yet the original approval can finish.
    let mut n = restarted(&m, &scope).await;
    assert!(n.pending.is_empty());
    let view = n.budget_recovery(&scope).await.unwrap();
    assert_eq!(
        view.items[0].evidence_reference.as_deref(),
        Some(input.evidence_reference.as_str())
    );
    assert_eq!(
        view.items[0].provider_order_id.as_deref(),
        Some(input.provider_order_id.as_str())
    );
    n.approve_budget_recovery(&scope, &input).await.unwrap();
    assert_eq!(n.pending_order_store.load().await.unwrap().len(), 1);
    // Same-process retry also persists, while preserving the independent fault latch.
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    assert!(m.persistence_blocked);
}
#[tokio::test]
async fn recovery_rejects_stale_revision_identity_conflict_and_missing_evidence() {
    let (f, scope, mut input) = setup().await;
    let mut m = f.manager.lock().await;
    let original = input.snapshot_revision.clone();
    input.snapshot_revision = "stale".into();
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    input.snapshot_revision = original;
    input.confirmed = false;
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    input.confirmed = true;
    input.client_order_id = Some("invented-client".into());
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    input.client_order_id = None;
    m.submitting
        .insert((scope.clone(), "005930".into()), OrderSide::Buy);
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    m.submitting.clear();
    m.modifying.insert("other".into());
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    m.modifying.clear();
    let original_profile = m.profiles.read().await.clone();
    let mut wrong = f.profile.clone();
    wrong.id = "other-profile".into();
    *m.profiles.write().await = ProfilesConfig {
        active_id: Some(wrong.id.clone()),
        profiles: vec![wrong],
    };
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    assert!(m.budget_recovery(&scope).await.unwrap().items[0]
        .blocked_reason
        .is_some());
    *m.profiles.write().await = original_profile;
    assert!(m.pending.is_empty());
    assert_eq!(
        m.auto_trading_budget(&scope)
            .await
            .unwrap()
            .krw
            .reserved_amount,
        1010
    );
}
#[tokio::test]
async fn recovery_never_invents_legacy_or_partially_applied_pending() {
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    m.load_budget().await.unwrap();
    let mut ledger = m.budget.clone().unwrap();
    let original = ledger.currency(&scope, false).reservations[&input.reservation_id].clone();
    ledger
        .currency(&scope, false)
        .reservations
        .get_mut(&input.reservation_id)
        .unwrap()
        .recovery_snapshot = None;
    m.save_budget(ledger).await.unwrap();
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    assert!(m.budget_recovery(&scope).await.unwrap().items[0]
        .blocked_reason
        .is_some());
    let mut ledger = m.budget.clone().unwrap();
    ledger
        .currency(&scope, false)
        .reservations
        .insert(input.reservation_id.clone(), original);
    m.save_budget(ledger).await.unwrap();
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    m.on_fill_confirmed(
        &input.provider_order_id,
        4,
        100,
        Some(400),
        chrono::Local::now().date_naive(),
    )
    .await
    .unwrap();
    m.pending.clear();
    m.symbol_to_odno.clear();
    m.persist_pending_orders().await.unwrap();
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    assert_eq!(
        m.auto_trading_budget(&scope).await.unwrap().krw.cash_amount,
        1616
    );
    assert!(m.budget_recovery(&scope).await.unwrap().items[0]
        .blocked_reason
        .as_deref()
        .unwrap()
        .contains("일부 체결"));
}
#[tokio::test]
async fn recovery_view_and_receipts_are_bounded() {
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    m.load_budget().await.unwrap();
    let mut ledger = m.budget.clone().unwrap();
    let receipt = ledger
        .recovery_receipts
        .remove(&input.reservation_id)
        .unwrap();
    for i in 0..MAX_RECEIPTS {
        let id = format!("fixture-{i}");
        let mut receipt = receipt.clone();
        receipt.snapshot.pending.record.id = id.clone();
        ledger.recovery_receipts.insert(id, receipt);
    }
    m.pending.clear();
    m.symbol_to_odno.clear();
    m.trade_guard = TradeGuard::default();
    m.save_budget(ledger).await.unwrap();
    let view = m.budget_recovery(&scope).await.unwrap();
    assert_eq!(view.items.len(), PAGE_SIZE);
    assert_eq!(view.total_count, MAX_RECEIPTS + 1);
    assert!(m
        .approve_budget_recovery(&scope, &input)
        .await
        .err()
        .unwrap()
        .to_string()
        .contains("한도"));
}

#[tokio::test]
async fn recovery_intent_write_failure_never_publishes_pending() {
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    let backup = f.path.join("orders/auto_trading_budget.json.bak");
    tokio::fs::remove_file(&backup).await.unwrap();
    tokio::fs::create_dir(&backup).await.unwrap();
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    assert!(m.pending.is_empty());
    assert!(m.persistence_blocked);
    let ledger: BudgetLedger = serde_json::from_slice(
        &tokio::fs::read(m.pending_order_store.budget_path())
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(ledger.recovery_receipts.is_empty());
    tokio::fs::remove_dir(&backup).await.unwrap();
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    assert!(m.persistence_blocked);
}

#[tokio::test]
async fn recovery_terminal_receipt_survives_pending_removal_failure() {
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    let path = f.path.join("orders/pending_orders.json");
    tokio::fs::remove_file(&path).await.unwrap();
    tokio::fs::create_dir(&path).await.unwrap();
    assert!(m
        .finalize_pending_order(
            &input.provider_order_id,
            OrderStatus::Expired,
            "expired",
            None
        )
        .await
        .is_err());
    tokio::fs::remove_dir(&path).await.unwrap();
    let mut n = restarted(&m, &scope).await;
    assert!(n.pending.is_empty());
    let view = n.approve_budget_recovery(&scope, &input).await.unwrap();
    assert_eq!(view.items[0].state, "resolved");
    assert!(n.pending.is_empty());
    assert_eq!(
        n.auto_trading_budget(&scope).await.unwrap().krw.cash_amount,
        2020
    );
}

#[tokio::test]
async fn automatic_unknown_submission_has_durable_snapshot_before_provider_receives_request() {
    async fn unknown(
        axum::extract::State(path): axum::extract::State<std::path::PathBuf>,
    ) -> axum::http::StatusCode {
        let ledger: BudgetLedger =
            serde_json::from_slice(&tokio::fs::read(path).await.unwrap()).unwrap();
        let reservation = ledger.accounts[0].krw.reservations.values().next().unwrap();
        assert!(reservation.recovery_snapshot.is_some());
        assert_eq!(reservation.reserved, 101);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let f = Fixture::new(format!("http://{}", listener.local_addr().unwrap())).await;
    let context = f.begin(1).await;
    let scope = context.broker_scope.clone();
    let path = {
        let mut m = f.manager.lock().await;
        m.update_auto_trading_budget(&scope, 2020, 0).await.unwrap();
        m.pending_order_store.budget_path()
    };
    let router = axum::Router::new()
        .route(
            "/uapi/domestic-stock/v1/trading/order-cash",
            axum::routing::post(unknown),
        )
        .with_state(path);
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let outcome = OrderManager::submit_signal_shared(
        &f.manager,
        context,
        crate::trading::strategy::StrategySignal {
            strategy_id: "fixture-strategy".into(),
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
    .await;
    assert!(outcome.is_err());
    let mut m = f.manager.lock().await;
    m.end_execution_run();
    let view = m.budget_recovery(&scope).await.unwrap();
    assert_eq!(view.items.len(), 1);
    assert!(view.items[0].snapshot_revision.is_some());
    assert!(view.items[0].blocked_reason.is_none());
    assert_eq!(view.items[0].state, "unresolved");
    assert!(m.pending.is_empty());
    assert_eq!(
        m.auto_trading_budget(&scope)
            .await
            .unwrap()
            .krw
            .reserved_amount,
        101
    );
    server.abort();
}

#[tokio::test]
async fn recovery_rejects_provider_id_already_used_in_old_order_or_trade_history() {
    use crate::storage::trade_store::{TradeRecord, TradeSide};
    for source in ["order", "trade"] {
        let (f, scope, input) = setup().await;
        let mut m = f.manager.lock().await;
        let old_day = chrono::NaiveDate::from_ymd_opt(2001, 1, 2).unwrap();
        if source == "order" {
            let mut record = OrderRecord::new(
                "OTHER".into(),
                "fixture".into(),
                OrderSide::Sell,
                1,
                100,
                "Limit".into(),
            )
            .with_broker_scope(&scope)
            .with_provider_trace(
                "kis",
                Some(input.provider_order_id.clone()),
                None,
                Some("TTTC0801U".into()),
            );
            record.status = OrderStatus::Filled;
            m.order_store.append_on(old_day, record).await.unwrap();
        } else {
            let record = TradeRecord::new(
                "OTHER".into(),
                "fixture".into(),
                TradeSide::Sell,
                1,
                100,
                0,
                input.provider_order_id.clone(),
                None,
                "fixture".into(),
            )
            .with_broker_scope(&scope);
            m.trade_store.append_on(old_day, record).await.unwrap();
        }
        assert!(m
            .approve_budget_recovery(&scope, &input)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("다른 로컬"));
        assert!(m.pending.is_empty());
        assert_eq!(
            m.auto_trading_budget(&scope)
                .await
                .unwrap()
                .krw
                .reserved_amount,
            1010
        );
    }
}

#[tokio::test]
async fn recovery_history_corrupt_primary_cannot_fall_back_to_empty_backup() {
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    let date = chrono::NaiveDate::from_ymd_opt(2001, 1, 2).unwrap();
    let path = crate::storage::build_daily_path(&f.path, "orders", date, "orders.json");
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&path, b"{broken primary").await.unwrap();
    tokio::fs::write(path.with_extension("json.bak"), b"[]")
        .await
        .unwrap();
    assert!(m.approve_budget_recovery(&scope, &input).await.is_err());
    assert!(m.pending.is_empty());
    assert_eq!(tokio::fs::read(&path).await.unwrap(), b"{broken primary");
}

#[tokio::test]
async fn recovery_list_hides_normal_pending_and_prioritizes_unresolved_over_completed_receipts() {
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    let mut p = m.budget.as_ref().unwrap().accounts[0].krw.reservations[&input.reservation_id]
        .recovery_snapshot
        .as_ref()
        .unwrap()
        .pending
        .clone();
    p.record.provider_order_id = Some(input.provider_order_id.clone());
    p.record.kis_order_id = Some(input.provider_order_id.clone());
    m.track_pending_order(input.provider_order_id.clone(), p);
    assert_eq!(m.budget_recovery(&scope).await.unwrap().total_count, 0);
    m.pending.clear();
    m.symbol_to_odno.clear();
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    m.load_budget().await.unwrap();
    let mut ledger = m.budget.clone().unwrap();
    let mut receipt = ledger
        .recovery_receipts
        .remove(&input.reservation_id)
        .unwrap();
    receipt.resolved = true;
    for i in 0..PAGE_SIZE {
        let id = format!("a-completed-{i}");
        let mut receipt = receipt.clone();
        receipt.snapshot.pending.record.id = id.clone();
        ledger.recovery_receipts.insert(id, receipt);
    }
    m.pending.clear();
    m.symbol_to_odno.clear();
    m.trade_guard = TradeGuard::default();
    m.save_budget(ledger).await.unwrap();
    let view = m.budget_recovery(&scope).await.unwrap();
    assert_eq!(view.total_count, PAGE_SIZE + 1);
    assert_eq!(view.items[0].reservation_id, input.reservation_id);
    assert_eq!(view.items[0].state, "unresolved");
}

#[tokio::test]
async fn recovery_oversized_retained_history_fails_closed_before_binding() {
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    let date = chrono::NaiveDate::from_ymd_opt(2001, 1, 2).unwrap();
    let path = crate::storage::build_daily_path(&f.path, "trades", date, "trades.json");
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .unwrap();
    let file = tokio::fs::File::create(path).await.unwrap();
    file.set_len(16 * 1024 * 1024 + 1).await.unwrap();
    assert!(m
        .approve_budget_recovery(&scope, &input)
        .await
        .err()
        .unwrap()
        .to_string()
        .contains("16MiB"));
    assert!(m.pending.is_empty());
    assert_eq!(
        m.auto_trading_budget(&scope)
            .await
            .unwrap()
            .krw
            .reserved_amount,
        1010
    );
}

#[tokio::test]
async fn recovery_missing_history_primary_with_backup_or_quarantine_fails_closed() {
    for suffix in ["bak", "corrupt-fixture"] {
        let (f, scope, input) = setup().await;
        let mut m = f.manager.lock().await;
        let date = chrono::NaiveDate::from_ymd_opt(2001, 1, 2).unwrap();
        let primary = crate::storage::build_daily_path(&f.path, "orders", date, "orders.json");
        tokio::fs::create_dir_all(primary.parent().unwrap())
            .await
            .unwrap();
        let artifact = primary.with_file_name(format!("orders.json.{suffix}"));
        tokio::fs::write(
            &artifact,
            b"retained artifact must never be parsed or restored",
        )
        .await
        .unwrap();
        assert!(m
            .approve_budget_recovery(&scope, &input)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("primary가 유실"));
        assert!(m.pending.is_empty());
        assert!(!primary.exists());
        assert_eq!(
            tokio::fs::read(&artifact).await.unwrap(),
            b"retained artifact must never be parsed or restored"
        );
    }
}

#[tokio::test]
async fn recovered_native_stop_guard_survives_full_fill_flat_restart_and_budget_load() {
    use crate::trading::order::ownership::OwnershipFill;
    use chrono::TimeZone;
    let (f, scope, input) = setup().await;
    let mut m = f.manager.lock().await;
    let submitted = chrono::Local
        .from_local_datetime(
            &chrono::Local::now()
                .date_naive()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
        )
        .single()
        .unwrap();
    let mut ledger = m.budget.clone().unwrap();
    let c = ledger.currency(&scope, false);
    c.cash = 1010;
    c.positions.insert("005930".into(), 10);
    let r = c.reservations.get_mut(&input.reservation_id).unwrap();
    r.buy = false;
    r.reserved = 0;
    let p = &mut r.recovery_snapshot.as_mut().unwrap().pending;
    p.record.side = OrderSide::Sell;
    p.record.provider_tr_id = Some("VTTC0801U".into());
    p.record.timestamp = submitted.to_rfc3339();
    p.signal_reason = "native 손절 매도 fixture".into();
    let mut owner = m.load_ownership_ledger().await.unwrap();
    owner
        .apply_cumulative(&OwnershipFill {
            order_record_id: "fixture-seed".into(),
            key: p.ownership_key.clone().unwrap(),
            side: OrderSide::Buy,
            ordered_quantity: 10,
            cumulative_quantity: Some(10),
            cumulative_notional: Some(1000),
        })
        .unwrap();
    m.save_ownership_ledger(&owner).await.unwrap();
    m.save_budget(ledger).await.unwrap();
    m.approve_budget_recovery(&scope, &input).await.unwrap();
    let buy = Signal::Buy {
        symbol: "005930".into(),
        quantity: 1,
        reason: "fixture buy".into(),
    };
    assert!(
        matches!(m.trade_guard.evaluate_for_scope_at(&scope,&buy,0,None,100,false,submitted+chrono::Duration::hours(2)), GuardDecision::Block{reason} if reason.contains("손절 후"))
    );
    m.on_fill_confirmed(
        &input.provider_order_id,
        10,
        100,
        Some(1000),
        submitted.date_naive(),
    )
    .await
    .unwrap();
    assert!(m.pending.is_empty());
    assert_eq!(m.budget_owned(&scope, "005930", false).await.unwrap(), 0);
    assert_eq!(
        m.budget_recovery(&scope).await.unwrap().items[0].state,
        "resolved"
    );
    let mut n = restarted(&m, &scope).await;
    assert!(matches!(
        n.trade_guard.evaluate_for_scope_at(
            &scope,
            &buy,
            0,
            None,
            100,
            false,
            submitted + chrono::Duration::hours(2)
        ),
        GuardDecision::Allow
    ));
    n.budget_owned(&scope, "005930", false).await.unwrap();
    assert!(
        matches!(n.trade_guard.evaluate_for_scope_at(&scope,&buy,0,None,100,false,submitted+chrono::Duration::hours(2)),GuardDecision::Block{reason} if reason.contains("손절 후"))
    );
    n.reset_day();
    n.budget_owned(&scope, "005930", false).await.unwrap();
    assert!(matches!(
        n.trade_guard.evaluate_for_scope_at(
            &scope,
            &buy,
            0,
            None,
            100,
            false,
            submitted + chrono::Duration::days(1)
        ),
        GuardDecision::Allow
    ));
    // Fresh runtime must load the resolved receipt before its first submission guard.
    let mut fresh = restarted(&m, &scope).await;
    let context = fresh
        .begin_execution_run(
            scope.clone(),
            Some(f.profile.id.clone()),
            1,
            f.client.clone(),
            Some(f.profile.clone()),
        )
        .unwrap();
    let fresh = Arc::new(Mutex::new(fresh));
    let outcome = OrderManager::submit_signal_shared(
        &fresh,
        context,
        crate::trading::strategy::StrategySignal {
            strategy_id: "fixture-strategy".into(),
            signal: buy,
        },
        "fixture".into(),
        100_000,
        None,
        100,
    )
    .await
    .unwrap();
    assert!(
        matches!(outcome,crate::trading::order::SubmissionOutcome::Skipped{reason,..} if reason.contains("손절 후"))
    );
}
