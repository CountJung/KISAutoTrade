use anyhow::anyhow;
use reqwest::{header::HeaderMap, StatusCode};
use serde::Deserialize;

use super::http::body_snippet;

#[derive(Debug, Deserialize)]
struct TossErrorResponse {
    error: TossApiError,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TossApiError {
    request_id: Option<String>,
    code: String,
    message: String,
    data: Option<serde_json::Value>,
}

pub(super) fn format_toss_error(
    status: StatusCode,
    headers: &HeaderMap,
    text: &str,
    context: &str,
) -> anyhow::Error {
    let request_id = headers
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let retry_after = headers
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    if let Ok(parsed) = serde_json::from_str::<TossErrorResponse>(text) {
        let data = parsed
            .error
            .data
            .as_ref()
            .map(|value| body_snippet(&value.to_string()));
        return anyhow!(
            "{context}: HTTP {status}; code={}; message={}; request_id={:?}; header_request_id={:?}; retry_after={:?}; data={:?}",
            parsed.error.code,
            body_snippet(&parsed.error.message),
            parsed.error.request_id,
            request_id,
            retry_after,
            data
        );
    }

    anyhow!(
        "{context}: HTTP {status}; request_id={:?}; retry_after={:?}; body={}",
        request_id,
        retry_after,
        body_snippet(text)
    )
}
