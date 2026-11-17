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
                Ok(RawBrokerChartPage {
                    rows,
                    next_cursor,
                    response_source: None,
                })
            }
            Self::Toss(client) => {
                let initial = toss_before(end_date)?;
                let before = cursor.unwrap_or(&initial);
                let bound = DateTime::parse_from_rfc3339(before)
                    .map_err(|_| "BROKER_CHART_INVALID_CURSOR")?;
                let (page, source) = client
                    .get_candles_with_source(
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
                    response_source: source.map(Into::into),
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
    // Production quote provider -> series -> serialized receipt, with synthetic wire bytes only.
    #[tokio::test]
    async fn broker_chart_toss_daily_raw_source_survives_series() {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        };
        // Independent Python hashlib oracles; exact bytes are retained in the task cache.
        const PAGES: [&str; 2] = [
            r#"{"result":{"candles":[{"timestamp":"2025-10-03T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-06T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-07T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-08T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-09T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-10T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-13T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-14T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-15T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-16T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-17T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-20T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-21T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-22T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-23T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-24T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-27T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-28T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-29T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-30T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-31T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"}],"nextBefore":"2025-10-03T00:00:00-04:00"}}"#,
            r#"{"result":{"candles":[{"timestamp":"2025-09-04T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-05T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-08T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-09T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-10T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-11T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-12T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-15T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-16T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-17T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-18T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-19T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-22T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-23T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-24T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-25T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-26T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-29T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-09-30T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-01T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"},{"timestamp":"2025-10-02T00:00:00-04:00","openPrice":"100","highPrice":"102","lowPrice":"99","closePrice":"101","volume":"12345","currency":"USD"}],"nextBefore":null}}"#,
        ];
        const EXPECTED: [[[(&str, u64); 2]; 12]; 2] = [
            [
                [("f30b36690d1ed30530dce528b98f506e0e6255952c19ff7240a4f2848cf5462a", 3152), ("4a85794790b0667a6f9df676fe066228865a29a1d92e8748a8c422bb3bd7f8ed", 3129)],
                [("66035765ba777a45efefdef4e3ceaeacec8a0ae9b7acd63fc28f9f65b27ba07d", 3153), ("3ad8a4ecbee268c0149eb6a520c7b302d71e5102690b552a095a31a991f257cf", 3130)],
                [("bdeeb61898a6b2ab6638e549651c4ce529dbf26b5d5ab4ba0b7bda3bcf814a92", 3154), ("7bf63081f649cb7624ad9cffe2d0bcfbd12bf026d183d02c6ba742d6d31553c8", 3131)],
                [("3ff46f81850420ede0d7505f68856fdeaba7b4887e31433a8a2846b6cb332bd2", 3155), ("aeb6885dac858474a4ce05581b3ec29a3ca58d523bf3faeea9f9f5231ec6c015", 3132)],
                [("cf16d59593dd2fb5ceb6724eedb96358bcbebdbd3294a41010b1d8f65b2cb810", 3156), ("9ff65b1b85c2a8ae51eca375608ef00ea2d2f5d914ab8601308a5482a7a76999", 3133)],
                [("9bb405a17b8b9ed3d054601c4c1c8dec9b80508abeed206d51171c6b3f8b17d8", 3157), ("a24fc636f370ce68a40c029d217816ff74e33a41654616a4e4feb06c3c794a10", 3134)],
                [("c61e88ea973c9364954d1ccb4da116dd9a0c12c1cca9584bbc28f081d9e872d4", 3158), ("14545cef48586ae725c9726e8dacaf81fa30e619a02356f056704b10258e68fa", 3135)],
                [("60881d35c2d36dac622c45de47697ab29d893c43857b837270d983af6f95d244", 3159), ("6433e68734d4502772c70665e61ecf0c3cd973b94f9f2db898c9d27b74a20f3e", 3136)],
                [("0a88e97ae55a8ed1f2ead0c676b6b798c9088d3c16bb3b62a6a6d2f2f15e06af", 3160), ("82fb5bf45ebae177b2556c2a74947be52260c6b2e0c327da54a37669f1757cdb", 3137)],
                [("324aca3f60d7980060217cb4b6e42ea791e0eaf96754aa203a6ec22ee6eb270a", 3161), ("0782e88343c8026bf168c305497b547b30347de3621efc03ce157693934385e6", 3138)],
                [("507dafe59f303dcbce704ab7ba8d41af73dbaa6ab186cea866314bf7e3912817", 3162), ("0f8596daaa2ba6a5d2ee5f4fdebcdedd3458882c83d8b9b62da29bc8e2fce04c", 3139)],
                [("dbfa3b2b20ebe1a7ca5f9456fa611ca76b448575d4fd41c2739c585d03fb7adb", 3163), ("a3f21a445bdda4557da38c4e0caa524f037dbb42383a5bc931eb2a083fcb4580", 3140)],
            ],
            [
                [("7ffc071009d0c22b0d1160e2e7b9de1da3e5ae2b8b762fdc0da1ef48526faa95", 3155), ("45042e91c484d4ad42d2835ef4ad309a9597c3ee75d60ed6fce59fc1e82205bb", 3132)],
                [("bd05e0158fa00d6ca8d8c059e6e15ac99480c1e8b5ba35ee951f872406751138", 3156), ("805b9590ca87ccad2a75f26e126565d47df854d4055d4ad7a7360a53b2e4ccde", 3133)],
                [("7a19bd299409b56f95878b31353a9f9915ea32302c155db50ea45155fe79e1c9", 3157), ("852fa4849109c3c8a70caae36f46b1d65faa20dc521b6a6c3faec0beb8d90148", 3134)],
                [("bda142336ba2f15556e23fc9cb9e4fe1e5abdea9c45ed1e1107bef0cf971e317", 3158), ("7dc24512c094704e52f3344c9d55495ac86cd3f9b2d092247f128e1148821004", 3135)],
                [("dbd9d14d431fb00fa67f369d8e7b11cd1043ab7d82681391ebcd3b4283eb25b3", 3159), ("26c0bb5f85ee1cc3dbcd5e333f979ddf7a9519cee1655e4b67f29ad85945ed8a", 3136)],
                [("fc5ebb5746fd386212aada71f1dfe9c4ca3d091c5c2d0ee424f66737eeb2137c", 3160), ("55dd66177a5c7b56b952d4c3096e70a3944ca22f0e258f28fd8d0b08b9efc80c", 3137)],
                [("e63ed8f68bdc666046fdfb406610cb986bb012851cb5a9b8d5d7f1b1cb507817", 3161), ("5cca3ab18790e1afcf45ee613e2ee7c439eac3383abcb49feaf95f5a472f15b5", 3138)],
                [("27bacc5d105fde98adf4ba8add5fd589067bcba9555cdddf9c40d749375aee3f", 3162), ("be9a5311dc5d9473e6bb6b3acbcac51077a42b25794e53f8f7bd0ae21be55e01", 3139)],
                [("f322dc8e1a9fdb5c3bf1d7aec8c39f86f59960402aac910e9a1aaccecf2c31e1", 3163), ("0685afcb0dc7c82af5a735100cd9c1ab1f6a454a948231c909cba51fba23dadf", 3140)],
                [("34e572ac12a50376f3029391fd25885b3359724771f707555ab63555022c90b1", 3164), ("249590458f170e24cbece3224608a92b8547de419eae2b32de5134a3e02a884a", 3141)],
                [("c3edf58c3a5b1c63753b63cdbb3e113b75c38ce9ad568e042e3af59eaad0e98e", 3165), ("76bea33cedbeecc00392362ec4cc607e00520820ed5608cc8d18cf20ad02a988", 3142)],
                [("043aba57edf3e79f1dcf1195775650b9934f99d450d7b9960bcb2843493626ac", 3166), ("b827e64659acb51ee334d2d20593f0d381a882739bfef91d3dd0390515167692", 3143)],
            ],
        ];
        let expected_requests: BTreeSet<_> = ALL_SYMBOLS.iter()
            .flat_map(|s| (1..=2).map(move |p| (s.to_string(), p))).collect();
        let mut observations = Vec::new();
        for (case, expected_sources) in EXPECTED.iter().enumerate() {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("synthetic loopback bind");
            let base = format!("http://{}", listener.local_addr().unwrap());
            let server = tokio::spawn(async move {
                let token =
                    r#"{"access_token":"synthetic-only","token_type":"Bearer","expires_in":3600}"#
                        .to_string();
                let mut received_requests = BTreeSet::new();
                for request_index in 0..25 {
                    let (mut socket, _) =
                        tokio::time::timeout(std::time::Duration::from_secs(10), listener.accept())
                            .await
                            .unwrap()
                            .unwrap();
                    // Retain only the request line. Discard headers and body without formatting values.
                    let mut request_line = Vec::new();
                    loop {
                        let byte = socket.read_u8().await.unwrap();
                        request_line.push(byte);
                        assert!(request_line.len() < 4096);
                        if request_line.ends_with(b"\r\n") { break; }
                    }
                    let mut content_length = 0;
                    loop {
                        let mut header = Vec::new();
                        while !header.ends_with(b"\r\n") {
                            header.push(socket.read_u8().await.unwrap());
                            assert!(header.len() < 65536);
                        }
                        if header == b"\r\n" { break; }
                        if let Some((key, value)) = std::str::from_utf8(&header).unwrap().split_once(':') {
                            if key.eq_ignore_ascii_case("content-length") {
                                content_length = value.trim().parse::<usize>().unwrap();
                            }
                        }
                    }
                    assert!(content_length < 65536);
                    let mut discarded = vec![0; content_length];
                    socket.read_exact(&mut discarded).await.unwrap();
                    let payload = if request_index == 0 {
                        token.clone()
                    } else {
                        let line = std::str::from_utf8(&request_line).unwrap();
                        let parts: Vec<_> = line.split_whitespace().collect();
                        assert_eq!(parts.len(), 3);
                        assert_eq!(parts[0], "GET");
                        let url = reqwest::Url::parse(&format!("http://localhost{}", parts[1])).unwrap();
                        assert_eq!(url.path(), "/api/v1/candles");
                        let query: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
                        assert_eq!(query.len(), 5);
                        assert_eq!(query.get("interval").map(String::as_str), Some("1d"));
                        assert_eq!(query.get("count").map(String::as_str), Some("200"));
                        assert_eq!(query.get("adjusted").map(String::as_str), Some("false"));
                        let symbol = query.get("symbol").unwrap();
                        let index = ALL_SYMBOLS.iter().position(|s| *s == symbol).unwrap();
                        let page = match query.get("before").map(String::as_str) {
                            Some("2025-10-31T23:59:59-04:00") => 1,
                            Some("2025-10-03T00:00:00-04:00") => 2,
                            _ => panic!("unexpected synthetic quote cursor"),
                        };
                        assert!(received_requests.insert((symbol.clone(), page)));
                        let base = PAGES[page - 1];
                        let body = if case == 0 { base.to_string() } else { format!(" \n{base}\n") };
                        format!("{}{body}", " ".repeat(index))
                    };
                    let reply = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", payload.len(), payload);
                    socket.write_all(reply.as_bytes()).await.unwrap();
                }
                received_requests
            });
            let client = BrokerChartClient::Toss(Arc::new(TossOpenApiClient::new(
                base,
                "synthetic-client",
                "synthetic-secret",
                None::<String>,
            )));
            let started = chrono::Utc::now();
            let collection = collect_broker_chart(
                &client,
                BrokerChartBroker::Toss,
                20,
                "2025-10-31",
                "2026-01-01T12:00:00Z",
                &[],
            )
            .await
            .unwrap();
            assert_eq!(server.await.unwrap(), expected_requests);
            let finished = chrono::Utc::now();
            assert_eq!(collection.input.history.len(), 22);
            assert_eq!(collection.input.evaluation.len(), 20);
            let exported = serde_json::to_value(&collection.receipts).unwrap();
            let receipts = exported.as_array().unwrap();
            assert_eq!(receipts.len(), 24);
            let mut receipt_keys = BTreeSet::new();
            for receipt in receipts {
                let symbol = receipt["symbol"].as_str().unwrap();
                let page = receipt["page"].as_u64().unwrap() as usize;
                assert!((1..=2).contains(&page));
                assert!(receipt_keys.insert((symbol.to_string(), page)));
                let index = ALL_SYMBOLS.iter().position(|s| *s == symbol).unwrap();
                let (expected_sha, expected_length) = expected_sources[index][page - 1];
                let source = &receipt["responseSource"];
                assert_eq!(source.as_object().unwrap().len(), 3);
                assert_eq!(source["sha256"], expected_sha, "source binding: {symbol} page {page}");
                assert_eq!(source["byteLength"], expected_length, "source length: {symbol} page {page}");
                let received = DateTime::parse_from_rfc3339(source["receivedAt"].as_str().unwrap()).unwrap();
                assert_eq!(received.offset().local_minus_utc(), 0);
                assert!(received >= started && received <= finished);
            }
            assert_eq!(receipt_keys, expected_requests);
            observations.push(collection);
        }
        assert_eq!(
            hash(&observations[0].input).unwrap(),
            hash(&observations[1].input).unwrap()
        );
        for (a, b) in observations[0]
            .receipts
            .iter()
            .zip(&observations[1].receipts)
        {
            assert_eq!(a.rows_hash, b.rows_hash);
        }
        assert_ne!(
            observations[0].collection_hash,
            observations[1].collection_hash
        );
        let mut reports = Vec::new();
        for mut collection in observations {
            collection.input.acknowledged_price_only = true;
            let report = preview_broker_chart_price_approximation(&collection.input).unwrap();
            assert!(!report.economic_return_verified && !report.dividend_return_included);
            reports.push(serde_json::to_value(report).unwrap());
        }
        assert_eq!(reports[0], reports[1]);
    }
}
