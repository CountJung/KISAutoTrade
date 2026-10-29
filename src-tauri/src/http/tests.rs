//! No external network or OS trust access. The bundled fixture private key is
//! deliberately public test material, never an application credential.
use super::*;
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
};

struct HttpFixture {
    address: SocketAddr,
    failures: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
}

async fn echo(
    uri: Uri,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Json<Value> {
    Json(json!({"uri":uri.to_string(),"query":query,"body":body,
        "fixture":headers.get("x-fixture").and_then(|h|h.to_str().ok()),
        "agent":headers.get(header::USER_AGENT).and_then(|h|h.to_str().ok()),
        "contentType":headers.get(header::CONTENT_TYPE).and_then(|h|h.to_str().ok())}))
}

impl HttpFixture {
    async fn new() -> Self {
        let failures = Arc::new(AtomicUsize::new(0));
        let router = Router::new()
            .route("/echo", post(echo))
            .route("/headers", get(|headers: HeaderMap| async move {
                Json(json!({
                    "authorization":headers.get(header::AUTHORIZATION).and_then(|h|h.to_str().ok()),
                    "cookie":headers.get(header::COOKIE).and_then(|h|h.to_str().ok())
                }))
            }))
            .route("/redirect-to", get(|Query(input): Query<HashMap<String,String>>| async move {
                (StatusCode::FOUND, [(header::LOCATION,input["target"].clone())])
            }))
            .route("/form", post(|headers: HeaderMap, body: String| async move {
                Json(json!({"body":body,"contentType":headers[header::CONTENT_TYPE].to_str().unwrap()}))
            }))
            .route("/set-cookie", get(|| async {
                ([(header::SET_COOKIE, "fixture=session; Path=/; HttpOnly")], "ok")
            }))
            .route("/cookie", get(|headers: HeaderMap| async move {
                headers.get(header::COOKIE).and_then(|h|h.to_str().ok()).unwrap_or("").to_string()
            }))
            .route("/redirect", get(|| async {
                (StatusCode::FOUND, [(header::LOCATION, "/cookie")])
            }))
            .route("/error", get(|State(count): State<Arc<AtomicUsize>>| async move {
                count.fetch_add(1, Ordering::SeqCst);
                (StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER,"3"),(header::HeaderName::from_static("x-request-id"),"fixture-id")], Json(json!({"error":"fixture unavailable"}))).into_response()
            }))
            .route("/slow", get(|| async {
                tokio::time::sleep(Duration::from_millis(250)).await;
                "late"
            }))
            .with_state(failures.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        Self {
            address,
            failures,
            task,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.address)
    }
}

