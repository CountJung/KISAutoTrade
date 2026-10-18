use super::*;

fn candle(time: &str, close: u64) -> ChartCandle {
    ChartCandle {
        date: time.into(),
        open: close.to_string(),
        high: close.to_string(),
        low: close.to_string(),
        close: close.to_string(),
        volume: "1000".into(),
    }
}

fn input(times: &[&str]) -> StrategyPreviewInput {
    StrategyPreviewInput {
        strategy_id: "price_condition_minute_boundary".into(),
        strategy_name: "분봉 시각 경계 fixture".into(),
        symbol: "005930".into(),
        is_overseas: false,
        order_quantity: 1,
        params: serde_json::json!({"symbols": [{
            "symbol": "005930", "quantity": 1,
            "buy_trigger_price": 70_000, "sell_trigger_price": 80_000,
            "take_profit_pct": 5, "stop_loss_pct": 3, "is_overseas": false
        }]}),
        candles: times
            .iter()
            .enumerate()
            .map(|(index, time)| candle(time, if index % 2 == 0 { 69_000 } else { 81_000 }))
            .collect(),
        history_candles: vec![],
        warmup_count: None,
        interval: Some("1m".into()),
        data_source: Some("providerPageFixture".into()),
        strategy_version: None,
        broker_id: Some(BrokerId::Toss),
        broker_account_id: Some("synthetic-account".into()),
        assumptions: SimulationAssumptions {
            initial_capital_krw: 1_000_000.0,
            max_position_ratio: 1.0,
            ..SimulationAssumptions::default()
        },
    }
}

#[test]
fn minute_provider_offsets_produce_identical_actual_replay_and_no_gap_fill() {
    // Deliberately nonadjacent bar END observations; no intermediate prices exist.
    let utc = [
        "2026-07-11T13:30:00.125Z",
        "2026-07-11T13:47:00.500Z",
        "2026-07-11T14:10:00Z",
    ];
    let offset = [
        "2026-07-11T09:30:00.125-04:00",
        "2026-07-11T22:47:00.500+09:00",
        "2026-07-11T16:10:00+02:00",
    ];
    let left = preview_strategy_from_candles(input(&utc)).unwrap();
    let right = preview_strategy_from_candles(input(&offset)).unwrap();
    assert_eq!(left.replay.input_hash, right.replay.input_hash);
    assert_eq!(left.candles.len(), 3);
    assert_eq!(left.backtest.equity_curve.len(), 3);
    assert!(!left.signals.is_empty());
    assert!(left.backtest.summary.filled_order_count > 0);
    assert_eq!(left.candles[0].date, "2026-07-11T13:30:00.125000000Z");
    assert_eq!(left.replay.data_start, left.candles[0].date);
    assert_eq!(left.replay.data_end, left.candles[2].date);
    let mut left_json = serde_json::to_value(left).unwrap();
    let mut right_json = serde_json::to_value(right).unwrap();
    left_json.as_object_mut().unwrap().remove("generatedAt");
    right_json.as_object_mut().unwrap().remove("generatedAt");
    assert_eq!(left_json, right_json);
}

#[test]
fn minute_replay_sorts_by_instant_rather_than_offset_text() {
    let mut fixture = input(&[
        "2026-07-11T22:30:00+09:00", // 13:30Z
        "2026-07-11T09:31:00-04:00", // 13:31Z
    ]);
    fixture.candles.reverse();
    let result = preview_strategy_from_candles(fixture).unwrap();
    assert_eq!(result.candles[0].date, "2026-07-11T13:30:00.000000000Z");
    assert_eq!(result.candles[0].close, "69000");
    assert_eq!(result.candles[1].date, "2026-07-11T13:31:00.000000000Z");
}

#[test]
fn minute_replay_rejects_same_instant_duplicates_and_history_overlap() {
    assert_eq!(
        preview_strategy_from_candles(input(&[
            "2026-07-11T13:30:00.125Z",
            "2026-07-11T22:30:00.125+09:00",
        ]))
        .unwrap_err()
        .code,
        "DUPLICATE_CANDLE"
    );
    for history_time in [
        "2026-07-11T22:30:00.125+09:00", // equal instant
        "2026-07-11T09:31:00-04:00",     // later despite earlier text
    ] {
        let mut fixture = input(&["2026-07-11T13:30:00.125Z"]);
        fixture.history_candles = vec![candle(history_time, 69_000)];
        assert_eq!(
            preview_strategy_from_candles(fixture).unwrap_err().code,
            "HISTORY_OVERLAP"
        );
    }
    let mut fixture = input(&["2026-07-11T09:30:00-04:00"]);
    fixture.history_candles = vec![candle("2026-07-11T22:29:00+09:00", 69_000)];
    assert_eq!(
        preview_strategy_from_candles(fixture)
            .unwrap()
            .replay
            .warmup_count,
        1
    );
}

