use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

fn input() -> LthResearchReserveInput {
    let prepared = lth_runtime::fixture();
    LthResearchReserveInput {
        preview: LeveragedTrendHoldPreviewInput {
            symbol: prepared.symbol,
            interval: Some("1m".into()),
            count: Some(20),
            params: serde_json::from_str(&prepared.params_fingerprint).unwrap(),
            assumptions: prepared.assumptions,
            expected_profile_id: Some(prepared.profile.id),
            expected_broker_account_id: Some(prepared.profile.account_no),
        },
        expected_broker_id: BrokerId::Toss,
        expected_paper_trading: true,
    }
}

#[test]
fn public_reserve_requires_mode_and_full_scope_without_io() {
    let runtime = LthResearchRuntime::new();
    let valid = input();
    let wire = serde_json::json!({
        "preview": { "symbol": valid.preview.symbol, "interval": "1m", "count": 20,
            "params": valid.preview.params, "expectedProfileId": "offline-A", "expectedBrokerAccountId": "offline-account" },
        "expectedBrokerId": "toss"
    });
    let missing_mode = serde_json::from_value::<LthResearchReserveInput>(wire)
        .err()
        .unwrap();
    assert!(missing_mode.to_string().contains("expectedPaperTrading"));
    for kind in 0..8 {
        let mut input = input();
        match kind {
            0 => input.expected_paper_trading = false,
            1 => input.expected_broker_id = BrokerId::Kis,
            2 => input.preview.expected_profile_id = None,
            3 => input.preview.expected_broker_account_id = None,
            4 => input.preview.expected_profile_id = Some("other".into()),
            5 => input.preview.interval = Some("1d".into()),
            6 => input.preview.params["entries"][0]["is_overseas"] = false.into(),
            _ => input.preview.symbol.clear(),
        }
        assert!(
            reserve_lth_research_for_profile(input, lth_runtime::fixture().profile, &runtime)
                .is_err()
        );
    }
    let handle =
        reserve_lth_research_for_profile(input(), lth_runtime::fixture().profile, &runtime)
            .unwrap();
    let value = serde_json::to_value(handle).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 2);
    assert!(value.get("jobId").is_some());
    assert!(value.get("owner").is_some());
}

