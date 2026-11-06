//! Bounded raw-chart collection. Only broker quote sources enter this module.
use super::sector_portfolio::*;
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
};
mod provider;
pub use provider::BrokerChartClient;

#[cfg(test)]
mod tests;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectBrokerChartInput {
    pub expected_profile_id: String,
    pub authenticated_quotes_acknowledged: bool,
    pub evaluation_bars: usize,
    pub end_date: Option<String>,
    #[serde(default)]
    pub exchange_mappings: Vec<super::sector_data::SectorExchangeMapping>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerChartPageReceipt {
    pub symbol: String,
    pub exchange: Option<String>,
    pub page: usize,
    pub row_count: usize,
    pub oldest_date: String,
    pub newest_date: String,
    pub rows_hash: String,
    pub request_adjustment: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerChartCollection {
    pub input: BrokerChartInput,
    pub diagnostics: Vec<String>,
    pub receipts: Vec<BrokerChartPageReceipt>,
    /// Effective KIS quotation codes and their evidence, not listing-venue certification.
    /// Toss uses its symbol API and therefore exports an empty mapping list.
    pub exchange_mappings: Vec<super::sector_data::SectorExchangeMapping>,
    pub collection_hash: String,
}

const KIS_AMS_MASTER_URL: &str = "https://new.real.download.dws.co.kr/common/master/amsmst.cod.zip";
const KIS_AMS_MASTER_RETRIEVED: &str = "2026-11-06T07:17:06.817973Z";
const KIS_AMS_MASTER_SHA256: &str =
    "508592b86497915bad003ee8917cf6fc215c43cb2cb76377d3b47deda17424ad";

/// Verified positive rows for the fixed twelve symbols in AMSMST.COD: US/USD/type3/AMS.
/// This is a dated snapshot, never a runtime master fetch or an Arca inference.
fn kis_default_mappings() -> Vec<super::sector_data::SectorExchangeMapping> {
    ALL_SYMBOLS.iter().map(|symbol| super::sector_data::SectorExchangeMapping {
        symbol: (*symbol).into(), exchange: "AMS".into(),
        evidence: format!("KIS AMSMST.COD US/USD/type3 Exchange code=AMS; URL={KIS_AMS_MASTER_URL}; retrievedAt={KIS_AMS_MASTER_RETRIEVED}; archiveSHA256={KIS_AMS_MASTER_SHA256}; dated snapshot, not listing-venue certification"),
    }).collect()
}

pub struct RawBrokerChartPage {
    pub rows: Vec<BrokerChartDay>,
    pub next_cursor: Option<String>,
}

/// Fetch one symbol only; each returned day's map must contain exactly that symbol.
pub trait BrokerChartQuoteSource {
    fn fetch(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        end_date: &str,
        cursor: Option<&str>,
    ) -> impl Future<Output = Result<RawBrokerChartPage, String>> + Send;
}

fn hash<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|b| hex::encode(Sha256::digest(b)))
        .map_err(|_| "BROKER_CHART_HASH_FAILED".into())
}

fn session(label: &str) -> Result<NaiveDate, String> {
    let day =
        NaiveDate::parse_from_str(label, "%Y-%m-%d").map_err(|_| "BROKER_CHART_INVALID_DATE")?;
    if day.format("%Y-%m-%d").to_string() != label
        || matches!(day.weekday(), Weekday::Sat | Weekday::Sun)
    {
        return Err("BROKER_CHART_INVALID_SESSION".into());
    }
    Ok(day)
}

fn valid_price(price: &BrokerChartPrice) -> bool {
    [price.open, price.high, price.low, price.close]
        .iter()
        .all(|x| x.is_finite() && (0.000_001..=1e9).contains(x))
        && price.high >= price.open.max(price.close)
        && price.low <= price.open.min(price.close)
        && price.high >= price.low
        && price.volume.is_finite()
        && (0.0..=9_007_199_254_740_991.0).contains(&price.volume)
}

fn same_price(a: &BrokerChartPrice, b: &BrokerChartPrice) -> bool {
    [a.open, a.high, a.low, a.close, a.volume] == [b.open, b.high, b.low, b.close, b.volume]
        && a.currency == b.currency
}

