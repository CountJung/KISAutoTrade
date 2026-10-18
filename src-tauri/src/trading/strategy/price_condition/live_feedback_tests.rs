use super::*;

fn strategy() -> PriceConditionStrategy {
    PriceConditionStrategy::new(StrategyConfig::new(
        "price_condition",
        "PC",
        true,
        vec![],
        2,
        serde_json::json!({"symbols":[{"symbol":"005930","symbol_name":"fixture","quantity":2,
        "buy_trigger_price":100.0,"take_profit_pct":5.0,"stop_loss_pct":3.0}]}),
    ))
}

#[test]
fn live_feedback_price_condition_changes_position_only_after_confirmed_fill() {
    let mut live = strategy();
    assert!(matches!(
        live.on_live_tick("005930", 99, 0),
        Signal::Buy { .. }
    ));
    assert_eq!(live.has_held_position("005930"), Some(false));
    live.sync_owned_position("005930", 1, 98);
    assert!(matches!(
        live.on_live_tick("005930", 90, 0),
        Signal::Sell { .. }
    ));
    assert_eq!(live.positions["005930"], (true, Some(98)));
    live.sync_owned_position("005930", 1, 97);
    assert_eq!(live.positions["005930"], (true, Some(97)));
    live.sync_owned_position("005930", 0, 0);
    assert_eq!(live.has_held_position("005930"), Some(false));
    let mut replay = strategy();
    assert!(matches!(
        replay.on_tick("005930", 99, 0),
        Signal::Buy { .. }
    ));
    assert_eq!(replay.has_held_position("005930"), Some(true));
}
