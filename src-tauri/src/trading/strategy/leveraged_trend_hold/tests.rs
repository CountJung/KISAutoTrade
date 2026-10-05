use super::*;

pub(super) fn upward_candles() -> Vec<OhlcCandle> {
    (0..90)
        .map(|i| {
            let open = 100 + i;
            OhlcCandle {
                open,
                high: open + 3,
                low: open.saturating_sub(1),
                close: open + 2,
            }
        })
        .collect()
}

fn downward_candles() -> Vec<OhlcCandle> {
    (0..90)
        .map(|i| {
            let open = 200_u64.saturating_sub(i);
            OhlcCandle {
                open,
                high: open + 1,
                low: open.saturating_sub(3),
                close: open.saturating_sub(2),
            }
        })
        .collect()
}

pub(super) fn entry(symbol: &str) -> LeveragedTrendHoldEntry {
    LeveragedTrendHoldEntry {
        leveraged_symbol: symbol.into(),
        leveraged_symbol_name: symbol.into(),
        inverse_leveraged_symbol: "SOXS".into(),
        inverse_leveraged_symbol_name: "Legacy inverse".into(),
        base_symbols: vec!["SOXX".into()],
        base_symbol_names: HashMap::new(),
        base_symbol_roles: HashMap::new(),
        quantity: 3,
        inverse_quantity: 2,
        is_overseas: true,
    }
}

pub(super) fn strategy_with_params(params: LeveragedTrendHoldParams) -> LeveragedTrendHoldStrategy {
    let config = StrategyConfig::new(
        "leveraged_trend_hold_test",
        "레버리지 단일 티커 추세 테스트",
        true,
        Vec::new(),
        1,
        serde_json::to_value(params).unwrap(),
    );
    LeveragedTrendHoldStrategy::new(config)
}

#[test]
fn target_symbols_include_only_configured_tickers() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXL")],
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);

    assert_eq!(strategy.config.target_symbols, vec!["SOXL".to_string()]);
    strategy.initialize_intraday_ohlc("SOXX", &upward_candles());
    assert_eq!(strategy.on_tick("SOXX", 190, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("SOXS", 50, 100), Signal::Hold);
}

#[test]
fn buys_any_target_ticker_when_itself_trends_up() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXS")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        entry_window_end_min: 120,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("SOXS", &upward_candles());

    let signal = strategy.on_tick("SOXS", 192, 100);

    assert!(
        matches!(signal, Signal::Buy { symbol, quantity, .. } if symbol == "SOXS" && quantity == 3)
    );
}

#[test]
fn sells_target_when_its_own_trend_breaks() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXL")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        trailing_stop_pct: 99.0,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("SOXL", &downward_candles());
    strategy.sync_position("SOXL", 3, 300);
    if let Some(pos) = strategy.positions.get_mut("SOXL") {
        pos.high_water = Some(330);
        pos.held_observations = 3;
    }

    let signal = strategy.on_tick("SOXL", 320, 100);

    assert!(
        matches!(signal, Signal::Sell { symbol, quantity, reason } if symbol == "SOXL" && quantity == 3 && reason.contains("수익 보호 추세 청산"))
    );
}

#[test]
fn holds_when_protection_profit_has_not_activated() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXL")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        trailing_stop_pct: 1.0,
        trailing_activation_profit_pct: 2.0,
        breakeven_buffer_pct: 0.2,
        min_hold_observations: 1,
        initial_stop_loss_pct: 99.0,
        entry_failure_observations: 120,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("SOXL", &downward_candles());
    strategy.positions.insert(
        "SOXL".to_string(),
        LeveragedTrendHoldPosition {
            in_position: true,
            entry_price: Some(100),
            high_water: Some(101),
            held_observations: 1,
        },
    );

    let signal = strategy.on_tick("SOXL", 98, 100);

    assert_eq!(signal, Signal::Hold);
}

#[test]
fn sells_initial_stop_loss_before_profit_protection_activates() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXL")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        initial_stop_loss_pct: 1.0,
        trailing_activation_profit_pct: 2.0,
        entry_failure_observations: 120,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("SOXL", &downward_candles());
    strategy.positions.insert(
        "SOXL".to_string(),
        LeveragedTrendHoldPosition {
            in_position: true,
            entry_price: Some(100),
            high_water: Some(101),
            held_observations: 1,
        },
    );

    let signal = strategy.on_tick("SOXL", 98, 100);

    assert!(
        matches!(signal, Signal::Sell { symbol, reason, .. } if symbol == "SOXL" && reason.contains("초기 손절"))
    );
}

#[test]
fn sells_failed_rebound_when_activation_profit_never_appears() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXL")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        initial_stop_loss_pct: 5.0,
        trailing_activation_profit_pct: 2.0,
        entry_failure_observations: 3,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("SOXL", &downward_candles());
    strategy.positions.insert(
        "SOXL".to_string(),
        LeveragedTrendHoldPosition {
            in_position: true,
            entry_price: Some(100),
            high_water: Some(101),
            held_observations: 3,
        },
    );

    let signal = strategy.on_tick("SOXL", 99, 100);

    assert!(
        matches!(signal, Signal::Sell { symbol, reason, .. } if symbol == "SOXL" && reason.contains("반등 실패 손절"))
    );
}

