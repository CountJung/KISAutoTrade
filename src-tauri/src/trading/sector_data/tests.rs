use super::*;
use crate::trading::sector_portfolio::{SectorCorporateAction, ALL_SYMBOLS};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::sync::Mutex;

pub(crate) fn fixture_manifest() -> SectorDataManifest {
    let mut dates = Vec::new();
    let mut date = NaiveDate::from_ymd_opt(2025, 9, 1).unwrap();
    while dates.len() < 25 {
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
            dates.push(date.to_string());
        }
        date += Duration::days(1);
    }
    SectorDataManifest {
        end_date: dates.last().unwrap().clone(),
        session_dates: dates,
        calendar_source: "synthetic weekdays, not actual exchange sessions".into(),
        calendar_evidence: "synthetic test declaration only".into(),
        synthetic: true,
        exchange_mappings: ALL_SYMBOLS
            .iter()
            .map(|symbol| SectorExchangeMapping {
                symbol: (*symbol).into(),
                exchange: "NYS".into(),
                evidence: "synthetic EXCD fixture, no real venue assertion".into(),
            })
            .collect(),
    }
}

pub(crate) fn fixture_input() -> ValidateSectorDataInput {
    let manifest = fixture_manifest();
    let series = ALL_SYMBOLS
        .iter()
        .map(|symbol| SectorDataSeries {
            symbol: (*symbol).into(),
            exchange: "NYS".into(),
            raw: manifest
                .session_dates
                .iter()
                .enumerate()
                .map(|(i, date)| SectorRawPrice {
                    date: date.clone(),
                    open: 100.0 + i as f64,
                    close: 102.0 + i as f64,
                })
                .collect(),
            adjusted: manifest
                .session_dates
                .iter()
                .enumerate()
                .map(|(i, date)| SectorAdjustedPrice {
                    date: date.clone(),
                    close: 50.0 + i as f64,
                })
                .collect(),
        })
        .collect();
    ValidateSectorDataInput {
        bundle: SectorDataBundle {
            schema_version: 1,
            manifest,
            retrieved_at: "2026-01-01T00:00:00Z".into(),
            series,
            pages: Vec::new(),
        },
        verification: SectorDataVerification {
            coverage_verified: true,
            adjustment_verified: true,
            corporate_actions_verified: true,
            adjustment_evidence: "synthetic price bases only".into(),
            corporate_actions_evidence: "synthetic fixture has no evaluation events".into(),
            corporate_actions: Vec::new(),
        },
        initial_cash_usd: 10_000.0,
        cost_bps: 10.0,
    }
}

#[derive(Clone, Copy)]
enum Fault {
    None,
    MissingAdjusted,
    Duplicate,
    Nonfinite,
    Zero,
    Future,
    Stalled,
    Provider,
    ScopeChanged,
}
type Request = (String, String, String, bool);
struct MockSource {
    dates: Vec<String>,
    page_size: usize,
    fault: Fault,
    requests: Mutex<Vec<Request>>,
}
impl MockSource {
    fn new(fault: Fault, page_size: usize) -> Self {
        Self {
            dates: fixture_manifest().session_dates,
            page_size,
            fault,
            requests: Mutex::new(Vec::new()),
        }
    }
}
impl SectorQuoteSource for MockSource {
    async fn fetch(
        &self,
        symbol: &str,
        exchange: &str,
        anchor: &str,
        adjusted: bool,
    ) -> Result<Vec<SectorQuoteRow>, String> {
        self.requests.lock().unwrap().push((
            symbol.into(),
            exchange.into(),
            anchor.into(),
            adjusted,
        ));
        if matches!(self.fault, Fault::Provider) {
            return Err("do not expose secret account response-body".into());
        }
        if matches!(self.fault, Fault::ScopeChanged) {
            return Err("SECTOR_QUOTE_SCOPE_CHANGED".into());
        }
        let mut rows: Vec<_> = self
            .dates
            .iter()
            .rev()
            .filter(|date| {
                (matches!(self.fault, Fault::Stalled) || date.replace('-', "").as_str() <= anchor)
                    && !(adjusted
                        && matches!(self.fault, Fault::MissingAdjusted)
                        && **date == self.dates[23])
            })
            .take(self.page_size)
            .map(|date| SectorQuoteRow {
                date: date.replace('-', ""),
                open: "100".into(),
                close: if adjusted { "50" } else { "102" }.into(),
            })
            .collect();
        match self.fault {
            Fault::Duplicate if !rows.is_empty() => {
                let mut other = rows[0].clone();
                other.close = "999".into();
                rows.push(other);
            }
            Fault::Nonfinite if !rows.is_empty() => rows[0].close = "NaN".into(),
            Fault::Zero if !rows.is_empty() => rows[0].open = "0".into(),
            Fault::Future if !rows.is_empty() => rows[0].date = "20990101".into(),
            _ => {}
        }
        Ok(rows)
    }
}

