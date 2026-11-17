use crate::trading::sector_portfolio::{SectorCorporateAction, SectorPortfolioReport};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerChartBroker {
    Kis,
    Toss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerChartAdjustment {
    Unadjusted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum BrokerChartCurrency {
    #[serde(rename = "USD")]
    Usd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum BrokerChartInterval {
    #[serde(rename = "1d")]
    Daily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokerChartSessionBasis {
    ObservedSpySessions,
    BrokerCalendar,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrokerChartSource {
    pub broker_id: BrokerChartBroker,
    pub name: String,
    pub retrieved_at: String,
    pub synthetic: bool,
    pub adjustment: BrokerChartAdjustment,
    pub currency: BrokerChartCurrency,
    pub interval: BrokerChartInterval,
    pub session_basis: BrokerChartSessionBasis,
    /// Input declaration only. Observed SPY rows do not establish an official calendar.
    pub calendar_evidence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrokerChartPrice {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub currency: BrokerChartCurrency,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrokerChartDay {
    pub date: String,
    #[serde(deserialize_with = "unique_prices")]
    pub prices: BTreeMap<String, BrokerChartPrice>,
}

/// JSON object duplicates are rejected before BTreeMap could silently overwrite
/// conflicting or identical ticker values. Repeated session rows are checked later.
fn unique_prices<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, BrokerChartPrice>, D::Error> {
    struct Prices;
    impl<'de> serde::de::Visitor<'de> for Prices {
        type Value = BTreeMap<String, BrokerChartPrice>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("unique ticker price entries")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut rows = BTreeMap::new();
            while let Some((symbol, price)) = map.next_entry::<String, BrokerChartPrice>()? {
                if rows.insert(symbol, price).is_some() {
                    return Err(serde::de::Error::custom("DUPLICATE_BROKER_CHART_TICKER"));
                }
                if rows.len() > 12 {
                    return Err(serde::de::Error::custom("OVERSIZED_BROKER_CHART_UNIVERSE"));
                }
            }
            Ok(rows)
        }
    }
    deserializer.deserialize_map(Prices)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrokerChartInput {
    pub schema_version: u32,
    pub initial_cash_usd: f64,
    pub cost_bps: f64,
    pub source: BrokerChartSource,
    pub history: Vec<BrokerChartDay>,
    pub evaluation: Vec<BrokerChartDay>,
    pub expected_session_dates: Vec<String>,
    pub acknowledged_price_only: bool,
    #[serde(default)]
    pub known_corporate_actions: Vec<SectorCorporateAction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerChartReport {
    pub mode: String,
    pub signal_basis: String,
    pub dividend_return_included: bool,
    pub economic_return_verified: bool,
    /// Canonical full BrokerChartInput DTO SHA256, not original file bytes.
    pub input_hash: String,
    pub warnings: Vec<String>,
    pub result: SectorPortfolioReport,
}
