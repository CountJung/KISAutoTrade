/// Tauri IPC 커맨드 모음
///
/// Frontend(React) ↔ Backend(Rust) 통신 인터페이스
/// 모든 커맨드는 AppState를 통해 공유 리소스에 접근합니다.
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

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
    market_hours::{is_domestic_symbol, is_market_open_for, open_markets_summary},
    notifications::{discord::DiscordNotifier, types::NotificationEvent},
    storage::{stats_store::DailyStats, stock_store::{StockListStats, StockStore}, strategy_store::StrategyStore, trade_store::TradeRecord, OrderStore, StatsStore, TradeStore},
    trading::{
        order::OrderManager,
        position::{Position, PositionTracker},
        risk::RiskManager,
    strategy::{
        ConsecutiveMoveParams, ConsecutiveMoveStrategy,
        DeviationParams, DeviationStrategy,
        FailedBreakoutParams, FailedBreakoutStrategy,
        FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy,
        MaCrossParams, MomentumParams, MomentumStrategy,
        MovingAverageCrossStrategy, RsiParams, RsiStrategy,
        StrongCloseParams, StrongCloseStrategy,
        TrendFilterParams, TrendFilterStrategy,
        VolatilityExpansionParams, VolatilityExpansionStrategy,
        MeanReversionParams, MeanReversionStrategy,
        StrategyConfig, StrategyManager,
    },
    },
};

// ────────────────────────────────────────────────────────────────────
// 체결 기록 보관 설정
// ────────────────────────────────────────────────────────────────────

/// 체결 기록 보관 설정 (보관 기간, 최대 저장 용량)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeArchiveConfig {
    pub retention_days: u32,  // 보관 기간 (일), 기본 90
    pub max_size_mb: u64,     // 최대 저장 용량 (MB), 기본 500
}

impl Default for TradeArchiveConfig {
    fn default() -> Self {
        Self { retention_days: 90, max_size_mb: 500 }
    }
}

impl TradeArchiveConfig {
    /// 저장 파일에서 로드, 없으면 기본값
    pub fn load_or_default(data_dir: &Path) -> Self {
        let path = data_dir.join("trade_archive_config.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// 파일에 동기 저장
    pub fn save_sync(&self, data_dir: &Path) -> std::result::Result<(), String> {
        if let Some(parent) = data_dir.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
        let path = data_dir.join("trade_archive_config.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

/// 체결 기록 저장소 통계
#[derive(Debug, Serialize)]
pub struct TradeArchiveStats {
    pub total_files: u64,
    pub size_bytes: u64,
    pub oldest_date: Option<String>,
    pub newest_date: Option<String>,
}

/// 날짜별 trades 디렉토리 목록 수집 (trades/YYYY/MM/DD/)
fn collect_trade_day_dirs(data_dir: &Path) -> Vec<(chrono::NaiveDate, PathBuf)> {
    let trades_dir = data_dir.join("trades");
    if !trades_dir.exists() {
        return vec![];
    }
    let mut result = Vec::new();
    let Ok(year_entries) = std::fs::read_dir(&trades_dir) else { return result; };
    for year_entry in year_entries.flatten() {
        let year_path = year_entry.path();
        if !year_path.is_dir() { continue; }
        let Some(year_str) = year_entry.file_name().into_string().ok() else { continue; };
        let Ok(year) = year_str.parse::<i32>() else { continue; };
        let Ok(month_entries) = std::fs::read_dir(&year_path) else { continue; };
        for month_entry in month_entries.flatten() {
            let month_path = month_entry.path();
            if !month_path.is_dir() { continue; }
            let Some(month_str) = month_entry.file_name().into_string().ok() else { continue; };
            let Ok(month) = month_str.parse::<u32>() else { continue; };
            let Ok(day_entries) = std::fs::read_dir(&month_path) else { continue; };
            for day_entry in day_entries.flatten() {
                let day_path = day_entry.path();
                if !day_path.is_dir() { continue; }
                let Some(day_str) = day_entry.file_name().into_string().ok() else { continue; };
                let Ok(day) = day_str.parse::<u32>() else { continue; };
                if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                    result.push((date, day_path));
                }
            }
        }
    }
    result.sort_by_key(|(d, _)| *d);
    result
}

/// 디렉토리 내 파일 총 크기 (바이트)
fn dir_size_bytes(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let ep = entry.path();
            if ep.is_file() {
                total += ep.metadata().map(|m| m.len()).unwrap_or(0);
            } else if ep.is_dir() {
                total += dir_size_bytes(&ep);
            }
        }
    }
    total
}

/// 오래된 체결 기록 파일 정리 (보관 기간 초과 + 용량 초과)
fn purge_old_trade_files(data_dir: &Path, cfg: &TradeArchiveConfig) {
    let cutoff = chrono::Local::now().date_naive()
        - chrono::Duration::days(cfg.retention_days as i64);

    let mut day_dirs = collect_trade_day_dirs(data_dir);

    // 보관 기간 초과 삭제
    let mut remaining: Vec<(chrono::NaiveDate, PathBuf)> = Vec::new();
    for (date, dir) in day_dirs.drain(..) {
        if date < cutoff {
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                tracing::warn!("체결 기록 기간 정리 실패 ({:?}): {}", dir, e);
            } else {
                tracing::info!("체결 기록 정리: {} ({} 이전)", date, cutoff);
            }
        } else {
            remaining.push((date, dir));
        }
    }

    // 용량 초과 시 오래된 것부터 삭제
    let max_bytes = cfg.max_size_mb * 1024 * 1024;
    let mut total_size: u64 = remaining.iter().map(|(_, d)| dir_size_bytes(d)).sum();
    for (date, dir) in &remaining {
        if total_size <= max_bytes { break; }
        let sz = dir_size_bytes(dir);
        if let Err(e) = std::fs::remove_dir_all(dir) {
            tracing::warn!("체결 기록 용량 정리 실패 ({:?}): {}", dir, e);
        } else {
            total_size = total_size.saturating_sub(sz);
            tracing::info!("체결 기록 용량 정리: {} ({}MB 초과)", date, cfg.max_size_mb);
        }
    }
}

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
    /// 주문 관리자 (전략 신호 → KIS 주문 + 체결 추적)
    pub order_manager: Arc<Mutex<OrderManager>>,
    /// 주문 이력 저장소
    pub order_store: Arc<OrderStore>,
    /// 리스크 관리자
    pub risk_manager: Arc<Mutex<RiskManager>>,
    /// 로그 디렉토리 경로
    pub log_dir: PathBuf,
    /// 로그 설정 (보관 기간, 최대 용량)
    pub log_config: Arc<RwLock<LogConfig>>,
    /// 체결 기록 보관 설정
    pub trade_archive_config: Arc<RwLock<TradeArchiveConfig>>,
    /// 데이터 저장 경로
    pub data_dir: PathBuf,
    /// KRX 캐시된 종목 목록 (이름 검색용, 레거시 — KRX WAF 차단 시 빈 채로 유지될 수 있음)
    pub stock_list: Arc<RwLock<Vec<crate::api::rest::StockSearchItem>>>,
    /// 영구 종목 목록 캐시 (KIS API 응답에서 자동 수집 + stocklist/stocklist.json)
    pub stock_store: Arc<StockStore>,
    /// 전략 설정 영구 저장소 (프로파일별 JSON)
    pub strategy_store: Arc<StrategyStore>,
    /// 웹 서버 포트
    pub web_port: u16,
    /// WebSocket 연결 상태 (Dashboard 실시간 반영용)
    pub ws_connected: Arc<AtomicBool>,
    /// 자동매매가 시작된 시점의 프로파일 ID (프로파일 전환 중에도 유지)
    pub trading_profile_id: Arc<RwLock<Option<String>>>,
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
        let order_store = Arc::new(OrderStore::new(data_dir.clone()));
        let risk_manager = Arc::new(Mutex::new(RiskManager::default()));
        let position_tracker = Arc::new(Mutex::new(PositionTracker::new()));

