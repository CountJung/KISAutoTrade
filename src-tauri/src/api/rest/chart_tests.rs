use super::*;
use crate::api::rest::{kis_http_client, KIS_MAX_RESPONSE_BYTES};
use crate::api::token::TokenManager;
use crate::config::{AccountProfile, AppConfig, DiscordConfig};
use axum::http::{Response as HttpResponse, StatusCode};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[test]
fn client_scope_matches_every_account_credential_and_mode() {
    let profile = AccountProfile::new(
        "chart fixture".into(),
        true,
        "chart-fixture-key".into(),
        "chart-fixture-secret".into(),
        "1234567801".into(),
    );
    let config = AppConfig::from_profile(&profile, &DiscordConfig::default());
    let client = KisRestClient::new(
        "https://example.invalid".into(),
        profile.app_key.clone(),
        profile.app_secret.clone(),
        profile.account_no.clone(),
        profile.is_paper_trading,
        std::sync::Arc::new(tokio::sync::RwLock::new(TokenManager::new(config))),
    );
    assert!(client.matches_account(
        "chart-fixture-key",
        "chart-fixture-secret",
        "1234567801",
        true
    ));
    for (key, secret, account, paper) in [
        ("other-key", "chart-fixture-secret", "1234567801", true),
        ("chart-fixture-key", "other-secret", "1234567801", true),
        (
            "chart-fixture-key",
            "chart-fixture-secret",
            "1234567802",
            true,
        ),
        (
            "chart-fixture-key",
            "chart-fixture-secret",
            "1234567801",
            false,
        ),
    ] {
        assert!(!client.matches_account(key, secret, account, paper));
    }
}

fn synthetic_auth() -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in [
        ("authorization", "Bearer chart-fixture-token"),
        ("appkey", "chart-fixture-key"),
        ("appsecret", "chart-fixture-secret"),
        ("custtype", "P"),
        ("content-type", "application/json; charset=utf-8"),
        ("tr_id", "fixture-header-must-be-replaced"),
    ] {
        headers.insert(name, value.parse().unwrap());
    }
    headers
}

fn assert_auth(request: &reqwest::Request) {
    assert_eq!(request.method(), reqwest::Method::GET);
    assert_eq!(
        request.headers()["authorization"],
        "Bearer chart-fixture-token"
    );
    assert_eq!(request.headers()["appkey"], "chart-fixture-key");
    assert_eq!(request.headers()["appsecret"], "chart-fixture-secret");
    assert_eq!(request.headers()["custtype"], "P");
    assert_eq!(request.headers().get_all("tr_id").iter().count(), 1);
    assert!(request.body().is_none());
}

#[test]
fn domestic_request_keeps_date_bounds_period_adjustment_and_query_order() {
    let client = kis_http_client();
    for period in ["D", "W", "M"] {
        let request = ChartRequest::domestic("0005A0", period, "20260102", "20261016")
            .build(&client, "https://example.invalid:9443", synthetic_auth())
            .build()
            .unwrap();
        assert_auth(&request);
        assert_eq!(request.headers()["tr_id"], "FHKST03010100");
        assert_eq!(
            request.url().path(),
            "/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice"
        );
        assert_eq!(
            request.url().query().unwrap(),
            format!(
                "FID_COND_MRKT_DIV_CODE=J&FID_INPUT_ISCD=0005A0&FID_INPUT_DATE_1=20260102&FID_INPUT_DATE_2=20261016&FID_PERIOD_DIV_CODE={period}&FID_ORG_ADJ_PRC=0"
            )
        );
    }
}

#[test]
fn overseas_dailyprice_request_uses_its_own_tr_id_and_period_codes() {
    let client = kis_http_client();
    for (period, gubn) in [("D", "0"), ("W", "1"), ("M", "2"), ("", "0")] {
        for base_date in ["20261016", ""] {
            let request = ChartRequest::overseas("BRK/B", "NYS", period, base_date)
                .build(&client, "https://example.invalid:29443", synthetic_auth())
                .build()
                .unwrap();
            assert_auth(&request);
            assert_eq!(request.headers()["tr_id"], "HHDFS76240000");
            assert_eq!(
                request.url().path(),
                "/uapi/overseas-price/v1/quotations/dailyprice"
            );
            assert_eq!(
                request.url().query().unwrap(),
                format!("AUTH=&EXCD=NYS&SYMB=BRK%2FB&GUBN={gubn}&BYMD={base_date}&MODP=1")
            );
        }
    }
}

fn response(status: StatusCode, body: impl Into<reqwest::Body>) -> Response {
    HttpResponse::builder()
        .status(status)
        .body(body)
        .unwrap()
        .into()
}

async fn parse_response(overseas: bool, response: Response) -> Result<Vec<ChartCandle>> {
    if overseas {
        parse_overseas_response(response).await
    } else {
        parse_domestic_response(response).await
    }
}

fn candle_values(
    candles: Vec<ChartCandle>,
) -> Vec<(String, String, String, String, String, String)> {
    candles
        .into_iter()
        .map(|c| (c.date, c.open, c.high, c.low, c.close, c.volume))
        .collect()
}

#[tokio::test]
async fn domestic_latest_first_fixture_reverses_and_maps_ohlcv() {
    let body = json!({
        "rt_cd": "0", "msg1": "정상처리",
        "output2": [
            {"stck_bsop_date":"20261016", "stck_oprc":"100", "stck_hgpr":"120", "stck_lwpr":"90", "stck_clpr":"110", "acml_vol":"999"},
            {"stck_bsop_date":""},
            {"stck_bsop_date":"20261015", "stck_oprc":"80", "stck_hgpr":"105", "stck_lwpr":"70", "stck_clpr":"100", "acml_vol":"888"}
        ]
    });
    let candles = parse_domestic_response(response(StatusCode::OK, body.to_string()))
        .await
        .unwrap();
    assert_eq!(
        candle_values(candles),
        [
            ("20261015", "80", "105", "70", "100", "888"),
            ("20261016", "100", "120", "90", "110", "999")
        ]
        .map(|(d, o, h, l, c, v)| (
            d.into(),
            o.into(),
            h.into(),
            l.into(),
            c.into(),
            v.into()
        ))
    );
}

