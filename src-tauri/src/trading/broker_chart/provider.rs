//! Quote-only broker transport. No balance/account/order endpoints.
use super::*;
use crate::{
    api::rest::{ChartCandle, KisRestClient},
    broker::{toss::TossOpenApiClient, BrokerSymbol},
};
use chrono::{DateTime, TimeZone, Timelike};
use std::sync::Arc;

pub enum BrokerChartClient {
    Kis(Arc<KisRestClient>),
    Toss(Arc<TossOpenApiClient>),
}

fn number(value: &str) -> Result<f64, String> {
    if value.is_empty() || value.len() > 64 {
        return Err("BROKER_CHART_INVALID_NUMBER".into());
    }
    let n = value
        .parse::<f64>()
        .map_err(|_| "BROKER_CHART_INVALID_NUMBER")?;
    if !n.is_finite() {
        return Err("BROKER_CHART_INVALID_NUMBER".into());
    }
    Ok(n)
}

fn row(symbol: &str, date: String, bar: ChartCandle) -> Result<BrokerChartDay, String> {
    session(&date)?;
    let price = BrokerChartPrice {
        open: number(&bar.open)?,
        high: number(&bar.high)?,
        low: number(&bar.low)?,
        close: number(&bar.close)?,
        volume: number(&bar.volume)?,
        currency: BrokerChartCurrency::Usd,
    };
    if !valid_price(&price) {
        return Err("BROKER_CHART_INVALID_OHLCV".into());
    }
    Ok(BrokerChartDay {
        date,
        prices: BTreeMap::from([(symbol.into(), price)]),
    })
}

pub fn toss_daily_date(value: &str) -> Result<String, String> {
    let stamp =
        DateTime::parse_from_rfc3339(value).map_err(|_| "BROKER_CHART_INVALID_TIMESTAMP")?;
    let ny = stamp.with_timezone(&chrono_tz::America::New_York);
    if ny.hour() != 0 || ny.minute() != 0 || ny.second() != 0 {
        return Err("BROKER_CHART_NOT_NY_DAILY_LABEL".into());
    }
    Ok(ny.date_naive().format("%Y-%m-%d").to_string())
}

fn toss_before(end: &str) -> Result<String, String> {
    let date =
        NaiveDate::parse_from_str(end, "%Y-%m-%d").map_err(|_| "BROKER_CHART_INVALID_END")?;
    let stamp = chrono_tz::America::New_York
        .from_local_datetime(
            &date
                .and_hms_opt(23, 59, 59)
                .ok_or("BROKER_CHART_INVALID_END")?,
        )
        .single()
        .ok_or("BROKER_CHART_AMBIGUOUS_DATE")?;
    Ok(stamp.to_rfc3339())
}

impl BrokerChartQuoteSource for BrokerChartClient {
    async fn fetch(
        &self,
        symbol: &str,
        exchange: Option<&str>,
        end_date: &str,
        cursor: Option<&str>,
    ) -> Result<RawBrokerChartPage, String> {
        // Conservative research pacing in addition to the existing shared limiter.
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        match self {
            Self::Kis(client) => {
                let anchor = cursor
                    .map(str::to_string)
                    .unwrap_or_else(|| end_date.replace('-', ""));
                let anchor_date = NaiveDate::parse_from_str(&anchor, "%Y%m%d")
                    .map_err(|_| "BROKER_CHART_INVALID_CURSOR")?;
                let bars = client
                    .get_broker_chart_daily_quotes(
                        symbol,
                        exchange.ok_or("BROKER_CHART_MISSING_EXCHANGE")?,
                        &anchor,
                    )
                    .await
                    .map_err(|e| {
                        let code = e.to_string();
                        if matches!(
                            code.as_str(),
                            "BROKER_CHART_AUTH_FAILED"
                                | "BROKER_CHART_TRANSPORT_FAILED"
                                | "BROKER_CHART_HTTP_FAILED"
                                | "BROKER_CHART_PROVIDER_REJECTED"
                                | "BROKER_CHART_EMPTY_ROWS"
                                | "BROKER_CHART_RATE_LIMITED"
                        ) {
                            code
                        } else {
                            "BROKER_CHART_KIS_RESPONSE_FAILED".into()
                        }
                    })?;
                let mut rows = Vec::with_capacity(bars.len());
                for bar in bars {
                    let date = NaiveDate::parse_from_str(&bar.date, "%Y%m%d")
                        .map_err(|_| "BROKER_CHART_INVALID_DATE")?;
                    if date > anchor_date {
                        return Err("BROKER_CHART_AFTER_ANCHOR".into());
                    }
                    rows.push(row(symbol, date.format("%Y-%m-%d").to_string(), bar)?);
                }
                let next_cursor = rows
                    .iter()
                    .map(|r| r.date.as_str())
                    .min()
                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                    .and_then(|d| d.pred_opt())
                    .map(|d| d.format("%Y%m%d").to_string());
                Ok(RawBrokerChartPage { rows, next_cursor })
            }
            Self::Toss(client) => {
                let initial = toss_before(end_date)?;
                let before = cursor.unwrap_or(&initial);
                let bound = DateTime::parse_from_rfc3339(before)
                    .map_err(|_| "BROKER_CHART_INVALID_CURSOR")?;
                let page = client
                    .get_candles(
                        &BrokerSymbol(symbol.into()),
                        "1d",
                        Some(200),
                        Some(before),
                        Some(false),
                    )
                    .await
                    .map_err(|_| "BROKER_CHART_TOSS_FETCH_FAILED")?;
                if page.candles.len() > 200 {
                    return Err("BROKER_CHART_OVERSIZED_PAGE".into());
                }
                let mut rows = Vec::with_capacity(page.candles.len());
                for bar in page.candles {
                    if bar.currency != "USD" {
                        return Err("BROKER_CHART_CURRENCY_MISMATCH".into());
                    }
                    let stamp = DateTime::parse_from_rfc3339(&bar.timestamp)
                        .map_err(|_| "BROKER_CHART_INVALID_TIMESTAMP")?;
                    if stamp > bound {
                        return Err("BROKER_CHART_AFTER_ANCHOR".into());
                    }
                    let date = toss_daily_date(&bar.timestamp)?;
                    rows.push(row(
                        symbol,
                        date,
                        ChartCandle {
                            date: bar.timestamp,
                            open: bar.open_price,
                            high: bar.high_price,
                            low: bar.low_price,
                            close: bar.close_price,
                            volume: bar.volume,
                        },
                    )?);
                }
                if let Some(next) = &page.next_before {
                    let t = DateTime::parse_from_rfc3339(next)
                        .map_err(|_| "BROKER_CHART_INVALID_CURSOR")?;
                    if t >= bound {
                        return Err("BROKER_CHART_CURSOR_DID_NOT_ADVANCE".into());
                    }
                }
                Ok(RawBrokerChartPage {
                    rows,
                    next_cursor: page.next_before,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broker_chart_toss_kst_daily_labels_preserve_ny_sessions_and_dst() {
        assert_eq!(
            toss_daily_date("2026-10-30T13:00:00+09:00").unwrap(),
            "2026-10-30"
        );
        assert_eq!(
            toss_daily_date("2026-11-02T14:00:00+09:00").unwrap(),
            "2026-11-02"
        );
        assert!(toss_daily_date("2026-11-02T00:00:00Z").is_err());
        assert!(toss_before("2026-10-30").unwrap().ends_with("-04:00"));
        assert!(toss_before("2026-11-02").unwrap().ends_with("-05:00"));
    }
}
