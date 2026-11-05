use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{broadcast, RwLock},
    task::JoinHandle,
};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{protocol::WebSocketConfig, Message},
    MaybeTlsStream, WebSocketStream,
};

use crate::api::token::TokenManager;

/// WebSocket 연결 상태 이벤트 (Tauri 프론트로 emit)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WsStatusEvent {
    /// 현재 연결 여부
    pub connected: bool,
    /// 상태 메시지 (디버그용)
    pub message: String,
}

/// KIS WebSocket 실시간 시세 이벤트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimePrice {
    /// 종목코드
    pub symbol: String,
    /// 현재가
    pub price: i64,
    /// 전일대비
    pub change: i64,
    /// 전일대비율
    pub change_rate: f64,
    /// 체결수량
    pub volume: u64,
    /// 체결시각 (HHmmss)
    pub trade_time: String,
}

/// KIS WebSocket 클라이언트
pub struct KisWebSocketClient {
    is_paper: bool,
    token_manager: Arc<RwLock<TokenManager>>,
    _app_key: String,
    _app_secret: String,
    /// 실시간 이벤트 브로드캐스트 채널
    pub price_tx: broadcast::Sender<RealtimePrice>,
}

impl KisWebSocketClient {
    pub fn new(
        is_paper: bool,
        app_key: String,
        app_secret: String,
        token_manager: Arc<RwLock<TokenManager>>,
    ) -> Self {
        let (price_tx, _) = broadcast::channel(256);
        Self {
            is_paper,
            token_manager,
            _app_key: app_key,
            _app_secret: app_secret,
            price_tx,
        }
    }

    fn ws_url(&self) -> &'static str {
        if self.is_paper {
            "wss://openapivts.koreainvestment.com:29443/websocket/client"
        } else {
            "wss://openapi.koreainvestment.com:9443/websocket/client"
        }
    }

    /// 지정 종목 실시간 시세 구독 시작 (백그라운드 태스크)
    ///
    /// - `app_handle`: Tauri AppHandle — 연결 상태 변경 시 `ws-status` 이벤트 emit
    /// - `ws_connected`: 공유 연결 상태 플래그 (AppState에서 참조)
    pub async fn subscribe(
        &self,
        symbols: Vec<String>,
        app_handle: AppHandle,
        ws_connected: Arc<AtomicBool>,
    ) -> Result<()> {
        let url = self.ws_url();
        let token = self.token_manager.read().await.get_token().await?;

        let ws_stream = connect_stream(url).await?;
        tracing::info!("KIS WebSocket 연결: {}", url);

        // The adapter alone owns AppHandle; the same transport lifecycle is
        // exercised by credential-free loopback tests.
        start_subscription(
            ws_stream,
            token,
            symbols,
            self.price_tx.clone(),
            ws_connected,
            move |event| {
                let _ = app_handle.emit("ws-status", event);
            },
        )
        .await?;
        Ok(())
    }
}

// 0.25 increased the eager read buffer to 128 KiB. Keep the former
// 0.24 read chunk size (4096); this does not limit frame or message sizes.
fn websocket_config() -> WebSocketConfig {
    WebSocketConfig::default().read_buffer_size(4 * 1024)
}

async fn connect_stream(
    url: &str,
) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
    // Keep native roots and the resolved ring provider. No custom verifier,
    // connector, process-wide provider install, or user-configurable endpoint.
    Ok(
        connect_async_with_config(url, Some(websocket_config()), false)
            .await?
            .0,
    )
}

fn publish_status<F: Fn(WsStatusEvent)>(connected: &AtomicBool, sink: &F, value: bool) {
    connected.store(value, Ordering::Relaxed);
    sink(WsStatusEvent {
        connected: value,
        message: if value {
            "KIS WebSocket 연결됨"
        } else {
            "KIS WebSocket 연결 끊김"
        }
        .into(),
    });
}

