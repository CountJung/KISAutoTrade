use super::*;
const RSI_ID: &str = "rsi_configuration";
async fn rsi_state(f: &Fixture, held: bool) -> ConfigurationUpdateState {
    let s = state(f, false).await;
    let cfg=StrategyConfig::new(RSI_ID,"fixture",true,vec!["005930".into()],2,serde_json::json!({"period":2,"oversold":30.0,"overbought":70.0,"stop_loss_pct":3.0,"max_hold_observations":63})).with_scope(f.profile.broker_id,Some(f.profile.account_no.clone()));
    {
        let mut m = s.strategies.lock().await;
        *m = StrategyManager::new();
        m.add(build_strategy(cfg.clone()));
        m.set_history_execution_identity(
            test_support::scope(&f.profile),
            Some(f.profile.id.clone()),
        );
        m.initialize_historical("005930", &[100, 90, 80]);
        if held {
            assert_eq!(m.on_tick("005930", 100, 0).len(), 1);
            let generation = m.history_generation();
            m.complete_signal(RSI_ID, "005930", generation);
            m.sync_position("005930", 3, 100);
        }
    }
    s.store.save(&f.profile.id, &[cfg]).await.unwrap();
    s
}
fn update(f: &Fixture) -> ConfigurationUpdateRequest {
    let mut r = request(f);
    r.id = RSI_ID.into();
    r.params = Some(
        serde_json::json!({"period":2,"oversold":30.0,"overbought":70.0,"stop_loss_pct":5.0,"long_trend_period":200.0,"max_hold_observations":63.0}),
    );
    r
}
#[tokio::test]
async fn rsi_optional_validation_precedes_save_and_valid_values_roundtrip() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = rsi_state(&f, false).await;
    let original = config(&s).await;
    let calls = Arc::new(AtomicUsize::new(0));
    for bad in [
        serde_json::json!({"long_trend_period":null}),
        serde_json::json!({"long_trend_period":513}),
        serde_json::json!({"max_hold_observations":0.5}),
        serde_json::json!({"max_hold_observations":1000001}),
        serde_json::json!({"stop_loss_pct":100}),
        serde_json::json!({"stop_loss_pct":"3"}),
    ] {
        let mut r = update(&f);
        r.params
            .as_mut()
            .unwrap()
            .as_object_mut()
            .unwrap()
            .extend(bad.as_object().unwrap().clone());
        let c = calls.clone();
        let error = run_update(s.clone(), r, move |_, _| {
            c.fetch_add(1, Ordering::SeqCst);
            async { Ok(()) }
        })
        .await
        .unwrap_err();
        assert_eq!(error.code, "INVALID_STRATEGY_PARAMS");
        assert_eq!(config(&s).await.params, original.params);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let r = update(&f);
    let expected = r.params.clone().unwrap();
    let result = update_strategy_configuration(s.clone(), r).await.unwrap();
    assert_eq!(result.params, expected);
    assert_eq!(
        s.store.load(&f.profile.id).await.unwrap()[0].params,
        expected
    );
}
#[tokio::test]
async fn rsi_save_replace_failure_rolls_back_without_losing_owned_basis_or_age() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = rsi_state(&f, true).await;
    let metadata = s
        .strategies
        .lock()
        .await
        .owned_position_metadata_for_strategy(
            RSI_ID,
            &test_support::scope(&f.profile),
            crate::broker::BrokerMarket::Kr,
            "005930",
        )
        .unwrap();
    let store = s.store.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    let error = run_update(s.clone(), update(&f), move |profile, configs| {
        let (store, c) = (store.clone(), c.clone());
        async move {
            store.save(&profile, &configs).await?;
            if c.fetch_add(1, Ordering::SeqCst) == 0 {
                anyhow::bail!("synthetic RSI save failure after replacement");
            }
            Ok(())
        }
    })
    .await
    .unwrap_err();
    assert_eq!(error.code, "PERSISTENCE_ERROR");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(config(&s).await.params["stop_loss_pct"], 3.0);
    assert_eq!(
        s.store.load(&f.profile.id).await.unwrap()[0].params["stop_loss_pct"],
        3.0
    );
    assert_eq!(
        s.strategies
            .lock()
            .await
            .owned_position_metadata_for_strategy(
                RSI_ID,
                &test_support::scope(&f.profile),
                crate::broker::BrokerMarket::Kr,
                "005930"
            )
            .unwrap(),
        metadata
    );
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 97, 0)[0].signal,
        Signal::Sell { quantity: 3, .. }
    ));
}
#[tokio::test]
async fn rsi_held_target_removal_is_rejected_and_successful_edit_preserves_age() {
    let f = Fixture::new("http://127.0.0.1:1".into()).await;
    let s = rsi_state(&f, true).await;
    let mut r = update(&f);
    r.target_symbols = Some(vec![]);
    assert_eq!(
        update_strategy_configuration(s.clone(), r)
            .await
            .unwrap_err()
            .code,
        "HELD_TARGET_REMOVAL"
    );
    let before = s
        .strategies
        .lock()
        .await
        .owned_position_metadata_for_strategy(
            RSI_ID,
            &test_support::scope(&f.profile),
            crate::broker::BrokerMarket::Kr,
            "005930",
        )
        .unwrap();
    update_strategy_configuration(s.clone(), update(&f))
        .await
        .unwrap();
    assert_eq!(
        s.strategies
            .lock()
            .await
            .owned_position_metadata_for_strategy(
                RSI_ID,
                &test_support::scope(&f.profile),
                crate::broker::BrokerMarket::Kr,
                "005930"
            )
            .unwrap(),
        before
    );
    assert!(matches!(
        s.strategies.lock().await.on_tick("005930", 95, 0)[0].signal,
        Signal::Sell { quantity: 3, .. }
    ));
}