#[test]
fn holds_failed_rebound_until_min_hold_observations_elapsed() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXL")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        initial_stop_loss_pct: 5.0,
        trailing_activation_profit_pct: 2.0,
        min_hold_observations: 5,
        entry_failure_observations: 3,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("SOXL", &downward_candles());
    strategy.positions.insert(
        "SOXL".to_string(),
        LeveragedTrendHoldPosition {
            in_position: true,
            entry_price: Some(100),
            high_water: Some(101),
            held_observations: 3,
        },
    );

    let signal = strategy.on_tick("SOXL", 99, 100);

    assert_eq!(signal, Signal::Hold);
}

#[test]
fn sells_at_breakeven_buffer_after_protection_profit_activates() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXL")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        trailing_stop_pct: 5.0,
        trailing_activation_profit_pct: 1.0,
        breakeven_buffer_pct: 0.2,
        min_hold_observations: 1,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("SOXL", &upward_candles());
    strategy.positions.insert(
        "SOXL".to_string(),
        LeveragedTrendHoldPosition {
            in_position: true,
            entry_price: Some(100),
            high_water: Some(103),
            held_observations: 1,
        },
    );

    let signal = strategy.on_tick("SOXL", 100, 100);

    assert!(
        matches!(signal, Signal::Sell { symbol, reason, .. } if symbol == "SOXL" && reason.contains("본전 보호 청산"))
    );
}

#[test]
fn sells_by_trailing_stop_only_after_protection_profit_activates() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("SOXL")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        trailing_stop_pct: 1.5,
        trailing_activation_profit_pct: 1.0,
        breakeven_buffer_pct: 0.2,
        min_hold_observations: 1,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("SOXL", &upward_candles());
    strategy.positions.insert(
        "SOXL".to_string(),
        LeveragedTrendHoldPosition {
            in_position: true,
            entry_price: Some(100),
            high_water: Some(104),
            held_observations: 1,
        },
    );

    let signal = strategy.on_tick("SOXL", 102, 100);

    assert!(
        matches!(signal, Signal::Sell { symbol, reason, .. } if symbol == "SOXL" && reason.contains("수익 보호 추적손절"))
    );
}

#[test]
fn buys_intraday_rebound_when_next_window_shows_buy_pressure() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        intraday_rebound_enabled: true,
        rebound_baseline_ticks: 2,
        rebound_confirm_ticks: 2,
        rebound_pullback_pct: 4.0,
        rebound_buy_pressure_pct: 2.0,
        rebound_rsi_min: 20.0,
        no_trade_adx_below: 0.0,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("KORU", &upward_candles());

    assert_eq!(strategy.on_tick("KORU", 190, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 180, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 181, 100), Signal::Hold);
    let signal = strategy.on_tick("KORU", 186, 100);

    assert!(
        matches!(signal, Signal::Buy { symbol, quantity, reason } if symbol == "KORU" && quantity == 3 && reason.contains("매수세 반동 진입"))
    );
}

#[test]
fn ignores_intraday_rebound_by_default() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        rebound_baseline_ticks: 2,
        rebound_confirm_ticks: 2,
        no_trade_adx_below: 0.0,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("KORU", &upward_candles());

    assert_eq!(strategy.on_tick("KORU", 190, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 180, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 181, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 186, 100), Signal::Hold);
}

#[test]
fn buys_rapid_rebound_without_trend_snapshot_when_enabled() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        rapid_rebound_enabled: true,
        rapid_rebound_lookback_ticks: 4,
        rapid_rebound_drop_pct: 3.0,
        rapid_rebound_recovery_pct: 1.5,
        rapid_rebound_max_low_age_ticks: 3,
        no_trade_adx_below: 90.0,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("KORU", &upward_candles());

    assert_eq!(strategy.on_tick("KORU", 100, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 96, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 97, 100), Signal::Hold);
    let signal = strategy.on_tick("KORU", 98, 100);

    assert!(
        matches!(signal, Signal::Buy { symbol, quantity, reason } if symbol == "KORU" && quantity == 3 && reason.contains("급반등 단독 진입"))
    );
}

#[test]
fn ignores_rapid_rebound_by_default() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        rapid_rebound_lookback_ticks: 4,
        rapid_rebound_drop_pct: 3.0,
        rapid_rebound_recovery_pct: 1.5,
        no_trade_adx_below: 0.0,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("KORU", &upward_candles());

    assert_eq!(strategy.on_tick("KORU", 100, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 96, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 97, 100), Signal::Hold);
    assert_eq!(strategy.on_tick("KORU", 98, 100), Signal::Hold);
}

