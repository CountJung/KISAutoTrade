use super::{
    broker_candles_to_timed_ohlc, daily_chart_time, daily_preview_time, daily_warmup_end_before,
    minute_replay_trading_date, preview_session_bounds, preview_strategy_from_candles,
    StrategyPreviewInput,
};
use crate::api::rest::ChartCandle;
use crate::broker::{BrokerCandle, BrokerMarket, BrokerMoney, BrokerQuantity, BrokerSymbol};
use crate::trading::simulation::SimulationAssumptions;
use crate::trading::strategy::{build_strategy, Signal, StrategyConfig};

#[test]
fn daily_preview_time_keeps_provider_date_and_selected_session_minute() {
    assert_eq!(
        daily_preview_time("2026-07-11", 22 * 60 + 35, 0).as_deref(),
        Some("20260711223500")
    );
    assert_eq!(
        daily_preview_time("2026-07-11", 5 * 60, 1).as_deref(),
        Some("20260712050000")
    );
    assert_eq!(daily_preview_time("invalid", 9 * 60, 0), None);
    assert_eq!(daily_chart_time("20260712050000", 1), "20260711");
    assert_eq!(daily_chart_time("20260711224000", 1), "20260711");
}

#[test]
fn daily_preview_uses_selected_market_session_entry_window() {
    assert_eq!(
        preview_session_bounds(false, "auto", 5),
        (9 * 60 + 5, 15 * 60 + 30, 0)
    );
    assert_eq!(
        preview_session_bounds(true, "regular", 10),
        (22 * 60 + 40, 5 * 60, 1)
    );
    assert_eq!(
        preview_session_bounds(true, "after", 0),
        (5 * 60, 7 * 60, 0)
    );
}

#[test]
fn daily_preview_reveals_only_open_at_session_start_then_completed_ohlc_at_close() {
    let source = BrokerCandle {
        symbol: BrokerSymbol("SOXL".into()),
        market: BrokerMarket::Us,
        date: "20260711".into(),
        open: BrokerMoney::usd("100"),
        high: BrokerMoney::usd("120"),
        low: BrokerMoney::usd("90"),
        close: BrokerMoney::usd("110"),
        volume: BrokerQuantity("1000".into()),
    };
    let timed = broker_candles_to_timed_ohlc(&[source], 1, "1d", 9 * 60 + 5, 16 * 60 + 50, 0);

    assert_eq!(timed.len(), 2);
    assert_eq!(timed[0].time, "20260711090500");
    assert_eq!(timed[0].candle.open, 10_000);
    assert_eq!(timed[0].candle.high, 10_000);
    assert_eq!(timed[0].candle.low, 10_000);
    assert_eq!(timed[0].candle.close, 10_000);
    assert_eq!(timed[1].time, "20260711165000");
    assert_eq!(timed[1].candle.high, 12_000);
    assert_eq!(timed[1].candle.low, 9_000);
    assert_eq!(timed[1].candle.close, 11_000);
}

#[test]
fn future_candle_change_does_not_change_prior_replay_signals() {
    let preview = |future_close: &str| {
        preview_strategy_from_candles(StrategyPreviewInput {
            strategy_id: "price_condition_test".into(),
            strategy_name: "가격 조건 테스트".into(),
            symbol: "005930".into(),
            is_overseas: false,
            order_quantity: 1,
            params: serde_json::json!({
                "symbols": [{
                    "symbol": "005930",
                    "quantity": 1,
                    "buy_trigger_price": 70_000,
                    "sell_trigger_price": 80_000,
                    "take_profit_pct": 5,
                    "stop_loss_pct": 3,
                    "is_overseas": false
                }]
            }),
            candles: vec![
                candle("20260701", "69000"),
                candle("20260702", "81000"),
                candle("20260703", future_close),
            ],
            history_candles: Vec::new(),
            warmup_count: Some(0),
            interval: Some("D".into()),
            data_source: Some("fixture".into()),
            strategy_version: Some("fixture-v1".into()),
            broker_id: None,
            broker_account_id: None,
            assumptions: SimulationAssumptions::default(),
        })
        .expect("fixture replay")
    };

    let left = preview("75000");
    let right = preview("95000");
    let prior_left = left
        .signals
        .iter()
        .filter(|signal| signal.time.as_str() < "20260703")
        .map(|signal| (&signal.time, &signal.side, signal.quantity))
        .collect::<Vec<_>>();
    let prior_right = right
        .signals
        .iter()
        .filter(|signal| signal.time.as_str() < "20260703")
        .map(|signal| (&signal.time, &signal.side, signal.quantity))
        .collect::<Vec<_>>();

    assert_eq!(prior_left, prior_right);
    assert!(left.replay.deterministic);
    assert!(left.replay.look_ahead_safe);
}

