pub mod discord;
pub mod types;

pub use discord::DiscordNotifier;
pub use types::{NotificationEvent, NotificationLevel};

use anyhow::Result;

/// 알림 서비스 trait - 테스트 시 Mock으로 교체 가능
#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    async fn send(&self, event: NotificationEvent) -> Result<()>;
}
