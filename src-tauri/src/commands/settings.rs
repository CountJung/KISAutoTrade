use super::*;

// ────────────────────────────────────────────────────────────────────
// 데이터 갱신 주기 설정
// ────────────────────────────────────────────────────────────────────

/// 데이터 갱신 주기 설정 — UI에서 변경 가능, JSON 영구 저장
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshConfig {
    /// 갱신 주기(초), 기본 30, 최소 5, 최대 3600
    pub interval_sec: u64,
}

impl Default for RefreshConfig {
    fn default() -> Self {
        Self { interval_sec: 30 }
    }
}

impl RefreshConfig {
    /// .env 파일에서 REFRESH_INTERVAL_SEC 읽기 — 없으면 env_fallback 값 사용
    pub fn load_from_env(env_fallback: u64) -> Self {
        let env_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(".env");
        let interval_sec = std::fs::read_to_string(&env_path)
            .unwrap_or_default()
            .lines()
            .find(|l| l.starts_with("REFRESH_INTERVAL_SEC="))
            .and_then(|l| l["REFRESH_INTERVAL_SEC=".len()..].parse::<u64>().ok())
            .unwrap_or(env_fallback)
            .clamp(5, 3600);
        Self { interval_sec }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExchangeRateSource {
    Toss,
    ExternalPublic,
    CachedFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRateView {
    pub rate: f64,
    pub source: ExchangeRateSource,
    pub fallback_used: bool,
    pub base_currency: String,
    pub quote_currency: String,
    pub rate_text: String,
    pub mid_rate: Option<String>,
    pub basis_point: Option<String>,
    pub rate_change_type: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub updated_at: String,
    pub message: String,
}

impl ExchangeRateView {
    pub(crate) fn default_krw() -> Self {
        Self::cached_fallback(1450.0, "앱 기본 USD/KRW 환율 1450원을 사용합니다.".into())
    }

    fn cached_fallback(rate: f64, message: String) -> Self {
        Self {
            rate,
            source: ExchangeRateSource::CachedFallback,
            fallback_used: true,
            base_currency: "USD".into(),
            quote_currency: "KRW".into(),
            rate_text: format!("{rate:.4}"),
            mid_rate: None,
            basis_point: None,
            rate_change_type: None,
            valid_from: None,
            valid_until: None,
            updated_at: chrono::Utc::now().to_rfc3339(),
            message,
        }
    }

    fn external_public(rate: f64) -> Self {
        Self {
            rate,
            source: ExchangeRateSource::ExternalPublic,
            fallback_used: false,
            base_currency: "USD".into(),
            quote_currency: "KRW".into(),
            rate_text: format!("{rate:.4}"),
            mid_rate: None,
            basis_point: None,
            rate_change_type: None,
            valid_from: None,
            valid_until: None,
            updated_at: chrono::Utc::now().to_rfc3339(),
            message: "공개 환율 API(open.er-api.com) USD/KRW 캐시입니다.".into(),
        }
    }

    fn toss(value: TossExchangeRateResponse) -> anyhow::Result<Self> {
        let rate = value.rate_as_f64()?;
        Ok(Self {
            rate,
            source: ExchangeRateSource::Toss,
            fallback_used: false,
            base_currency: value.base_currency,
            quote_currency: value.quote_currency,
            rate_text: value.rate,
            mid_rate: Some(value.mid_rate),
            basis_point: Some(value.basis_point),
            rate_change_type: Some(value.rate_change_type),
            valid_from: Some(value.valid_from),
            valid_until: Some(value.valid_until),
            updated_at: chrono::Utc::now().to_rfc3339(),
            message: "토스증권 exchange-rate USD/KRW 참고 환율입니다.".into(),
        })
    }
}

// ────────────────────────────────────────────────────────────────────
// 앱 설정 조회 (민감 정보 마스킹)
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AppConfigView {
    pub active_broker_id: BrokerId,
    pub active_broker_account_id: Option<String>,
    pub kis_app_key_masked: String,
    pub kis_account_no: String,
    pub kis_is_paper_trading: bool,
    pub kis_configured: bool,
    pub active_broker_configured: bool,
    pub discord_enabled: bool,
    pub notification_levels: Vec<String>,
    pub active_profile_id: Option<String>,
    pub active_profile_name: Option<String>,
}

#[tauri::command]
pub async fn get_app_config(state: State<'_, AppState>) -> CmdResult<AppConfigView> {
    let cfg = state.config.read().await.clone();
    let masked_key = if cfg.kis_app_key.len() > 6 {
        format!("{}****", &cfg.kis_app_key[..6])
    } else if cfg.kis_app_key.is_empty() {
        "(미설정)".into()
    } else {
        "****".into()
    };

    let (active_id, active_name, active_broker_id, active_account_id) = {
        let profiles = state.profiles.read().await;
        match profiles.get_active() {
            Some(p) => (
                Some(p.id.clone()),
                Some(p.name.clone()),
                p.broker_id,
                Some(p.broker_account_id()),
            ),
            None => (None, None, cfg.broker_id, None),
        }
    };

    Ok(AppConfigView {
        active_broker_id,
        active_broker_account_id: active_account_id,
        kis_app_key_masked: masked_key,
        kis_account_no: cfg.kis_account_no.clone(),
        kis_is_paper_trading: cfg.kis_is_paper_trading,
        kis_configured: cfg.is_kis_configured(),
        active_broker_configured: cfg.is_active_broker_configured(),
        discord_enabled: cfg.discord_bot_token.is_some(),
        notification_levels: cfg.notification_levels.clone(),
        active_profile_id: active_id,
        active_profile_name: active_name,
    })
}

// ────────────────────────────────────────────────────────────────────
// 진단 모드 — 설정 상태 점검
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ConfigDiagnostic {
    pub broker_id: BrokerId,
    pub broker_account_id: Option<String>,
    pub real_key_set: bool,
    pub real_account_set: bool,
    pub paper_key_set: bool,
    pub active_mode: String,
    pub is_ready: bool,
    pub discord_configured: bool,
    pub base_url: String,
    pub issues: Vec<String>,
}

#[tauri::command]
pub async fn check_config(state: State<'_, AppState>) -> CmdResult<ConfigDiagnostic> {
    let cfg = state.config.read().await.clone();
    let mut issues = Vec::new();

    match cfg.broker_id {
        BrokerId::Kis => {
            if cfg.kis_app_key.is_empty() {
                issues.push(
                    "KIS APP KEY가 설정되지 않았습니다. Settings에서 계좌 프로파일을 추가하세요."
                        .into(),
                );
            }
            if cfg.kis_app_secret.is_empty() {
                issues.push("KIS APP SECRET이 설정되지 않았습니다.".into());
            }
            if cfg.kis_account_no.is_empty() {
                issues.push("KIS 계좌번호가 설정되지 않았습니다.".into());
            }
        }
        BrokerId::Toss => {
            if cfg.kis_app_key.is_empty() {
                issues.push("토스증권 Client ID가 설정되지 않았습니다.".into());
            }
            if cfg.kis_app_secret.is_empty() {
                issues.push("토스증권 Client Secret이 설정되지 않았습니다.".into());
            }
            if cfg.broker_account_id.is_empty() {
                issues.push("토스증권 accountSeq가 설정되지 않았습니다.".into());
            }
        }
    }

    let profiles = state.profiles.read().await;
    let active_profile = profiles.get_active();
    let paper_available = profiles
        .profiles
        .iter()
        .any(|p| p.broker_id == BrokerId::Kis && p.is_paper_trading && p.is_configured());

    Ok(ConfigDiagnostic {
        broker_id: active_profile.map(|p| p.broker_id).unwrap_or(cfg.broker_id),
        broker_account_id: active_profile.map(|p| p.broker_account_id()),
        real_key_set: !cfg.kis_app_key.is_empty(),
        real_account_set: !cfg.kis_account_no.is_empty(),
        paper_key_set: paper_available,
        active_mode: match cfg.broker_id {
            BrokerId::Kis if cfg.kis_is_paper_trading => "모의투자".into(),
            BrokerId::Kis => "실전투자".into(),
            BrokerId::Toss => "실전투자".into(),
        },
        is_ready: cfg.is_active_broker_configured(),
        discord_configured: cfg.discord_bot_token.is_some(),
        base_url: cfg.kis_base_url().to_string(),
        issues,
    })
}

// ────────────────────────────────────────────────────────────────────
// 환율 조회
// ────────────────────────────────────────────────────────────────────

/// 현재 USD/KRW 환율 조회 (캐시값 반환 — REFRESH_INTERVAL_SEC마다 자동 갱신)
#[tauri::command]
pub async fn get_exchange_rate(state: State<'_, AppState>) -> CmdResult<f64> {
    Ok(*state.exchange_rate_krw.read().await)
}

/// 현재 USD/KRW 환율 조회 정책과 출처/유효시간 메타데이터
#[tauri::command]
pub async fn get_exchange_rate_status(state: State<'_, AppState>) -> CmdResult<ExchangeRateView> {
    Ok(state.exchange_rate_status.read().await.clone())
}

pub async fn refresh_exchange_rate_status(
    profiles: &Arc<RwLock<ProfilesConfig>>,
    exchange_rate_krw: &Arc<RwLock<f64>>,
    exchange_rate_status: &Arc<RwLock<ExchangeRateView>>,
) -> ExchangeRateView {
    let cached_rate = *exchange_rate_krw.read().await;
    let view = resolve_exchange_rate_policy(profiles, cached_rate).await;
    *exchange_rate_krw.write().await = view.rate;
    *exchange_rate_status.write().await = view.clone();
    view
}

async fn resolve_exchange_rate_policy(
    profiles: &Arc<RwLock<ProfilesConfig>>,
    cached_rate: f64,
) -> ExchangeRateView {
    let active_profile = {
        let profiles = profiles.read().await;
        profiles.get_active().cloned()
    };

    let mut toss_error: Option<String> = None;
    if let Some(profile) = active_profile.as_ref() {
        if profile.broker_id == BrokerId::Toss && profile.is_configured() {
            let adapter = TossBrokerAdapter::with_credentials(
                TossBrokerAdapter::DEFAULT_BASE_URL,
                profile.app_key.clone(),
                profile.app_secret.clone(),
                Some(profile.account_no.clone()),
            );
            match adapter
                .get_exchange_rate(BrokerCurrency::Usd, BrokerCurrency::Krw)
                .await
            {
                Ok(rate) => match ExchangeRateView::toss(rate) {
                    Ok(view) => return view,
                    Err(e) => toss_error = Some(e.to_string()),
                },
                Err(e) => toss_error = Some(e.to_string()),
            }
        }
    }

    match crate::api::rest::fetch_usd_krw_rate().await {
        Ok(rate) => {
            let mut view = ExchangeRateView::external_public(rate);
            if let Some(error) = toss_error {
                view.fallback_used = true;
                view.message =
                    format!("Toss exchange-rate 조회 실패로 공개 환율 API를 사용합니다: {error}");
            }
            view
        }
        Err(external_error) => {
            let message = match toss_error {
                Some(toss_error) => format!(
                    "Toss exchange-rate와 공개 환율 API가 모두 실패해 마지막 캐시를 유지합니다: Toss={toss_error}; external={external_error}"
                ),
                None => format!(
                    "공개 환율 API 조회 실패로 마지막 캐시를 유지합니다: {external_error}"
                ),
            };
            ExchangeRateView::cached_fallback(cached_rate, message)
        }
    }
}

/// 데이터 갱신 주기 조회 (초) — refresh_config.interval_sec
#[tauri::command]
pub async fn get_refresh_interval(state: State<'_, AppState>) -> CmdResult<u64> {
    Ok(state.refresh_config.read().await.interval_sec)
}

/// 데이터 갱신 주기 설정 전체 조회
#[tauri::command]
pub async fn get_refresh_config(state: State<'_, AppState>) -> CmdResult<RefreshConfig> {
    Ok(state.refresh_config.read().await.clone())
}

/// 데이터 갱신 주기 변경 — .env 영구 저장 + 백그라운드 데몬 즉시 적용
#[tauri::command]
pub async fn set_refresh_config(
    interval_sec: u64,
    state: State<'_, AppState>,
) -> CmdResult<RefreshConfig> {
    use std::io::Write;
    let new_cfg = RefreshConfig {
        interval_sec: interval_sec.clamp(5, 3600),
    };
    // .env 파일에서 REFRESH_INTERVAL_SEC 줄만 교체 (save_web_config 동일 패턴)
    let env_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".env");
    let existing = std::fs::read_to_string(&env_path).unwrap_or_default();
    let mut lines: Vec<String> = existing
        .lines()
        .filter(|l| !l.starts_with("REFRESH_INTERVAL_SEC="))
        .map(String::from)
        .collect();
    lines.push(format!("REFRESH_INTERVAL_SEC={}", new_cfg.interval_sec));
    let content = lines.join("\n");
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&env_path)
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map_err(|e| CmdError {
            code: "SAVE_FAILED".into(),
            message: e.to_string(),
        })?;
    *state.refresh_config.write().await = new_cfg.clone();
    // 백그라운드 데몬에 새 주기 즉시 전달 (슬립 취소 → 새 주기로 재시작)
    let _ = state.refresh_interval_tx.send(new_cfg.interval_sec);
    tracing::info!(
        ".env 저장 완료 — REFRESH_INTERVAL_SEC={}",
        new_cfg.interval_sec
    );
    Ok(new_cfg)
}

// ────────────────────────────────────────────────────────────────────
// 로그 설정 조회 / 변경
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_log_config(state: State<'_, AppState>) -> CmdResult<LogConfig> {
    Ok(state.log_config.read().await.clone())
}

#[derive(Debug, Deserialize)]
pub struct SetLogConfigInput {
    pub retention_days: u32,
    pub max_size_mb: u64,
    #[serde(default)]
    pub api_debug: bool,
}

#[tauri::command]
pub async fn set_log_config(
    input: SetLogConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<LogConfig> {
    let new_cfg = LogConfig {
        retention_days: input.retention_days.clamp(1, 365),
        max_size_mb: input.max_size_mb.clamp(10, 10240),
        api_debug: input.api_debug,
    };

    // AppState 업데이트
    *state.log_config.write().await = new_cfg.clone();

    // REST 클라이언트에 즉시 반영
    state
        .rest_client
        .read()
        .await
        .set_api_debug(new_cfg.api_debug);

    // 파일 저장
    new_cfg.save_sync(&state.log_dir).map_err(CmdError::from)?;

    // 즉시 정리 실행
    crate::logging::cleanup(&state.log_dir, &new_cfg);

    tracing::info!(
        "로그 설정 변경: 보관 {}일, 최대 {}MB, API 진단={}",
        new_cfg.retention_days,
        new_cfg.max_size_mb,
        new_cfg.api_debug
    );

    Ok(new_cfg)
}

// ── 업데이트 확인 ────────────────────────────────────────────────
#[tauri::command]
pub async fn check_for_update() -> CmdResult<crate::updater::UpdateInfo> {
    let client = reqwest::Client::new();
    crate::updater::check(&client)
        .await
        .map_err(|message| CmdError {
            code: "UPDATE_CHECK_FAILED".into(),
            message,
        })
}

// ── 웹 접속 설정 ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebConfig {
    pub running_port: u16,
    pub access_url: String,
}

#[tauri::command]
pub async fn get_web_config(state: State<'_, AppState>) -> CmdResult<WebConfig> {
    let port = state.web_port;
    Ok(WebConfig {
        running_port: port,
        access_url: format!("http://localhost:{}", port),
    })
}

#[tauri::command]
pub async fn save_web_config(new_port: u16) -> CmdResult<String> {
    use std::io::Write;
    if !(1024..=65535).contains(&new_port) {
        return Err(CmdError {
            code: "INVALID_PORT".into(),
            message: "포트는 1024~65535 사이여야 합니다".into(),
        });
    }
    let env_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".env");
    // 기존 .env 읽어서 WEB_PORT 줄만 교체
    let existing = std::fs::read_to_string(&env_path).unwrap_or_default();
    let mut lines: Vec<String> = existing
        .lines()
        .filter(|l| !l.starts_with("WEB_PORT="))
        .map(String::from)
        .collect();
    lines.push(format!("WEB_PORT={}", new_port));
    let content = lines.join("\n");
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&env_path)
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map_err(|e| CmdError {
            code: "SAVE_FAILED".into(),
            message: e.to_string(),
        })?;
    tracing::info!(".env 저장 완료 — WEB_PORT={}", new_port);
    Ok(format!(".env 저장 완료: WEB_PORT={}", new_port))
}

