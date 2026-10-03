use super::*;

fn config(params: serde_json::Value) -> StrategyConfig {
    StrategyConfig::new("test", "test", true, vec!["SOXQ".into()], 1, params)
}

fn candle(open: u64, high: u64, low: u64, close: u64) -> OhlcCandle {
    OhlcCandle {
        open,
        high,
        low,
        close,
    }
}

fn volatility(lookback: usize) -> VolatilityExpansionStrategy {
    VolatilityExpansionStrategy::new(config(serde_json::json!({
        "lookback_days": lookback,
        "expansion_factor": 2.0,
        "stop_loss_pct": 3.0,
    })))
}

#[test]
fn strong_close_uses_each_completed_day_only_on_the_following_day() {
    let mut strategy = StrongCloseStrategy::new(config(serde_json::json!({})));
    let strong = candle(90, 100, 85, 99);
    let weak = candle(100, 110, 90, 95);
    strategy.on_trading_day_start("SOXQ", strong.open);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &strong, 1),
        Signal::Hold
    ));
    assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    strategy.on_completed_candle("SOXQ", &strong);
    assert!(strategy.history_readiness("SOXQ").unwrap().ready);
    strategy.on_trading_day_start("SOXQ", weak.open);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &weak, 1),
        Signal::Buy { .. }
    ));
    // 차단된 매수의 실제 수량 0 피드백을 먼저 반영한다.
    strategy.sync_position("SOXQ", 0, 0);
    strategy.on_completed_candle("SOXQ", &weak);
    assert!(!strategy.states["SOXQ"].pending_buy);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &strong, 1),
        Signal::Hold
    ));
    strategy.on_completed_candle("SOXQ", &strong);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &strong, 1),
        Signal::Buy { .. }
    ));
}

#[test]
fn strong_close_completed_day_uses_actual_position_and_keeps_stop_loss() {
    let mut strategy = StrongCloseStrategy::new(config(serde_json::json!({})));
    strategy.sync_position("SOXQ", 1, 100);
    strategy.on_completed_candle("SOXQ", &candle(95, 100, 90, 99));
    assert!(strategy.states["SOXQ"].in_position);
    assert_eq!(strategy.states["SOXQ"].entry_price, Some(100));
    assert!(!strategy.states["SOXQ"].pending_buy);
    strategy.on_trading_day_start("SOXQ", 95);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &candle(95, 99, 90, 94), 1),
        Signal::Sell { .. }
    ));
    // 체결되지 않은 손절은 보유 상태를 다시 복원하며 다음 날 매수를 예약하지 않는다.
    strategy.sync_position("SOXQ", 1, 100);
    strategy.on_completed_candle("SOXQ", &candle(94, 94, 94, 94));
    assert!(!strategy.states["SOXQ"].pending_buy);
    assert_eq!(strategy.states["SOXQ"].entry_price, Some(100));
}

#[test]
fn volatility_compares_current_full_range_against_prior_completed_window() {
    let mut strategy = volatility(2);
    strategy.initialize_range_data("SOXQ", &[10, 10]);
    let expanded = candle(100, 120, 95, 110);
    strategy.on_trading_day_start("SOXQ", expanded.open);
    assert_eq!(strategy.states["SOXQ"].day_high, 100);
    assert_eq!(strategy.states["SOXQ"].day_low, 100);
    assert!(matches!(strategy.on_tick("SOXQ", 100, 0), Signal::Hold));
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &expanded, 1),
        Signal::Buy { .. }
    ));
    // 당일 범위 25를 먼저 평균에 넣으면 임계값 35로 오판해 매수가 사라진다.
    assert_eq!(strategy.states["SOXQ"].avg_range, Some(10.0));
    strategy.sync_position("SOXQ", 0, 0);
    strategy.on_completed_candle("SOXQ", &expanded);
    assert_eq!(strategy.states["SOXQ"].avg_range, Some(17.5));
    assert_eq!(
        strategy.states["SOXQ"].completed_ranges,
        VecDeque::from([10, 25])
    );
}

