/// 모바일/외부 접속용 경량 웹 서버 (axum)
///
/// 엔드포인트:
///   GET  /                                    → React 앱(dist/) 또는 모바일 대시보드 HTML
///   GET  /api/info                            → 앱 정보 JSON
///   GET  /api/balance                         → 잔고 JSON
///   GET  /api/broker-holdings                 → 활성 broker 보유 종목 JSON
///   GET  /api/toss-market-snapshot/:symbol    → Toss 현재가/호가/체결/상하한가 JSON
///   GET  /api/toss-stock-safety/:symbol        → Toss 종목 기본 정보/매수 유의사항 JSON
///   POST /api/toss-order-preflight             → Toss 주문 전 read-only 검증 JSON
///   POST /api/toss-open-orders                 → Toss OPEN 주문 목록 JSON
///   POST /api/toss-order-modify                → Toss 접수 주문 정정
///   POST /api/toss-small-buy-verification      → Toss 소액매매 검증 1주 시장가 매수 실행
///   GET  /api/toss-market-calendar             → Toss KR/US 정규장 캘린더 JSON
///   GET  /api/toss-chart/:symbol              → Toss candles JSON (?interval=1d&count=200)
///   POST /api/strategy/leveraged-trend-hold/preview → Toss 1분봉 기반 레버리지 전략 미리보기
///   POST /api/strategy/preview            → 제공된 캔들 기반 범용 전략 미리보기
///   GET  /api/price/:symbol                   → 국내 현재가 JSON
///   GET  /api/overseas-price/:ex/:sym         → 해외 현재가 JSON (NAS/NYS/AMS)
///   GET  /api/executed                        → 당일 체결 JSON
///   GET  /api/search/:query                   → 종목 검색 (KRX 로컬 캐시)
///   POST /api/order                           → 국내 주문 실행
///   POST /api/overseas-order                  → 해외 주문 실행
///   GET  /api/chart/:symbol                   → 국내 차트 데이터 (?period=D&count=100)
///   GET  /api/overseas-chart/:ex/:symbol      → 해외 차트 데이터 (?period=D&count=100)
///   GET  /api/trading/status                  → 자동매매 상태 JSON
///   POST /api/trading/start                   → 자동매매 시작 (is_trading=true)
///   POST /api/trading/stop                    → 자동매매 정지 (is_trading=false)
///   GET  /api/strategies                      → 활성 전략 목록
///   POST /api/strategies/:id                  → 전략 파라미터 업데이트
///   GET  /api/check-config                    → 설정 진단
///   POST /api/profiles/add                    → 프로파일 추가
///   POST /api/profiles/update                 → 프로파일 수정
///   POST /api/profiles/delete                 → 프로파일 삭제
///   POST /api/profiles/:id/set-active         → 활성 프로파일 변경
///   POST /api/profiles/:id/detect             → 실전/모의 자동 감지 + 저장
///   POST /api/toss-accounts                   → 입력한 토스 키로 accountSeq 목록 조회
///   POST /api/profiles/:id/toss-accounts      → 저장된 토스 프로파일 accountSeq 목록 조회
///   POST /api/detect-trading-type             → 입력 키로 실전/모의 자동 감지
///   GET  /api/stock-list-stats                → 종목 목록 통계
///   POST /api/stock-update-interval           → 종목 목록 갱신 주기 변경
///   POST /api/refresh-stock-list              → 종목 목록 즉시 갱신
///   POST /api/test-discord                    → Discord 테스트 알림 전송
///   GET  /api/today-trades                    → 당일 체결 기록 (로컬 JSON)
///   GET  /api/check-update                    → 업데이트 확인 (웹 모드: 항상 최신)
///   POST /api/web-config/save                 → 웹 포트 저장 (.env WEB_PORT)
///   GET  /api/exchange-rate                   → USD/KRW 환율
///   GET  /api/exchange-rate/status            → USD/KRW 환율 출처/유효시간
///   GET  /api/refresh-interval                → 갱신 주기(초)
///   POST /api/buy-suspension/clear            → 매수 정지 해제
///   POST /api/activate-emergency              → 비상 정지 수동 활성화
///   POST /api/save-trade                      → 체결 기록 저장
///   POST /api/upsert-stats                    → 일별 통계 저장/갱신
///   POST /api/frontend-log                    → 프론트엔드 로그 기록
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};
use tower_http::cors::CorsLayer;