// ────────────────────────────────────────────────────────────────────
// 실전/모의투자 자동 감지
// ────────────────────────────────────────────────────────────────────

/// 자동 감지 결과
#[derive(Debug, Serialize)]
pub struct DetectTradingTypeResult {
    /// true = 모의투자, false = 실전투자
    pub is_paper_trading: bool,
    pub message: String,
}

/// APP KEY + APP SECRET으로 실전/모의투자 여부를 자동 감지합니다.
///
/// 실전 URL → 모의 URL 순서로 토큰 발급을 시도하여
/// `access_token` 또는 도메인/앱키 불일치 오류 메시지를 기준으로 판별합니다.
#[tauri::command]
pub async fn detect_trading_type(
    app_key: String,
    app_secret: String,
) -> CmdResult<DetectTradingTypeResult> {
    if app_key.trim().is_empty() || app_secret.trim().is_empty() {
        return Err(CmdError {
            code: "INVALID_INPUT".into(),
            message: "APP KEY와 APP SECRET을 모두 입력하세요.".into(),
        });
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| CmdError {
            code: "CLIENT_BUILD".into(),
            message: e.to_string(),
        })?;

    let detected =
        crate::api::detect::detect_trading_type(&client, app_key.trim(), app_secret.trim())
            .await
            .map_err(|e| CmdError {
                code: "DETECT_FAILED".into(),
                message: e.to_string(),
            })?;

    tracing::info!(
        "자동 감지 완료: {}",
        if detected.is_paper() {
            "모의투자 키"
        } else {
            "실전투자 키"
        }
    );
    Ok(DetectTradingTypeResult {
        is_paper_trading: detected.is_paper(),
        message: detected.message().into(),
    })
}