#[tokio::test(start_paused = true)]
async fn public_handles_strict_uuid_owner_expiry_reuse_and_mode_are_factory_zero() {
    let runtime = LthResearchRuntime::new();
    let handle =
        reserve_lth_research_for_profile(input(), lth_runtime::fixture().profile, &runtime)
            .unwrap();
    let internal = handle.decode().unwrap();
    let scope = Scope::from_profile(&lth_runtime::fixture().profile);
    let calls = AtomicUsize::new(0);
    let make = |_, _| {
        calls.fetch_add(1, Ordering::SeqCst);
        std::future::ready(Ok(()))
    };
    for invalid in [
        "",
        "00000000000040008000000000000001",
        "urn:uuid:00000000-0000-4000-8000-000000000001",
    ] {
        assert!(Handle::from_wire(invalid, &handle.owner).is_err());
    }
    let wrong = Handle::from_wire(&handle.job_id, &uuid::Uuid::new_v4().to_string()).unwrap();
    assert!(runtime.run(&wrong, &scope, make).await.is_err());
    let unknown = Handle::from_wire(&uuid::Uuid::new_v4().to_string(), &handle.owner).unwrap();
    assert!(runtime.run(&unknown, &scope, make).await.is_err());
    let mut profile = lth_runtime::fixture().profile;
    profile.is_paper_trading = false;
    assert!(runtime
        .run(&internal, &Scope::from_profile(&profile), make)
        .await
        .is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    tokio::time::advance(std::time::Duration::from_secs(31)).await;
    assert!(runtime.run(&internal, &scope, make).await.is_err());
    assert!(cancel_lth_research_owned(handle, &runtime).await.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let next = reserve_lth_research_for_profile(input(), lth_runtime::fixture().profile, &runtime)
        .unwrap();
    let next = next.decode().unwrap();
    runtime.run(&next, &scope, make).await.unwrap();
    assert!(runtime.run(&next, &scope, make).await.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(serde_json::from_value::<LthResearchHandle>(
        serde_json::json!({"jobId": next.to_wire().0, "owner":next.to_wire().1, "count":200})
    )
    .is_err());
}

#[tokio::test]
async fn public_cancel_uses_captured_owner_after_scope_switch_and_waits_for_drop() {
    let runtime = Arc::new(LthResearchRuntime::new());
    let handle =
        reserve_lth_research_for_profile(input(), lth_runtime::fixture().profile, &runtime)
            .unwrap();
    let internal = handle.decode().unwrap();
    let scope = Scope::from_profile(&lth_runtime::fixture().profile);
    let dropped = Arc::new(AtomicUsize::new(0));
    let (entered, pending) = tokio::sync::oneshot::channel();
    let (release, gate) = tokio::sync::oneshot::channel::<()>();
    let r = runtime.clone();
    let d = dropped.clone();
    struct DropWork(Arc<AtomicUsize>);
    impl Drop for DropWork {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let task = tokio::spawn(async move {
        r.run(&internal, &scope, |_, _| async move {
            let _owned = DropWork(d);
            entered.send(()).unwrap();
            let _ = gate.await;
            panic!("gate must never resume after cancellation ACK");
            #[allow(unreachable_code)]
            Ok::<(), CmdError>(())
        })
        .await
    });
    pending.await.unwrap();
    // No active-profile parameter exists on this boundary. A different current
    // selection therefore cannot reassign or reject the captured old capability.
    let mut other = lth_runtime::fixture().profile;
    other.id = "B".into();
    other.is_paper_trading = false;
    assert!(reserve_lth_research_for_profile(input(), other, &runtime).is_err());
    assert_eq!(
        cancel_lth_research_owned(handle, &runtime)
            .await
            .unwrap()
            .status,
        "quiescent"
    );
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    let _ = release.send(());
    tokio::task::yield_now().await;
    assert!(task.await.unwrap().is_err());
    assert!(
        reserve_lth_research_for_profile(input(), lth_runtime::fixture().profile, &runtime).is_ok()
    );
}

#[tokio::test]
async fn public_cancel_failed_unknown_and_wrong_owner_never_acknowledge_success() {
    let runtime = LthResearchRuntime::new();
    let handle =
        reserve_lth_research_for_profile(input(), lth_runtime::fixture().profile, &runtime)
            .unwrap();
    let internal = handle.decode().unwrap();
    let wrong = LthResearchHandle {
        job_id: handle.job_id.clone(),
        owner: uuid::Uuid::new_v4().to_string(),
    };
    assert!(cancel_lth_research_owned(wrong, &runtime).await.is_err());
    runtime
        .run(
            &internal,
            &Scope::from_profile(&lth_runtime::fixture().profile),
            |_, _| async { Err::<(), _>(rejected()) },
        )
        .await
        .err()
        .unwrap();
    assert!(cancel_lth_research_owned(handle, &runtime).await.is_err());
    let unknown = LthResearchHandle {
        job_id: uuid::Uuid::new_v4().to_string(),
        owner: uuid::Uuid::new_v4().to_string(),
    };
    assert!(cancel_lth_research_owned(unknown, &runtime).await.is_err());
}

#[tokio::test(start_paused = true)]
async fn public_expired_reservation_returns_terminal_failure_without_provider_factory() {
    let runtime = LthResearchRuntime::new();
    let handle =
        reserve_lth_research_for_profile(input(), lth_runtime::fixture().profile, &runtime)
            .unwrap();
    let internal = handle.decode().unwrap();
    for (candidate, profile) in [
        (internal.wrong_owner(), lth_runtime::fixture().profile),
        (
            Handle::from_wire(&uuid::Uuid::new_v4().to_string(), &handle.owner).unwrap(),
            lth_runtime::fixture().profile,
        ),
        (internal.clone(), {
            let mut p = lth_runtime::fixture().profile;
            p.is_paper_trading = false;
            p
        }),
    ] {
        let error = FORBID_PROVIDER_FACTORY
            .scope(
                (),
                run_lth_research_for_profile(
                    LthResearchHandle::encode(candidate),
                    profile,
                    &runtime,
                ),
            )
            .await
            .err()
            .unwrap();
        assert_eq!(error.code, "LTH_RUNTIME_REJECTED");
    }
    tokio::time::advance(std::time::Duration::from_secs(31)).await;
    // Expired reservation is rejected before runtime invokes the production factory.
    let error = FORBID_PROVIDER_FACTORY
        .scope(
            (),
            run_lth_research_for_profile(handle, lth_runtime::fixture().profile, &runtime),
        )
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, "LTH_RESEARCH_FAILED");
}

#[tokio::test]
async fn public_failed_runtime_is_terminal_failure_but_never_cancel_ack() {
    let runtime = LthResearchRuntime::new();
    let handle =
        reserve_lth_research_for_profile(input(), lth_runtime::fixture().profile, &runtime)
            .unwrap();
    let internal = handle.decode().unwrap();
    let scope = Scope::from_profile(&lth_runtime::fixture().profile);
    runtime
        .run(&internal, &scope, |_, _| async { Err::<(), _>(rejected()) })
        .await
        .err()
        .unwrap();
    let error = FORBID_PROVIDER_FACTORY
        .scope(
            (),
            run_lth_research_for_profile(
                LthResearchHandle::encode(internal),
                lth_runtime::fixture().profile,
                &runtime,
            ),
        )
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, "LTH_RESEARCH_FAILED");
    assert!(cancel_lth_research_owned(handle, &runtime).await.is_err());
}
