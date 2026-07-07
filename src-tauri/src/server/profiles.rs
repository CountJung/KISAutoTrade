use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use super::ServerState;
use crate::api::rest::KisRestClient;
use crate::api::token::TokenManager;
use crate::broker::BrokerId;
use crate::commands::profile_to_view;
use crate::config::{AccountProfile, AppConfig};

/// GET /api/profiles
pub(super) async fn profiles_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let profiles = s.profiles.read().await;
    let views: Vec<_> = profiles
        .profiles
        .iter()
        .map(|p| profile_to_view(p, &profiles.active_id))
        .collect();
    Json(serde_json::to_value(views).unwrap_or_default())
}

/// 활성 프로파일 변경 시 config + rest_client 갱신 (웹 서버 내부용)
async fn apply_profile_change(s: &ServerState) {
    let new_config = {
        let profiles = s.profiles.read().await;
        let existing = s.config.read().await.clone();
        match profiles.get_active() {
            Some(p) => Arc::new(AppConfig {
                broker_id: p.broker_id,
                broker_account_id: p.broker_account_id(),
                kis_app_key: p.app_key.clone(),
                kis_app_secret: p.app_secret.clone(),
                kis_account_no: p.account_no.clone(),
                kis_is_paper_trading: p.is_paper_trading,
                discord_bot_token: existing.discord_bot_token.clone(),
                discord_channel_id: existing.discord_channel_id.clone(),
                notification_levels: existing.notification_levels.clone(),
            }),
            None => Arc::new(AppConfig {
                broker_id: BrokerId::Kis,
                broker_account_id: String::new(),
                kis_app_key: String::new(),
                kis_app_secret: String::new(),
                kis_account_no: String::new(),
                kis_is_paper_trading: false,
                discord_bot_token: existing.discord_bot_token.clone(),
                discord_channel_id: existing.discord_channel_id.clone(),
                notification_levels: existing.notification_levels.clone(),
            }),
        }
    };
    let token_mgr = Arc::new(RwLock::new(TokenManager::new(Arc::clone(&new_config))));
    let new_client = Arc::new(KisRestClient::new(
        new_config.kis_base_url().to_string(),
        new_config.kis_app_key.clone(),
        new_config.kis_app_secret.clone(),
        new_config.kis_account_no.clone(),
        new_config.kis_is_paper_trading,
        token_mgr,
    ));
    *s.config.write().await = new_config;
    *s.rest_client.write().await = new_client;
}

/// profiles.json 저장 (웹 서버 내부용)
async fn save_profiles_server(s: &ServerState) {
    let profiles = s.profiles.read().await.clone();
    if let Err(e) = profiles.save(&s.profiles_path).await {
        tracing::warn!("프로파일 저장 실패 (웹 API): {}", e);
    }
}

#[derive(Deserialize)]
pub(super) struct AddProfileBody {
    #[serde(default = "default_body_broker_id", alias = "brokerId")]
    broker_id: BrokerId,
    name: String,
    #[serde(alias = "isPaperTrading")]
    is_paper_trading: bool,
    #[serde(default, alias = "liveTradingConsent")]
    live_trading_consent: bool,
    #[serde(alias = "appKey")]
    app_key: String,
    #[serde(alias = "appSecret")]
    app_secret: String,
    #[serde(alias = "accountNo")]
    account_no: String,
}

fn default_body_broker_id() -> BrokerId {
    BrokerId::Kis
}

/// POST /api/profiles/add
pub(super) async fn add_profile_handler(
    State(s): State<ServerState>,
    Json(body): Json<AddProfileBody>,
) -> Json<serde_json::Value> {
    let profile = AccountProfile::new(
        body.name,
        body.is_paper_trading,
        body.app_key,
        body.app_secret,
        body.account_no,
    );
    let profile = AccountProfile {
        broker_id: body.broker_id,
        live_trading_consent: body.live_trading_consent,
        ..profile
    };
    let (view, is_first) = {
        let mut profiles = s.profiles.write().await;
        let was_empty = profiles.profiles.is_empty();
        let added = profiles.add(profile);
        let view = profile_to_view(&added, &profiles.active_id);
        (view, was_empty)
    };
    if is_first {
        apply_profile_change(&s).await;
    }
    save_profiles_server(&s).await;
    Json(serde_json::to_value(view).unwrap_or_default())
}