#[test]
fn sector_data_preparation_preserves_price_bases_history_and_manual_claims() {
    let request = fixture_input();
    let report = validate_sector_bundle(&request);
    assert!(report.ready, "{:?}", report.diagnostics);
    let input = report.input.unwrap();
    assert_eq!(input.history.len(), 22);
    assert_eq!(input.evaluation.len(), 3);
    assert_eq!(input.history[0].prices["SPY"].open, 100.0);
    assert_eq!(input.history[0].prices["SPY"].close, 102.0);
    assert_eq!(input.history[0].prices["SPY"].adjusted_close, 50.0);
    assert!(input.source.synthetic);
    assert_eq!(report.bundle_hash.len(), 64);
    assert!(report.warnings.iter().any(|v| v.contains("자동 인증하지")));
    assert!(report.warnings.iter().any(|v| v.contains("총수익")));
}

#[test]
fn sector_data_verification_is_not_automatically_promoted() {
    let baseline = fixture_input();
    let original_hash = validate_sector_bundle(&baseline).bundle_hash;
    for index in 0..5 {
        let mut input = baseline.clone();
        match index {
            0 => input.verification.coverage_verified = false,
            1 => input.verification.adjustment_verified = false,
            2 => input.verification.corporate_actions_verified = false,
            3 => input.verification.adjustment_evidence.clear(),
            _ => input.verification.corporate_actions_evidence.clear(),
        }
        let report = validate_sector_bundle(&input);
        assert!(!report.ready);
        assert!(report.input.is_none());
        assert_eq!(report.diagnostics[0].code, "MANUAL_VERIFICATION_REQUIRED");
        assert_eq!(report.bundle_hash, original_hash);
        assert_ne!(
            report.verification_hash,
            validate_sector_bundle(&baseline).verification_hash
        );
    }
}

#[test]
fn sector_data_rejects_split_and_dividend_even_when_claimed_verified() {
    for kind in ["split", "dividend"] {
        let mut input = fixture_input();
        input
            .verification
            .corporate_actions
            .push(SectorCorporateAction {
                symbol: "SPY".into(),
                date: input.bundle.manifest.session_dates[22].clone(),
                kind: kind.into(),
                split_ratio: Some(2.0),
                dividend_per_share: Some(1.0),
                pay_date: None,
            });
        let report = validate_sector_bundle(&input);
        assert!(!report.ready);
        assert_eq!(report.diagnostics[0].code, "UNSUPPORTED_CORPORATE_ACTIONS");
        // Current engine intentionally also rejects history events if explicitly supplied.
        input.verification.corporate_actions[0].date =
            input.bundle.manifest.session_dates[0].clone();
        assert!(!validate_sector_bundle(&input).ready);
    }
}

#[test]
fn sector_data_rejects_missing_misaligned_duplicate_and_invalid_price_for_all_accounts() {
    for index in 0..8 {
        let mut input = fixture_input();
        match index {
            0 => {
                input.bundle.series[0].adjusted.pop();
            }
            1 => {
                input.bundle.series[0].adjusted[2].date =
                    input.bundle.series[0].adjusted[1].date.clone()
            }
            2 => input.bundle.series[0].raw[2].date = input.bundle.series[0].raw[1].date.clone(),
            3 => input.bundle.series[0].raw[0].open = 0.0,
            4 => input.bundle.series[0].adjusted[0].close = f64::NAN,
            5 => input.bundle.series[0].exchange = "AMS".into(),
            6 => input.bundle.series[0] = input.bundle.series[1].clone(),
            _ => input.initial_cash_usd = -1.0,
        }
        let report = validate_sector_bundle(&input);
        assert!(!report.ready, "case {index}");
        assert!(report.input.is_none());
    }
}

