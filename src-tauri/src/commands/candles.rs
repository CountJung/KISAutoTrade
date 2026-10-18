//! Read-only page API shared by desktop and web, with a frozen scope/cutoff.
use super::*;
use crate::broker::{
    BrokerCandle, BrokerCandlePage, CandleCursor, CandlePageRequest, KisBrokerAdapter,
};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlePageInput {
    pub symbol: String,
    pub broker_id: BrokerId,
    pub is_overseas: bool,
    pub interval: String,
    pub count: Option<u16>,
    pub exchange: Option<String>,
    pub cursor: Option<String>,
    pub expected_profile_id: String,
    pub expected_broker_account_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlePageView {
    pub candles: Vec<ChartCandle>,
    pub next_cursor: Option<String>,
    pub end_reason: Option<String>,
    pub broker_id: BrokerId,
    pub profile_id: String,
    pub scope_key: String,
    pub symbol: String,
    pub interval: String,
    pub exchange: Option<String>,
    pub source: String,
    pub timestamp_semantics: String,
    pub completion_policy: String,
    pub as_of: String,
    pub excluded_unfinished_bars: usize,
    pub excluded_invalid_bars: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageCursor {
    version: u8,
    scope_key: String,
    as_of: DateTime<Utc>,
    provider_cursor: CandleCursor,
}

fn page_error(code: &str, message: impl ToString) -> CmdError {
    CmdError {
        code: code.into(),
        message: message.to_string(),
    }
}

fn normalize_input(input: &mut CandlePageInput) -> CmdResult<()> {
    input.symbol = input.symbol.trim().to_uppercase();
    input.interval = match input.interval.as_str() {
        "1d" | "D" | "d" => "D",
        "1m" | "M1" => "1m",
        "W" => "W",
        "M" => "M",
        _ => {
            return Err(page_error(
                "INVALID_INTERVAL",
                "지원하지 않는 캔들 간격입니다.",
            ))
        }
    }
    .into();
    if (input.broker_id == BrokerId::Kis && input.interval == "1m")
        || (input.broker_id == BrokerId::Toss && matches!(input.interval.as_str(), "W" | "M"))
    {
        return Err(page_error(
            "INVALID_INTERVAL",
            "선택한 broker가 지원하지 않는 캔들 간격입니다.",
        ));
    }
    if input
        .cursor
        .as_ref()
        .is_some_and(|cursor| cursor.len() > 2048)
    {
        return Err(page_error(
            "INVALID_CURSOR",
            "캔들 커서 길이 상한을 초과했습니다.",
        ));
    }
    let request = domain_request(input, None);
    request
        .validate()
        .map_err(|e| page_error("INVALID_CANDLE_REQUEST", e))
}

fn domain_request(input: &CandlePageInput, cursor: Option<CandleCursor>) -> CandlePageRequest {
    CandlePageRequest {
        symbol: BrokerSymbol(input.symbol.clone()),
        market: if input.is_overseas {
            BrokerMarket::Us
        } else {
            BrokerMarket::Kr
        },
        interval: input.interval.clone(),
        count: input.count.unwrap_or(200),
        exchange: input.exchange.clone(),
        cursor,
    }
}

fn scope_key(input: &CandlePageInput, profile: &AccountProfile) -> String {
    let contract = serde_json::json!([
        profile.id,
        profile.broker_id,
        profile.account_no,
        profile.is_paper_trading,
        input.symbol,
        input.is_overseas,
        input.interval,
        input.exchange,
        input.count.unwrap_or(200)
    ]);
    format!("{:x}", Sha256::digest(contract.to_string().as_bytes()))
}

fn decode_cursor(
    input: &CandlePageInput,
    scope_key: &str,
    now: DateTime<Utc>,
) -> CmdResult<(DateTime<Utc>, Option<CandleCursor>)> {
    let Some(cursor) = &input.cursor else {
        return Ok((now, None));
    };
    let cursor: PageCursor = serde_json::from_str(cursor)
        .map_err(|_| page_error("INVALID_CURSOR", "올바르지 않은 캔들 커서입니다."))?;
    if cursor.version != 1 || cursor.scope_key != scope_key || cursor.as_of > now {
        return Err(page_error(
            "SCOPE_MISMATCH",
            "캔들 커서의 계좌·종목·간격·기준 시각이 현재 요청과 다릅니다.",
        ));
    }
    let valid_provider_cursor = match (&cursor.provider_cursor, input.broker_id) {
        (CandleCursor::Kis { before }, BrokerId::Kis) => {
            before.len() == 8
                && before.chars().all(|c| c.is_ascii_digit())
                && NaiveDate::parse_from_str(before, "%Y%m%d")
                    .is_ok_and(|day| day <= market_date(cursor.as_of, input.is_overseas))
        }
        (CandleCursor::Toss { before }, BrokerId::Toss) => {
            before.len() <= 64
                && DateTime::parse_from_rfc3339(before).is_ok_and(|time| time <= cursor.as_of)
        }
        _ => false,
    };
    if !valid_provider_cursor {
        return Err(page_error(
            "INVALID_CURSOR",
            "provider와 기준 시각에 맞지 않는 캔들 커서입니다.",
        ));
    }
    Ok((cursor.as_of, Some(cursor.provider_cursor)))
}

fn market_date(as_of: DateTime<Utc>, overseas: bool) -> NaiveDate {
    if overseas {
        as_of
            .with_timezone(&chrono_tz::America::New_York)
            .date_naive()
    } else {
        as_of.with_timezone(&chrono_tz::Asia::Seoul).date_naive()
    }
}

fn completed_before(as_of: DateTime<Utc>, overseas: bool, interval: &str) -> NaiveDate {
    let today = market_date(as_of, overseas);
    match interval {
        "W" => today - chrono::Duration::days(i64::from(today.weekday().num_days_from_monday())),
        "M" => today.with_day(1).expect("first day is valid"),
        _ => today,
    }
}

fn valid_ohlcv(bar: &BrokerCandle, overseas: bool) -> bool {
    let expected = if overseas {
        crate::broker::BrokerCurrency::Usd
    } else {
        crate::broker::BrokerCurrency::Krw
    };
    let values = [&bar.open, &bar.high, &bar.low, &bar.close];
    if values.iter().any(|money| money.currency != expected) {
        return false;
    }
    let parsed: Option<Vec<f64>> = values
        .iter()
        .map(|money| money.amount.replace(',', "").parse::<f64>().ok())
        .collect();
    let Some(p) = parsed else {
        return false;
    };
    let volume = bar.volume.0.replace(',', "").parse::<f64>().ok();
    p.iter().all(|v| {
        v.is_finite() && *v > 0.0 && *v * if overseas { 100.0 } else { 1.0 } < u64::MAX as f64
    }) && p[1] >= p[0].max(p[3])
        && p[2] <= p[0].min(p[3])
        && p[1] >= p[2]
        && volume.is_some_and(|v| v.is_finite() && v >= 0.0)
}

fn page_view(
    input: &CandlePageInput,
    profile_id: String,
    scope_key: String,
    as_of: DateTime<Utc>,
    page: BrokerCandlePage,
) -> CmdResult<CandlePageView> {
    if page.candles.len() > 200 {
        return Err(page_error(
            "CANDLE_PAGE_FAILED",
            "provider의 캔들 페이지 크기가 상한을 초과했습니다.",
        ));
    }
    let mut invalid = 0;
    let mut unfinished = 0;
    let mut rows = std::collections::BTreeMap::new();
    let cutoff = completed_before(as_of, input.is_overseas, &input.interval);
    for bar in page.candles {
        if !valid_ohlcv(&bar, input.is_overseas) {
            invalid += 1;
            continue;
        }
        let (key, date, complete) = if input.interval == "1m" {
            let Ok(time) = DateTime::parse_from_rfc3339(&bar.date) else {
                invalid += 1;
                continue;
            };
            (time.timestamp_millis(), bar.date.clone(), time <= as_of)
        } else {
            let day = match bar.date.len() {
                8 => NaiveDate::parse_from_str(&bar.date, "%Y%m%d").ok(),
                10 => NaiveDate::parse_from_str(&bar.date, "%Y-%m-%d").ok(),
                _ => DateTime::parse_from_rfc3339(&bar.date)
                    .ok()
                    .map(|time| market_date(time.with_timezone(&Utc), input.is_overseas)),
            };
            let Some(day) = day else {
                invalid += 1;
                continue;
            };
            (
                i64::from(day.num_days_from_ce()),
                day.format("%Y%m%d").to_string(),
                day < cutoff,
            )
        };
        if !complete {
            unfinished += 1;
            continue;
        }
        rows.insert(
            key,
            ChartCandle {
                date,
                open: bar.open.amount,
                high: bar.high.amount,
                low: bar.low.amount,
                close: bar.close.amount,
                volume: bar.volume.0,
            },
        );
    }
    let count = usize::from(input.count.unwrap_or(200));
    let skip = rows.len().saturating_sub(count);
    let candles = rows.into_values().skip(skip).collect();
    let next_cursor = page
        .next_cursor
        .map(|provider_cursor| {
            serde_json::to_string(&PageCursor {
                version: 1,
                scope_key: scope_key.clone(),
                as_of,
                provider_cursor,
            })
        })
        .transpose()
        .map_err(|error| page_error("CURSOR_SERIALIZATION", error))?;
    Ok(CandlePageView {
        candles,
        next_cursor,
        end_reason: page.end_reason,
        broker_id: input.broker_id,
        profile_id,
        scope_key,
        symbol: input.symbol.clone(),
        interval: input.interval.clone(),
        exchange: input.exchange.clone(),
        source: format!(
            "{}:{}",
            if input.broker_id == BrokerId::Toss {
                "toss"
            } else {
                "kis"
            },
            input.interval
        ),
        timestamp_semantics: page.timestamp_semantics.into(),
        completion_policy: if input.interval == "1m" {
            "barCloseAtOrBeforeAsOf"
        } else {
            "strictlyBeforeCurrentMarketPeriod"
        }
        .into(),
        as_of: as_of.to_rfc3339(),
        excluded_unfinished_bars: unfinished,
        excluded_invalid_bars: invalid,
    })
}

fn validate_profile(input: &CandlePageInput, profile: &AccountProfile) -> CmdResult<()> {
    if input.broker_id != profile.broker_id
        || input.expected_profile_id != profile.id
        || input
            .expected_broker_account_id
            .as_deref()
            .is_some_and(|id| id != profile.account_no)
    {
        return Err(page_error(
            "SCOPE_MISMATCH",
            "캔들 요청의 활성 프로파일·broker·계좌가 변경되었습니다.",
        ));
    }
    Ok(())
}

pub async fn get_candle_page_for_profile(
    mut input: CandlePageInput,
    profile: AccountProfile,
    rest: Arc<KisRestClient>,
) -> CmdResult<CandlePageView> {
    normalize_input(&mut input)?;
    validate_profile(&input, &profile)?;
    let key = scope_key(&input, &profile);
    let (as_of, cursor) = decode_cursor(&input, &key, Utc::now())?;
    let request = domain_request(&input, cursor);
    let page = if profile.broker_id == BrokerId::Toss {
        TossBrokerAdapter::with_credentials(
            TossBrokerAdapter::DEFAULT_BASE_URL,
            profile.app_key,
            profile.app_secret,
            Some(profile.account_no),
        )
        .get_candle_page(&request)
        .await
    } else {
        if !rest.matches_account(
            &profile.app_key,
            &profile.app_secret,
            &profile.account_no,
            profile.is_paper_trading,
        ) {
            return Err(page_error(
                "SCOPE_MISMATCH",
                "KIS 시세 client와 활성 프로파일이 다릅니다. 다시 실행하세요.",
            ));
        }
        KisBrokerAdapter::new(rest).get_candle_page(&request).await
    }
    .map_err(|error| page_error("CANDLE_PAGE_FAILED", error))?;
    page_view(&input, profile.id, key, as_of, page)
}

#[tauri::command]
pub async fn get_candle_page(
    input: CandlePageInput,
    state: State<'_, AppState>,
) -> CmdResult<CandlePageView> {
    let profile = state
        .profiles
        .read()
        .await
        .get_active()
        .cloned()
        .ok_or_else(|| page_error("CONFIG_NOT_READY", "활성 프로파일이 없습니다."))?;
    let rest = state.rest_client.read().await.clone();
    get_candle_page_for_profile(input, profile, rest).await
}

#[cfg(test)]
mod tests;
