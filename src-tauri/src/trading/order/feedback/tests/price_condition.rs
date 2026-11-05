//! Production PC signal -> loopback submission -> confirmed fill -> ownership feedback.
//! Synthetic credentials, isolated temporary stores and localhost only.
use super::*;
use crate::trading::{
    guard::GuardDecision,
    order::{
        holdings::{HoldingsMarkets, HoldingsUpdate},
        submission::SubmissionOutcome,
        ExecutionRunContext,
    },
    strategy::StrategySignal,
};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Default)]
struct Provider {
    reject: Arc<AtomicBool>,
    requests: Arc<Mutex<Vec<serde_json::Value>>>,
}

async fn order(
    axum::extract::State(provider): axum::extract::State<Provider>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let mut requests = provider.requests.lock().await;
    requests.push(body);
    if provider.reject.load(Ordering::SeqCst) {
        axum::Json(serde_json::json!({"rt_cd":"1","msg_cd":"FIXTURE_REJECT",
            "msg1":"synthetic definitive rejection"}))
    } else {
        axum::Json(serde_json::json!({"rt_cd":"0","msg1":"synthetic accepted",
            "output":{"odno":format!("pc-order-{}",requests.len()),"ord_tmd":"120000"}}))
    }
}

struct PcLive {
    fixture: Fixture,
    strategies: Arc<Mutex<StrategyManager>>,
    run: ExecutionRunContext,
    provider: Provider,
    server: tokio::task::JoinHandle<()>,
}

impl PcLive {
    async fn new(quantity: u64) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let provider = Provider::default();
        let router = axum::Router::new()
            .route(
                "/uapi/domestic-stock/v1/trading/order-cash",
                axum::routing::post(order),
            )
            .with_state(provider.clone());
        let fixture = Fixture::new(format!("http://{}", listener.local_addr().unwrap())).await;
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let run = fixture.begin(0).await;
        fixture
            .manager
            .lock()
            .await
            .update_auto_trading_budget(&run.broker_scope, 2_000_000, 0)
            .await
            .unwrap();
        let mut cfg = pc_config("price_condition_live_contract", &fixture.profile.account_no);
        cfg.params["symbols"][0]["quantity"] = quantity.into();
        cfg.params["symbols"][0]["buy_trigger_price"] = 101_000.into();
        cfg.params["symbols"][0]["take_profit_pct"] = 0.into();
        let strategies = manager(&fixture, vec![cfg]);
        Self {
            fixture,
            strategies,
            run,
            provider,
            server,
        }
    }

    async fn sync(&self) {
        OrderManager::sync_strategy_ownership_shared(&self.fixture.manager, &self.strategies)
            .await
            .unwrap();
    }

    async fn publish(&self, quantity: u64) {
        let mut orders = self.fixture.manager.lock().await;
        let query = orders
            .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Domestic)
            .await
            .unwrap();
        orders
            .publish_holdings(
                query,
                HoldingsUpdate::Domestic(if quantity == 0 {
                    vec![]
                } else {
                    // Account-wide average deliberately differs from the exact owner's cost.
                    vec![("005930".into(), "fixture".into(), quantity, 400_000, 95_000)]
                }),
            )
            .await
            .unwrap();
    }

    async fn raw(&self, price: u64) -> StrategySignal {
        let mut strategies = self.strategies.lock().await;
        let mut signals = strategies.on_tick("005930", price, 0);
        assert_eq!(
            signals.len(),
            1,
            "expected one production PC candidate at {price}"
        );
        signals.remove(0)
    }

    async fn submit(
        &self,
        signal: StrategySignal,
        price: u64,
    ) -> anyhow::Result<SubmissionOutcome> {
        let result = OrderManager::submit_signal_shared(
            &self.fixture.manager,
            self.run.clone(),
            signal.clone(),
            "synthetic PC".into(),
            100_000_000,
            None,
            price,
        )
        .await;
        // Same signal-completion fence used by the live daemon, before feedback.
        let mut strategies = self.strategies.lock().await;
        let revision = strategies.history_generation();
        strategies.complete_signal(&signal.strategy_id, "005930", revision);
        result
    }

    async fn fill(&self, id: &str, quantity: u64, consideration: u128) {
        self.fixture
            .manager
            .lock()
            .await
            .on_fill_confirmed(
                id,
                quantity,
                u64::try_from(consideration / u128::from(quantity)).unwrap(),
                Some(consideration),
                chrono::Local::now().date_naive(),
            )
            .await
            .unwrap();
        self.sync().await;
    }

    async fn confirmed(&self) -> (u64, u128) {
        let mut orders = self.fixture.manager.lock().await;
        let ledger = orders.load_ownership_ledger().await.unwrap();
        let position = ledger.position(&key(
            &self.fixture,
            "price_condition_live_contract",
            BrokerMarket::Kr,
            "005930",
        ));
        (position.quantity, position.cost_notional)
    }

    async fn full_buy(&self, quantity: u64) {
        let signal = self.raw(101_000).await;
        let SubmissionOutcome::Submitted { provider_order_id } =
            self.submit(signal, 101_000).await.unwrap()
        else {
            panic!("fixture buy must be accepted")
        };
        self.fill(&provider_order_id, quantity, u128::from(quantity) * 100_000)
            .await;
        self.publish(quantity + 10).await;
    }
}