use crate::api::rest::{KisRestClient, StockSearchItem};
use crate::broker::BrokerId;
use crate::commands::{build_risk_view, ExchangeRateView, RefreshConfig, TradeArchiveConfig};
use crate::config::{AppConfig, ProfilesConfig};
use crate::logging::LogConfig;
use crate::notifications::discord::DiscordNotifier;
use crate::storage::{
    database::DatabaseManager, order_store::OrderStore, stats_store::StatsStore,
    stock_store::StockStore, strategy_store::StrategyStore, trade_store::TradeStore,
};
use crate::trading::{
    order::OrderManager, position::PositionTracker, risk::RiskManager, strategy::StrategyManager,
};

mod market;
mod profiles;
mod records;
mod toss;
mod trading;
use market::*;
use profiles::*;
use records::*;
use toss::*;
use trading::*;

#[derive(Clone)]
struct ServerState {
    rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    stock_list: Arc<RwLock<Vec<StockSearchItem>>>,
    web_port: u16,
    dist_path: PathBuf,
    /// dist/index.html 존재 여부 (서버 시작 시 평가)
    dist_found: bool,
    /// 자동매매 활성 여부 (commands.rs AppState 와 Arc 공유)
    is_trading: Arc<Mutex<bool>>,
    /// DB backend 관리와 자동매매 시작을 직렬화한다.
    storage_maintenance: Arc<Mutex<()>>,
    database_manager: Arc<DatabaseManager>,
    /// 전략 관리자
    strategy_manager: Arc<Mutex<StrategyManager>>,
    strategy_update_lock: Arc<Mutex<()>>,
    /// 포지션 트래커
    position_tracker: Arc<Mutex<PositionTracker>>,
    /// 활성 프로파일 설정 (AppConfig)
    config: Arc<RwLock<Arc<AppConfig>>>,
    /// 계좌 프로파일 목록
    profiles: Arc<RwLock<ProfilesConfig>>,
    /// 체결 기록 저장소
    trade_store: Arc<TradeStore>,
    /// 주문 이력 저장소
    order_store: Arc<OrderStore>,
    /// 일별 통계 저장소
    stats_store: Arc<StatsStore>,
    /// 로그 설정 (AppState 와 Arc 공유)
    log_config: Arc<RwLock<LogConfig>>,
    /// 로그 디렉토리 경로
    log_dir: PathBuf,
    /// 체결 기록 보관 설정
    trade_archive_config: Arc<RwLock<TradeArchiveConfig>>,
    /// 데이터 디렉토리 경로
    data_dir: PathBuf,
    /// 리스크 관리자
    risk_manager: Arc<Mutex<RiskManager>>,
    /// 주문 관리자 (미체결 주문 조회용)
    order_manager: Arc<Mutex<OrderManager>>,
    /// 영구 종목목록 캐시 (전략 업데이트 시 종목명 조회용)
    stock_store: Arc<StockStore>,
    /// 전략 설정 저장소
    strategy_store: Arc<StrategyStore>,
    /// 프로파일 저장 경로 (profiles.json)
    profiles_path: PathBuf,
    /// Discord 알림 (테스트 발송용)
    discord: Option<Arc<DiscordNotifier>>,
    /// USD/KRW 환율 캐시 (AppState와 Arc 공유)
    exchange_rate_krw: Arc<RwLock<f64>>,
    /// USD/KRW 환율 출처/유효시간 메타데이터
    exchange_rate_status: Arc<RwLock<ExchangeRateView>>,
    /// 데이터 갱신 주기 설정 (AppState와 Arc 공유)
    refresh_config: Arc<RwLock<RefreshConfig>>,
}

