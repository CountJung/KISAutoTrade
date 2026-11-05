use super::{route_cases::CASES, test_support::*};
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use std::collections::BTreeSet;

#[tokio::test]
async fn every_production_binding_is_matched_before_cross_origin_rejection() {
    let fixture = Fixture::new(TOKEN).await;
    assert_eq!(CASES.len(), 74);
    assert_eq!(
        CASES
            .iter()
            .map(|(_, pattern, _)| *pattern)
            .collect::<BTreeSet<_>>()
            .len(),
        70
    );
    assert_eq!(
        CASES
            .iter()
            .filter(|(_, p, _)| p.contains(':'))
            .map(|(_, p, _)| *p)
            .collect::<BTreeSet<_>>()
            .len(),
        13
    );
    for (method, pattern, uri) in CASES {
        let response = fixture
            .send(
                request(method, uri)
                    .header(header::ORIGIN, "https://cross-origin.invalid")
                    .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
                    .body(Body::from("not JSON and must never reach a handler"))
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{method} {uri}");
        assert_eq!(
            canonical(response.headers()["x-fixture-route"].to_str().unwrap()),
            canonical(pattern),
            "{method} {uri}"
        );
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());
        assert_eq!(json_body(response).await["code"], "CROSS_ORIGIN_FORBIDDEN");
    }
    fixture.assert_no_effects().await;
    eprintln!("route bindings checked: 74; distinct URLs: 70; capture URLs: 13");
}

#[tokio::test]
async fn every_post_rejects_unconfigured_missing_and_incorrect_tokens_before_extractors() {
    for (configured, authorization, status, code) in [
        (
            "",
            None,
            StatusCode::SERVICE_UNAVAILABLE,
            "WEB_API_TOKEN_REQUIRED",
        ),
        (
            "short",
            Some(TOKEN),
            StatusCode::SERVICE_UNAVAILABLE,
            "WEB_API_TOKEN_REQUIRED",
        ),
        (TOKEN, None, StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
        (
            TOKEN,
            Some("incorrect-synthetic-token"),
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
        ),
    ] {
        let fixture = Fixture::new(configured).await;
        for (method, _, uri) in CASES.iter().filter(|(m, _, _)| *m == "POST") {
            let mut builder = request(method, uri);
            if let Some(token) = authorization {
                builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
            }
            let response = fixture
                .send(builder.body(Body::from("malformed")).unwrap())
                .await;
            assert_eq!(response.status(), status, "{method} {uri}");
            assert_eq!(json_body(response).await["code"], code);
        }
        fixture.assert_no_effects().await;
    }
    eprintln!("POST token-denial cases checked: 35 x 4 = 140");
}

#[tokio::test]
async fn head_and_options_keep_existing_origin_and_method_policy() {
    let fixture = Fixture::new(TOKEN).await;
    for (_, pattern, uri) in CASES.iter().filter(|(m, _, _)| *m == "GET") {
        let response = fixture
            .send(
                request("HEAD", uri)
                    .header(header::ORIGIN, "null")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            canonical(response.headers()["x-fixture-route"].to_str().unwrap()),
            canonical(pattern)
        );
        assert!(body(response).await.is_empty());
    }
    let response = fixture
        .send(request("HEAD", "/api/info").body(Body::empty()).unwrap())
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(body(response).await.is_empty());
    for (auth, status) in [
        (false, StatusCode::UNAUTHORIZED),
        (true, StatusCode::METHOD_NOT_ALLOWED),
    ] {
        let builder = if auth {
            authorized("OPTIONS", "/api/info")
        } else {
            request("OPTIONS", "/api/info")
        };
        let response = fixture.send(builder.body(Body::empty()).unwrap()).await;
        assert_eq!(response.status(), status);
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());
    }
    fixture.assert_no_effects().await;
}

