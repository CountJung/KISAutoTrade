//! Offline production-engine comparison. No credentials, provider or order APIs.
use kis_auto_trade_lib::{
    api::rest::ChartCandle,
    commands::{
        preview_strategy_continuous_from_candles, preview_strategy_from_candles,
        StrategyPreviewInput,
    },
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
        let mut report = serde_json::to_value(&preview.backtest)?;
        let mut signals = serde_json::to_value(&preview.signals)?;
        let (report_labels, signal_labels) = if strategy_id.starts_with("consecutive_move") {
            (
                normalize_observation_labels(&mut report),
                normalize_observation_labels(&mut signals),
            )
        } else {
            (0, 0)
        };
        baseline.push(
            json!({"key":format!("{}|{strategy_id}|{}",window["id"],row["costBps"]),
                "summary":preview.backtest.summary,"normalizedReportReasons":report_labels,"normalizedSignalReasons":signal_labels,
                "reportHash":replay_input_hash(&[&serde_json::to_string(&report)?]),
                "signalsHash":replay_input_hash(&[&serde_json::to_string(&signals)?])
            }),
        );
        parity += 1;
    }
    fs::write(
        format!("{root}/docs/research/consecutive-filters-20261018/baseline-hashes.json"),
        serde_json::to_string(&baseline)?,
    )?;
    let design: Value = serde_json::from_str(&fs::read_to_string(format!(
        "{root}/docs/research/consecutive-filters-20261018/design.json"
    ))?)?;
    let mut long_baseline = Vec::new();
    for config in design["configs"].as_array().ok_or("configs")? {
        let data = datasets
            .iter()
            .find(|d| d["symbol"] == config["symbol"])
            .ok_or("dataset")?;
        let candles = data["candles"].as_array().ok_or("candles")?;
        let h = config["historyStart"].as_u64().ok_or("history")? as usize;
        let start = config["evaluationStart"].as_u64().ok_or("start")? as usize;
        let end = config["evaluationEndExclusive"].as_u64().ok_or("end")? as usize;
        for cost in [10_u64, 20] {
            let request = json!({"strategyId":config["strategyId"],"strategyName":config["strategyId"],
                "symbol":config["symbol"],"isOverseas":true,"orderQuantity":config["orderQuantity"],"params":config["params"],
                "historyCandles":&candles[h..start],"chunks":candles[start..end].chunks(500).map(|c|json!({"candles":c})).collect::<Vec<_>>(),
                "interval":"1d","assumptions":{"initialCapitalKrw":10000000.0,"exchangeRateKrw":1450.0,
                "executionModel":"nextOpen","feeBps":cost,"slippageBps":cost,"taxBps":0.0}});
            let result = preview_strategy_continuous_from_candles(serde_json::from_value(request)?)
                .map_err(|e| format!("{}: {}", e.code, e.message))?;
            fs::write(
                format!(
                    "{root}/.cache/consecutive-old-{}-{cost}.json",
                    config["symbol"].as_str().ok_or("symbol")?
                ),
                serde_json::to_string(&result)?,
            )?;
            let mut report = serde_json::to_value(&result.preview.backtest)?;
            let mut signals = serde_json::to_value(&result.preview.signals)?;
            let report_labels = normalize_observation_labels(&mut report);
            let signal_labels = normalize_observation_labels(&mut signals);
            long_baseline.push(json!({"symbol":config["symbol"],"costBps":cost,"summary":result.preview.backtest.summary,"normalizedReportReasons":report_labels,"normalizedSignalReasons":signal_labels,
                "reportHash":replay_input_hash(&[&serde_json::to_string(&report)?]),
                "signalsHash":replay_input_hash(&[&serde_json::to_string(&signals)?])}));
        }
    }
    fs::write(
        format!("{root}/docs/research/consecutive-filters-20261018/long-baseline-hashes.json"),
        serde_json::to_string_pretty(&long_baseline)?,
    )?;
    println!("HEAD baseline {} complete production runs", parity);
    Ok(())
}

// Canonicalize only reason fields matching either complete numeric legacy template.
fn normalize_observation_labels(value: &mut Value) -> usize {
    match value {
        Value::Array(values) => values.iter_mut().map(normalize_observation_labels).sum(),
        Value::Object(values) => values
            .iter_mut()
            .map(|(key, value)| {
                if key == "reason" {
                    if let Value::String(text) = value {
                        let terminal_suffix = " · 평가 종료: 다음 관측 봉이 없어 미체결";
                        let (base, terminal) = text
                            .strip_suffix(terminal_suffix)
                            .map_or((text.as_str(), ""), |base| (base, terminal_suffix));
                        for (old, new) in [
                            ("일 연속 상승 → 매수", "회 연속 가격 상승 → 매수"),
                            ("일 연속 하락 → 매도", "회 연속 가격 하락 → 매도"),
                        ] {
                            if let Some(number) = base.strip_suffix(old) {
                                if !number.is_empty() && number.bytes().all(|c| c.is_ascii_digit())
                                {
                                    *text = format!("{number}{new}{terminal}");
                                    return 1;
                                }
                            }
                        }
                    }
                    0
                } else {
                    normalize_observation_labels(value)
                }
            })
            .sum(),
        _ => 0,
    }
}
