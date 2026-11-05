//! Futures are synthetic channels; fixture credentials never reach any provider.

use super::*;
use crate::{
    api::token::TokenManager,
    config::AppConfig,
    storage::{OrderStore, PendingOrderStore, RiskStore, StatsStore, TradeStore},
    trading::{
        position::{OverseasPositionTracker, PositionTracker},
        risk::RiskManager,
        strategy::{Signal, Strategy, StrategyConfig},
    },
};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::oneshot;

struct Probe {
    config: StrategyConfig,
    ticks: Arc<AtomicUsize>,
    held: Arc<AtomicU64>,
}

impl Strategy for Probe {
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
    fn set_enabled(&mut self, value: bool) {
        self.config.enabled = value;
    }
    fn on_tick(&mut self, _: &str, _: u64, _: u64) -> Signal {
        self.ticks.fetch_add(1, Ordering::SeqCst);
        Signal::Hold
    }
    fn sync_position(&mut self, _: &str, quantity: u64, _: u64) {
        self.held.store(quantity, Ordering::SeqCst);
    }
    fn reset(&mut self) {}
}

#[derive(Clone)]
struct Fixture {
    running: Arc<Mutex<bool>>,
    strategies: Arc<Mutex<StrategyManager>>,
    orders: Arc<Mutex<OrderManager>>,
    rest: Arc<KisRestClient>,
    ticks: Arc<AtomicUsize>,
    held: Arc<AtomicU64>,
}

impl Fixture {
    async fn new() -> (Self, ExecutionRunContext) {
        let config = Arc::new(AppConfig {
            broker_id: BrokerId::Kis,
            broker_account_id: String::new(),
            kis_app_key: String::new(),
            kis_app_secret: String::new(),
            kis_account_no: String::new(),
            kis_is_paper_trading: true,
            discord_bot_token: None,
            discord_channel_id: None,
            notification_levels: vec![],
        });
        // No method here calls the client. A loopback-only base is a second fence.
        let rest = Arc::new(KisRestClient::new(
            "http://127.0.0.1:1".into(),
            String::new(),
            String::new(),
            String::new(),
            true,
            Arc::new(RwLock::new(TokenManager::new(config))),
        ));
        let path = std::env::temp_dir().join(format!("daemon-fence-{}", uuid::Uuid::new_v4()));
        let orders = OrderManager::new(
            Arc::new(RwLock::new(Arc::clone(&rest))),
            Arc::new(RwLock::new(ProfilesConfig::default())),
            Arc::new(OrderStore::new(path.clone())),
            Arc::new(PendingOrderStore::new(path.clone())),
            Arc::new(TradeStore::new(path.clone())),
            Arc::new(Mutex::new(PositionTracker::new())),
            Arc::new(Mutex::new(OverseasPositionTracker::new())),
            Arc::new(StatsStore::new(path.clone())),
            Arc::new(RwLock::new(1400.0)),
            Arc::new(Mutex::new(RiskManager::default())),
            Arc::new(RiskStore::new(path)),
            None,
        );
        let ticks = Arc::new(AtomicUsize::new(0));
        let held = Arc::new(AtomicU64::new(7));
        let mut strategies = StrategyManager::new();
        strategies.add(Box::new(Probe {
            config: StrategyConfig::new(
                "price_condition",
                "probe",
                true,
                vec!["005930".into()],
                1,
                serde_json::json!({}),
            ),
            ticks: Arc::clone(&ticks),
            held: Arc::clone(&held),
        }));
        let fixture = Self {
            running: Arc::new(Mutex::new(false)),
            strategies: Arc::new(Mutex::new(strategies)),
            orders: Arc::new(Mutex::new(orders)),
            rest,
            ticks,
            held,
        };
        let context = fixture.restart().await;
        (fixture, context)
    }

    fn fence<'a>(&'a self, expected: &'a ExecutionRunContext) -> RunFence<'a> {
        RunFence {
            running: &self.running,
            strategies: &self.strategies,
            orders: &self.orders,
            expected,
        }
    }

    async fn restart(&self) -> ExecutionRunContext {
        let mut running = self.running.lock().await;
        *running = false;
        let mut strategies = self.strategies.lock().await;
        let mut orders = self.orders.lock().await;
        orders.end_execution_run();
        strategies.advance_history_generation();
        let context = orders
            .begin_execution_run(
                BrokerScope::kis_legacy(),
                None,
                strategies.history_generation(),
                Arc::clone(&self.rest),
                None,
            )
            .unwrap();
        *running = true;
        context
    }
}

