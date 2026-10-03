use crate::broker::{BrokerId, BrokerMarket};

use super::*;

#[test]
fn broker_position_snapshot_prevents_duplicate_price_condition_buy() {
    let params = PriceConditionParams {
        symbols: vec![PriceConditionSymbolConfig {
            symbol: "AAPL".into(),
            symbol_name: "Apple Inc.".into(),
            quantity: 2,
            buy_trigger_price: 200.0,
            sell_trigger_price: 0.0,
            take_profit_pct: 0.0,
            stop_loss_pct: 0.0,
            is_overseas: true,
        }],
    };
    let config = StrategyConfig::new(
        "price_condition_test",
        "가격 조건",
        true,
        vec!["AAPL".into()],
        1,
        serde_json::to_value(params).unwrap(),
    )
    .with_scope(BrokerId::Toss, Some("acct-1".into()));
    let mut manager = StrategyManager::new();
    manager.add(Box::new(PriceConditionStrategy::new(config)));

    manager.sync_position_for_broker(&BrokerPositionSnapshot {
        broker_id: BrokerId::Toss,
        market: BrokerMarket::Us,
        symbol: "AAPL".into(),
        quantity: 1,
        avg_price: 15_000,
    });

    let signals = manager.on_tick("AAPL", 18_000, 100);

    assert!(
        signals.is_empty(),
        "existing Toss holding should restore in-position state and block duplicate buy"
    );
}

#[test]
fn apply_saved_configs_for_scope_resets_previous_profile_and_stamps_scope() {
    let mut manager = StrategyManager::new();
    let config = StrategyConfig::new(
        "ma_cross_default",
        "이동평균 교차 전략",
        true,
        vec!["005930".into()],
        3,
        serde_json::to_value(MaCrossParams::default()).unwrap(),
    )
    .with_scope(BrokerId::Kis, Some("kis-1".into()));
    manager.add(Box::new(MovingAverageCrossStrategy::new(config)));

    manager.apply_saved_configs_for_scope(&[], BrokerId::Toss, Some("toss-1".into()));

    let cfg = manager
        .all_configs()
        .into_iter()
        .find(|cfg| cfg.id == "ma_cross_default")
        .expect("strategy should exist");
    assert!(!cfg.enabled);
    assert!(cfg.target_symbols.is_empty());
    assert!(cfg.matches_scope(BrokerId::Toss, Some("toss-1")));
}

#[test]
fn warmup_fifty_two_week_keeps_daily_highs_and_legacy_api_parity() {
    let config = StrategyConfig::new(
        "fifty_two_week_high",
        "52주 신고가",
        true,
        vec!["SOXQ".into()],
        10,
        serde_json::to_value(FiftyTwoWeekHighParams::default()).unwrap(),
    );
    let candles = vec![
        OhlcCandle {
            open: 10_000,
            high: 20_000,
            low: 9_000,
            close: 10_000,
        };
        253
    ];
    let mut manager = StrategyManager::new();
    manager.add(Box::new(FiftyTwoWeekHighStrategy::new(config.clone())));
    manager.initialize_warmup("SOXQ", &candles, &[]);
    let mut legacy = FiftyTwoWeekHighStrategy::new(config);
    legacy.initialize_historical("SOXQ", &vec![20_000; 252]);

    for price in [11_000, 15_000] {
        assert!(manager.on_tick("SOXQ", price, 0).is_empty());
        assert!(matches!(legacy.on_tick("SOXQ", price, 0), Signal::Hold));
    }
    let signals = manager.on_tick("SOXQ", 20_001, 0);
    assert_eq!(signals.len(), 1);
    assert!(matches!(signals[0].signal, Signal::Buy { .. }));
    assert!(matches!(
        legacy.on_tick("SOXQ", 20_001, 0),
        Signal::Buy { .. }
    ));
}

#[test]
fn warmup_dispatch_calls_historical_once_with_daily_closes() {
    struct Probe {
        config: StrategyConfig,
        calls: usize,
        prices: Vec<u64>,
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
        fn set_enabled(&mut self, enabled: bool) {
            self.config.enabled = enabled;
        }
        fn on_tick(&mut self, _: &str, _: u64, _: u64) -> Signal {
            Signal::Hold
        }
        fn initialize_historical(&mut self, symbol: &str, prices: &[u64]) {
            assert_eq!(symbol, "SOXQ");
            self.calls += 1;
            self.prices = prices.to_vec();
        }
        fn reset(&mut self) {}
    }
    let mut probe = Probe {
        config: StrategyConfig::new(
            "probe",
            "probe",
            true,
            vec!["SOXQ".into()],
            1,
            serde_json::json!({}),
        ),
        calls: 0,
        prices: Vec::new(),
    };
    let daily = [OhlcCandle {
        open: 100,
        high: 200,
        low: 80,
        close: 110,
    }];
    let intraday = [OhlcCandle {
        open: 300,
        high: 400,
        low: 250,
        close: 350,
    }];
    initialize_strategy_warmup(&mut probe, "SOXQ", &daily, &intraday);
    assert_eq!(probe.calls, 1);
    assert_eq!(probe.prices, [110]);
    initialize_strategy_warmup(&mut probe, "SOXQ", &[], &intraday);
    assert_eq!(probe.calls, 1);
    assert_eq!(probe.prices, [110]);
}
