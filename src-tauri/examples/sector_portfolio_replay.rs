//! Offline supplied-data calculation. Does not start the app or load credentials.
use kis_auto_trade_lib::trading::sector_portfolio::{
    preview_sector_portfolio, SectorPortfolioInput,
};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: sector_portfolio_replay INPUT.json > REPORT.json")?;
    if args.next().is_some() {
        return Err("expected one input file".into());
    }
    let mut bytes = vec![];
    File::open(path)?
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("input exceeds 2 MiB".into());
    }
    let input: SectorPortfolioInput = serde_json::from_slice(&bytes)?;
    let report = preview_sector_portfolio(&input)?;
    eprintln!("rawInputHash={}", hex::encode(Sha256::digest(&bytes)));
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
