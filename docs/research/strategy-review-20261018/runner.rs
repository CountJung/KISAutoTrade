//! Offline research harness: production strategy replay, unchanged guards and frozen inputs.
//! This executable never loads account configuration or submits provider orders.

use kis_auto_trade_lib::{
    api::rest::ChartCandle,
    commands::{preview_strategy_from_candles, StrategyPreviewInput},
    trading::{
        simulation::{
            report_position_snapshot, run_backtest, BacktestReportView, ReplayAssessmentView,
            SimulationAssumptions, SimulationEvent,
        },
        strategy::*,
    },
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
};

type ResearchResult<T> = Result<T, Box<dyn Error>>;

fn text<'a>(value: &'a Value, field: &str) -> ResearchResult<&'a str> {
    value[field]
        .as_str()
        .ok_or_else(|| format!("missing string field {field}").into())
}

fn number(value: &Value, field: &str) -> ResearchResult<f64> {
    value[field]
        .as_f64()
        .filter(|n| n.is_finite() && *n > 0.0)
        .ok_or_else(|| format!("{field} must be finite and positive").into())
}

fn index(value: &Value, field: &str) -> ResearchResult<usize> {
    value[field]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("invalid index {field}").into())
}

fn price(raw: &str) -> ResearchResult<f64> {
    let value = raw.parse::<f64>()?;
    if !value.is_finite() || value <= 0.0 || value * 100.0 > u64::MAX as f64 {
        return Err(format!("invalid positive price {raw}").into());
    }
    Ok(value)
}

fn validate_candles(candles: &[ChartCandle]) -> ResearchResult<()> {
    if candles.is_empty() {
        return Err("dataset has no candles".into());
    }
    let mut previous = "";
    for c in candles {
        if c.date.len() != 8 || !c.date.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("daily date must be YYYYMMDD: {}", c.date).into());
        }
        if c.date.as_str() <= previous {
            return Err(format!("duplicate or nonchronological date {}", c.date).into());
        }
        let (o, h, l, close) = (
            price(&c.open)?,
            price(&c.high)?,
            price(&c.low)?,
            price(&c.close)?,
        );
        let volume = c.volume.parse::<f64>()?;
        if l > o.min(close) || h < o.max(close) || !volume.is_finite() || volume < 0.0 {
            return Err(format!("invalid OHLCV at {}", c.date).into());
        }
        previous = &c.date;
    }
    Ok(())
}

fn strategy_cases(
    symbol: &str,
    quantity: u64,
    reference_close: f64,
) -> ResearchResult<Vec<(&'static str, Value)>> {
    Ok(vec![
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
            json!({"symbols":[{"symbol":symbol,"quantity":quantity,
            "buy_trigger_price":reference_close,"take_profit_pct":5.0,
            "stop_loss_pct":3.0,"is_overseas":true}]}),
        ),
        (
            "leveraged_trend_hold",
            json!({"entries":[{"leveraged_symbol":symbol,
            "quantity":quantity,"is_overseas":true}]}),
        ),
    ])
}

fn benchmark(
    symbol: &str,
    candles: &[ChartCandle],
    quantity: u64,
    assumptions: SimulationAssumptions,
) -> ResearchResult<BacktestReportView> {
    let mut events = candles
        .iter()
        .map(|c| {
            Ok(SimulationEvent {
                time: c.date.clone(),
                chart_time: c.date.clone(),
                open_units: (price(&c.open)? * 100.0).round() as u64,
                close_units: (price(&c.close)? * 100.0).round() as u64,
                high_units: (price(&c.high)? * 100.0).round() as u64,
                low_units: (price(&c.low)? * 100.0).round() as u64,
                signal: None,
            })
        })
        .collect::<ResearchResult<Vec<_>>>()?;
    let first = events.first_mut().ok_or("empty benchmark window")?;
    first.signal = Some(Signal::Buy {
        symbol: symbol.into(),
        quantity,
        reason: "Buy and hold at first evaluation close, same fixed quantity and guards".into(),
    });
    Ok(run_backtest("buy_hold", symbol, true, assumptions, &events))
}