        // rest_client를 RwLock으로 감싸서 OrderManager와 공유
        let rest_client_rw = Arc::new(RwLock::new(rest_client));

        let order_manager = Arc::new(Mutex::new(OrderManager::new(
            Arc::clone(&rest_client_rw),
            Arc::clone(&order_store),
            Arc::clone(&trade_store),
            Arc::clone(&position_tracker),
            Arc::clone(&stats_store),
            Arc::clone(&risk_manager),
            discord.clone(),
        )));

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

        // RSI 전략 (기본 등록, 비활성)
        let rsi_strategy = StrategyConfig {
            id: "rsi_default".to_string(),
            name: "RSI 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(RsiParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(RsiStrategy::new(rsi_strategy)));

        // 모멘텀 전략 (기본 등록, 비활성)
        let momentum_strategy = StrategyConfig {
            id: "momentum_default".to_string(),
            name: "모멘텀 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(MomentumParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(MomentumStrategy::new(momentum_strategy)));

        // 이격도 전략 (기본 등록, 비활성)
        let deviation_strategy = StrategyConfig {
            id: "deviation_default".to_string(),
            name: "이격도 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(DeviationParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(DeviationStrategy::new(deviation_strategy)));

        // 52주 신고가 전략 (기본 등록, 비활성)
        let fifty_two_week_high_strategy = StrategyConfig {
            id: "fifty_two_week_high_default".to_string(),
            name: "52주 신고가 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(FiftyTwoWeekHighParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(FiftyTwoWeekHighStrategy::new(fifty_two_week_high_strategy)));

        // 연속 상승/하락 전략 (기본 등록, 비활성)
        let consecutive_move_strategy = StrategyConfig {
            id: "consecutive_move_default".to_string(),
            name: "연속 상승/하락 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(ConsecutiveMoveParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(ConsecutiveMoveStrategy::new(consecutive_move_strategy)));

        // 돌파 실패 전략 (기본 등록, 비활성)
        let failed_breakout_strategy = StrategyConfig {
            id: "failed_breakout_default".to_string(),
            name: "돌파 실패 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(FailedBreakoutParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(FailedBreakoutStrategy::new(failed_breakout_strategy)));

        // 강한 종가 전략 (기본 등록, 비활성)
        let strong_close_strategy = StrategyConfig {
            id: "strong_close_default".to_string(),
            name: "강한 종가 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(StrongCloseParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(StrongCloseStrategy::new(strong_close_strategy)));

        // 변동성 확장 전략 (기본 등록, 비활성)
        let volatility_expansion_strategy = StrategyConfig {
            id: "volatility_expansion_default".to_string(),
            name: "변동성 확장 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(VolatilityExpansionParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(VolatilityExpansionStrategy::new(volatility_expansion_strategy)));

        // 평균회귀 전략 (기본 등록, 비활성)
        let mean_reversion_strategy = StrategyConfig {
            id: "mean_reversion_default".to_string(),
            name: "평균회귀 전략 (볼린저 밴드)".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(MeanReversionParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(MeanReversionStrategy::new(mean_reversion_strategy)));

        // 추세 필터 전략 (기본 등록, 비활성)
        let trend_filter_strategy = StrategyConfig {
            id: "trend_filter_default".to_string(),
            name: "추세 필터 전략".to_string(),
            enabled: false,
            target_symbols: vec![],
            order_quantity: 1,
            params: serde_json::to_value(TrendFilterParams::default()).unwrap_or_default(),
        };
        strategy_manager.add(Box::new(TrendFilterStrategy::new(trend_filter_strategy)));

        // 전략 설정 영구 저장소
        let strategy_store = Arc::new(StrategyStore::new(&data_dir));

        // 저장된 전략 설정 로드 (프로파일별, 프로그램 재시작 후 복원)
        if let Some(profile_id) = profiles.active_id.as_deref() {
            let saved = strategy_store.load_sync(profile_id);
            if !saved.is_empty() {
                strategy_manager.apply_saved_configs(&saved);
                tracing::info!(
                    "전략 설정 복원: 프로파일 '{}', {}개 전략",
                    profile_id, saved.len()
                );
            }
        }

        Self {
            config: Arc::new(RwLock::new(config)),
            rest_client: rest_client_rw,
            discord,
            discord_config,
            profiles: Arc::new(RwLock::new(profiles)),
            profiles_path,
            trade_store,
            stats_store,
            order_store,
            is_trading: Arc::new(Mutex::new(false)),
            strategy_manager: Arc::new(Mutex::new(strategy_manager)),
            position_tracker,
            order_manager,
            risk_manager,
            log_dir,
            log_config: Arc::new(RwLock::new(log_config)),
            trade_archive_config: Arc::new(RwLock::new(TradeArchiveConfig::load_or_default(&data_dir))),
            data_dir: data_dir.clone(),
            stock_list: Arc::new(RwLock::new(vec![])),
            stock_store: Arc::new(StockStore::new(&data_dir)),
            strategy_store,
            web_port,
            ws_connected: Arc::new(AtomicBool::new(false)),
            trading_profile_id: Arc::new(RwLock::new(None)),
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

    // 자동매매 실행 중에는 REST 클라이언트/config 교체를 하지 않는다.
    // active_id만 변경(UI 반영용)하여 진행 중 주문·포지션에 영향이 없도록 한다.
    if *state.is_trading.lock().await {
        tracing::warn!(
            "자동매매 실행 중 프로파일 전환 요청 (id={}): UI active_id만 변경, REST 클라이언트 유지",
            id
        );
        save_profiles(&state).await?;
        return get_app_config(state).await;
    }

    apply_active_profile(&state).await?;
    save_profiles(&state).await?;
    get_app_config(state).await
}

/// 현재 active_id 기반으로 config + rest_client + 전략 설정 교체
async fn apply_active_profile(state: &AppState) -> CmdResult<()> {
    let (new_config, active_id) = {
        let profiles = state.profiles.read().await;
        let cfg = match profiles.get_active() {
            Some(p) => AppConfig::from_profile(p, &state.discord_config),
            None => AppConfig::empty(&state.discord_config),
        };
        (cfg, profiles.active_id.clone())
    };

    let new_client = make_rest_client(&new_config);

    *state.config.write().await = new_config;
    *state.rest_client.write().await = new_client;

    // 프로파일 전환 시 해당 프로파일의 전략 설정 로드 (재시작 없이도 반영)
    if let Some(pid) = &active_id {
        let saved = state.strategy_store.load_sync(pid);
        if !saved.is_empty() {
            let mut mgr = state.strategy_manager.lock().await;
            mgr.apply_saved_configs(&saved);
            tracing::info!(
                "프로파일 전환 — 전략 설정 복원: 프로파일 '{}', {}개 전략",
                pid, saved.len()
            );
        }
    }

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
            // 잔고 응답의 종목코드+이름 데이터 자동 수집
            state.stock_store.upsert_many(
                resp.items.iter().map(|i| (i.pdno.clone(), i.prdt_name.clone()))
            ).await;
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
    let result = client.get_price(&symbol).await.map_err(CmdError::from)?;
    // 현재가 응답에서 종목명 자동 수집
    if !result.hts_kor_isnm.is_empty() {
        state.stock_store.upsert(&symbol, &result.hts_kor_isnm).await;
    }
    Ok(result)
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
    /// WebSocket 실시간 시세 연결 여부
    pub ws_connected: bool,
    /// 자동매매가 실행 중인 프로파일 ID (미실행 시 None)
    pub trading_profile_id: Option<String>,
}

#[tauri::command]
pub async fn get_trading_status(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let is_running = *state.is_trading.lock().await;
    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let trading_profile_id = state.trading_profile_id.read().await.clone();
    Ok(TradingStatus {
        is_running,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
    })
}

#[tauri::command]
pub async fn start_trading(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<TradingStatus> {
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

    // 자동매매 시작 시점의 활성 프로파일 ID 스냅샷 저장
    {
        let active_id = state.profiles.read().await.active_id.clone();
        *state.trading_profile_id.write().await = active_id;
    }

    if let Some(notifier) = &state.discord {
        let _ = notifier.send(NotificationEvent::info(
            "자동 매매 시작".to_string(),
            "AutoConditionTrade 자동 매매가 시작되었습니다.".to_string(),
        )).await;
    }
    drop(is_running);

    // 활성 전략의 종목별 일봉 차트 데이터 로드 → 히스토리 기반 전략 초기화 (52주 신고가 등)
    // 국내 종목: get_chart_data (KRW 정수 가격)
    // 해외 종목: get_overseas_chart_data (USD float → ×100 센트로 정수화)
    {
        let active_symbols: Vec<String> = state.strategy_manager.lock().await.active_symbols();
        if !active_symbols.is_empty() {
            let rest = state.rest_client.read().await.clone();
            let today = chrono::Local::now();
            let end_date = today.format("%Y%m%d").to_string();
            // 400일치 조회 (52주 = 252거래일 + 여유분)
            let start_date = (today - chrono::Duration::days(400)).format("%Y%m%d").to_string();

            for symbol in &active_symbols {
                if is_domestic_symbol(symbol) {
                    // ── 국내 종목 초기화 ──
                    match rest.get_chart_data(symbol, "D", &start_date, &end_date).await {
                        Ok(candles) if !candles.is_empty() => {
                            let highs: Vec<u64> = candles.iter()
                                .filter_map(|c| c.high.parse::<u64>().ok())
                                .collect();
                            if !highs.is_empty() {
                                state.strategy_manager.lock().await
                                    .initialize_historical(symbol, &highs);
                                tracing::info!("전략 히스토리 초기화 완료: {} ({}봉)", symbol, highs.len());
                            }
                            let high_close: Vec<(u64, u64)> = candles.iter()
                                .filter_map(|c| {
                                    let h = c.high.parse::<u64>().ok()?;
                                    let cl = c.close.parse::<u64>().ok()?;
                                    Some((h, cl))
                                })
                                .collect();
                            if !high_close.is_empty() {
                                state.strategy_manager.lock().await
                                    .initialize_candles(symbol, &high_close);
                            }
                            let ranges: Vec<u64> = candles.iter()
                                .filter_map(|c| {
                                    let h = c.high.parse::<u64>().ok()?;
                                    let l = c.low.parse::<u64>().ok()?;
                                    Some(h.saturating_sub(l))
                                })
                                .collect();
                            if !ranges.is_empty() {
                                state.strategy_manager.lock().await
                                    .initialize_range_data(symbol, &ranges);
                            }
                        }
                        Ok(_) => tracing::debug!("차트 데이터 없음 (히스토리 초기화 건너뜀): {}", symbol),
                        Err(e) => tracing::warn!(
                            "차트 데이터 조회 실패 (히스토리 초기화 건너뜀): {} — {}", symbol, e
                        ),
                    }
                } else {
                    // ── 해외 종목 초기화 (NAS → NYS → AMS 순 시도) ──
                    let mut initialized = false;
                    for exchange in &["NAS", "NYS", "AMS"] {
                        match rest.get_overseas_chart_data(symbol, exchange, "D", &end_date).await {
                            Ok(candles) if !candles.is_empty() => {
                                // USD float 문자열 → ×100 센트(u64)로 변환하여 전략 히스토리 초기화
                                let highs: Vec<u64> = candles.iter()
                                    .filter_map(|c| {
                                        c.high.parse::<f64>().ok()
                                            .map(|v| (v * 100.0).round() as u64)
                                    })
                                    .filter(|&v| v > 0)
                                    .collect();
                                if !highs.is_empty() {
                                    state.strategy_manager.lock().await
                                        .initialize_historical(symbol, &highs);
                                    tracing::info!(
                                        "해외 전략 히스토리 초기화: {} @ {} ({}봉, 센트 단위)",
                                        symbol, exchange, highs.len()
                                    );
                                }
                                let high_close: Vec<(u64, u64)> = candles.iter()
                                    .filter_map(|c| {
                                        let h = c.high.parse::<f64>().ok()
                                            .map(|v| (v * 100.0).round() as u64)?;
                                        let cl = c.close.parse::<f64>().ok()
                                            .map(|v| (v * 100.0).round() as u64)?;
                                        if h > 0 && cl > 0 { Some((h, cl)) } else { None }
                                    })
                                    .collect();
                                if !high_close.is_empty() {
                                    state.strategy_manager.lock().await
                                        .initialize_candles(symbol, &high_close);
                                }
                                let ranges: Vec<u64> = candles.iter()
                                    .filter_map(|c| {
                                        let h = c.high.parse::<f64>().ok()?;
                                        let l = c.low.parse::<f64>().ok()?;
                                        let diff = ((h - l) * 100.0).round() as u64;
                                        if diff > 0 { Some(diff) } else { None }
                                    })
                                    .collect();
                                if !ranges.is_empty() {
                                    state.strategy_manager.lock().await
                                        .initialize_range_data(symbol, &ranges);
                                }
                                initialized = true;
                                break;
                            }
                            Ok(_) => continue,
                            Err(_) => continue,
                        }
                    }
                    if !initialized {
                        tracing::warn!(
                            "해외 종목 히스토리 초기화 실패: {} (NAS/NYS/AMS 모두 실패, 실시간 틱 누적 모드로 시작)",
                            symbol
                        );
                    }
                }
            }
        }
    }

    // WebSocket 연결 시작 (보조 — 실패해도 폴링 루프가 독립 동작)
    {
        let rest = state.rest_client.read().await.clone();
        let ws_client = crate::api::KisWebSocketClient::new(
            rest.is_paper(),
            rest.app_key().to_string(),
            rest.app_secret().to_string(),
            rest.token_manager(),
        );

        // 활성 전략에서 구독할 종목 수집
        let symbols: Vec<String> = state
            .strategy_manager
            .lock()
            .await
            .active_symbols();

        let ws_connected = Arc::clone(&state.ws_connected);
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = ws_client.subscribe(symbols, app_handle, ws_connected).await {
                tracing::error!("WebSocket 연결 실패: {}", e);
            }
        });
    }

    // ── 폴링 기반 자동매매 루프 ──────────────────────────────────
    // run_trading_daemon() 이 앱 시작 시 영구 데몬으로 이미 실행 중이다.
    // is_trading 플래그가 true 로 바뀌면 데몬이 자동으로 폴링을 재개한다.
    // (이전 spawn 블록은 lib.rs → tauri::async_runtime::spawn(run_trading_daemon(...)) 로 이동)

    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let trading_profile_id = state.trading_profile_id.read().await.clone();
    Ok(TradingStatus {
        is_running: true,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
    })
}

#[tauri::command]
pub async fn stop_trading(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let mut is_running = state.is_trading.lock().await;
    *is_running = false;
    tracing::info!("자동 매매 정지");

    // 자동매매 종료 시 트레이딩 프로파일 ID 클리어
    *state.trading_profile_id.write().await = None;

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
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    Ok(TradingStatus {
        is_running: false,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id: None,
    })
}

// ────────────────────────────────────────────────────────────────────
// 자동매매 폴링 데몬 (lib.rs 에서 앱 시작 시 영구 spawn)
//
// is_trading 플래그가 false 이면 5초마다 재확인하며 대기.
// true 로 바뀌면 즉시 폴링 재개. start_trading / web API 모두 이 방식으로 제어.
// ────────────────────────────────────────────────────────────────────
pub async fn run_trading_daemon(
    is_trading:   Arc<Mutex<bool>>,
    strategy_mgr: Arc<Mutex<crate::trading::strategy::StrategyManager>>,
    order_mgr:    Arc<Mutex<crate::trading::order::OrderManager>>,
    risk_mgr:     Arc<Mutex<crate::trading::risk::RiskManager>>,
    rest_arc:     Arc<RwLock<Arc<KisRestClient>>>,
    stock_store:  Arc<crate::storage::stock_store::StockStore>,
) {
    tracing::info!("자동매매 폴링 데몬 시작 (is_trading=false 대기 중)");
    let mut last_reset_date = chrono::Local::now().date_naive();
    let mut market_pause_until: Option<tokio::time::Instant> = None;
    let mut fills_pending: Vec<(String, u64)> = Vec::new();
    let mut was_running = false;

    'main_loop: loop {
        let is_running = *is_trading.lock().await;

        // ── 자동매매 비활성 → 5초 슬립 후 재확인 ─────────────────
        if !is_running {
            if was_running {
                fills_pending.clear();
                market_pause_until = None;
                tracing::info!("자동매매 폴링 데몬 일시 정지 (is_trading=false)");
            }
            was_running = false;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue 'main_loop;
        }

        // ── 방금 활성화됨 → 로컬 상태 초기화 ─────────────────────
        if !was_running {
            was_running = true;
            fills_pending.clear();
            market_pause_until = None;
            last_reset_date = chrono::Local::now().date_naive();
            tracing::info!("자동매매 폴링 데몬 활성화");
        }

        // 장 마감으로 대기 중 → 30초 슬립 후 재진입
        if let Some(pause_until) = market_pause_until {
            if tokio::time::Instant::now() < pause_until {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                continue 'main_loop;
            }
            tracing::info!("장 마감 대기 완료 — 폴링 재개");
            market_pause_until = None;
        }

        // ── 이전 틱 시장가 주문 자동 체결 확인 ──────────────────────
        if !fills_pending.is_empty() {
            let fills = std::mem::take(&mut fills_pending);
            for (sym, fill_price) in fills {
                if let Err(e) = order_mgr.lock().await
                    .confirm_fill_by_symbol(&sym, fill_price)
                    .await
                {
                    tracing::warn!("자동 체결 확인 실패 ({}): {}", sym, e);
                }
            }
        }

        // 날짜 변경 시 일별 초기화
        let today = chrono::Local::now().date_naive();
        if today != last_reset_date {
            last_reset_date = today;
            risk_mgr.lock().await.reset_if_new_day();
            order_mgr.lock().await.reset_day();
            tracing::info!("자동매매 일별 초기화 완료 ({})", today);
        }

        // 활성 전략의 종목 수집
        let symbols: Vec<String> = strategy_mgr.lock().await.active_symbols();
        if symbols.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            continue 'main_loop;
        }

        let rest = rest_arc.read().await.clone();
        let is_paper = rest.is_paper();
        let delay_ms: u64 = if is_paper { 700 } else { 150 };

        // ── 시장 개장 여부 사전 체크 ─────────────────────────────────
        {
            let all_closed = symbols.iter().all(|s| !is_market_open_for(s));
            if all_closed {
                tracing::info!(
                    "모든 시장 폐장 ({}) — 5분 대기 후 재확인",
                    open_markets_summary()
                );
                market_pause_until = Some(
                    tokio::time::Instant::now() + tokio::time::Duration::from_secs(300)
                );
                continue 'main_loop;
            }
            tracing::debug!("시장 상태: {}", open_markets_summary());
        }

        // ── 종목별 현재가 조회 + 전략 신호 처리 ─────────────────────
        'symbol_loop: for symbol in &symbols {
            if !*is_trading.lock().await {
                break 'symbol_loop;
            }

            if !is_market_open_for(symbol) {
                tracing::debug!(
                    "시장 폐장 — 건너뜀: {} ({})",
                    symbol,
                    if is_domestic_symbol(symbol) { "KRX" } else { "US" }
                );
                continue;
            }

            let tick = if is_domestic_symbol(symbol) {
                rest.get_price(symbol).await
                    .map(|p| {
                        let price  = p.stck_prpr.parse::<u64>().unwrap_or(0);
                        let volume = p.acml_vol.parse::<u64>().unwrap_or(0);
                        (price, volume)
                    })
                    .map_err(|e| e.to_string())
            } else {
                fetch_overseas_tick(&rest, symbol).await
                    .map_err(|e| e.to_string())
            };

            match tick {
                Ok((price, volume)) if price > 0 => {
                    let signals = strategy_mgr.lock().await.on_tick(symbol, price, volume);
                    for signal in signals {
                        use crate::trading::strategy::Signal;
                        if matches!(signal, Signal::Hold) { continue; }
                        let symbol_name = stock_store.get_name(symbol).await
                            .unwrap_or_else(|| symbol.clone());
                        match order_mgr.lock().await
                            .submit_signal(signal, &symbol_name, 0)
                            .await
                        {
                            Ok(()) => {
                                fills_pending.push((symbol.clone(), price));
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                            }
                            Err(e) => {
                                let msg = e.to_string();
                                if is_market_closed_error(&msg) {
                                    tracing::info!(
                                        "장 마감/장외 시간 감지 (주문, {}) — 5분 대기: {}",
                                        symbol, msg
                                    );
                                    market_pause_until = Some(
                                        tokio::time::Instant::now()
                                            + tokio::time::Duration::from_secs(300)
                                    );
                                    break 'symbol_loop;
                                }
                                tracing::warn!("신호 처리 실패 ({}): {}", symbol, msg);
                            }
                        }
                    }
                }
                Ok(_) => { tracing::debug!("현재가 0 — 건너뜀: {}", symbol); }
                Err(e) => {
                    if is_market_closed_error(&e) {
                        tracing::info!(
                            "장 마감/장외 시간 감지 (현재가, {}) — 5분 대기: {}",
                            symbol, e
                        );
                        market_pause_until = Some(
                            tokio::time::Instant::now() + tokio::time::Duration::from_secs(300)
                        );
                        break 'symbol_loop;
                    }
                    tracing::warn!("현재가 조회 실패 ({}): {}", symbol, e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            if !*is_trading.lock().await { break 'symbol_loop; }
        }

        if market_pause_until.is_some() { continue 'main_loop; }

        // 다음 틱까지 10초 대기 (100ms × 100 — 종료 신호 즉시 반응)
        for _ in 0u32..100 {
            if !*is_trading.lock().await { break; }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
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
    /// 종목코드 → 종목명 (StockStore에서 조회, 없으면 코드 그대로)
    pub target_symbol_names: std::collections::HashMap<String, String>,
    pub order_quantity: u64,
    pub params: serde_json::Value,
}

#[tauri::command]
pub async fn get_strategies(state: State<'_, AppState>) -> CmdResult<Vec<StrategyView>> {
    let mgr = state.strategy_manager.lock().await;
    let mut views = Vec::new();
    for c in mgr.all_configs() {
        let mut symbol_names = std::collections::HashMap::new();
        for code in &c.target_symbols {
            let name = state.stock_store.get_name(code).await
                .unwrap_or_else(|| code.clone());
            symbol_names.insert(code.clone(), name);
        }
        views.push(StrategyView {
            id: c.id.clone(),
            name: c.name.clone(),
            enabled: c.enabled,
            target_symbols: c.target_symbols.clone(),
            target_symbol_names: symbol_names,
            order_quantity: c.order_quantity,
            params: c.params.clone(),
        });
    }
    Ok(views)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    let target_symbols_snapshot = {
        let mut mgr = state.strategy_manager.lock().await;
        let cfg = mgr.get_config_mut(&input.id).ok_or_else(|| CmdError {
            code: "STRATEGY_NOT_FOUND".into(),
            message: format!("전략을 찾을 수 없습니다: {}", input.id),
        })?;

        if let Some(enabled) = input.enabled { cfg.enabled = enabled; }
        if let Some(symbols) = input.target_symbols { cfg.target_symbols = symbols; }
        if let Some(qty) = input.order_quantity { cfg.order_quantity = qty; }
        if let Some(params) = input.params { cfg.params = params; }

        cfg.target_symbols.clone()
    };

    // StockStore에서 종목명 조회
    let mut symbol_names = std::collections::HashMap::new();
    for code in &target_symbols_snapshot {
        let name = state.stock_store.get_name(code).await
            .unwrap_or_else(|| code.clone());
        symbol_names.insert(code.clone(), name);
    }

    let view = {
        let mgr = state.strategy_manager.lock().await;
        let cfg = mgr.all_configs().into_iter().find(|c| c.id == input.id)
            .ok_or_else(|| CmdError {
                code: "STRATEGY_NOT_FOUND".into(),
                message: format!("전략을 찾을 수 없습니다: {}", input.id),
            })?;
        StrategyView {
            id: cfg.id.clone(),
            name: cfg.name.clone(),
            enabled: cfg.enabled,
            target_symbols: cfg.target_symbols.clone(),
            target_symbol_names: symbol_names,
            order_quantity: cfg.order_quantity,
            params: cfg.params.clone(),
        }
    };

    // 변경된 전략 설정을 디스크에 영구 저장 (프로파일별)
    let profile_id = state.profiles.read().await.active_id.clone();
    if let Some(pid) = &profile_id {
        let all_configs: Vec<crate::trading::strategy::StrategyConfig> = {
            let mgr = state.strategy_manager.lock().await;
            mgr.all_configs().into_iter().cloned().collect()
        };
        if let Err(e) = state.strategy_store.save(pid, &all_configs).await {
            tracing::warn!("전략 설정 저장 실패 (프로파일 {}): {}", pid, e);
        }
    }

    Ok(view)
}

// ────────────────────────────────────────────────────────────────────
// 리스크 관리 설정 조회 / 변경 / 비상 정지 해제
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskConfigView {
    /// 일일 최대 손실 한도 (원, 양수)
    pub daily_loss_limit: i64,
    /// 단일 종목 최대 비중 (0.0~1.0)
    pub max_position_ratio: f64,
    /// 오늘 누적 손실 (음수)
    pub current_loss: i64,
    /// 손실 한도 소진율 (0.0 ~ 1.0+)
    pub loss_ratio: f64,
    /// 비상 정지 여부
    pub is_emergency_stop: bool,
    /// 추가 거래 가능 여부
    pub can_trade: bool,
}

#[tauri::command]
pub async fn get_risk_config(state: State<'_, AppState>) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    // 날짜가 바뀌면 자동으로 당일 손실 초기화
    risk.reset_if_new_day();
    Ok(RiskConfigView {
        daily_loss_limit: risk.daily_loss_limit,
        max_position_ratio: risk.max_position_ratio,
        current_loss: risk.current_loss(),
        loss_ratio: risk.loss_ratio(),
        is_emergency_stop: risk.is_emergency_stop(),
        can_trade: risk.can_trade(),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRiskConfigInput {
    pub daily_loss_limit: Option<i64>,
    /// 0.01 ~ 1.0 (1% ~ 100%)
    pub max_position_ratio: Option<f64>,
}

#[tauri::command]
pub async fn update_risk_config(
    input: UpdateRiskConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    if let Some(limit) = input.daily_loss_limit {
        if limit < 0 {
            return Err(CmdError {
                code: "INVALID_PARAM".into(),
                message: "손실 한도는 0 이상이어야 합니다.".into(),
            });
        }
        risk.daily_loss_limit = limit;
    }
    if let Some(ratio) = input.max_position_ratio {
        if !(0.0..=1.0).contains(&ratio) {
            return Err(CmdError {
                code: "INVALID_PARAM".into(),
                message: "포지션 비중은 0.0~1.0 범위여야 합니다.".into(),
            });
        }
        risk.max_position_ratio = ratio;
    }
    tracing::info!(
        "리스크 설정 변경: 일일손실한도={}원, 종목비중={:.0}%",
        risk.daily_loss_limit,
        risk.max_position_ratio * 100.0
    );
    Ok(RiskConfigView {
        daily_loss_limit: risk.daily_loss_limit,
        max_position_ratio: risk.max_position_ratio,
        current_loss: risk.current_loss(),
        loss_ratio: risk.loss_ratio(),
        is_emergency_stop: risk.is_emergency_stop(),
        can_trade: risk.can_trade(),
    })
}

/// 비상 정지 수동 해제
#[tauri::command]
pub async fn clear_emergency_stop(state: State<'_, AppState>) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    risk.clear_emergency_stop();
    Ok(RiskConfigView {
        daily_loss_limit: risk.daily_loss_limit,
        max_position_ratio: risk.max_position_ratio,
        current_loss: risk.current_loss(),
        loss_ratio: risk.loss_ratio(),
        is_emergency_stop: risk.is_emergency_stop(),
        can_trade: risk.can_trade(),
    })
}

// ────────────────────────────────────────────────────────────────────
// 미체결 주문 목록 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingOrderView {
    pub odno: String,
    pub symbol: String,
    pub symbol_name: String,
    /// "buy" | "sell"
    pub side: String,
    pub quantity: u64,
    pub timestamp: String,
    pub signal_reason: String,
}

#[tauri::command]
pub async fn get_pending_orders(state: State<'_, AppState>) -> CmdResult<Vec<PendingOrderView>> {
    let mgr = state.order_manager.lock().await;
    let views = mgr
        .pending_orders()
        .iter()
        .map(|p| PendingOrderView {
            odno: p.record.kis_order_id.clone().unwrap_or_default(),
            symbol: p.record.symbol.clone(),
            symbol_name: p.record.symbol_name.clone(),
            side: match &p.record.side {
                crate::storage::order_store::OrderSide::Buy => "buy".into(),
                crate::storage::order_store::OrderSide::Sell => "sell".into(),
            },
            quantity: p.record.quantity,
            timestamp: p.record.timestamp.clone(),
            signal_reason: p.signal_reason.clone(),
        })
        .collect();
    Ok(views)
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
    state.rest_client.read().await.set_api_debug(new_cfg.api_debug);

    // 파일 저장
    new_cfg.save_sync(&state.log_dir).map_err(CmdError::from)?;

    // 즉시 정리 실행
    crate::logging::cleanup(&state.log_dir, &new_cfg);

    tracing::info!(
        "로그 설정 변경: 보관 {}일, 최대 {}MB, API 진단={}",
        new_cfg.retention_days, new_cfg.max_size_mb, new_cfg.api_debug
    );

    Ok(new_cfg)
}

// ────────────────────────────────────────────────────────────────────
// 체결 기록 보관 설정 조회 / 변경 / 통계
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_trade_archive_config(state: State<'_, AppState>) -> CmdResult<TradeArchiveConfig> {
    Ok(state.trade_archive_config.read().await.clone())
}

#[derive(Debug, Deserialize)]
pub struct SetTradeArchiveConfigInput {
    pub retention_days: u32,
    pub max_size_mb: u64,
}

#[tauri::command]
pub async fn set_trade_archive_config(
    input: SetTradeArchiveConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<TradeArchiveConfig> {
    let new_cfg = TradeArchiveConfig {
        retention_days: input.retention_days.clamp(1, 3650),
        max_size_mb: input.max_size_mb.clamp(50, 102400),
    };

    *state.trade_archive_config.write().await = new_cfg.clone();
    new_cfg.save_sync(&state.data_dir).map_err(|e| CmdError { code: "SAVE_ERR".into(), message: e })?;

    // 즉시 정리 실행
    let data_dir = state.data_dir.clone();
    let cfg_clone = new_cfg.clone();
    tokio::task::spawn_blocking(move || purge_old_trade_files(&data_dir, &cfg_clone));

    tracing::info!(
        "체결 기록 보관 설정 변경: 보관 {}일, 최대 {}MB",
        new_cfg.retention_days, new_cfg.max_size_mb
    );

    Ok(new_cfg)
}

#[tauri::command]
pub async fn get_trade_archive_stats(state: State<'_, AppState>) -> CmdResult<TradeArchiveStats> {
    let data_dir = state.data_dir.clone();
    let stats = tokio::task::spawn_blocking(move || {
        let day_dirs = collect_trade_day_dirs(&data_dir);
        let mut total_files: u64 = 0;
        let mut size_bytes: u64 = 0;
        for (_, dir) in &day_dirs {
            // trades.json 파일 수 카운트
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if entry.path().is_file() {
                        total_files += 1;
                        size_bytes += entry.path().metadata().map(|m| m.len()).unwrap_or(0);
                    }
                }
            }
        }
        let oldest_date = day_dirs.first().map(|(d, _)| d.format("%Y-%m-%d").to_string());
        let newest_date = day_dirs.last().map(|(d, _)| d.format("%Y-%m-%d").to_string());
        TradeArchiveStats { total_files, size_bytes, oldest_date, newest_date }
    })
    .await
    .map_err(|e| CmdError { code: "TASK_ERR".into(), message: e.to_string() })?;

    Ok(stats)
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
    if query.len() < 2 {
        return Ok(vec![]);
    }

    // ① 6자리 영숫자 코드 입력 → KIS 현재가에서 이름 확인 (0005A0 등 ETF 코드 포함)
    if query.len() == 6 && query.chars().all(|c| c.is_ascii_alphanumeric()) {
        let code = query.to_uppercase();
        // StockStore에 이미 있으면 빠르게 반환
        if let Some(name) = state.stock_store.get_name(&code).await {
            return Ok(vec![StockSearchItem { pdno: code, prdt_name: name }]);
        }
        // 없으면 KIS get_price로 확인
        let client = state.rest_client.read().await.clone();
        if let Ok(p) = client.get_price(&code).await {
            if !p.hts_kor_isnm.is_empty() {
                state.stock_store.upsert(&code, &p.hts_kor_isnm).await;
                return Ok(vec![StockSearchItem { pdno: code, prdt_name: p.hts_kor_isnm }]);
            }
        }
        // KIS 실패 시 Yahoo Finance로 이름 조회 (설정 없이도 동작)
        tracing::debug!("KIS 현재가 실패 → Yahoo Finance로 종목명 조회: {}", code);
        match crate::market::lookup_name_by_code(&code).await {
            Ok(name) => {
                tracing::info!("Yahoo Finance 이름 조회 성공: {} → {}", code, name);
                state.stock_store.upsert(&code, &name).await;
                return Ok(vec![StockSearchItem { pdno: code, prdt_name: name }]);
            }
            Err(e) => {
                tracing::warn!("Yahoo Finance 이름 조회 실패: {} — {}", code, e);
                return Ok(vec![]);
            }
        }
    }

    // ② StockStore(영구 캐시) 검색 — 우선순위 최상
    let local_results = state.stock_store.search(&query, 20).await;
    if !local_results.is_empty() {
        tracing::debug!("StockStore 검색: query={:?}, {}개 결과", query, local_results.len());
        return Ok(local_results);
    }

    // ③ KRX 레거시 캐시 검색 (stock_list — KRX 다운로드 성공 시에만 유효)
    {
        let stock_list = state.stock_list.read().await;
        if !stock_list.is_empty() {
            let results = crate::market::search_local(&stock_list, &query, 20);
            if !results.is_empty() {
                tracing::debug!("KRX 캐시 검색: query={:?}, {}개 결과", query, results.len());
                return Ok(results);
            }
        }
    }

    // ④ NAVER Finance 실시간 검색 폴백
    tracing::info!("search_stock: 로컬 검색 결과 없음 → NAVER 실시간 검색 (query={:?})", query);
    match crate::market::search_naver_live(&query).await {
        Ok(results) if !results.is_empty() => {
            tracing::info!("NAVER 검색 성공: {}개 결과 (query={:?})", results.len(), query);
            // NAVER 결과도 StockStore에 캐시
            state.stock_store.upsert_many(
                results.iter().map(|r| (r.pdno.clone(), r.prdt_name.clone()))
            ).await;
            return Ok(results);
        }
        Ok(_) => {
            tracing::debug!("NAVER 검색 결과 없음 (query={:?})", query);
            return Ok(vec![]);
        }
        Err(e) => {
            tracing::warn!("NAVER 검색 실패: {} (query={:?})", e, query);
            return Err(CmdError {
                code: "STOCK_LIST_EMPTY".into(),
                message: "종목 검색에 실패했습니다. 네트워크 연결을 확인하거나 '종목 목록 새로고침'을 눌러주세요.".into(),
            });
        }
    }
}

// ── 종목 목록 새로고침 ─────────────────────────────────────────────
#[tauri::command]
pub async fn refresh_stock_list(state: State<'_, AppState>) -> CmdResult<usize> {
    tracing::info!("수동 종목 목록 새로고침 시작 (KRX 다운로드 시도)...");
    let items = crate::market::StockList::fetch_from_krx()
        .await
        .map_err(CmdError::from)?;

    if items.is_empty() {
        tracing::warn!(
            "KRX 다운로드 결과가 0개입니다. \
             KRX 데이터 포털(data.krx.co.kr)이 봇 차단(WAF)을 적용 중이거나 \
             네트워크 문제일 수 있습니다. \
             종목 검색은 NAVER Finance 실시간 검색으로 자동 대체됩니다."
        );
        return Err(CmdError {
            code: "KRX_EMPTY".into(),
            message: "KRX에서 종목 목록을 가져오지 못했습니다 (0개). 종목 검색은 실시간 검색으로 동작합니다.".into(),
        });
    }

    let count = items.len();

    // 메모리 갱신
    *state.stock_list.write().await = items.clone();

    // 캐시 파일 갱신
    let cache_path = state.data_dir.join("stock_list.json");
    if let Some(dir) = cache_path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(&items) {
        let _ = std::fs::write(&cache_path, json);
    }

    tracing::info!("종목 목록 수동 갱신 완료: {}개", count);
    Ok(count)
}

// ── 종목 목록 통계 조회 ────────────────────────────────────────────
#[tauri::command]
pub async fn get_stock_list_stats(state: State<'_, AppState>) -> CmdResult<StockListStats> {
    let count = state.stock_store.size().await;
    let last_updated_at = state.stock_store.last_updated_at().await;
    let update_interval_hours = state.stock_store.get_interval_hours().await;
    let file_path = state.data_dir
        .join("stocklist")
        .join("stocklist.json")
        .to_string_lossy()
        .to_string();
    Ok(StockListStats {
        count,
        last_updated_at,
        file_path,
        update_interval_hours,
    })
}

// ── 종목 목록 자동 갱신 간격 설정 ────────────────────────────────
#[tauri::command]
pub async fn set_stock_update_interval(
    hours: u32,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    state.stock_store.set_interval_hours(hours).await.map_err(CmdError::from)?;
    tracing::info!("종목 목록 갱신 간격 변경: {}시간", hours);
    Ok(())
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

// ────────────────────────────────────────────────────────────────────
// 해외(미국) 주식 현재가 조회
// ────────────────────────────────────────────────────────────────────

/// 해외 현재가 뷰 (camelCase → TypeScript 1:1)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverseasPriceView {
    pub symbol: String,
    pub exchange: String,
    pub name: String,
    pub last: String,
    pub diff: String,
    pub rate: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub h52p: String,
    pub l52p: String,
    pub tvol: String,
}

/// 해외 주문 입력 (TypeScript PlaceOverseasOrderInput 1:1)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverseasOrderInput {
    pub symbol: String,
    pub exchange: String, // NASD / NYSE / AMEX
    pub side: String,
    pub price: f64,
    pub quantity: u64,
}

#[tauri::command]
pub async fn get_overseas_chart_data(
    symbol: String,
    exchange: String,
    period_code: String, // "D", "W", "M"
    base_date: String,   // YYYYMMDD — 비워두면 당일 기준
    state: State<'_, AppState>,
) -> CmdResult<Vec<ChartCandle>> {
    let client = state.rest_client.read().await.clone();
    client
        .get_overseas_chart_data(&symbol, &exchange, &period_code, &base_date)
        .await
        .map_err(CmdError::from)
}

#[tauri::command]
pub async fn get_overseas_price(
    symbol: String,
    exchange: String,
    state: State<'_, AppState>,
) -> CmdResult<OverseasPriceView> {
    let client = state.rest_client.read().await.clone();
    let resp = client
        .get_overseas_price(&symbol, &exchange)
        .await
        .map_err(CmdError::from)?;

    Ok(OverseasPriceView {
        symbol,
        exchange,
        name:  resp.name,
        last:  resp.last,
        diff:  resp.diff,
        rate:  resp.rate,
        open:  resp.open,
        high:  resp.high,
        low:   resp.low,
        h52p:  resp.h52p,
        l52p:  resp.l52p,
        tvol:  resp.tvol,
    })
}

#[tauri::command]
pub async fn place_overseas_order(
    input: OverseasOrderInput,
    state: State<'_, AppState>,
) -> CmdResult<OrderResponse> {
    use crate::api::rest::{OrderSide, OverseasOrderRequest};

    tracing::info!(
        "해외 주문 요청: {} {} {} 수량={} 가격={}",
        input.exchange, input.symbol, input.side, input.quantity, input.price
    );

    let side = match input.side.as_str() {
        "Buy" => OrderSide::Buy,
        _ => OrderSide::Sell,
    };

    let req = OverseasOrderRequest {
        symbol: input.symbol.clone(),
        exchange: input.exchange.clone(),
        side,
        quantity: input.quantity,
        price: input.price,
    };

    let client = state.rest_client.read().await.clone();
    match client.place_overseas_order(&req).await {
        Ok(resp) => {
            tracing::info!(
                "해외 주문 완료: {} {} — 주문번호={}, 시각={}",
                input.exchange, input.symbol, resp.odno, resp.ord_tmd
            );
            Ok(resp)
        }
        Err(e) => {
            tracing::error!(
                "해외 주문 실패: {} {} 수량={} 가격={} — {}",
                input.exchange, input.symbol, input.quantity, input.price, e
            );
            Err(CmdError::from(e))
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 자동매매 폴링 루프 헬퍼
// ────────────────────────────────────────────────────────────────────

/// 장 마감 / 장외 시간 오류 여부 감지
///
/// KIS API가 시장 비운영 시간에 반환하는 공통 메시지 패턴을 검사한다.
///
/// ## 실제 KIS 응답 예시 (에러 로그에서 수집)
/// - `"모의투자 장종료 입니다."`
/// - `"모의투자 장시작전 입니다."`
/// - `"장운영시간이 아닙니다."`
/// - `"시간외거래"`
fn is_market_closed_error(msg: &str) -> bool {
    msg.contains("장종료")
        || msg.contains("장마감")
        || msg.contains("장시작전")
        || msg.contains("장운영시간")
        || msg.contains("시간외거래")
        || msg.contains("OPCODE-100")
}

/// 국내 주식 종목코드 판별 — `crate::market_hours::is_domestic_symbol` 에서 재공개
// (이 함수는 market_hours.rs로 이전됨)

/// 해외 주식 현재가 조회 (NAS → NYS → AMS 순으로 시도)
/// 반환값: (price_cents: u64, volume: u64)
/// - price_cents = USD 현재가 × 100 (정수화하여 on_tick에 전달)
async fn fetch_overseas_tick(
    rest: &std::sync::Arc<crate::api::rest::KisRestClient>,
    symbol: &str,
) -> anyhow::Result<(u64, u64)> {
    for exchange in &["NAS", "NYS", "AMS"] {
        match rest.get_overseas_price(symbol, exchange).await {
            Ok(p) => {
                let price_f: f64 = p.last.parse().unwrap_or(0.0);
                if price_f > 0.0 {
                    // USD → 센트(×100) 변환으로 u64 정수화
                    let price_cents = (price_f * 100.0).round() as u64;
                    let volume = p.tvol.parse::<u64>().unwrap_or(0);
                    return Ok((price_cents, volume));
                }
            }
            Err(_) => continue,
        }
    }
    anyhow::bail!("해외 현재가 조회 실패: {} (NAS/NYS/AMS 모두 실패)", symbol)
}