/// 서버 시작 (포트 바인드 실패 시 경고만 내고 종료)
#[allow(clippy::too_many_arguments)]
pub async fn start(
    rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    stock_list: Arc<RwLock<Vec<StockSearchItem>>>,
    port: u16,
    is_trading: Arc<Mutex<bool>>,
    storage_maintenance: Arc<Mutex<()>>,
    database_manager: Arc<DatabaseManager>,
    strategy_manager: Arc<Mutex<StrategyManager>>,
    strategy_update_lock: Arc<Mutex<()>>,
    position_tracker: Arc<Mutex<PositionTracker>>,
    config: Arc<RwLock<Arc<AppConfig>>>,
    profiles: Arc<RwLock<ProfilesConfig>>,
    order_store: Arc<OrderStore>,
    trade_store: Arc<TradeStore>,
    stats_store: Arc<StatsStore>,
    log_config: Arc<RwLock<LogConfig>>,
    log_dir: PathBuf,
    trade_archive_config: Arc<RwLock<TradeArchiveConfig>>,
    data_dir: PathBuf,
    risk_manager: Arc<Mutex<RiskManager>>,
    order_manager: Arc<Mutex<OrderManager>>,
    stock_store: Arc<StockStore>,
    strategy_store: Arc<StrategyStore>,
    profiles_path: PathBuf,
    discord: Option<Arc<DiscordNotifier>>,
    exchange_rate_krw: Arc<RwLock<f64>>,
    refresh_config: Arc<RwLock<RefreshConfig>>,
    exchange_rate_status: Arc<RwLock<ExchangeRateView>>,
) {
    let dist_path = web_dist_path();
    let dist_found = dist_path.join("index.html").exists();

    if dist_found {
        tracing::info!("웹 모드: React 앱을 {:?} 에서 서비스합니다", dist_path);
    } else {
        tracing::warn!(
            "웹 모드: dist/index.html 없음 ({:?}) — 설치 안내 페이지를 서비스합니다. \
             'npm run build' 실행 후 앱을 재시작하거나 DIST_PATH 환경 변수를 설정하세요.",
            dist_path
        );
    }

    let state = ServerState {
        rest_client,
        stock_list,
        web_port: port,
        dist_path,
        dist_found,
        is_trading,
        storage_maintenance,
        database_manager,
        strategy_manager,
        strategy_update_lock,
        position_tracker,
        config,
        profiles,
        order_store,
        trade_store,
        stats_store,
        log_config,
        log_dir,
        trade_archive_config,
        data_dir,
        risk_manager,
        order_manager,
        stock_store,
        strategy_store,
        profiles_path,
        discord,
        exchange_rate_krw,
        exchange_rate_status,
        refresh_config,
    };

    let app = Router::new()
        .route("/api/info", get(info_handler))
        .route("/api/app-config", get(app_config_handler))
        .route("/api/profiles", get(profiles_handler))
        .route("/api/balance", get(balance_handler))
        .route("/api/overseas-balance", get(overseas_balance_handler))
        .route("/api/broker-holdings", get(broker_holdings_handler))
        .route(
            "/api/toss-market-snapshot/:symbol",
            get(toss_market_snapshot_handler),
        )
        .route(
            "/api/toss-stock-safety/:symbol",
            get(toss_stock_safety_handler),
        )
        .route(
            "/api/toss-market-calendar",
            get(toss_market_calendar_handler),
        )
        .route("/api/toss-chart/:symbol", get(toss_chart_handler))
        .route(
            "/api/strategy/leveraged-trend-hold/preview",
            post(leveraged_trend_hold_preview_handler),
        )
        .route("/api/positions", get(positions_handler))
        .route("/api/price/:symbol", get(price_handler))
        .route(
            "/api/overseas-price/:ex/:symbol",
            get(overseas_price_handler),
        )
        .route("/api/executed", get(executed_handler))
        .route("/api/kis-executed", get(kis_executed_handler))
        .route("/api/pending-orders", get(pending_orders_handler))
        .route("/api/search/:query", get(search_handler))
        .route("/api/order", post(order_handler))
        .route("/api/overseas-order", post(overseas_order_handler))
        .route("/api/chart/:symbol", get(chart_handler))
        .route(
            "/api/overseas-chart/:ex/:symbol",
            get(overseas_chart_handler),
        )
        .route("/api/today-stats", get(today_stats_handler))
        .route("/api/stats", get(stats_by_range_handler))
        .route("/api/trades", get(trades_by_range_handler))
        .route(
            "/api/log-config",
            get(log_config_handler).post(set_log_config_handler),
        )
        .route("/api/recent-logs", get(recent_logs_handler))
        .route(
            "/api/archive-config",
            get(archive_config_handler).post(set_archive_config_handler),
        )
        .route("/api/archive-stats", get(archive_stats_handler))
        .route(
            "/api/risk-config",
            get(risk_config_handler).post(update_risk_config_handler),
        )
        .route(
            "/api/risk-config/clear-emergency",
            post(clear_emergency_handler),
        )
        .route("/api/web-config", get(web_config_handler))
        // ── 자동매매 제어 ──
        .route("/api/trading/status", get(trading_status_handler))
        .route("/api/trading/start", post(trading_start_handler))
        .route("/api/trading/stop", post(trading_stop_handler))
        .route("/api/strategies", get(strategies_handler))
        .route("/api/strategies/:id", post(update_strategy_handler))
        .route("/api/strategy/preview", post(strategy_preview_handler))
        // ── 프로파일 관리 ──
        .route("/api/profiles/add", post(add_profile_handler))
        .route("/api/profiles/update", post(update_profile_handler))
        .route("/api/profiles/delete", post(delete_profile_handler))
        .route(
            "/api/profiles/:id/set-active",
            post(set_active_profile_handler),
        )
        .route("/api/profiles/:id/detect", post(detect_profile_handler))
        .route("/api/toss-accounts", post(toss_accounts_handler))
        .route(
            "/api/profiles/:id/toss-accounts",
            post(toss_profile_accounts_handler),
        )
        .route(
            "/api/profiles/:id/toss-diagnostic",
            post(toss_profile_diagnostic_handler),
        )
        .route(
            "/api/toss-order-preflight",
            post(toss_order_preflight_handler),
        )
        .route("/api/toss-open-orders", post(toss_open_orders_handler))
        .route("/api/toss-order-modify", post(toss_modify_order_handler))
        .route(
            "/api/toss-small-buy-verification",
            post(toss_small_buy_verification_handler),
        )
        // ── 설정 진단 / 감지 ──
        .route("/api/check-config", get(check_config_handler))
        .route(
            "/api/detect-trading-type",
            post(detect_trading_type_handler),
        )
        // ── 종목 목록 ──
        .route("/api/stock-list-stats", get(stock_list_stats_handler))
        .route(
            "/api/stock-update-interval",
            post(set_stock_update_interval_handler),
        )
        .route("/api/refresh-stock-list", post(refresh_stock_list_handler))
        // ── Discord 테스트 ──
        .route("/api/test-discord", post(test_discord_handler))
        // ── 당일 체결 기록 ──
        .route("/api/today-trades", get(today_trades_handler))
        // ── 업데이트 확인 ──
        .route("/api/check-update", get(check_update_handler))
        // ── 웹 설정 저장 ──
        .route("/api/web-config/save", post(save_web_config_handler))
        // ── 환율 / 갱신 주기 ──
        .route("/api/exchange-rate", get(exchange_rate_handler))
        .route(
            "/api/exchange-rate/status",
            get(exchange_rate_status_handler),
        )
        .route("/api/refresh-interval", get(refresh_interval_handler))
        // ── 매수 정지 / 비상 정지 ──
        .route(
            "/api/buy-suspension/clear",
            post(clear_buy_suspension_handler),
        )
        .route("/api/activate-emergency", post(activate_emergency_handler))
        // ── 체결 기록 / 통계 저장 ──
        .route("/api/save-trade", post(save_trade_handler))
        .route("/api/upsert-stats", post(upsert_stats_handler))
        // ── 프론트엔드 로그 ──
        .route("/api/frontend-log", post(frontend_log_handler))
        .fallback(get(spa_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!(
                "웹 서버 시작: http://0.0.0.0:{} (같은 네트워크에서 접속 가능)",
                port
            );
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("웹 서버 종료: {}", e);
            }
        }
        Err(e) => {
            tracing::warn!("웹 서버 포트 {} 바인드 실패: {} — 웹 접속 비활성", port, e);
        }
    }
}

