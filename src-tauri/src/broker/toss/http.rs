use std::time::Duration as StdDuration;

use anyhow::{anyhow, Context};
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING},
    Client,
};

pub(super) const TOSS_HTTP_TIMEOUT: StdDuration = StdDuration::from_secs(15);
pub(super) const TOSS_MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

pub(super) fn toss_http_client() -> Client {
    // This client has no content-decoding feature. Explicitly negotiate the
    // representation it can read, without changing any other HTTP client.
    let headers = HeaderMap::from_iter([(ACCEPT_ENCODING, HeaderValue::from_static("identity"))]);
    crate::http::client_builder()
        .timeout(TOSS_HTTP_TIMEOUT)
        .default_headers(headers)
        .build()
        .expect("Toss reqwest client with timeout should build")
}

pub(super) async fn read_toss_response_text(resp: reqwest::Response) -> anyhow::Result<String> {
    read_response_text(resp, None).await
}

/// Candles diagnostics never retain raw bytes, dynamic paths or header values.
pub(super) async fn read_toss_candle_response_text(
    resp: reqwest::Response,
) -> anyhow::Result<String> {
    let status = resp.status().as_u16();
    let values = resp.headers().get_all(CONTENT_ENCODING);
    let mut values = values.iter();
    // Multiple lines are a stacked coding just like a comma-separated value.
    // This client supports identity only; never ignore a later coding.
    let encoding = match (values.next(), values.next()) {
        (None, None) => "identity",
        (Some(header), None) => {
            let value = header.to_str().unwrap_or("").trim();
            ["identity", "gzip", "br", "deflate", "zstd"]
                .into_iter()
                .find(|known| value.eq_ignore_ascii_case(known))
                .unwrap_or("other")
        }
        _ => "other",
    };
    if encoding != "identity" {
        return Err(anyhow!(
            "토스증권 응답 본문 인코딩 미지원: endpoint=/api/v1/candles; status={status}; stage=content_encoding; encoding={encoding}"
        ));
    }
    read_response_text(resp, Some(status)).await
}

async fn read_response_text(
    mut resp: reqwest::Response,
    candle_status: Option<u16>,
) -> anyhow::Result<String> {
    let limit_error = || {
        match candle_status {
        Some(status) => anyhow!("토스증권 응답 본문이 {} bytes 상한을 초과했습니다: endpoint=/api/v1/candles; status={status}; stage=body_limit; limit_bytes={}", TOSS_MAX_RESPONSE_BYTES, TOSS_MAX_RESPONSE_BYTES),
        None => anyhow!("토스증권 응답 본문이 {} bytes 상한을 초과했습니다", TOSS_MAX_RESPONSE_BYTES),
    }
    };
    if resp.content_length().unwrap_or(0) > TOSS_MAX_RESPONSE_BYTES as u64 {
        return Err(limit_error());
    }
    let mut body = Vec::new();
    loop {
        let chunk = match resp.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(error) => {
                return match candle_status {
                    Some(status) => Err(anyhow!("토스증권 응답 본문 읽기 실패: endpoint=/api/v1/candles; status={status}; stage=body_read; bytes_read={}", body.len())),
                    None => Err(error).context("토스증권 응답 본문 읽기 실패"),
                };
            }
        };
        if body.len().saturating_add(chunk.len()) > TOSS_MAX_RESPONSE_BYTES {
            return Err(limit_error());
        }
        body.extend_from_slice(&chunk);
    }
    let bytes_read = body.len();
    match String::from_utf8(body) {
        Ok(text) => Ok(text),
        Err(error) => match candle_status {
            Some(status) => {
                let utf8 = error.utf8_error();
                Err(anyhow!("토스증권 응답 본문 UTF-8 파싱 실패: endpoint=/api/v1/candles; status={status}; stage=body_utf8; bytes_read={bytes_read}; utf8_offset={}; utf8_error_len={:?}", utf8.valid_up_to(), utf8.error_len()))
            }
            None => Err(error).context("토스증권 응답 본문 UTF-8 파싱 실패"),
        },
    }
}

pub(super) fn body_snippet(text: &str) -> String {
    const BODY_SNIPPET_CHARS: usize = 2048;
    let mut snippet: String = text.chars().take(BODY_SNIPPET_CHARS).collect();
    if text.chars().nth(BODY_SNIPPET_CHARS).is_some() {
        snippet.push_str("…[truncated]");
    }
    snippet
}

pub(super) fn trim_base_url(value: String) -> String {
    value.trim_end_matches('/').to_string()
}

pub(super) fn url_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_' | b'~' => {
                vec![byte as char]
            }
            other => format!("%{other:02X}").chars().collect(),
        })
        .collect()
}
