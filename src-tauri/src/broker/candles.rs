//! Provider-specific cursors remain typed inside the broker boundary.

use chrono::{DateTime, Days, NaiveDate};
use serde::{Deserialize, Serialize};

use super::{BrokerCandle, BrokerMarket, BrokerSymbol};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "camelCase")]
pub enum CandleCursor {
    Kis { before: String },
    Toss { before: String },
}

#[derive(Debug, Clone)]
pub struct CandlePageRequest {
    pub symbol: BrokerSymbol,
    pub market: BrokerMarket,
    pub interval: String,
    pub count: u16,
    pub exchange: Option<String>,
    pub cursor: Option<CandleCursor>,
}

impl CandlePageRequest {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            (1..=200).contains(&self.count),
            "candle page count must be 1..200"
        );
        anyhow::ensure!(
            !self.symbol.0.is_empty()
                && self.symbol.0.len() <= 32
                && self
                    .symbol
                    .0
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'),
            "invalid candle symbol"
        );
        anyhow::ensure!(
            matches!(self.interval.as_str(), "1m" | "D" | "W" | "M"),
            "invalid candle interval"
        );
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BrokerCandlePage {
    pub candles: Vec<BrokerCandle>,
    pub next_cursor: Option<CandleCursor>,
    /// None while another page is available. This does not promise retention.
    pub end_reason: Option<String>,
    pub timestamp_semantics: &'static str,
}

/// KIS daily/weekly/monthly APIs use a date anchor, never a Toss timestamp.
pub fn kis_cursor_date(
    cursor: Option<&CandleCursor>,
    fallback: NaiveDate,
) -> anyhow::Result<NaiveDate> {
    match cursor {
        None => Ok(fallback),
        Some(CandleCursor::Kis { before }) if before.len() == 8 => {
            Ok(NaiveDate::parse_from_str(before, "%Y%m%d")?)
        }
        _ => anyhow::bail!("KIS candle cursor must contain a YYYYMMDD date"),
    }
}

pub fn previous_kis_cursor(candles: &[BrokerCandle], anchor: NaiveDate) -> Option<CandleCursor> {
    let oldest = candles
        .iter()
        .filter_map(|bar| NaiveDate::parse_from_str(&bar.date, "%Y%m%d").ok())
        .min()?;
    let previous = oldest.checked_sub_days(Days::new(1))?;
    (previous < anchor).then(|| CandleCursor::Kis {
        before: previous.format("%Y%m%d").to_string(),
    })
}

/// Toss documents inclusive `before`; forwarding `nextBefore` unchanged lets
/// consumers deduplicate the overlap instead of inventing a timestamp offset.
pub fn toss_cursor_before(cursor: Option<&CandleCursor>) -> anyhow::Result<Option<&str>> {
    match cursor {
        None => Ok(None),
        Some(CandleCursor::Toss { before }) if before.len() <= 64 => {
            DateTime::parse_from_rfc3339(before)?;
            Ok(Some(before))
        }
        _ => anyhow::bail!("Toss candle cursor must contain an ISO8601 timestamp"),
    }
}