impl Drop for PcLive {
    fn drop(&mut self) {
        self.server.abort();
    }
}

#[tokio::test]
async fn pc_live_rejected_buy_and_accepted_zero_fill_never_become_owned() {
    let live = PcLive::new(3).await;
    live.provider.reject.store(true, Ordering::SeqCst);
    let raw = live.raw(101_000).await;
    assert!(matches!(raw.signal, Signal::Buy { quantity: 3, .. }));
    assert!(live.submit(raw, 101_000).await.is_err());
    live.sync().await;
    assert_eq!(live.confirmed().await, (0, 0));
    assert!(live.fixture.manager.lock().await.pending.is_empty());
    live.provider.reject.store(false, Ordering::SeqCst);
    let raw = live.raw(101_000).await;
    assert!(matches!(
        live.submit(raw, 101_000).await.unwrap(),
        SubmissionOutcome::Submitted { .. }
    ));
    live.sync().await;
    assert_eq!(live.confirmed().await, (0, 0));
    assert!(live
        .strategies
        .lock()
        .await
        .on_tick("005930", 95_000, 0)
        .is_empty());
    live.fixture
        .manager
        .lock()
        .await
        .on_cancel("pc-order-2")
        .await
        .unwrap();
    live.sync().await;
    let raw = live.raw(101_000).await;
    assert!(matches!(raw.signal, Signal::Buy { quantity: 3, .. }));
    let skipped = live.submit(raw, 101_000).await.unwrap();
    assert!(
        matches!(skipped, SubmissionOutcome::Skipped { reason, .. } if reason.contains("동일 방향"))
    );
    assert_eq!(live.provider.requests.lock().await.len(), 2);
    assert_eq!(live.confirmed().await, (0, 0));
}