async fn series<S: BrokerChartQuoteSource>(
    source: &S,
    symbol: &str,
    exchange: Option<&str>,
    end: &str,
    target: usize,
    first: Option<&str>,
    receipts: &mut Vec<BrokerChartPageReceipt>,
) -> Result<BTreeMap<String, BrokerChartPrice>, String> {
    let mut rows = BTreeMap::new();
    let mut cursor = None;
    let mut cursors = BTreeSet::new();
    for page_number in 1..=16 {
        let page = source
            .fetch(symbol, exchange, end, cursor.as_deref())
            .await?;
        if page.rows.is_empty() {
            return Err("BROKER_CHART_EMPTY_ROWS".into());
        }
        if page.rows.len() > 500 {
            return Err("BROKER_CHART_OVERSIZED_PAGE".into());
        }
        let mut page_rows = BTreeMap::new();
        for day in &page.rows {
            session(&day.date)?;
            if day.date.as_str() > end || day.prices.len() != 1 || !day.prices.contains_key(symbol)
            {
                return Err("BROKER_CHART_PAGE_IDENTITY_OR_DATE".into());
            }
            let price = &day.prices[symbol];
            if !valid_price(price) {
                return Err("BROKER_CHART_INVALID_OHLCV".into());
            }
            if page_rows.insert(day.date.clone(), price.clone()).is_some() {
                return Err("BROKER_CHART_DUPLICATE_PAGE_DATE".into());
            }
        }
        let oldest = page_rows
            .first_key_value()
            .ok_or("BROKER_CHART_EMPTY_PAGE")?
            .0
            .clone();
        let newest = page_rows
            .last_key_value()
            .ok_or("BROKER_CHART_EMPTY_PAGE")?
            .0
            .clone();
        receipts.push(BrokerChartPageReceipt {
            symbol: symbol.into(),
            exchange: exchange.map(str::to_string),
            page: page_number,
            row_count: page.rows.len(),
            oldest_date: oldest.clone(),
            newest_date: newest,
            rows_hash: hash(&page.rows)?,
            request_adjustment: "unadjusted".into(),
        });
        let before = rows.len();
        for (date, price) in page_rows {
            if let Some(prior) = rows.get(&date) {
                if !same_price(prior, &price) {
                    return Err("BROKER_CHART_CONFLICTING_OVERLAP".into());
                }
            } else {
                rows.insert(date, price);
            }
        }
        if (first.is_none() && rows.len() >= target)
            || first.is_some_and(|start| oldest.as_str() <= start)
        {
            return Ok(rows);
        }
        if rows.len() == before {
            return Err("BROKER_CHART_PAGE_DID_NOT_ADVANCE".into());
        }
        let next = page
            .next_cursor
            .ok_or("BROKER_CHART_INSUFFICIENT_HISTORY")?;
        if next.len() > 128 || !cursors.insert(next.clone()) {
            return Err("BROKER_CHART_REPEATED_CURSOR".into());
        }
        cursor = Some(next);
    }
    Err("BROKER_CHART_PAGE_BOUND".into())
}

