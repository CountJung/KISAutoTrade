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