/// dist/ 폴더 경로 탐색
///
/// 탐색 순서:
///   1. `DIST_PATH` 환경 변수 (사용자 직접 지정, 최우선)
///   2. 실행 파일 기준 상위 디렉토리 최대 5단계 탐색
///      - 디버그 빌드: src-tauri/target/debug/exe → 프로젝트 루트(4단계 위)
///      - macOS .app: Contents/MacOS/exe → ../Resources/dist 탐색
///   3. 현재 작업 디렉토리 `dist/` (cwd에서 실행할 경우)
fn web_dist_path() -> PathBuf {
    // 1. DIST_PATH 환경 변수 최우선
    if let Ok(env_path) = std::env::var("DIST_PATH") {
        let p = PathBuf::from(&env_path);
        if p.join("index.html").exists() {
            tracing::info!("dist/ 경로: DIST_PATH 환경 변수 = {:?}", p);
            return p;
        }
        tracing::warn!("DIST_PATH={:?} 에 index.html 없음 — 자동 탐색 시도", p);
    }

    // 2. exe 기준 상위 디렉토리 탐색 (5단계)
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(PathBuf::from);
        for depth in 0..5u32 {
            if let Some(ref d) = dir {
                // dist/ 직접 확인
                let candidate = d.join("dist");
                if candidate.join("index.html").exists() {
                    tracing::info!("dist/ 경로: exe 기준 {}단계 위 = {:?}", depth, candidate);
                    return candidate;
                }
                // macOS 앱 번들: Contents/MacOS/ → ../Resources/dist
                let resources = d.join("../Resources/dist");
                if resources.join("index.html").exists() {
                    let resolved = resources.canonicalize().unwrap_or(resources);
                    tracing::info!("dist/ 경로: macOS Resources = {:?}", resolved);
                    return resolved;
                }
                dir = d.parent().map(PathBuf::from);
            } else {
                break;
            }
        }
    }

    // 3. 현재 작업 디렉토리 기준 (cwd/dist)
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("dist")
}