/// No credentials, account balances, orders or persistence. SPY dates are observed,
/// not a certified exchange calendar; missing dates in another ETF are fatal.
pub async fn collect_broker_chart<S: BrokerChartQuoteSource>(
    source: &S,
    broker: BrokerChartBroker,
    evaluation_bars: usize,
    end_date: &str,
    retrieved_at: &str,
    mappings: &[super::sector_data::SectorExchangeMapping],
) -> Result<BrokerChartCollection, String> {
    if !(1..=120).contains(&evaluation_bars) {
        return Err("BROKER_CHART_INVALID_WINDOW".into());
    }
    let end =
        NaiveDate::parse_from_str(end_date, "%Y-%m-%d").map_err(|_| "BROKER_CHART_INVALID_END")?;
    if end.format("%Y-%m-%d").to_string() != end_date {
        return Err("BROKER_CHART_INVALID_END".into());
    }
    let collected = chrono::DateTime::parse_from_rfc3339(retrieved_at)
        .map_err(|_| "BROKER_CHART_INVALID_RETRIEVAL")?;
    if collected.with_timezone(&chrono::Utc) > chrono::Utc::now() {
        return Err("BROKER_CHART_FUTURE_RETRIEVAL".into());
    }
    if end
        >= collected
            .with_timezone(&chrono_tz::America::New_York)
            .date_naive()
    {
        return Err("BROKER_CHART_UNFINISHED_DAY".into());
    }
    let default_mapping = broker == BrokerChartBroker::Kis && mappings.is_empty();
    let exchange_mappings = match broker {
        BrokerChartBroker::Toss if !mappings.is_empty() => {
            return Err("BROKER_CHART_TOSS_MAPPING_NOT_SUPPORTED".into())
        }
        BrokerChartBroker::Toss => Vec::new(),
        BrokerChartBroker::Kis if default_mapping => kis_default_mappings(),
        BrokerChartBroker::Kis => mappings.to_vec(),
    };
    let mappings = exchange_mappings.as_slice();
    if !mappings.is_empty() {
        let names: BTreeSet<_> = mappings.iter().map(|x| x.symbol.as_str()).collect();
        if mappings.len() != 12
            || names.len() != 12
            || names.iter().any(|s| !ALL_SYMBOLS.contains(s))
            || mappings.iter().any(|x| {
                !matches!(x.exchange.as_str(), "NAS" | "NYS" | "AMS")
                    || x.evidence.trim().is_empty()
                    || x.evidence.len() > 512
                    || x.evidence.chars().any(char::is_control)
            })
        {
            return Err("BROKER_CHART_INVALID_MAPPING".into());
        }
    }
    let mut receipts = Vec::new();
    let mut values = BTreeMap::new();
    let total = 22 + evaluation_bars;
    let mut dates: Vec<String> = Vec::new();
    for symbol in std::iter::once("SPY").chain(ALL_SYMBOLS.iter().copied().filter(|s| *s != "SPY"))
    {
        let exchange = if broker == BrokerChartBroker::Toss {
            None
        } else {
            Some(
                mappings
                    .iter()
                    .find(|m| m.symbol == symbol)
                    .ok_or("BROKER_CHART_MISSING_MAPPING")?
                    .exchange
                    .as_str(),
            )
        };
        // One selected API code per symbol. Stale mappings, empty success pages,
        // auth/provider/HTTP errors all stop; no exchange probing or substitution.
        let rows = series(
            source,
            symbol,
            exchange,
            end_date,
            total,
            dates.first().map(String::as_str),
            &mut receipts,
        )
        .await?;
        if symbol == "SPY" {
            dates = rows.keys().rev().take(total).cloned().collect();
            dates.reverse();
            if dates.len() != total {
                return Err("BROKER_CHART_INSUFFICIENT_SPY_HISTORY".into());
            }
        }
        if dates.iter().any(|d| !rows.contains_key(d)) {
            return Err(format!("BROKER_CHART_SESSION_GAP: {symbol}"));
        }
        values.insert(symbol.to_string(), rows);
    }
    let days: Vec<BrokerChartDay> = dates
        .iter()
        .map(|d| BrokerChartDay {
            date: d.clone(),
            prices: values
                .iter()
                .map(|(s, rows)| (s.clone(), rows[d].clone()))
                .collect(),
        })
        .collect();
    let broker_name = if broker == BrokerChartBroker::Kis {
        "KIS"
    } else {
        "Toss"
    };
    let input=BrokerChartInput{schema_version:1,initial_cash_usd:10000.0,cost_bps:10.0,source:BrokerChartSource{broker_id:broker,name:format!("{broker_name} daily unadjusted chart; price-only approximation"),retrieved_at:retrieved_at.into(),synthetic:false,adjustment:BrokerChartAdjustment::Unadjusted,currency:BrokerChartCurrency::Usd,interval:BrokerChartInterval::Daily,session_basis:BrokerChartSessionBasis::ObservedSpySessions,calendar_evidence:"Same-provider SPY returned daily dates; exchange-calendar completeness is not independently certified. API route acceptance is not listing-venue certification.".into()},history:days[..22].to_vec(),evaluation:days[22..].to_vec(),expected_session_dates:dates,acknowledged_price_only:false,known_corporate_actions:Vec::new()};
    let collection_hash = hash(&(&input, &receipts, &exchange_mappings))?;
    let mut diagnostics = vec!["동일 증권사 SPY의 관측 거래일을 기준으로12종목을 대조했습니다. 공식 달력·당시 버전의 완전성 인증은 아닙니다.".into(),"기업행위 목록이 비어 있어도 부재를 검증한 것이 아닙니다. 배당/분할 미정산 가격근사이며 한계 확인 후 계산하세요.".into(),"페이지 hash는 보존 시세 필드의 정규화 hash이며 HTTP 원문이나 출처 진위 인증이 아닙니다.".into()];
    if default_mapping {
        diagnostics.push("KIS 기본 조회코드는 공식 AMSMST.COD의 2026-11-06 스냅샷(12종목 US/USD/ETP type3/AMS)을 사용합니다. URL·수집시각·archive SHA256은 exchangeMappings에 보존했습니다. 현재 상장시장·매핑 최신성의 자동 인증이 아니며, 오래되어 오류나 빈 자료가 반환되면 다른 거래소 코드로 추정하지 않고 중단합니다.".into());
    } else if broker == BrokerChartBroker::Kis {
        diagnostics.push("KIS 사용자 지정 12개 조회 매핑을 사용했습니다. 오류나 빈 자료가 반환되면 다른 거래소 코드를 자동 추정하지 않고 중단합니다.".into());
    }
    Ok(BrokerChartCollection {
        input,
        receipts,
        exchange_mappings,
        collection_hash,
        diagnostics,
    })
}