#[tokio::test]
async fn unsupported_methods_keep_405_and_allow_for_every_registered_url() {
    let fixture = Fixture::new(TOKEN).await;
    let patterns = CASES
        .iter()
        .map(|(_, p, u)| (*p, *u))
        .collect::<BTreeSet<_>>();
    for (pattern, uri) in patterns {
        let response = fixture
            .send(authorized("PUT", uri).body(Body::empty()).unwrap())
            .await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED, "{uri}");
        let actual = response.headers()[header::ALLOW]
            .to_str()
            .unwrap()
            .split(',')
            .map(str::trim)
            .collect::<BTreeSet<_>>();
        let mut expected = CASES
            .iter()
            .filter(|(_, p, _)| *p == pattern)
            .map(|(m, _, _)| *m)
            .collect::<BTreeSet<_>>();
        if expected.contains("GET") {
            expected.insert("HEAD");
        }
        assert_eq!(actual, expected, "{uri}");
    }
    fixture.assert_no_effects().await;
    eprintln!("405/Allow URL cases checked: 70");
}

#[tokio::test]
async fn origin_host_and_exact_bearer_rules_are_preserved_before_body_parsing() {
    let fixture = Fixture::new(TOKEN).await;
    for (origin, status) in [
        (None, StatusCode::UNSUPPORTED_MEDIA_TYPE),
        (
            Some("http://fixture.test:7474"),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            Some("HTTPS://FIXTURE.TEST:7474"),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (Some("http://fixture.test:7475"), StatusCode::FORBIDDEN),
        (Some("null"), StatusCode::FORBIDDEN),
    ] {
        let mut builder = authorized("POST", "/api/frontend-log");
        if let Some(origin) = origin {
            builder = builder.header(header::ORIGIN, origin);
        }
        assert_eq!(
            fixture
                .send(builder.body(Body::empty()).unwrap())
                .await
                .status(),
            status
        );
    }
    let no_host = Request::post("/api/frontend-log")
        .header(header::ORIGIN, "http://fixture.test:7474")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(fixture.send(no_host).await.status(), StatusCode::FORBIDDEN);
    let bad_origin = authorized("POST", "/api/frontend-log")
        .header(
            header::ORIGIN,
            axum::http::HeaderValue::from_bytes(&[0xff]).unwrap(),
        )
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        fixture.send(bad_origin).await.status(),
        StatusCode::FORBIDDEN
    );
    for value in [
        format!("bearer {TOKEN}"),
        format!("Bearer {TOKEN} "),
        format!("Basic {TOKEN}"),
    ] {
        let response = fixture
            .send(
                request("POST", "/api/frontend-log")
                    .header(header::AUTHORIZATION, value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    fixture.assert_no_effects().await;
}

#[tokio::test]
async fn spa_fallback_assets_head_and_unknown_api_behavior_are_preserved() {
    let fixture = Fixture::new(TOKEN).await;
    let response = fixture
        .send(
            request("GET", "/unregistered-route")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers()[header::CONTENT_TYPE]
        .to_str()
        .unwrap()
        .starts_with("text/html"));
    assert!(String::from_utf8(body(response).await)
        .unwrap()
        .contains("KISAutoTrade"));
    std::fs::create_dir_all(&fixture.state.dist_path).unwrap();
    std::fs::write(
        fixture.state.dist_path.join("index.html"),
        "<html>synthetic SPA</html>",
    )
    .unwrap();
    std::fs::write(
        fixture.state.dist_path.join("asset.js"),
        "const fixture = true;",
    )
    .unwrap();
    let response = fixture
        .send(request("GET", "/asset.js").body(Body::empty()).unwrap())
        .await;
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/javascript; charset=utf-8"
    );
    assert_eq!(body(response).await, b"const fixture = true;");
    let response = fixture
        .send(
            request("GET", "/api/not-registered")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.headers()["x-fixture-route"], "<fallback>");
    assert_eq!(body(response).await, b"<html>synthetic SPA</html>");
    let response = fixture
        .send(
            authorized("POST", "/api/not-registered")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    fixture.assert_no_effects().await;
}

#[tokio::test]
async fn isolated_socket_serves_the_production_router_without_app_startup() {
    let fixture = Fixture::new(TOKEN).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = fixture.app();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = crate::http::client_builder().no_proxy().build().unwrap();
    let result: serde_json::Value = client
        .get(format!("http://{address}/api/info"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(result["app"], "KISAutoTrade");
    let response = client
        .post(format!("http://{address}/api/order"))
        .body("malformed")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    task.abort();
    fixture.assert_no_effects().await;
}