// ────────────────────────────────────────────────────────────────────
// 기존 프로파일의 실전/모의 자동 감지 + 즉시 저장
// ────────────────────────────────────────────────────────────────────

/// 저장된 프로파일의 실제 키로 실전/모의 여부를 감지하고 자동으로 업데이트합니다.
///
/// detect_trading_type 과 달리 키를 UI로 전달할 필요 없이
/// profile_id 하나로 백엔드가 직접 저장된 키를 읽어 판별합니다.
#[tauri::command]
pub async fn detect_profile_trading_type(
    profile_id: String,
    state: State<'_, AppState>,
) -> CmdResult<ProfileView> {
    // 1) 해당 프로파일의 키 복사 (read lock 빠르게 해제)
    let (app_key, app_secret) = {
        let profiles = state.profiles.read().await;
        let p = profiles
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .ok_or_else(|| CmdError {
                code: "PROFILE_NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {}", profile_id),
            })?;
        if p.app_key.is_empty() || p.app_secret.is_empty() {
            return Err(CmdError {
                code: "KEY_NOT_SET".into(),
                message: "APP KEY 또는 APP SECRET이 설정되지 않았습니다.".into(),
            });
        }
        (p.app_key.clone(), p.app_secret.clone())
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| CmdError {
            code: "CLIENT_BUILD".into(),
            message: e.to_string(),
        })?;

    let is_paper = crate::api::detect::detect_trading_type(&client, &app_key, &app_secret)
        .await
        .map_err(|e| CmdError {
            code: "DETECT_FAILED".into(),
            message: e.to_string(),
        })?
        .is_paper();

    // 3) 프로파일 업데이트 및 저장
    let view = {
        let mut profiles = state.profiles.write().await;
        let updated = profiles
            .update(
                &profile_id,
                None,
                None,
                Some(is_paper),
                None,
                None,
                None,
                None,
            )
            .ok_or_else(|| CmdError {
                code: "PROFILE_NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {}", profile_id),
            })?;
        profile_to_view(&updated, &profiles.active_id)
    };

    // 4) 해당 프로파일이 활성 프로파일이면 런타임 config도 갱신
    let is_active = {
        let profiles = state.profiles.read().await;
        profiles.active_id.as_deref() == Some(&profile_id)
    };
    if is_active {
        apply_active_profile(&state).await?;
    }

    save_profiles(&state).await?;

    tracing::info!(
        "프로파일 '{}' 감지 완료: {}",
        view.name,
        if is_paper {
            "모의투자"
        } else {
            "실전투자"
        }
    );
    Ok(view)
}
