//! Production replay only. Offline frozen prices, no provider/account/order APIs.
use kis_auto_trade_lib::{
    commands::{preview_strategy_continuous_from_candles, preview_strategy_from_candles},
    trading::simulation::replay_input_hash,
};
use serde_json::{json, Value};
use std::{error::Error, fs, io::Write, time::Instant};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn preview(input: Value, continuous: bool) -> Result<(Value, String)> {
    let result = if continuous {
        serde_json::to_value(
            preview_strategy_continuous_from_candles(serde_json::from_value(input)?)
                .map_err(|e| format!("{}: {}", e.code, e.message))?,
        )?
    } else {
        serde_json::to_value(
            preview_strategy_from_candles(serde_json::from_value(input)?)
                .map_err(|e| format!("{}: {}", e.code, e.message))?,
        )?
    };
    let signals_hash = replay_input_hash(&[&serde_json::to_string(&result["signals"])?]);
    Ok((result, signals_hash))
}

fn economic_hash(result: &Value) -> Result<String> {
    Ok(replay_input_hash(&[&serde_json::to_string(&json!({
        "backtest":result["backtest"], "signals":result["signals"],
        "preparation":result["preparation"], "entryFilterPreparation":result["entryFilterPreparation"]
    }))?]))
}

fn request(config: &Value, candles: &[Value], history: &[Value], cost: u64) -> Value {
    json!({
        "strategyId":config["strategyId"],"strategyName":config["strategyId"],
        "symbol":config["symbol"],"isOverseas":true,"orderQuantity":config["orderQuantity"],
        "params":config["params"],"candles":candles,"historyCandles":history,
        "interval":"1d","dataSource":"frozen strategy-review-20261018; predeclared single entry-filter 2023-2025 study",
        "assumptions":{"initialCapitalKrw":10000000.0,"exchangeRateKrw":1450.0,
            "executionModel":"nextOpen","feeBps":cost,"slippageBps":cost,"taxBps":0.0}
    })
}

fn continuous_request(mut request: Value, groups: &[Vec<Value>]) -> Value {
    request.as_object_mut().unwrap().remove("candles");
    request["chunks"] = json!(groups
        .iter()
        .map(|rows| json!({"candles":rows}))
        .collect::<Vec<_>>());
    request
}

fn quarter(date: &str) -> String {
    let month: u32 = date[4..6].parse().expect("normalized frozen daily date");
    format!("{}Q{}", &date[..4], (month - 1) / 3 + 1)
}