#[test]
fn sector_data_manifest_requires_exact_past_window_and_manual_mapping() {
    for index in 0..8 {
        let mut manifest = fixture_manifest();
        match index {
            0 => manifest.session_dates.swap(0, 1),
            1 => manifest.session_dates[1] = manifest.session_dates[0].clone(),
            2 => manifest.exchange_mappings[0].evidence.clear(),
            3 => manifest.exchange_mappings[0].exchange = "ARCA".into(),
            4 => manifest.calendar_evidence.clear(),
            5 => manifest.end_date = "2025-10-06".into(),
            6 => {
                manifest.session_dates.truncate(22);
                manifest.end_date = manifest.session_dates[21].clone();
            }
            _ => manifest.session_dates[0] = "2025-09-06".into(),
        }
        assert!(
            validate_manifest(&manifest, "2026-01-01T00:00:00Z").is_err(),
            "case {index}"
        );
    }
    let manifest = fixture_manifest();
    // 01:00 UTC next calendar day is still the same NY date: incomplete day rejected.
    let next_utc = (NaiveDate::parse_from_str(&manifest.end_date, "%Y-%m-%d").unwrap()
        + Duration::days(1))
    .to_string();
    assert!(validate_manifest(&manifest, &format!("{next_utc}T01:00:00Z")).is_err());
    assert!(validate_manifest(&manifest, "2099-01-01T00:00:00Z").is_err());
}

#[tokio::test]
async fn sector_data_mock_collection_is_sequential_paired_anchored_and_bounded() {
    let source = MockSource::new(Fault::None, 7);
    let bundle = collect_sector_bundle(&source, fixture_manifest(), "2026-01-01T00:00:00Z".into())
        .await
        .unwrap();
    assert_eq!(bundle.series.len(), 12);
    assert_eq!(bundle.pages.len(), 12 * 2 * 4);
    let requests = source.requests.lock().unwrap();
    assert_eq!(requests.len(), 96);
    for symbol_requests in requests.as_chunks::<8>().0 {
        assert!(symbol_requests[..4].iter().all(|r| !r.3));
        assert!(symbol_requests[4..].iter().all(|r| r.3));
        assert!(symbol_requests[..4].windows(2).all(|w| w[1].2 < w[0].2));
        assert_eq!(
            symbol_requests[0].2,
            fixture_manifest().end_date.replace('-', "")
        );
    }
    let mut request = fixture_input();
    request.bundle = bundle;
    let report = validate_sector_bundle(&request);
    assert!(report.ready, "{:?}", report.diagnostics);
    assert_eq!(
        report.input.unwrap().evaluation[0].prices["SPY"].adjusted_close,
        50.0
    );
}

#[tokio::test]
async fn sector_data_mock_failures_never_return_partial_data_or_provider_bodies() {
    for fault in [
        Fault::MissingAdjusted,
        Fault::Duplicate,
        Fault::Nonfinite,
        Fault::Zero,
        Fault::Future,
        Fault::Stalled,
        Fault::Provider,
        Fault::ScopeChanged,
    ] {
        let source = MockSource::new(fault, 7);
        let error =
            collect_sector_bundle(&source, fixture_manifest(), "2026-01-01T00:00:00Z".into())
                .await
                .unwrap_err();
        assert!(error.starts_with("SECTOR_QUOTE_"));
        assert!(!error.contains("secret"));
        assert!(!error.contains("response-body"));
        assert!(source.requests.lock().unwrap().len() <= 8);
    }
    let source = MockSource::new(Fault::None, 1);
    let error = collect_sector_bundle(&source, fixture_manifest(), "2026-01-01T00:00:00Z".into())
        .await
        .unwrap_err();
    assert_eq!(error, "SECTOR_QUOTE_PAGE_BOUND_REACHED");
    assert_eq!(source.requests.lock().unwrap().len(), 16);
}
