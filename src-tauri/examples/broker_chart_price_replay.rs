//! Bounded offline raw-price replay. Does not initialize the app or providers.
use kis_auto_trade_lib::trading::sector_portfolio::{
    preview_broker_chart_price_approximation, BrokerChartInput,
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: broker_chart_price_replay INPUT.json|- > REPORT.json")?;
    if args.next().is_some() {
        return Err("expected one input file or -".into());
    }
    let source: Box<dyn Read> = if path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(path)?)
    };
    let mut bytes = Vec::new();
    source.take(2 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("input exceeds 2 MiB".into());
    }
    let input: BrokerChartInput =
        serde_json::from_slice(&bytes).map_err(|_| "invalid broker chart JSON")?;
    let report = preview_broker_chart_price_approximation(&input)?;
    eprintln!("rawInputHash={}", hex::encode(Sha256::digest(&bytes)));
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