#[tokio::test]
async fn pc_live_partial_buy_uses_ceil_native_cost_and_sell_clamps_below_config() {
    let live = PcLive::new(3).await;
    let raw = live.raw(101_000).await;
    assert!(matches!(
        live.submit(raw, 101_000).await.unwrap(),
        SubmissionOutcome::Submitted { .. }
    ));
    live.fill("pc-order-1", 1, 100_001).await;
    assert_eq!(live.confirmed().await, (1, 100_001));
    live.fill("pc-order-1", 2, 200_003).await;
    assert_eq!(live.confirmed().await, (2, 200_003));
    assert!(live
        .strategies
        .lock()
        .await
        .on_tick("005930", 97_001, 0)
        .is_empty());
    live.fixture
        .manager
        .lock()
        .await
        .on_cancel("pc-order-1")
        .await
        .unwrap();
    live.publish(12).await;
    live.sync().await;
    let raw = live.raw(97_001).await;
    assert!(
        matches!(&raw.signal, Signal::Sell { quantity: 3, reason, .. }
        if reason.contains("손절") && reason.contains("100002.00"))
    );
    assert!(matches!(
        live.submit(raw, 97_001).await.unwrap(),
        SubmissionOutcome::Submitted { .. }
    ));
    assert_eq!(live.provider.requests.lock().await[1]["ORD_QTY"], "2");
    assert_eq!(live.confirmed().await, (2, 200_003));
    live.fill("pc-order-2", 1, 97_001).await;
    assert_eq!(live.confirmed().await, (1, 100_002));
    // Equal cumulative feedback is a retry, never another sale.
    live.fill("pc-order-2", 1, 97_001).await;
    assert_eq!(live.confirmed().await, (1, 100_002));
    live.fill("pc-order-2", 2, 194_002).await;
    assert_eq!(live.confirmed().await, (0, 0));
    live.publish(10).await;
    live.sync().await;
    // Buy preflight reports account exposure; this is not confirmed strategy ownership.
    let raw = live.raw(97_001).await;
    assert!(matches!(raw.signal, Signal::Buy { .. }));
    let outcome = live.submit(raw, 97_001).await.unwrap();
    assert!(
        matches!(&outcome,
        SubmissionOutcome::Skipped { reason, held_quantity: 10, avg_price: Some(400_000) } if reason.contains("손절 후 당일")),
        "{outcome:?}"
    );
    assert_eq!(live.provider.requests.lock().await.len(), 2);
}

#[tokio::test]
async fn pc_live_config_sized_sale_leaves_remainder_protected_by_default_risk_and_cooldown() {
    for filled in [1, 2] {
        let live = PcLive::new(5).await;
        live.full_buy(5).await;
        live.strategies
            .lock()
            .await
            .get_config_mut("price_condition_live_contract")
            .unwrap()
            .params["symbols"][0]["quantity"] = 2.into();
        let raw = live.raw(95_000).await;
        assert!(matches!(raw.signal, Signal::Sell { quantity: 2, .. }));
        assert!(matches!(
            live.submit(raw, 95_000).await.unwrap(),
            SubmissionOutcome::Submitted { .. }
        ));
        assert_eq!(live.provider.requests.lock().await[1]["ORD_QTY"], "2");
        live.fill("pc-order-2", filled, u128::from(filled) * 95_000)
            .await;
        let remaining = 5 - filled;
        assert_eq!(
            live.confirmed().await,
            (remaining, u128::from(remaining) * 100_000)
        );
        if filled == 1 {
            live.fixture
                .manager
                .lock()
                .await
                .on_cancel("pc-order-2")
                .await
                .unwrap();
        }
        live.publish(remaining + 10).await;
        live.sync().await;
        let raw = live.raw(95_000).await;
        assert!(matches!(raw.signal, Signal::Sell { quantity: 2, .. }));
        // Default daily sell risk runs before TradeGuard; preserve both protections.
        let outcome = live.submit(raw.clone(), 95_000).await.unwrap();
        assert!(
            matches!(&outcome,
            SubmissionOutcome::Skipped { reason, held_quantity, avg_price: Some(100_000) }
            if *held_quantity == remaining && reason.contains("일일 매도 주문 제한")),
            "{outcome:?}"
        );
        let guard = live
            .fixture
            .manager
            .lock()
            .await
            .trade_guard
            .evaluate_for_scope(
                &live.run.broker_scope,
                &raw.signal,
                remaining,
                Some(100_000),
                95_000,
                false,
            );
        assert!(matches!(guard, GuardDecision::Block { reason } if reason.contains("동일 방향")));
        live.sync().await;
        let retry = live.raw(94_000).await;
        assert!(matches!(retry.signal, Signal::Sell { quantity: 2, .. }));
        assert_eq!(
            live.confirmed().await,
            (remaining, u128::from(remaining) * 100_000)
        );
        assert_eq!(live.provider.requests.lock().await.len(), 2);
    }
}