#[tokio::test]
async fn overseas_latest_first_fixture_reverses_and_maps_decimal_ohlcv() {
    let body = json!({
        "rt_cd": "0", "msg1": "정상처리",
        "output2": [
            {"xymd":"20261016", "open":"10.10", "high":"12.20", "low":"9.30", "clos":"11.40", "tvol":"999"},
            {},
            {"xymd":"20261015", "open":"8.10", "high":"10.20", "low":"7.30", "clos":"10.40", "tvol":"888"}
        ]
    });
    let candles = parse_overseas_response(response(StatusCode::OK, body.to_string()))
        .await
        .unwrap();
    assert_eq!(
        candle_values(candles),
        [
            ("20261015", "8.10", "10.20", "7.30", "10.40", "888"),
            ("20261016", "10.10", "12.20", "9.30", "11.40", "999")
        ]
        .map(|(d, o, h, l, c, v)| (
            d.into(),
            o.into(),
            h.into(),
            l.into(),
            c.into(),
            v.into()
        ))
    );
}

#[tokio::test]
async fn chart_response_preserves_http_and_provider_errors() {
    for overseas in [false, true] {
        let error = parse_response(overseas, response(StatusCode::TOO_MANY_REQUESTS, "quota"))
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("HTTP 429"), "{error}");
        assert!(error.contains("quota"), "{error}");
        let body = json!({"rt_cd":"1", "msg1":"조회 기준일 오류"});
        let error = parse_response(overseas, response(StatusCode::OK, body.to_string()))
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("조회 기준일 오류"), "{error}");
    }
}

#[tokio::test]
async fn chart_response_handles_empty_output_and_malformed_multibyte_body() {
    for overseas in [false, true] {
        for output in [None, Some(json!(null)), Some(json!([]))] {
            let mut body = json!({"rt_cd":"0", "msg1":"정상처리"});
            if let Some(output) = output {
                body["output2"] = output;
            }
            assert!(
                parse_response(overseas, response(StatusCode::OK, body.to_string()))
                    .await
                    .unwrap()
                    .is_empty()
            );
        }
        // The old byte-offset preview could panic at a UTF-8 character boundary.
        let error = parse_response(
            overseas,
            response(StatusCode::OK, format!("x{}", "가".repeat(120))),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(error.contains("JSON 파싱 실패"), "{error}");
        let error = parse_response(overseas, response(StatusCode::OK, vec![0xff]))
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("UTF-8 파싱 실패"), "{error}");
    }
}

#[tokio::test]
async fn chart_response_propagates_announced_and_observed_body_caps() {
    for overseas in [false, true] {
        for status in [StatusCode::OK, StatusCode::BAD_GATEWAY] {
            let announced = oversized_local_response(status, true).await;
            assert_eq!(
                announced.content_length(),
                Some((KIS_MAX_RESPONSE_BYTES + 1) as u64)
            );
            let error = parse_response(overseas, announced)
                .await
                .unwrap_err()
                .to_string();
            assert!(error.contains("상한을 초과"), "{error}");
            let observed = oversized_local_response(status, false).await;
            assert_eq!(observed.content_length(), None);
            let error = parse_response(overseas, observed)
                .await
                .unwrap_err()
                .to_string();
            assert!(error.contains("상한을 초과"), "{error}");
        }
    }
}

#[tokio::test]
async fn chart_response_accepts_body_at_exact_size_limit() {
    for overseas in [false, true] {
        let mut body = r#"{"rt_cd":"0","msg1":"ok","output2":[]}"#.to_string();
        body.push_str(&" ".repeat(KIS_MAX_RESPONSE_BYTES - body.len()));
        assert!(parse_response(overseas, response(StatusCode::OK, body))
            .await
            .unwrap()
            .is_empty());
    }
}

/// A real HTTP response is needed: Response::from ignores a synthetic length header.
/// All traffic stays on loopback; no provider or token endpoint is contacted.
async fn oversized_local_response(status: StatusCode, announced: bool) -> Response {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 4096];
        let mut received = 0;
        loop {
            let count = stream.read(&mut request[received..]).await.unwrap();
            if count == 0 {
                return;
            }
            received += count;
            if request[..received]
                .windows(4)
                .any(|part| part == b"\r\n\r\n")
            {
                break;
            }
            if received == request.len() {
                return;
            }
        }
        let size = KIS_MAX_RESPONSE_BYTES + 1;
        let framing = if announced {
            format!("Content-Length: {size}\r\n")
        } else {
            "Transfer-Encoding: chunked\r\n".to_string()
        };
        let header = format!("HTTP/1.1 {}\r\n{framing}Connection: close\r\n\r\n", status);
        // A client that rejects the advertised length may close immediately.
        if stream.write_all(header.as_bytes()).await.is_err() || announced {
            return;
        }
        let chunk = format!("{size:x}\r\n");
        if stream.write_all(chunk.as_bytes()).await.is_ok()
            && stream.write_all(&vec![b' '; size]).await.is_ok()
        {
            let _ = stream.write_all(b"\r\n0\r\n\r\n").await;
        }
    });
    kis_http_client()
        .get(format!("http://{address}/chart-fixture"))
        .send()
        .await
        .unwrap()
}
