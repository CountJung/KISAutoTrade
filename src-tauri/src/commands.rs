/// Tauri IPC 커맨드 모음
///
/// Frontend(React) ↔ Backend(Rust) 통신 인터페이스
/// 모든 커맨드는 AppState를 통해 공유 리소스에 접근합니다.
use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::{Mutex, RwLock};

use crate::{
    api::{
        rest::{BalanceItem, BalanceSummary, ChartCandle, ExecutedOrder, KisRestClient, OrderRequest, OrderResponse, PriceResponse, StockSearchItem},
        token::TokenManager,
    },
    config::{AccountProfile, AppConfig, DiscordConfig, ProfilesConfig},
    logging::LogConfig,
    notifications::{discord::DiscordNotifier, types::NotificationEvent},
    storage::{stats_store::DailyStats, trade_store::TradeRecord, StatsStore, TradeStore},
    trading::{
        position::{Position, PositionTracker},
        strategy::{MaCrossParams, MovingAverageCrossStrategy, StrategyConfig, StrategyManager},
    },
};

// ────────────────────────────────────────────────────────────────────
// AppState — Tauri manage() 로 등록
// ────────────────────────────────────────────────────────────────────

pub struct AppState {
    /// 현재 활성 설정 (프로파일 전환 시 Arc 교체)
    pub config: Arc<RwLock<Arc<AppConfig>>>,
    /// KIS REST 클라이언트 (프로파일 전환 시 Arc 교체)
    pub rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    /// Discord 알림 (프로파일 전환 무관, 앱 수명 동안 고정)
    pub discord: Option<Arc<DiscordNotifier>>,
    pub discord_config: Arc<DiscordConfig>,
    /// 계좌 프로파일 목록
    pub profiles: Arc<RwLock<ProfilesConfig>>,
    pub profiles_path: PathBuf,
    pub trade_store: Arc<TradeStore>,
    pub stats_store: Arc<StatsStore>,
    /// 자동 매매 실행 여부
    pub is_trading: Arc<Mutex<bool>>,
    /// 전략 관리자
    pub strategy_manager: Arc<Mutex<StrategyManager>>,
    /// 포지션 트래커
    pub position_tracker: Arc<Mutex<PositionTracker>>,
    /// 로그 디렉토리 경로
    pub log_dir: PathBuf,
    /// 로그 설정 (보관 기간, 최대 용량)
    pub log_config: Arc<RwLock<LogConfig>>,
    /// 데이터 저장 경로
    pub data_dir: PathBuf,
    /// KRX 캐시된 종목 목록 (이름 검색용)
    pub stock_list: Arc<RwLock<Vec<crate::api::rest::StockSearchItem>>>,
    /// 웹 서버 포트
    pub web_port: u16,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        discord_config: Arc<DiscordConfig>,
        profiles: ProfilesConfig,
        profiles_path: PathBuf,
        data_dir: PathBuf,
        log_dir: PathBuf,
        log_config: LogConfig,
        web_port: u16,
    ) -> Self {
        let rest_client = make_rest_client(&config);

        let discord = match (&discord_config.bot_token, &discord_config.channel_id) {
            (Some(token), Some(channel)) if !token.is_empty() && !channel.is_empty() => {
                Some(Arc::new(DiscordNotifier::new(token.clone(), channel.clone())))
            }
            _ => None,
        };

        let trade_store = Arc::new(TradeStore::new(data_dir.clone()));
        let stats_store = Arc::new(StatsStore::new(data_dir.clone()));

        // 기본 MA 크로스 전략 등록
        let mut strategy_manager = StrategyManager::new();
        let default_strategy = StrategyConfig {
            id: "ma_cross_default".to_string(),
            name: "이동평균 교차 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(MaCrossParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(MovingAverageCrossStrategy::new(default_strategy)));

        Self {
            config: Arc::new(RwLock::new(config)),
            rest_client: Arc::new(RwLock::new(rest_client)),
            discord,
            discord_config,
            profiles: Arc::new(RwLock::new(profiles)),
            profiles_path,
            trade_store,
            stats_store,
            is_trading: Arc::new(Mutex::new(false)),
            strategy_manager: Arc::new(Mutex::new(strategy_manager)),
            position_tracker: Arc::new(Mutex::new(PositionTracker::new())),
            log_dir,
            log_config: Arc::new(RwLock::new(log_config)),
            data_dir,
            stock_list: Arc::new(RwLock::new(vec![])),
            web_port,
        }
    }
}

