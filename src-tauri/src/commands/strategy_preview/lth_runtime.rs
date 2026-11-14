//! App-lifetime, in-memory US1m research ownership. One active job, 16 retained records.
//! Lazy pruning runs only on API calls: expired reservations become terminal; terminal
//! records expire after 60s. Running/Cancelling work is never pruned before quiescence.
//! This is app policy, not broker quota, financial ledger fencing, or a public wire API.
use super::{
    lth_collection::Context, AccountProfile, BrokerId, CmdError, CmdResult, PreparedLthPreview,
};
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Mutex, MutexGuard},
    time::Duration,
};
use tokio::{sync::watch, time::Instant};
use uuid::Uuid;

// Deliberately no Debug/Serialize on scope, handle, prepared profile or registry.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Scope {
    broker: BrokerId,
    profile: String,
    account: String,
    paper: bool,
}
impl Scope {
    pub(super) fn from_profile(profile: &AccountProfile) -> Self {
        Self {
            broker: profile.broker_id,
            profile: profile.id.clone(),
            account: profile.account_no.clone(),
            paper: profile.is_paper_trading,
        }
    }
}
#[derive(Clone)]
pub(super) struct Handle {
    id: Uuid,
    owner: Uuid,
}
impl Handle {
    pub(super) fn from_wire(id: &str, owner: &str) -> CmdResult<Self> {
        let parse = |value: &str| {
            Uuid::parse_str(value)
                .ok()
                .filter(|uuid| uuid.hyphenated().to_string() == value)
                .ok_or_else(|| error("invalid_handle"))
        };
        Ok(Self {
            id: parse(id)?,
            owner: parse(owner)?,
        })
    }
    pub(super) fn to_wire(&self) -> (String, String) {
        (self.id.to_string(), self.owner.to_string())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum End {
    Completed,
    Failed,
    Cancelled,
    Expired,
    Abandoned,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Reserved,
    Running,
    Cancelling,
    Terminal(End),
}
struct Record {
    owner: Uuid,
    scope: Scope,
    prepared: Option<PreparedLthPreview>,
    context: Context,
    phase: Phase,
    changed: watch::Sender<Phase>,
    terminal_at: Option<Instant>,
}
impl Record {
    fn terminal(&mut self, end: End) {
        self.prepared.take();
        self.phase = Phase::Terminal(end);
        self.terminal_at = Some(Instant::now());
        self.changed.send_replace(self.phase);
    }
}
struct Policy {
    retained: usize,
    ttl: Duration,
    budget: Duration,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            retained: 16,
            ttl: Duration::from_secs(60),
            budget: Duration::from_secs(30),
        }
    }
}
pub(crate) struct LthResearchRuntime {
    records: Mutex<HashMap<Uuid, Record>>,
    policy: Policy,
}
fn error(reason: &'static str) -> CmdError {
    CmdError {
        code: "LTH_RUNTIME_REJECTED".into(),
        message: format!("US 분봉 연구 작업 거절: {reason}"),
    }
}
impl LthResearchRuntime {
    pub(crate) fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
            policy: Policy::default(),
        }
    }
    fn lock(&self) -> MutexGuard<'_, HashMap<Uuid, Record>> {
        // No user code, provider dispatch or await occurs under this short registry lock.
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn prune(&self, records: &mut HashMap<Uuid, Record>) {
        let now = Instant::now();
        for record in records.values_mut() {
            if record.phase == Phase::Reserved && now >= record.context.deadline {
                record.terminal(End::Expired);
            }
        }
        records.retain(|_, record| {
            record
                .terminal_at
                .is_none_or(|at| now.duration_since(at) < self.policy.ttl)
        });
    }
    pub(super) fn reserve(&self, prepared: PreparedLthPreview) -> CmdResult<Handle> {
        self.reserve_at(prepared, Utc::now())
    }
    fn reserve_at(&self, prepared: PreparedLthPreview, as_of: DateTime<Utc>) -> CmdResult<Handle> {
        let mut records = self.lock();
        self.prune(&mut records);
        if records
            .values()
            .any(|r| !matches!(r.phase, Phase::Terminal(_)))
        {
            return Err(error("active_limit"));
        }
        if records.len() >= self.policy.retained {
            return Err(error("retained_limit"));
        }
        let handle = Handle {
            id: Uuid::new_v4(),
            owner: Uuid::new_v4(),
        };
        let context =
            Context::with_deadline(as_of, prepared.count, Instant::now() + self.policy.budget);
        let (changed, _) = watch::channel(Phase::Reserved);
        records.insert(
            handle.id,
            Record {
                owner: handle.owner,
                scope: Scope::from_profile(&prepared.profile),
                prepared: Some(prepared),
                context,
                phase: Phase::Reserved,
                changed,
                terminal_at: None,
            },
        );
        Ok(handle)
    }
    fn validate<'a>(
        records: &'a mut HashMap<Uuid, Record>,
        handle: &Handle,
        scope: &Scope,
    ) -> CmdResult<&'a mut Record> {
        let record = records
            .get_mut(&handle.id)
            .ok_or_else(|| error("unknown_or_expired"))?;
        if record.owner != handle.owner || record.scope != *scope {
            return Err(error("owner_or_scope"));
        }
        Ok(record)
    }
    pub(super) async fn run<T, F, Make>(
        &self,
        handle: &Handle,
        scope: &Scope,
        make: Make,
    ) -> CmdResult<T>
    where
        F: Future<Output = CmdResult<T>>,
        Make: FnOnce(PreparedLthPreview, Context) -> F,
    {
        let (prepared, context, mut changed) = {
            let mut records = self.lock();
            self.prune(&mut records);
            let record = Self::validate(&mut records, handle, scope)?;
            if record.phase != Phase::Reserved {
                return Err(error("not_reserved"));
            }
            if Instant::now() >= record.context.deadline {
                record.terminal(End::Expired);
                return Err(error("expired"));
            }
            let prepared = record.prepared.take().ok_or_else(|| error("consumed"))?;
            record.phase = Phase::Running;
            record.changed.send_replace(Phase::Running);
            (prepared, record.context, record.changed.subscribe())
        };
        let lease = Lease {
            runtime: self,
            handle: handle.clone(),
            scope: scope.clone(),
            done: false,
        };
        // The lease exists before factory invocation, including panic/error paths.
        if Instant::now() >= context.deadline {
            return Err(error("expired"));
        }
        let mut owned = OwnedWork {
            work: Some(Box::pin(make(prepared, context))),
            lease,
        };
        let result = tokio::select! {
            biased;
            _ = async {
                loop {
                    if matches!(*changed.borrow_and_update(), Phase::Cancelling | Phase::Terminal(_)) { break; }
                    if changed.changed().await.is_err() { break; }
                }
            } => Err(error("cancelled")),
            result = tokio::time::timeout_at(context.deadline, owned.work.as_mut().expect("owned work")) =>
                result.unwrap_or_else(|_| Err(error("expired"))),
        };
        // Drop the actual boxed future BEFORE terminal publication/notification. Dropping
        // Pin<&mut F> would only drop a borrow and would not provide this ACK guarantee.
        drop(owned.work.take());
        let end = owned.lease.finish(if result.is_ok() {
            End::Completed
        } else {
            End::Failed
        });
        match end {
            End::Completed | End::Failed => result,
            End::Expired => Err(error("expired")),
            _ => Err(error("cancelled")),
        }
    }
    // Internal failure receipt, never a cancel ACK. Running/Cancelling records
    // cannot be pruned; OwnedWork destroys its future before terminal publication.
    pub(super) fn terminal_failure_receipt(&self, handle: &Handle, scope: &Scope) -> bool {
        let mut records = self.lock();
        self.prune(&mut records);
        Self::validate(&mut records, handle, scope).is_ok_and(|record| {
            matches!(
                record.phase,
                Phase::Terminal(End::Failed | End::Expired | End::Abandoned)
            )
        })
    }
    // Owner capability cancellation deliberately uses the stored scope, even after
    // the user has selected another profile/mode. Reuse the existing quiescent await.
    pub(super) async fn cancel_owned(&self, handle: &Handle) -> CmdResult<CancelAck> {
        let scope = {
            let mut records = self.lock();
            self.prune(&mut records);
            let record = records
                .get(&handle.id)
                .ok_or_else(|| error("unknown_or_expired"))?;
            if record.owner != handle.owner {
                return Err(error("owner_or_scope"));
            }
            if matches!(
                record.phase,
                Phase::Terminal(End::Expired | End::Failed | End::Abandoned)
            ) {
                return Err(error("expired_or_failed"));
            }
            record.scope.clone()
        };
        let ack = self.cancel(handle, &scope).await?;
        // A running future can fail/expire between the first lock and cancel.
        // Preserve the old internal oracle, but never expose that race as public ACK.
        let records = self.lock();
        let record = records
            .get(&handle.id)
            .ok_or_else(|| error("unknown_or_expired"))?;
        if !matches!(
            record.phase,
            Phase::Terminal(End::Cancelled | End::Completed)
        ) {
            return Err(error("expired_or_failed"));
        }
        Ok(ack)
    }
    pub(super) async fn cancel(&self, handle: &Handle, scope: &Scope) -> CmdResult<CancelAck> {
        let mut changed = {
            let mut records = self.lock();
            self.prune(&mut records);
            let record = Self::validate(&mut records, handle, scope)?;
            match record.phase {
                Phase::Reserved => {
                    record.terminal(End::Cancelled);
                    return Ok(CancelAck::Quiescent);
                }
                Phase::Terminal(_) => return Ok(CancelAck::AlreadyTerminal),
                Phase::Running => {
                    record.phase = Phase::Cancelling;
                    record.changed.send_replace(Phase::Cancelling);
                }
                Phase::Cancelling => {}
            }
            record.changed.subscribe()
        };
        loop {
            if matches!(*changed.borrow_and_update(), Phase::Terminal(_)) {
                return Ok(CancelAck::Quiescent);
            }
            changed.changed().await.map_err(|_| error("closed"))?;
        }
    }
}
#[derive(PartialEq, Eq, Debug)]
pub(super) enum CancelAck {
    Quiescent,
    AlreadyTerminal,
}
struct Lease<'a> {
    runtime: &'a LthResearchRuntime,
    handle: Handle,
    scope: Scope,
    done: bool,
}
impl Lease<'_> {
    fn finish(&mut self, requested: End) -> End {
        let mut records = self.runtime.lock();
        // Running records cannot be pruned; handle is owned by this single consumer.
        let record = records
            .get_mut(&self.handle.id)
            .expect("running lease retained");
        let end = if record.owner != self.handle.owner || record.scope != self.scope {
            End::Abandoned
        } else if record.phase == Phase::Cancelling {
            End::Cancelled
        } else if Instant::now() >= record.context.deadline {
            End::Expired
        } else if record.phase != Phase::Running {
            End::Abandoned
        } else {
            requested
        };
        record.terminal(end);
        self.done = true;
        end
    }
}
impl Drop for Lease<'_> {
    fn drop(&mut self) {
        if !self.done {
            self.finish(End::Abandoned);
        }
    }
}
struct OwnedWork<'a, F: Future> {
    work: Option<Pin<Box<F>>>,
    lease: Lease<'a>,
}
impl<F: Future> Drop for OwnedWork<'_, F> {
    fn drop(&mut self) {
        // On caller drop/abort/panic, provider/collection destruction precedes Lease::drop.
        drop(self.work.take());
    }
}

