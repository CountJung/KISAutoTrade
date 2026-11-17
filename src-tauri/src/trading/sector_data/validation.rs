use super::*;
use crate::trading::sector_portfolio::{
    validate_sector_portfolio_input, SectorDataSource, SectorDay, SectorPortfolioInput,
    SectorPrice, ALL_SYMBOLS,
};
use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn hash<T: serde::Serialize>(value: &T) -> String {
    // All DTO values are serializable finite prices after validation; serde also handles
    // imported nonfinite f64 by emitting null. This is canonical DTO JSON, not raw bytes.
    hex::encode(Sha256::digest(
        serde_json::to_vec(value).unwrap_or_default(),
    ))
}

fn note(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
}

pub(super) fn date(value: &str) -> Result<NaiveDate, String> {
    let parsed = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| "INVALID_MANIFEST_DATE".to_string())?;
    if parsed.format("%Y-%m-%d").to_string() != value
        || matches!(parsed.weekday(), Weekday::Sat | Weekday::Sun)
    {
        return Err("INVALID_MANIFEST_SESSION".into());
    }
    Ok(parsed)
}

/// Calendar correctness is an explicit caller assertion, never an automatic certification.
pub fn validate_manifest(manifest: &SectorDataManifest, retrieved_at: &str) -> Result<(), String> {
    if !(23..=500).contains(&manifest.session_dates.len()) {
        return Err("INVALID_MANIFEST_SIZE: history22+evaluation1..478이 필요합니다.".into());
    }
    if !note(&manifest.calendar_source) || !note(&manifest.calendar_evidence) {
        return Err("MISSING_CALENDAR_EVIDENCE".into());
    }
    let mut previous = None;
    for label in &manifest.session_dates {
        let current = date(label)?;
        if previous.is_some_and(|last| current <= last) {
            return Err("NON_MONOTONIC_MANIFEST_DATES".into());
        }
        previous = Some(current);
    }
    if manifest.session_dates.last() != Some(&manifest.end_date) {
        return Err("MANIFEST_END_MISMATCH".into());
    }
    if retrieved_at.len() > 64 {
        return Err("INVALID_COLLECTION_TIME".into());
    }
    let retrieved = DateTime::parse_from_rfc3339(retrieved_at)
        .map_err(|_| "INVALID_COLLECTION_TIME".to_string())?;
    if retrieved.with_timezone(&Utc) > Utc::now()
        || date(&manifest.end_date)?
            >= retrieved
                .with_timezone(&chrono_tz::America::New_York)
                .date_naive()
    {
        return Err("UNFINISHED_OR_FUTURE_COLLECTION_DAY".into());
    }
    let mut symbols = BTreeSet::new();
    if manifest.exchange_mappings.len() != ALL_SYMBOLS.len() {
        return Err("INCOMPLETE_EXCHANGE_MAPPING".into());
    }
    for mapping in &manifest.exchange_mappings {
        if !ALL_SYMBOLS.contains(&mapping.symbol.as_str())
            || !symbols.insert(&mapping.symbol)
            || !matches!(mapping.exchange.as_str(), "NAS" | "NYS" | "AMS")
            || !note(&mapping.evidence)
        {
            return Err("UNVERIFIED_EXCHANGE_MAPPING".into());
        }
    }
    Ok(())
}

fn prices(bundle: &SectorDataBundle) -> Result<Vec<SectorDay>, String> {
    if bundle.schema_version != 1 || bundle.series.len() != ALL_SYMBOLS.len() {
        return Err("INVALID_BUNDLE_UNIVERSE_OR_VERSION".into());
    }
    // Page receipts cannot establish completeness or prove the origin of an imported bundle.
    if bundle.pages.len() > 384 {
        return Err("TOO_MANY_PAGE_RECEIPTS".into());
    }
    for page in &bundle.pages {
        let mapping = bundle
            .manifest
            .exchange_mappings
            .iter()
            .find(|m| m.symbol == page.symbol);
        if mapping.is_none_or(|m| m.exchange != page.exchange)
            || !matches!(page.modp.as_str(), "0" | "1")
            || page.row_count > 500
            || page.rows_hash.len() != 64
            || !page.rows_hash.bytes().all(|c| c.is_ascii_hexdigit())
            || NaiveDate::parse_from_str(&page.anchor, "%Y%m%d").is_err()
        {
            return Err("INVALID_PAGE_RECEIPT".into());
        }
    }
    let expected = &bundle.manifest.session_dates;
    let mut days: Vec<_> = expected
        .iter()
        .map(|date| SectorDay {
            date: date.clone(),
            prices: BTreeMap::new(),
        })
        .collect();
    let mut seen = BTreeSet::new();
    for series in &bundle.series {
        let mapping = bundle
            .manifest
            .exchange_mappings
            .iter()
            .find(|m| m.symbol == series.symbol);
        if !seen.insert(&series.symbol) || mapping.is_none_or(|m| m.exchange != series.exchange) {
            return Err("BUNDLE_MAPPING_MISMATCH".into());
        }
        if series.raw.len() != expected.len() || series.adjusted.len() != expected.len() {
            return Err("INCOMPLETE_PAIRED_PRICES".into());
        }
        for (index, (raw, adjusted)) in series.raw.iter().zip(&series.adjusted).enumerate() {
            if raw.date != expected[index] || adjusted.date != expected[index] {
                return Err("PAIRED_PRICE_SESSION_MISMATCH".into());
            }
            if [raw.open, raw.close, adjusted.close]
                .iter()
                .any(|v| !v.is_finite() || !(0.000_001..=1_000_000_000.0).contains(v))
            {
                return Err("INVALID_PAIRED_PRICE".into());
            }
            days[index].prices.insert(
                series.symbol.clone(),
                SectorPrice {
                    adjusted_close: adjusted.close,
                    open: raw.open,
                    close: raw.close,
                },
            );
        }
    }
    Ok(days)
}