#[test]
fn legacy_compact_minutes_are_explicit_kst_and_invalid_times_fail_closed() {
    let compact =
        preview_strategy_from_candles(input(&["202607112230", "20260711223100"])).unwrap();
    let offset = preview_strategy_from_candles(input(&[
        "2026-07-11T22:30:00+09:00",
        "2026-07-11T22:31:00+09:00",
    ]))
    .unwrap();
    assert_eq!(compact.replay.input_hash, offset.replay.input_hash);
    for invalid in [
        "2026-07-11T22:30:00", // timezone must be explicit
        "2026-07-11T25:30:00+09:00",
        "2026-02-30T22:30:00+09:00",
        "20260711226000",
        "20260711223000garbage",
    ] {
        assert_eq!(
            preview_strategy_from_candles(input(&[invalid]))
                .unwrap_err()
                .code,
            "INVALID_CANDLE",
            "{invalid}"
        );
    }
}

#[test]
fn daily_rows_keep_existing_trading_dates_and_hash_semantics() {
    let mut compact = input(&["20260711", "20260712"]);
    compact.interval = Some("D".into());
    let mut separated = input(&["2026-07-11", "2026-07-12"]);
    separated.interval = Some("D".into());
    let compact = preview_strategy_from_candles(compact).unwrap();
    let separated = preview_strategy_from_candles(separated).unwrap();
    assert_eq!(compact.replay.input_hash, separated.replay.input_hash);
    assert_eq!(compact.candles[0].date, "20260711");
    assert_eq!(compact.candles[1].date, "20260712");
    assert_eq!(compact.replay.replay_cadence, "dailyCloseWithDayBoundary");
}

fn market_midnight_fixture(overseas: bool) -> StrategyPreviewView {
    let offset = if overseas { "-04:00" } else { "+09:00" };
    let dates = [
        "2026-07-10T23:00:00",
        "2026-07-10T23:02:00",
        "2026-07-11T00:30:00",
        "2026-07-11T00:32:00",
    ];
    let times = dates.map(|date| format!("{date}{offset}"));
    let mut fixture = input(&times.iter().map(String::as_str).collect::<Vec<_>>());
    fixture.is_overseas = overseas;
    fixture.params["symbols"][0]["is_overseas"] = overseas.into();
    fixture.candles = times
        .iter()
        .zip([69_000, 65_000, 69_000, 81_000])
        .map(|(time, close)| candle(time, if overseas { close / 1000 } else { close }))
        .collect();
    if overseas {
        fixture.params["symbols"][0]["buy_trigger_price"] = 70.into();
        fixture.params["symbols"][0]["sell_trigger_price"] = 80.into();
    }
    fixture.assumptions.fee_bps = 0.0;
    fixture.assumptions.tax_bps = 0.0;
    fixture.assumptions.slippage_bps = 0.0;
    // The loss breaches today's limit, which must reset on the market's next day.
    fixture.assumptions.daily_loss_limit_krw = 1000.0;
    let result = preview_strategy_from_candles(fixture).unwrap();
    assert_eq!(result.backtest.summary.filled_order_count, 4);
    assert_eq!(result.backtest.summary.blocked_order_count, 0);
    result
}

#[test]
fn minute_replay_timezone_subprocess_fixture() {
    if std::env::var_os("KIS_REPLAY_TIMEZONE_FIXTURE").is_none() {
        return;
    }
    let views = [
        market_midnight_fixture(false),
        market_midnight_fixture(true),
    ];
    let mut value = serde_json::to_value(views).unwrap();
    for view in value.as_array_mut().unwrap() {
        view.as_object_mut().unwrap().remove("generatedAt");
    }
    println!("REPLAY_TIMEZONE_RESULT={value}");
}