/// Start one explicitly requested subscription; no retries or reconnect loop.
async fn start_subscription<S, F>(
    mut stream: WebSocketStream<S>,
    token: String,
    symbols: Vec<String>,
    price_tx: broadcast::Sender<RealtimePrice>,
    connected: Arc<AtomicBool>,
    status_sink: F,
) -> Result<JoinHandle<()>>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    F: Fn(WsStatusEvent) + Send + 'static,
{
    publish_status(&connected, &status_sink, true);
    if let Err(error) = send_subscription(&mut stream, &token, &symbols).await {
        publish_status(&connected, &status_sink, false);
        return Err(error);
    }
    Ok(tokio::spawn(async move {
        while let Some(message) = stream.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Some(event) = parse_realtime_price(&text) {
                        let _ = price_tx.send(event);
                    }
                }
                Ok(Message::Close(_)) => {
                    // Tungstenite queues the peer's close reply while reading.
                    // Flush it before dropping the transport, but never let a
                    // stalled peer delay the disconnected status indefinitely.
                    match tokio::time::timeout(Duration::from_secs(2), stream.flush()).await {
                        Ok(Ok(())) => {}
                        Ok(Err(error)) => tracing::warn!("KIS WebSocket 종료 응답 오류: {}", error),
                        Err(_) => tracing::warn!("KIS WebSocket 종료 응답 시간 초과"),
                    }
                    tracing::warn!("KIS WebSocket 연결 종료");
                    break;
                }
                Err(error) => {
                    tracing::error!("KIS WebSocket 오류: {}", error);
                    break;
                }
                _ => {}
            }
        }
        publish_status(&connected, &status_sink, false);
        tracing::info!("KIS WebSocket 수신 루프 종료");
    }))
}

async fn send_subscription<S>(
    stream: &mut WebSocketStream<S>,
    token: &str,
    symbols: &[String],
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // 승인 키 발급 요청 (CTLT_TYPE = G)
    let approval_req = serde_json::json!({
        "header": {
            "approval_key": token,
            "custtype": "P",
            "tr_type": "1",
            "content-type": "utf-8"
        },
        "body": {
            "input": {
                "tr_id": "CTLT_TYPE",
                "tr_key": ""
            }
        }
    });
    stream
        .send(Message::Text(approval_req.to_string().into()))
        .await?;

    // 구독 요청 (H0STCNT0 = 체결가)
    for symbol in symbols {
        let sub_req = serde_json::json!({
            "header": {
                "approval_key": token,
                "custtype": "P",
                "tr_type": "1",
                "content-type": "utf-8"
            },
            "body": {
                "input": {
                    "tr_id": "H0STCNT0",
                    "tr_key": symbol
                }
            }
        });
        stream
            .send(Message::Text(sub_req.to_string().into()))
            .await?;
    }

    Ok(())
}

/// KIS WebSocket 실시간 체결 데이터 파싱
/// 수신 포맷: "0|H0STCNT0|001|{symbol}^{time}^{price}^{change}^{change_rate}^{volume}^..."
fn parse_realtime_price(text: &str) -> Option<RealtimePrice> {
    // JSON 형식 (승인 응답 등)은 스킵
    if text.starts_with('{') {
        return None;
    }

    let parts: Vec<&str> = text.splitn(4, '|').collect();
    if parts.len() < 4 {
        return None;
    }

    // parts[1] = tr_id, parts[3] = pipe-separated data
    let tr_id = parts[1];
    if tr_id != "H0STCNT0" {
        return None;
    }

    let fields: Vec<&str> = parts[3].split('^').collect();
    if fields.len() < 6 {
        return None;
    }

    Some(RealtimePrice {
        symbol: fields[0].to_string(),
        trade_time: fields[1].to_string(),
        price: fields[2].parse().unwrap_or(0),
        change: fields[3].parse().unwrap_or(0),
        change_rate: fields[4].parse().unwrap_or(0.0),
        volume: fields[5].parse().unwrap_or(0),
    })
}

#[cfg(test)]
#[path = "websocket/tests.rs"]
mod tests;
