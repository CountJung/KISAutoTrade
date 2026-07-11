use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Clone)]
pub(super) struct WebSecurity {
    token: Arc<str>,
    pub allow_lan: bool,
}

impl WebSecurity {
    pub(super) fn from_environment() -> Self {
        Self {
            token: std::env::var("WEB_API_TOKEN").unwrap_or_default().into(),
            allow_lan: parse_bool_env("WEB_ALLOW_LAN"),
        }
    }

    pub(super) fn token_configured(&self) -> bool {
        self.token.len() >= 32
    }
}

fn parse_bool_env(name: &str) -> bool {
    std::env::var(name).ok().is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut diff = left.len() ^ right.len();
    let max_len = left.len().max(right.len());
    for index in 0..max_len {
        let l = left.get(index).copied().unwrap_or(0);
        let r = right.get(index).copied().unwrap_or(0);
        diff |= usize::from(l ^ r);
    }
    diff == 0
}

fn bearer_token(headers: &HeaderMap) -> Option<&[u8]> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::as_bytes)
}

fn has_same_origin(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN) else {
        return true;
    };
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    origin.eq_ignore_ascii_case(&format!("http://{host}"))
        || origin.eq_ignore_ascii_case(&format!("https://{host}"))
}

fn rejection(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({ "code": code, "message": message })),
    )
        .into_response()
}

pub(super) async fn require_web_security(
    State(security): State<WebSecurity>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !has_same_origin(request.headers()) {
        return rejection(
            StatusCode::FORBIDDEN,
            "CROSS_ORIGIN_FORBIDDEN",
            "교차 origin 웹 요청은 허용되지 않습니다.",
        );
    }

    if matches!(*request.method(), Method::GET | Method::HEAD) {
        return next.run(request).await;
    }

    if !security.token_configured() {
        return rejection(
            StatusCode::SERVICE_UNAVAILABLE,
            "WEB_API_TOKEN_REQUIRED",
            "웹 변경 API가 잠겨 있습니다. 데스크톱 Settings에서 32자 이상의 API token을 설정하세요.",
        );
    }

    let Some(provided) = bearer_token(request.headers()) else {
        return rejection(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "Authorization: Bearer token이 필요합니다.",
        );
    };
    if !constant_time_eq(provided, security.token.as_bytes()) {
        return rejection(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "웹 API token이 올바르지 않습니다.",
        );
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{middleware, routing::post, Router};
    use tower::ServiceExt;

    fn app(token: &str) -> Router {
        let security = WebSecurity {
            token: Arc::from(token),
            allow_lan: false,
        };
        Router::new()
            .route("/change", post(|| async { StatusCode::NO_CONTENT }))
            .layer(middleware::from_fn_with_state(
                security.clone(),
                require_web_security,
            ))
    }

    #[tokio::test]
    async fn mutation_without_token_is_rejected() {
        let response = app("0123456789abcdef0123456789abcdef")
            .oneshot(Request::post("/change").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_bearer_token_allows_mutation() {
        let response = app("0123456789abcdef0123456789abcdef")
            .oneshot(
                Request::post("/change")
                    .header(
                        header::AUTHORIZATION,
                        "Bearer 0123456789abcdef0123456789abcdef",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn cross_origin_mutation_is_rejected_even_with_token() {
        let response = app("0123456789abcdef0123456789abcdef")
            .oneshot(
                Request::post("/change")
                    .header(header::HOST, "127.0.0.1:7474")
                    .header(header::ORIGIN, "https://evil.example")
                    .header(
                        header::AUTHORIZATION,
                        "Bearer 0123456789abcdef0123456789abcdef",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