// ── 핸들러 ──────────────────────────────────────────────────────────

/// SPA fallback: 요청된 파일이 dist/에 존재하면 서빙, 없으면 index.html 또는 모바일 대시보드 HTML 반환
async fn spa_handler(State(s): State<ServerState>, uri: Uri) -> Response {
    let req_path = uri.path().trim_start_matches('/');

    // 경로가 비어있지 않으면 dist/ 에서 실제 파일 서빙 시도
    if !req_path.is_empty() {
        let file_path = s.dist_path.join(req_path);
        // 경로 순회 공격 방지: dist_path 내부인지 확인
        if file_path.starts_with(&s.dist_path) && file_path.is_file() {
            if let Ok(bytes) = tokio::fs::read(&file_path).await {
                let mime = guess_mime(req_path);
                return ([(header::CONTENT_TYPE, mime)], bytes).into_response();
            }
        }
    }

    // SPA 라우팅 fallback: index.html 반환 또는 설치 안내 HTML
    let index_path = s.dist_path.join("index.html");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(content) => Html(content).into_response(),
        Err(_) => Html(SETUP_HTML).into_response(),
    }
}

/// 파일 확장자로 MIME 타입 추론
fn guess_mime(path: &str) -> &'static str {
    if path.ends_with(".js") || path.ends_with(".mjs") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}

