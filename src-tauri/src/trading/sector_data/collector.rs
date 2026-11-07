use super::*;
use chrono::{Duration, NaiveDate};
use std::{collections::BTreeMap, future::Future};

/// Implementations must sanitize transport/auth errors and must not log response bodies.
pub trait SectorQuoteSource {
    fn fetch(
        &self,
        symbol: &str,
        exchange: &str,
        anchor: &str,
        adjusted: bool,
    ) -> impl Future<Output = Result<Vec<SectorQuoteRow>, String>> + Send;
}

/// No account state, persistence, orders or background tasks. All requests are sequential.
/// At most 12 symbols × two MODP modes × 16 pages, with no retries.
pub async fn collect_sector_bundle<S: SectorQuoteSource>(
    source: &S,
    manifest: SectorDataManifest,
    retrieved_at: String,
) -> Result<SectorDataBundle, String> {
    validate_manifest(&manifest, &retrieved_at)?;
    let mut bundle = SectorDataBundle {
        schema_version: 1,
        manifest,
        retrieved_at,
        series: Vec::new(),
        pages: Vec::new(),
    };
    for mapping in &bundle.manifest.exchange_mappings {
        let raw = collect_mode(source, &bundle.manifest, mapping, false, &mut bundle.pages).await?;
        let adjusted =
            collect_mode(source, &bundle.manifest, mapping, true, &mut bundle.pages).await?;
        bundle.series.push(SectorDataSeries {
            symbol: mapping.symbol.clone(),
            exchange: mapping.exchange.clone(),
            raw: raw
                .iter()
                .map(|(date, (open, close))| SectorRawPrice {
                    date: date.clone(),
                    open: *open,
                    close: *close,
                })
                .collect(),
            adjusted: adjusted
                .iter()
                .map(|(date, (_, close))| SectorAdjustedPrice {
                    date: date.clone(),
                    close: *close,
                })
                .collect(),
        });
    }
    Ok(bundle)
}

async fn collect_mode<S: SectorQuoteSource>(
    source: &S,
    manifest: &SectorDataManifest,
    mapping: &SectorExchangeMapping,
    adjusted: bool,
    receipts: &mut Vec<SectorPageReceipt>,
) -> Result<BTreeMap<String, (f64, f64)>, String> {
    let first = &manifest.session_dates[0];
    let mut anchor = manifest.end_date.replace('-', "");
    let mut retained = BTreeMap::new();
    for _ in 0..16 {
        // Errors from an arbitrary source are deliberately not echoed to reports/logs.
        let rows = source
            .fetch(&mapping.symbol, &mapping.exchange, &anchor, adjusted)
            .await
            .map_err(|_| "SECTOR_QUOTE_FETCH_FAILED".to_string())?;
        if rows.is_empty() || rows.len() > 500 {
            return Err("SECTOR_QUOTE_PAGE_EMPTY_OR_OVERSIZED".into());
        }
        let mut page = BTreeMap::new();
        for row in &rows {
            if row.date.len() != 8 || !row.date.bytes().all(|c| c.is_ascii_digit()) {
                return Err("SECTOR_QUOTE_INVALID_DATE".into());
            }
            let parsed = NaiveDate::parse_from_str(&row.date, "%Y%m%d")
                .map_err(|_| "SECTOR_QUOTE_INVALID_DATE".to_string())?;
            let label = parsed.format("%Y-%m-%d").to_string();
            if row.date > anchor || label > manifest.end_date {
                return Err("SECTOR_QUOTE_AFTER_FROZEN_ANCHOR".into());
            }
            let open = price(&row.open)?;
            let close = price(&row.close)?;
            if page.insert(label, (open, close)).is_some() {
                return Err("SECTOR_QUOTE_DUPLICATE_DATE".into());
            }
        }
        receipts.push(SectorPageReceipt {
            symbol: mapping.symbol.clone(),
            exchange: mapping.exchange.clone(),
            modp: if adjusted { "1" } else { "0" }.into(),
            anchor: anchor.clone(),
            row_count: rows.len(),
            rows_hash: super::validation::hash(&rows),
        });
        let oldest = page.first_key_value().expect("nonempty page").0.clone();
        for (label, values) in page {
            if label >= *first && retained.insert(label, values).is_some() {
                return Err("SECTOR_QUOTE_REPEATED_PAGE".into());
            }
        }
        if oldest <= *first {
            if retained.keys().ne(manifest.session_dates.iter()) {
                return Err("SECTOR_QUOTE_SESSION_GAP_OR_EXTRA".into());
            }
            return Ok(retained);
        }
        anchor = NaiveDate::parse_from_str(&oldest, "%Y-%m-%d")
            .map_err(|_| "SECTOR_QUOTE_INVALID_DATE".to_string())?
            .checked_sub_signed(Duration::days(1))
            .ok_or_else(|| "SECTOR_QUOTE_ANCHOR_UNDERFLOW".to_string())?
            .format("%Y%m%d")
            .to_string();
    }
    Err("SECTOR_QUOTE_PAGE_BOUND_REACHED".into())
}

fn price(raw: &str) -> Result<f64, String> {
    if raw.len() > 64 {
        return Err("SECTOR_QUOTE_INVALID_PRICE".into());
    }
    let price: f64 = raw
        .parse()
        .map_err(|_| "SECTOR_QUOTE_INVALID_PRICE".to_string())?;
    if !price.is_finite() || !(0.000_001..=1_000_000_000.0).contains(&price) {
        return Err("SECTOR_QUOTE_INVALID_PRICE".into());
    }
    Ok(price)
}
