use super::*;
use crate::{
    broker::{BrokerAccountId, BrokerId, BrokerScope},
    trading::strategy::{build_strategy, OhlcCandle, Signal, StrategyConfig},
};
use std::sync::atomic::{AtomicUsize, Ordering};

fn now() -> DateTime<Utc> {
    "2026-10-19T13:30:00Z".parse().unwrap()
}
fn scope() -> BrokerScope {
    BrokerScope::new(
        BrokerId::Kis,
        Some(BrokerAccountId("fixture-account".into())),
    )
}
fn manager(symbols: &[&str], configs: usize) -> Arc<Mutex<StrategyManager>> {
    let mut manager = StrategyManager::new();
    for index in 0..configs {
        manager.add(build_strategy(
            StrategyConfig::new(
                format!("fifty_two_week_high_{index}"),
                "fixture",
                true,
                symbols.iter().map(|symbol| (*symbol).into()).collect(),
                1,
                serde_json::json!({"lookback_days":3+index,"stop_loss_pct":3.0}),
            )
            .with_scope(BrokerId::Kis, Some("fixture-account".into())),
        ));
    }
    manager.set_history_execution_identity(scope(), Some("fixture-profile".into()));
    Arc::new(Mutex::new(manager))
}
async fn targets(manager: &Arc<Mutex<StrategyManager>>) -> Vec<DailyHistoryTicket> {
    manager
        .lock()
        .await
        .prepare_daily_history_targets(&scope(), "fixture-profile", now())
}
fn history(count: usize) -> CollectedDailyHistory {
    CollectedDailyHistory {
        candles: vec![
            OhlcCandle {
                open: 9_000,
                high: 10_000,
                low: 8_000,
                close: 9_000
            };
            count
        ],
        source: "fixture:D".into(),
        last_completed_date: Some("2026-10-18".into()),
        end_reason: "fixture end".into(),
    }
}
fn success(counter: &Arc<AtomicUsize>) -> impl Fn(DailyHistoryTicket) -> FetchFuture {
    let counter = counter.clone();
    move |ticket| {
        counter.fetch_add(1, Ordering::SeqCst);
        Box::pin(std::future::ready(Ok(history(ticket.required_bars))))
    }
}
async fn completed(jobs: &DailyHistoryJobs) {
    for _ in 0..100 {
        if jobs.entries.values().all(|entry| {
            entry
                .handle
                .as_ref()
                .is_none_or(|handle| handle.is_finished())
        }) {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("synthetic futures did not finish");
}

#[tokio::test]
async fn same_scope_instrument_is_fetched_once_and_not_refetched_each_tick() {
    let manager = manager(&["SOXQ"], 2);
    let running = Arc::new(Mutex::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut jobs = DailyHistoryJobs::default();
    let instant = Instant::now();
    jobs.tick(
        targets(&manager).await,
        now(),
        instant,
        &running,
        &manager,
        success(&calls),
    )
    .await;
    completed(&jobs).await;
    jobs.tick(
        targets(&manager).await,
        now(),
        instant,
        &running,
        &manager,
        success(&calls),
    )
    .await;
    for _ in 0..10 {
        jobs.tick(
            targets(&manager).await,
            now(),
            instant + Duration::from_secs(20),
            &running,
            &manager,
            success(&calls),
        )
        .await;
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let statuses = manager.lock().await.daily_history_status();
    assert_eq!(statuses.len(), 2);
    assert!(statuses.iter().all(|status| status.state == "ready"));
    assert_eq!(
        statuses
            .iter()
            .map(|status| status.available_bars)
            .collect::<Vec<_>>(),
        [3, 4]
    );
}

#[tokio::test]
async fn pending_network_does_not_block_ticks_or_held_stop_and_stop_aborts_jobs() {
    let manager = manager(&["SOXQ"], 1);
    let running = Arc::new(Mutex::new(true));
    manager.lock().await.sync_position("SOXQ", 1, 10_000);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(0));
    let finished = Arc::new(AtomicUsize::new(0));
    let mut jobs = DailyHistoryJobs::default();
    let capture = semaphore.clone();
    let output = finished.clone();
    tokio::time::timeout(
        Duration::from_millis(250),
        jobs.tick(
            targets(&manager).await,
            now(),
            Instant::now(),
            &running,
            &manager,
            move |_| {
                let semaphore = capture.clone();
                let finished = output.clone();
                Box::pin(async move {
                    let _permit = semaphore.acquire_owned().await.unwrap();
                    finished.fetch_add(1, Ordering::SeqCst);
                    Ok(history(3))
                })
            },
        ),
    )
    .await
    .expect("scheduler awaited provider I/O");
    assert!(matches!(
        manager.lock().await.on_tick("SOXQ", 9_000, 0)[0].signal,
        Signal::Sell { .. }
    ));
    *running.lock().await = false;
    manager.lock().await.advance_history_generation();
    jobs.tick(vec![], now(), Instant::now(), &running, &manager, |_| {
        Box::pin(std::future::ready(Ok(history(3))))
    })
    .await;
    assert!(jobs.entries.is_empty());
    semaphore.add_permits(1);
    tokio::task::yield_now().await;
    assert_eq!(finished.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn old_same_scope_completed_result_cannot_cross_stop_start_generation() {
    let manager = manager(&["SOXQ"], 1);
    let running = Arc::new(Mutex::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut jobs = DailyHistoryJobs::default();
    let old = targets(&manager).await;
    jobs.tick(
        old.clone(),
        now(),
        Instant::now(),
        &running,
        &manager,
        success(&calls),
    )
    .await;
    completed(&jobs).await;
    {
        let mut manager = manager.lock().await;
        manager.advance_history_generation();
        manager.advance_history_generation();
    }
    let fresh = targets(&manager).await;
    jobs.tick(
        old,
        now(),
        Instant::now(),
        &running,
        &manager,
        success(&calls),
    )
    .await;
    assert!(manager
        .lock()
        .await
        .daily_history_status()
        .iter()
        .all(|status| status.state == "pending"));
    jobs.tick(
        fresh,
        now(),
        Instant::now(),
        &running,
        &manager,
        success(&calls),
    )
    .await;
    completed(&jobs).await;
    jobs.tick(
        targets(&manager).await,
        now(),
        Instant::now(),
        &running,
        &manager,
        success(&calls),
    )
    .await;
    assert_eq!(
        manager.lock().await.daily_history_status()[0].state,
        "ready"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn failures_use_backoff_and_do_not_retry_on_every_quote_tick() {
    let manager = manager(&["SOXQ"], 1);
    let running = Arc::new(Mutex::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let instant = Instant::now();
    let mut jobs = DailyHistoryJobs::default();
    let fail = || {
        let calls = calls.clone();
        move |_| -> FetchFuture {
            calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(std::future::ready(Err("fixture failure".into())))
        }
    };
    jobs.tick(
        targets(&manager).await,
        now(),
        instant,
        &running,
        &manager,
        fail(),
    )
    .await;
    completed(&jobs).await;
    jobs.tick(
        targets(&manager).await,
        now(),
        instant,
        &running,
        &manager,
        fail(),
    )
    .await;
    assert_eq!(
        manager.lock().await.daily_history_status()[0].state,
        "failed"
    );
    assert!(manager.lock().await.daily_history_status()[0]
        .next_retry_at
        .is_some());
    jobs.tick(
        targets(&manager).await,
        now(),
        instant + Duration::from_secs(29),
        &running,
        &manager,
        fail(),
    )
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    jobs.tick(
        targets(&manager).await,
        now(),
        instant + Duration::from_secs(30),
        &running,
        &manager,
        fail(),
    )
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    completed(&jobs).await;
    assert_eq!(retry_delay(1).as_secs(), 30);
    assert_eq!(retry_delay(2).as_secs(), 60);
    assert_eq!(retry_delay(u32::MAX).as_secs(), 900);
}

#[tokio::test]
async fn insufficient_history_reports_real_count_reason_and_retry() {
    let manager = manager(&["SOXQ"], 1);
    let running = Arc::new(Mutex::new(true));
    let mut jobs = DailyHistoryJobs::default();
    let instant = Instant::now();
    let fetch = |_| -> FetchFuture { Box::pin(std::future::ready(Ok(history(1)))) };
    jobs.tick(
        targets(&manager).await,
        now(),
        instant,
        &running,
        &manager,
        fetch,
    )
    .await;
    completed(&jobs).await;
    jobs.tick(
        targets(&manager).await,
        now(),
        instant,
        &running,
        &manager,
        fetch,
    )
    .await;
    let status = manager.lock().await.daily_history_status().remove(0);
    assert_eq!(
        (
            status.state.as_str(),
            status.available_bars,
            status.required_bars
        ),
        ("insufficient", 1, 3)
    );
    assert!(status.next_retry_at.is_some());
    assert!(status.last_error.unwrap().contains("fixture end"));
    assert!(manager.lock().await.on_tick("SOXQ", 20_000, 0).is_empty());
}

#[tokio::test]
async fn at_most_two_jobs_run_and_other_instruments_are_not_starved() {
    let manager = manager(&["SOXQ", "SPY", "QQQ"], 1);
    let running = Arc::new(Mutex::new(true));
    let semaphore = Arc::new(tokio::sync::Semaphore::new(0));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut jobs = DailyHistoryJobs::default();
    let instant = Instant::now();
    let capture = semaphore.clone();
    let counter = calls.clone();
    jobs.tick(
        targets(&manager).await,
        now(),
        instant,
        &running,
        &manager,
        move |ticket| {
            counter.fetch_add(1, Ordering::SeqCst);
            let semaphore = capture.clone();
            Box::pin(async move {
                let permit = semaphore.acquire_owned().await.unwrap();
                permit.forget();
                Ok(history(ticket.required_bars))
            })
        },
    )
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        jobs.entries
            .values()
            .filter(|entry| entry.handle.is_some())
            .count(),
        2
    );
    semaphore.add_permits(2);
    completed(&jobs).await;
    jobs.tick(
        targets(&manager).await,
        now(),
        instant,
        &running,
        &manager,
        success(&calls),
    )
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}