#[cfg(test)]
impl LthResearchRuntime {
    pub(super) fn reserve_with_reference(
        &self,
        prepared: PreparedLthPreview,
        as_of: DateTime<Utc>,
    ) -> CmdResult<Handle> {
        self.reserve_at(prepared, as_of)
    }
    pub(super) fn is_terminal(&self, handle: &Handle) -> bool {
        self.lock()
            .get(&handle.id)
            .is_some_and(|r| matches!(r.phase, Phase::Terminal(_)))
    }
}
#[cfg(test)]
impl Handle {
    pub(super) fn wrong_owner(&self) -> Self {
        Self {
            id: self.id,
            owner: Uuid::new_v4(),
        }
    }
}

#[cfg(test)]
pub(super) fn fixture() -> PreparedLthPreview {
    super::prepare_lth_preview(super::LeveragedTrendHoldPreviewInput {
        symbol: " koru ".into(), interval: Some("M1".into()), count: Some(20),
        params: serde_json::json!({"entries":[{"leveraged_symbol":"KORU","quantity":1,"is_overseas":true}]}),
        assumptions: Default::default(), expected_profile_id: Some("offline-A".into()),
        expected_broker_account_id: Some("offline-account".into()),
    }, AccountProfile { id: "offline-A".into(), broker_id: BrokerId::Toss,
        name: "offline fixture".into(), is_paper_trading: true, live_trading_consent: false,
        app_key: "offline-unused".into(), app_secret: "offline-unused".into(),
        account_no: "offline-account".into(),
    }).expect("offline prepared fixture")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn failure_receipt_rejects_nonterminal_unknown_wrong_owner_and_mode() {
        let runtime = LthResearchRuntime::new();
        let scope = Scope::from_profile(&fixture().profile);
        let handle = runtime.reserve(fixture()).unwrap();
        let mut other = scope.clone();
        other.paper = !other.paper;
        let unknown = Handle {
            id: Uuid::new_v4(),
            owner: handle.owner,
        };
        for phase in [
            Phase::Reserved,
            Phase::Running,
            Phase::Cancelling,
            Phase::Terminal(End::Completed),
            Phase::Terminal(End::Cancelled),
        ] {
            runtime.lock().get_mut(&handle.id).unwrap().phase = phase;
            assert!(!runtime.terminal_failure_receipt(&handle, &scope));
        }
        for end in [End::Failed, End::Expired, End::Abandoned] {
            runtime.lock().get_mut(&handle.id).unwrap().terminal(end);
            assert!(runtime.terminal_failure_receipt(&handle, &scope));
            assert!(!runtime.terminal_failure_receipt(&handle.wrong_owner(), &scope));
            assert!(!runtime.terminal_failure_receipt(&unknown, &scope));
            assert!(!runtime.terminal_failure_receipt(&handle, &other));
        }
    }

    fn never_factory<'a>(
        calls: &'a AtomicUsize,
    ) -> impl FnOnce(PreparedLthPreview, Context) -> std::future::Ready<CmdResult<()>> + 'a {
        move |_, _| {
            calls.fetch_add(1, Ordering::SeqCst);
            std::future::ready(Ok(()))
        }
    }
    fn input_for(prepared: &PreparedLthPreview) -> super::super::LeveragedTrendHoldPreviewInput {
        super::super::LeveragedTrendHoldPreviewInput {
            symbol: prepared.symbol.clone(),
            interval: Some(prepared.interval.into()),
            count: Some(prepared.count),
            params: serde_json::from_str(&prepared.params_fingerprint).unwrap(),
            assumptions: prepared.assumptions.clone(),
            expected_profile_id: Some(prepared.profile.id.clone()),
            expected_broker_account_id: Some(prepared.profile.account_no.clone()),
        }
    }
    #[tokio::test]
    async fn actual_preview_entry_prepare_guards_and_two_shared_arc_calls_are_io_zero() {
        let runtime = std::sync::Arc::new(LthResearchRuntime::new());
        for kind in 0..7 {
            let prepared = fixture();
            let mut input = input_for(&prepared);
            let mut profile = prepared.profile;
            match kind {
                0 => profile.broker_id = BrokerId::Kis,
                1 => input.expected_profile_id = Some("offline-other".into()),
                2 => input.expected_broker_account_id = Some("offline-other".into()),
                3 => input.symbol = String::new(),
                4 => input.interval = Some("unsupported".into()),
                5 => input.params = serde_json::json!({"entries": "invalid"}),
                _ => {
                    input.interval = Some("D".into());
                    input.params["bollinger"] = serde_json::json!({"enabled":true});
                }
            }
            assert!(super::super::preview_leveraged_trend_hold_for_profile(
                input, profile, &runtime
            )
            .await
            .is_err());
            assert!(runtime.lock().is_empty());
        }
        let prepared = fixture();
        let scope = Scope::from_profile(&prepared.profile);
        let handle = runtime.reserve(prepared).unwrap();
        let first = runtime.clone();
        let second = runtime.clone();
        assert!(std::sync::Arc::ptr_eq(&first, &second));
        // Execute the production for_profile entry through two Arc clones; both see
        // the same active slot before the real adapter factory can execute. Native
        // Tauri dispatch and Axum routing remain static wiring proof, not this test.
        for caller in [&first, &second] {
            let prepared = fixture();
            let input = input_for(&prepared);
            let err = super::super::preview_leveraged_trend_hold_for_profile(
                input,
                prepared.profile,
                caller,
            )
            .await
            .err()
            .unwrap();
            assert!(err.message.contains("active_limit"));
        }
        runtime.cancel(&handle, &scope).await.unwrap();
    }
    #[tokio::test(start_paused = true)]
    async fn final_publication_rechecks_deadline_even_for_ready_work() {
        let runtime = LthResearchRuntime::new();
        let scope = Scope::from_profile(&fixture().profile);
        let handle = runtime.reserve(fixture()).unwrap();
        let err = runtime
            .run(&handle, &scope, |_, _| async {
                tokio::time::advance(Duration::from_secs(31)).await;
                Ok(())
            })
            .await
            .err()
            .unwrap();
        assert!(err.message.contains("expired"));
        assert!(runtime.is_terminal(&handle));
        let next = runtime.reserve(fixture()).unwrap();
        runtime.cancel(&next, &scope).await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn reserve_owner_scope_single_consume_expiry_and_unknown_are_io_zero() {
        let runtime = LthResearchRuntime::new();
        let prepared = fixture();
        let scope = Scope::from_profile(&prepared.profile);
        let handle = runtime.reserve(prepared).unwrap();
        let calls = AtomicUsize::new(0);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(runtime.reserve(fixture()).is_err());
        assert!(runtime
            .run(&handle.wrong_owner(), &scope, never_factory(&calls))
            .await
            .is_err());
        for index in 0..4 {
            let mut other = scope.clone();
            match index {
                0 => other.broker = BrokerId::Kis,
                1 => other.profile.push('x'),
                2 => other.account.push('x'),
                _ => other.paper = !other.paper,
            }
            assert!(runtime
                .run(&handle, &other, never_factory(&calls))
                .await
                .is_err());
            assert!(runtime.cancel(&handle, &other).await.is_err());
        }
        assert!(runtime.cancel(&handle.wrong_owner(), &scope).await.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        tokio::time::advance(Duration::from_secs(31)).await;
        assert!(runtime
            .run(&handle, &scope, never_factory(&calls))
            .await
            .is_err());
        assert!(runtime.is_terminal(&handle));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            runtime.cancel(&handle, &scope).await.unwrap(),
            CancelAck::AlreadyTerminal
        );
        tokio::time::advance(Duration::from_secs(61)).await;
        assert!(runtime
            .run(&handle, &scope, never_factory(&calls))
            .await
            .is_err());
        assert!(runtime.cancel(&handle, &scope).await.is_err());
        assert!(runtime.lock().is_empty());
        let next = runtime.reserve(fixture()).unwrap();
        runtime
            .run(&next, &scope, never_factory(&calls))
            .await
            .unwrap();
        assert!(runtime
            .run(&next, &scope, never_factory(&calls))
            .await
            .is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[tokio::test(start_paused = true)]
    async fn cancel_before_run_terminal_and_scope_aba_do_not_recreate_old_jobs() {
        let runtime = LthResearchRuntime::new();
        let scope = Scope::from_profile(&fixture().profile);
        let old = runtime.reserve(fixture()).unwrap();
        assert_eq!(
            runtime.cancel(&old, &scope).await.unwrap(),
            CancelAck::Quiescent
        );
        assert_eq!(
            runtime.cancel(&old, &scope).await.unwrap(),
            CancelAck::AlreadyTerminal
        );
        let calls = AtomicUsize::new(0);
        assert!(runtime
            .run(&old, &scope, never_factory(&calls))
            .await
            .is_err());
        let mut other = fixture();
        other.profile.id = "offline-B".into();
        let scope_b = Scope::from_profile(&other.profile);
        let b = runtime.reserve(other).unwrap();
        runtime.cancel(&b, &scope_b).await.unwrap();
        let new = runtime.reserve(fixture()).unwrap();
        assert!(runtime
            .run(&old, &scope, never_factory(&calls))
            .await
            .is_err());
        assert_eq!(
            runtime.cancel(&old, &scope).await.unwrap(),
            CancelAck::AlreadyTerminal
        );
        assert!(!runtime.is_terminal(&new));
        runtime
            .run(&new, &scope, never_factory(&calls))
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[tokio::test(start_paused = true)]
    async fn retained_cap_ttl_and_error_cleanup_are_bounded() {
        let runtime = LthResearchRuntime {
            records: Mutex::new(HashMap::new()),
            policy: Policy {
                retained: 2,
                ttl: Duration::from_secs(2),
                budget: Duration::from_secs(1),
            },
        };
        let scope = Scope::from_profile(&fixture().profile);
        let a = runtime.reserve(fixture()).unwrap();
        runtime
            .run(&a, &scope, |_, _| async {
                Err::<(), _>(error("offline_failure"))
            })
            .await
            .err()
            .unwrap();
        assert!(runtime.is_terminal(&a));
        let b = runtime.reserve(fixture()).unwrap();
        runtime.cancel(&b, &scope).await.unwrap();
        assert_eq!(runtime.lock().len(), 2);
        assert!(runtime.reserve(fixture()).is_err());
        tokio::time::advance(Duration::from_secs(3)).await;
        let c = runtime.reserve(fixture()).unwrap();
        assert_eq!(runtime.lock().len(), 1);
        runtime.cancel(&c, &scope).await.unwrap();
        // Verify the production default cap as well, without spawning any work.
        let default = LthResearchRuntime::new();
        for _ in 0..16 {
            let h = default.reserve(fixture()).unwrap();
            default.cancel(&h, &scope).await.unwrap();
        }
        assert_eq!(default.lock().len(), 16);
        assert!(default.reserve(fixture()).is_err());
    }
    #[tokio::test]
    async fn factory_panic_and_running_work_panic_release_slot() {
        let runtime = std::sync::Arc::new(LthResearchRuntime::new());
        let scope = Scope::from_profile(&fixture().profile);
        let h = runtime.reserve(fixture()).unwrap();
        let r = runtime.clone();
        let s = scope.clone();
        let copy = h.clone();
        let task = tokio::spawn(async move {
            r.run(&copy, &s, |_, _| -> std::future::Ready<CmdResult<()>> {
                panic!("offline factory panic")
            })
            .await
        });
        assert!(task.await.err().unwrap().is_panic());
        assert!(runtime.is_terminal(&h));
        let h = runtime.reserve(fixture()).unwrap();
        let r = runtime.clone();
        let s = scope.clone();
        let copy = h.clone();
        let task = tokio::spawn(async move {
            r.run(&copy, &s, |_, _| async {
                panic!("offline work panic");
                #[allow(unreachable_code)]
                Ok::<(), CmdError>(())
            })
            .await
        });
        assert!(task.await.err().unwrap().is_panic());
        assert!(runtime.is_terminal(&h));
        let next = runtime.reserve(fixture()).unwrap();
        runtime.cancel(&next, &scope).await.unwrap();
    }
}
