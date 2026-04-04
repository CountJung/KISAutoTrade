use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::api::token::TokenManager;

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
    pub async fn subscribe(&self, symbols: Vec<String>) -> Result<()> {
        let url = self.ws_url();
        let token = self
            .token_manager
            .read()
            .await
            .get_token()
            .await?;

        let (ws_stream, _) = connect_async(url).await?;
        tracing::info!("KIS WebSocket 연결: {}", url);

        let (mut write, mut read) = ws_stream.split();

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
        write.send(Message::Text(approval_req.to_string())).await?;

        // 구독 요청 (H0STCNT0 = 체결가)
        for symbol in &symbols {
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
            write.send(Message::Text(sub_req.to_string())).await?;
        }

        let price_tx = self.price_tx.clone();

        // 수신 루프 (별도 태스크로 구동)
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Some(event) = parse_realtime_price(&text) {
                            let _ = price_tx.send(event);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::warn!("KIS WebSocket 연결 종료");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("KIS WebSocket 오류: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
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