fn diagnostics(report: &BacktestReportView) -> Value {
    let realized_pnl = report
        .trades
        .iter()
        .filter_map(|trade| trade.realized_pnl_krw)
        .sum::<f64>();
    let mut blocked_reasons = BTreeMap::<&str, usize>::new();
    for trade in &report.trades {
        if trade.status == "blocked" {
            *blocked_reasons
                .entry(trade.blocked_reason.as_deref().unwrap_or("unspecified"))
                .or_default() += 1;
        }
    }
    let (held_quantity, _) = report_position_snapshot(report, true);
    json!({
        "finalEquityPoint":report.equity_curve.last(),
        "realizedPnlKrw":realized_pnl,
        // Includes remaining position mark-to-market and allocated entry costs.
        "residualPnlKrw":report.summary.final_equity_krw
            - report.summary.initial_capital_krw - realized_pnl,
        "heldQuantity":held_quantity,
        "blockedReasons":blocked_reasons,
        "signalCount":report.summary.signal_count,
        "completedTrades":report.summary.completed_trades
    })
}

fn write_row(writer: &mut impl Write, row: &Value) -> ResearchResult<()> {
    serde_json::to_writer(&mut *writer, row)?;
    writer.write_all(b"\n")?;
    Ok(())
}

fn ohlc(c: &ChartCandle) -> ResearchResult<OhlcCandle> {
    let result = OhlcCandle {
        open: (price(&c.open)? * 100.0).round() as u64,
        high: (price(&c.high)? * 100.0).round() as u64,
        low: (price(&c.low)? * 100.0).round() as u64,
        close: (price(&c.close)? * 100.0).round() as u64,
    };
    if result.low > result.open.min(result.close) || result.high < result.open.max(result.close) {
        return Err(format!("invalid minute OHLC at {}", c.date).into());
    }
    Ok(result)
}

fn minute_diagnostics(input: &Value, writer: &mut impl Write) -> ResearchResult<usize> {
    let Some(dataset) = input.get("minuteDataset") else {
        return Ok(0);
    };
    let symbol = text(dataset, "symbol")?;
    let candles: Vec<ChartCandle> = serde_json::from_value(dataset["candles"].clone())?;
    let daily: Vec<ChartCandle> = serde_json::from_value(dataset["dailyContext"].clone())?;
    validate_candles(&daily)?;
    if daily.len() != 252 || candles.is_empty() || candles.len() > 10_000 {
        return Err("minute diagnostic requires 252 daily context and 1..10000 minute bars".into());
    }
    let mut observations = Vec::with_capacity(candles.len());
    let mut previous = "";
    for candle in &candles {
        if candle.date.len() != 14
            || !candle.date.bytes().all(|b| b.is_ascii_digit())
            || candle.date.as_str() <= previous
        {
            return Err(format!("invalid or nonchronological minute time {}", candle.date).into());
        }
        let volume = candle.volume.parse::<f64>()?;
        if !volume.is_finite() || volume < 0.0 {
            return Err(format!("invalid minute volume at {}", candle.date).into());
        }
        observations.push(LeveragedTrendHoldTimedCandle {
            time: candle.date.clone(),
            candle: ohlc(candle)?,
        });
        previous = &candle.date;
    }
    if daily[251].date.as_str() >= &candles[0].date[..8] {
        return Err("daily context must strictly precede first minute date".into());
    }
    let context = daily.iter().map(ohlc).collect::<ResearchResult<Vec<_>>>()?;
    let config = &input["assumptions"];
    let reference_close = price(&daily[251].close)?;
    let quantity = (number(config, "targetEntryRatio")? * number(config, "initialCapitalKrw")?
        / (reference_close * number(config, "exchangeRateKrw")? * 1.002))
        .floor() as u64;
    if quantity == 0 {
        return Err("minute diagnostic quantity is zero".into());
    }
    let mut count = 0;
    for bollinger in [false, true] {
        let params: LeveragedTrendHoldParams = serde_json::from_value(json!({
            "entries":[{"leveraged_symbol":symbol,"quantity":quantity,"is_overseas":true}],
            "bollinger":{"enabled":bollinger}
        }))?;
        for cost in config["costBps"].as_array().ok_or("missing costBps")? {
            let cost_bps = cost
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 0.0 && *n <= 1000.0)
                .ok_or("invalid minute costBps")?;
            let assumptions = SimulationAssumptions {
                initial_capital_krw: number(config, "initialCapitalKrw")?,
                exchange_rate_krw: number(config, "exchangeRateKrw")?,
                fee_bps: cost_bps,
                slippage_bps: cost_bps,
                tax_bps: 0.0,
                ..SimulationAssumptions::default()
            };
            let mut events: Vec<_> = observations
                .iter()
                .map(|timed| SimulationEvent {
                    time: timed.time.clone(),
                    chart_time: timed.time.clone(),
                    open_units: timed.candle.open,
                    close_units: timed.candle.close,
                    high_units: timed.candle.high,
                    low_units: timed.candle.low,
                    signal: None,
                })
                .collect();
            let mut missing_event = false;
            let signals = LeveragedTrendHoldStrategy::preview_signals_with_execution(
                symbol,
                params.clone(),
                &context,
                &observations,
                |signal, timed| {
                    let Ok(i) = observations.binary_search_by(|c| c.time.cmp(&timed.time)) else {
                        missing_event = true;
                        return false;
                    };
                    events[i].signal = Some(if signal.side == "buy" {
                        Signal::Buy {
                            symbol: symbol.into(),
                            quantity: signal.quantity,
                            reason: signal.reason.clone(),
                        }
                    } else {
                        Signal::Sell {
                            symbol: symbol.into(),
                            quantity: signal.quantity,
                            reason: signal.reason.clone(),
                        }
                    });
                    let report = run_backtest(
                        "leveraged_trend_hold",
                        symbol,
                        true,
                        assumptions.clone(),
                        &events[..=i],
                    );
                    report
                        .trades
                        .last()
                        .is_some_and(|trade| trade.time == timed.time && trade.status == "filled")
                },
            );
            if missing_event {
                return Err("LTH feedback event was not found".into());
            }
            let report = run_backtest(
                "leveraged_trend_hold",
                symbol,
                true,
                assumptions.clone(),
                &events,
            );
            // This harness is not the provider command with a 200-bar fetch cap.
            // Keep the production status vocabulary but supply dataset-specific limitations.
            let mut assessment = ReplayAssessmentView::lth("1m", daily.len(), observations.len());
            assessment.limitations = vec![
                "최근 7개 세션의 누락 분봉이 있는 표본이며 3개월 평가가 아니다".into(),
                "UTC 원본을 KST 봉 시작 시각으로 변환했으며 종가 공개 시점과 체결 시각은 검증되지 않았다".into(),
                "엔진의 고정 KST 세션을 유지했다. 거래소 DST·휴장·조기폐장과 live 10초 관측 동등성은 검증되지 않았다".into(),
            ];
            write_row(
                writer,
                &json!({
                    "kind":"lthDiagnostic","strategyId":"leveraged_trend_hold","symbol":symbol,
                    "variant":if bollinger {"default_bollinger_enabled"} else {"default"},
                    "costBps":cost_bps,"orderQuantity":quantity,"referenceClose":reference_close,
                    "historyCount":daily.len(),"evaluationCount":observations.len(),
                    "evaluationStart":candles[0].date,"evaluationEnd":candles[candles.len()-1].date,
                    "params":params,"assumptions":assumptions,"assessment":assessment,
                    "sourceMetadata":dataset["metadata"],"signals":signals,
                    "diagnostics":diagnostics(&report),"backtest":report
                }),
            )?;
            count += 1;
        }
    }
    Ok(count)
}