fn write_row(file: &mut fs::File, row: &Value) -> Result<()> {
    serde_json::to_writer(&mut *file, row)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn main() -> Result<()> {
    let start_clock = Instant::now();
    let root = std::env::args().nth(1).ok_or("repository path required")?;
    let frozen = format!("{root}/docs/research/strategy-review-20261018");
    let output = format!("{root}/docs/research/consecutive-filters-20261018");
    let input: Value = serde_json::from_str(&fs::read_to_string(format!("{frozen}/input.json"))?)?;
    let design: Value =
        serde_json::from_str(&fs::read_to_string(format!("{output}/design.json"))?)?;
    let baseline: Vec<Value> = serde_json::from_str(&fs::read_to_string(format!(
        "{output}/baseline-hashes.json"
    ))?)?;
    let datasets = input["datasets"].as_array().ok_or("datasets")?;
    let mut parity = 0;
    for line in fs::read_to_string(format!("{frozen}/results.jsonl"))?.lines() {
        let row: Value = serde_json::from_str(line)?;
        if row["kind"] != "strategy" {
            continue;
        }
        let data = datasets
            .iter()
            .find(|d| d["symbol"] == row["symbol"])
            .ok_or("dataset")?;
        let candles = data["candles"].as_array().ok_or("candles")?;
        let window = &row["window"];
        let h = window["historyStart"].as_u64().ok_or("history")? as usize;
        let s = window["evaluationStart"].as_u64().ok_or("start")? as usize;
        let e = window["evaluationEndExclusive"].as_u64().ok_or("end")? as usize;
        let mut req = request(
            &row,
            &candles[s..e],
            &candles[h..s],
            row["costBps"].as_f64().ok_or("cost")? as u64,
        );
        req["dataSource"] = json!("frozen strategy-review-20261018");
        req["assumptions"] = row["assumptions"].clone();
        let (mut result, _) = preview(req, false)?;
        if row["strategyId"]
            .as_str()
            .is_some_and(|s| s.starts_with("consecutive_move"))
        {
            normalize_observation_labels(&mut result["backtest"]);
            normalize_observation_labels(&mut result["signals"]);
        }
        let signals_hash = replay_input_hash(&[&serde_json::to_string(&result["signals"])?]);
        let key = format!(
            "{}|{}|{}",
            window["id"],
            row["strategyId"].as_str().ok_or("strategy")?,
            row["costBps"]
        );
        let prior = baseline
            .iter()
            .find(|r| r["key"] == key)
            .ok_or("baseline missing")?;
        let report_hash = replay_input_hash(&[&serde_json::to_string(&result["backtest"])?]);
        if prior["reportHash"] != report_hash || prior["signalsHash"] != signals_hash {
            fs::write(
                format!("{root}/.cache/consecutive-parity-diagnostic.json"),
                serde_json::to_string_pretty(
                    &json!({"key":key,"prior":prior,"current":result,"reportHash":report_hash,"signalsHash":signals_hash}),
                )?,
            )?;
            return Err(format!("Flat compatibility regression: {key}").into());
        }
        parity += 1;
    }
    if parity != 1224 {
        return Err(format!("Unexpected flat parity coverage {parity}").into());
    }
    let mut continuous_file = fs::File::create(format!("{output}/continuous-results.jsonl"))?;
    let mut reset_file = fs::File::create(format!("{output}/reset-results.jsonl"))?;
    let mut continuous_count = 0;
    let mut reset_count = 0;
    let mut partitions = Vec::new();
    let mut long_parity_keys = std::collections::BTreeSet::new();
    for original_config in design["configs"].as_array().ok_or("configs")? {
        for variant in design["variants"].as_array().ok_or("variants")? {
            let mut merged_config = original_config.clone();
            for (key, value) in variant["params"].as_object().ok_or("variant params")? {
                merged_config["params"][key] = value.clone();
            }
            let config = &merged_config;
            let data = datasets
                .iter()
                .find(|d| d["symbol"] == config["symbol"])
                .ok_or("dataset")?;
            let candles = data["candles"].as_array().ok_or("candles")?;
            let h = config["historyStart"].as_u64().ok_or("history")? as usize;
            let s = config["evaluationStart"].as_u64().ok_or("start")? as usize;
            let e = config["evaluationEndExclusive"].as_u64().ok_or("end")? as usize;
            let evaluation = &candles[s..e];
            let history = &candles[h..s];
            assert_eq!(evaluation.len(), 752);
            assert_eq!(history.len(), 252);
            let mut groups = Vec::<Vec<Value>>::new();
            let mut last_quarter = String::new();
            for candle in evaluation {
                let q = quarter(candle["date"].as_str().ok_or("date")?);
                if q != last_quarter {
                    groups.push(Vec::new());
                    last_quarter = q;
                }
                groups.last_mut().unwrap().push(candle.clone());
            }
            assert_eq!(groups.len(), 12);
            let alternative = evaluation
                .chunks(500)
                .map(|part| part.to_vec())
                .collect::<Vec<_>>();
            assert_eq!(alternative.len(), 2);
            for cost in [10_u64, 20] {
                let req = request(config, evaluation, history, cost);
                let (result, _) = preview(continuous_request(req.clone(), &groups), true)?;
                let (alternative_result, _) = preview(continuous_request(req, &alternative), true)?;
                if variant["id"] == "baseline" {
                    let old: Vec<Value> = serde_json::from_str(&fs::read_to_string(format!(
                        "{output}/long-baseline-hashes.json"
                    ))?)?;
                    let prior = old
                        .iter()
                        .find(|r| r["symbol"] == config["symbol"] && r["costBps"] == cost)
                        .ok_or("long baseline")?;
                    let mut backtest = result["backtest"].clone();
                    let mut signals = result["signals"].clone();
                    normalize_observation_labels(&mut backtest);
                    normalize_observation_labels(&mut signals);
                    if prior["reportHash"]
                        != replay_input_hash(&[&serde_json::to_string(&backtest)?])
                        || prior["signalsHash"]
                            != replay_input_hash(&[&serde_json::to_string(&signals)?])
                    {
                        fs::write(
                            format!("{root}/.cache/consecutive-current-diagnostic.json"),
                            serde_json::to_string_pretty(&result)?,
                        )?;
                        return Err(format!(
                            "Long default regression: {} cost{cost}",
                            config["symbol"]
                        )
                        .into());
                    }
                }
                if variant["id"] == "baseline" {
                    long_parity_keys.insert(format!("{}|{cost}", config["symbol"]));
                }
                let economic = economic_hash(&result)?;
                if economic != economic_hash(&alternative_result)?
                    || result["replay"]["inputHash"] != alternative_result["replay"]["inputHash"]
                {
                    return Err(format!(
                        "Partition variance: {} {} {cost}",
                        config["symbol"], config["strategyId"]
                    )
                    .into());
                }
                partitions.push(
                    json!({"symbol":config["symbol"],"strategyId":config["strategyId"],
                "variant":variant["id"],"costBps":cost,"quarterChunks":12,"alternativeChunks":2,
                "economicHash":economic,"inputHash":result["replay"]["inputHash"]}),
                );
                write_row(
                    &mut continuous_file,
                    &json!({"kind":"continuous","config":config,"variant":variant["id"],"costBps":cost,"result":result}),
                )?;
                continuous_count += 1;
                for group in &groups {
                    let first = group[0]["date"].as_str().ok_or("date")?;
                    let reset_start = candles
                        .iter()
                        .position(|c| c["date"] == first)
                        .ok_or("reset start")?;
                    assert!(reset_start >= 252);
                    let (result, _) = preview(
                        request(
                            config,
                            group,
                            &candles[reset_start - 252..reset_start],
                            cost,
                        ),
                        false,
                    )?;
                    write_row(
                        &mut reset_file,
                        &json!({"kind":"quarterReset","config":config,"variant":variant["id"],"costBps":cost,
                    "quarter":quarter(first),"historyStart":reset_start-252,"evaluationStart":reset_start,
                    "evaluationEndExclusive":reset_start+group.len(),"result":result}),
                    )?;
                    reset_count += 1;
                }
            }
        }
    }
    if continuous_count != 24
        || reset_count != 288
        || partitions.len() != 24
        || long_parity_keys.len() != 6
    {
        return Err(format!(
            "Unexpected counts {continuous_count}/{reset_count}/{}",
            partitions.len()
        )
        .into());
    }
    fs::write(
        format!("{output}/run-summary.json"),
        serde_json::to_string_pretty(&json!({
            "baselineCommit":"6f9ca35","flatFullReportAndSignalParityRuns":parity,
            "longDefaultFullReportAndSignalParityRuns":long_parity_keys.len(),"longDefaultParityKeys":long_parity_keys,"continuousRuns":continuous_count,"independentQuarterResetRuns":reset_count,
            "partitionEquivalenceRuns":partitions.len(),"partitions":partitions,
            "note":"Quarter-reset returns are separate accounts and are never aggregated into continuous performance",
            "elapsedSeconds":start_clock.elapsed().as_secs_f64()
        }))? + "\n",
    )?;
    println!("flat parity {parity}; continuous {continuous_count}; partition equivalents {}; quarter resets {reset_count}",partitions.len());
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
