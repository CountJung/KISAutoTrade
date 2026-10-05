use super::tests::{entry, strategy_with_params, upward_candles};
use super::*;

fn params() -> LeveragedTrendHoldParams {
    LeveragedTrendHoldParams {
        entries: vec![entry("KORU")],
        entry_adx_min: 0.0,
        no_trade_adx_below: 0.0,
        ..LeveragedTrendHoldParams::default()
    }
}

fn timed(prices: &[u64]) -> Vec<LeveragedTrendHoldTimedCandle> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &price)| LeveragedTrendHoldTimedCandle {
            time: format!("2026070713{:02}00", i),
            candle: OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            },
        })
        .collect()
}

#[test]
fn daily_context_cannot_prepare_minute_indicators_or_rebound() {
    let mut strategy = strategy_with_params(params());
    strategy.initialize_ohlc("KORU", &upward_candles());
    assert!(!strategy.states.contains_key("KORU"));
    assert_eq!(strategy.preview_snapshot("KORU"), (None, None, None, None));
    assert!(strategy.bollinger_candles.is_empty());
    assert_eq!(strategy.on_tick("KORU", 192, 0), Signal::Hold);
    assert!(strategy.snapshot_for("KORU").is_none());
    assert_eq!(strategy.states["KORU"].intraday_candles.len(), 1);
    assert_eq!(strategy.states["KORU"].rebound_prices.len(), 1);
}

#[test]
fn daily_reinitialization_keeps_intraday_bollinger_and_actual_position() {
    let mut strategy = strategy_with_params(params());
    strategy.initialize_intraday_ohlc("KORU", &upward_candles());
    strategy.sync_position("KORU", 3, 200);
    let position = strategy.positions.get_mut("KORU").unwrap();
    position.high_water = Some(240);
    position.held_observations = 7;
    let before = strategy.preview_snapshot("KORU");
    let minute_len = strategy.states["KORU"].intraday_candles.len();
    let rebound_len = strategy.states["KORU"].rebound_prices.len();
    let bollinger_len = strategy.bollinger_candles["KORU"].len();
    for _ in 0..2 {
        strategy.initialize_ohlc(
            "KORU",
            &vec![
                OhlcCandle {
                    open: 900,
                    high: 1000,
                    low: 800,
                    close: 950
                };
                1000
            ],
        );
    }
    assert_eq!(strategy.daily_context["KORU"].len(), strategy.window_cap());
    assert_eq!(strategy.preview_snapshot("KORU"), before);
    assert_eq!(strategy.states["KORU"].intraday_candles.len(), minute_len);
    assert_eq!(strategy.states["KORU"].rebound_prices.len(), rebound_len);
    assert_eq!(strategy.bollinger_candles["KORU"].len(), bollinger_len);
    let position = &strategy.positions["KORU"];
    assert!(position.in_position);
    assert_eq!(position.entry_price, Some(200));
    assert_eq!(position.high_water, Some(240));
    assert_eq!(position.held_observations, 7);
    // Missing/changed daily context cannot prevent the existing risk exit.
    assert!(matches!(
        strategy.on_tick("KORU", 180, 0),
        Signal::Sell { .. }
    ));
}

#[test]
fn repeated_intraday_seed_replaces_buffers_without_daily_mix_or_position_reset() {
    let mut strategy = strategy_with_params(params());
    strategy.initialize_ohlc("KORU", &upward_candles());
    strategy.sync_position("KORU", 3, 100);
    let bars: Vec<_> = timed(&[90, 92, 94, 96]).iter().map(|c| c.candle).collect();
    strategy.initialize_intraday_ohlc("KORU", &bars);
    let before = strategy.preview_snapshot("KORU");
    strategy.initialize_intraday_ohlc("KORU", &bars);
    assert_eq!(strategy.preview_snapshot("KORU"), before);
    assert_eq!(strategy.states["KORU"].intraday_candles.len(), 4);
    assert_eq!(
        strategy.states["KORU"]
            .rebound_prices
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![90, 92, 94, 96]
    );
    assert_eq!(strategy.bollinger_candles["KORU"].len(), 4);
    assert_eq!(strategy.positions["KORU"].entry_price, Some(100));
    assert!(strategy.positions["KORU"].in_position);
    strategy.initialize_intraday_ohlc("KORU", &[]);
    assert!(strategy.states["KORU"].intraday_candles.is_empty());
    assert!(strategy.states["KORU"].rebound_prices.is_empty());
    assert!(strategy.bollinger_candles["KORU"].is_empty());
    assert!(strategy.positions["KORU"].in_position);
}