#[test]
fn minute_replay_warmup_excludes_replay_day_and_future_daily_candles() {
    let rows = ["20260709", "20260710", "20260711", "20260712"]
        .into_iter()
        .map(|date| BrokerCandle {
            symbol: BrokerSymbol("SOXL".into()),
            market: BrokerMarket::Us,
            date: date.into(),
            open: BrokerMoney::usd("100"),
            high: BrokerMoney::usd("110"),
            low: BrokerMoney::usd("90"),
            close: BrokerMoney::usd("105"),
            volume: BrokerQuantity("1000".into()),
        })
        .collect::<Vec<_>>();

    assert_eq!(daily_warmup_end_before(&rows, "20260711"), 2);
    assert_eq!(
        minute_replay_trading_date("20260712020000", true, "auto"),
        "20260711"
    );
    assert_eq!(
        minute_replay_trading_date("20260712060000", true, "auto"),
        "20260712"
    );
}

#[test]
fn blocked_buy_is_fed_back_before_the_next_raw_signal() {
    let result = preview_strategy_from_candles(StrategyPreviewInput {
        strategy_id: "price_condition_test".into(),
        strategy_name: "가격 조건 테스트".into(),
        symbol: "005930".into(),
        is_overseas: false,
        order_quantity: 1_000,
        params: serde_json::json!({
            "symbols": [{
                "symbol": "005930",
                "quantity": 1_000,
                "buy_trigger_price": 70_000,
                "sell_trigger_price": 80_000,
                "take_profit_pct": 5,
                "stop_loss_pct": 3,
                "is_overseas": false
            }]
        }),
        candles: vec![candle("20260701", "69000"), candle("20260702", "68000")],
        history_candles: Vec::new(),
        warmup_count: Some(0),
        interval: Some("D".into()),
        data_source: Some("fixture".into()),
        strategy_version: Some("fixture-v1".into()),
        broker_id: None,
        broker_account_id: None,
        assumptions: SimulationAssumptions {
            initial_capital_krw: 10_000.0,
            max_position_ratio: 1.0,
            ..SimulationAssumptions::default()
        },
    })
    .expect("fixture replay");

    assert_eq!(result.signals.len(), 2);
    assert_eq!(result.backtest.summary.filled_order_count, 0);
    assert_eq!(result.backtest.summary.blocked_order_count, 2);
    assert_eq!(result.preparation.outcome, "noTrades");
}

#[test]
fn reproduction_hash_covers_ohlcv_and_order_quantity() {
    let preview_hash = |high: &str, order_quantity: u64, warmup_count: usize| {
        let mut row = candle("20260701", "69000");
        row.high = high.into();
        preview_strategy_from_candles(StrategyPreviewInput {
            strategy_id: "price_condition_test".into(),
            strategy_name: "가격 조건 테스트".into(),
            symbol: "005930".into(),
            is_overseas: false,
            order_quantity,
            params: serde_json::json!({"symbols": []}),
            candles: vec![row, candle("20260702", "70000")],
            history_candles: Vec::new(),
            warmup_count: Some(warmup_count),
            interval: Some("D".into()),
            data_source: Some("fixture".into()),
            strategy_version: Some("fixture-v1".into()),
            broker_id: None,
            broker_account_id: None,
            assumptions: SimulationAssumptions::default(),
        })
        .expect("fixture replay")
        .replay
        .input_hash
    };

    assert_ne!(preview_hash("70000", 1, 0), preview_hash("71000", 1, 0));
    assert_ne!(preview_hash("70000", 1, 0), preview_hash("70000", 2, 0));
    assert_ne!(preview_hash("70000", 1, 0), preview_hash("70000", 1, 1));
}

