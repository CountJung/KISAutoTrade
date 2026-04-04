use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::AppConfig;

/// KIS Access Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

impl AccessToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at - chrono::Duration::minutes(5)
    }
}

/// 토큰 발급 응답
#[derive(Debug, Deserialize)]
struct TokenIssueResponse {
    access_token: String,
    _token_type: String,
    expires_in: i64,
}

/// KIS 토큰 관리자 (자동 갱신 포함)
pub struct TokenManager {
    client: Client,
    config: Arc<AppConfig>,
    current_token: Mutex<Option<AccessToken>>,
}

impl TokenManager {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            client: Client::new(),
            config,
            current_token: Mutex::new(None),
        }
    }

    /// 유효한 토큰 반환 (만료 시 자동 갱신)
    pub async fn get_token(&self) -> Result<String> {
        let mut token_guard = self.current_token.lock().await;

        if let Some(token) = &*token_guard {
            if !token.is_expired() {
                return Ok(token.token.clone());
            }
        }

        // 토큰 갱신
        let new_token = self.issue_token().await?;
        let token_str = new_token.token.clone();
        *token_guard = Some(new_token);

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

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("토큰 발급 실패 HTTP {}: {}", status, text);
        }

        let data: TokenIssueResponse = resp.json().await?;

        let expires_at = Utc::now() + chrono::Duration::seconds(data.expires_in);
        Ok(AccessToken {
            token: data.access_token,
            expires_at,
        })
    }
}
