//! Synthetic wire responses only. No operating config, keychain or broker is used.
use super::*;
use crate::broker::toss::error::{definitively_rejected, TossHttpError};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};

const BODY_MARKER: &str = "DO_NOT_LEAK_BODY";
const TOKEN_MARKER: &str = "DO_NOT_LEAK_TOKEN";
const CURSOR_MARKER: &str = "DO_NOT_LEAK_CURSOR";
const ENDPOINT: &str = "/api/v1/candles";
const EMPTY_PAGE: &[u8] = br#"{"result":{"candles":[],"nextBefore":null}}"#;

struct Wire {
    base: String,
    requests: Arc<AsyncMutex<Vec<String>>>,
    accept_encodings: Arc<AsyncMutex<Vec<Option<String>>>>,
    count: Arc<AtomicUsize>,
    task: JoinHandle<()>,
}
impl Drop for Wire {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Wire {
    async fn start(responses: Vec<Vec<u8>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(AsyncMutex::new(Vec::new()));
        let count = Arc::new(AtomicUsize::new(0));
        let seen = requests.clone();
        let accept_encodings = Arc::new(AsyncMutex::new(Vec::new()));
        let encodings = accept_encodings.clone();
        let calls = count.clone();
        let task = tokio::spawn(async move {
            for response in responses {
                let (mut socket, _) =
                    tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
                        .await
                        .unwrap()
                        .unwrap();
                let mut head = Vec::new();
                while !head.ends_with(b"\r\n\r\n") {
                    assert!(head.len() < 65536);
                    head.push(socket.read_u8().await.unwrap());
                }
                let head = String::from_utf8(head).unwrap();
                encodings.lock().await.push(head.lines().find_map(|line| {
                    line.split_once(':')
                        .filter(|(name, _)| name.eq_ignore_ascii_case("accept-encoding"))
                        .map(|(_, value)| value.trim().to_string())
                }));
                let length = head
                    .lines()
                    .find_map(|line| {
                        line.split_once(':')
                            .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                            .map(|(_, value)| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                assert!(length <= 65536);
                let mut body = vec![0; length];
                socket.read_exact(&mut body).await.unwrap();
                seen.lock()
                    .await
                    .push(head.lines().next().unwrap().to_string());
                calls.fetch_add(1, Ordering::SeqCst);
                let _ = socket.write_all(&response).await;
            }
        });
        Self {
            base,
            requests,
            accept_encodings,
            count,
            task,
        }
    }
    async fn client(&self, seed: bool) -> TossOpenApiClient {
        let client = TossOpenApiClient::new(
            &self.base,
            "synthetic-client",
            "synthetic-secret",
            Some("fixture-account"),
        );
        if seed {
            *client.token_state.current_token.lock().await = Some(TossAccessToken {
                access_token: TOKEN_MARKER.into(),
                token_type: "Bearer".into(),
                expires_at: Utc::now() + Duration::hours(1),
            });
        }
        client
    }
}
fn response(status: u16, body: &[u8]) -> Vec<u8> {
    let mut bytes=format!("HTTP/1.1 {status} Synthetic\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
fn chunked(body: &[u8], split: usize) -> Vec<u8> {
    let mut bytes =
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
    for chunk in body.chunks(split) {
        bytes.extend_from_slice(format!("{:X}\r\n", chunk.len()).as_bytes());
        bytes.extend_from_slice(chunk);
        bytes.extend_from_slice(b"\r\n");
    }
    bytes.extend_from_slice(b"0\r\n\r\n");
    bytes
}
async fn candles(client: &TossOpenApiClient) -> anyhow::Result<TossCandlePageResponse> {
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.get_candles(
            &BrokerSymbol("SOXQ".into()),
            "1m",
            Some(200),
            Some(CURSOR_MARKER),
            Some(true),
        ),
    )
    .await
    .unwrap()
}
fn example(name: &str) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(include_str!("response_parsing_fixtures.json"))
        .unwrap()[name]
        .clone()
}
fn malformed_cases() -> Vec<(&'static str, Vec<u8>, &'static str)> {
    let mut number = example("minuteCandles");
    number["result"]["candles"][0]["openPrice"] = serde_json::json!(1.25);
    number["note"] = BODY_MARKER.into();
    let mut missing = example("dailyCandles");
    missing["result"]["candles"][0]
        .as_object_mut()
        .unwrap()
        .remove("volume");
    missing["note"] = BODY_MARKER.into();
    vec![
        ("empty", Vec::new(), "Eof"),
        (
            "html",
            format!("<html>{BODY_MARKER}</html>").into_bytes(),
            "Syntax",
        ),
        (
            "malformed",
            format!(r#"{{"note":"{BODY_MARKER}","result": invalid}}"#).into_bytes(),
            "Syntax",
        ),
        (
            "truncated",
            format!(r#"{{"note":"{BODY_MARKER}","result":"#).into_bytes(),
            "Eof",
        ),
        ("empty-object", b"{}".to_vec(), "Data"),
        (
            "missing-result",
            format!(r#"{{"note":"{BODY_MARKER}"}}"#).into_bytes(),
            "Data",
        ),
        (
            "null-result",
            format!(r#"{{"note":"{BODY_MARKER}","result":null}}"#).into_bytes(),
            "Data",
        ),
        (
            "null-candles",
            format!(r#"{{"note":"{BODY_MARKER}","result":{{"candles":null}}}}"#).into_bytes(),
            "Data",
        ),
        (
            "numeric-price",
            serde_json::to_vec(&number).unwrap(),
            "Data",
        ),
        (
            "missing-volume",
            serde_json::to_vec(&missing).unwrap(),
            "Data",
        ),
    ]
}
fn assert_private(message: &str) {
    for marker in [
        BODY_MARKER,
        TOKEN_MARKER,
        CURSOR_MARKER,
        "synthetic-secret",
        "fixture-account",
    ] {
        assert!(
            !message.contains(marker),
            "unexpected synthetic marker in diagnostic"
        );
    }
}

#[tokio::test]
async fn public_daily_minute_last_page_use_the_real_loopback_client() {
    for name in ["dailyCandles", "minuteCandles", "lastPage"] {
        let expected = example(name);
        let wire = Wire::start(vec![response(200, &serde_json::to_vec(&expected).unwrap())]).await;
        let actual = candles(&wire.client(true).await).await.unwrap();
        assert_eq!(serde_json::to_value(&actual).unwrap(), expected["result"]);
        for candle in actual.candles {
            candle
                .to_broker_candle(&BrokerSymbol("SOXQ".into()))
                .unwrap();
        }
        assert_eq!(wire.count.load(Ordering::SeqCst), 1);
        assert_eq!(
            wire.accept_encodings.lock().await[0].as_deref(),
            Some("identity")
        );
        assert!(wire.requests.lock().await[0].contains("before=DO_NOT_LEAK_CURSOR"));
        assert!(wire.requests.lock().await[0].starts_with(&format!("GET {ENDPOINT}?")));
    }
}
#[tokio::test]
async fn market_json_failures_preserve_only_safe_outer_metadata() {
    for (name, body, category) in malformed_cases() {
        let raw = if name == "html" {
            String::from_utf8(response(200, &body))
                .unwrap()
                .replacen("application/json", "text/html", 1)
                .into_bytes()
        } else {
            response(200, &body)
        };
        let wire = Wire::start(vec![raw]).await;
        let error = candles(&wire.client(true).await).await.unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("endpoint=/api/v1/candles; status=200; stage=json"),
            "case={name}"
        );
        assert!(
            message.contains(&format!("category={category}; line=")),
            "case={name}"
        );
        assert!(message.contains("; column="));
        assert_private(&format!("{error:#}"));
        assert_eq!(error.chain().count(), 1);
        assert!(!message.contains("body="));
        assert_eq!(
            crate::broker::BrokerAdapterError::Provider(error).to_string(),
            message
        );
        assert_eq!(wire.count.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn invalid_utf8_and_interrupted_body_keep_distinct_safe_metadata() {
    let wire = Wire::start(vec![response(200, b"prefix-\xff-DO_NOT_LEAK_BODY")]).await;
    let error = candles(&wire.client(true).await).await.unwrap_err();
    assert!(error
        .to_string()
        .contains("endpoint=/api/v1/candles; status=200; stage=body_utf8"));
    assert!(error
        .to_string()
        .contains("utf8_offset=7; utf8_error_len=Some(1)"));
    assert_private(&format!("{error:#}"));
    assert_eq!(error.chain().count(), 1);
    let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{".to_vec();
    let wire = Wire::start(vec![raw]).await;
    let error = candles(&wire.client(true).await).await.unwrap_err();
    assert!(error
        .to_string()
        .contains("endpoint=/api/v1/candles; status=200; stage=body_read"));
    assert_private(&format!("{error:#}"));
    assert_eq!(error.chain().count(), 1);
    assert_eq!(wire.count.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn multibyte_chunk_boundaries_and_nullable_last_cursor_stay_valid() {
    let body = "{\"result\":{\"candles\":[],\"nextBefore\":null},\"note\":\"합성\"}".as_bytes();
    let wire = Wire::start(vec![chunked(body, 1)]).await;
    let page = candles(&wire.client(true).await).await.unwrap();
    assert!(page.candles.is_empty());
    assert!(page.next_before.is_none());
}
#[tokio::test]
async fn body_limit_preserves_header_stream_and_exact_boundary() {
    use crate::broker::toss::http::TOSS_MAX_RESPONSE_BYTES as LIMIT;
    let raw = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        LIMIT + 1
    )
    .into_bytes();
    let wire = Wire::start(vec![raw]).await;
    assert!(candles(&wire.client(true).await)
        .await
        .unwrap_err()
        .to_string()
        .contains("상한을 초과"));
    let mut exact = EMPTY_PAGE.to_vec();
    exact.resize(LIMIT, b' ');
    let wire = Wire::start(vec![response(200, &exact)]).await;
    assert!(candles(&wire.client(true).await)
        .await
        .unwrap()
        .candles
        .is_empty());
    exact.push(b' ');
    let wire = Wire::start(vec![chunked(&exact, 65536)]).await;
    assert!(candles(&wire.client(true).await)
        .await
        .unwrap_err()
        .to_string()
        .contains("상한을 초과"));
}
#[tokio::test]
async fn valid_and_invalid_token_bodies_keep_the_existing_reader_contract() {
    let token = br#"{"access_token":"fixture-token","token_type":"Bearer","expires_in":3600}"#;
    let wire = Wire::start(vec![response(200, token), response(200, EMPTY_PAGE)]).await;
    assert!(candles(&wire.client(false).await)
        .await
        .unwrap()
        .candles
        .is_empty());
    assert_eq!(wire.count.load(Ordering::SeqCst), 2);
    let wire = Wire::start(vec![response(200, b"\xff")]).await;
    assert_eq!(
        candles(&wire.client(false).await)
            .await
            .unwrap_err()
            .to_string(),
        "토스증권 응답 본문 UTF-8 파싱 실패"
    );
    let raw = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        super::super::http::TOSS_MAX_RESPONSE_BYTES + 1
    )
    .into_bytes();
    let wire = Wire::start(vec![raw]).await;
    assert!(candles(&wire.client(false).await)
        .await
        .unwrap_err()
        .to_string()
        .contains("상한을 초과"));
}
#[tokio::test]
async fn unauthorized_refresh_stays_once_and_malformed_utf8_stays_body_first() {
    let reject = br#"{"error":{"code":"invalid-token","message":"synthetic unauthorized"}}"#;
    let token = br#"{"access_token":"new-fixture-token","token_type":"Bearer","expires_in":3600}"#;
    for status in [200, 401] {
        let final_body = if status == 200 { EMPTY_PAGE } else { reject };
        let wire = Wire::start(vec![
            response(401, reject),
            response(200, token),
            response(status, final_body),
        ])
        .await;
        let result = candles(&wire.client(true).await).await;
        if status == 200 {
            assert!(result.is_ok());
        } else {
            assert_eq!(
                result
                    .unwrap_err()
                    .downcast_ref::<TossHttpError>()
                    .unwrap()
                    .status,
                StatusCode::UNAUTHORIZED
            );
        }
        assert_eq!(wire.count.load(Ordering::SeqCst), 3);
        assert!(wire
            .accept_encodings
            .lock()
            .await
            .iter()
            .all(|header| header.as_deref() == Some("identity")));
        assert!(wire.requests.lock().await[1].starts_with("POST /oauth2/token "));
    }
    let wire = Wire::start(vec![response(401, b"\xff")]).await;
    let error = candles(&wire.client(true).await).await.unwrap_err();
    assert!(error.to_string().contains("UTF-8"));
    assert!(error.downcast_ref::<TossHttpError>().is_none());
    assert_eq!(wire.count.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn typed_http_failures_and_currency_validation_do_not_become_parse_success() {
    let body = br#"{"error":{"code":"synthetic-reject","message":"synthetic"}}"#;
    for status in [400, 429, 500] {
        let wire = Wire::start(vec![response(status, body)]).await;
        let error = candles(&wire.client(true).await).await.unwrap_err();
        assert_eq!(
            error
                .downcast_ref::<TossHttpError>()
                .unwrap()
                .status
                .as_u16(),
            status
        );
        assert_eq!(definitively_rejected(&error), status == 400);
        assert_eq!(wire.count.load(Ordering::SeqCst), 1);
    }
    let mut body = example("lastPage");
    body["result"]["candles"][0]["currency"] = "EUR".into();
    let wire = Wire::start(vec![response(200, &serde_json::to_vec(&body).unwrap())]).await;
    let page = candles(&wire.client(true).await).await.unwrap();
    assert!(page.candles[0]
        .to_broker_candle(&BrokerSymbol("SOXQ".into()))
        .is_err());
}
#[tokio::test]
async fn send_failure_retries_get_once_but_read_failure_does_not_resubmit_post() {
    let wire = Wire::start(vec![Vec::new(), response(200, EMPTY_PAGE)]).await;
    assert!(candles(&wire.client(true).await).await.is_ok());
    assert_eq!(wire.count.load(Ordering::SeqCst), 2);
    let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{".to_vec();
    let wire = Wire::start(vec![raw]).await;
    let client = wire.client(true).await;
    let result: anyhow::Result<serde_json::Value> = client
        .post_json(
            "/api/v1/orders",
            Some("fixture-account"),
            &serde_json::json!({}),
            None,
        )
        .await;
    assert_eq!(
        result.unwrap_err().to_string(),
        "토스증권 응답 본문 읽기 실패"
    );
    assert_eq!(wire.count.load(Ordering::SeqCst), 1);
}

fn encoded_response(coding: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Encoding: {coding}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
#[tokio::test]
async fn identity_negotiation_compares_observed_wire_headers_and_raw_gzip() {
    let gzip = include_bytes!("response_parsing_last_page.json.gz");
    assert_eq!(&gzip[..2], &[0x1f, 0x8b]);
    for identity in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = tokio::sync::oneshot::channel();
        let plain = serde_json::to_vec(&example("lastPage")).unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut head = Vec::new();
            while !head.ends_with(b"\r\n\r\n") {
                assert!(head.len() < 65536);
                head.push(socket.read_u8().await.unwrap());
            }
            let head = String::from_utf8(head).unwrap();
            let accept = head.lines().find_map(|line| {
                line.split_once(':')
                    .filter(|(name, _)| name.eq_ignore_ascii_case("accept-encoding"))
                    .map(|(_, value)| value.trim().to_string())
            });
            let bytes = if accept.as_deref() == Some("identity") {
                response(200, &plain)
            } else {
                encoded_response("gzip", gzip)
            };
            socket.write_all(&bytes).await.unwrap();
            let _ = tx.send(accept);
        });
        let mut client = TossOpenApiClient::without_credentials(base);
        *client.token_state.current_token.lock().await = Some(TossAccessToken {
            access_token: TOKEN_MARKER.into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + Duration::hours(1),
        });
        if !identity {
            client.http = crate::http::client_builder()
                .timeout(super::super::http::TOSS_HTTP_TIMEOUT)
                .build()
                .unwrap();
        }
        let result = candles(&client).await;
        let observed = rx.await.unwrap();
        server.await.unwrap();
        eprintln!("synthetic coding negotiation: explicit_identity={identity}; observed_accept_encoding={observed:?}; success={}",result.is_ok());
        if identity {
            assert_eq!(observed.as_deref(), Some("identity"));
            assert_eq!(result.unwrap().candles.len(), 1);
        } else {
            assert_eq!(observed, None);
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("stage=content_encoding; encoding=gzip"));
        }
    }
}
#[tokio::test]
async fn unconditional_coding_and_truncated_multibyte_are_distinct_controls() {
    let gzip = include_bytes!("response_parsing_last_page.json.gz");
    for (coding, label) in [
        ("gzip", "gzip"),
        ("br", "br"),
        ("deflate", "deflate"),
        ("zstd", "zstd"),
        ("x-DO_NOT_LEAK_BODY", "other"),
    ] {
        let wire = Wire::start(vec![encoded_response(coding, gzip)]).await;
        let error = candles(&wire.client(true).await).await.unwrap_err();
        let message = error.to_string();
        assert!(message.contains("endpoint=/api/v1/candles; status=200; stage=content_encoding"));
        assert!(message.contains(&format!("encoding={label}")));
        assert_private(&format!("{error:#}"));
        assert_eq!(error.chain().count(), 1);
        assert_eq!(wire.count.load(Ordering::SeqCst), 1);
    }
    let mut truncated = b"{\"result\":{\"candles\":[]},\"note\":\"".to_vec();
    truncated.push(0xed);
    let wire = Wire::start(vec![response(200, &truncated)]).await;
    let error = candles(&wire.client(true).await).await.unwrap_err();
    assert!(error.to_string().contains("stage=body_utf8"));
    assert!(error.to_string().contains("utf8_error_len=None"));
}

#[tokio::test]
async fn explicit_identity_and_optional_next_cursor_remain_valid() {
    for coding in ["identity", "IdEnTiTy"] {
        let mut body = example("lastPage");
        body["result"].as_object_mut().unwrap().remove("nextBefore");
        body["result"]["extra"] = "ignored synthetic field".into();
        let wire = Wire::start(vec![encoded_response(
            coding,
            &serde_json::to_vec(&body).unwrap(),
        )])
        .await;
        let page = candles(&wire.client(true).await).await.unwrap();
        assert_eq!(page.candles.len(), 1);
        assert!(page.next_before.is_none());
    }
}
#[tokio::test]
async fn nonmarket_get_and_public_spec_errors_keep_the_legacy_surface() {
    for path in [
        "/api/v1/accounts",
        "/api/v1/orders/DO_NOT_LEAK_ORDER_ID?cursor=DO_NOT_LEAK_CURSOR",
    ] {
        let wire = Wire::start(vec![response(200, b"\xff")]).await;
        let client = wire.client(true).await;
        let result: anyhow::Result<serde_json::Value> =
            client.get_json(path, Some("fixture-account"), None).await;
        let error = result.unwrap_err();
        assert_eq!(error.to_string(), "토스증권 응답 본문 UTF-8 파싱 실패");
        assert_private(&error.to_string());
        assert!(!error.to_string().contains("DO_NOT_LEAK_ORDER_ID"));
        assert_eq!(
            wire.accept_encodings.lock().await[0].as_deref(),
            Some("identity")
        );
    }
    let wire = Wire::start(vec![response(200, b"\xff")]).await;
    let error = wire
        .client(true)
        .await
        .fetch_openapi_overview()
        .await
        .unwrap_err();
    assert_eq!(error.to_string(), "토스증권 응답 본문 UTF-8 파싱 실패");
    assert_eq!(wire.count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn stacked_and_repeated_content_encoding_never_hide_a_later_coding() {
    for coding in [
        "identity\r\nContent-Encoding: gzip",
        "gzip\r\nContent-Encoding: identity",
        "identity, gzip",
        "gzip, br",
        "identity\r\nContent-Encoding: x-DO_NOT_LEAK_BODY",
        "identity\r\nContent-Encoding: identity",
    ] {
        let wire = Wire::start(vec![encoded_response(coding, EMPTY_PAGE)]).await;
        let error = candles(&wire.client(true).await).await.unwrap_err();
        assert!(error
            .to_string()
            .contains("stage=content_encoding; encoding=other"));
        assert_private(&format!("{error:#}"));
        assert_eq!(error.chain().count(), 1);
        assert_eq!(wire.count.load(Ordering::SeqCst), 1);
    }
}