impl Drop for HttpFixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[tokio::test]
async fn http_query_headers_json_and_oauth_form_keep_wire_contracts() {
    let fixture = HttpFixture::new().await;
    let mut headers = HeaderMap::new();
    headers.insert("x-fixture", "synthetic-only".parse().unwrap());
    let client = client_builder()
        .no_proxy()
        .default_headers(headers)
        .user_agent("KISAutoTrade/fixture")
        .build()
        .unwrap();
    let result: Value = client
        .post(fixture.url("/echo"))
        .query(&[("symbol", "A&B +한글"), ("period", "D")])
        .json(&json!({"quantity":"2","price":"3.01"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(result["query"]["symbol"], "A&B +한글");
    assert_eq!(result["query"]["period"], "D");
    assert_eq!(result["body"], json!({"quantity":"2","price":"3.01"}));
    assert_eq!(result["fixture"], "synthetic-only");
    assert_eq!(result["agent"], "KISAutoTrade/fixture");
    assert_eq!(result["contentType"], "application/json");
    let result: Value = client
        .post(fixture.url("/form"))
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", "fixture &id"),
            ("client_secret", "fixture+secret"),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        result["body"],
        "grant_type=client_credentials&client_id=fixture+%26id&client_secret=fixture%2Bsecret"
    );
    assert_eq!(result["contentType"], "application/x-www-form-urlencoded");
}

#[tokio::test]
async fn http_cookies_and_redirect_policy_remain_configurable() {
    let fixture = HttpFixture::new().await;
    let client = client_builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    client.get(fixture.url("/set-cookie")).send().await.unwrap();
    let response = client.get(fixture.url("/redirect")).send().await.unwrap();
    assert_eq!(response.url().path(), "/cookie");
    assert_eq!(response.text().await.unwrap(), "fixture=session");
    let plain = client_builder().no_proxy().build().unwrap();
    plain.get(fixture.url("/set-cookie")).send().await.unwrap();
    assert_eq!(
        plain
            .get(fixture.url("/cookie"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        ""
    );
    let no_redirect = client_builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    assert_eq!(
        no_redirect
            .get(fixture.url("/redirect"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FOUND
    );
}

#[tokio::test]
async fn http_error_status_keeps_metadata_without_adding_status_retries() {
    let fixture = HttpFixture::new().await;
    let client = client_builder().no_proxy().build().unwrap();
    let response = client.get(fixture.url("/error")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[header::RETRY_AFTER], "3");
    assert_eq!(response.headers()["x-request-id"], "fixture-id");
    assert_eq!(
        response.error_for_status_ref().unwrap_err().status(),
        Some(StatusCode::SERVICE_UNAVAILABLE)
    );
    assert_eq!(
        response.json::<Value>().await.unwrap(),
        json!({"error":"fixture unavailable"})
    );
    assert_eq!(fixture.failures.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn http_timeout_and_explicit_proxy_settings_are_preserved() {
    let fixture = HttpFixture::new().await;
    let client = client_builder()
        .no_proxy()
        .timeout(Duration::from_millis(30))
        .build()
        .unwrap();
    assert!(client
        .get(fixture.url("/slow"))
        .send()
        .await
        .unwrap_err()
        .is_timeout());
    // The explicit loopback proxy prevents DNS or requests to the synthetic target.
    let proxy = reqwest::Proxy::http(fixture.url("")).unwrap();
    let client = client_builder().no_proxy().proxy(proxy).build().unwrap();
    let response: Value = client
        .post("http://proxy-target.invalid/echo")
        .json(&json!({"proxy":true}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["body"], json!({"proxy":true}));
    assert_eq!(response["uri"], "http://proxy-target.invalid/echo");
}

const TEST_CA: &[u8] = include_bytes!("testdata/ca.der");
const TEST_LEAF: &[u8] = include_bytes!("testdata/server.der");
const TEST_KEY: &[u8] = include_bytes!("testdata/server-key.der");
type Handshake = (
    Option<String>,
    Option<Vec<u8>>,
    Option<rustls::ProtocolVersion>,
);

struct TlsFixture {
    address: SocketAddr,
    handshakes: Arc<Mutex<Vec<Handshake>>>,
    task: tokio::task::JoinHandle<()>,
}

impl TlsFixture {
    async fn new(version: &'static rustls::SupportedProtocolVersion) -> Self {
        let mut tls = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_protocol_versions(&[version])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(TEST_LEAF.to_vec())],
            PrivatePkcs8KeyDer::from(TEST_KEY.to_vec()).into(),
        )
        .unwrap();
        tls.alpn_protocols = vec![b"http/1.1".to_vec()];
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let handshakes = Arc::new(Mutex::new(Vec::new()));
        let recorded = handshakes.clone();
        let task = tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let (socket, _) = accepted.unwrap();
                        let acceptor = acceptor.clone();
                        let recorded = recorded.clone();
                        connections.spawn(async move {
                            let Ok(Ok(mut stream)) = tokio::time::timeout(Duration::from_secs(2), acceptor.accept(socket)).await else { return };
                            let connection = &stream.get_ref().1;
                            recorded.lock().await.push((connection.server_name().map(str::to_string),
                                connection.alpn_protocol().map(<[_]>::to_vec),connection.protocol_version()));
                            let mut request = Vec::new();
                            while !request.ends_with(b"\r\n\r\n") && request.len() < 65_536 {
                                let Ok(byte) = stream.read_u8().await else { return };
                                request.push(byte);
                            }
                            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}").await.unwrap();
                            let _ = stream.shutdown().await;
                        });
                    }
                    _ = connections.join_next(), if !connections.is_empty() => {}
                }
            }
        });
        Self {
            address,
            handshakes,
            task,
        }
    }

    fn url(&self, host: &str) -> String {
        format!("https://{host}:{}/fixture", self.address.port())
    }

    fn trusted_builder(&self, host: &str) -> ClientBuilder {
        let mut roots = RootCertStore::empty();
        roots.add(CertificateDer::from(TEST_CA.to_vec())).unwrap();
        // Same TLS factory as production, with a CA confined to this test client.
        builder_with_roots(roots)
            .no_proxy()
            .resolve(host, self.address)
            .timeout(Duration::from_secs(2))
    }
}

impl Drop for TlsFixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn error_chain(error: &reqwest::Error) -> String {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(error) = source {
        message.push_str(&format!("; {error}"));
        source = error.source();
    }
    message
}

#[tokio::test]
async fn tls_12_and_13_keep_certificate_verification_dns_sni_and_http1_alpn() {
    for (version, expected) in [
        (&rustls::version::TLS12, rustls::ProtocolVersion::TLSv1_2),
        (&rustls::version::TLS13, rustls::ProtocolVersion::TLSv1_3),
    ] {
        let fixture = TlsFixture::new(version).await;
        let client = fixture.trusted_builder("fixture.test").build().unwrap();
        assert_eq!(
            client
                .get(fixture.url("fixture.test"))
                .send()
                .await
                .unwrap()
                .json::<Value>()
                .await
                .unwrap(),
            json!({"ok":true})
        );
        let handshakes = fixture.handshakes.lock().await;
        assert_eq!(
            *handshakes,
            vec![(
                Some("fixture.test".into()),
                Some(b"http/1.1".to_vec()),
                Some(expected)
            )]
        );
    }
}

#[tokio::test]
async fn tls_production_bundled_roots_reject_the_untrusted_fixture_ca() {
    let fixture = TlsFixture::new(&rustls::version::TLS13).await;
    let client = client_builder()
        .no_proxy()
        .resolve("fixture.test", fixture.address)
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let error = client
        .get(fixture.url("fixture.test"))
        .send()
        .await
        .unwrap_err();
    let message = error_chain(&error);
    assert!(
        error.is_connect() && message.contains("UnknownIssuer"),
        "{message}"
    );
}

#[tokio::test]
async fn tls_trusted_fixture_ca_still_rejects_a_wrong_hostname() {
    let fixture = TlsFixture::new(&rustls::version::TLS13).await;
    let client = fixture
        .trusted_builder("wrong.fixture.test")
        .build()
        .unwrap();
    let error = client
        .get(fixture.url("wrong.fixture.test"))
        .send()
        .await
        .unwrap_err();
    let message = error_chain(&error);
    assert!(
        error.is_connect() && message.contains("certificate not valid for name"),
        "{message}"
    );
}

#[test]
fn http_ring_selection_keeps_existing_websocket_default_builder_usable() {
    // Exercise the same default-provider constructor used by the unchanged WS
    // dependency. Only ring is enabled in the resolved dependency graph.
    let roots = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let tls = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let ring = rustls::crypto::ring::default_provider();
    assert_eq!(
        tls.crypto_provider().cipher_suites.len(),
        ring.cipher_suites.len()
    );
    for (actual, expected) in tls
        .crypto_provider()
        .cipher_suites
        .iter()
        .zip(&ring.cipher_suites)
    {
        assert_eq!(actual.suite(), expected.suite());
    }
    assert!(client_builder().build().is_ok());
}

#[tokio::test]
async fn http_redirect_retains_same_origin_but_strips_cross_origin_sensitive_headers() {
    let origin = HttpFixture::new().await;
    let other = HttpFixture::new().await;
    let client = client_builder().no_proxy().build().unwrap();
    for (target, forwarded) in [
        (origin.url("/headers"), true),
        (other.url("/headers"), false),
    ] {
        let result: Value = client
            .get(origin.url("/redirect-to"))
            .query(&[("target", target)])
            .header(header::AUTHORIZATION, "Bearer synthetic-only")
            .header(header::COOKIE, "fixture=explicit")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if forwarded {
            assert_eq!(result["authorization"], "Bearer synthetic-only");
            assert_eq!(result["cookie"], "fixture=explicit");
        } else {
            assert!(result["authorization"].is_null());
            assert!(result["cookie"].is_null());
        }
    }
}

#[tokio::test]
async fn http_auth_and_order_posts_are_not_resubmitted_after_response_disconnect() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let recorded = received.clone();
    let server = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut header_bytes = Vec::new();
            while !header_bytes.ends_with(b"\r\n\r\n") {
                header_bytes.push(socket.read_u8().await.unwrap());
                assert!(header_bytes.len() < 16_384);
            }
            let headers = String::from_utf8(header_bytes).unwrap();
            let length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().unwrap())
                })
                .unwrap();
            let mut body = vec![0; length];
            socket.read_exact(&mut body).await.unwrap();
            recorded
                .lock()
                .await
                .push((headers.lines().next().unwrap().to_string(), body));
            // Provider received the complete POST; no HTTP response is sent.
            socket.shutdown().await.unwrap();
        }
    });
    let client = client_builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    for (index, path) in ["/fixture-auth", "/fixture-order"].iter().enumerate() {
        let payload = json!({"intent":path,"quantity":"1"});
        let error = client
            .post(format!("http://{address}{path}"))
            .json(&payload)
            .send()
            .await
            .unwrap_err();
        assert!(
            error.is_request() && error.status().is_none(),
            "{}",
            error_chain(&error)
        );
        let received = received.lock().await;
        assert_eq!(
            received.len(),
            index + 1,
            "a received POST must never be automatically resubmitted"
        );
        assert_eq!(received[index].0, format!("POST {path} HTTP/1.1"));
        assert_eq!(
            serde_json::from_slice::<Value>(&received[index].1).unwrap(),
            payload
        );
    }
    server.abort();
}
