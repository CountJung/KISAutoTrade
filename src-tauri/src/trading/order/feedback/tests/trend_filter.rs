//! 실제 소유 원장 caller를 통과하는 합성 fixture. provider 호출 없음.
use super::*;
#[tokio::test]
async fn trend_filter_feedback_cold_restore_unknown_and_repeated_partial_keep_streak() {
    for known in [false, true] {
        let f = Fixture::new("http://127.0.0.1:1".into()).await;
        f.begin(0).await;
        let id = "trend_filter_feedback";
        let cfg=StrategyConfig::new(id,"fixture",true,vec!["005930".into()],7,serde_json::json!({"long_period":20,"short_period":1,"mid_period":2,"exit_confirm_observations":2})).with_scope(BrokerId::Kis,Some(f.profile.account_no.clone()));
        let k = key(&f, id, BrokerMarket::Kr, "005930");
        let mut ledger = f
            .manager
            .lock()
            .await
            .load_ownership_ledger()
            .await
            .unwrap();
        buy(&mut ledger, k.clone(), "actual-fill", 3, 100);
        let checkpoint = serde_json::json!({"version":1,"kind":"trendFilter","ageKnown":known,"heldObservations":if known {12} else {0},"candidateEntry":false,"observedPeak":null,"profitArmed":false});
        ledger
            .set_position_metadata(&k, checkpoint.clone())
            .unwrap();
        f.manager
            .lock()
            .await
            .save_ownership_ledger(&ledger)
            .await
            .unwrap();
        let m = manager(&f, vec![cfg.clone()]);
        m.lock().await.initialize_historical("005930", &[100; 20]);
        OrderManager::sync_strategy_ownership_shared(&f.manager, &m)
            .await
            .unwrap();
        let read = || scope(&f.profile);
        assert_eq!(
            m.lock().await.owned_position_metadata_for_strategy(
                id,
                &read(),
                BrokerMarket::Kr,
                "005930"
            ),
            Some(checkpoint)
        );
        // had_metadata를 먼저 확인하는 실제 caller: 동량 publication이 cold restore로 취급되면 매번 streak=0이 된다.
        assert!(m.lock().await.on_tick("005930", 90, 0).is_empty());
        OrderManager::sync_strategy_ownership_shared(&f.manager, &m)
            .await
            .unwrap();
        let signals = m.lock().await.on_tick("005930", 80, 0);
        assert!(matches!(
            &signals[0].signal,
            Signal::Sell { quantity: 3, .. }
        ));
        {
            let mut manager = m.lock().await;
            let revision = manager.history_generation();
            manager.complete_signal(id, "005930", revision);
        }
        let mut ledger = f
            .manager
            .lock()
            .await
            .load_ownership_ledger()
            .await
            .unwrap();
        ledger
            .apply_cumulative(&OwnershipFill {
                order_record_id: "partial".into(),
                key: k.clone(),
                side: OrderSide::Sell,
                ordered_quantity: 3,
                cumulative_quantity: Some(1),
                cumulative_notional: Some(80),
            })
            .unwrap();
        f.manager
            .lock()
            .await
            .save_ownership_ledger(&ledger)
            .await
            .unwrap();
        OrderManager::sync_strategy_ownership_shared(&f.manager, &m)
            .await
            .unwrap();
        let signals = m.lock().await.on_tick("005930", 70, 0);
        assert!(matches!(
            &signals[0].signal,
            Signal::Sell { quantity: 2, .. }
        ));
        OrderManager::sync_strategy_ownership_shared(&f.manager, &m)
            .await
            .unwrap();
        let meta = m
            .lock()
            .await
            .owned_position_metadata_for_strategy(id, &read(), BrokerMarket::Kr, "005930")
            .unwrap();
        assert_eq!(meta["ageKnown"], known);
        assert_eq!(meta["heldObservations"], if known { 15 } else { 0 });
        let cold = manager(&f, vec![cfg]);
        cold.lock()
            .await
            .initialize_historical("005930", &[100; 20]);
        OrderManager::sync_strategy_ownership_shared(&f.manager, &cold)
            .await
            .unwrap();
        assert!(cold.lock().await.on_tick("005930", 90, 0).is_empty());
        OrderManager::sync_strategy_ownership_shared(&f.manager, &cold)
            .await
            .unwrap();
        assert!(matches!(
            &cold.lock().await.on_tick("005930", 80, 0)[0].signal,
            Signal::Sell { quantity: 2, .. }
        ));
        let mut wrong = read();
        wrong.account_id.as_mut().unwrap().0 = "other".into();
        assert!(!cold
            .lock()
            .await
            .restore_owned_position_metadata_for_strategy(
                id,
                &wrong,
                BrokerMarket::Kr,
                "005930",
                &meta
            )
            .unwrap());
    }
}