#[derive(Deserialize)]
pub(super) struct UpdateProfileBody {
    id: String,
    #[serde(alias = "brokerId")]
    broker_id: Option<BrokerId>,
    name: Option<String>,
    #[serde(alias = "isPaperTrading")]
    is_paper_trading: Option<bool>,
    #[serde(alias = "liveTradingConsent")]
    live_trading_consent: Option<bool>,
    #[serde(alias = "appKey")]
    app_key: Option<String>,
    #[serde(alias = "appSecret")]
    app_secret: Option<String>,
    #[serde(alias = "accountNo")]
    account_no: Option<String>,
}

/// POST /api/profiles/update
pub(super) async fn update_profile_handler(
    State(s): State<ServerState>,
    Json(body): Json<UpdateProfileBody>,
) -> Json<serde_json::Value> {
    let (view, is_active) = {
        let mut profiles = s.profiles.write().await;
        match profiles.update(
            &body.id,
            body.broker_id,
            body.name,
            body.is_paper_trading,
            body.live_trading_consent,
            body.app_key,
            body.app_secret,
            body.account_no,
        ) {
            Some(p) => {
                let active = profiles.active_id.as_deref() == Some(&body.id);
                (profile_to_view(&p, &profiles.active_id), active)
            }
            None => {
                return Json(
                    serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", body.id) }),
                )
            }
        }
    };
    if is_active {
        apply_profile_change(&s).await;
    }
    save_profiles_server(&s).await;
    Json(serde_json::to_value(view).unwrap_or_default())
}

#[derive(Deserialize)]
pub(super) struct DeleteProfileBody {
    id: String,
}

/// POST /api/profiles/delete
pub(super) async fn delete_profile_handler(
    State(s): State<ServerState>,
    Json(body): Json<DeleteProfileBody>,
) -> Json<serde_json::Value> {
    let deleted = {
        let mut profiles = s.profiles.write().await;
        profiles.delete(&body.id)
    };
    if !deleted {
        return Json(
            serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", body.id) }),
        );
    }
    apply_profile_change(&s).await;
    save_profiles_server(&s).await;
    Json(serde_json::json!({ "ok": true }))
}

/// POST /api/profiles/:id/set-active
pub(super) async fn set_active_profile_handler(
    State(s): State<ServerState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let ok = {
        let mut profiles = s.profiles.write().await;
        profiles.set_active(&id)
    };
    if !ok {
        return Json(
            serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", id) }),
        );
    }
    if !*s.is_trading.lock().await {
        apply_profile_change(&s).await;
    }
    save_profiles_server(&s).await;
    super::app_config_handler(State(s)).await
}

#[derive(Deserialize)]
pub(super) struct DetectTradingTypeBody {
    #[serde(alias = "appKey")]
    app_key: String,
    #[serde(alias = "appSecret")]
    app_secret: String,
}

#[derive(Deserialize)]
pub(super) struct TossAccountsBody {
    #[serde(alias = "clientId", alias = "appKey")]
    client_id: String,
    #[serde(alias = "clientSecret", alias = "appSecret")]
    client_secret: String,
}

