//! Offline production-engine comparison. No credentials, provider or order APIs.
use kis_auto_trade_lib::{
    api::rest::ChartCandle,
    commands::{preview_strategy_from_candles, StrategyPreviewInput},
    trading::simulation::{replay_input_hash, SimulationAssumptions},
};
use serde_json::{json, Value};
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::args().nth(1).ok_or("repository path required")?;
    let frozen = format!("{root}/docs/research/strategy-review-20261018");
    let input: Value = serde_json::from_str(&fs::read_to_string(format!("{frozen}/input.json"))?)?;
    let datasets = input["datasets"].as_array().ok_or("datasets")?;
    let mut baseline = Vec::new();
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
        let run = || {
            let assumptions = assumptions.clone();
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
        let preview = run()?;
        let report = serde_json::to_value(&preview.backtest)?;
        baseline.push(
            json!({"key":format!("{}|{strategy_id}|{}",window["id"],row["costBps"]),
                "summary":preview.backtest.summary,
                "reportHash":replay_input_hash(&[&serde_json::to_string(&report)?]),
                "signalsHash":replay_input_hash(&[&serde_json::to_string(&preview.signals)?])
            }),
        );
        parity += 1;
    }
    fs::write(
        format!("{root}/docs/research/replay-accounting-20261018/baseline-hashes.json"),
        serde_json::to_string(&baseline)?,
    )?;
    println!("HEAD baseline {} complete production runs", parity);
    Ok(())
}
