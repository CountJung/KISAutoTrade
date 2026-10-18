use super::*;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
}

#[test]
fn fifty_two_week_requires_full_window_and_uses_last_completed_high_and_close() {
    let mut strategy = FiftyTwoWeekHighStrategy::new(config(serde_json::json!({})));
    let candle = OhlcCandle {
        open: 100,
        high: 200,
        low: 90,
        close: 100,
    };
    strategy.initialize_ohlc("SOXQ", &vec![candle; 251]);
    let readiness = strategy.history_readiness("SOXQ").unwrap();
    assert_eq!(readiness.required_bars, 252);
    assert_eq!(readiness.available_bars, 251);
    assert!(!readiness.ready);
    assert!(matches!(strategy.on_tick("SOXQ", 300, 0), Signal::Hold));
    let mut candles = vec![candle; 252];
    candles[251].high = 300;
    strategy.initialize_ohlc("SOXQ", &candles);
    assert!(strategy.history_readiness("SOXQ").unwrap().ready);
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(300));
    assert_eq!(strategy.states["SOXQ"].prev_price, Some(100));
    assert!(matches!(
        strategy.on_tick("SOXQ", 301, 0),
        Signal::Buy { .. }
    ));
    strategy.initialize_ohlc("SOXQ", &candles);
    assert_eq!(strategy.states["SOXQ"].buy_price, Some(301));
}

fn bar(high: u64, close: u64) -> OhlcCandle {
    OhlcCandle {
        open: close,
        high,
        low: close,
        close,
    }
}

fn strategy(lookback: usize) -> FiftyTwoWeekHighStrategy {
    FiftyTwoWeekHighStrategy::new(config(serde_json::json!({
        "lookback_days": lookback,
        "stop_loss_pct": 3.0,
    })))
}

fn complete_day(strategy: &mut FiftyTwoWeekHighStrategy, candle: OhlcCandle) {
    strategy.on_trading_day_start("SOXQ", candle.open);
    strategy.on_completed_candle("SOXQ", &candle);
}

#[test]
fn oldest_high_expires_after_one_completed_day_and_changes_next_day_signal() {
    let mut strategy = strategy(252);
    let mut history = vec![bar(200, 100); 252];
    history[0].high = 1_000;
    strategy.initialize_ohlc("SOXQ", &history);
    strategy.on_trading_day_start("SOXQ", 100);
    assert!(matches!(strategy.on_tick("SOXQ", 250, 0), Signal::Hold));
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(1_000));
    strategy.on_completed_candle("SOXQ", &bar(300, 100));
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(300));
    assert_eq!(strategy.states["SOXQ"].completed_highs.len(), 252);
    strategy.on_trading_day_start("SOXQ", 100);
    assert!(matches!(
        strategy.on_tick("SOXQ", 301, 0),
        Signal::Buy { .. }
    ));
}

#[test]
fn completed_high_not_close_sets_next_reference_without_lookahead() {
    let mut strategy = strategy(2);
    strategy.initialize_ohlc("SOXQ", &[bar(200, 100); 2]);
    let future = bar(1_000, 201);
    strategy.on_trading_day_start("SOXQ", future.open);
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(200));
    // The complete day's high must not enter the reference before close evaluation.
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &future, 10),
        Signal::Buy { .. }
    ));
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(200));
    strategy.sync_position("SOXQ", 0, 0);
    strategy.on_completed_candle("SOXQ", &future);
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(1_000));
    assert_eq!(strategy.states["SOXQ"].prev_price, Some(201));
    strategy.on_trading_day_start("SOXQ", 201);
    assert!(matches!(strategy.on_tick("SOXQ", 202, 0), Signal::Hold));
    assert!(matches!(
        strategy.on_tick("SOXQ", 1_001, 0),
        Signal::Buy { .. }
    ));
}

