use std::time::Duration as StdDuration;

use anyhow::{anyhow, Context};
use reqwest::Client;

pub(super) const TOSS_HTTP_TIMEOUT: StdDuration = StdDuration::from_secs(15);
pub(super) const TOSS_MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

pub(super) fn toss_http_client() -> Client {
    Client::builder()
        .timeout(TOSS_HTTP_TIMEOUT)
        .build()
        .expect("Toss reqwest client with timeout should build")
}

pub(super) async fn read_toss_response_text(mut resp: reqwest::Response) -> anyhow::Result<String> {
    if resp.content_length().unwrap_or(0) > TOSS_MAX_RESPONSE_BYTES as u64 {
        return Err(anyhow!(
            "토스증권 응답 본문이 {} bytes 상한을 초과했습니다",
            TOSS_MAX_RESPONSE_BYTES
        ));
    }
    let mut body = Vec::new();
    while let Some(chunk) = resp.chunk().await.context("토스증권 응답 본문 읽기 실패")?
    {
        if body.len().saturating_add(chunk.len()) > TOSS_MAX_RESPONSE_BYTES {
            return Err(anyhow!(
                "토스증권 응답 본문이 {} bytes 상한을 초과했습니다",
                TOSS_MAX_RESPONSE_BYTES
            ));
        }
        body.extend_from_slice(&chunk);
    }
    String::from_utf8(body).context("토스증권 응답 본문 UTF-8 파싱 실패")
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