#[test]
fn volatility_day_range_and_direction_are_independent_across_days() {
    let mut strategy = volatility(2);
    strategy.initialize_range_data("SOXQ", &[10, 10]);
    strategy.on_trading_day_start("SOXQ", 100);
    strategy.on_completed_candle("SOXQ", &candle(100, 150, 50, 100));
    strategy.on_trading_day_start("SOXQ", 200);
    assert_eq!(strategy.states["SOXQ"].day_open, Some(200));
    assert_eq!(strategy.states["SOXQ"].day_high, 200);
    assert_eq!(strategy.states["SOXQ"].day_low, 200);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &candle(200, 203, 198, 202), 1),
        Signal::Hold
    ));
    assert_eq!(strategy.states["SOXQ"].day_high, 203);
    assert_eq!(strategy.states["SOXQ"].day_low, 198);
    strategy.on_completed_candle("SOXQ", &candle(200, 203, 198, 202));
    strategy.on_trading_day_start("SOXQ", 300);
    // 큰 범위여도 당일 시가보다 낮은 종가는 상승 방향으로 판정하지 않는다.
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &candle(300, 350, 190, 299), 1),
        Signal::Hold
    ));
}

#[test]
fn volatility_zero_range_days_count_and_window_rolls_with_bounded_length() {
    let mut strategy = volatility(2);
    strategy.initialize_range_data("SOXQ", &[0, 0]);
    assert!(strategy.history_readiness("SOXQ").unwrap().ready);
    assert_eq!(strategy.states["SOXQ"].avg_range, Some(0.0));
    let expanded = candle(100, 101, 99, 101);
    strategy.on_trading_day_start("SOXQ", expanded.open);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &expanded, 1),
        Signal::Buy { .. }
    ));
    strategy.on_completed_candle("SOXQ", &expanded);
    assert_eq!(strategy.states["SOXQ"].avg_range, Some(1.0));
    strategy.on_completed_candle("SOXQ", &candle(100, 104, 100, 102));
    assert_eq!(strategy.states["SOXQ"].avg_range, Some(3.0));
    strategy.on_completed_candle("SOXQ", &candle(100, 100, 100, 100));
    assert_eq!(strategy.states["SOXQ"].avg_range, Some(2.0));
    assert_eq!(strategy.states["SOXQ"].history_bars, 2);
    assert_eq!(
        strategy.states["SOXQ"].completed_ranges,
        VecDeque::from([4, 0])
    );
}

#[test]
fn volatility_collects_missing_history_after_close_for_later_days() {
    let mut strategy = volatility(2);
    let normal = candle(100, 110, 100, 105);
    for available in 0..2 {
        strategy.on_trading_day_start("SOXQ", normal.open);
        assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
        assert!(matches!(
            strategy.on_daily_close_tick("SOXQ", &normal, 1),
            Signal::Hold
        ));
        assert_eq!(strategy.states["SOXQ"].history_bars, available);
        strategy.on_completed_candle("SOXQ", &normal);
    }
    assert!(strategy.history_readiness("SOXQ").unwrap().ready);
    strategy.on_trading_day_start("SOXQ", 100);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &candle(100, 125, 100, 110), 1),
        Signal::Buy { .. }
    ));
}

#[test]
fn volatility_day_start_completion_and_reinitialization_preserve_position() {
    let mut strategy = volatility(2);
    strategy.initialize_range_data("SOXQ", &[10, 20]);
    strategy.sync_position("SOXQ", 1, 100);
    strategy.on_trading_day_start("SOXQ", 99);
    strategy.on_completed_candle("SOXQ", &candle(99, 101, 95, 99));
    assert!(strategy.states["SOXQ"].in_position);
    assert_eq!(strategy.states["SOXQ"].entry_price, Some(100));
    strategy.initialize_range_data("SOXQ", &[]);
    assert!(!strategy.history_readiness("SOXQ").unwrap().ready);
    assert!(strategy.states["SOXQ"].completed_ranges.is_empty());
    strategy.on_trading_day_start("SOXQ", 95);
    assert!(strategy.states["SOXQ"].in_position);
    assert!(matches!(
        strategy.on_daily_close_tick("SOXQ", &candle(95, 96, 90, 94), 1),
        Signal::Sell { .. }
    ));
}

#[test]
fn daily_hooks_do_not_create_unknown_symbol_states() {
    let mut strong = StrongCloseStrategy::new(config(serde_json::json!({})));
    let mut vol = volatility(2);
    let bar = candle(100, 130, 90, 125);
    for strategy in [&mut strong as &mut dyn Strategy, &mut vol] {
        strategy.on_trading_day_start("OTHER", 100);
        assert!(matches!(
            strategy.on_daily_close_tick("OTHER", &bar, 1),
            Signal::Hold
        ));
        strategy.on_completed_candle("OTHER", &bar);
    }
    assert!(strong.states.is_empty());
    assert!(vol.states.is_empty());
}