#[tokio::test]
async fn pc_live_blocked_and_provider_rejected_stops_record_no_new_daily_ban() {
    for blocked in [true, false] {
        let live = PcLive::new(2).await;
        live.full_buy(2).await;
        if blocked {
            live.fixture
                .manager
                .lock()
                .await
                .trade_guard
                .record_submitted_for_scope(
                    &live.run.broker_scope,
                    &Signal::Sell {
                        symbol: "005930".into(),
                        quantity: 1,
                        reason: "prior profit sale".into(),
                    },
                );
        } else {
            live.provider.reject.store(true, Ordering::SeqCst);
        }
        let raw = live.raw(95_000).await;
        let result = live.submit(raw, 95_000).await;
        if blocked {
            assert!(
                matches!(result.unwrap(), SubmissionOutcome::Skipped { reason, .. }
                if reason.contains("동일 방향"))
            );
        } else {
            assert!(result.is_err());
        }
        live.sync().await;
        assert_eq!(live.confirmed().await, (2, 200_000));
        assert!(live.fixture.manager.lock().await.pending.is_empty());
        let buy = Signal::Buy {
            symbol: "005930".into(),
            quantity: 2,
            reason: "probe".into(),
        };
        let decision = live
            .fixture
            .manager
            .lock()
            .await
            .trade_guard
            .evaluate_for_scope(&live.run.broker_scope, &buy, 0, None, 95_000, false);
        assert!(matches!(decision, GuardDecision::Block { reason } if !reason.contains("손절 후")));
        if !blocked {
            // Definitive provider rejection permits an immediate same-side retry.
            live.provider.reject.store(false, Ordering::SeqCst);
            live.sync().await;
            let retry = live.raw(94_000).await;
            assert!(matches!(
                live.submit(retry, 94_000).await.unwrap(),
                SubmissionOutcome::Submitted { .. }
            ));
            live.fixture
                .manager
                .lock()
                .await
                .on_cancel("pc-order-3")
                .await
                .unwrap();
            live.sync().await;
            assert_eq!(live.confirmed().await, (2, 200_000));
            let decision = live
                .fixture
                .manager
                .lock()
                .await
                .trade_guard
                .evaluate_for_scope(&live.run.broker_scope, &buy, 0, None, 94_000, false);
            assert!(
                matches!(decision, GuardDecision::Block { reason } if reason.contains("손절 후 당일"))
            );
            let raw = live.raw(94_000).await;
            let outcome = live.submit(raw, 94_000).await.unwrap();
            assert!(
                matches!(&outcome,
                SubmissionOutcome::Skipped { reason, .. } if reason.contains("일일 매도 주문 제한")),
                "{outcome:?}"
            );
        }
    }
}

