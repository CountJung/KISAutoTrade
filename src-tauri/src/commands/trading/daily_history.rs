//! Completed daily history is fetched independently of the quote/stop-loss loop.
use std::{collections::BTreeMap, future::Future, sync::Arc};

use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    api::rest::KisRestClient,
    broker::{candles::candle_market_date, BrokerId, BrokerMarket},
    commands::{get_candle_page_for_profile, CandlePageInput, CandlePageView, CmdError},
    config::AccountProfile,
    market_hours::is_domestic_symbol,
    trading::strategy::{
        parse_completed_daily_volume, CompletedDailyBar, DailyHistoryTicket, OhlcCandle,
    },
};

mod jobs;
pub(super) use jobs::DailyHistoryJobs;

const MAX_PAGES: usize = 8;
// Live strategy windows are bounded at 512; preview has a separate 500-bar limit.
const MAX_RETAINED: usize = 512;

#[derive(Debug)]
pub(super) struct CollectedDailyHistory {
    pub candles: Vec<CompletedDailyBar>,
    pub source: String,
    pub last_completed_date: Option<String>,
    pub end_reason: String,
}

fn price(value: &str, overseas: bool) -> Option<u64> {
    let amount = value.replace(',', "").parse::<f64>().ok()?;
    let units = amount * if overseas { 100.0 } else { 1.0 };
    (units.is_finite() && units.round() >= 1.0 && units < u64::MAX as f64)
        .then(|| units.round() as u64)
}

fn completed_bar(bar: &crate::api::rest::ChartCandle, overseas: bool) -> Option<CompletedDailyBar> {
    let ohlc = OhlcCandle {
        open: price(&bar.open, overseas)?,
        high: price(&bar.high, overseas)?,
        low: price(&bar.low, overseas)?,
        close: price(&bar.close, overseas)?,
    };
    (ohlc.high >= ohlc.open.max(ohlc.close) && ohlc.low <= ohlc.open.min(ohlc.close)).then_some(
        CompletedDailyBar {
            ohlc,
            volume: parse_completed_daily_volume(&bar.volume),
        },
    )
}

pub(super) async fn collect_daily_history(
    ticket: DailyHistoryTicket,
    profile: AccountProfile,
    rest: Arc<KisRestClient>,
) -> Result<CollectedDailyHistory, String> {
    collect_pages(&ticket, |input| {
        get_candle_page_for_profile(input, profile.clone(), rest.clone())
    })
    .await
}

/// The injectable boundary uses exactly the same DTO as production IPC/web.
async fn collect_pages<F, Fut>(
    ticket: &DailyHistoryTicket,
    mut fetch: F,
) -> Result<CollectedDailyHistory, String>
where
    F: FnMut(CandlePageInput) -> Fut,
    Fut: Future<Output = Result<CandlePageView, CmdError>>,
{
    let overseas = !is_domestic_symbol(&ticket.symbol);
    let exchanges = if ticket.broker_id == BrokerId::Kis && overseas {
        vec![
            Some("NAS".to_string()),
            Some("NYS".to_string()),
            Some("AMS".to_string()),
        ]
    } else {
        vec![None]
    };
    let mut exchange = 0;
    let mut cursor: Option<String> = None;
    let mut seen = std::collections::HashSet::new();
    let mut identity: Option<(String, String, String)> = None;
    let mut retained = BTreeMap::new();
    let target = ticket.required_bars.min(MAX_RETAINED);
    let mut end_reason = "최대 8회 조회 도달".to_string();
    for attempt in 0..MAX_PAGES {
        let input = CandlePageInput {
            symbol: ticket.symbol.clone(),
            broker_id: ticket.broker_id,
            is_overseas: overseas,
            interval: "D".into(),
            count: Some(200),
            exchange: exchanges[exchange].clone(),
            cursor: cursor.clone(),
            expected_profile_id: ticket.profile_id.clone(),
            expected_broker_account_id: ticket.broker_account_id.clone(),
        };
        let page = match fetch(input).await {
            Ok(page) => page,
            Err(error) if error.code == "CANDLE_PAGE_FAILED" => {
                if identity.is_none() && exchange + 1 < exchanges.len() && attempt + 1 < MAX_PAGES {
                    exchange += 1;
                    continue;
                }
                if identity.is_none() {
                    return Err(error.message);
                }
                end_reason = format!("일부 페이지 조회 실패: {}", error.message);
                break;
            }
            Err(error) => return Err(format!("{}: {}", error.code, error.message)),
        };
        let as_of = DateTime::parse_from_rfc3339(&page.as_of)
            .map_err(|_| "완료 일봉 조회 기준 시각 형식 오류")?
            .with_timezone(&Utc);
        let next_identity = (
            page.scope_key.clone(),
            page.as_of.clone(),
            page.source.clone(),
        );
        if page.broker_id != ticket.broker_id
            || page.profile_id != ticket.profile_id
            || page.symbol != ticket.symbol
            || page.interval != "D"
            || page.exchange != exchanges[exchange]
            || page.scope_key.is_empty()
            || page.timestamp_semantics != "tradingDate"
            || page.completion_policy != "strictlyBeforeCurrentMarketPeriod"
            || candle_market_date(
                as_of,
                if overseas {
                    BrokerMarket::Us
                } else {
                    BrokerMarket::Kr
                },
            ) != ticket.market_date
            || identity
                .as_ref()
                .is_some_and(|value| *value != next_identity)
        {
            return Err("완료 일봉의 계좌·시장 날짜·페이지 기준이 변경되었습니다.".into());
        }
        if page.candles.len() > 200 {
            return Err("완료 일봉 페이지가 200봉 상한을 초과했습니다.".into());
        }
        identity.get_or_insert(next_identity);
        let previous_len = retained.len();
        let had_rows = !page.candles.is_empty();
        for bar in page.candles {
            let Ok(day) = NaiveDate::parse_from_str(&bar.date, "%Y%m%d") else {
                continue;
            };
            // No sampled tick or current-day bar can enter the rolling high window.
            if day >= ticket.market_date {
                continue;
            }
            if let Some(ohlc) = completed_bar(&bar, overseas) {
                retained.entry(day).or_insert(ohlc);
                if retained.len() > MAX_RETAINED {
                    retained.pop_first();
                }
            }
        }
        if retained.len() >= target {
            end_reason = "요구 완료봉 확보".into();
            break;
        }
        let Some(next) = page.next_cursor else {
            end_reason = format!(
                "provider 자료 끝: {}",
                page.end_reason.as_deref().unwrap_or("providerExhausted")
            );
            break;
        };
        if Some(&next) == cursor.as_ref() || !seen.insert(next.clone()) {
            end_reason = "페이지 커서 반복".into();
            break;
        }
        if had_rows && retained.len() == previous_len {
            end_reason = "추가 유효 완료봉 없음".into();
            break;
        }
        cursor = Some(next);
    }
    let last_completed_date = retained.last_key_value().map(|(date, _)| date.to_string());
    let skip = retained.len().saturating_sub(target);
    Ok(CollectedDailyHistory {
        candles: retained.into_values().skip(skip).collect(),
        source: identity
            .map(|value| match &exchanges[exchange] {
                Some(exchange) => format!("{} @ {exchange}", value.2),
                None => value.2,
            })
            .unwrap_or_default(),
        last_completed_date,
        end_reason,
    })
}

#[cfg(test)]
mod tests;
