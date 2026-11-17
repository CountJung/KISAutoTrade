use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration as StdDuration};
use tokio::sync::Mutex;

use crate::config::AppConfig;

/// KIS Access Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[cfg(test)]
mod redirect_tests {
    use super::*;
    use crate::config::{AccountProfile, DiscordConfig};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    #[tokio::test]
    async fn kis_token_client_never_forwards_post_body_to_redirect_destination() {
        let destination = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let destination_address = destination.local_addr().unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let destination_task = tokio::spawn(async move {
            let (mut stream, _) = destination.accept().await.unwrap();
            observed.fetch_add(1, Ordering::SeqCst);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .await;
        });
        let profile = AccountProfile::new(
            "redirect fixture".into(),
            true,
            "synthetic-key".into(),
            "synthetic-secret".into(),
            "1234567801".into(),
        );
        let manager =
            TokenManager::new(AppConfig::from_profile(&profile, &DiscordConfig::default()));
        for status in [307, 308] {
            let source = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let source_address = source.local_addr().unwrap();
            let source_task = tokio::spawn(async move {
                let (mut stream, _) = source.accept().await.unwrap();
                let mut head = Vec::new();
                while !head.ends_with(b"\r\n\r\n") {
                    assert!(head.len() < 8192);
                    head.push(stream.read_u8().await.unwrap());
                }
                let head = String::from_utf8(head).unwrap();
                let length = head
                    .lines()
                    .find_map(|line| {
                        line.split_once(':')
                            .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                            .map(|(_, value)| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap();
                assert!(length < 1024);
                let mut body = vec![0; length];
                stream.read_exact(&mut body).await.unwrap();
                let redirect = format!("HTTP/1.1 {status} Redirect\r\nLocation: http://{destination_address}/must-not-receive\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                stream.write_all(redirect.as_bytes()).await.unwrap();
            });
            // Exercise the exact client built by TokenManager::new, using synthetic
            // JSON and loopback only; never call issue_token or read operating config.
            let response = manager
                .client
                .post(format!("http://{source_address}/oauth2/tokenP"))
                .json(&serde_json::json!({"appkey":"synthetic-key","appsecret":"synthetic-secret"}))
                .send()
                .await
                .unwrap();
            assert_eq!(response.status().as_u16(), status);
            source_task.await.unwrap();
        }
        tokio::time::sleep(StdDuration::from_millis(100)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        destination_task.abort();
    }
}

impl AccessToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at - chrono::Duration::minutes(5)
    }
}

/// 토큰 발급 응답
/// KIS API 응답: { "access_token": "...", "token_type": "Bearer", "expires_in": 86400, ... }
/// token_type 등 미사용 필드는 serde가 자동으로 무시함
#[derive(Debug, Deserialize)]
struct TokenIssueResponse {
    access_token: String,
    expires_in: i64,
}

/// KIS 토큰 관리자 (자동 갱신 포함)
pub struct TokenManager {
    client: Client,
    config: Arc<AppConfig>,
    current_token: Mutex<Option<AccessToken>>,
    token_refresh: Mutex<()>,
}

impl TokenManager {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            client: crate::http::client_builder()
                .timeout(StdDuration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("KIS reqwest client with timeout should build"),
            config,
            current_token: Mutex::new(None),
            token_refresh: Mutex::new(()),
        }
    }

    /// Localhost 회귀 전용: 실제 token endpoint를 호출하지 않도록 synthetic 캐시를 주입한다.
    #[cfg(test)]
    pub(crate) async fn set_test_access_token(&self, token: String) {
        *self.current_token.lock().await = Some(AccessToken {
            token,
            expires_at: Utc::now() + chrono::Duration::hours(1),
        });
    }

    /// 유효한 토큰 반환 (만료 시 자동 갱신)
    pub async fn get_token(&self) -> Result<String> {
        if let Some(token) = self.current_token.lock().await.as_ref().cloned() {
            if !token.is_expired() {
                return Ok(token.token);
            }
        }

        let _refresh_guard = self.token_refresh.lock().await;
        if let Some(token) = self.current_token.lock().await.as_ref().cloned() {
            if !token.is_expired() {
                return Ok(token.token);
            }
        }

        // 토큰 갱신
        let new_token = self.issue_token().await?;
        let token_str = new_token.token.clone();
        *self.current_token.lock().await = Some(new_token);

        tracing::info!("KIS 액세스 토큰 갱신 완료");
        Ok(token_str)
    }

    /// 신규 토큰 발급 (KIS API 호출)
    async fn issue_token(&self) -> Result<AccessToken> {
        let url = format!("{}/oauth2/tokenP", self.config.kis_base_url());

        #[derive(Serialize)]
        struct TokenRequest {
            grant_type: String,
            appkey: String,
            appsecret: String,
        }

        let body = TokenRequest {
            grant_type: "client_credentials".into(),
            appkey: self.config.kis_app_key.clone(),
            appsecret: self.config.kis_app_secret.clone(),
        };

        let resp = self
            .client
            .post(&url)
            .header("content-type", "application/json; charset=utf-8")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("토큰 발급 실패 HTTP {}: {}", status, text);
        }

        let data: TokenIssueResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow!("토큰 발급 응답 파싱 실패: {}; body={}", e, text))?;

        let expires_at = Utc::now() + chrono::Duration::seconds(data.expires_in);
        Ok(AccessToken {
            token: data.access_token,
            expires_at,
        })
    }
}