async fn info_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "app": "KISAutoTrade",
        "version": env!("CARGO_PKG_VERSION"),
        "port": s.web_port,
        "mode": "web",
    }))
}

// ── 앱 설정 / 프로파일 ────────────────────────────────────────────

/// GET /api/app-config
async fn app_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let cfg = s.config.read().await.clone();
    let masked_key = if cfg.kis_app_key.len() > 6 {
        format!("{}****", &cfg.kis_app_key[..6])
    } else if cfg.kis_app_key.is_empty() {
        "(미설정)".into()
    } else {
        "****".into()
    };
    let (active_id, active_name, active_broker_id, active_account_id) = {
        let profiles = s.profiles.read().await;
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
    Json(serde_json::json!({
        "active_broker_id":          active_broker_id,
        "active_broker_account_id":  active_account_id,
        "kis_app_key_masked":   masked_key,
        "kis_account_no":       cfg.kis_account_no,
        "kis_is_paper_trading": cfg.kis_is_paper_trading,
        "kis_configured":       cfg.is_kis_configured(),
        "active_broker_configured": cfg.is_active_broker_configured(),
        "discord_enabled":      cfg.discord_bot_token.is_some(),
        "notification_levels":  cfg.notification_levels,
        "active_profile_id":    active_id,
        "active_profile_name":  active_name,
    }))
}

// ── 리스크 관리 ───────────────────────────────────────────────────

/// GET /api/risk-config
async fn risk_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mut risk = s.risk_manager.lock().await;
    risk.reset_if_new_day();
    Json(serde_json::to_value(build_risk_view(&risk)).unwrap_or_default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateRiskConfigBody {
    enabled: Option<bool>,
    daily_loss_limit: Option<i64>,
    max_position_ratio: Option<f64>,
    max_daily_buy_orders_per_symbol: Option<u32>,
    max_daily_sell_orders_per_symbol: Option<u32>,
    max_consecutive_losses_per_strategy_symbol: Option<u32>,
    volatility_sizing_enabled: Option<bool>,
    risk_per_trade_bps: Option<u32>,
    atr_stop_multiplier: Option<f64>,
}

/// POST /api/risk-config
async fn update_risk_config_handler(
    State(s): State<ServerState>,
    Json(body): Json<UpdateRiskConfigBody>,
) -> Json<serde_json::Value> {
    let mut risk = s.risk_manager.lock().await;
    if let Some(en) = body.enabled {
        risk.set_enabled(en);
    }
    if let Some(limit) = body.daily_loss_limit {
        if limit >= 0 {
            risk.daily_loss_limit = limit;
        }
    }
    if let Some(ratio) = body.max_position_ratio {
        if (0.0..=1.0).contains(&ratio) {
            risk.max_position_ratio = ratio;
        }
    }
    if body.max_daily_buy_orders_per_symbol.is_some() {
        risk.max_daily_buy_orders_per_symbol = 0;
    }
    if let Some(limit) = body.max_daily_sell_orders_per_symbol {
        risk.max_daily_sell_orders_per_symbol = limit;
    }
    if let Some(limit) = body.max_consecutive_losses_per_strategy_symbol {
        risk.max_consecutive_losses_per_strategy_symbol = limit;
    }
    if let Some(enabled) = body.volatility_sizing_enabled {
        risk.volatility_sizing_enabled = enabled;
    }
    if let Some(bps) = body.risk_per_trade_bps {
        if bps <= 10_000 {
            risk.risk_per_trade_bps = bps;
        }
    }
    if let Some(multiplier) = body.atr_stop_multiplier {
        if (0.1..=20.0).contains(&multiplier) {
            risk.atr_stop_multiplier = multiplier;
        }
    }
    Json(serde_json::to_value(build_risk_view(&risk)).unwrap_or_default())
}

/// POST /api/risk-config/clear-emergency
async fn clear_emergency_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mut risk = s.risk_manager.lock().await;
    risk.clear_emergency_stop();
    Json(serde_json::to_value(build_risk_view(&risk)).unwrap_or_default())
}

