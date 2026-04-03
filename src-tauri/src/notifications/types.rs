use chrono::Local;
use serde::{Deserialize, Serialize};

/// 알림 레벨
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationLevel {
    Critical,
    Error,
    Warning,
    Trade,
    Info,
}

impl NotificationLevel {
    pub fn emoji(&self) -> &str {
        match self {
            Self::Critical => "🔴",
            Self::Error => "🟠",
            Self::Warning => "🟡",
            Self::Trade => "🟢",
            Self::Info => "🔵",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Critical => "CRITICAL",
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
            Self::Trade => "TRADE",
            Self::Info => "INFO",
        }
    }
}

/// 알림 이벤트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub level: NotificationLevel,
    pub title: String,
    pub content: String,
    pub cause: Option<String>,
    pub action: Option<String>,
}

impl NotificationEvent {
    pub fn critical(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Critical,
            title: title.into(),
            content: content.into(),
            cause: None,
            action: Some("앱을 재시작하거나 로그를 확인하세요.".to_string()),
        }
    }

    pub fn error(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Error,
            title: title.into(),
            content: content.into(),
            cause: None,
            action: None,
        }
    }

    pub fn trade(content: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Trade,
            title: "체결 완료".to_string(),
            content: content.into(),
            cause: None,
            action: None,
        }
    }

    pub fn info(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Info,
            title: title.into(),
            content: content.into(),
            cause: None,
            action: None,
        }
    }

    /// Discord 메시지 포맷으로 변환
    pub fn to_discord_message(&self) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S KST");
        let header = format!(
            "[{} {}] AutoConditionTrade",
            self.level.emoji(),
            self.level.label()
        );

        let mut lines = vec![
            format!("**{}**", header),
            format!("시각: {}", now),
            format!("내용: {}", self.content),
        ];

        if let Some(cause) = &self.cause {
            lines.push(format!("원인: {}", cause));
        }
        if let Some(action) = &self.action {
            lines.push(format!("조치: {}", action));
        }

        lines.join("\n")
    }
}