#[tokio::test]
async fn delayed_quote_cannot_tick_a_restarted_same_scope_run() {
    let (fixture, old) = Fixture::new().await;
    let (entered_tx, entered_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let task_fixture = fixture.clone();
    let task = tokio::spawn(async move {
        let fence = task_fixture.fence(&old);
        let quote = fence
            .after(async {
                entered_tx.send(()).unwrap();
                release_rx.await.unwrap()
            })
            .await;
        match quote {
            Some(price) => {
                fence
                    .with_current(|manager| manager.on_tick("005930", price, 1))
                    .await
            }
            None => None,
        }
    });
    entered_rx.await.unwrap();
    // Network wait holds none of the lifecycle/configuration guards.
    assert!(fixture.running.try_lock().is_ok());
    assert!(fixture.strategies.try_lock().is_ok());
    assert!(fixture.orders.try_lock().is_ok());
    fixture.restart().await;
    release_tx.send(100).unwrap();
    assert!(task.await.unwrap().is_none());
    assert_eq!(fixture.ticks.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn delayed_name_and_late_rollback_do_not_mutate_the_new_run() {
    let (fixture, old) = Fixture::new().await;
    fixture
        .fence(&old)
        .with_current(|manager| manager.on_tick("005930", 100, 1))
        .await
        .unwrap();
    let (entered_tx, entered_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let task_fixture = fixture.clone();
    let old_for_name = old.clone();
    let task = tokio::spawn(async move {
        task_fixture
            .fence(&old_for_name)
            .after(async {
                entered_tx.send(()).unwrap();
                release_rx.await.unwrap()
            })
            .await
    });
    entered_rx.await.unwrap();
    let fresh = fixture.restart().await;
    fixture
        .fence(&fresh)
        .with_current(|manager| manager.sync_position("005930", 9, 200))
        .await
        .unwrap();
    release_tx.send("synthetic name").unwrap();
    assert!(task.await.unwrap().is_none());
    // Both Skipped's provider snapshot and Err's pre-submit snapshot use this gate.
    for old_quantity in [0, 2] {
        assert!(fixture
            .fence(&old)
            .with_current(|manager| { manager.sync_position("005930", old_quantity, 100) })
            .await
            .is_none());
    }
    assert_eq!(fixture.held.load(Ordering::SeqCst), 9);
    assert_eq!(fixture.ticks.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn configuration_save_rollback_aba_invalidates_quote_and_position_feedback() {
    let (fixture, old) = Fixture::new().await;
    let (entered_tx, entered_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let task_fixture = fixture.clone();
    let old_for_quote = old.clone();
    let task = tokio::spawn(async move {
        task_fixture
            .fence(&old_for_quote)
            .after(async {
                entered_tx.send(()).unwrap();
                release_rx.await.unwrap()
            })
            .await
    });
    entered_rx.await.unwrap();
    {
        let mut manager = fixture.strategies.lock().await;
        let previous = manager
            .all_configs()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        manager
            .update_config("price_condition", |config| config.order_quantity = 2)
            .unwrap();
        fixture
            .orders
            .lock()
            .await
            .publish_strategy_revision(manager.history_generation());
        manager.apply_saved_configs_for_scope(&previous, BrokerId::Kis, None);
        fixture
            .orders
            .lock()
            .await
            .publish_strategy_revision(manager.history_generation());
        assert_eq!(manager.all_configs()[0].order_quantity, 1);
    }
    release_tx.send(100).unwrap();
    assert!(task.await.unwrap().is_none());
    let observed = Arc::new(AtomicUsize::new(0));
    assert!(fixture
        .fence(&old)
        .with_current(|manager| {
            observed.fetch_add(1, Ordering::SeqCst);
            manager.on_tick("005930", 100, 1);
            manager.sync_position("005930", 0, 0);
        })
        .await
        .is_none());
    assert_eq!(observed.load(Ordering::SeqCst), 0);
    let current = capture_cycle(&fixture.running, &fixture.strategies, &fixture.orders)
        .await
        .unwrap();
    assert_eq!(old.epoch, current.context.epoch);
    assert_ne!(
        old.configuration_revision,
        current.context.configuration_revision
    );
    assert!(Arc::ptr_eq(&current.rest, &fixture.rest));
}

#[tokio::test]
async fn stale_balance_cannot_clear_or_add_new_run_health_failures() {
    let (fixture, old) = Fixture::new().await;
    let fresh = fixture.restart().await;
    fixture
        .fence(&fresh)
        .apply_balance(Err("fresh failure".into()))
        .await;
    let before = fixture.orders.lock().await.health_status();
    assert!(fixture
        .fence(&old)
        .apply_balance(Ok(1_000_000))
        .await
        .is_none());
    assert!(fixture
        .fence(&old)
        .apply_balance(Err("old failure".into()))
        .await
        .is_none());
    let after = fixture.orders.lock().await.health_status();
    assert_eq!(
        before.last_holdings_sync_attempt_at,
        after.last_holdings_sync_attempt_at
    );
    assert_eq!(after.holdings_consecutive_failures, 1);
    assert_eq!(
        after.last_holdings_sync_error.as_deref(),
        Some("fresh failure")
    );
    assert!(fixture.orders.lock().await.is_buy_suspended());
    assert_eq!(
        fixture.fence(&fresh).apply_balance(Ok(1_000_000)).await,
        Some(1_000_000)
    );
    assert!(!fixture.orders.lock().await.is_buy_suspended());
}

#[tokio::test]
async fn unpublished_revision_or_stopped_flag_fail_closed_but_current_tick_runs() {
    let (fixture, context) = Fixture::new().await;
    fixture
        .fence(&context)
        .with_current(|manager| manager.on_tick("005930", 100, 1))
        .await
        .unwrap();
    assert_eq!(fixture.ticks.load(Ordering::SeqCst), 1);
    *fixture.running.lock().await = false;
    assert!(!fixture.fence(&context).is_current().await);
    assert!(
        capture_cycle(&fixture.running, &fixture.strategies, &fixture.orders)
            .await
            .is_none()
    );
    *fixture.running.lock().await = true;
    fixture.strategies.lock().await.advance_history_generation();
    assert!(!fixture.fence(&context).is_current().await);
    assert!(
        capture_cycle(&fixture.running, &fixture.strategies, &fixture.orders)
            .await
            .is_none()
    );
    fixture.orders.lock().await.end_execution_run();
    assert!(
        capture_cycle(&fixture.running, &fixture.strategies, &fixture.orders)
            .await
            .is_none()
    );
}
