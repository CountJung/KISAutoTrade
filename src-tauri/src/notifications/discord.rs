use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

use super::types::NotificationEvent;

/// Discord Bot API를 이용한 알림 전송
/// Webhook이 아닌 Bot API (HTTP) 방식 사용
pub struct DiscordNotifier {
    client: Client,
    bot_token: String,
    channel_id: String,
}

impl DiscordNotifier {
    pub fn new(bot_token: String, channel_id: String) -> Self {
        Self {
            client: Client::new(),
            bot_token,
            channel_id,
        }
    }

    /// 알림 이벤트를 Discord 채널에 전송
    pub async fn send(&self, event: NotificationEvent) -> Result<()> {
        let message = event.to_discord_message();
        self.send_message(&message).await
    }

    /// Discord 채널에 메시지 전송
    /// POST https://discord.com/api/v10/channels/{channel_id}/messages
    async fn send_message(&self, content: &str) -> Result<()> {
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages",
            self.channel_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&json!({ "content": content }))
            .send()
            .await
            .context("Discord API 요청 실패")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Discord API 오류 {} - {}", status, body);
        }

        tracing::debug!("Discord 알림 전송 성공 (channel: {})", self.channel_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications::types::NotificationEvent;

    #[test]
    fn test_discord_message_format() {
        let event = NotificationEvent::critical(
            "테스트",
            "Rust 패닉 발생",
        );
        let msg = event.to_discord_message();
        assert!(msg.contains("CRITICAL"));
        assert!(msg.contains("패닉"));
    }
}