#[test]
fn minute_replay_risk_guard_and_full_reports_are_independent_of_host_timezone() {
    let executable = std::env::current_exe().unwrap();
    let mut reference = None;
    for timezone in ["UTC", "Asia/Seoul", "America/New_York"] {
        let output = std::process::Command::new(&executable)
            .args(["--exact", "commands::strategy_preview::generic::tests::minute_replay_timezone_subprocess_fixture", "--nocapture"])
            .env("TZ", timezone)
            .env("KIS_REPLAY_TIMEZONE_FIXTURE", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{timezone}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let result: serde_json::Value = serde_json::from_str(
            stdout
                .lines()
                .find_map(|line| line.strip_prefix("REPLAY_TIMEZONE_RESULT="))
                .unwrap(),
        )
        .unwrap();
        if let Some(reference) = &reference {
            assert_eq!(reference, &result, "host timezone {timezone}");
        } else {
            reference = Some(result);
        }
    }
}

#[test]
fn next_open_settles_before_the_next_strategy_close_and_retains_final_pending() {
    use crate::trading::simulation::ExecutionModel;
    let mut fixture = input(&["20260706", "20260707", "20260708", "20260709"]);
    fixture.interval = Some("1d".into());
    fixture.assumptions.execution_model = ExecutionModel::NextOpen;
    fixture.assumptions.fee_bps = 0.0;
    fixture.assumptions.tax_bps = 0.0;
    fixture.assumptions.slippage_bps = 0.0;
    fixture.candles = [
        (69_000, 69_000),
        (75_000, 76_000),
        (76_000, 82_000),
        (81_000, 81_000),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (open, close))| ChartCandle {
        date: format!("202607{:02}", 6 + index),
        open: open.to_string(),
        close: close.to_string(),
        high: open.max(close).to_string(),
        low: open.min(close).to_string(),
        volume: "1000".into(),
    })
    .collect();
    let result = preview_strategy_from_candles(fixture).unwrap();
    assert!(result.replay.look_ahead_safe);
    assert_eq!(result.signals.len(), 2);
    assert_eq!(
        result.signals[1].time, "20260708",
        "entry must be the next open, not the previous close"
    );
    assert_eq!(result.backtest.trades[0].fill_price, Some(75_000.0));
    assert_eq!(result.backtest.trades[1].fill_price, Some(81_000.0));
    let first = &result.backtest.trades[0];
    assert_eq!(
        first.signal_available_at.as_ref().unwrap().bar_time,
        "20260706"
    );
    assert_eq!(first.signal_available_at.as_ref().unwrap().phase, "close");
    assert_eq!(first.fill_at.as_ref().unwrap().bar_time, "20260707");
    assert_eq!(first.fill_at.as_ref().unwrap().phase, "open");
    assert_eq!(result.backtest.summary.unfilled_order_count, 0);

    let mut pending = input(&["20260706"]);
    pending.interval = Some("1d".into());
    pending.assumptions.execution_model = ExecutionModel::NextOpen;
    let pending = preview_strategy_from_candles(pending).unwrap();
    assert_eq!(pending.backtest.summary.filled_order_count, 0);
    assert_eq!(pending.backtest.summary.blocked_order_count, 0);
    assert_eq!(pending.backtest.summary.unfilled_order_count, 1);
    assert_eq!(pending.backtest.trades[0].status, "unfilled");
    assert!(pending.backtest.trades[0].fill_at.is_none());
}

#[test]
fn future_open_cannot_change_an_earlier_signal_or_fill() {
    use crate::trading::simulation::ExecutionModel;
    let build = |future| {
        let mut fixture = input(&["20260706", "20260707", "20260708"]);
        fixture.interval = Some("1d".into());
        fixture.assumptions.execution_model = ExecutionModel::NextOpen;
        fixture.candles[2] = candle("20260708", future);
        preview_strategy_from_candles(fixture).unwrap()
    };
    let left = build(40_000);
    let right = build(140_000);
    assert_eq!(
        serde_json::to_value(&left.signals[0]).unwrap(),
        serde_json::to_value(&right.signals[0]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&left.backtest.trades[0]).unwrap(),
        serde_json::to_value(&right.backtest.trades[0]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&left.backtest.equity_curve[..2]).unwrap(),
        serde_json::to_value(&right.backtest.equity_curve[..2]).unwrap()
    );
}

#[test]
fn frozen_qqq_cost_and_next_open_feedback_change_the_real_strategy_exit_path() {
    use crate::trading::simulation::ExecutionModel;
    let data: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../docs/research/strategy-review-20261018/input.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let dataset = data["datasets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["symbol"] == "QQQ")
        .unwrap();
    let bars: Vec<ChartCandle> = serde_json::from_value(dataset["candles"].clone()).unwrap();
    for (cost, model, expected_return, exit_signal, exit_fill) in [
        (
            10.0,
            ExecutionModel::SameClose,
            0.6274,
            "20260921",
            "20260921",
        ),
        (
            20.0,
            ExecutionModel::SameClose,
            -0.5659,
            "20260729",
            "20260729",
        ),
        (
            10.0,
            ExecutionModel::NextOpen,
            -0.4372,
            "20260729",
            "20260730",
        ),
        (
            20.0,
            ExecutionModel::NextOpen,
            -0.4772,
            "20260729",
            "20260730",
        ),
    ] {
        let result = preview_strategy_from_candles(StrategyPreviewInput {
            strategy_id: "mean_reversion".into(),
            strategy_name: "Frozen QQQ feedback".into(),
            symbol: "QQQ".into(),
            is_overseas: true,
            order_quantity: 1,
            params: serde_json::to_value(crate::trading::strategy::MeanReversionParams::default())
                .unwrap(),
            candles: bars[1179..1243].to_vec(),
            history_candles: bars[927..1179].to_vec(),
            warmup_count: None,
            interval: Some("1d".into()),
            data_source: Some("frozen fixture; calendar coverage unverified".into()),
            strategy_version: None,
            broker_id: None,
            broker_account_id: None,
            assumptions: SimulationAssumptions {
                execution_model: model,
                fee_bps: cost,
                slippage_bps: cost,
                tax_bps: 0.0,
                ..SimulationAssumptions::default()
            },
        })
        .unwrap();
        assert_eq!(
            result.backtest.summary.cumulative_return_pct,
            expected_return
        );
        let exit = &result.backtest.trades[1];
        assert_eq!(
            exit.signal_available_at.as_ref().unwrap().bar_time,
            exit_signal
        );
        assert_eq!(exit.fill_at.as_ref().unwrap().bar_time, exit_fill);
        assert_eq!(exit.reason.contains("손절"), exit_signal == "20260729");
        assert_eq!(result.backtest.summary.completed_trades, 1);
    }
}
