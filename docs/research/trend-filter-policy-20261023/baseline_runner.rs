//! Archived production replay only. No provider, account or order API access.
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
        "interval":"1d","dataSource":"frozen strategy-review-20261018; predeclared TrendFilter ownership and exit-policy 2023-2025 study",
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
    let clock = Instant::now();
    let root = std::env::args().nth(1).ok_or("repository path")?;
    let output = format!("{root}/docs/research/trend-filter-policy-20261023");
    let frozen = format!("{root}/docs/research/strategy-review-20261018");
    let source: Value = serde_json::from_str(&fs::read_to_string(format!("{frozen}/input.json"))?)?;
    let design: Value =
        serde_json::from_str(&fs::read_to_string(format!("{output}/design.json"))?)?;
    let datasets = source["datasets"].as_array().ok_or("datasets")?;
    let mut baseline = Vec::new();
    for line in fs::read_to_string(format!("{frozen}/results.jsonl"))?.lines() {
        let row: Value = serde_json::from_str(line)?;
        if row["kind"] != "strategy" {
            continue;
        }
        let candles = datasets
            .iter()
            .find(|d| d["symbol"] == row["symbol"])
            .ok_or("dataset")?["candles"]
            .as_array()
            .ok_or("candles")?;
        let window = &row["window"];
        let h = window["historyStart"].as_u64().ok_or("h")? as usize;
        let s = window["evaluationStart"].as_u64().ok_or("s")? as usize;
        let e = window["evaluationEndExclusive"].as_u64().ok_or("e")? as usize;
        let mut req = request(
            &row,
            &candles[s..e],
            &candles[h..s],
            row["costBps"].as_f64().ok_or("cost")? as u64,
        );
        req["assumptions"] = row["assumptions"].clone();
        req["dataSource"] = json!("frozen strategy-review-20261018");
        let (result, signals_hash) = preview(req, false)?;
        baseline.push(json!({"key":format!("{}|{}|{}",window["id"],row["strategyId"].as_str().ok_or("id")?,row["costBps"]),
   "strategyId":row["strategyId"],"symbol":row["symbol"],"costBps":row["costBps"],"window":window,
   "summary":result["backtest"]["summary"],"accounting":result["backtest"]["accounting"],
   "reportHash":replay_input_hash(&[&serde_json::to_string(&result["backtest"])?]),"signalsHash":signals_hash}));
    }
    assert_eq!(baseline.len(), 1224);
    fs::write(
        format!("{output}/baseline-hashes.json"),
        serde_json::to_string(&baseline)?,
    )?;
    let mut full = fs::File::create(format!("{output}/legacy-continuous.jsonl"))?;
    let mut reset = fs::File::create(format!("{output}/legacy-reset.jsonl"))?;
    let mut partitions = Vec::new();
    let mut reset_count = 0;
    for config in design["configs"].as_array().ok_or("configs")? {
        let candles = datasets
            .iter()
            .find(|d| d["symbol"] == config["symbol"])
            .ok_or("dataset")?["candles"]
            .as_array()
            .ok_or("candles")?;
        let h = config["historyStart"].as_u64().ok_or("h")? as usize;
        let s = config["evaluationStart"].as_u64().ok_or("s")? as usize;
        let e = config["evaluationEndExclusive"].as_u64().ok_or("e")? as usize;
        let mut groups = Vec::<Vec<Value>>::new();
        let mut last = String::new();
        for candle in &candles[s..e] {
            let q = quarter(candle["date"].as_str().ok_or("date")?);
            if q != last {
                groups.push(Vec::new());
                last = q;
            }
            groups.last_mut().unwrap().push(candle.clone());
        }
        let alternative = candles[s..e]
            .chunks(500)
            .map(|c| c.to_vec())
            .collect::<Vec<_>>();
        assert_eq!(groups.len(), 12);
        for cost in [10_u64, 20] {
            let req = request(config, &candles[s..e], &candles[h..s], cost);
            let (result, _) = preview(continuous_request(req.clone(), &groups), true)?;
            let (alt, _) = preview(continuous_request(req, &alternative), true)?;
            let economic = economic_hash(&result)?;
            assert_eq!(economic, economic_hash(&alt)?);
            assert_eq!(result["replay"]["inputHash"], alt["replay"]["inputHash"]);
            partitions.push(json!({"symbol":config["symbol"],"strategyId":config["strategyId"],"variant":"legacy","costBps":cost,"quarterChunks":12,"alternativeChunks":2,"economicHash":economic,"inputHash":result["replay"]["inputHash"]}));
            write_row(
                &mut full,
                &json!({"kind":"continuous","config":config,"variant":"legacy","costBps":cost,"result":result}),
            )?;
            for group in &groups {
                let first = group[0]["date"].as_str().ok_or("date")?;
                let start = candles
                    .iter()
                    .position(|c| c["date"] == first)
                    .ok_or("start")?;
                let (result, _) = preview(
                    request(config, group, &candles[start - 252..start], cost),
                    false,
                )?;
                write_row(
                    &mut reset,
                    &json!({"kind":"quarterReset","config":config,"variant":"legacy","costBps":cost,"quarter":quarter(first),"historyStart":start-252,"evaluationStart":start,"evaluationEndExclusive":start+group.len(),"result":result}),
                )?;
                reset_count += 1;
            }
        }
    }
    assert_eq!(partitions.len(), 6);
    assert_eq!(reset_count, 72);
    fs::write(
        format!("{output}/baseline-run-summary.json"),
        serde_json::to_string_pretty(
            &json!({"baselineCommit":"81fa00d3aa50bc59606667568f36c4b686902412","flatRuns":1224,"continuousRuns":6,"quarterResetRuns":reset_count,"partitionEquivalenceRuns":6,"partitions":partitions,"elapsedSeconds":clock.elapsed().as_secs_f64()}),
        )? + "\n",
    )?;
    println!("Archived81fa00d3aa50bc59606667568f36c4b686902412: flat1224,continuous6,partition6,reset72");
    Ok(())
}