#[tokio::test]
async fn pc_live_equal_quantity_feedback_updates_native_average_without_account_total_leak() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    fixture.begin(0).await;
    let id = "price_condition_average_contract";
    let strategies = manager(&fixture, vec![pc_config(id, &fixture.profile.account_no)]);
    let owned = key(&fixture, id, BrokerMarket::Kr, "005930");
    let mut ledger = fixture
        .manager
        .lock()
        .await
        .load_ownership_ledger()
        .await
        .unwrap();
    buy(&mut ledger, owned.clone(), "initial-buy", 2, 100_000);
    fixture
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    fixture.seed_holding("005930", 30, 500_000).await;
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    assert!(strategies
        .lock()
        .await
        .on_tick("005930", 101_000, 0)
        .is_empty());
    ledger
        .apply_cumulative(&OwnershipFill {
            order_record_id: "replace-sell".into(),
            key: owned.clone(),
            side: OrderSide::Sell,
            ordered_quantity: 2,
            cumulative_quantity: Some(2),
            cumulative_notional: Some(200_000),
        })
        .unwrap();
    buy(&mut ledger, owned, "replacement-buy", 2, 110_000);
    fixture
        .manager
        .lock()
        .await
        .save_ownership_ledger(&ledger)
        .await
        .unwrap();
    OrderManager::sync_strategy_ownership_shared(&fixture.manager, &strategies)
        .await
        .unwrap();
    let raw = strategies.lock().await.on_tick("005930", 105_000, 0);
    assert!(
        matches!(&raw[0].signal, Signal::Sell { quantity: 2, reason, .. }
        if reason.contains("손절") && reason.contains("110000.00"))
    );
    assert_eq!(
        fixture
            .manager
            .lock()
            .await
            .position_tracker
            .lock()
            .await
            .get("005930")
            .unwrap()
            .quantity,
        30
    );
}

#[tokio::test]
async fn pc_live_accepted_unfilled_stop_terminal_outcomes_keep_scoped_submission_ban() {
    for terminal in [OrderStatus::Cancelled, OrderStatus::Rejected] {
        let live = PcLive::new(2).await;
        live.full_buy(2).await;
        let raw = live.raw(95_000).await;
        assert!(matches!(
            live.submit(raw, 95_000).await.unwrap(),
            SubmissionOutcome::Submitted { .. }
        ));
        live.sync().await;
        assert_eq!(live.confirmed().await, (2, 200_000));
        assert_eq!(
            live.fixture.manager.lock().await.pending["pc-order-2"].filled_quantity,
            0
        );
        assert!(live
            .strategies
            .lock()
            .await
            .on_tick("005930", 94_000, 0)
            .is_empty());
        {
            let mut orders = live.fixture.manager.lock().await;
            if terminal == OrderStatus::Cancelled {
                orders.on_cancel("pc-order-2").await.unwrap();
            } else {
                orders
                    .finalize_pending_order(
                        "pc-order-2",
                        terminal,
                        "REJECTED_AFTER_ACCEPTANCE",
                        None,
                    )
                    .await
                    .unwrap();
            }
            let records = orders
                .order_store
                .get_by_date(chrono::Local::now().date_naive())
                .await
                .unwrap();
            assert_eq!(
                records
                    .iter()
                    .find(|record| record.kis_order_id.as_deref() == Some("pc-order-2"))
                    .unwrap()
                    .status,
                terminal
            );
        }
        live.sync().await;
        assert_eq!(live.confirmed().await, (2, 200_000));
        let buy = Signal::Buy {
            symbol: "005930".into(),
            quantity: 2,
            reason: "scope probe".into(),
        };
        let mut orders = live.fixture.manager.lock().await;
        assert!(
            matches!(orders.trade_guard.evaluate_for_scope(&live.run.broker_scope,
            &buy, 0, None, 95_000, false), GuardDecision::Block { reason }
            if reason.contains("손절 후 당일"))
        );
        let mut other_account = live.run.broker_scope.clone();
        other_account.account_id.as_mut().unwrap().0 = "synthetic-other-account".into();
        assert!(matches!(
            orders
                .trade_guard
                .evaluate_for_scope(&other_account, &buy, 0, None, 95_000, false),
            GuardDecision::Allow
        ));
        let other_symbol = Signal::Buy {
            symbol: "000660".into(),
            quantity: 2,
            reason: "scope probe".into(),
        };
        assert!(matches!(
            orders.trade_guard.evaluate_for_scope(
                &live.run.broker_scope,
                &other_symbol,
                0,
                None,
                95_000,
                false
            ),
            GuardDecision::Allow
        ));
        drop(orders);
        assert!(matches!(
            live.raw(94_000).await.signal,
            Signal::Sell { quantity: 2, .. }
        ));
        assert_eq!(live.provider.requests.lock().await.len(), 2);
    }
}