/// POST /api/detect-trading-type
pub(super) async fn detect_trading_type_handler(
    State(_s): State<ServerState>,
    Json(body): Json<DetectTradingTypeBody>,
) -> Json<serde_json::Value> {
    if body.app_key.trim().is_empty() || body.app_secret.trim().is_empty() {
        return Json(serde_json::json!({ "error": "APP KEY와 APP SECRET을 모두 입력하세요." }));
    }
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };
    match crate::api::detect::detect_trading_type(
        &client,
        body.app_key.trim(),
        body.app_secret.trim(),
    )
    .await
    {
        Ok(detected) => Json(serde_json::json!({
            "is_paper_trading": detected.is_paper(),
            "message": detected.message(),
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// POST /api/profiles/:id/detect
pub(super) async fn detect_profile_handler(
    State(s): State<ServerState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let (app_key, app_secret) = {
        let profiles = s.profiles.read().await;
        match profiles.profiles.iter().find(|p| p.id == id) {
            Some(p) if p.broker_id != BrokerId::Kis => {
                return Json(
                    serde_json::json!({ "error": "실전/모의 자동 감지는 KIS 프로파일에서만 사용할 수 있습니다. Toss 프로파일은 연결 진단을 사용하세요." }),
                )
            }
            Some(p) if !p.app_key.is_empty() && !p.app_secret.is_empty() => {
                (p.app_key.clone(), p.app_secret.clone())
            }
            Some(_) => {
                return Json(
                    serde_json::json!({ "error": "APP KEY 또는 APP SECRET이 설정되지 않았습니다." }),
                )
            }
            None => {
                return Json(
                    serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", id) }),
                )
            }
        }
    };
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };
    let is_paper =
        match crate::api::detect::detect_trading_type(&client, &app_key, &app_secret).await {
            Ok(detected) => detected.is_paper(),
            Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
        };
    let view = {
        let mut profiles = s.profiles.write().await;
        match profiles.update(&id, None, None, Some(is_paper), None, None, None, None) {
            Some(p) => profile_to_view(&p, &profiles.active_id),
            None => {
                return Json(
                    serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", id) }),
                )
            }
        }
    };
    let is_active = s.profiles.read().await.active_id.as_deref() == Some(&id);
    if is_active {
        apply_profile_change(&s).await;
    }
    save_profiles_server(&s).await;
    Json(serde_json::to_value(view).unwrap_or_default())
}

/// POST /api/toss-accounts
pub(super) async fn toss_accounts_handler(
    State(_s): State<ServerState>,
    Json(body): Json<TossAccountsBody>,
) -> Json<serde_json::Value> {
    match crate::commands::lookup_toss_accounts_with_credentials(body.client_id, body.client_secret)
        .await
    {
        Ok(accounts) => Json(serde_json::to_value(accounts).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.message })),
    }
}

/// POST /api/profiles/:id/toss-accounts
pub(super) async fn toss_profile_accounts_handler(
    State(s): State<ServerState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let profile = {
        let profiles = s.profiles.read().await;
        match profiles.profiles.iter().find(|p| p.id == id) {
            Some(p) => p.clone(),
            None => {
                return Json(
                    serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", id) }),
                )
            }
        }
    };

    if profile.broker_id != BrokerId::Toss {
        return Json(
            serde_json::json!({ "error": "토스증권 프로파일만 accountSeq 목록을 조회할 수 있습니다." }),
        );
    }

    match crate::commands::lookup_toss_accounts_with_credentials(
        profile.app_key,
        profile.app_secret,
    )
    .await
    {
        Ok(accounts) => Json(serde_json::to_value(accounts).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.message })),
    }
}

/// POST /api/profiles/:id/toss-diagnostic
pub(super) async fn toss_profile_diagnostic_handler(
    State(s): State<ServerState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let profile = {
        let profiles = s.profiles.read().await;
        match profiles.profiles.iter().find(|p| p.id == id) {
            Some(p) => p.clone(),
            None => {
                return Json(
                    serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", id) }),
                )
            }
        }
    };

    if profile.broker_id != BrokerId::Toss {
        return Json(
            serde_json::json!({ "error": "토스증권 프로파일만 Toss 연결 진단을 실행할 수 있습니다." }),
        );
    }

    let diagnostic = crate::commands::run_toss_connection_diagnostic(profile).await;
    Json(serde_json::to_value(diagnostic).unwrap_or_else(
        |e| serde_json::json!({ "error": format!("Toss 연결 진단 결과 직렬화 실패: {}", e) }),
    ))
}
