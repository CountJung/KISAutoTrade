//! Offline production-engine comparison. No credentials, provider or order APIs.
use kis_auto_trade_lib::{
    api::rest::ChartCandle,
    commands::{preview_strategy_from_candles, StrategyPreviewInput},
    trading::simulation::{
        replay_input_hash, run_backtest, ExecutionModel, SimulationAssumptions, SimulationEvent,
    },
};
use serde_json::{json, Value};
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::args().nth(1).ok_or("repository path required")?;
    let frozen = format!("{root}/docs/research/strategy-review-20261018");
    let input: Value = serde_json::from_str(&fs::read_to_string(format!("{frozen}/input.json"))?)?;
    let datasets = input["datasets"].as_array().ok_or("datasets")?;
    let baseline: Vec<Value> = serde_json::from_str(&fs::read_to_string(format!(
        "{root}/docs/research/replay-execution-20261018/baseline-hashes.json"
    ))?)?;
    let mut historical_summary_differences = 0;
    let mut comparisons = Vec::new();
    let mut parity = 0;
    for line in fs::read_to_string(format!("{frozen}/results.jsonl"))?.lines() {
        let row: Value = serde_json::from_str(line)?;
        if row["kind"] != "strategy" {
            continue;
        }
        let symbol = row["symbol"].as_str().ok_or("symbol")?;
        let strategy_id = row["strategyId"].as_str().ok_or("strategy")?;
        let data = datasets
            .iter()
            .find(|data| data["symbol"] == symbol)
            .ok_or("dataset")?;
        let candles: Vec<ChartCandle> = serde_json::from_value(data["candles"].clone())?;
        let window = &row["window"];
        let h = window["historyStart"].as_u64().ok_or("history")? as usize;
        let start = window["evaluationStart"].as_u64().ok_or("start")? as usize;
        let end = window["evaluationEndExclusive"].as_u64().ok_or("end")? as usize;
        let quantity = row["orderQuantity"].as_u64().ok_or("quantity")?;
        let assumptions: SimulationAssumptions =
            serde_json::from_value(row["assumptions"].clone())?;
        let run = |model| {
            let mut assumptions = assumptions.clone();
            assumptions.execution_model = model;
            preview_strategy_from_candles(StrategyPreviewInput {
                strategy_id: strategy_id.into(),
                strategy_name: strategy_id.into(),
                symbol: symbol.into(),
                is_overseas: true,
                order_quantity: quantity,
                params: row["params"].clone(),
                candles: candles[start..end].to_vec(),
                history_candles: candles[h..start].to_vec(),
                warmup_count: None,
                interval: Some("1d".into()),
                data_source: Some("frozen strategy-review-20261018".into()),
                strategy_version: None,
                broker_id: None,
                broker_account_id: None,
                assumptions,
            })
            .map_err(|error| format!("{}: {}", error.code, error.message))
        };
        let same_close = run(ExecutionModel::SameClose)?;
        let summary = serde_json::to_value(&same_close.backtest.summary)?;
        if row["summary"]
            .as_object()
            .ok_or("summary")?
            .iter()
            .any(|(key, old)| &summary[key] != old)
        {
            historical_summary_differences += 1;
        }
        let key = format!("{}|{strategy_id}|{}", window["id"], row["costBps"]);
        let prior = baseline
            .iter()
            .find(|record| record["key"] == key)
            .ok_or("HEAD baseline missing")?;
        let mut report = serde_json::to_value(&same_close.backtest)?;
        report.as_object_mut().unwrap().remove("executionWarnings");
        for field in [
            "executionModel",
            "intrabarExitPolicy",
            "stopLossPct",
            "takeProfitPct",
        ] {
            report["assumptions"].as_object_mut().unwrap().remove(field);
        }
        for field in ["unfilledOrderCount", "ambiguousBarCount"] {
            report["summary"].as_object_mut().unwrap().remove(field);
        }
        for trade in report["trades"].as_array_mut().unwrap() {
            for field in ["signalAvailableAt", "fillAt", "intrabarPathStatus"] {
                trade.as_object_mut().unwrap().remove(field);
            }
        }
        let report_hash = replay_input_hash(&[&serde_json::to_string(&report)?]);
        let signal_hash = replay_input_hash(&[&serde_json::to_string(&same_close.signals)?]);
        if prior["reportHash"] != report_hash || prior["signalsHash"] != signal_hash {
            return Err(format!("HEAD sameClose accounting/signal regression {key}").into());
        }
        parity += 1;
        if window["id"] != "QQQ-2026Q3" || !matches!(strategy_id, "mean_reversion" | "strong_close")
        {
            continue;
        }
        for (model, preview) in [
            (ExecutionModel::SameClose, same_close),
            (ExecutionModel::NextOpen, run(ExecutionModel::NextOpen)?),
        ] {
            let mut benchmark_assumptions = assumptions.clone();
            benchmark_assumptions.execution_model = model;
            let events = candles[start..end]
                .iter()
                .enumerate()
                .map(|(index, c)| {
                    Ok(SimulationEvent {
                        time: c.date.clone(),
                        chart_time: c.date.clone(),
                        open_units: (c.open.parse::<f64>()? * 100.0).round() as u64,
                        close_units: (c.close.parse::<f64>()? * 100.0).round() as u64,
                        high_units: (c.high.parse::<f64>()? * 100.0).round() as u64,
                        low_units: (c.low.parse::<f64>()? * 100.0).round() as u64,
                        signal: (index == 0).then(|| {
                            kis_auto_trade_lib::trading::strategy::Signal::Buy {
                                symbol: symbol.into(),
                                quantity,
                                reason: "First evaluation close buy-and-hold instruction".into(),
                            }
                        }),
                    })
                })
                .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
            let benchmark = run_backtest(
                "execution_benchmark",
                symbol,
                true,
                benchmark_assumptions,
                &events,
            );
            comparisons.push(json!({"window":window,"strategyId":strategy_id,"symbol":symbol,
                "costBps":row["costBps"],"orderQuantity":quantity,"params":row["params"],
                "historyBars":start-h,"evaluationBars":end-start,"replay":preview.replay,
                "backtest":preview.backtest,"preparation":preview.preparation,
                "benchmark":{"policy":"first evaluation close instruction; same execution model; no forced liquidation", "report":benchmark}
            }));
        }
    }
    fs::write(
        format!("{root}/docs/research/replay-execution-20261018/results.json"),
        serde_json::to_string_pretty(&json!({
            "source":"../strategy-review-20261018/input.json","baselineCommit":"ba74e03","sameCloseFullReportAndSignalParityRuns":parity,
            "historicalSummaryDifferencesAlreadyInBaseline":historical_summary_differences,
            "comparisons":comparisons,
            "limitations":["Frozen daily bars; calendar/corporate-action coverage unverified", "Fixed quantity and FX, no dividend or forced liquidation", "OHLC path and actual market fills unverified; no research overlay enabled in these comparisons"]
        }))? + "\n",
    )?;
    println!(
        "sameClose full accounting report and signal parity {parity}; production comparisons {}",
        comparisons.len()
    );
    Ok(())
}
