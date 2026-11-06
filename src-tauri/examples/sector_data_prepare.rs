//! Bounded offline validator; does not initialize AppState, authenticate or call providers.
use kis_auto_trade_lib::trading::sector_data::{validate_sector_bundle, ValidateSectorDataInput};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: sector_data_prepare INPUT.json|- > REPORT.json")?;
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
    // Do not print parse error snippets from imported arbitrary text.
    let input: ValidateSectorDataInput =
        serde_json::from_slice(&bytes).map_err(|_| "invalid sector data JSON")?;
    let report = validate_sector_bundle(&input);
    eprintln!("rawInputHash={}", hex::encode(Sha256::digest(&bytes)));
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
