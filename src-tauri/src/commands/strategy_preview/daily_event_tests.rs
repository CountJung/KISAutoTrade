use super::{preview_strategy_from_candles, StrategyPreviewInput};
use crate::api::rest::ChartCandle;
use crate::trading::{simulation::SimulationAssumptions, strategy::*};

fn candle(day: u32, open: f64, high: f64, low: f64, close: f64) -> ChartCandle {
    ChartCandle {
        date: format!("202601{day:02}"),
        open: open.to_string(),
        high: high.to_string(),
        low: low.to_string(),
        close: close.to_string(),
        volume: "1000".into(),
    }
}

fn input(
    id: &str,
    params: serde_json::Value,
    history: Vec<ChartCandle>,
    candles: Vec<ChartCandle>,
) -> StrategyPreviewInput {
    StrategyPreviewInput {
        strategy_id: id.into(),
        strategy_name: id.into(),
        symbol: "SOXQ".into(),
        is_overseas: true,
        order_quantity: 1,
        params,
        candles,
        history_candles: history,
        warmup_count: None,
        interval: Some("D".into()),
        data_source: Some("daily-event-fixture".into()),
        strategy_version: None,
        broker_id: None,
        broker_account_id: None,
        assumptions: SimulationAssumptions {
            max_position_ratio: 1.0,
            ..Default::default()
        },
    }
}

#[test]
fn strong_close_uses_previous_completed_day_and_keeps_position_across_day_changes() {
    let preview = |future: f64| {
        preview_strategy_from_candles(input(
            "strong_close",
            serde_json::to_value(StrongCloseParams {
                threshold_pct: 1.0,
                ..Default::default()
            })
            .unwrap(),
            vec![candle(1, 100.0, 110.0, 99.0, 100.0)],
            vec![
                candle(2, 108.0, 110.0, 107.0, 109.5),
                candle(3, 94.0, 95.0, 89.0, 90.0),
                candle(4, 88.0, 88.5, 85.0, 86.0),
                candle(5, 108.0, 110.0, 107.0, 109.5),
                candle(6, future, future + 5.0, future - 1.0, future),
            ],
        ))
        .unwrap()
    };
    let result = preview(100.0);
    let sides = result
        .signals
        .iter()
        .map(|s| (s.time.as_str(), s.side.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        sides,
        vec![
            ("20260103", "buy"),
            ("20260104", "sell"),
            ("20260106", "buy")
        ]
    );
    assert_eq!(result.replay.replay_cadence, "dailyCloseWithDayBoundary");
    assert_eq!(result.preparation.evaluated_bars, Some(5));
    let changed = preview(200.0);
    let prior = |p: &super::StrategyPreviewView| {
        p.signals
            .iter()
            .filter(|s| s.time.as_str() < "20260106")
            .map(|s| (&s.time, &s.side, s.quantity))
            .map(|(t, s, q)| (t.clone(), s.clone(), q))
            .collect::<Vec<_>>()
    };
    assert_eq!(prior(&result), prior(&changed));
}

#[test]
fn blocked_strong_close_buy_does_not_discard_next_days_completed_condition() {
    let mut fixture = input(
        "strong_close",
        serde_json::to_value(StrongCloseParams::default()).unwrap(),
        vec![candle(1, 100.0, 100.0, 100.0, 100.0)],
        (2..=4)
            .map(|day| candle(day, 100.0, 100.0, 100.0, 100.0))
            .collect(),
    );
    fixture.order_quantity = 1000;
    fixture.assumptions.initial_capital_krw = 10_000.0;
    let preview = preview_strategy_from_candles(fixture).unwrap();
    assert_eq!(preview.signals.len(), 3);
    assert!(preview.signals.iter().all(|s| s.side == "buy"));
    assert_eq!(preview.backtest.summary.filled_order_count, 0);
    assert_eq!(preview.backtest.summary.blocked_order_count, 3);
    assert_eq!(preview.preparation.outcome, "noTrades");
}

#[test]
fn volatility_uses_previous_days_rolling_ranges_and_independent_today_ohlc() {
    let preview = preview_strategy_from_candles(input(
        "volatility_expansion",
        serde_json::to_value(VolatilityExpansionParams {
            lookback_days: 2,
            expansion_factor: 2.0,
            ..Default::default()
        })
        .unwrap(),
        vec![
            candle(1, 100.0, 110.0, 100.0, 100.0),
            candle(2, 100.0, 120.0, 100.0, 100.0),
        ],
        vec![
            candle(3, 100.0, 120.0, 100.0, 115.0),
            candle(4, 100.0, 135.0, 100.0, 134.0),
            candle(5, 100.0, 160.0, 100.0, 155.0),
            candle(6, 156.0, 157.0, 154.0, 155.0),
            candle(7, 153.0, 154.0, 148.0, 149.0),
        ],
    ))
    .unwrap();
    assert_eq!(
        preview
            .signals
            .iter()
            .map(|s| (s.time.as_str(), s.side.as_str()))
            .collect::<Vec<_>>(),
        vec![("20260105", "buy"), ("20260107", "sell")]
    );
    assert_eq!(preview.preparation.evaluated_bars, Some(5));
    assert_eq!(preview.backtest.summary.completed_trades, 1);
}

#[test]
fn flat_completed_days_fill_history_without_compressing_the_window() {
    let preview = preview_strategy_from_candles(input(
        "volatility_expansion",
        serde_json::to_value(VolatilityExpansionParams {
            lookback_days: 2,
            ..Default::default()
        })
        .unwrap(),
        vec![candle(1, 100.0, 100.0, 100.0, 100.0)],
        vec![
            candle(2, 100.0, 100.0, 100.0, 100.0),
            candle(3, 100.0, 100.0, 100.0, 100.0),
            candle(4, 100.0, 103.0, 100.0, 102.0),
        ],
    ))
    .unwrap();
    assert_eq!(preview.preparation.available_history_bars, Some(1));
    assert_eq!(preview.preparation.ready_at_start, Some(false));
    assert_eq!(
        preview.preparation.first_ready_time.as_deref(),
        Some("20260102")
    );
    assert_eq!(preview.preparation.evaluated_bars, Some(2));
    assert_eq!(preview.signals[0].time, "20260104");
}

#[test]
fn daily_event_strategies_reject_other_cadences() {
    for id in ["strong_close", "volatility_expansion"] {
        for interval in ["1m", "M1", "W", "M"] {
            let mut fixture = input(
                id,
                serde_json::json!({}),
                vec![],
                vec![candle(1, 100.0, 101.0, 99.0, 100.0)],
            );
            fixture.interval = Some(interval.into());
            assert_eq!(
                preview_strategy_from_candles(fixture).unwrap_err().code,
                "UNSUPPORTED_REPLAY_INTERVAL"
            );
        }
    }
}
