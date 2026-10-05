use kis_auto_trade_lib::{
    api::rest::ChartCandle,
    commands::{preview_strategy_from_candles, StrategyPreviewInput},
    trading::simulation::{
        run_backtest, BacktestReportView, SimulationAssumptions, SimulationEvent,
    },
    trading::strategy::*,
};
use serde_json::{json, Value};
use std::{error::Error, fs};
fn ohlc(c: &ChartCandle) -> OhlcCandle {
    let units = |s: &str| (s.parse::<f64>().unwrap() * 100.0).round() as u64;
    OhlcCandle {
        open: units(&c.open),
        high: units(&c.high),
        low: units(&c.low),
        close: units(&c.close),
    }
}
fn event(c: &ChartCandle) -> SimulationEvent {
    let p = ohlc(c);
    SimulationEvent {
        time: c.date.clone(),
        chart_time: c.date.clone(),
        close_units: p.close,
        high_units: p.high,
        low_units: p.low,
        signal: None,
    }
}
fn assumptions() -> SimulationAssumptions {
    let cost_bps = std::env::var("SOXQ_COST_BPS")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(10.0);
    SimulationAssumptions {
        fee_bps: cost_bps,
        tax_bps: 0.0,
        slippage_bps: cost_bps,
        ..SimulationAssumptions::default()
    }
}
fn lth(candles: &[ChartCandle], warmup: usize, params: LeveragedTrendHoldParams) -> Value {
    // Daily input cannot reconstruct minute indicators or entry-time prices.
    // Retain chart/equity diagnostics but never call the intraday strategy engine.
    let events: Vec<_> = candles[warmup..].iter().map(event).collect();
    let report = run_backtest("leveraged_trend_hold", "SOXQ", true, assumptions(), &events);
    let assessment =
        kis_auto_trade_lib::trading::simulation::ReplayAssessmentView::lth("1d", warmup, 0);
    json!({"strategyId":"leveraged_trend_hold","params":params,"warmupCount":0,"dailyContextCount":warmup,"syntheticDailySession":false,"signals":[],"assessment":assessment,"backtest":report})
}
fn benchmark(candles: &[ChartCandle], qty: u64, all_capital: bool) -> BacktestReportView {
    let mut events: Vec<_> = candles.iter().map(event).collect();
    events[0].signal = Some(Signal::Buy {
        symbol: "SOXQ".into(),
        quantity: qty,
        reason: "First evaluation close buy-and-hold benchmark".into(),
    });
    let mut a = assumptions();
    if all_capital {
        a.max_position_ratio = 1.0;
    }
    run_backtest("buy_hold", "SOXQ", true, a, &events)
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let input: Value = serde_json::from_str(&fs::read_to_string(&args[1])?)?;
    let candles: Vec<ChartCandle> = serde_json::from_value(input["candles"].clone())?;
    let warmup = input["warmupCount"].as_u64().unwrap() as usize;
    let reference = candles[warmup - 1].close.parse::<f64>()?;
    let mut cases: Vec<(&str, Value)> = vec![
        ("ma_cross", serde_json::to_value(MaCrossParams::default())?),
        ("rsi", serde_json::to_value(RsiParams::default())?),
        ("momentum", serde_json::to_value(MomentumParams::default())?),
        (
            "deviation",
            serde_json::to_value(DeviationParams::default())?,
        ),
        (
            "fifty_two_week_high",
            serde_json::to_value(FiftyTwoWeekHighParams::default())?,
        ),
        (
            "consecutive_move",
            serde_json::to_value(ConsecutiveMoveParams::default())?,
        ),
        (
            "failed_breakout",
            serde_json::to_value(FailedBreakoutParams::default())?,
        ),
        (
            "strong_close",
            serde_json::to_value(StrongCloseParams::default())?,
        ),
        (
            "volatility_expansion",
            serde_json::to_value(VolatilityExpansionParams::default())?,
        ),
        (
            "mean_reversion",
            serde_json::to_value(MeanReversionParams::default())?,
        ),
        (
            "trend_filter",
            serde_json::to_value(TrendFilterParams::default())?,
        ),
        (
            "price_condition",
            json!({"symbols":[{"symbol":"SOXQ","quantity":10,"buy_trigger_price":reference,"take_profit_pct":5.0,"stop_loss_pct":3.0,"is_overseas":true}]}),
        ),
    ];
    let lp: LeveragedTrendHoldParams = serde_json::from_value(
        json!({"entries":[{"leveraged_symbol":"SOXQ","quantity":10,"is_overseas":true}]}),
    )?;
    // Verify generic LTH is refused; only the timed specialized path evaluates it.
    cases.push(("leveraged_trend_hold", serde_json::to_value(&lp)?));
    let mut runs = Vec::new();
    let mut rejected_runs = Vec::new();
    for (id, params) in cases {
        for (mode, history) in [
            ("historical_warmup", candles[..warmup].to_vec()),
            ("ui_three_month_defaults", Vec::new()),
        ] {
            let preview = preview_strategy_from_candles(StrategyPreviewInput {
                strategy_id: id.into(),
                strategy_name: id.into(),
                symbol: "SOXQ".into(),
                is_overseas: true,
                order_quantity: 10,
                params: params.clone(),
                candles: candles[warmup..].to_vec(),
                history_candles: history,
                warmup_count: None,
                interval: Some("1d".into()),
                data_source: Some(
                    "Yahoo Finance SOXQ unadjusted daily OHLCV frozen 2026-10-03".into(),
                ),
                strategy_version: None,
                broker_id: None,
                broker_account_id: None,
                assumptions: assumptions(),
            });
            match preview {
                Ok(p) => runs.push(json!({"mode":mode,"params":params,"preview":p})),
                Err(e)
                    if id == "leveraged_trend_hold" && e.code == "UNSUPPORTED_GENERIC_REPLAY" =>
                {
                    rejected_runs.push(json!({"mode":mode,"strategyId":id,"error":{"code":e.code,"message":e.message}}));
                }
                Err(e) => return Err(format!("{id} {mode}: {} {}", e.code, e.message).into()),
            }
        }
    }
    let evaluation = &candles[warmup..];
    let first = evaluation[0].close.parse::<f64>()?;
    let qty = (10_000_000.0 / (first * 1450.0 * 1.001 * 1.001)).floor() as u64;
    let output = json!({"input":input,"runs":runs,"rejectedRuns":rejected_runs,"lthSpecializedDaily":lth(&candles,warmup,lp),"benchmarks":{
        "sameQuantity10":benchmark(evaluation,10,false),"allCapital":benchmark(evaluation,qty,true)
    }});
    fs::write(&args[2], serde_json::to_string_pretty(&output)?)?;
    for row in output["runs"].as_array().unwrap() {
        let p = &row["preview"];
        let s = &p["backtest"]["summary"];
        println!(
            "{} {} start={} return={} mdd={} completed={} filled={} blocked={}",
            row["mode"],
            p["strategyId"],
            p["replay"]["dataStart"],
            s["cumulativeReturnPct"],
            s["mddPct"],
            s["completedTrades"],
            s["filledOrderCount"],
            s["blockedOrderCount"]
        );
    }
    println!(
        "LTH specialized: {}",
        output["lthSpecializedDaily"]["backtest"]["summary"]
    );
    Ok(())
}
