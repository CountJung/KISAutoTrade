use super::*;
use crate::broker::{
    BrokerAdapterError, BrokerAdapterResult, BrokerCandlePage, BrokerPriceQuote, BrokerQuantity,
};
use std::collections::VecDeque;
use std::sync::Mutex;

enum Reply {
    Page(BrokerCandlePage),
    Error,
    Pending(std::sync::Arc<std::sync::atomic::AtomicBool>),
    Gated {
        entered: tokio::sync::oneshot::Sender<()>,
        gate: tokio::sync::oneshot::Receiver<()>,
        on_drop: Box<dyn FnOnce() + Send>,
    },
}
struct Fake {
    replies: Mutex<VecDeque<Reply>>,
    calls: Mutex<Vec<CandlePageRequest>>,
    broker: BrokerId,
}
impl Fake {
    fn new(replies: Vec<Reply>) -> Self {
        Self {
            replies: Mutex::new(replies.into()),
            calls: Mutex::new(Vec::new()),
            broker: BrokerId::Toss,
        }
    }
}
#[async_trait::async_trait]
impl BrokerAdapter for Fake {
    fn broker_id(&self) -> BrokerId {
        self.broker
    }
    async fn get_price(&self, _: &BrokerSymbol) -> BrokerAdapterResult<BrokerPriceQuote> {
        panic!("offline page only")
    }
    async fn get_candles(
        &self,
        _: &BrokerSymbol,
        _: &str,
        _: &str,
        _: &str,
    ) -> BrokerAdapterResult<Vec<BrokerCandle>> {
        panic!("page only")
    }
    async fn get_candle_page(
        &self,
        request: &CandlePageRequest,
    ) -> BrokerAdapterResult<BrokerCandlePage> {
        self.calls.lock().unwrap().push(request.clone());
        let reply = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected dispatch");
        match reply {
            Reply::Page(page) => Ok(page),
            Reply::Error => Err(BrokerAdapterError::InvalidRequest(
                "secret body/query must not escape".into(),
            )),
            Reply::Gated {
                entered,
                gate,
                on_drop,
            } => {
                struct DropProof(Option<Box<dyn FnOnce() + Send>>);
                impl Drop for DropProof {
                    fn drop(&mut self) {
                        if let Some(proof) = self.0.take() {
                            proof();
                        }
                    }
                }
                let _proof = DropProof(Some(on_drop));
                let _ = entered.send(());
                let _ = gate.await;
                Ok(BrokerCandlePage {
                    candles: vec![row("2026-01-07T15:59:00-05:00")],
                    next_cursor: Some(cursor("2026-01-07T15:59:00-05:00")),
                    end_reason: None,
                    timestamp_semantics: "barClose",
                })
            }
            Reply::Pending(flag) => {
                struct Dropped(std::sync::Arc<std::sync::atomic::AtomicBool>);
                impl Drop for Dropped {
                    fn drop(&mut self) {
                        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                }
                let _guard = Dropped(flag);
                std::future::pending().await
            }
        }
    }
}
fn row(time: &str) -> BrokerCandle {
    BrokerCandle {
        symbol: BrokerSymbol("KORU".into()),
        market: BrokerMarket::Us,
        date: time.into(),
        open: BrokerMoney::usd("100.00"),
        high: BrokerMoney::usd("102.00"),
        low: BrokerMoney::usd("99.00"),
        close: BrokerMoney::usd("101.00"),
        volume: BrokerQuantity("1234".into()),
    }
}
fn cursor(time: &str) -> CandleCursor {
    CandleCursor::Toss {
        before: time.into(),
    }
}
fn page(rows: Vec<BrokerCandle>, next: Option<CandleCursor>, daily: bool) -> Reply {
    Reply::Page(BrokerCandlePage {
        candles: rows,
        next_cursor: next,
        end_reason: None,
        timestamp_semantics: if daily { "tradingDate" } else { "barClose" },
    })
}
fn daily() -> Reply {
    page(vec![row("2026-01-06T00:00:00-05:00")], None, true)
}
fn context() -> Context {
    Context::new(
        DateTime::parse_from_rfc3339("2026-01-07T23:00:00-05:00")
            .unwrap()
            .to_utc(),
        2,
    )
}
async fn run(fake: &Fake, c: Context) -> CmdResult<(Vec<BrokerCandle>, Vec<BrokerCandle>)> {
    super::super::fetch_leveraged_trend_hold_candles(
        fake,
        &BrokerSymbol("KORU".into()),
        "1m",
        Some(c),
    )
    .await
}
async fn failure(replies: Vec<Reply>, c: Context, reason: &str, admitted: usize) -> Fake {
    let fake = Fake::new(replies);
    let error = run(&fake, c).await.unwrap_err();
    assert_eq!(error.code, "LTH_COLLECTION_INCOMPLETE");
    assert!(error.message.contains(reason), "{}", error.message);
    assert!(
        error.message.contains(&format!(
            "admitted={admitted} missing={}",
            usize::from(c.count) - admitted
        )),
        "{}",
        error.message
    );
    assert!(!error.message.contains("secret"));
    fake
}

use super::super::lth_runtime::{self, CancelAck, LthResearchRuntime, Scope};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

fn minute_rows() -> Vec<BrokerCandle> {
    (41..60)
        .map(|minute| row(&format!("2026-01-07T15:{minute:02}:00-05:00")))
        .chain(std::iter::once(row("2026-01-07T16:00:00-05:00")))
        .collect()
}

#[tokio::test(start_paused = true)]
async fn runtime_rejected_handles_never_create_factory_or_dispatch_collector() {
    let runtime = LthResearchRuntime::new();
    let fake = Fake::new(vec![]);
    let factories = AtomicUsize::new(0);
    let prepared = lth_runtime::fixture();
    let scope = Scope::from_profile(&prepared.profile);
    let handle = runtime
        .reserve_with_reference(prepared, context().as_of)
        .unwrap();
    assert!(runtime
        .run(&handle.wrong_owner(), &scope, |_, c| {
            factories.fetch_add(1, Ordering::SeqCst);
            run(&fake, c)
        })
        .await
        .is_err());
    let mut changed = lth_runtime::fixture();
    changed.profile.is_paper_trading = false;
    let changed_scope = Scope::from_profile(&changed.profile);
    assert!(runtime
        .run(&handle, &changed_scope, |_, c| {
            factories.fetch_add(1, Ordering::SeqCst);
            run(&fake, c)
        })
        .await
        .is_err());
    runtime.cancel(&handle, &scope).await.unwrap();
    assert!(runtime
        .run(&handle, &scope, |_, c| {
            factories.fetch_add(1, Ordering::SeqCst);
            run(&fake, c)
        })
        .await
        .is_err());
    let expired = runtime
        .reserve_with_reference(lth_runtime::fixture(), context().as_of)
        .unwrap();
    tokio::time::advance(Duration::from_secs(31)).await;
    assert!(runtime
        .run(&expired, &scope, |_, c| {
            factories.fetch_add(1, Ordering::SeqCst);
            run(&fake, c)
        })
        .await
        .is_err());
    tokio::time::advance(Duration::from_secs(61)).await;
    assert!(runtime
        .run(&expired, &scope, |_, c| {
            factories.fetch_add(1, Ordering::SeqCst);
            run(&fake, c)
        })
        .await
        .is_err());
    assert_eq!(factories.load(Ordering::SeqCst), 0);
    assert_eq!(fake.calls.lock().unwrap().len(), 0);
}

#[tokio::test(start_paused = true)]
async fn runtime_actual_pipeline_freezes_profile_mode_params_cutoff_and_budget() {
    let runtime = LthResearchRuntime::new();
    let mut prepared = lth_runtime::fixture();
    let mut source_profile = prepared.profile.clone();
    let scope = Scope::from_profile(&prepared.profile);
    let fingerprint = prepared.params_fingerprint.clone();
    let params = serde_json::to_value(&prepared.params).unwrap();
    prepared.assumptions.initial_capital_krw = 12345.0;
    let reference = context().as_of;
    let handle = runtime.reserve_with_reference(prepared, reference).unwrap();
    source_profile.is_paper_trading = false;
    source_profile.id = "offline-changed".into();
    source_profile.account_no = "offline-changed".into();
    let fake = Fake::new(vec![daily(), page(minute_rows(), None, false)]);
    assert!(fake.calls.lock().unwrap().is_empty()); // reserve/factory/provider IO zero
    let factory_calls = AtomicUsize::new(0);
    let reserved_at = tokio::time::Instant::now();
    tokio::time::advance(Duration::from_secs(7)).await;
    let (_, minute) = runtime
        .run(&handle, &scope, |snapshot, c| {
            factory_calls.fetch_add(1, Ordering::SeqCst);
            assert!(snapshot.profile.is_paper_trading);
            assert!(snapshot.profile.id != source_profile.id);
            assert!(snapshot.profile.account_no != source_profile.account_no);
            assert!(snapshot.profile.broker_id == BrokerId::Toss);
            assert!(snapshot.profile.app_key == "offline-unused");
            assert!(snapshot.profile.app_secret == "offline-unused");
            assert!(snapshot.symbol == "KORU" && snapshot.interval == "1m" && snapshot.count == 20);
            assert!(snapshot.params_fingerprint == fingerprint);
            assert!(serde_json::to_value(&snapshot.params).unwrap() == params);
            assert!(snapshot.assumptions.initial_capital_krw == 12345.0);
            assert_eq!(c.as_of, reference);
            assert_eq!(c.deadline, reserved_at + Duration::from_secs(30));
            run(&fake, c)
        })
        .await
        .unwrap();
    assert_eq!(factory_calls.load(Ordering::SeqCst), 1);
    assert_eq!(minute.len(), 20);
    let calls = fake.calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].interval, "D");
    assert_eq!(calls[1].interval, "1m");
    assert_eq!(calls[0].cursor, Some(cursor(&reference.to_rfc3339())));
    assert_eq!(calls[1].cursor, calls[0].cursor);
    drop(calls);
    let duplicate = runtime
        .run(&handle, &scope, |_, c| {
            factory_calls.fetch_add(1, Ordering::SeqCst);
            run(&fake, c)
        })
        .await;
    assert!(duplicate.is_err());
    assert_eq!(factory_calls.load(Ordering::SeqCst), 1);
    assert_eq!(fake.calls.lock().unwrap().len(), 2);
    assert_eq!(
        runtime.cancel(&handle, &scope).await.unwrap(),
        CancelAck::AlreadyTerminal
    );
}