// ── 웹 설정 ───────────────────────────────────────────────────────

/// GET /api/web-config
async fn web_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "runningPort": s.web_port,
        "accessUrl":   format!("http://localhost:{}", s.web_port),
        "distPath":    s.dist_path.to_string_lossy(),
        "distFound":   s.dist_found,
    }))
}

// ── 설정 진단 ─────────────────────────────────────────────────────

/// GET /api/check-config
async fn check_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let cfg = s.config.read().await.clone();
    let mut issues: Vec<String> = Vec::new();
    match cfg.broker_id {
        BrokerId::Kis => {
            if cfg.kis_app_key.is_empty() {
                issues.push("KIS APP KEY가 설정되지 않았습니다.".into());
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
    let paper_available = {
        let p = s.profiles.read().await;
        p.profiles
            .iter()
            .any(|p| p.broker_id == BrokerId::Kis && p.is_paper_trading && p.is_configured())
    };
    Json(serde_json::json!({
        "broker_id":           cfg.broker_id,
        "broker_account_id":   if cfg.broker_account_id.is_empty() { None::<String> } else { Some(cfg.broker_account_id.clone()) },
        "real_key_set":       !cfg.kis_app_key.is_empty(),
        "real_account_set":   !cfg.kis_account_no.is_empty(),
        "paper_key_set":      paper_available,
        "active_mode":        match cfg.broker_id {
            BrokerId::Kis if cfg.kis_is_paper_trading => "모의투자",
            BrokerId::Kis => "실전투자",
            BrokerId::Toss => "실전투자",
        },
        "is_ready":           cfg.is_active_broker_configured(),
        "discord_configured": cfg.discord_bot_token.is_some(),
        "base_url":           cfg.kis_base_url(),
        "issues":             issues,
    }))
}

// ── Discord 테스트 ────────────────────────────────────────────────

/// POST /api/test-discord
async fn test_discord_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    match &s.discord {
        Some(discord) => {
            let event = crate::notifications::types::NotificationEvent::info(
                "테스트 알림",
                "KISAutoTrade 알림 시스템이 정상 작동 중입니다.",
            );
            match discord.send(event).await {
                Ok(_) => Json(
                    serde_json::json!({ "ok": true, "message": "Discord 테스트 알림 전송 완료" }),
                ),
                Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
            }
        }
        None => Json(serde_json::json!({ "error": "Discord 봇이 설정되지 않았습니다." })),
    }
}

// ── 업데이트 확인 ─────────────────────────────────────────────────

/// GET /api/check-update — 웹 모드에서는 항상 "최신 버전"
async fn check_update_handler(_s: State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "hasUpdate":    false,
        "current":      env!("CARGO_PKG_VERSION"),
        "latest":       env!("CARGO_PKG_VERSION"),
        "releaseNotes": null,
        "downloadUrl":  null,
    }))
}

// ── 웹 설정 저장 ──────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveWebConfigBody {
    new_port: u16,
    /// DIST_PATH 환경 변수에 저장할 dist/ 경로 (비어있으면 수정 안 함)
    dist_path: Option<String>,
}

