//! Explicit local quote probe. Never starts AppState, logs secrets, or saves profiles/tokens.
use kis_auto_trade_lib::{
    api::{rest::KisRestClient, token::TokenManager},
    broker::BrokerId,
    config::{AppConfig, DiscordConfig, ProfilesConfig},
    trading::{
        broker_chart::*, sector_data::SectorExchangeMapping, sector_portfolio::BrokerChartBroker,
    },
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};
use tokio::sync::RwLock;

fn bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "BROKER_CHART_PROFILE_OR_INPUT_UNAVAILABLE")?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "BROKER_CHART_PROFILE_OR_INPUT_READ_FAILED")?;
    if bytes.len() as u64 > limit {
        return Err("BROKER_CHART_LOCAL_INPUT_TOO_LARGE".into());
    }
    Ok(bytes)
}

struct ProbeSource {
    client: BrokerChartClient,
    profile_path: PathBuf,
    profile_digest: [u8; 32],
    modified: Option<SystemTime>,
}
impl ProbeSource {
    fn unchanged(&self) -> Result<(), String> {
        let bytes = bounded(&self.profile_path, 2 * 1024 * 1024)?;
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        let modified = std::fs::metadata(&self.profile_path)
            .ok()
            .and_then(|m| m.modified().ok());
        if digest != self.profile_digest || modified != self.modified {
            return Err("BROKER_CHART_PROFILE_CHANGED".into());
        }
        Ok(())
    }
}
impl BrokerChartQuoteSource for ProbeSource {
    async fn fetch(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        end: &str,
        cursor: Option<&str>,
    ) -> Result<RawBrokerChartPage, String> {
        self.unchanged()?;
        let page = self.client.fetch(symbol, exchange, end, cursor).await;
        self.unchanged()?;
        page
    }
}

async fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut ack = false;
    let mut check_only = false;
    let mut mapping_path = None;
    let mut output = PathBuf::from(".cache/broker-chart-live");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--acknowledge-quotes-only" => ack = true,
            "--check-existing-profile-only" => check_only = true,
            "--mapping-file" => {
                mapping_path = Some(PathBuf::from(
                    args.next().ok_or("BROKER_CHART_MISSING_MAPPING_PATH")?,
                ))
            }
            "--output-dir" => {
                output = PathBuf::from(args.next().ok_or("BROKER_CHART_MISSING_OUTPUT_PATH")?)
            }
            _ => return Err("BROKER_CHART_UNKNOWN_ARGUMENT".into()),
        }
    }
    if !ack {
        return Err("BROKER_CHART_EXPLICIT_QUOTE_ACK_REQUIRED".into());
    }
    // The normal app's single supported macOS location. No directory search,
    // secure_config, Discord, keychain, migration, settings write or .env loading.
    let profile_path =
        PathBuf::from(std::env::var_os("HOME").ok_or("BROKER_CHART_HOME_UNAVAILABLE")?)
            .join("Library/Application Support/com.countjung.kisautotrade/profiles.json");
    let bytes = bounded(&profile_path, 2 * 1024 * 1024)?;
    let profiles: ProfilesConfig =
        serde_json::from_slice(&bytes).map_err(|_| "BROKER_CHART_PROFILE_FORMAT_INVALID")?;
    let profile = profiles
        .get_active()
        .cloned()
        .ok_or("BROKER_CHART_NO_ACTIVE_PROFILE")?;
    if !profile.is_configured() {
        return Err("BROKER_CHART_ACTIVE_PROFILE_UNCONFIGURED".into());
    }
    // A new standalone Toss token would invalidate an operational process's token.
    // Toss collection is supported in the app's existing shared-token session only.
    if profile.broker_id == BrokerId::Toss {
        return Err("BROKER_CHART_TOSS_PROBE_REQUIRES_EXISTING_APP_SESSION".into());
    }
    if check_only {
        println!(
            "BROKER_CHART_EXISTING_KIS_PROFILE_READY; no authentication or marketdata request made"
        );
        return Ok(());
    }
    let mappings: Vec<SectorExchangeMapping> = if let Some(path) = mapping_path {
        serde_json::from_slice(&bounded(&path, 65536)?)
            .map_err(|_| "BROKER_CHART_INVALID_MAPPING_FILE")?
    } else {
        Vec::new()
    };
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "BROKER_CHART_OUTPUT_CREATE_FAILED")?;
    }
    std::fs::create_dir(&output).map_err(|_| "BROKER_CHART_OUTPUT_MUST_BE_NEW_DIRECTORY")?;
    let config = AppConfig::from_profile(&profile, &DiscordConfig::default());
    let token = Arc::new(RwLock::new(TokenManager::new(config.clone())));
    let client = KisRestClient::new(
        config.kis_base_url().into(),
        profile.app_key,
        profile.app_secret,
        profile.account_no,
        profile.is_paper_trading,
        token,
    );
    let source = ProbeSource {
        client: BrokerChartClient::Kis(Arc::new(client)),
        profile_digest: Sha256::digest(bytes).into(),
        modified: std::fs::metadata(&profile_path)
            .ok()
            .and_then(|m| m.modified().ok()),
        profile_path,
    };
    let now = chrono::Utc::now();
    let end = now
        .with_timezone(&chrono_tz::America::New_York)
        .date_naive()
        .pred_opt()
        .ok_or("BROKER_CHART_INVALID_CURRENT_DATE")?
        .format("%Y-%m-%d")
        .to_string();
    let collection = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        collect_broker_chart(
            &source,
            BrokerChartBroker::Kis,
            20,
            &end,
            &now.to_rfc3339(),
            &mappings,
        ),
    )
    .await
    .map_err(|_| "BROKER_CHART_COLLECTION_TIMEOUT")??;
    source.unchanged()?;
    // Market fields only. No config, profile ID/name, account or credentials enter output.
    let encoded = serde_json::to_vec_pretty(&collection)
        .map_err(|_| "BROKER_CHART_OUTPUT_SERIALIZE_FAILED")?;
    File::options()
        .write(true)
        .create_new(true)
        .open(output.join("collection.json"))
        .and_then(|mut f| f.write_all(&encoded))
        .map_err(|_| "BROKER_CHART_OUTPUT_WRITE_FAILED")?;
    let input = serde_json::to_vec_pretty(&collection.input)
        .map_err(|_| "BROKER_CHART_OUTPUT_SERIALIZE_FAILED")?;
    File::options()
        .write(true)
        .create_new(true)
        .open(output.join("input.json"))
        .and_then(|mut f| f.write_all(&input))
        .map_err(|_| "BROKER_CHART_OUTPUT_WRITE_FAILED")?;
    println!("BROKER_CHART_COLLECTION_READY: 12 symbols, history22, evaluation20; price-only acknowledgement remains false");
    Ok(())
}

#[tokio::main]
async fn main() {
    // Intentionally installs no logging subscriber and never prints source errors.
    if let Err(code) = run().await {
        eprintln!("{code}");
        std::process::exit(1);
    }
}