#[test]
fn live_tick_and_preview_emit_identical_signals_for_normalized_fixture() {
    let params = serde_json::json!({
        "symbols": [{
            "symbol": "005930",
            "quantity": 1,
            "buy_trigger_price": 70_000,
            "sell_trigger_price": 80_000,
            "take_profit_pct": 5,
            "stop_loss_pct": 3,
            "is_overseas": false
        }]
    });
    let preview = preview_strategy_from_candles(StrategyPreviewInput {
        strategy_id: "price_condition_fixture".into(),
        strategy_name: "가격 조건 fixture".into(),
        symbol: "005930".into(),
        is_overseas: false,
        order_quantity: 1,
        params: params.clone(),
        candles: vec![candle("20260701", "69000"), candle("20260702", "81000")],
        history_candles: Vec::new(),
        warmup_count: Some(0),
        interval: Some("D".into()),
        data_source: Some("fixture".into()),
        strategy_version: Some("fixture-v1".into()),
        broker_id: None,
        broker_account_id: None,
        assumptions: SimulationAssumptions {
            initial_capital_krw: 1_000_000.0,
            max_position_ratio: 1.0,
            ..SimulationAssumptions::default()
        },
    })
    .expect("fixture replay");

    let mut live_strategy = build_strategy(StrategyConfig::new(
        "price_condition_fixture",
        "가격 조건 fixture",
        true,
        vec!["005930".into()],
        1,
        params,
    ));
    let live_sides = [69_000, 81_000]
        .into_iter()
        .filter_map(
            |price| match live_strategy.on_tick("005930", price, 1_000) {
                Signal::Buy { .. } => Some("buy"),
                Signal::Sell { .. } => Some("sell"),
                Signal::Hold => None,
            },
        )
        .collect::<Vec<_>>();
    let preview_sides = preview
        .signals
        .iter()
        .map(|signal| signal.side.as_str())
        .collect::<Vec<_>>();

    assert_eq!(preview_sides, live_sides);
}

#[test]
fn daily_warmup_preview_uses_closes_and_excludes_future_bars() {
    use crate::trading::strategy::{MeanReversionParams, TrendFilterParams};

    for (id, params, first_signal_index) in [
        (
            "mean_reversion",
            serde_json::to_value(MeanReversionParams::default()).unwrap(),
            253,
        ),
        (
            "trend_filter",
            serde_json::to_value(TrendFilterParams::default()).unwrap(),
            252,
        ),
    ] {
        let start = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let date = |index| {
            (start + chrono::Duration::days(index))
                .format("%Y%m%d")
                .to_string()
        };
        let preview = |warmup_high: u64, future_close: u64| {
            let mut candles = (0..252)
                .map(|index| {
                    let close = if id == "mean_reversion" {
                        10_000
                    } else {
                        10_000 + index
                    };
                    // Overseas input is USD; the strategy receives integer cents.
                    ChartCandle {
                        date: date(index as i64),
                        open: format!("{:.2}", close as f64 / 100.0),
                        high: format!("{:.2}", warmup_high as f64 / 100.0),
                        low: format!("{:.2}", (close - 100) as f64 / 100.0),
                        close: format!("{:.2}", close as f64 / 100.0),
                        volume: "1000".into(),
                    }
                })
                .collect::<Vec<_>>();
            let prices = if id == "mean_reversion" {
                [10_000, 9_000, future_close]
            } else {
                [10_252, 10_253, future_close]
            };
            candles.extend(prices.into_iter().enumerate().map(|(index, price)| {
                candle(
                    &date(252 + index as i64),
                    &format!("{:.2}", price as f64 / 100.0),
                )
            }));
            preview_strategy_from_candles(StrategyPreviewInput {
                strategy_id: id.into(),
                strategy_name: id.into(),
                symbol: "SOXQ".into(),
                is_overseas: true,
                order_quantity: 10,
                params: params.clone(),
                candles,
                history_candles: Vec::new(),
                warmup_count: Some(252),
                interval: Some("1d".into()),
                data_source: Some("high-close-fixture".into()),
                strategy_version: None,
                broker_id: None,
                broker_account_id: None,
                assumptions: SimulationAssumptions::default(),
            })
            .unwrap()
        };

        let baseline = preview(20_000, 10_300);
        let changed_highs = preview(30_000, 10_300);
        assert_eq!(baseline.signals[0].time, date(first_signal_index));
        assert_eq!(baseline.signals[0].side, "buy");
        assert_eq!(baseline.replay.data_start, date(252));
        assert_eq!(baseline.replay.data_end, date(254));
        assert_eq!(baseline.replay.warmup_count, 252);
        assert_eq!(
            serde_json::to_value(&baseline.signals).unwrap(),
            serde_json::to_value(&changed_highs.signals).unwrap()
        );
        assert_ne!(baseline.replay.input_hash, changed_highs.replay.input_hash);

        let future_changed = preview(20_000, 8_000);
        let prior_signals = |result: &super::StrategyPreviewView| {
            result
                .signals
                .iter()
                .filter(|signal| signal.time < date(254))
                .map(|signal| serde_json::to_value(signal).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(prior_signals(&baseline), prior_signals(&future_changed));
    }
}

fn candle(date: &str, close: &str) -> ChartCandle {
    ChartCandle {
        date: date.into(),
        open: close.into(),
        high: close.into(),
        low: close.into(),
        close: close.into(),
        volume: "1000".into(),
    }
}
