use super::*;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadBuf},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::timeout,
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{protocol::frame::coding::CloseCode, protocol::CloseFrame, protocol::Role},
};

const WAIT: Duration = Duration::from_secs(5);
const TOKEN: &str = "synthetic-approval";

async fn pair() -> (
    WebSocketStream<TcpStream>,
    WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
) {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        accept_async(tcp).await.unwrap()
    });
    let client = connect_stream(&format!("ws://{address}")).await.unwrap();
    (server.await.unwrap(), client)
}

#[tokio::test]
async fn peer_close_is_echoed_before_disconnect_status() {
    timeout(WAIT, async {
        let (mut server, client) = pair().await;
        let (prices, _) = broadcast::channel(256);
        let connected = Arc::new(AtomicBool::new(false));
        let (statuses, mut events) = mpsc::channel(8);
        let task = start_subscription(
            client,
            TOKEN.into(),
            vec![],
            prices,
            connected.clone(),
            move |s| {
                statuses.try_send(s).unwrap();
            },
        )
        .await
        .unwrap();
        assert!(events.recv().await.unwrap().connected);
        assert!(matches!(
            server.next().await.unwrap().unwrap(),
            Message::Text(_)
        ));
        let close = CloseFrame {
            code: CloseCode::Normal,
            reason: "합성 종료".into(),
        };
        server
            .send(Message::Close(Some(close.clone())))
            .await
            .unwrap();
        assert_eq!(
            server.next().await.unwrap().unwrap(),
            Message::Close(Some(close))
        );
        task.await.unwrap();
        assert!(!events.recv().await.unwrap().connected);
        assert!(!connected.load(Ordering::Relaxed));
    })
    .await
    .unwrap();
}

struct FailedWrite;
impl AsyncRead for FailedWrite {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Pending
    }
}
impl AsyncWrite for FailedWrite {
    fn poll_write(self: Pin<&mut Self>, _: &mut Context<'_>, _: &[u8]) -> Poll<io::Result<usize>> {
        Poll::Ready(Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "synthetic send failure",
        )))
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn subscription_send_failure_clears_connected_status() {
    let stream = WebSocketStream::from_raw_socket(FailedWrite, Role::Client, None).await;
    let (prices, _) = broadcast::channel(256);
    let connected = Arc::new(AtomicBool::new(false));
    let (statuses, mut events) = mpsc::channel(8);
    let result = start_subscription(
        stream,
        TOKEN.into(),
        vec![],
        prices,
        connected.clone(),
        move |s| {
            statuses.try_send(s).unwrap();
        },
    )
    .await;
    assert!(result.is_err());
    assert!(!connected.load(Ordering::Relaxed));
    assert!(events.recv().await.unwrap().connected);
    assert!(!events.recv().await.unwrap().connected);
}

struct SessionProbe {
    task: Option<JoinHandle<()>>,
    connected: Arc<AtomicBool>,
    statuses: mpsc::Receiver<WsStatusEvent>,
    prices: broadcast::Receiver<RealtimePrice>,
}
impl Drop for SessionProbe {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
impl SessionProbe {
    async fn start<S>(stream: WebSocketStream<S>) -> Self
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (price_tx, prices) = broadcast::channel(256);
        let connected = Arc::new(AtomicBool::new(false));
        let (statuses, mut events) = mpsc::channel(8);
        let task = start_subscription(
            stream,
            TOKEN.into(),
            vec!["TEST_A".into(), "TEST_B".into()],
            price_tx,
            connected.clone(),
            move |event| {
                statuses.try_send(event).unwrap();
            },
        )
        .await
        .unwrap();
        let event = events.recv().await.unwrap();
        assert!(event.connected);
        assert_eq!(event.message, "KIS WebSocket 연결됨");
        assert!(connected.load(Ordering::Relaxed));
        Self {
            task: Some(task),
            connected,
            statuses: events,
            prices,
        }
    }
    async fn finished(&mut self) {
        self.task.take().unwrap().await.unwrap();
        assert!(!self.connected.load(Ordering::Relaxed));
        let event = self.statuses.recv().await.unwrap();
        assert!(!event.connected);
        assert_eq!(event.message, "KIS WebSocket 연결 끊김");
        assert!(self.statuses.try_recv().is_err());
    }
}

async fn drain_requests<S>(server: &mut WebSocketStream<S>) -> Vec<serde_json::Value>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut requests = Vec::new();
    for _ in 0..3 {
        let message = server.next().await.unwrap().unwrap();
        assert!(message.is_text());
        requests.push(serde_json::from_str(message.to_text().unwrap()).unwrap());
    }
    requests
}

async fn close_peer<S>(server: &mut WebSocketStream<S>)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    server.send(Message::Close(None)).await.unwrap();
    assert_eq!(server.next().await.unwrap().unwrap(), Message::Close(None));
}

const PRICE: &str = "0|H0STCNT0|001|TEST_A^101112^12345^-15^-0.12^37";