pub fn validate_sector_bundle(request: &ValidateSectorDataInput) -> SectorDataValidationReport {
    let mut report = SectorDataValidationReport {
        ready: false, diagnostics: Vec::new(), input: None,
        bundle_hash: hash(&request.bundle), verification_hash: hash(&request.verification),
        warnings: vec![
            "달력·EXCD·조정 및 기업행위 증거는 입력자 선언입니다. 프로그램이 거래소/공급원 사실을 자동 인증하지 않습니다.".into(),
            "KIS MODP=0 미반영 / MODP=1 반영만 확인되었습니다. 배당 총수익 의미·ETF EXCD·페이지100봉 보장은 별도 확인이 필요합니다.".into(),
            "pages.rowsHash와 bundleHash는 정규화 DTO JSON의 SHA256이며 HTTP 응답 원문·출처 진위를 인증하지 않습니다.".into(),
            "기업행위 부재 선언은 평가창 전체에 필요합니다. 사전22일은 보유하지 않지만 adjustmentEvidence로 조정 신호의 의미를 확인해야 합니다. 입력 corporateActions가 비어 있지 않으면 전체 평가를 거부합니다.".into(),
            "실행은 다음 거래일 시가의 사후 근사이며 실제 09:50 관측값/모의 체결과 다릅니다.".into(),
        ],
    };
    let checked: Result<SectorPortfolioInput, String> = (|| {
        validate_manifest(&request.bundle.manifest, &request.bundle.retrieved_at)?;
        let days = prices(&request.bundle)?;
        let v = &request.verification;
        if !v.coverage_verified
            || !v.adjustment_verified
            || !v.corporate_actions_verified
            || !note(&v.adjustment_evidence)
            || !note(&v.corporate_actions_evidence)
        {
            return Err("MANUAL_VERIFICATION_REQUIRED".into());
        }
        if !v.corporate_actions.is_empty() {
            return Err("UNSUPPORTED_CORPORATE_ACTIONS".into());
        }
        let input = SectorPortfolioInput {
            initial_cash_usd: request.initial_cash_usd,
            cost_bps: request.cost_bps,
            source: SectorDataSource {
                name: if request.bundle.manifest.synthetic {
                    "synthetic sector paired-price fixture"
                } else {
                    "KIS HHDFS76240000 dailyprice MODP0/MODP1; caller-verified"
                }
                .into(),
                retrieved_at: request.bundle.retrieved_at.clone(),
                synthetic: request.bundle.manifest.synthetic,
                adjustment_policy: "adjustedSignalRawExecution".into(),
                calendar_source: request.bundle.manifest.calendar_source.clone(),
                coverage_verified: v.coverage_verified,
                corporate_actions_verified: v.corporate_actions_verified,
            },
            history: days[..22].to_vec(),
            evaluation: days[22..].to_vec(),
            expected_session_dates: request.bundle.manifest.session_dates.clone(),
            corporate_actions: v.corporate_actions.clone(),
        };
        validate_sector_portfolio_input(&input)
            .map_err(|_| "INVALID_REPLAY_ASSUMPTIONS".to_string())?;
        Ok(input)
    })();
    match checked {
        Ok(input) => {
            report.ready = true;
            report.input = Some(input);
        }
        Err(code) => report.diagnostics.push(SectorDataDiagnostic {
            message: diagnostic_message(&code).into(),
            code,
        }),
    }
    report
}

fn diagnostic_message(code: &str) -> &'static str {
    match code {
        "MANUAL_VERIFICATION_REQUIRED" => {
            "거래일 coverage·조정 의미·평가창 기업행위 부재의 수동 확인과 근거가 필요합니다."
        }
        "UNSUPPORTED_CORPORATE_ACTIONS" => {
            "분할/배당 정산 미지원으로 해당 입력 전체는 평가할 수 없습니다."
        }
        "INCOMPLETE_PAIRED_PRICES" | "PAIRED_PRICE_SESSION_MISMATCH" => {
            "모든 12종목 MODP0/1의 날짜가 선언 거래일과 정확히 같아야 합니다."
        }
        _ => "자료 계약 검증을 통과하지 못했습니다. 코드와 manifest·가격·근거를 확인하세요.",
    }
}