/// POST /api/web-config/save — .env WEB_PORT (및 선택적 DIST_PATH) 저장 (재시작 후 반영)
async fn save_web_config_handler(
    State(_s): State<ServerState>,
    Json(body): Json<SaveWebConfigBody>,
) -> Json<serde_json::Value> {
    use std::io::Write;
    let env_path = std::env::current_dir().unwrap_or_default().join(".env");
    let mut content = format!("WEB_PORT={}\n", body.new_port);
    if let Some(ref dp) = body.dist_path {
        if !dp.is_empty() {
            content.push_str(&format!("DIST_PATH={}\n", dp));
        }
    }
    match std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&env_path)
        .and_then(|mut f| f.write_all(content.as_bytes()))
    {
        Ok(_) => Json(
            serde_json::json!({ "ok": true, "message": format!(".env 저장 완료: WEB_PORT={}", body.new_port) }),
        ),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ── 환율 / 갱신 주기 ─────────────────────────────────────────────

/// GET /api/exchange-rate
async fn exchange_rate_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!(*s.exchange_rate_krw.read().await))
}

/// GET /api/exchange-rate/status
async fn exchange_rate_status_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(
        serde_json::to_value(s.exchange_rate_status.read().await.clone()).unwrap_or_else(
            |e| serde_json::json!({ "error": format!("환율 상태 직렬화 실패: {}", e) }),
        ),
    )
}

/// GET /api/refresh-interval
async fn refresh_interval_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!(
        s.refresh_config.read().await.interval_sec
    ))
}

// ── 매수 정지 / 비상 정지 ─────────────────────────────────────────

/// POST /api/buy-suspension/clear
async fn clear_buy_suspension_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    s.order_manager.lock().await.clear_buy_suspension();
    tracing::info!("매수 정지 해제 (웹 API)");
    Json(serde_json::json!({ "ok": true }))
}

/// POST /api/activate-emergency
async fn activate_emergency_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mut risk = s.risk_manager.lock().await;
    risk.trigger_emergency_stop();
    tracing::warn!("비상 정지 수동 활성화 (웹 API)");
    Json(serde_json::to_value(build_risk_view(&risk)).unwrap_or_default())
}

// ── 프론트엔드 미빌드 시 안내 페이지 ────────────────────────────────

static SETUP_HTML: &str = r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>KISAutoTrade — 설정 필요</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, sans-serif; background: #121212; color: #e0e0e0;
           max-width: 640px; margin: 60px auto; padding: 24px; line-height: 1.6; }
    h1 { color: #90caf9; font-size: 22px; margin-bottom: 6px; }
    .subtitle { color: #757575; font-size: 13px; margin-bottom: 24px; }
    .warn { background: #2c1500; border: 1px solid #f57c00; border-radius: 8px;
            padding: 14px 16px; margin: 16px 0; color: #ffb74d; font-size: 14px; }
    pre { background: #1a1a1a; border: 1px solid #333; border-radius: 8px; padding: 16px;
          font-size: 13px; color: #80cbc4; overflow-x: auto; margin: 12px 0; }
    p { margin: 10px 0; font-size: 14px; color: #bdbdbd; }
    a { color: #90caf9; }
    .ok { color: #81c784; }
  </style>
</head>
<body>
  <h1>KISAutoTrade 웹 서버</h1>
  <div class="subtitle">API 서버가 실행 중입니다.</div>

  <div class="warn">
    ⚠️ <strong>프론트엔드 빌드 파일을 찾을 수 없습니다.</strong><br>
    <code>dist/index.html</code> 이 서버에서 탐색되지 않았습니다.
  </div>

  <p><strong>해결 방법 1</strong> — 프론트엔드를 빌드하세요:</p>
  <pre>cd KISAutoTrade
npm run build
# 이후 앱(또는 서버)을 재시작</pre>

  <p><strong>해결 방법 2</strong> — <code>.env</code> 파일에 dist/ 경로를 직접 지정하세요:</p>
  <pre># .env
DIST_PATH=/절대경로/KISAutoTrade/dist</pre>

  <p>API는 정상 작동 중: <span class="ok">✓</span>
     <a href="/api/info">/api/info</a> ·
     <a href="/api/trading/status">/api/trading/status</a>
  </p>
</body>
</html>"#;