// The entry barrier proves a real Pending provider future was polled. Its Drop
// observer proves quiescence precedes terminal publication; no timing sleeps.
fn gated_fixture(
    runtime: &Arc<LthResearchRuntime>,
    handle: &lth_runtime::Handle,
) -> (
    Arc<Fake>,
    tokio::sync::oneshot::Receiver<()>,
    tokio::sync::oneshot::Sender<()>,
    Arc<AtomicBool>,
    Arc<AtomicBool>,
) {
    let (entered_tx, entered) = tokio::sync::oneshot::channel();
    let (gate, gate_rx) = tokio::sync::oneshot::channel();
    let dropped = Arc::new(AtomicBool::new(false));
    let before_terminal = Arc::new(AtomicBool::new(false));
    let r = runtime.clone();
    let h = handle.clone();
    let d = dropped.clone();
    let before = before_terminal.clone();
    let fake = Arc::new(Fake::new(vec![
        daily(),
        page(
            vec![row("2026-01-07T16:00:00-05:00")],
            Some(cursor("2026-01-07T16:00:00-05:00")),
            false,
        ),
        Reply::Gated {
            entered: entered_tx,
            gate: gate_rx,
            on_drop: Box::new(move || {
                before.store(!r.is_terminal(&h), Ordering::SeqCst);
                d.store(true, Ordering::SeqCst);
            }),
        },
        page(minute_rows(), None, false),
    ]));
    (fake, entered, gate, dropped, before_terminal)
}