/// AppConfig에서 KisRestClient 생성 (초기 + 프로파일 전환 공용)
fn make_rest_client(config: &Arc<AppConfig>) -> Arc<KisRestClient> {
    let token_manager = Arc::new(RwLock::new(TokenManager::new(Arc::clone(config))));
    Arc::new(KisRestClient::new(
        config.kis_base_url().to_string(),
        config.kis_app_key.clone(),
        config.kis_app_secret.clone(),
        config.kis_account_no.clone(),
        config.kis_is_paper_trading,
        token_manager,
    ))
}

// ────────────────────────────────────────────────────────────────────
// 공통 응답 타입
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CmdError {
    pub code: String,
    pub message: String,
}

impl CmdError {
    fn from(e: anyhow::Error) -> Self {
        Self {
            code: "ERROR".into(),
            message: e.to_string(),
        }
    }
}

type CmdResult<T> = Result<T, CmdError>;

// ────────────────────────────────────────────────────────────────────
// 앱 설정 조회 (민감 정보 마스킹)
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AppConfigView {
    pub kis_app_key_masked: String,
    pub kis_account_no: String,
    pub kis_is_paper_trading: bool,
    pub kis_configured: bool,
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

    let (active_id, active_name) = {
        let profiles = state.profiles.read().await;
        match profiles.get_active() {
            Some(p) => (Some(p.id.clone()), Some(p.name.clone())),
            None => (None, None),
        }
    };

    Ok(AppConfigView {
        kis_app_key_masked: masked_key,
        kis_account_no: cfg.kis_account_no.clone(),
        kis_is_paper_trading: cfg.kis_is_paper_trading,
        kis_configured: cfg.is_kis_configured(),
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

    if cfg.kis_app_key.is_empty() {
        issues.push("KIS APP KEY가 설정되지 않았습니다. Settings에서 계좌 프로파일을 추가하세요.".into());
    }
    if cfg.kis_app_secret.is_empty() {
        issues.push("KIS APP SECRET이 설정되지 않았습니다.".into());
    }
    if cfg.kis_account_no.is_empty() {
        issues.push("KIS 계좌번호가 설정되지 않았습니다.".into());
    }

    let profiles = state.profiles.read().await;
    let paper_available = profiles.profiles.iter().any(|p| p.is_paper_trading && p.is_configured());

    Ok(ConfigDiagnostic {
        real_key_set: !cfg.kis_app_key.is_empty(),
        real_account_set: !cfg.kis_account_no.is_empty(),
        paper_key_set: paper_available,
        active_mode: if cfg.kis_is_paper_trading { "모의투자".into() } else { "실전투자".into() },
        is_ready: cfg.is_kis_configured(),
        discord_configured: cfg.discord_bot_token.is_some(),
        base_url: cfg.kis_base_url().to_string(),
        issues,
    })
}

// ────────────────────────────────────────────────────────────────────
// 계좌 프로파일 관리
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProfileView {
    pub id: String,
    pub name: String,
    pub is_paper_trading: bool,
    pub app_key_masked: String,
    pub account_no: String,
    pub is_active: bool,
    pub is_configured: bool,
}

fn profile_to_view(p: &AccountProfile, active_id: &Option<String>) -> ProfileView {
    let masked = if p.app_key.len() > 6 {
        format!("{}****", &p.app_key[..6])
    } else if p.app_key.is_empty() {
        "(미설정)".into()
    } else {
        "****".into()
    };
    ProfileView {
        id: p.id.clone(),
        name: p.name.clone(),
        is_paper_trading: p.is_paper_trading,
        app_key_masked: masked,
        account_no: p.account_no.clone(),
        is_active: active_id.as_deref() == Some(&p.id),
        is_configured: p.is_configured(),
    }
}

#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> CmdResult<Vec<ProfileView>> {
    let profiles = state.profiles.read().await;
    Ok(profiles
        .profiles
        .iter()
        .map(|p| profile_to_view(p, &profiles.active_id))
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct AddProfileInput {
    pub name: String,
    pub is_paper_trading: bool,
    pub app_key: String,
    pub app_secret: String,
    pub account_no: String,
}

#[tauri::command]
pub async fn add_profile(
    input: AddProfileInput,
    state: State<'_, AppState>,
) -> CmdResult<ProfileView> {
    let profile = AccountProfile::new(
        input.name,
        input.is_paper_trading,
        input.app_key,
        input.app_secret,
        input.account_no,
    );

    let (view, is_first) = {
        let mut profiles = state.profiles.write().await;
        let was_empty = profiles.profiles.is_empty();
        let added = profiles.add(profile);
        let view = profile_to_view(&added, &profiles.active_id);
        (view, was_empty)
    };

    // 첫 번째 프로파일이면 자동 활성화
    if is_first {
        apply_active_profile(&state).await?;
    }

    save_profiles(&state).await?;
    Ok(view)
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileInput {
    pub id: String,
    pub name: Option<String>,
    pub is_paper_trading: Option<bool>,
    /// 빈 문자열 = 변경 안 함
    pub app_key: Option<String>,
    /// 빈 문자열 = 변경 안 함
    pub app_secret: Option<String>,
    pub account_no: Option<String>,
}

#[tauri::command]
pub async fn update_profile(
    input: UpdateProfileInput,
    state: State<'_, AppState>,
) -> CmdResult<ProfileView> {
    let view = {
        let mut profiles = state.profiles.write().await;
        let updated = profiles
            .update(
                &input.id,
                input.name,
                input.is_paper_trading,
                input.app_key,
                input.app_secret,
                input.account_no,
            )
            .ok_or_else(|| CmdError {
                code: "PROFILE_NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {}", input.id),
            })?;
        profile_to_view(&updated, &profiles.active_id)
    };

    // 수정된 프로파일이 현재 활성이면 즉시 반영
    let is_active = {
        let profiles = state.profiles.read().await;
        profiles.active_id.as_deref() == Some(&input.id)
    };
    if is_active {
        apply_active_profile(&state).await?;
    }

    save_profiles(&state).await?;
    Ok(view)
}

#[tauri::command]
pub async fn delete_profile(
    id: String,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    let deleted = {
        let mut profiles = state.profiles.write().await;
        profiles.delete(&id)
    };

    if !deleted {
        return Err(CmdError {
            code: "PROFILE_NOT_FOUND".into(),
            message: format!("프로파일을 찾을 수 없습니다: {}", id),
        });
    }

    apply_active_profile(&state).await?;
    save_profiles(&state).await?;
    Ok(())
}

#[tauri::command]
pub async fn set_active_profile(
    id: String,
    state: State<'_, AppState>,
) -> CmdResult<AppConfigView> {
    let ok = {
        let mut profiles = state.profiles.write().await;
        profiles.set_active(&id)
    };

    if !ok {
        return Err(CmdError {
            code: "PROFILE_NOT_FOUND".into(),
            message: format!("프로파일을 찾을 수 없습니다: {}", id),
        });
    }

    apply_active_profile(&state).await?;
    save_profiles(&state).await?;
    get_app_config(state).await
}

/// 현재 active_id 기반으로 config + rest_client 교체
async fn apply_active_profile(state: &AppState) -> CmdResult<()> {
    let new_config = {
        let profiles = state.profiles.read().await;
        match profiles.get_active() {
            Some(p) => AppConfig::from_profile(p, &state.discord_config),
            None => AppConfig::empty(&state.discord_config),
        }
    };

    let new_client = make_rest_client(&new_config);

    *state.config.write().await = new_config;
    *state.rest_client.write().await = new_client;

    tracing::info!("활성 프로파일 적용 완료");
    Ok(())
}

/// profiles.json 비동기 저장
async fn save_profiles(state: &AppState) -> CmdResult<()> {
    let profiles = state.profiles.read().await.clone();
    profiles
        .save(&state.profiles_path)
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 잔고 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BalanceResult {
    pub items: Vec<BalanceItem>,
    pub summary: Option<BalanceSummary>,
}

#[tauri::command]
pub async fn get_balance(state: State<'_, AppState>) -> CmdResult<BalanceResult> {
    let client = state.rest_client.read().await.clone();
    match client.get_balance().await {
        Ok(resp) => {
            tracing::info!(
                "잔고 조회 성공: 보유종목 {}개, 총평가금액 {}원",
                resp.items.len(),
                resp.summary.as_ref().map(|s| s.tot_evlu_amt.as_str()).unwrap_or("미제공")
            );
            Ok(BalanceResult { items: resp.items, summary: resp.summary })
        }
        Err(e) => {
            tracing::error!("잔고 조회 실패: {}", e);
            Err(CmdError::from(e))
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 차트 데이터 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChartDataInput {
    pub symbol: String,
    /// "D"=일봉, "W"=주봉, "M"=월봉
    pub period_code: String,
    pub start_date: String, // YYYYMMDD
    pub end_date: String,   // YYYYMMDD
}

#[tauri::command]
pub async fn get_chart_data(
    input: ChartDataInput,
    state: State<'_, AppState>,
) -> CmdResult<Vec<ChartCandle>> {
    let client = state.rest_client.read().await.clone();
    client
        .get_chart_data(&input.symbol, &input.period_code, &input.start_date, &input.end_date)
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 현재가 조회
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_price(symbol: String, state: State<'_, AppState>) -> CmdResult<PriceResponse> {
    let client = state.rest_client.read().await.clone();
    client.get_price(&symbol).await.map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 주문
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PlaceOrderInput {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: u64,
    pub price: u64,
}

#[tauri::command]
pub async fn place_order(
    input: PlaceOrderInput,
    state: State<'_, AppState>,
) -> CmdResult<OrderResponse> {
    use crate::api::rest::{OrderSide, OrderType};

    let side = match input.side.as_str() {
        "buy" | "Buy" => OrderSide::Buy,
        "sell" | "Sell" => OrderSide::Sell,
        other => return Err(CmdError {
            code: "INVALID_SIDE".into(),
            message: format!("알 수 없는 주문 방향: {}", other),
        }),
    };

    let order_type = match input.order_type.as_str() {
        "limit" | "Limit" => OrderType::Limit,
        "market" | "Market" => OrderType::Market,
        other => return Err(CmdError {
            code: "INVALID_ORDER_TYPE".into(),
            message: format!("알 수 없는 주문 유형: {}", other),
        }),
    };

    let req = OrderRequest { symbol: input.symbol, side, order_type, quantity: input.quantity, price: input.price };
    let client = state.rest_client.read().await.clone();
    client.place_order(&req).await.map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 당일 체결 내역 (KIS 실시간)
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_executed(state: State<'_, AppState>) -> CmdResult<Vec<ExecutedOrder>> {
    let client = state.rest_client.read().await.clone();
    client.get_today_executed_orders().await.map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 로컬 체결 기록 (JSON 저장소)
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_trades(state: State<'_, AppState>) -> CmdResult<Vec<TradeRecord>> {
    let today = chrono::Local::now().date_naive();
    state.trade_store.get_by_date(today).await.map_err(CmdError::from)
}

#[derive(Debug, Deserialize)]
pub struct GetTradesByRangeInput {
    pub from: String,
    pub to: String,
}

#[tauri::command]
pub async fn get_trades_by_range(
    input: GetTradesByRangeInput,
    state: State<'_, AppState>,
) -> CmdResult<Vec<TradeRecord>> {
    use chrono::NaiveDate;
    let from = NaiveDate::parse_from_str(&input.from, "%Y-%m-%d").map_err(|e| CmdError {
        code: "INVALID_DATE".into(),
        message: format!("from 날짜 형식 오류: {}", e),
    })?;
    let to = NaiveDate::parse_from_str(&input.to, "%Y-%m-%d").map_err(|e| CmdError {
        code: "INVALID_DATE".into(),
        message: format!("to 날짜 형식 오류: {}", e),
    })?;
    state.trade_store.get_by_range(from, to).await.map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 일별 통계
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_stats(state: State<'_, AppState>) -> CmdResult<DailyStats> {
    let today = chrono::Local::now().date_naive();
    state.stats_store.get_by_date(today).await.map_err(CmdError::from)
}

#[derive(Debug, Deserialize)]
pub struct GetStatsByRangeInput {
    pub from: String,
    pub to: String,
}

#[tauri::command]
pub async fn get_stats_by_range(
    input: GetStatsByRangeInput,
    state: State<'_, AppState>,
) -> CmdResult<Vec<DailyStats>> {
    use chrono::NaiveDate;
    let from = NaiveDate::parse_from_str(&input.from, "%Y-%m-%d").map_err(|e| CmdError {
        code: "INVALID_DATE".into(),
        message: format!("from 날짜 형식 오류: {}", e),
    })?;
    let to = NaiveDate::parse_from_str(&input.to, "%Y-%m-%d").map_err(|e| CmdError {
        code: "INVALID_DATE".into(),
        message: format!("to 날짜 형식 오류: {}", e),
    })?;
    state.stats_store.get_by_range(from, to).await.map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// Discord 테스트 알림
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn send_test_discord(state: State<'_, AppState>) -> CmdResult<String> {
    match &state.discord {
        None => Err(CmdError {
            code: "DISCORD_NOT_CONFIGURED".into(),
            message: "Discord 봇이 설정되지 않았습니다. secure_config.json을 확인하세요.".into(),
        }),
        Some(notifier) => {
            let event = NotificationEvent::info(
                "테스트 알림".to_string(),
                "AutoConditionTrade 알림 시스템이 정상 작동 중입니다.".to_string(),
            );
            notifier.send(event).await.map_err(CmdError::from)?;
            Ok("Discord 테스트 알림 전송 완료".into())
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 체결 기록 저장
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SaveTradeInput {
    pub symbol: String,
    pub symbol_name: String,
    pub side: String,
    pub quantity: u64,
    pub price: u64,
    pub fee: u64,
    pub order_id: String,
    pub strategy_id: Option<String>,
}

#[tauri::command]
pub async fn save_trade(
    input: SaveTradeInput,
    state: State<'_, AppState>,
) -> CmdResult<TradeRecord> {
    use crate::storage::trade_store::TradeSide;

    let side = match input.side.as_str() {
        "buy" | "Buy" => TradeSide::Buy,
        "sell" | "Sell" => TradeSide::Sell,
        other => return Err(CmdError {
            code: "INVALID_SIDE".into(),
            message: format!("알 수 없는 방향: {}", other),
        }),
    };

    let record = TradeRecord::new(
        input.symbol, input.symbol_name, side.clone(),
        input.quantity, input.price, input.fee,
        input.order_id, input.strategy_id,
    );

    state.trade_store.append(record.clone()).await.map_err(CmdError::from)?;

    if let Some(notifier) = &state.discord {
        let side_label = if side == TradeSide::Buy { "매수" } else { "매도" };
        let _ = notifier
            .send(NotificationEvent::trade(format!(
                "{} {} {}주 @{}원",
                record.symbol_name, side_label, record.quantity, record.price
            )))
            .await;
    }

    Ok(record)
}

// ────────────────────────────────────────────────────────────────────
// 통계 업데이트
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn upsert_daily_stats(
    stats: DailyStats,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    state.stats_store.upsert(stats).await.map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 자동 매매 제어
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingStatus {
    pub is_running: bool,
    pub active_strategies: Vec<String>,
    pub position_count: usize,
    pub total_unrealized_pnl: i64,
}

#[tauri::command]
pub async fn get_trading_status(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let is_running = *state.is_trading.lock().await;
    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    Ok(TradingStatus { is_running, active_strategies: strategies, position_count, total_unrealized_pnl: total_pnl })
}

#[tauri::command]
pub async fn start_trading(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    if !state.config.read().await.is_kis_configured() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "KIS API 설정이 완료되지 않았습니다. Settings에서 API 키를 확인하세요.".into(),
        });
    }

    let mut is_running = state.is_trading.lock().await;
    if *is_running {
        return Err(CmdError {
            code: "ALREADY_RUNNING".into(),
            message: "자동 매매가 이미 실행 중입니다.".into(),
        });
    }
    *is_running = true;
    tracing::info!("자동 매매 시작");

    if let Some(notifier) = &state.discord {
        let _ = notifier.send(NotificationEvent::info(
            "자동 매매 시작".to_string(),
            "AutoConditionTrade 자동 매매가 시작되었습니다.".to_string(),
        )).await;
    }
    drop(is_running);

    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    Ok(TradingStatus { is_running: true, active_strategies: strategies, position_count, total_unrealized_pnl: total_pnl })
}

#[tauri::command]
pub async fn stop_trading(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let mut is_running = state.is_trading.lock().await;
    *is_running = false;
    tracing::info!("자동 매매 정지");

    if let Some(notifier) = &state.discord {
        let _ = notifier.send(NotificationEvent::info(
            "자동 매매 정지".to_string(),
            "AutoConditionTrade 자동 매매가 정지되었습니다.".to_string(),
        )).await;
    }
    drop(is_running);

    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    Ok(TradingStatus { is_running: false, active_strategies: strategies, position_count, total_unrealized_pnl: total_pnl })
}

// ────────────────────────────────────────────────────────────────────
// 포지션 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionView {
    pub symbol: String,
    pub symbol_name: String,
    pub quantity: u64,
    pub avg_price: f64,
    pub current_price: u64,
    pub unrealized_pnl: i64,
    pub unrealized_pnl_rate: f64,
}

impl From<&Position> for PositionView {
    fn from(p: &Position) -> Self {
        Self {
            symbol: p.symbol.clone(),
            symbol_name: p.symbol_name.clone(),
            quantity: p.quantity,
            avg_price: p.avg_price,
            current_price: p.current_price,
            unrealized_pnl: p.unrealized_pnl(),
            unrealized_pnl_rate: p.unrealized_pnl_rate(),
        }
    }
}

#[tauri::command]
pub async fn get_positions(state: State<'_, AppState>) -> CmdResult<Vec<PositionView>> {
    let tracker = state.position_tracker.lock().await;
    let mut positions: Vec<PositionView> = tracker.all().iter().map(|p| PositionView::from(*p)).collect();
    positions.sort_by(|a, b| b.quantity.cmp(&a.quantity));
    Ok(positions)
}

// ────────────────────────────────────────────────────────────────────
// 전략 관리
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyView {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub target_symbols: Vec<String>,
    pub order_quantity: u64,
    pub params: serde_json::Value,
}

#[tauri::command]
pub async fn get_strategies(state: State<'_, AppState>) -> CmdResult<Vec<StrategyView>> {
    let mgr = state.strategy_manager.lock().await;
    Ok(mgr.all_configs().iter().map(|c| StrategyView {
        id: c.id.clone(),
        name: c.name.clone(),
        enabled: c.enabled,
        target_symbols: c.target_symbols.clone(),
        order_quantity: c.order_quantity,
        params: c.params.clone(),
    }).collect())
}

#[derive(Debug, Deserialize)]
pub struct UpdateStrategyInput {
    pub id: String,
    pub enabled: Option<bool>,
    pub target_symbols: Option<Vec<String>>,
    pub order_quantity: Option<u64>,
    pub params: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn update_strategy(
    input: UpdateStrategyInput,
    state: State<'_, AppState>,
) -> CmdResult<StrategyView> {
    let mut mgr = state.strategy_manager.lock().await;
    let cfg = mgr.get_config_mut(&input.id).ok_or_else(|| CmdError {
        code: "STRATEGY_NOT_FOUND".into(),
        message: format!("전략을 찾을 수 없습니다: {}", input.id),
    })?;

    if let Some(enabled) = input.enabled { cfg.enabled = enabled; }
    if let Some(symbols) = input.target_symbols { cfg.target_symbols = symbols; }
    if let Some(qty) = input.order_quantity { cfg.order_quantity = qty; }
    if let Some(params) = input.params { cfg.params = params; }

    Ok(StrategyView {
        id: cfg.id.clone(),
        name: cfg.name.clone(),
        enabled: cfg.enabled,
        target_symbols: cfg.target_symbols.clone(),
        order_quantity: cfg.order_quantity,
        params: cfg.params.clone(),
    })
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
}

#[tauri::command]
pub async fn set_log_config(
    input: SetLogConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<LogConfig> {
    let new_cfg = LogConfig {
        retention_days: input.retention_days.clamp(1, 365),
        max_size_mb: input.max_size_mb.clamp(10, 10240),
    };

    // AppState 업데이트
    *state.log_config.write().await = new_cfg.clone();

    // 파일 저장
    new_cfg.save_sync(&state.log_dir).map_err(CmdError::from)?;

    // 즉시 정리 실행
    crate::logging::cleanup(&state.log_dir, &new_cfg);

    tracing::info!(
        "로그 설정 변경: 보관 {}일, 최대 {}MB",
        new_cfg.retention_days, new_cfg.max_size_mb
    );

    Ok(new_cfg)
}

// ────────────────────────────────────────────────────────────────────
// 프론트엔드 로그 기록
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FrontendLogInput {
    /// "error" | "warn" | "info" | "debug"
    pub level: String,
    pub message: String,
    pub context: Option<String>,
}

#[tauri::command]
pub async fn write_frontend_log(input: FrontendLogInput) -> CmdResult<()> {
    let msg = if let Some(ctx) = &input.context {
        format!("[{}] {}", ctx, input.message)
    } else {
        input.message.clone()
    };
    match input.level.to_lowercase().as_str() {
        "error" => tracing::error!(target: "frontend", "{}", msg),
        "warn"  => tracing::warn!(target: "frontend", "{}", msg),
        "debug" => tracing::debug!(target: "frontend", "{}", msg),
        _       => tracing::info!(target: "frontend", "{}", msg),
    }
    Ok(())
}

// ── 종목 검색 ─────────────────────────────────────────────────────────
#[tauri::command]
pub async fn search_stock(
    query: String,
    state: State<'_, AppState>,
) -> CmdResult<Vec<StockSearchItem>> {
    // 6자리 숫자 코드 → KIS 현재가로 종목명 확인
    if query.len() == 6 && query.chars().all(|c| c.is_ascii_digit()) {
        let client = state.rest_client.read().await.clone();
        return Ok(match client.get_price(&query).await {
            Ok(p) if !p.hts_kor_isnm.is_empty() => vec![StockSearchItem {
                pdno: query,
                prdt_name: p.hts_kor_isnm,
            }],
            _ => vec![],
        });
    }

    if query.len() < 2 {
        return Ok(vec![]);
    }

    // 이름/부분 검색 → 캐시된 KRX 목록에서 로컬 검색
    let stock_list = state.stock_list.read().await;
    Ok(crate::market::search_local(&stock_list, &query, 20))
}

// ── 종목 목록 새로고침 ─────────────────────────────────────────────
#[tauri::command]
pub async fn refresh_stock_list(state: State<'_, AppState>) -> CmdResult<usize> {
    let items = crate::market::StockList::fetch_from_krx()
        .await
        .map_err(CmdError::from)?;

    let count = items.len();

    // 메모리 갱신
    *state.stock_list.write().await = items.clone();

    // 캐시 파일 갱신
    let cache_path = state.data_dir.join("stock_list.json");
    if let Ok(json) = serde_json::to_string(&items) {
        let _ = std::fs::write(&cache_path, json);
    }

    tracing::info!("종목 목록 수동 갱신: {}개", count);
    Ok(count)
}

// ── KIS 기간별 체결 내역 ──────────────────────────────────────────
#[tauri::command]
pub async fn get_kis_executed_by_range(
    from: String, // YYYY-MM-DD
    to: String,   // YYYY-MM-DD
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::api::rest::ExecutedOrder>> {
    let from_fmt = from.replace('-', "");
    let to_fmt = to.replace('-', "");
    let client = state.rest_client.read().await.clone();
    client
        .get_executed_orders_range(&from_fmt, &to_fmt)
        .await
        .map_err(CmdError::from)
}

// ── 최근 로그 엔트리 (파일 기반) ──────────────────────────────────
#[tauri::command]
pub async fn get_recent_logs(
    count: u32,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::logging::LogEntry>> {
    Ok(crate::logging::read_recent_entries(&state.log_dir, count as usize))
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

/// 실전/모의 토큰 발급 테스트용 요청 바디
#[derive(Serialize)]
struct DetectTokenReq {
    grant_type: String,
    appkey: String,
    appsecret: String,
}

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
/// 실제로 `access_token`이 반환된 환경을 기준으로 판별합니다.
#[tauri::command]
pub async fn detect_trading_type(
    app_key: String,
    app_secret: String,
) -> CmdResult<DetectTradingTypeResult> {
    const REAL_URL: &str = "https://openapi.koreainvestment.com:9443/oauth2/tokenP";
    const PAPER_URL: &str = "https://openapivts.koreainvestment.com:29443/oauth2/tokenP";

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

    // ── 실전투자 URL 시도 ──────────────────────────────────────────
    let real_result = client
        .post(REAL_URL)
        .header("content-type", "application/json; charset=utf-8")
        .json(&DetectTokenReq {
            grant_type: "client_credentials".into(),
            appkey: app_key.clone(),
            appsecret: app_secret.clone(),
        })
        .send()
        .await;

    if let Ok(resp) = real_result {
        if resp.status().is_success() {
            if let Ok(val) = resp.json::<serde_json::Value>().await {
                let token_ok = val
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .map(|t| !t.is_empty())
                    .unwrap_or(false);
                if token_ok {
                    tracing::info!("자동 감지 완료: 실전투자 키");
                    return Ok(DetectTradingTypeResult {
                        is_paper_trading: false,
                        message: "실전투자 키로 확인되었습니다.".into(),
                    });
                }
            }
        }
    }

    // ── 모의투자 URL 시도 ──────────────────────────────────────────
    let paper_result = client
        .post(PAPER_URL)
        .header("content-type", "application/json; charset=utf-8")
        .json(&DetectTokenReq {
            grant_type: "client_credentials".into(),
            appkey: app_key,
            appsecret: app_secret,
        })
        .send()
        .await;

    if let Ok(resp) = paper_result {
        if resp.status().is_success() {
            if let Ok(val) = resp.json::<serde_json::Value>().await {
                let token_ok = val
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .map(|t| !t.is_empty())
                    .unwrap_or(false);
                if token_ok {
                    tracing::info!("자동 감지 완료: 모의투자 키");
                    return Ok(DetectTradingTypeResult {
                        is_paper_trading: true,
                        message: "모의투자 키로 확인되었습니다.".into(),
                    });
                }
            }
        }
    }

    Err(CmdError {
        code: "DETECT_FAILED".into(),
        message: "실전/모의 키를 자동 감지하지 못했습니다. 네트워크 또는 API 키를 확인하거나 직접 선택해 주세요.".into(),
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

    // 2) 실전/모의 토큰 발급 시도 (detect_trading_type 로직 재사용)
    const REAL_URL: &str = "https://openapi.koreainvestment.com:9443/oauth2/tokenP";
    const PAPER_URL: &str = "https://openapivts.koreainvestment.com:29443/oauth2/tokenP";

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| CmdError {
            code: "CLIENT_BUILD".into(),
            message: e.to_string(),
        })?;

    let mut detected_paper: Option<bool> = None;

    // 실전 시도
    if let Ok(resp) = client
        .post(REAL_URL)
        .header("content-type", "application/json; charset=utf-8")
        .json(&DetectTokenReq {
            grant_type: "client_credentials".into(),
            appkey: app_key.clone(),
            appsecret: app_secret.clone(),
        })
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(val) = resp.json::<serde_json::Value>().await {
                let ok = val
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .map(|t| !t.is_empty())
                    .unwrap_or(false);
                if ok {
                    detected_paper = Some(false);
                }
            }
        }
    }

    // 실전 실패 시 모의 시도
    if detected_paper.is_none() {
        if let Ok(resp) = client
            .post(PAPER_URL)
            .header("content-type", "application/json; charset=utf-8")
            .json(&DetectTokenReq {
                grant_type: "client_credentials".into(),
                appkey: app_key,
                appsecret: app_secret,
            })
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(val) = resp.json::<serde_json::Value>().await {
                    let ok = val
                        .get("access_token")
                        .and_then(|v| v.as_str())
                        .map(|t| !t.is_empty())
                        .unwrap_or(false);
                    if ok {
                        detected_paper = Some(true);
                    }
                }
            }
        }
    }

    let is_paper = detected_paper.ok_or_else(|| CmdError {
        code: "DETECT_FAILED".into(),
        message: "실전/모의 키를 자동 감지하지 못했습니다. 네트워크 또는 API 키를 확인해 주세요.".into(),
    })?;

    // 3) 프로파일 업데이트 및 저장
    let view = {
        let mut profiles = state.profiles.write().await;
        let updated = profiles
            .update(&profile_id, None, Some(is_paper), None, None, None)
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
        if is_paper { "모의투자" } else { "실전투자" }
    );
    Ok(view)
}