fn main() -> ResearchResult<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: strategy-review INPUT.json FULL-OUTPUT.jsonl".into());
    }
    let input: Value = serde_json::from_slice(&fs::read(&args[1])?)?;
    let datasets = input["datasets"].as_array().ok_or("missing datasets")?;
    let windows = input["windows"].as_array().ok_or("missing windows")?;
    let config = &input["assumptions"];
    let target_ratio = number(config, "targetEntryRatio")?;
    let exchange_rate = number(config, "exchangeRateKrw")?;
    let initial_capital = number(config, "initialCapitalKrw")?;
    if target_ratio > 0.3 || windows.is_empty() {
        return Err(
            "entry ratio must not exceed unchanged 30% cap; windows cannot be empty".into(),
        );
    }
    let costs = config["costBps"].as_array().ok_or("missing costBps")?;
    if costs.is_empty() {
        return Err("costBps cannot be empty".into());
    }
    let mut data = BTreeMap::new();
    for dataset in datasets {
        let symbol = text(dataset, "symbol")?;
        let candles: Vec<ChartCandle> = serde_json::from_value(dataset["candles"].clone())?;
        validate_candles(&candles)?;
        if data.insert(symbol, candles).is_some() {
            return Err(format!("duplicate dataset symbol {symbol}").into());
        }
    }
    let mut writer = BufWriter::new(File::create(&args[2])?);
    let mut ids = HashSet::new();
    let mut row_count = 0;
    for window in windows {
        let id = text(window, "id")?;
        if !ids.insert(id) {
            return Err(format!("duplicate window id {id}").into());
        }
        let symbol = text(window, "symbol")?;
        let kind = text(window, "kind")?;
        if !matches!(kind, "quarter" | "latestLong") {
            return Err(format!("unsupported window kind {kind}").into());
        }
        let all_candles = data
            .get(symbol)
            .ok_or_else(|| format!("no dataset for {symbol}"))?;
        let h = index(window, "historyStart")?;
        let start = index(window, "evaluationStart")?;
        let end = index(window, "evaluationEndExclusive")?;
        if h >= start
            || start >= end
            || end > all_candles.len()
            || start - h != 252
            || end - start > 248
        {
            return Err(format!(
                "{id}: require 252 history + 1..248 evaluation bars within dataset"
            )
            .into());
        }
        let history = &all_candles[h..start];
        let evaluation = &all_candles[start..end];
        let reference_close = price(&history[251].close)?;
        let quantity = (target_ratio * initial_capital / (reference_close * exchange_rate * 1.002))
            .floor() as u64;
        if quantity == 0 {
            return Err(format!("{id}: sized quantity is zero").into());
        }
        for cost in costs {
            let cost_bps = cost
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 0.0 && *n <= 1000.0)
                .ok_or("costBps entries must be finite, 0..1000")?;
            let assumptions = SimulationAssumptions {
                initial_capital_krw: initial_capital,
                exchange_rate_krw: exchange_rate,
                fee_bps: cost_bps,
                slippage_bps: cost_bps,
                tax_bps: 0.0,
                ..SimulationAssumptions::default()
            };
            let base = json!({
                "window":window,"symbol":symbol,"costBps":cost_bps,
                "orderQuantity":quantity,"referenceClose":reference_close,
                "sizing":{"targetEntryRatio":target_ratio,"reserveMultiplier":1.002,
                    "basis":"last historical close; fixed across costs"},
                "historyCount":history.len(),"evaluationCount":evaluation.len(),
                "historyStart":history[0].date,"historyEnd":history[251].date,
                "evaluationStart":evaluation[0].date,"evaluationEnd":evaluation[evaluation.len()-1].date,
                "assumptions":assumptions
            });
            for (strategy_id, params) in strategy_cases(symbol, quantity, reference_close)? {
                let result = preview_strategy_from_candles(StrategyPreviewInput {
                    strategy_id: strategy_id.into(),
                    strategy_name: strategy_id.into(),
                    symbol: symbol.into(),
                    is_overseas: true,
                    order_quantity: quantity,
                    params: params.clone(),
                    candles: evaluation.to_vec(),
                    history_candles: history.to_vec(),
                    warmup_count: None,
                    interval: Some("1d".into()),
                    data_source: Some(format!(
                        "Frozen Yahoo Finance {symbol} unadjusted daily OHLCV 2026-10-18"
                    )),
                    strategy_version: None,
                    broker_id: None,
                    broker_account_id: None,
                    assumptions: assumptions.clone(),
                });
                let mut row = base.clone();
                row["strategyId"] = json!(strategy_id);
                row["params"] = params;
                match result {
                    Ok(preview) => {
                        if strategy_id == "leveraged_trend_hold" {
                            return Err("generic LTH must reject historical replay".into());
                        }
                        row["kind"] = json!("strategy");
                        row["diagnostics"] = diagnostics(&preview.backtest);
                        row["preview"] = serde_json::to_value(preview)?;
                    }
                    Err(error)
                        if strategy_id == "leveraged_trend_hold"
                            && error.code == "UNSUPPORTED_GENERIC_REPLAY" =>
                    {
                        row["kind"] = json!("unsupported");
                        row["error"] = json!({"code":error.code,"message":error.message});
                    }
                    Err(error) => {
                        return Err(format!(
                            "{id}/{strategy_id}/{cost_bps}: {} {}",
                            error.code, error.message
                        )
                        .into())
                    }
                }
                write_row(&mut writer, &row)?;
                row_count += 1;
            }
            let report = benchmark(symbol, evaluation, quantity, assumptions)?;
            let mut row = base;
            row["kind"] = json!("benchmark");
            row["strategyId"] = json!("buy_hold_same_quantity");
            row["diagnostics"] = diagnostics(&report);
            row["backtest"] = serde_json::to_value(report)?;
            write_row(&mut writer, &row)?;
            row_count += 1;
        }
    }
    row_count += minute_diagnostics(&input, &mut writer)?;
    writer.flush()?;
    println!(
        "Wrote {row_count} research rows for {} windows",
        windows.len()
    );
    Ok(())
}