#[test]
fn many_intraday_prices_and_intraday_initializers_do_not_age_daily_highs() {
    let mut strategy = strategy(252);
    let mut history = vec![bar(200, 100); 252];
    history[0].high = 1_000;
    strategy.initialize_ohlc("SOXQ", &history);
    strategy.initialize_intraday_ohlc("SOXQ", &vec![bar(5_000, 4_000); 500]);
    strategy.initialize_intraday_prices("SOXQ", &vec![5_000; 500]);
    strategy.on_trading_day_start("SOXQ", 100);
    for price in [100, 1_001, 500].into_iter().cycle().take(3_000) {
        strategy.on_tick("SOXQ", price, 0);
        strategy.sync_position("SOXQ", 0, 0);
    }
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(1_000));
    assert_eq!(strategy.states["SOXQ"].completed_highs.len(), 252);
    strategy.on_completed_candle("SOXQ", &bar(600, 500));
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(600));
    // A repeated completion without a new daily boundary cannot age the window twice.
    let once = strategy.states["SOXQ"].completed_highs.clone();
    strategy.on_completed_candle("SOXQ", &bar(9_000, 8_000));
    assert_eq!(strategy.states["SOXQ"].completed_highs, once);
}

#[test]
fn missing_history_becomes_ready_only_after_252_completed_daily_highs() {
    let mut strategy = strategy(252);
    strategy.initialize_ohlc("SOXQ", &[bar(200, 100); 251]);
    strategy.on_trading_day_start("SOXQ", 100);
    for _ in 0..1_000 {
        assert!(matches!(strategy.on_tick("SOXQ", 201, 1), Signal::Hold));
    }
    assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    assert_eq!(
        strategy.history_readiness("SOXQ").unwrap().available_bars,
        251
    );
    strategy.on_completed_candle("SOXQ", &bar(201, 100));
    assert!(strategy.history_readiness("SOXQ").unwrap().ready);
    assert_eq!(
        strategy.history_readiness("SOXQ").unwrap().available_bars,
        252
    );
    assert!(matches!(
        strategy.on_tick("SOXQ", 202, 0),
        Signal::Buy { .. }
    ));
}

#[test]
fn reinitialization_keeps_positions_and_stop_loss_before_readiness() {
    let mut strategy = strategy(252);
    strategy.initialize_ohlc("SOXQ", &[bar(200, 100); 252]);
    strategy.sync_position("SOXQ", 1, 100);
    complete_day(&mut strategy, bar(210, 100));
    assert_eq!(strategy.states["SOXQ"].buy_price, Some(100));
    strategy.initialize_ohlc("SOXQ", &[]);
    assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    assert_eq!(strategy.states["SOXQ"].buy_price, Some(100));
    assert_eq!(strategy.can_evaluate_next_tick("SOXQ"), Some(true));
    assert!(matches!(
        strategy.on_tick("SOXQ", 96, 0),
        Signal::Sell { .. }
    ));
    strategy.sync_position("SOXQ", 1, 100);
    strategy.initialize_historical("SOXQ", &[200; 251]);
    assert_eq!(strategy.states["SOXQ"].buy_price, Some(100));
    assert!(matches!(
        strategy.on_tick("SOXQ", 96, 0),
        Signal::Sell { .. }
    ));
}

#[test]
fn initialization_keeps_last_window_and_daily_bookkeeping_is_symbol_scoped() {
    let mut strategy = strategy(2);
    strategy.initialize_ohlc("SOXQ", &[bar(9_000, 100), bar(200, 100), bar(300, 100)]);
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(300));
    strategy.on_trading_day_start("OTHER", 100);
    strategy.on_completed_candle("OTHER", &bar(9_000, 100));
    assert_eq!(strategy.states.len(), 1);
    strategy.on_completed_candle("SOXQ", &bar(9_000, 100));
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(300));
    complete_day(&mut strategy, bar(400, 100));
    complete_day(&mut strategy, bar(100, 100));
    complete_day(&mut strategy, bar(100, 100));
    assert_eq!(strategy.states["SOXQ"].high_52w, Some(100));
    assert_eq!(strategy.states["SOXQ"].completed_highs.len(), 2);
}
