use crate::trading::sector_portfolio::{SectorCorporateAction, SectorPortfolioInput};
use serde::{Deserialize, Serialize};

/// Sanitized provider fields only, without response body, credentials or account scope.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorQuoteRow {
    pub date: String,
    pub open: String,
    pub close: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorExchangeMapping {
    pub symbol: String,
    /// Caller-confirmed KIS quotation EXCD; listing venue does not establish this value.
    pub exchange: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorDataManifest {
    /// Exactly 22 history sessions followed by one or more evaluation sessions.
    pub session_dates: Vec<String>,
    pub end_date: String,
    pub calendar_source: String,
    pub calendar_evidence: String,
    pub exchange_mappings: Vec<SectorExchangeMapping>,
    pub synthetic: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorRawPrice {
    pub date: String,
    pub open: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorAdjustedPrice {
    pub date: String,
    pub close: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorDataSeries {
    pub symbol: String,
    pub exchange: String,
    /// MODP=0 open/close; no OHLC adjustment is inferred from MODP=1.
    pub raw: Vec<SectorRawPrice>,
    /// MODP=1 close; dividend-total-return meaning requires separate manual evidence.
    pub adjusted: Vec<SectorAdjustedPrice>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorPageReceipt {
    pub symbol: String,
    pub exchange: String,
    pub modp: String,
    pub anchor: String,
    pub row_count: usize,
    /// SHA256 of canonical retained quote fields, not the HTTP response bytes.
    pub rows_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorDataBundle {
    pub schema_version: u32,
    pub manifest: SectorDataManifest,
    pub retrieved_at: String,
    pub series: Vec<SectorDataSeries>,
    #[serde(default)]
    pub pages: Vec<SectorPageReceipt>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SectorDataVerification {
    pub coverage_verified: bool,
    pub adjustment_verified: bool,
    pub corporate_actions_verified: bool,
    pub adjustment_evidence: String,
    /// Explicit evidence of no evaluation-window split/dividend/receivable events.
    /// History adjustment semantics must also be checked in adjustmentEvidence.
    pub corporate_actions_evidence: String,
    #[serde(default)]
    pub corporate_actions: Vec<SectorCorporateAction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValidateSectorDataInput {
    pub bundle: SectorDataBundle,
    pub verification: SectorDataVerification,
    pub initial_cash_usd: f64,
    pub cost_bps: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectSectorDataInput {
    pub expected_profile_id: String,
    pub authenticated_quotes_acknowledged: bool,
    pub manifest: SectorDataManifest,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorQuoteScope {
    pub profile_id: String,
    pub provider: String,
    pub paper: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorDataDiagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorDataValidationReport {
    pub ready: bool,
    pub diagnostics: Vec<SectorDataDiagnostic>,
    pub warnings: Vec<String>,
    pub input: Option<SectorPortfolioInput>,
    /// Canonical bundle JSON SHA256 (includes manifest and page receipts).
    pub bundle_hash: String,
    /// Canonical verification JSON SHA256; claims are caller declarations.
    pub verification_hash: String,
}