#[test]
fn preview_signals_marks_intraday_rebound_without_order_state() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        intraday_rebound_enabled: true,
        rebound_baseline_ticks: 2,
        rebound_confirm_ticks: 2,
        rebound_pullback_pct: 4.0,
        rebound_buy_pressure_pct: 2.0,
        rebound_rsi_min: 20.0,
        no_trade_adx_below: 0.0,
        ..LeveragedTrendHoldParams::default()
    };
    let intraday = [190, 180, 181, 186]
        .into_iter()
        .enumerate()
        .map(|(idx, price)| LeveragedTrendHoldTimedCandle {
            time: format!("20260707090{}00", idx + 1),
            candle: OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            },
        })
        .collect::<Vec<_>>();

    let signals =
        LeveragedTrendHoldStrategy::preview_signals("KORU", params, &upward_candles(), &intraday);

    assert!(
        matches!(signals.first(), Some(signal) if signal.side == "buy" && signal.reason.contains("매수세 반동 진입"))
    );
}

#[test]
fn live_tick_and_preview_have_signal_parity_for_normalized_rebound_fixture() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        intraday_rebound_enabled: true,
        rebound_baseline_ticks: 2,
        rebound_confirm_ticks: 2,
        rebound_pullback_pct: 4.0,
        rebound_buy_pressure_pct: 2.0,
        rebound_rsi_min: 20.0,
        no_trade_adx_below: 0.0,
        ..LeveragedTrendHoldParams::default()
    };
    let intraday = [190, 180, 181, 186]
        .into_iter()
        .enumerate()
        .map(|(idx, price)| LeveragedTrendHoldTimedCandle {
            time: format!("20260707090{}00", idx + 1),
            candle: OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            },
        })
        .collect::<Vec<_>>();
    let preview = LeveragedTrendHoldStrategy::preview_signals(
        "KORU",
        params.clone(),
        &upward_candles(),
        &intraday,
    )
    .into_iter()
    .map(|signal| (signal.side, signal.reason))
    .collect::<Vec<_>>();

    let mut live = strategy_with_params(params);
    live.initialize_ohlc("KORU", &upward_candles());
    let live_signals = intraday
        .iter()
        .filter_map(
            |timed| match live.on_tick("KORU", timed.candle.close, 100) {
                Signal::Buy { reason, .. } => Some(("buy".to_string(), reason)),
                Signal::Sell { reason, .. } => Some(("sell".to_string(), reason)),
                Signal::Hold => None,
            },
        )
        .collect::<Vec<_>>();

    assert_eq!(preview, live_signals);
}

#[test]
fn preview_signals_marks_rapid_rebound_without_trend_snapshot() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        rapid_rebound_enabled: true,
        rapid_rebound_lookback_ticks: 4,
        rapid_rebound_drop_pct: 3.0,
        rapid_rebound_recovery_pct: 1.5,
        rapid_rebound_max_low_age_ticks: 3,
        no_trade_adx_below: 90.0,
        ..LeveragedTrendHoldParams::default()
    };
    let intraday = [100, 96, 97, 98]
        .into_iter()
        .enumerate()
        .map(|(idx, price)| LeveragedTrendHoldTimedCandle {
            time: format!("20260707131{}00", idx + 1),
            candle: OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            },
        })
        .collect::<Vec<_>>();

    let signals = LeveragedTrendHoldStrategy::preview_signals("KORU", params, &[], &intraday);

    assert!(
        matches!(signals.first(), Some(signal) if signal.side == "buy" && signal.reason.contains("급반등 단독 진입"))
    );
}

#[test]
fn preview_rebound_does_not_require_trend_snapshot() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        intraday_rebound_enabled: true,
        rebound_baseline_ticks: 2,
        rebound_confirm_ticks: 2,
        rebound_pullback_pct: 4.0,
        rebound_buy_pressure_pct: 2.0,
        rebound_rsi_min: 20.0,
        no_trade_adx_below: 90.0,
        ..LeveragedTrendHoldParams::default()
    };
    let intraday = [190, 180, 181, 186]
        .into_iter()
        .enumerate()
        .map(|(idx, price)| LeveragedTrendHoldTimedCandle {
            time: format!("20260707130{}00", idx + 1),
            candle: OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            },
        })
        .collect::<Vec<_>>();

    let signals = LeveragedTrendHoldStrategy::preview_signals("KORU", params, &[], &intraday);

    assert!(
        matches!(signals.first(), Some(signal) if signal.side == "buy" && signal.reason.contains("RSI/ADX 준비 전"))
    );
}

#[test]
fn intraday_ohlc_initialization_matches_rebound_preview_window() {
    let params = LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        intraday_rebound_enabled: true,
        rebound_baseline_ticks: 2,
        rebound_confirm_ticks: 2,
        rebound_pullback_pct: 4.0,
        rebound_buy_pressure_pct: 2.0,
        rebound_rsi_min: 20.0,
        no_trade_adx_below: 90.0,
        ..LeveragedTrendHoldParams::default()
    };
    let mut strategy = strategy_with_params(params);
    strategy.initialize_intraday_ohlc("KORU", &upward_candles());
    let intraday = [190, 180, 181, 186]
        .into_iter()
        .map(|price| OhlcCandle {
            open: price,
            high: price,
            low: price,
            close: price,
        })
        .collect::<Vec<_>>();

    strategy.initialize_intraday_ohlc("KORU", &intraday);

    assert!(strategy.rebound_entry_ok("KORU").is_some());
}
