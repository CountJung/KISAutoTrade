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

#[test]
fn atomic_daily_default_hooks_forward_established_behavior_exactly_once() {
    struct Probe {
        config: StrategyConfig,
        calls: Vec<&'static str>,
        bars: Vec<OhlcCandle>,
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
            true
        }
        fn set_enabled(&mut self, _: bool) {}
        fn on_tick(&mut self, _: &str, _: u64, _: u64) -> Signal {
            Signal::Hold
        }
        fn reset(&mut self) {}
        fn initialize_candles(&mut self, _: &str, _: &[(u64, u64)]) {
            self.calls.push("candles");
        }
        fn initialize_ohlc(&mut self, _: &str, bars: &[OhlcCandle]) {
            self.calls.push("ohlc");
            self.bars = bars.to_vec();
        }
        fn initialize_range_data(&mut self, _: &str, _: &[u64]) {
            self.calls.push("ranges");
        }
        fn apply_completed_daily_history(&mut self, _: &str, bars: &[OhlcCandle]) {
            self.calls.push("apply");
            self.bars = bars.to_vec();
        }
        fn on_completed_candle(&mut self, _: &str, bar: &OhlcCandle) {
            self.calls.push("complete");
            self.bars.push(*bar);
        }
    }
    let mut p = Probe {
        config: StrategyConfig::new(
            "probe",
            "probe",
            true,
            vec!["TEST".into()],
            1,
            serde_json::json!({}),
        ),
        calls: vec![],
        bars: vec![],
    };
    let bars = [CompletedDailyBar {
        ohlc: OhlcCandle {
            open: 100,
            high: 200,
            low: 90,
            close: 110,
        },
        volume: Some(0),
    }];
    initialize_strategy_warmup_with_daily_bars(&mut p, "TEST", &bars, &[]);
    assert_eq!(p.calls, ["candles", "ohlc", "ranges"]);
    assert_eq!(p.bars, [bars[0].ohlc]);
    p.apply_completed_daily_bars("TEST", &bars);
    assert_eq!(p.calls, ["candles", "ohlc", "ranges", "apply"]);
    p.on_completed_daily_bar("TEST", &bars[0]);
    assert_eq!(p.calls, ["candles", "ohlc", "ranges", "apply", "complete"]);
    assert_eq!(p.bars.len(), 2);
}
#[test]
fn raw_daily_volume_parser_preserves_unknown_and_zero_without_float_loss() {
    for raw in [
        "",
        "1.0",
        "1e3",
        "+1",
        "-1",
        "null",
        "NaN",
        "1,,000",
        "1,00",
        "1234,000",
        "18446744073709551616",
    ] {
        assert_eq!(parse_completed_daily_volume(raw), None, "{raw}");
    }
    for (raw, n) in [
        ("0", 0),
        (" 1,000 ", 1000),
        ("18446744073709551615", u64::MAX),
    ] {
        assert_eq!(parse_completed_daily_volume(raw), Some(n));
    }
}
