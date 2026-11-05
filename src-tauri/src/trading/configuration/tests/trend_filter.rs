use super::*;
#[tokio::test]
async fn trend_filter_strict_save_rejects_before_persistence_and_roundtrips_options() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = state(&f, false).await;
    let id = "trend_filter_configuration";
    let cfg = StrategyConfig::new(
        id,
        "fixture",
        true,
        vec!["005930".into()],
        2,
        serde_json::json!({"long_period":200,"short_period":5,"mid_period":20}),
    )
    .with_scope(f.profile.broker_id, Some(f.profile.account_no.clone()));
    {
        let mut m = s.strategies.lock().await;
        *m = StrategyManager::new();
        m.add(build_strategy(cfg.clone()));
        m.set_history_execution_identity(
            test_support::scope(&f.profile),
            Some(f.profile.id.clone()),
        );
    }
    s.store.save(&f.profile.id, &[cfg]).await.unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    for bad in [
        serde_json::json!({"exit_confirm_observations":null}),
        serde_json::json!({"exit_confirm_observations":0.5}),
        serde_json::json!({"exit_confirm_observations":0}),
        serde_json::json!({"profit_activation_pct":5}),
        serde_json::json!({"profit_trailing_pct":100}),
        serde_json::json!({"max_hold_observations":false}),
        serde_json::json!({"max_hold_observations":1000001}),
    ] {
        let mut r = request(&f);
        r.id = id.into();
        r.params = Some(bad);
        let c = calls.clone();
        let e = run_update(s.clone(), r, move |_, _| {
            c.fetch_add(1, Ordering::SeqCst);
            async { Ok(()) }
        })
        .await
        .unwrap_err();
        assert_eq!(e.code, "INVALID_STRATEGY_PARAMS");
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let mut r = request(&f);
    r.id = id.into();
    r.params = Some(
        serde_json::json!({"long_period":200,"short_period":5,"mid_period":20,"exit_confirm_observations":2.0,"profit_activation_pct":5,"profit_trailing_pct":3,"max_hold_observations":63.0}),
    );
    let expected = r.params.clone().unwrap();
    let result = update_strategy_configuration(s.clone(), r).await.unwrap();
    assert_eq!(result.params, expected);
    assert_eq!(
        s.store.load(&f.profile.id).await.unwrap()[0].params,
        expected
    );
}