#[test]
fn replay_daily_values_cannot_change_minute_signals_or_indicator_snapshot() {
    let p = LeveragedTrendHoldParams {
        intraday_rebound_enabled: true,
        rebound_baseline_ticks: 2,
        rebound_confirm_ticks: 2,
        rebound_pullback_pct: 4.0,
        rebound_buy_pressure_pct: 2.0,
        rebound_rsi_min: 20.0,
        ..params()
    };
    let mut intraday: Vec<_> = upward_candles()
        .into_iter()
        .enumerate()
        .map(|(i, candle)| LeveragedTrendHoldTimedCandle {
            time: format!("20260706{:02}{:02}00", 13 + (30 + i) / 60, (30 + i) % 60),
            candle,
        })
        .collect();
    let evaluation: Vec<_> = [190, 180, 181, 186]
        .into_iter()
        .enumerate()
        .map(|(i, price)| LeveragedTrendHoldTimedCandle {
            time: format!("20260707090{}00", i + 1),
            candle: OhlcCandle {
                open: price,
                high: price,
                low: price,
                close: price,
            },
        })
        .collect();
    intraday.extend(evaluation);
    let altered_daily = vec![
        OhlcCandle {
            open: 10000,
            high: 20000,
            low: 1,
            close: 2
        };
        252
    ];
    let baseline = LeveragedTrendHoldStrategy::preview_signals(
        "KORU",
        p.clone(),
        &upward_candles(),
        &intraday,
    );
    assert_eq!(baseline.len(), 1);
    assert_eq!(baseline[0].time, "20260707090400");
    assert!(
        baseline[0].ema_short.is_some() && baseline[0].rsi.is_some() && baseline[0].adx.is_some()
    );
    for daily in [&altered_daily[..], &[][..]] {
        let changed =
            LeveragedTrendHoldStrategy::preview_signals("KORU", p.clone(), daily, &intraday);
        assert_eq!(
            serde_json::to_value(&baseline).unwrap(),
            serde_json::to_value(changed).unwrap()
        );
    }
}

#[test]
fn failed_buy_feedback_does_not_create_phantom_position() {
    let p = LeveragedTrendHoldParams {
        rapid_rebound_enabled: true,
        rapid_rebound_lookback_ticks: 4,
        ..params()
    };
    let observations = timed(&[100, 96, 97, 98, 99, 98, 97]);
    let signals = LeveragedTrendHoldStrategy::preview_signals_with_execution(
        "KORU",
        p,
        &upward_candles(),
        &observations,
        |_, _| false,
    );
    assert!(signals.iter().any(|s| s.side == "buy"));
    assert!(signals.iter().all(|s| s.side == "buy"));
}

#[test]
fn intraday_and_daily_context_buffers_have_independent_bounds() {
    let mut strategy = strategy_with_params(params());
    let bar = OhlcCandle {
        open: 100,
        high: 101,
        low: 99,
        close: 100,
    };
    strategy.initialize_ohlc("KORU", &vec![bar; 2000]);
    strategy.initialize_intraday_ohlc("KORU", &vec![bar; 2000]);
    assert_eq!(strategy.daily_context["KORU"].len(), strategy.window_cap());
    assert_eq!(
        strategy.states["KORU"].intraday_candles.len(),
        strategy.window_cap()
    );
    assert_eq!(
        strategy.states["KORU"].rebound_prices.len(),
        strategy.rebound_price_cap()
    );
    assert_eq!(strategy.bollinger_candles["KORU"].len(), 512);
    strategy.initialize_ohlc("KORU", &[]);
    assert!(strategy.daily_context["KORU"].is_empty());
    assert_eq!(
        strategy.states["KORU"].intraday_candles.len(),
        strategy.window_cap()
    );
}

#[test]
fn failed_sell_feedback_keeps_actual_position_for_next_risk_exit() {
    let p = LeveragedTrendHoldParams {
        rapid_rebound_enabled: true,
        rapid_rebound_lookback_ticks: 4,
        ..params()
    };
    let observations = timed(&[100, 96, 97, 98, 90, 89]);
    let signals = LeveragedTrendHoldStrategy::preview_signals_with_execution(
        "KORU",
        p,
        &[],
        &observations,
        |s, _| s.side == "buy",
    );
    assert_eq!(
        signals.iter().map(|s| s.side.as_str()).collect::<Vec<_>>(),
        vec!["buy", "sell", "sell"]
    );
}
