use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
use tokio::fs;
use uuid::Uuid;

// ────────────────────────────────────────────────────────────────────
// AccountProfile — 계좌 프로파일 (민감 정보 포함, profiles.json에 저장)
// ────────────────────────────────────────────────────────────────────

/// 하나의 KIS 계좌/앱키 세트를 나타내는 프로파일
/// profiles.json에 저장되며, gitignore에 포함되어 커밋되지 않습니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfile {
    pub id: String,
    pub name: String,
    pub is_paper_trading: bool,
    pub app_key: String,
    pub app_secret: String,
    pub account_no: String,
}

impl AccountProfile {
    pub fn new(
        name: String,
        is_paper_trading: bool,
        app_key: String,
        app_secret: String,
        account_no: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            is_paper_trading,
            app_key,
            app_secret,
            account_no,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.app_key.is_empty()
            && !self.app_secret.is_empty()
            && !self.account_no.is_empty()
    }
}

// ────────────────────────────────────────────────────────────────────
// ProfilesConfig — 프로파일 목록 + 활성 ID (profiles.json)
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfilesConfig {
    pub active_id: Option<String>,
    pub profiles: Vec<AccountProfile>,
}

impl ProfilesConfig {
    /// profiles.json 로드 (없으면 빈 기본값)
    pub async fn load(path: &Path) -> Self {
        match fs::read_to_string(path).await {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => ProfilesConfig::default(),
        }
    }

    /// profiles.json 저장
    pub async fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).await?;
        Ok(())
    }

    /// 현재 활성 프로파일 반환
    pub fn get_active(&self) -> Option<&AccountProfile> {
        self.active_id
            .as_ref()
            .and_then(|id| self.profiles.iter().find(|p| &p.id == id))
    }

    /// 프로파일 추가 (최초 추가 시 자동으로 활성화)
    pub fn add(&mut self, profile: AccountProfile) -> AccountProfile {
        if self.active_id.is_none() {
            self.active_id = Some(profile.id.clone());
        }
        self.profiles.push(profile.clone());
        profile
    }

    /// 프로파일 수정 (빈 문자열은 기존 값 유지)
    pub fn update(
        &mut self,
        id: &str,
        name: Option<String>,
        is_paper_trading: Option<bool>,
        app_key: Option<String>,
        app_secret: Option<String>,
        account_no: Option<String>,
    ) -> Option<AccountProfile> {
        if let Some(p) = self.profiles.iter_mut().find(|p| p.id == id) {
            if let Some(v) = name {
                p.name = v;
            }
            if let Some(v) = is_paper_trading {
                p.is_paper_trading = v;
            }
            // 빈 문자열이 아닌 경우에만 교체 (UI에서 "변경 안 함" 유지)
            if let Some(v) = app_key {
                if !v.is_empty() {
                    p.app_key = v;
                }
            }
            if let Some(v) = app_secret {
                if !v.is_empty() {
                    p.app_secret = v;
                }
            }
            if let Some(v) = account_no {
                if !v.is_empty() {
                    p.account_no = v;
                }
            }
            Some(p.clone())
        } else {
            None
        }
    }

    /// 프로파일 삭제 (활성이면 첫 번째 프로파일로 자동 변경)
    pub fn delete(&mut self, id: &str) -> bool {
        let before = self.profiles.len();
        self.profiles.retain(|p| p.id != id);
        if self.active_id.as_deref() == Some(id) {
            self.active_id = self.profiles.first().map(|p| p.id.clone());
        }
        self.profiles.len() < before
    }

    /// 활성 프로파일 변경
    pub fn set_active(&mut self, id: &str) -> bool {
        if self.profiles.iter().any(|p| p.id == id) {
            self.active_id = Some(id.to_string());
            true
        } else {
            false
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// AppConfig — 활성 프로파일에서 파생되는 런타임 설정
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // ── KIS 활성 설정 ─────────────────────────────────────────────
    pub kis_app_key: String,
    pub kis_app_secret: String,
    pub kis_account_no: String,
    pub kis_is_paper_trading: bool,

    // ── Discord ──────────────────────────────────────────────────
    pub discord_bot_token: Option<String>,
    pub discord_channel_id: Option<String>,
    pub notification_levels: Vec<String>,
}

impl AppConfig {
    /// 활성 AccountProfile + DiscordConfig에서 AppConfig 생성
    pub fn from_profile(profile: &AccountProfile, discord: &DiscordConfig) -> Arc<Self> {
        Arc::new(Self {
            kis_app_key: profile.app_key.clone(),
            kis_app_secret: profile.app_secret.clone(),
            kis_account_no: profile.account_no.clone(),
            kis_is_paper_trading: profile.is_paper_trading,
            discord_bot_token: discord.bot_token.clone(),
            discord_channel_id: discord.channel_id.clone(),
            notification_levels: discord.notification_levels.clone(),
        })
    }

    /// 프로파일이 없는 경우의 빈 설정
    pub fn empty(discord: &DiscordConfig) -> Arc<Self> {
        Arc::new(Self {
            kis_app_key: String::new(),
            kis_app_secret: String::new(),
            kis_account_no: String::new(),
            kis_is_paper_trading: false,
            discord_bot_token: discord.bot_token.clone(),
            discord_channel_id: discord.channel_id.clone(),
            notification_levels: discord.notification_levels.clone(),
        })
    }

    pub fn kis_base_url(&self) -> &str {
        if self.kis_is_paper_trading {
            "https://openapivts.koreainvestment.com:29443"
        } else {
            "https://openapi.koreainvestment.com:9443"
        }
    }

    pub fn is_kis_configured(&self) -> bool {
        !self.kis_app_key.is_empty()
            && !self.kis_app_secret.is_empty()
            && !self.kis_account_no.is_empty()
    }
}

// ────────────────────────────────────────────────────────────────────
// DiscordConfig — Discord 전용 설정 (secure_config.json에서만 로드)
// ────────────────────────────────────────────────────────────────────

/// Discord 설정 (profiles.json과 분리)
#[derive(Debug, Clone, Default)]
pub struct DiscordConfig {
    pub bot_token: Option<String>,
    pub channel_id: Option<String>,
    pub notification_levels: Vec<String>,
}

impl DiscordConfig {
    /// secure_config.json에서 Discord 설정만 로드
    pub async fn load(app_data_dir: &Path) -> Self {
        let secure = load_secure_config(app_data_dir).await;
        Self {
            bot_token: non_empty(secure.discord_bot_token),
            channel_id: non_empty(secure.discord_channel_id),
            notification_levels: if secure.notification_levels.is_empty() {
                vec!["CRITICAL".into(), "ERROR".into(), "TRADE".into()]
            } else {
                secure.notification_levels
            },
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// secure_config.json 스키마 (Discord 읽기 전용)
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
struct SecureConfig {
    discord_bot_token: Option<String>,
    discord_channel_id: Option<String>,
    #[serde(default)]
    notification_levels: Vec<String>,
}

async fn load_secure_config(app_data_dir: &Path) -> SecureConfig {
    let candidates = [
        app_data_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("secure_config.json")),
        Some(
            std::env::current_dir()
                .unwrap_or_default()
                .join("secure_config.json"),
        ),
    ];

    for path_opt in &candidates {
        if let Some(path) = path_opt {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path).await {
                    if let Ok(cfg) = serde_json::from_str::<SecureConfig>(&content) {
                        tracing::info!("secure_config.json(Discord) 로드: {:?}", path);
                        return cfg;
                    }
                }
            }
        }
    }
    SecureConfig::default()
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}
