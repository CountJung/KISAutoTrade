//! Retrospective procedure only: fixed strategies, isolated accounts, production replay.
use kis_auto_trade_lib::commands::{
    preview_strategy_continuous_from_candles, ContinuousStrategyPreviewInput,
};
use kis_auto_trade_lib::trading::simulation::replay_input_hash;
use serde_json::{json, Value};
use std::{
    error::Error,
    fs,
    io::{BufWriter, Write},
    time::Instant,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn index(v: &Value, key: &str) -> Result<usize> {
    v[key]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("missing index {key}").into())
}
fn run(mut request: Value, groups: &[Vec<Value>]) -> Result<Value> {
    request["chunks"] = json!(groups
        .iter()
        .map(|rows| json!({"candles":rows}))
        .collect::<Vec<_>>());
    let input: ContinuousStrategyPreviewInput = serde_json::from_value(request)?;
    let result = preview_strategy_continuous_from_candles(input)
        .map_err(|e| format!("{}: {}", e.code, e.message))?;
    Ok(serde_json::to_value(result)?)
}
fn rows<'a>(source: &'a Value, symbol: &Value) -> Result<&'a [Value]> {
    source["datasets"]
        .as_array()
        .ok_or("datasets")?
        .iter()
        .find(|r| r["symbol"] == *symbol)
        .and_then(|r| r["candles"].as_array())
        .map(Vec::as_slice)
        .ok_or_else(|| "dataset missing".into())
}
fn request(
    strategy: &Value,
    symbol: &Value,
    quantity: &Value,
    params: Value,
    history: &[Value],
    assumptions: Value,
    data_source: &str,
) -> Value {
    json!({"strategyId":strategy,"strategyName":strategy,"symbol":symbol,"isOverseas":true,
    "orderQuantity":quantity,"params":params,"historyCandles":history,"interval":"1d",
    "dataSource":data_source,"assumptions":assumptions})
}
fn pc_params(symbol: &Value, quantity: &Value, reference: &Value) -> Result<Value> {
    let price: f64 = reference.as_str().ok_or("PC price")?.parse()?;
    Ok(
        json!({"symbols":[{"symbol":symbol,"quantity":quantity,"buy_trigger_price":price,
    "take_profit_pct":5.,"stop_loss_pct":3.,"is_overseas":true}]}),
    )
}
fn economic(value: &Value) -> Value {
    let mut report = value["backtest"].clone();
    report
        .as_object_mut()
        .expect("report")
        .remove("overfitWarning");
    report
        .as_object_mut()
        .expect("report")
        .remove("executionWarnings");
    for phase in report["phases"].as_array_mut().expect("phases") {
        phase.as_object_mut().expect("phase").remove("diagnostics");
    }
    json!({"backtest":report,"signals":value["signals"],"preparation":value["preparation"]})
}
fn write(file: &mut BufWriter<fs::File>, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}
fn main() -> Result<()> {
    let started = Instant::now();
    let root = std::env::args().nth(1).ok_or("repository root required")?;
    let output = format!("{root}/docs/research/phase-walk-forward-20261029");
    let design_text = fs::read_to_string(format!("{output}/design.json"))?;
    let design: Value = serde_json::from_str(&design_text)?;
    let source: Value = serde_json::from_str(&fs::read_to_string(format!(
        "{root}/docs/research/strategy-review-20261018/input.json"
    ))?)?;
    let mut file = BufWriter::new(fs::File::create(format!(
        "{output}/walk-forward-results.jsonl"
    ))?);
    let mut count = 0;
    for fold in design["folds"].as_array().ok_or("folds")? {
        for symbol in fold["symbols"].as_array().ok_or("symbols")? {
            let candles = rows(&source, &symbol["symbol"])?;
            for strategy in design["strategies"].as_array().ok_or("strategies")? {
                let params = if strategy["strategyId"] == "price_condition" {
                    pc_params(
                        &symbol["symbol"],
                        &symbol["orderQuantity"],
                        &symbol["quantityReferenceClose"],
                    )?
                } else {
                    strategy["params"].clone()
                };
                for cost in design["costBps"].as_array().ok_or("costs")? {
                    for (kind, start, end) in [
                        (
                            "context",
                            index(symbol, "contextStart")?,
                            index(symbol, "contextEndExclusive")?,
                        ),
                        (
                            "evaluation",
                            index(symbol, "evaluationStart")?,
                            index(symbol, "evaluationEndExclusive")?,
                        ),
                    ] {
                        if start < 252 || start >= end {
                            return Err("invalid window".into());
                        }
                        let history = &candles[start - 252..start];
                        let evaluation = &candles[start..end];
                        let groups = evaluation
                            .chunks(500)
                            .map(<[_]>::to_vec)
                            .collect::<Vec<_>>();
                        let mut assumptions = design["assumptions"].clone();
                        assumptions["feeBps"] = cost.clone();
                        assumptions["slippageBps"] = cost.clone();
                        let result = run(request(&strategy["strategyId"],&symbol["symbol"],&symbol["orderQuantity"],params.clone(),history,assumptions,
                            "frozen strategy-review-20261018; retrospective fixed-candidate walk-forward procedure"),&groups)?;
                        if result["replay"]["engineVersion"] != design["plannedEngineVersion"] {
                            return Err("engine version drift".into());
                        }
                        write(
                            &mut file,
                            &json!({"kind":kind,"fold":fold["id"],"symbol":symbol["symbol"],"strategyId":strategy["strategyId"],
                        "costBps":cost,"orderQuantity":symbol["orderQuantity"],"params":params,"quantityReferenceDate":symbol["quantityReferenceDate"],
                        "quantityReferenceClose":symbol["quantityReferenceClose"],"historyStart":start-252,"evaluationStart":start,"evaluationEndExclusive":end,"result":result}),
                        )?;
                        count += 1;
                    }
                }
            }
        }
        println!("Completed {}: {count} isolated replay accounts", fold["id"]);
    }
    file.flush()?;
    if count
        != design["expected"]["totalWalkForwardRuns"]
            .as_u64()
            .ok_or("count")? as usize
    {
        return Err("walk-forward count".into());
    }
    // Separate compatibility work uses exactly the prior PC economic inputs.
    let pc_design: Value = serde_json::from_str(&fs::read_to_string(format!(
        "{root}/docs/research/price-condition-contracts-20261029/design.json"
    ))?)?;
    let mut pc_file = BufWriter::new(fs::File::create(format!(
        "{output}/pc-compatibility-results.jsonl"
    ))?);
    let mut pc_count = 0;
    let mut partitions = Vec::new();
    for config in pc_design["configs"].as_array().ok_or("PC configs")? {
        let candles = rows(&source, &config["symbol"])?;
        let start = index(config, "evaluationStart")?;
        let end = index(config, "evaluationEndExclusive")?;
        let mut groups: Vec<Vec<Value>> = Vec::new();
        let mut previous = String::new();
        for candle in &candles[start..end] {
            let date = candle["date"].as_str().ok_or("date")?;
            let quarter = format!(
                "{}Q{}",
                &date[..4],
                (date[4..6].parse::<u32>()? - 1) / 3 + 1
            );
            if quarter != previous {
                groups.push(Vec::new());
                previous = quarter;
            }
            groups.last_mut().ok_or("group")?.push(candle.clone());
        }
        for cost in [10, 20] {
            let assumptions = json!({"initialCapitalKrw":10000000.,"exchangeRateKrw":1450.,"executionModel":"nextOpen","feeBps":cost,"slippageBps":cost,"taxBps":0.});
            let req = request(
                &config["strategyId"],
                &config["symbol"],
                &config["orderQuantity"],
                config["params"].clone(),
                &candles[start - 252..start],
                assumptions.clone(),
                "frozen strategy-review-20261018; predeclared 2023-2025 continuity study",
            );
            let result = run(req.clone(), &groups)?;
            let alternative = run(
                req,
                &candles[start..end]
                    .chunks(500)
                    .map(<[_]>::to_vec)
                    .collect::<Vec<_>>(),
            )?;
            if result["backtest"] != alternative["backtest"]
                || result["signals"] != alternative["signals"]
                || result["preparation"] != alternative["preparation"]
                || result["replay"]["inputHash"] != alternative["replay"]["inputHash"]
            {
                return Err("PC partition variance".into());
            }
            partitions.push(json!({"symbol":config["symbol"],"costBps":cost,"inputHash":result["replay"]["inputHash"],"economicHash":replay_input_hash(&[&serde_json::to_string(&economic(&result))?])}));
            write(
                &mut pc_file,
                &json!({"kind":"continuous","config":config,"costBps":cost,"evaluationStart":start,"evaluationEndExclusive":end,"result":result}),
            )?;
            pc_count += 1;
            for group in &groups {
                let date = group[0]["date"].as_str().ok_or("date")?;
                let s = candles
                    .iter()
                    .position(|c| c["date"] == date)
                    .ok_or("reset index")?;
                let result = run(
                    request(
                        &config["strategyId"],
                        &config["symbol"],
                        &config["orderQuantity"],
                        config["params"].clone(),
                        &candles[s - 252..s],
                        assumptions.clone(),
                        "frozen strategy-review-20261018; predeclared 2023-2025 continuity study",
                    ),
                    std::slice::from_ref(group),
                )?;
                write(
                    &mut pc_file,
                    &json!({"kind":"quarterReset","config":config,"costBps":cost,
                "quarter":format!("{}Q{}",&date[..4],(date[4..6].parse::<u32>()?-1)/3+1),"evaluationStart":s,"evaluationEndExclusive":s+group.len(),"result":result}),
                )?;
                pc_count += 1;
            }
        }
    }
    pc_file.flush()?;
    if pc_count != 78 || partitions.len() != 6 {
        return Err("PC counts".into());
    }
    fs::write(
        format!("{output}/run-summary.json"),
        serde_json::to_string_pretty(
            &json!({"design":design,"designReplayHash":replay_input_hash(&[&design_text]),
    "contextRuns":count/2,"evaluationRuns":count/2,"totalWalkForwardRuns":count,"pcCompatibilityRuns":pc_count,"partitionCompatibilityRuns":partitions.len(),"partitions":partitions,
    "elapsedSeconds":started.elapsed().as_secs_f64(),"comparisonTarget":"Round2 archived PC economic reports; no fresh previous-commit build"}),
        )? + "\n",
    )?;
    println!(
        "Finished {count} walk-forward, {pc_count} compatibility, {} partition checks",
        partitions.len()
    );
    Ok(())
}