pub fn next_toss_cursor(
    before: Option<&str>,
    next: Option<String>,
) -> anyhow::Result<(Option<CandleCursor>, Option<String>)> {
    let Some(next) = next else {
        return Ok((None, Some("providerExhausted".into())));
    };
    anyhow::ensure!(next.len() <= 64, "invalid Toss nextBefore length");
    let next_time = DateTime::parse_from_rfc3339(&next)?;
    if let Some(before) = before {
        if next_time >= DateTime::parse_from_rfc3339(before)? {
            return Ok((None, Some("cursorDidNotAdvance".into())));
        }
    }
    Ok((Some(CandleCursor::Toss { before: next }), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_cursor_types_cannot_be_mixed_and_invalid_dates_are_rejected() {
        let day = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
        let toss = CandleCursor::Toss {
            before: "2026-10-16T16:00:00-04:00".into(),
        };
        assert!(kis_cursor_date(Some(&toss), day).is_err());
        let kis = CandleCursor::Kis {
            before: "20261016".into(),
        };
        assert!(toss_cursor_before(Some(&kis)).is_err());
        assert!(kis_cursor_date(
            Some(&CandleCursor::Kis {
                before: "20260230".into()
            }),
            day
        )
        .is_err());
    }

    #[test]
    fn toss_inclusive_overlap_is_preserved_but_repeated_and_forward_cursors_stop() {
        let cursor = "2026-10-16T16:00:00-04:00";
        let (next, reason) = next_toss_cursor(None, Some(cursor.into())).unwrap();
        assert_eq!(toss_cursor_before(next.as_ref()).unwrap(), Some(cursor));
        assert!(reason.is_none());
        for value in [cursor, "2026-10-16T21:00:00+00:00"] {
            let (next, reason) = next_toss_cursor(Some(cursor), Some(value.into())).unwrap();
            assert!(next.is_none());
            assert_eq!(reason.as_deref(), Some("cursorDidNotAdvance"));
        }
        let (next, reason) =
            next_toss_cursor(Some(cursor), Some("2026-10-15T16:00:00-04:00".into())).unwrap();
        assert!(next.is_some());
        assert!(reason.is_none());
    }

    #[tokio::test]
    async fn toss_adapter_forwards_inclusive_cursor_through_real_http_encoding() {
        use crate::broker::{BrokerAdapter, TossBrokerAdapter};
        use axum::{
            extract::{OriginalUri, State},
            routing::{get, post},
            Json, Router,
        };
        use std::sync::Arc;
        use tokio::sync::Mutex;

        async fn candles(
            State(requests): State<Arc<Mutex<Vec<String>>>>,
            OriginalUri(uri): OriginalUri,
        ) -> Json<serde_json::Value> {
            let mut requests = requests.lock().await;
            requests.push(uri.to_string());
            let old = if requests.len() == 1 {
                "2026-10-14T00:00:00+09:00"
            } else {
                "2026-10-13T00:00:00+09:00"
            };
            let latest = if requests.len() == 1 {
                "2026-10-15T00:00:00+09:00"
            } else {
                "2026-10-14T00:00:00+09:00"
            };
            let bar = |timestamp| serde_json::json!({"timestamp":timestamp,"openPrice":"100","highPrice":"120","lowPrice":"90","closePrice":"110","volume":"10","currency":"KRW"});
            Json(
                serde_json::json!({"result":{"candles":[bar(latest),bar(old)],"nextBefore":"2026-10-14T00:00:00+09:00"}}),
            )
        }
        let requests = Arc::new(Mutex::new(Vec::<String>::new()));
        let app = Router::new()
            .route("/oauth2/token", post(|| async { Json(serde_json::json!({"access_token":"fixture-token","token_type":"Bearer","expires_in":3600})) }))
            .route("/api/v1/candles", get(candles))
            .with_state(requests.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let adapter = TossBrokerAdapter::with_credentials(
            format!("http://{address}"),
            "fixture-key",
            "fixture-secret",
            Some("fixture-account"),
        );
        let mut request = CandlePageRequest {
            symbol: BrokerSymbol("005930".into()),
            market: BrokerMarket::Kr,
            interval: "D".into(),
            count: 2,
            exchange: None,
            cursor: None,
        };
        let first = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            adapter.get_candle_page(&request),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(first.candles.len(), 2);
        assert!(first.end_reason.is_none());
        request.cursor = first.next_cursor;
        let second = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            adapter.get_candle_page(&request),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(second.candles[0].date, first.candles[1].date); // inclusive overlap is preserved
        assert!(second.next_cursor.is_none());
        assert_eq!(second.end_reason.as_deref(), Some("cursorDidNotAdvance"));
        let requests = requests.lock().await;
        assert_eq!(requests.len(), 2);
        let url = reqwest::Url::parse(&format!("http://localhost{}", requests[1])).unwrap();
        let query = url
            .query_pairs()
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(query["before"], "2026-10-14T00:00:00+09:00");
        assert_eq!(query["interval"], "1d");
        assert_eq!(query["count"], "2");
        assert_eq!(query["adjusted"], "true");
        assert!(requests[1].contains("%2B09%3A00"));
        server.abort();
    }
}
