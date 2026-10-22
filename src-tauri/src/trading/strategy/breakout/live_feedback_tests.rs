use super::*;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 2, params)
}

#[test]
fn live_feedback_fifty_two_week_keeps_crossing_progress_without_optimistic_holdings() {
    let mut strategy = FiftyTwoWeekHighStrategy::new(config(
        serde_json::json!({"lookback_days":3,"stop_loss_pct":3.0}),
    ));
    strategy.initialize_ohlc(
        "SOXQ",
        &[OhlcCandle {
            open: 90,
            high: 100,
            low: 80,
            close: 90,
        }; 3],
    );
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 101, 0),
        Signal::Buy { .. }
    ));
    assert_eq!(strategy.states["SOXQ"].buy_price, None);
    assert_eq!(strategy.states["SOXQ"].prev_price, Some(101));
    strategy.sync_owned_position("SOXQ", 2, 100);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
    assert_eq!(strategy.states["SOXQ"].buy_price, Some(100));
    assert_eq!(strategy.states["SOXQ"].prev_price, Some(90)); // The stop tick still advances the crossing baseline.
}

#[test]
fn live_feedback_strong_close_consumes_candidate_only_after_buy_fill() {
    let mut strategy = StrongCloseStrategy::new(config(
        serde_json::json!({"threshold_pct":3.0,"stop_loss_pct":3.0}),
    ));
    strategy.initialize_candles("SOXQ", &[(100, 99)]);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 100, 0),
        Signal::Buy { .. }
    ));
    assert!(strategy.states["SOXQ"].owned_quantity == 0);
    assert!(strategy.states["SOXQ"].pending_buy);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 101, 0),
        Signal::Buy { .. }
    ));
    strategy.sync_owned_position("SOXQ", 1, 100);
    assert!(!strategy.states["SOXQ"].pending_buy);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
    assert!(strategy.states["SOXQ"].owned_quantity > 0);
    assert_eq!(strategy.states["SOXQ"].avg_price, 100);
}

#[test]
fn live_feedback_volatility_keeps_intraday_extremes_and_confirmed_position() {
    let mut strategy = VolatilityExpansionStrategy::new(config(
        serde_json::json!({"lookback_days":2,"expansion_factor":1.0,"stop_loss_pct":3.0}),
    ));
    strategy.apply_completed_daily_bars_for_market_day(
        "SOXQ",
        "2026-10-22",
        &[super::super::CompletedDailyBar {
            ohlc: OhlcCandle {
                open: 100,
                high: 105,
                low: 100,
                close: 100,
            },
            volume: None,
        }; 2],
    );
    strategy.on_trading_day_start("SOXQ", 100);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 110, 0),
        Signal::Buy { .. }
    ));
    assert_eq!(strategy.states["SOXQ"].owned_quantity, 0);
    assert_eq!(strategy.states["SOXQ"].day_high, 110);
    strategy.sync_owned_position("SOXQ", 2, 110);
    assert!(matches!(
        strategy.on_live_tick("SOXQ", 90, 0),
        Signal::Sell { .. }
    ));
    assert!(strategy.states["SOXQ"].owned_quantity > 0);
    assert_eq!(strategy.states["SOXQ"].day_low, 90);
    assert_eq!(strategy.states["SOXQ"].day_high, 110);
}