#[tokio::test(start_paused = true)]
async fn runtime_cancel_ack_waits_for_owned_pending_drop_and_fences_next_dispatch() {
    let runtime = Arc::new(LthResearchRuntime::new());
    let prepared = lth_runtime::fixture();
    let scope = Scope::from_profile(&prepared.profile);
    let handle = runtime
        .reserve_with_reference(prepared, context().as_of)
        .unwrap();
    let (fake, entered, gate, dropped, before_terminal) = gated_fixture(&runtime, &handle);
    let r = runtime.clone();
    let h = handle.clone();
    let s = scope.clone();
    let f = fake.clone();
    let task =
        tokio::spawn(async move { r.run(&h, &s, |_, c| async move { run(&f, c).await }).await });
    entered.await.unwrap();
    assert!(!dropped.load(Ordering::SeqCst));
    assert!(runtime.cancel(&handle.wrong_owner(), &scope).await.is_err());
    assert!(!dropped.load(Ordering::SeqCst));
    let factory = AtomicUsize::new(0);
    assert!(runtime
        .run(&handle, &scope, |_, c| {
            factory.fetch_add(1, Ordering::SeqCst);
            run(&fake, c)
        })
        .await
        .is_err());
    assert_eq!(factory.load(Ordering::SeqCst), 0);
    let mut cancel = Box::pin(runtime.cancel(&handle, &scope));
    std::future::poll_fn(|cx| {
        assert!(
            std::future::Future::poll(cancel.as_mut(), cx).is_pending(),
            "cancel request is not an ACK while owned provider work is alive"
        );
        std::task::Poll::Ready(())
    })
    .await;
    assert!(!dropped.load(Ordering::SeqCst));
    assert!(!runtime.is_terminal(&handle));
    assert!(runtime.reserve(lth_runtime::fixture()).is_err()); // Cancelling holds slot
    let (first_ack, second_ack) = tokio::join!(cancel, runtime.cancel(&handle, &scope));
    assert_eq!(first_ack.unwrap(), CancelAck::Quiescent);
    assert_eq!(second_ack.unwrap(), CancelAck::Quiescent);
    assert!(dropped.load(Ordering::SeqCst));
    assert!(before_terminal.load(Ordering::SeqCst));
    assert!(runtime.is_terminal(&handle));
    let at_ack = fake.calls.lock().unwrap().len();
    assert_eq!(at_ack, 3);
    let _ = gate.send(());
    tokio::task::yield_now().await;
    assert_eq!(
        fake.calls.lock().unwrap().len() - at_ack,
        0,
        "post-ACK dispatch must be literal zero"
    );
    assert!(task.await.unwrap().is_err());
    assert_eq!(
        runtime.cancel(&handle, &scope).await.unwrap(),
        CancelAck::AlreadyTerminal
    );
    let next = runtime.reserve(lth_runtime::fixture()).unwrap();
    runtime.cancel(&next, &scope).await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn runtime_request_abort_drops_provider_before_terminal_and_releases_slot() {
    let runtime = Arc::new(LthResearchRuntime::new());
    let prepared = lth_runtime::fixture();
    let scope = Scope::from_profile(&prepared.profile);
    let handle = runtime
        .reserve_with_reference(prepared, context().as_of)
        .unwrap();
    let (fake, entered, gate, dropped, before_terminal) = gated_fixture(&runtime, &handle);
    let r = runtime.clone();
    let h = handle.clone();
    let s = scope.clone();
    let f = fake.clone();
    let task =
        tokio::spawn(async move { r.run(&h, &s, |_, c| async move { run(&f, c).await }).await });
    entered.await.unwrap();
    task.abort();
    assert!(task.await.err().unwrap().is_cancelled());
    assert!(dropped.load(Ordering::SeqCst) && before_terminal.load(Ordering::SeqCst));
    assert!(runtime.is_terminal(&handle));
    let _ = gate.send(());
    tokio::task::yield_now().await;
    assert_eq!(fake.calls.lock().unwrap().len(), 3);
    let next = runtime.reserve(lth_runtime::fixture()).unwrap();
    runtime.cancel(&next, &scope).await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn runtime_request_future_drop_owns_and_destroys_actual_collection_before_lease() {
    let runtime = Arc::new(LthResearchRuntime::new());
    let prepared = lth_runtime::fixture();
    let scope = Scope::from_profile(&prepared.profile);
    let handle = runtime
        .reserve_with_reference(prepared, context().as_of)
        .unwrap();
    let (fake, mut entered, gate, dropped, before_terminal) = gated_fixture(&runtime, &handle);
    let mut request = Box::pin(runtime.run(&handle, &scope, |_, c| run(&fake, c)));
    tokio::select! {
        result = &mut request => panic!("unexpected ready request: {}", result.is_ok()),
        result = &mut entered => result.unwrap(),
    }
    assert!(!dropped.load(Ordering::SeqCst));
    drop(request); // Drop the owned request future, not Pin<&mut Future>.
    assert!(dropped.load(Ordering::SeqCst) && before_terminal.load(Ordering::SeqCst));
    assert!(runtime.is_terminal(&handle));
    let _ = gate.send(());
    tokio::task::yield_now().await;
    assert_eq!(fake.calls.lock().unwrap().len(), 3);
    let next = runtime.reserve(lth_runtime::fixture()).unwrap();
    runtime.cancel(&next, &scope).await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn runtime_pending_timeout_uses_reservation_remainder_and_drops_before_terminal() {
    let runtime = Arc::new(LthResearchRuntime::new());
    let prepared = lth_runtime::fixture();
    let scope = Scope::from_profile(&prepared.profile);
    let reserved_at = tokio::time::Instant::now();
    let handle = runtime
        .reserve_with_reference(prepared, context().as_of)
        .unwrap();
    tokio::time::advance(Duration::from_secs(25)).await;
    let (fake, entered, gate, dropped, before_terminal) = gated_fixture(&runtime, &handle);
    let r = runtime.clone();
    let h = handle.clone();
    let s = scope.clone();
    let f = fake.clone();
    let task =
        tokio::spawn(async move { r.run(&h, &s, |_, c| async move { run(&f, c).await }).await });
    entered.await.unwrap();
    tokio::time::advance(Duration::from_secs(6)).await;
    assert!(task.await.unwrap().is_err());
    assert!(
        reserved_at.elapsed() <= Duration::from_secs(31),
        "reservation budget was recharged: elapsed {:?}",
        reserved_at.elapsed()
    );
    assert!(dropped.load(Ordering::SeqCst) && before_terminal.load(Ordering::SeqCst));
    assert!(runtime.is_terminal(&handle));
    let _ = gate.send(());
    tokio::task::yield_now().await;
    assert_eq!(fake.calls.lock().unwrap().len(), 3);
    let next = runtime.reserve(lth_runtime::fixture()).unwrap();
    runtime.cancel(&next, &scope).await.unwrap();
}

#[tokio::test]
async fn runtime_invalid_frame_preserves_real_collector_guard_and_releases_slot() {
    let runtime = LthResearchRuntime::new();
    let prepared = lth_runtime::fixture();
    let scope = Scope::from_profile(&prepared.profile);
    let handle = runtime
        .reserve_with_reference(prepared, context().as_of)
        .unwrap();
    let mut invalid = row("2026-01-07T16:00:00-05:00");
    invalid.market = BrokerMarket::Kr;
    let fake = Fake::new(vec![daily(), page(vec![invalid], None, false)]);
    let err = runtime
        .run(&handle, &scope, |_, c| run(&fake, c))
        .await
        .err()
        .unwrap();
    assert!(err.message.contains("invalid_scope"));
    assert_eq!(fake.calls.lock().unwrap().len(), 2);
    assert!(runtime.is_terminal(&handle));
    let next = runtime.reserve(lth_runtime::fixture()).unwrap();
    runtime.cancel(&next, &scope).await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn reservation_wait_does_not_recharge_collection_deadline() {
    let c = context();
    tokio::time::advance(Duration::from_secs(31)).await;
    let fake = Fake::new(vec![
        daily(),
        page(
            vec![
                row("2026-01-07T16:00:00-05:00"),
                row("2026-01-07T15:59:00-05:00"),
            ],
            None,
            false,
        ),
    ]);
    let result = run(&fake, c).await;
    assert_eq!(
        fake.calls.lock().unwrap().len(),
        0,
        "reservation absolute deadline must prevent dispatch after waiting 31 seconds"
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn overlap_offset_alias_preserves_original_label_and_exact_ohlcv() {
    let original = row("2026-01-07T16:00:00.000-05:00");
    let alias = row("2026-01-07T21:00:00Z");
    let earlier = row("2026-01-07T15:59:00-05:00");
    let next = cursor("2026-01-07T21:00:00.000Z");
    let fake = Fake::new(vec![
        daily(),
        page(vec![original.clone()], Some(next.clone()), false),
        page(vec![alias, earlier.clone()], None, false),
    ]);
    let (d, m) = run(&fake, context()).await.unwrap();
    assert_eq!(d.len(), 1);
    assert_eq!(m, vec![earlier, original]);
    let calls = fake.calls.lock().unwrap();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].cursor, Some(cursor(&context().as_of.to_rfc3339())));
    assert_eq!(calls[1].cursor, calls[0].cursor);
    assert_eq!(calls[2].cursor, Some(next));
    assert_eq!(
        lth_regular::prepare(&m, &d, 2, context().as_of)
            .unwrap()
            .timed
            .len(),
        2
    );
}

#[tokio::test]
async fn adapter_reported_cursor_stall_cannot_be_accepted_as_complete() {
    let Reply::Page(mut stalled) = page(
        vec![
            row("2026-01-07T16:00:00-05:00"),
            row("2026-01-07T15:59:00-05:00"),
        ],
        None,
        false,
    ) else {
        unreachable!()
    };
    // The real Toss adapter normalizes a non-retreating cursor this way.
    stalled.end_reason = Some("cursorDidNotAdvance".into());
    let fake = Fake::new(vec![daily(), Reply::Page(stalled)]);
    let result = run(&fake, context()).await;
    assert!(
        result.is_err(),
        "adapter-normalized cursor stall must not become successful collection"
    );
    assert!(result.unwrap_err().message.contains("cursor_no_progress"));
    assert_eq!(fake.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn overlap_conflict_blocks_instead_of_first_or_last_wins() {
    let mut conflict = row("2026-01-07T21:00:00Z");
    conflict.volume = BrokerQuantity("1235".into());
    failure(
        vec![
            daily(),
            page(
                vec![row("2026-01-07T16:00:00-05:00")],
                Some(cursor("2026-01-07T21:00:00Z")),
                false,
            ),
            page(vec![conflict], None, false),
        ],
        context(),
        "conflicting_overlap",
        1,
    )
    .await;
}

#[tokio::test]
async fn exhaustion_empty_no_progress_partial_error_and_repeated_cursor_are_explicit() {
    let t = "2026-01-07T16:00:00-05:00";
    failure(
        vec![daily(), page(vec![row(t)], None, false)],
        context(),
        "provider_exhausted",
        1,
    )
    .await;
    failure(
        vec![daily(), page(vec![], None, false)],
        context(),
        "empty_page",
        0,
    )
    .await;
    failure(
        vec![
            daily(),
            page(vec![row(t)], Some(cursor(t)), false),
            Reply::Error,
        ],
        context(),
        "provider_error",
        1,
    )
    .await;
    failure(
        vec![
            daily(),
            page(vec![row(t)], Some(cursor(t)), false),
            page(
                vec![row(t)],
                Some(cursor("2026-01-07T15:59:00-05:00")),
                false,
            ),
        ],
        context(),
        "no_progress",
        1,
    )
    .await;
    // Different label for the same cursor instant is still non-retreating.
    failure(
        vec![
            daily(),
            page(vec![row(t)], Some(cursor(t)), false),
            page(
                vec![row("2026-01-07T15:59:00-05:00")],
                Some(cursor("2026-01-07T21:00:00Z")),
                false,
            ),
        ],
        context(),
        "cursor_no_progress",
        1,
    )
    .await;
}

#[tokio::test]
async fn raw_pre_after_and_holiday_rows_do_not_fill_the_admission_target() {
    failure(
        vec![
            daily(),
            page(
                vec![
                    row("2026-01-07T17:00:00-05:00"),
                    row("2026-01-07T09:00:00-05:00"),
                    row("2026-01-01T12:00:00-05:00"),
                ],
                None,
                false,
            ),
        ],
        context(),
        "provider_exhausted",
        0,
    )
    .await;
    let mut ended = match page(
        vec![row("2026-01-07T16:00:00-05:00")],
        Some(cursor("2026-01-07T16:00:00-05:00")),
        false,
    ) {
        Reply::Page(p) => p,
        _ => unreachable!(),
    };
    ended.end_reason = Some("provider terminal; do not dispatch again".into());
    let f = failure(
        vec![daily(), Reply::Page(ended)],
        context(),
        "provider_exhausted",
        1,
    )
    .await;
    assert_eq!(f.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn page_and_actual_raw_row_caps_stop_dispatch() {
    let mut c = context();
    c.max_pages = 2;
    let f = failure(
        vec![
            daily(),
            page(
                vec![row("2026-01-07T16:00:00-05:00")],
                Some(cursor("2026-01-07T16:00:00-05:00")),
                false,
            ),
        ],
        c,
        "page_limit",
        1,
    )
    .await;
    assert_eq!(f.calls.lock().unwrap().len(), 2);
    c = context();
    c.max_rows = 2;
    let f = failure(
        vec![
            daily(),
            page(
                vec![row("2026-01-07T16:00:00-05:00")],
                Some(cursor("2026-01-07T16:00:00-05:00")),
                false,
            ),
        ],
        c,
        "row_limit",
        1,
    )
    .await;
    assert_eq!(
        f.calls
            .lock()
            .unwrap()
            .iter()
            .map(|r| r.count)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
    c = context();
    c.page_size = 1;
    failure(
        vec![
            daily(),
            page(
                vec![
                    row("2026-01-07T16:00:00-05:00"),
                    row("2026-01-07T15:59:00-05:00"),
                ],
                None,
                false,
            ),
        ],
        c,
        "invalid_page_size",
        0,
    )
    .await;
}

#[tokio::test]
async fn expired_total_deadline_never_dispatches_ready_pages() {
    let mut c = context();
    c.deadline = tokio::time::Instant::now();
    let fake = Fake::new(vec![
        daily(),
        page(
            vec![
                row("2026-01-07T16:00:00-05:00"),
                row("2026-01-07T15:59:00-05:00"),
            ],
            None,
            false,
        ),
    ]);
    let result = run(&fake, c).await;
    assert_eq!(
        fake.calls.lock().unwrap().len(),
        0,
        "expired aggregate deadline must prevent provider dispatch, even for ready futures"
    );
    let error = result.unwrap_err();
    assert_eq!(error.code, "LTH_COLLECTION_INCOMPLETE");
    assert!(error.message.contains("total_deadline"));
}

#[tokio::test(start_paused = true)]
async fn total_deadline_drops_an_actual_pending_page_future() {
    let mut c = context();
    c.deadline = tokio::time::Instant::now() + Duration::from_millis(5);
    let dropped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let f = failure(
        vec![
            daily(),
            page(
                vec![row("2026-01-07T16:00:00-05:00")],
                Some(cursor("2026-01-07T16:00:00-05:00")),
                false,
            ),
            Reply::Pending(dropped.clone()),
        ],
        c,
        "total_deadline",
        1,
    )
    .await;
    assert_eq!(f.calls.lock().unwrap().len(), 3);
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::test]
async fn invalid_provider_cursor_time_scope_semantics_and_future_are_blocked() {
    let mut fake = Fake::new(vec![]);
    fake.broker = BrokerId::Kis;
    let err = run(&fake, context()).await.unwrap_err();
    assert!(err.message.contains("invalid_provider"));
    assert!(fake.calls.lock().unwrap().is_empty());
    for next in [
        CandleCursor::Kis {
            before: "20260107".into(),
        },
        cursor("bad-time"),
    ] {
        failure(
            vec![
                daily(),
                page(vec![row("2026-01-07T16:00:00-05:00")], Some(next), false),
            ],
            context(),
            "invalid_cursor",
            0,
        )
        .await;
    }
    for (r, reason) in [
        (row("invalid"), "invalid_time"),
        (row("2026-01-08T16:00:00-05:00"), "time_upper_bound"),
    ] {
        failure(
            vec![daily(), page(vec![r], None, false)],
            context(),
            reason,
            0,
        )
        .await;
    }
    for market in [false, true] {
        let mut r = row("2026-01-07T16:00:00-05:00");
        if market {
            r.market = BrokerMarket::Kr;
        } else {
            r.symbol = BrokerSymbol("OTHER".into());
        }
        failure(
            vec![daily(), page(vec![r], None, false)],
            context(),
            "invalid_scope",
            0,
        )
        .await;
    }
    failure(
        vec![
            daily(),
            page(vec![row("2026-01-07T16:00:00-05:00")], None, true),
        ],
        context(),
        "invalid_semantics",
        0,
    )
    .await;
}

#[tokio::test]
async fn daily_context_is_strictly_earlier_and_can_use_a_later_page() {
    let make_minute = || {
        page(
            vec![
                row("2026-01-07T16:00:00-05:00"),
                row("2026-01-07T15:59:00-05:00"),
            ],
            None,
            false,
        )
    };
    failure(
        vec![
            page(vec![row("2026-01-07T00:00:00-05:00")], None, true),
            make_minute(),
        ],
        context(),
        "daily_context_missing",
        2,
    )
    .await;
    let next = cursor("2026-01-07T00:00:00-05:00");
    let fake = Fake::new(vec![
        page(
            vec![row("2026-01-07T00:00:00-05:00")],
            Some(next.clone()),
            true,
        ),
        make_minute(),
        page(
            vec![
                row("2026-01-07T00:00:00-05:00"),
                row("2026-01-06T00:00:00-05:00"),
            ],
            None,
            true,
        ),
    ]);
    let (d, m) = run(&fake, context()).await.unwrap();
    let prepared = lth_regular::prepare(&m, &d, 2, context().as_of).unwrap();
    assert_eq!(prepared.warmup, vec![row("2026-01-06T00:00:00-05:00")]);
    let calls = fake.calls.lock().unwrap();
    assert_eq!(
        calls
            .iter()
            .map(|r| r.interval.as_str())
            .collect::<Vec<_>>(),
        vec!["D", "1m", "D"]
    );
    assert_eq!(calls[2].cursor, Some(next));
}

#[path = "provenance_tests.rs"]
mod provenance_tests;