#[tokio::test]
async fn approval_and_subscription_json_order_and_price_fields_are_preserved() {
    timeout(WAIT, async {
        let (mut server, client) = pair().await;
        assert_eq!(client.get_config().read_buffer_size, 4096);
        let mut probe = SessionProbe::start(client).await;
        let requests = drain_requests(&mut server).await;
        for (request, (tr_id, tr_key)) in requests.iter().zip([
            ("CTLT_TYPE", ""), ("H0STCNT0", "TEST_A"), ("H0STCNT0", "TEST_B")
        ]) {
            assert_eq!(*request, serde_json::json!({
                "header": {"approval_key":TOKEN,"custtype":"P","tr_type":"1","content-type":"utf-8"},
                "body":{"input":{"tr_id":tr_id,"tr_key":tr_key}}
            }));
        }
        server.send(Message::Text(PRICE.into())).await.unwrap();
        let price = probe.prices.recv().await.unwrap();
        assert_eq!(price.symbol, "TEST_A");
        assert_eq!(price.trade_time, "101112");
        assert_eq!(price.price, 12345);
        assert_eq!(price.change, -15);
        assert_eq!(price.change_rate, -0.12);
        assert_eq!(price.volume, 37);
        close_peer(&mut server).await;
        probe.finished().await;
    }).await.unwrap();
}

