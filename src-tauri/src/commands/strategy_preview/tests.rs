use super::{
    broker_candles_to_timed_ohlc, daily_chart_time, daily_warmup_end_before,
    minute_replay_trading_date, preview_session_bounds, preview_strategy_from_candles,
    StrategyPreviewInput,
};
use crate::api::rest::ChartCandle;
use crate::broker::{BrokerCandle, BrokerMarket, BrokerMoney, BrokerQuantity, BrokerSymbol};
use crate::trading::simulation::SimulationAssumptions;
use crate::trading::strategy::{build_strategy, Signal, StrategyConfig};

struct FetchAdapter {
    daily: crate::broker::BrokerAdapterResult<Vec<BrokerCandle>>,
    minute: crate::broker::BrokerAdapterResult<Vec<BrokerCandle>>,
    calls: std::sync::Mutex<Vec<String>>,
}

impl FetchAdapter {
    fn new(
        daily: crate::broker::BrokerAdapterResult<Vec<BrokerCandle>>,
        minute: crate::broker::BrokerAdapterResult<Vec<BrokerCandle>>,
    ) -> Self {
        Self {
            daily,
            minute,
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::broker::BrokerAdapter for FetchAdapter {
    fn broker_id(&self) -> crate::broker::BrokerId {
        crate::broker::BrokerId::Toss
    }

    async fn get_price(
        &self,
        _symbol: &BrokerSymbol,
    ) -> crate::broker::BrokerAdapterResult<crate::broker::BrokerPriceQuote> {
        panic!("fetch fixture must not request prices")
    }

    async fn get_candles(
        &self,
        symbol: &BrokerSymbol,
        period_code: &str,
        from: &str,
        to: &str,
    ) -> crate::broker::BrokerAdapterResult<Vec<BrokerCandle>> {
        assert_eq!(symbol.0, "KORU");
        assert_eq!((from, to), ("", ""));
        self.calls.lock().unwrap().push(period_code.into());
        let result = match period_code {
            "D" => &self.daily,
            "1m" => &self.minute,
            _ => panic!("unexpected candle interval: {period_code}"),
        };
        match result {
            Ok(candles) => Ok(candles.clone()),
            Err(crate::broker::BrokerAdapterError::InvalidRequest(message)) => Err(
                crate::broker::BrokerAdapterError::InvalidRequest(message.clone()),
            ),
            Err(error) => panic!("unexpected fixture error: {error}"),
        }
    }
    // Other operations, including get_candle_page, keep the trait's unsupported contract.
}

#[tokio::test]
async fn lth_fetch_sorts_daily_and_preserves_raw_minute_order() {
    let daily = vec![a2_us_bar("20260702", 102), a2_us_bar("20260701", 101)];
    let minute = vec![
        a2_us_bar("20260702223100", 104),
        a2_us_bar("20260702223000", 103),
    ];
    let adapter = FetchAdapter::new(Ok(daily.clone()), Ok(minute.clone()));
    let (actual_daily, actual_minute) = super::fetch_leveraged_trend_hold_candles(
        &adapter,
        &BrokerSymbol("KORU".into()),
        "1m",
        None,
    )
    .await
    .unwrap();

    assert_eq!(actual_daily, vec![daily[1].clone(), daily[0].clone()]);
    assert_eq!(actual_minute, minute);
    assert_eq!(*adapter.calls.lock().unwrap(), ["D", "1m"]);
}

#[tokio::test]
async fn lth_fetch_daily_only_returns_an_independent_sorted_clone() {
    let daily = vec![a2_us_bar("20260702", 102), a2_us_bar("20260701", 101)];
    let adapter = FetchAdapter::new(Ok(daily.clone()), Ok(Vec::new()));
    let (actual_daily, mut simulation) = super::fetch_leveraged_trend_hold_candles(
        &adapter,
        &BrokerSymbol("KORU".into()),
        "1d",
        None,
    )
    .await
    .unwrap();

    assert_eq!(actual_daily, vec![daily[1].clone(), daily[0].clone()]);
    assert_eq!(simulation, actual_daily);
    simulation[0].close = BrokerMoney::usd("999");
    assert_eq!(actual_daily[0].close, BrokerMoney::usd("101"));
    assert_eq!(*adapter.calls.lock().unwrap(), ["D"]);
}

#[tokio::test]
async fn lth_fetch_daily_failure_stops_before_minute_request() {
    let adapter = FetchAdapter::new(
        Err(crate::broker::BrokerAdapterError::InvalidRequest(
            "daily fixture".into(),
        )),
        Ok(vec![a2_us_bar("20260702223000", 103)]),
    );
    let error = super::fetch_leveraged_trend_hold_candles(
        &adapter,
        &BrokerSymbol("KORU".into()),
        "1m",
        None,
    )
    .await
    .unwrap_err();

    assert_eq!(error.code, "TOSS_CANDLES_ERROR");
    assert_eq!(
        error.message,
        "Toss 일봉 조회 실패: invalid broker request: daily fixture"
    );
    assert_eq!(*adapter.calls.lock().unwrap(), ["D"]);
}

#[tokio::test]
async fn lth_fetch_minute_failure_is_propagated_after_daily_request() {
    let adapter = FetchAdapter::new(
        Ok(vec![a2_us_bar("20260701", 101)]),
        Err(crate::broker::BrokerAdapterError::InvalidRequest(
            "minute fixture".into(),
        )),
    );
    let error = super::fetch_leveraged_trend_hold_candles(
        &adapter,
        &BrokerSymbol("KORU".into()),
        "1m",
        None,
    )
    .await
    .unwrap_err();

    assert_eq!(error.code, "TOSS_CANDLES_ERROR");
    assert_eq!(
        error.message,
        "Toss 1분봉 조회 실패: invalid broker request: minute fixture"
    );
    assert_eq!(*adapter.calls.lock().unwrap(), ["D", "1m"]);
}

#[test]
fn daily_diagnostic_keeps_the_provider_date_without_synthetic_session_offset() {
    assert_eq!(daily_chart_time("20260711", 1), "20260711");
    assert_eq!(daily_chart_time("20260712050000", 1), "20260711");
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
fn daily_diagnostic_contains_one_completed_bar_and_no_synthetic_entry_observation() {
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

    assert_eq!(timed.len(), 1);
    assert_eq!(timed[0].time, "20260711");
    assert_eq!(timed[0].candle.open, 10_000);
    assert_eq!(timed[0].candle.high, 12_000);
    assert_eq!(timed[0].candle.low, 9_000);
    assert_eq!(timed[0].candle.close, 11_000);
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
    // Signal prefix invariance does not prove execution at the just-observed close.
    assert!(!left.replay.look_ahead_safe);
    assert!(left
        .backtest
        .execution_warnings
        .iter()
        .any(|warning| warning.contains("같은 종가")));
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

#[test]
fn daily_diagnostic_never_evaluates_strategy_or_execution_even_with_intraday_input() {
    use crate::trading::simulation::ReplayAssessmentView;
    use crate::trading::strategy::{
        LeveragedTrendHoldParams, LeveragedTrendHoldTimedCandle, OhlcCandle,
    };
    let bar = OhlcCandle {
        open: 100,
        high: 110,
        low: 90,
        close: 105,
    };
    let timed = vec![LeveragedTrendHoldTimedCandle {
        time: "20260707091500".into(),
        candle: bar,
    }];
    let signals = super::lth_preview_signals_for_interval(
        "1d",
        "SOXQ",
        LeveragedTrendHoldParams::default(),
        &[bar],
        &timed,
        |_| panic!("daily diagnostics must not run execution"),
    );
    assert!(signals.is_empty());
    let assessment =
        serde_json::to_value(ReplayAssessmentView::lth("1d", 252, timed.len())).unwrap();
    assert_eq!(assessment["performanceStatus"], "notEvaluable");
    assert_eq!(assessment["intradayBars"], 0);
    assert_eq!(assessment["dailyContextBars"], 252);
}

#[test]
fn us_minute_preparation_same_instant_keeps_ny_date_and_daily_warmup() {
    use chrono::{Duration, TimeZone};
    let ny = chrono_tz::America::New_York;
    let kst = chrono_tz::Asia::Seoul;
    let mut mismatches = Vec::new();
    // Winter's 05:30 KST is still the prior NY trading day's regular session.
    // Summer control covers the same NY close-labelled minute under DST.
    for (month, day) in [(1, 7), (7, 7)] {
        let instant = ny.with_ymd_and_hms(2026, month, day, 15, 30, 0).unwrap();
        let expected_date = instant.format("%Y%m%d").to_string();
        let daily = [-1, 0, 1]
            .into_iter()
            .map(|delta| BrokerCandle {
                symbol: BrokerSymbol("KORU".into()),
                market: BrokerMarket::Us,
                date: (instant.date_naive() + Duration::days(delta))
                    .format("%Y%m%d")
                    .to_string(),
                open: BrokerMoney::usd("100"),
                high: BrokerMoney::usd("101"),
                low: BrokerMoney::usd("99"),
                close: BrokerMoney::usd("100"),
                volume: BrokerQuantity("1000".into()),
            })
            .collect::<Vec<_>>();
        assert_eq!(daily_warmup_end_before(&daily, &expected_date), 1);
        for label in [
            instant.to_utc().to_rfc3339(),
            instant.to_rfc3339(),
            instant.with_timezone(&kst).to_rfc3339(),
        ] {
            assert_eq!(
                chrono::DateTime::parse_from_rfc3339(&label).unwrap(),
                instant
            );
            let mut source = daily[1].clone();
            source.date = label.clone();
            // Same production preparation chain used before LTH initialization.
            // The async profile entrypoint requires broker I/O; intentionally not called here.
            let prepared =
                super::lth_regular::prepare(&[source], &daily, 1, instant.to_utc()).unwrap();
            let date = minute_replay_trading_date(&prepared.timed[0].time, true, "regular");
            assert_eq!(prepared.sessions[0].date.format("%Y%m%d").to_string(), date);
            let warmup = prepared.warmup.len();
            if date != expected_date || warmup != 1 {
                mismatches.push((label, expected_date.clone(), date, warmup));
            }
        }
    }
    assert!(mismatches.is_empty(),
        "US preparation expected NY date and one prior daily warmup; (input, expected, actual, warmup): {mismatches:#?}");
}

fn a2_us_bar(label: &str, price: u64) -> BrokerCandle {
    BrokerCandle {
        symbol: BrokerSymbol("KORU".into()),
        market: BrokerMarket::Us,
        date: label.into(),
        open: BrokerMoney::usd(price.to_string()),
        high: BrokerMoney::usd(price.to_string()),
        low: BrokerMoney::usd(price.to_string()),
        close: BrokerMoney::usd(price.to_string()),
        volume: BrokerQuantity("1000".into()),
    }
}

fn a2_us_params() -> crate::trading::strategy::LeveragedTrendHoldParams {
    let mut params: crate::trading::strategy::LeveragedTrendHoldParams = serde_json::from_value(
        serde_json::json!({"entries":[{"leveraged_symbol":"KORU","quantity":1,"is_overseas":true}]})
    ).unwrap();
    params.intraday_rebound_enabled = true;
    params.rapid_rebound_enabled = false;
    params.rebound_baseline_ticks = 2;
    params.rebound_confirm_ticks = 2;
    params.rebound_pullback_pct = 4.0;
    params.rebound_buy_pressure_pct = 2.0;
    params.rebound_rsi_min = 20.0;
    params.no_trade_adx_below = 0.0;
    params.entry_window_start_min = 10000;
    params.entry_window_end_min = 10001;
    params
}

// Same production preparation and execution helpers used after the async fetch.
// No provider/profile/account construction is needed for synthetic input.
fn a2_us_run(
    source: &[BrokerCandle],
    daily: &[BrokerCandle],
    as_of: chrono::DateTime<chrono::Utc>,
    params: crate::trading::strategy::LeveragedTrendHoldParams,
) -> (super::lth_regular::Prepared, serde_json::Value) {
    use crate::trading::simulation::SimulationEvent;
    let prepared = super::lth_regular::prepare(source, daily, 200, as_of).unwrap();
    let events = prepared
        .timed
        .iter()
        .map(|row| SimulationEvent {
            time: row.time.clone(),
            chart_time: row.time.clone(),
            open_units: row.candle.open,
            high_units: row.candle.high,
            low_units: row.candle.low,
            close_units: row.candle.close,
            signal: None,
        })
        .collect::<Vec<_>>();
    let (signals, session) = super::lth_regular::execute(
        "1m",
        "KORU",
        params,
        &super::broker_candles_to_ohlc(&prepared.warmup),
        &prepared.timed,
        &events,
        true,
        SimulationAssumptions::default(),
    );
    (
        prepared,
        serde_json::json!({"signals": signals, "backtest": session.finish()}),
    )
}

#[test]
fn us_regular_production_preparation_and_execution_admission_boundaries() {
    use crate::trading::strategy::lth_research_session::Exclusion;
    use chrono::Duration;
    let cases = [
        ("2026-01-07T09:30:00-05:00", Some(Exclusion::OutsideRegular)),
        ("2026-01-07T09:30:59-05:00", Some(Exclusion::OutsideRegular)),
        ("2026-01-07T09:31:00-05:00", None),
        ("2026-01-07T16:00:00-05:00", None),
        ("2026-01-07T16:00:01-05:00", Some(Exclusion::OutsideRegular)),
        ("2026-11-27T13:00:00-05:00", None),
        ("2026-11-27T13:00:01-05:00", Some(Exclusion::OutsideRegular)),
        ("2026-07-02T16:00:00-04:00", None), // no invented half-day before July3
        ("2027-12-31T16:00:00-05:00", None), // no bank-calendar substitution
        ("2023-07-03T13:00:00-04:00", None),
        ("2024-12-24T13:00:00-05:00", None),
        ("2025-12-24T13:00:00-05:00", None),
        ("2025-12-25T12:00:00-05:00", Some(Exclusion::PlannedHoliday)),
        ("2026-04-03T12:00:00-04:00", Some(Exclusion::PlannedHoliday)),
        (
            "2025-01-09T12:00:00-05:00",
            Some(Exclusion::ExtraordinaryClosure),
        ),
        ("2026-07-11T12:00:00-04:00", Some(Exclusion::Weekend)),
        (
            "2022-12-30T12:00:00-05:00",
            Some(Exclusion::CalendarUnknown),
        ),
        (
            "2029-01-02T12:00:00-05:00",
            Some(Exclusion::CalendarUnknown),
        ),
    ];
    for (label, reason) in cases {
        let instant = chrono::DateTime::parse_from_rfc3339(label)
            .unwrap()
            .to_utc();
        let (prepared, result) = a2_us_run(&[a2_us_bar(label, 100)], &[], instant, a2_us_params());
        assert_eq!(prepared.as_of, instant);
        assert!(prepared.calendar_source.contains("2023-2028"));
        match reason {
            Some(reason) => {
                assert!(prepared.timed.is_empty(), "{label}");
                assert_eq!(prepared.excluded, vec![(0, reason)]);
            }
            None => {
                assert_eq!(prepared.timed.len(), 1, "{label}");
                assert!(prepared.excluded.is_empty());
            }
        }
        assert_eq!(
            result["backtest"]["equityCurve"].as_array().unwrap().len(),
            prepared.timed.len()
        );
        assert_eq!(result["signals"], serde_json::json!([])); // genuinely insufficient, no synthetic seed
        let assessment =
            super::lth_preview_assessment("1m", 0, 1, 200, &prepared.timed, true, instant);
        assert_eq!(assessment.timestamp_status, "unverified");
        assert_eq!(assessment.session_status, "unverified");
        if reason.is_some() {
            assert_eq!(assessment.performance_status, "notEvaluable");
        }
    }
    let label = "2026-01-07T09:31:00-05:00";
    let instant = chrono::DateTime::parse_from_rfc3339(label)
        .unwrap()
        .to_utc();
    for delta in [-1, 0, 1] {
        let (prepared, _) = a2_us_run(
            &[a2_us_bar(label, 100)],
            &[],
            instant + Duration::seconds(delta),
            a2_us_params(),
        );
        assert_eq!(prepared.timed.len(), usize::from(delta >= 0));
        if delta < 0 {
            assert_eq!(prepared.excluded, vec![(0, Exclusion::Future)]);
        }
    }
}

#[test]
fn us_regular_production_replay_excluded_extremes_cannot_contaminate_warmup_or_fills() {
    use crate::trading::strategy::lth_research_session::Exclusion;
    use chrono::{Duration, TimeZone};
    let ny = chrono_tz::America::New_York;
    let warmup = ny.with_ymd_and_hms(2026, 1, 6, 12, 30, 0).unwrap();
    let open = ny.with_ymd_and_hms(2026, 1, 7, 9, 31, 0).unwrap();
    let exit = ny.with_ymd_and_hms(2026, 1, 7, 15, 55, 0).unwrap();
    let mut source = (0..90)
        .map(|i| {
            a2_us_bar(
                &(warmup + Duration::minutes(i)).to_rfc3339(),
                102 + i as u64,
            )
        })
        .collect::<Vec<_>>();
    source.extend(
        [190, 180, 181, 186]
            .into_iter()
            .enumerate()
            .map(|(i, p)| a2_us_bar(&(open + Duration::minutes(i as i64)).to_rfc3339(), p)),
    );
    source.push(a2_us_bar(&exit.to_rfc3339(), 186));
    let daily = ["20260105", "20260106", "20260107", "20260108"].map(|label| a2_us_bar(label, 100));
    let snapshot = serde_json::to_value(&source).unwrap();
    let (prepared, baseline) = a2_us_run(&source, &daily, exit.to_utc(), a2_us_params());
    assert_eq!(prepared.warmup.len(), 1);
    assert_eq!(baseline["signals"].as_array().unwrap().len(), 2);
    assert_eq!(baseline["signals"][0]["side"], "buy");
    assert_eq!(baseline["signals"][1]["side"], "sell");
    assert_eq!(baseline["backtest"]["summary"]["filledOrderCount"], 2);
    assert_eq!(baseline["backtest"]["summary"]["completedTrades"], 1);
    let bad_labels = [
        "2026-01-07T09:30:00-05:00",
        "2026-01-06T16:01:00-05:00",
        "2026-01-01T12:00:00-05:00",
        "2025-01-09T12:00:00-05:00",
        "2026-01-07T15:56:00-05:00",
        "2029-01-02T12:00:00-05:00",
        "bad-20260107100000",
    ];
    for extreme in [1, 9999999] {
        let mut contaminated = source.clone();
        contaminated.extend(bad_labels.iter().map(|label| a2_us_bar(label, extreme)));
        contaminated.reverse(); // preparation sorts instants, not source labels
        let contaminated_snapshot = serde_json::to_value(&contaminated).unwrap();
        let (actual, result) = a2_us_run(&contaminated, &daily, exit.to_utc(), a2_us_params());
        assert_eq!(result, baseline);
        assert_eq!(actual.sessions, prepared.sessions);
        assert_eq!(
            serde_json::to_value(&actual.warmup).unwrap(),
            serde_json::to_value(&prepared.warmup).unwrap()
        );
        assert_eq!(actual.excluded.len(), bad_labels.len());
        assert!(actual.excluded.iter().any(|(_, r)| *r == Exclusion::Future));
        assert_eq!(
            serde_json::to_value(&contaminated).unwrap(),
            contaminated_snapshot
        );
        assert_eq!(actual.as_of, prepared.as_of);
    }
    assert_eq!(
        a2_us_run(&source, &daily, exit.to_utc(), a2_us_params()).1,
        baseline
    );
    assert_eq!(serde_json::to_value(&source).unwrap(), snapshot);
    // Conflicting prices at one instant: reject both, with no arbitrary winner.
    let mut duplicate = source.clone();
    duplicate.push(a2_us_bar(&open.to_utc().to_rfc3339(), 9999999));
    let (dup, _) = a2_us_run(&duplicate, &daily, exit.to_utc(), a2_us_params());
    assert_eq!(
        dup.excluded
            .iter()
            .filter(|(_, r)| *r == Exclusion::DuplicateInstant)
            .count(),
        2
    );
    assert_eq!(dup.timed.len(), source.len() - 1);
    // Live session selection does not authorize extended research.
    for policy in ["auto", "day", "pre", "regular", "after"] {
        let mut params = a2_us_params();
        params.toss_us_session = policy.into();
        assert_eq!(
            a2_us_run(&source, &daily, exit.to_utc(), params).1,
            baseline
        );
    }
}

#[test]
fn us_regular_production_dst_offsets_warmup_and_clock_agree() {
    use chrono::{Duration, FixedOffset, TimeZone};
    let ny = chrono_tz::America::New_York;
    for (month, previous, day) in [
        (1, 6, 7),
        (3, 5, 6),
        (3, 6, 9),
        (7, 6, 7),
        (10, 29, 30),
        (11, 2, 3),
        (11, 25, 27),
    ] {
        let start = ny
            .with_ymd_and_hms(2026, month, previous, 12, 30, 0)
            .unwrap();
        let open = ny.with_ymd_and_hms(2026, month, day, 9, 31, 0).unwrap();
        let (hour, minute) = if month == 11 && day == 27 {
            (13, 0)
        } else {
            (15, 55)
        };
        let exit = ny
            .with_ymd_and_hms(2026, month, day, hour, minute, 0)
            .unwrap();
        let mut rows = (0..90)
            .map(|i| a2_us_bar(&(start + Duration::minutes(i)).to_rfc3339(), 102 + i as u64))
            .collect::<Vec<_>>();
        rows.extend(
            [190, 180, 181, 186]
                .into_iter()
                .enumerate()
                .map(|(i, p)| a2_us_bar(&(open + Duration::minutes(i as i64)).to_rfc3339(), p)),
        );
        rows.push(a2_us_bar(&exit.to_rfc3339(), 186));
        let daily = [start, open, exit + Duration::days(1)]
            .map(|time| a2_us_bar(&time.format("%Y%m%d").to_string(), 100));
        let (base_prepared, base) = a2_us_run(&rows, &daily, exit.to_utc(), a2_us_params());
        assert_eq!(
            base_prepared.warmup.len(),
            0,
            "first admitted minute's day must not warm itself"
        );
        assert_eq!(
            base["signals"].as_array().unwrap().len(),
            2,
            "{month}/{day}"
        );
        assert_eq!(base["backtest"]["summary"]["filledOrderCount"], 2);
        let canonical = |mut value: serde_json::Value| {
            for signal in value["signals"].as_array_mut().unwrap() {
                let instant =
                    chrono::DateTime::parse_from_rfc3339(signal["time"].as_str().unwrap()).unwrap();
                signal["time"] = serde_json::json!(instant.to_utc().to_rfc3339());
            }
            (
                value["signals"].clone(),
                value["backtest"]["summary"].clone(),
            )
        };
        for offset in [0, -4 * 3600, -5 * 3600, 9 * 3600] {
            let mut alternate = rows.clone();
            for row in &mut alternate {
                let instant = chrono::DateTime::parse_from_rfc3339(&row.date).unwrap();
                row.date = instant
                    .with_timezone(&FixedOffset::east_opt(offset).unwrap())
                    .to_rfc3339();
            }
            alternate.reverse();
            let (prepared, result) = a2_us_run(&alternate, &daily, exit.to_utc(), a2_us_params());
            assert_eq!(prepared.sessions, base_prepared.sessions);
            assert_eq!(canonical(result), canonical(base.clone()));
        }
        // Legacy compact labels are strictly KST; the same NY dates still apply.
        let mut compact = rows.clone();
        for row in &mut compact {
            row.date = chrono::DateTime::parse_from_rfc3339(&row.date)
                .unwrap()
                .with_timezone(&chrono_tz::Asia::Seoul)
                .format("%Y%m%d%H%M%S")
                .to_string();
        }
        let (prepared, result) = a2_us_run(&compact, &daily, exit.to_utc(), a2_us_params());
        assert_eq!(prepared.sessions, base_prepared.sessions);
        assert_eq!(result["backtest"]["summary"], base["backtest"]["summary"]);
        assert_eq!(result["signals"].as_array().unwrap().len(), 2);
    }
}

// Explicit synthetic provider data only; this adapter performs no session filtering.
struct PagedFetchAdapter {
    provider: Vec<BrokerCandle>,
    daily: Vec<BrokerCandle>,
    legacy_calls: std::sync::Mutex<Vec<String>>,
    page_calls: std::sync::Mutex<Vec<crate::broker::CandlePageRequest>>,
}

#[async_trait::async_trait]
impl crate::broker::BrokerAdapter for PagedFetchAdapter {
    fn broker_id(&self) -> crate::broker::BrokerId {
        crate::broker::BrokerId::Toss
    }

    async fn get_price(
        &self,
        _: &BrokerSymbol,
    ) -> crate::broker::BrokerAdapterResult<crate::broker::BrokerPriceQuote> {
        panic!("synthetic pagination fixture must not request prices")
    }

    async fn get_candles(
        &self,
        symbol: &BrokerSymbol,
        period: &str,
        from: &str,
        to: &str,
    ) -> crate::broker::BrokerAdapterResult<Vec<BrokerCandle>> {
        assert_eq!(symbol.0, "KORU");
        assert_eq!((from, to), ("", ""));
        self.legacy_calls.lock().unwrap().push(period.into());
        Ok(match period {
            "D" => self.daily.clone(),
            "1m" => self.provider.iter().take(200).cloned().collect(),
            _ => panic!("unexpected fixture interval: {period}"),
        })
    }

    async fn get_candle_page(
        &self,
        request: &crate::broker::CandlePageRequest,
    ) -> crate::broker::BrokerAdapterResult<crate::broker::BrokerCandlePage> {
        use crate::broker::{BrokerCandlePage, CandleCursor};
        request.validate()?;
        assert_eq!(request.symbol.0, "KORU");
        assert_eq!(request.market, BrokerMarket::Us);
        assert!(matches!(request.interval.as_str(), "D" | "1m"));
        self.page_calls.lock().unwrap().push(request.clone());
        let provider = if request.interval == "D" {
            &self.daily
        } else {
            &self.provider
        };
        let start = match &request.cursor {
            None => 0,
            Some(CandleCursor::Toss { before }) => {
                let bound = chrono::DateTime::parse_from_rfc3339(before).unwrap();
                provider
                    .iter()
                    .position(|row| {
                        chrono::DateTime::parse_from_rfc3339(&row.date).unwrap() <= bound
                    })
                    .unwrap_or(provider.len())
            }
            _ => panic!("wrong provider cursor"),
        };
        let end = (start + usize::from(request.count)).min(provider.len());
        let candles = provider[start..end].to_vec();
        let next_cursor = (end < provider.len()).then(|| CandleCursor::Toss {
            before: candles.last().unwrap().date.clone(),
        });
        Ok(BrokerCandlePage {
            candles,
            end_reason: next_cursor.is_none().then(|| "fixture exhausted".into()),
            next_cursor,
            timestamp_semantics: if request.interval == "D" {
                "tradingDate"
            } else {
                "barClose"
            },
        })
    }
}

#[tokio::test]
async fn lth_fetch_pagination_restores_the_requested_regular_evaluation_cohort() {
    use crate::broker::{BrokerAdapter, CandlePageRequest};
    use chrono::{Duration, TimeZone};
    let ny = chrono_tz::America::New_York;
    let close = ny.with_ymd_and_hms(2026, 1, 7, 16, 0, 0).unwrap();
    let fixed_as_of = ny.with_ymd_and_hms(2026, 1, 7, 23, 0, 0).unwrap().to_utc();
    // Independent expected cohort: 200 close-labelled regular minutes, no prepare clone.
    let reference: Vec<_> = (0..200)
        .map(|i| {
            a2_us_bar(
                &(close - Duration::minutes(199 - i)).to_rfc3339(),
                100 + i as u64,
            )
        })
        .collect();
    let after_hours: Vec<_> = (1..=80)
        .map(|i| {
            a2_us_bar(
                &(close + Duration::minutes(i)).to_rfc3339(),
                10000 + i as u64,
            )
        })
        .collect();
    assert_eq!(reference.len(), 200);
    assert_eq!(after_hours.len(), 80);
    assert_eq!(reference.last().unwrap().date, close.to_rfc3339());
    for row in reference.iter().chain(&after_hours) {
        let instant = chrono::DateTime::parse_from_rfc3339(&row.date).unwrap();
        assert_eq!(instant.with_timezone(&ny).date_naive(), close.date_naive());
        assert!(instant.to_utc() < fixed_as_of);
        assert_eq!(row.symbol.0, "KORU");
        assert_eq!(row.market, BrokerMarket::Us);
    }
    let fake = PagedFetchAdapter {
        provider: reference
            .iter()
            .chain(&after_hours)
            .rev()
            .cloned()
            .collect(),
        daily: vec![a2_us_bar("2026-01-06T00:00:00-05:00", 99)],
        legacy_calls: Default::default(),
        page_calls: Default::default(),
    };
    // Verify provider contract separately; no collector or regular-session algorithm here.
    let mut request = CandlePageRequest {
        symbol: BrokerSymbol("KORU".into()),
        market: BrokerMarket::Us,
        interval: "1m".into(),
        count: 200,
        exchange: None,
        cursor: None,
    };
    let first = fake.get_candle_page(&request).await.unwrap();
    assert_eq!(first.candles.len(), 200);
    request.cursor = first.next_cursor.clone();
    assert!(request.cursor.is_some());
    let second = fake.get_candle_page(&request).await.unwrap();
    assert_eq!(second.candles.len(), 81);
    assert_eq!(first.candles.last(), second.candles.first());
    assert!(second.next_cursor.is_none());
    assert_eq!(
        fake.page_calls
            .lock()
            .unwrap()
            .iter()
            .map(|r| r.cursor.clone())
            .collect::<Vec<_>>(),
        vec![None, request.cursor.clone()]
    );
    fake.page_calls.lock().unwrap().clear();

    // Exercise the actual helper with the same fixed cutoff/count as production.
    let symbol = BrokerSymbol("KORU".into());
    let (daily, minute) = super::fetch_leveraged_trend_hold_candles(
        &fake,
        &symbol,
        "1m",
        Some(super::lth_collection::Context::new(fixed_as_of, 200)),
    )
    .await
    .unwrap();
    let prepared = super::lth_regular::prepare(&minute, &daily, 200, fixed_as_of).unwrap();
    assert!(fake.legacy_calls.lock().unwrap().is_empty());
    let calls = fake.page_calls.lock().unwrap();
    assert_eq!(calls.len(), 3);
    assert_eq!(
        calls
            .iter()
            .map(|r| r.interval.as_str())
            .collect::<Vec<_>>(),
        vec!["D", "1m", "1m"]
    );
    assert!(calls.iter().all(|r| r.count == 200));
    let seed = Some(crate::broker::CandleCursor::Toss {
        before: fixed_as_of.to_rfc3339(),
    });
    assert_eq!(calls[0].cursor, seed);
    assert_eq!(calls[1].cursor, seed);
    assert_eq!(calls[2].cursor, request.cursor);
    drop(calls);
    assert_eq!(prepared.warmup.len(), 1); // Daily context is separate from intraday EMA20/60.
    assert_eq!(
        prepared.timed.len(),
        reference.len(),
        "pagination must replenish regular observations after excluding 80 after-hours rows"
    );
    for (actual, expected) in prepared.timed.iter().zip(&reference) {
        assert_eq!(
            chrono::DateTime::parse_from_rfc3339(&actual.time).unwrap(),
            chrono::DateTime::parse_from_rfc3339(&expected.date).unwrap()
        );
        let expected_timed = broker_candles_to_timed_ohlc(&[expected.clone()], 1, "1m", 0, 0, 0);
        assert_eq!(actual.candle, expected_timed[0].candle);
        // Prepared has no volume field: check retained raw OHLCV at the admitted instant.
        let raw = minute
            .iter()
            .find(|row| {
                chrono::DateTime::parse_from_rfc3339(&row.date).unwrap()
                    == chrono::DateTime::parse_from_rfc3339(&actual.time).unwrap()
            })
            .unwrap();
        assert_eq!(
            (&raw.open, &raw.high, &raw.low, &raw.close, &raw.volume),
            (
                &expected.open,
                &expected.high,
                &expected.low,
                &expected.close,
                &expected.volume
            )
        );
    }
}