#[tokio::test]
async fn json_other_tr_malformed_binary_and_pong_do_not_publish_prices() {
    timeout(WAIT, async {
        let (mut server, client) = pair().await;
        let mut probe = SessionProbe::start(client).await;
        drain_requests(&mut server).await;
        for text in [
            "{\"header\":{\"tr_id\":\"PINGPONG\"}}",
            "bad",
            "0|OTHER|001|a^b^c^d^e^f",
            "0|H0STCNT0|001|a^b",
        ] {
            server.send(Message::Text(text.into())).await.unwrap();
        }
        server
            .send(Message::Binary(PRICE.as_bytes().to_vec().into()))
            .await
            .unwrap();
        server
            .send(Message::Pong(vec![3, 2, 1].into()))
            .await
            .unwrap();
        server.send(Message::Text(PRICE.into())).await.unwrap();
        assert_eq!(probe.prices.recv().await.unwrap().price, 12345);
        close_peer(&mut server).await;
        probe.finished().await;
        assert!(probe.prices.try_recv().is_err());
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn auto_pong_echoes_each_ping_exactly_once_and_text_still_arrives() {
    timeout(WAIT, async {
        let (mut server, client) = pair().await;
        let mut probe = SessionProbe::start(client).await;
        drain_requests(&mut server).await;
        for payload in [vec![0, 1, 255], vec![2, 3, 254]] {
            server
                .send(Message::Ping(payload.clone().into()))
                .await
                .unwrap();
            assert_eq!(
                server.next().await.unwrap().unwrap(),
                Message::Pong(payload.into())
            );
        }
        server.send(Message::Text(PRICE.into())).await.unwrap();
        assert_eq!(probe.prices.recv().await.unwrap().symbol, "TEST_A");
        close_peer(&mut server).await;
        probe.finished().await;
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn small_read_buffer_does_not_become_a_message_size_limit() {
    timeout(WAIT, async {
        let (mut server, client) = pair().await;
        let mut probe = SessionProbe::start(client).await;
        drain_requests(&mut server).await;
        let text = serde_json::json!({"ignored": "x".repeat(20*1024)}).to_string();
        server.send(Message::Text(text.into())).await.unwrap();
        server.send(Message::Text(PRICE.into())).await.unwrap();
        assert_eq!(probe.prices.recv().await.unwrap().price, 12345);
        close_peer(&mut server).await;
        probe.finished().await;
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn abrupt_transport_eof_disconnects_without_panicking() {
    timeout(WAIT, async {
        let (mut server, client) = pair().await;
        let mut probe = SessionProbe::start(client).await;
        drain_requests(&mut server).await;
        drop(server);
        probe.finished().await;
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn invalid_utf8_text_disconnects_without_publishing_a_price() {
    timeout(WAIT, async {
        let (mut server, client) = pair().await;
        let mut probe = SessionProbe::start(client).await;
        drain_requests(&mut server).await;
        server
            .get_mut()
            .write_all(&[0x81, 0x01, 0xff])
            .await
            .unwrap();
        probe.finished().await;
        assert!(probe.prices.try_recv().is_err());
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn explicitly_requested_second_subscription_reuses_channels_after_disconnect() {
    timeout(WAIT, async {
        let (price_tx, mut prices) = broadcast::channel(256);
        let connected = Arc::new(AtomicBool::new(false));
        let (status_tx, mut statuses) = mpsc::channel(8);
        for sequence in ["101112", "131415"] {
            // A second connection exists only because this caller asks for it.
            let (mut server, client) = pair().await;
            let sink = status_tx.clone();
            let task = start_subscription(
                client,
                TOKEN.into(),
                vec![],
                price_tx.clone(),
                connected.clone(),
                move |event| {
                    sink.try_send(event).unwrap();
                },
            )
            .await
            .unwrap();
            assert!(statuses.recv().await.unwrap().connected);
            assert!(server.next().await.unwrap().unwrap().is_text());
            server
                .send(Message::Text(PRICE.replace("101112", sequence).into()))
                .await
                .unwrap();
            assert_eq!(prices.recv().await.unwrap().trade_time, sequence);
            close_peer(&mut server).await;
            task.await.unwrap();
            assert!(!statuses.recv().await.unwrap().connected);
            assert!(!connected.load(Ordering::Relaxed));
        }
        assert!(prices.try_recv().is_err());
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn rejected_http_handshake_never_starts_a_subscription() {
    timeout(WAIT, async {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut tcp, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                assert!(request.len() < 16384);
                request.push(tcp.read_u8().await.unwrap());
            }
            tcp.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
                .await
                .unwrap();
        });
        let error = connect_stream(&url).await.unwrap_err();
        match error.downcast_ref::<tokio_tungstenite::tungstenite::Error>() {
            Some(tokio_tungstenite::tungstenite::Error::Http(response)) => {
                assert_eq!(response.status().as_u16(), 403);
            }
            other => panic!("expected HTTP handshake refusal, got {other:?}"),
        }
        server.await.unwrap();
    })
    .await
    .unwrap();
}

#[test]
fn read_buffer_override_preserves_all_other_protocol_limits() {
    let config = websocket_config();
    let defaults = WebSocketConfig::default();
    assert_eq!(config.read_buffer_size, 4096);
    assert_eq!(config.max_message_size, defaults.max_message_size);
    assert_eq!(config.max_frame_size, defaults.max_frame_size);
    assert_eq!(config.write_buffer_size, defaults.write_buffer_size);
    assert_eq!(config.max_write_buffer_size, defaults.max_write_buffer_size);
    assert!(!config.accept_unmasked_frames);
}

#[test]
fn legacy_numeric_parse_fallback_and_json_omission_are_preserved() {
    assert!(parse_realtime_price("{\"ignored\":true}").is_none());
    let result = parse_realtime_price("0|H0STCNT0|001|TEST_A^101112^bad^bad^bad^bad").unwrap();
    assert_eq!((result.price, result.change, result.volume), (0, 0, 0));
    assert_eq!(result.change_rate, 0.0);
}

// Reuse only the checked-in synthetic HTTP fixture. No OS trust-store mutation,
// real provider URL or credential is involved in these connector tests.
const TEST_CA: &[u8] = include_bytes!("../../http/testdata/ca.der");
const TEST_LEAF: &[u8] = include_bytes!("../../http/testdata/server.der");
const TEST_KEY: &[u8] = include_bytes!("../../http/testdata/server-key.der");

async fn tls_fixture(trusted: bool, version: &'static rustls::SupportedProtocolVersion) {
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
    use tokio_tungstenite::{connect_async_tls_with_config, Connector};
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let server_tls = rustls::ServerConfig::builder_with_provider(Arc::new(
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
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let accepted = tokio_rustls::TlsAcceptor::from(Arc::new(server_tls))
            .accept(tcp)
            .await;
        if !trusted {
            assert!(accepted.is_err());
            return;
        }
        let stream = accepted.unwrap();
        assert_eq!(stream.get_ref().1.protocol_version(), Some(version.version));
        let mut server = accept_async(stream).await.unwrap();
        let requests = drain_requests(&mut server).await;
        assert_eq!(requests[0]["header"]["approval_key"], TOKEN);
        server.send(Message::Text(PRICE.into())).await.unwrap();
        close_peer(&mut server).await;
    });
    let mut roots = rustls::RootCertStore::empty();
    if trusted {
        roots.add(CertificateDer::from(TEST_CA.to_vec())).unwrap();
    }
    // The default builder must work with the existing resolved ring-only
    // feature set; installing a process-wide provider would hide ambiguity.
    let tls = rustls::ClientConfig::builder_with_protocol_versions(&[version])
        .with_root_certificates(roots)
        .with_no_client_auth();
    let result = connect_async_tls_with_config(
        format!("wss://localhost:{}", address.port()),
        Some(websocket_config()),
        false,
        Some(Connector::Rustls(Arc::new(tls))),
    )
    .await;
    if trusted {
        let (stream, _) = result.unwrap();
        assert_eq!(stream.get_config().read_buffer_size, 4096);
        let mut probe = SessionProbe::start(stream).await;
        assert_eq!(probe.prices.recv().await.unwrap().symbol, "TEST_A");
        probe.finished().await;
    } else {
        assert!(
            result.is_err(),
            "an untrusted synthetic CA must be rejected"
        );
    }
    server.await.unwrap();
}

#[tokio::test]
async fn synthetic_wss_trusted_root_works_for_tls12_and_tls13() {
    timeout(WAIT, async {
        for version in [&rustls::version::TLS12, &rustls::version::TLS13] {
            tls_fixture(true, version).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn synthetic_wss_rejects_untrusted_root_for_tls12_and_tls13() {
    timeout(WAIT, async {
        for version in [&rustls::version::TLS12, &rustls::version::TLS13] {
            tls_fixture(false, version).await;
        }
    })
    .await
    .unwrap();
}
