/// 모바일/외부 접속용 경량 웹 서버 (axum)
///
/// 엔드포인트:
///   GET  /                                    → React 앱(dist/) 또는 모바일 대시보드 HTML
///   GET  /api/info                            → 앱 정보 JSON
///   GET  /api/balance                         → 잔고 JSON
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
///   POST /api/detect-trading-type             → 입력 키로 실전/모의 자동 감지
///   GET  /api/stock-list-stats                → 종목 목록 통계
///   POST /api/stock-update-interval           → 종목 목록 갱신 주기 변경
///   POST /api/refresh-stock-list              → 종목 목록 즉시 갱신
///   POST /api/test-discord                    → Discord 테스트 알림 전송
///   GET  /api/today-trades                    → 당일 체결 기록 (로컬 JSON)
///   GET  /api/check-update                    → 업데이트 확인 (웹 모드: 항상 최신)
///   POST /api/web-config/save                 → 웹 포트 저장 (.env WEB_PORT)
///   GET  /api/exchange-rate                   → USD/KRW 환율
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
    extract::{Path, Query, State},
    http::{header, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};
use tower_http::cors::CorsLayer;

use crate::api::rest::{KisRestClient, OverseasOrderRequest, OrderRequest, OrderSide, OrderType, StockSearchItem};
use crate::commands::TradeArchiveConfig;
use crate::config::{AccountProfile, AppConfig, ProfilesConfig};
use crate::logging::LogConfig;
use crate::market;
use crate::notifications::discord::DiscordNotifier;
use crate::storage::{
    stats_store::StatsStore, stock_store::StockStore, strategy_store::StrategyStore,
    trade_store::TradeStore,
};
use crate::trading::{
    order::OrderManager,
    position::PositionTracker,
    risk::RiskManager,
    strategy::StrategyManager,
};

#[derive(Clone)]
struct ServerState {
    rest_client:           Arc<RwLock<Arc<KisRestClient>>>,
    stock_list:            Arc<RwLock<Vec<StockSearchItem>>>,
    web_port:              u16,
    dist_path:             PathBuf,
    /// 자동매매 활성 여부 (commands.rs AppState 와 Arc 공유)
    is_trading:            Arc<Mutex<bool>>,
    /// 전략 관리자
    strategy_manager:      Arc<Mutex<StrategyManager>>,
    /// 포지션 트래커
    position_tracker:      Arc<Mutex<PositionTracker>>,
    /// 활성 프로파일 설정 (AppConfig)
    config:                Arc<RwLock<Arc<AppConfig>>>,
    /// 계좌 프로파일 목록
    profiles:              Arc<RwLock<ProfilesConfig>>,
    /// 체결 기록 저장소
    trade_store:           Arc<TradeStore>,
    /// 일별 통계 저장소
    stats_store:           Arc<StatsStore>,
    /// 로그 설정 (AppState 와 Arc 공유)
    log_config:            Arc<RwLock<LogConfig>>,
    /// 로그 디렉토리 경로
    log_dir:               PathBuf,
    /// 체결 기록 보관 설정
    trade_archive_config:  Arc<RwLock<TradeArchiveConfig>>,
    /// 데이터 디렉토리 경로
    data_dir:              PathBuf,
    /// 리스크 관리자
    risk_manager:          Arc<Mutex<RiskManager>>,
    /// 주문 관리자 (미체결 주문 조회용)
    order_manager:         Arc<Mutex<OrderManager>>,
    /// 영구 종목목록 캐시 (전략 업데이트 시 종목명 조회용)
    stock_store:           Arc<StockStore>,
    /// 전략 설정 저장소
    strategy_store:        Arc<StrategyStore>,
    /// 프로파일 저장 경로 (profiles.json)
    profiles_path:         PathBuf,
    /// Discord 알림 (테스트 발송용)
    discord:               Option<Arc<DiscordNotifier>>,
    /// USD/KRW 환율 캐시 (AppState와 Arc 공유)
    exchange_rate_krw:     Arc<RwLock<f64>>,
    /// 공통 데이터 갱신 주기(초)
    refresh_interval_sec:  u64,
}

/// 서버 시작 (포트 바인드 실패 시 경고만 내고 종료)
#[allow(clippy::too_many_arguments)]
pub async fn start(
    rest_client:          Arc<RwLock<Arc<KisRestClient>>>,
    stock_list:           Arc<RwLock<Vec<StockSearchItem>>>,
    port:                 u16,
    is_trading:           Arc<Mutex<bool>>,
    strategy_manager:     Arc<Mutex<StrategyManager>>,
    position_tracker:     Arc<Mutex<PositionTracker>>,
    config:               Arc<RwLock<Arc<AppConfig>>>,
    profiles:             Arc<RwLock<ProfilesConfig>>,
    trade_store:          Arc<TradeStore>,
    stats_store:          Arc<StatsStore>,
    log_config:           Arc<RwLock<LogConfig>>,
    log_dir:              PathBuf,
    trade_archive_config: Arc<RwLock<TradeArchiveConfig>>,
    data_dir:             PathBuf,
    risk_manager:         Arc<Mutex<RiskManager>>,
    order_manager:        Arc<Mutex<OrderManager>>,
    stock_store:          Arc<StockStore>,
    strategy_store:       Arc<StrategyStore>,
    profiles_path:        PathBuf,
    discord:              Option<Arc<DiscordNotifier>>,
    exchange_rate_krw:    Arc<RwLock<f64>>,
    refresh_interval_sec: u64,
) {
    let dist_path = web_dist_path();

    if dist_path.join("index.html").exists() {
        tracing::info!("웹 모드: React 앱을 {:?} 에서 서비스합니다", dist_path);
    } else {
        tracing::info!("웹 모드: dist/ 없음 — 모바일 대시보드 HTML로 서비스합니다");
        tracing::info!(
            "React 앱을 서비스하려면 프로젝트 루트에서 'npm run build' 를 실행하세요"
        );
    }

    let state = ServerState {
        rest_client, stock_list, web_port: port, dist_path,
        is_trading, strategy_manager, position_tracker,
        config, profiles, trade_store, stats_store,
        log_config, log_dir, trade_archive_config, data_dir,
        risk_manager, order_manager, stock_store, strategy_store,
        profiles_path, discord, exchange_rate_krw, refresh_interval_sec,
    };

    let app = Router::new()
        .route("/api/info",                        get(info_handler))
        .route("/api/app-config",                  get(app_config_handler))
        .route("/api/profiles",                    get(profiles_handler))
        .route("/api/balance",                     get(balance_handler))
        .route("/api/overseas-balance",             get(overseas_balance_handler))
        .route("/api/positions",                   get(positions_handler))
        .route("/api/price/:symbol",               get(price_handler))
        .route("/api/overseas-price/:ex/:symbol",  get(overseas_price_handler))
        .route("/api/executed",                    get(executed_handler))
        .route("/api/kis-executed",                get(kis_executed_handler))
        .route("/api/pending-orders",              get(pending_orders_handler))
        .route("/api/search/:query",               get(search_handler))
        .route("/api/order",                       post(order_handler))
        .route("/api/overseas-order",              post(overseas_order_handler))
        .route("/api/chart/:symbol",               get(chart_handler))
        .route("/api/overseas-chart/:ex/:symbol",  get(overseas_chart_handler))
        .route("/api/today-stats",                 get(today_stats_handler))
        .route("/api/stats",                       get(stats_by_range_handler))
        .route("/api/trades",                      get(trades_by_range_handler))
        .route("/api/log-config",                  get(log_config_handler)
                                                     .post(set_log_config_handler))
        .route("/api/recent-logs",                 get(recent_logs_handler))
        .route("/api/archive-config",              get(archive_config_handler)
                                                     .post(set_archive_config_handler))
        .route("/api/archive-stats",               get(archive_stats_handler))
        .route("/api/risk-config",                 get(risk_config_handler)
                                                     .post(update_risk_config_handler))
        .route("/api/risk-config/clear-emergency", post(clear_emergency_handler))
        .route("/api/web-config",                  get(web_config_handler))
        // ── 자동매매 제어 ──
        .route("/api/trading/status",              get(trading_status_handler))
        .route("/api/trading/start",               post(trading_start_handler))
        .route("/api/trading/stop",                post(trading_stop_handler))
        .route("/api/strategies",                  get(strategies_handler))
        .route("/api/strategies/:id",              post(update_strategy_handler))
        // ── 프로파일 관리 ──
        .route("/api/profiles/add",                post(add_profile_handler))
        .route("/api/profiles/update",             post(update_profile_handler))
        .route("/api/profiles/delete",             post(delete_profile_handler))
        .route("/api/profiles/:id/set-active",     post(set_active_profile_handler))
        .route("/api/profiles/:id/detect",         post(detect_profile_handler))
        // ── 설정 진단 / 감지 ──
        .route("/api/check-config",                get(check_config_handler))
        .route("/api/detect-trading-type",         post(detect_trading_type_handler))
        // ── 종목 목록 ──
        .route("/api/stock-list-stats",            get(stock_list_stats_handler))
        .route("/api/stock-update-interval",       post(set_stock_update_interval_handler))
        .route("/api/refresh-stock-list",          post(refresh_stock_list_handler))
        // ── Discord 테스트 ──
        .route("/api/test-discord",                post(test_discord_handler))
        // ── 당일 체결 기록 ──
        .route("/api/today-trades",                get(today_trades_handler))
        // ── 업데이트 확인 ──
        .route("/api/check-update",                get(check_update_handler))
        // ── 웹 설정 저장 ──
        .route("/api/web-config/save",             post(save_web_config_handler))
        // ── 환율 / 갱신 주기 ──
        .route("/api/exchange-rate",               get(exchange_rate_handler))
        .route("/api/refresh-interval",            get(refresh_interval_handler))
        // ── 매수 정지 / 비상 정지 ──
        .route("/api/buy-suspension/clear",        post(clear_buy_suspension_handler))
        .route("/api/activate-emergency",          post(activate_emergency_handler))
        // ── 체결 기록 / 통계 저장 ──
        .route("/api/save-trade",                  post(save_trade_handler))
        .route("/api/upsert-stats",                post(upsert_stats_handler))
        // ── 프론트엔드 로그 ──
        .route("/api/frontend-log",                post(frontend_log_handler))
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

/// dist/ 폴더 경로 탐색: 바이너리 옆 dist/ → 현재 작업 디렉토리의 dist/
fn web_dist_path() -> PathBuf {
    if let Some(p) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("dist")))
    {
        if p.exists() {
            return p;
        }
    }
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

    // SPA 라우팅 fallback: index.html 반환 또는 모바일 대시보드 HTML
    let index_path = s.dist_path.join("index.html");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(content) => Html(content).into_response(),
        Err(_) => Html(MOBILE_HTML).into_response(),
    }
}

/// 파일 확장자로 MIME 타입 추론
fn guess_mime(path: &str) -> &'static str {
    if path.ends_with(".js") || path.ends_with(".mjs") { "application/javascript; charset=utf-8" }
    else if path.ends_with(".css")   { "text/css; charset=utf-8" }
    else if path.ends_with(".html")  { "text/html; charset=utf-8" }
    else if path.ends_with(".json")  { "application/json; charset=utf-8" }
    else if path.ends_with(".ico")   { "image/x-icon" }
    else if path.ends_with(".png")   { "image/png" }
    else if path.ends_with(".svg")   { "image/svg+xml" }
    else if path.ends_with(".woff")  { "font/woff" }
    else if path.ends_with(".woff2") { "font/woff2" }
    else if path.ends_with(".webp")  { "image/webp" }
    else { "application/octet-stream" }
}

async fn info_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "app": "KISAutoTrade",
        "version": env!("CARGO_PKG_VERSION"),
        "port": s.web_port,
        "mode": "web",
    }))
}

async fn balance_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_balance().await {
        Ok(b) => Json(serde_json::to_value(b).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn overseas_balance_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_overseas_balance().await {
        Ok(b) => Json(serde_json::to_value(b).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn price_handler(
    State(s): State<ServerState>,
    Path(symbol): Path<String>,
) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_price(&symbol).await {
        Ok(p) => Json(serde_json::to_value(p).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn overseas_price_handler(
    State(s): State<ServerState>,
    Path((exchange, symbol)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_overseas_price(&symbol, &exchange).await {
        Ok(p) => {
            let mut val = serde_json::to_value(p).unwrap_or_default();
            // exchange, symbol 필드 보완
            if let serde_json::Value::Object(ref mut m) = val {
                m.insert("exchange".into(), serde_json::Value::String(exchange));
                m.insert("symbol".into(),   serde_json::Value::String(symbol));
            }
            Json(val)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn executed_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_today_executed_orders().await {
        Ok(e) => Json(serde_json::to_value(e).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/search/:query — KRX 로컬 캐시에서 이름/코드 검색
async fn search_handler(
    State(s): State<ServerState>,
    Path(query): Path<String>,
) -> Json<serde_json::Value> {
    let list = s.stock_list.read().await;
    let results = market::search_local(&list, &query, 30);
    Json(serde_json::to_value(results).unwrap_or_default())
}

/// POST /api/order 바디
#[derive(Deserialize)]
struct OrderBody {
    symbol:     String,
    side:       String,       // "Buy" | "Sell"
    order_type: String,       // "Limit" | "Market"
    quantity:   u64,
    price:      u64,
}

/// POST /api/order — 국내 주식 주문
async fn order_handler(
    State(s): State<ServerState>,
    Json(body): Json<OrderBody>,
) -> Json<serde_json::Value> {
    let side = if body.side == "Buy" { OrderSide::Buy } else { OrderSide::Sell };
    let order_type = if body.order_type == "Limit" { OrderType::Limit } else { OrderType::Market };

    let req = OrderRequest { symbol: body.symbol, side, order_type, quantity: body.quantity, price: body.price };
    let client = s.rest_client.read().await.clone();
    match client.place_order(&req).await {
        Ok(r)  => Json(serde_json::to_value(r).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// POST /api/overseas-order 바디
#[derive(Deserialize)]
struct OverseasOrderBody {
    symbol:   String,
    exchange: String, // NASD | NYSE | AMEX
    side:     String, // "Buy" | "Sell"
    quantity: u64,
    price:    f64,
}

/// POST /api/overseas-order — 해외 주식 주문
async fn overseas_order_handler(
    State(s): State<ServerState>,
    Json(body): Json<OverseasOrderBody>,
) -> Json<serde_json::Value> {
    use crate::api::rest::OrderSide;
    let side = if body.side == "Buy" { OrderSide::Buy } else { OrderSide::Sell };
    let req = OverseasOrderRequest {
        symbol: body.symbol, exchange: body.exchange, side,
        quantity: body.quantity, price: body.price,
    };
    let client = s.rest_client.read().await.clone();
    match client.place_overseas_order(&req).await {
        Ok(r)  => Json(serde_json::to_value(r).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/chart/:symbol?period=D&count=100
#[derive(Deserialize)]
struct ChartQuery {
    period: Option<String>, // D / W / M
    count:  Option<i64>,
}

async fn chart_handler(
    State(s): State<ServerState>,
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> Json<serde_json::Value> {
    let period = params.period.as_deref().unwrap_or("D");
    let count  = params.count.unwrap_or(100).max(1).min(500);

    // count → 시작일 계산 (넉넉하게 캘린더 일수로 변환)
    let factor: i64 = match period {
        "W" => 7,
        "M" => 31,
        _   => 2, // D: 거래일 기준이므로 2배 여유
    };
    let today     = chrono::Local::now().date_naive();
    let start_day = today - chrono::Duration::days(count * factor + 10);
    let end_date   = today.format("%Y%m%d").to_string();
    let start_date = start_day.format("%Y%m%d").to_string();

    let client = s.rest_client.read().await.clone();
    match client.get_chart_data(&symbol, period, &start_date, &end_date).await {
        Ok(candles) => Json(serde_json::to_value(candles).unwrap_or_default()),
        Err(e)      => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/overseas-chart/:ex/:symbol?period=D&count=100
async fn overseas_chart_handler(
    State(s): State<ServerState>,
    Path((exchange, symbol)): Path<(String, String)>,
    Query(params): Query<ChartQuery>,
) -> Json<serde_json::Value> {
    let period = params.period.as_deref().unwrap_or("D");
    let count  = params.count.unwrap_or(100).max(1).min(500);

    // count → 기준일 계산 (당일로부터 과거 N일 기준)
    let factor: i64 = match period {
        "W" => 7,
        "M" => 31,
        _   => 2, // D: 거래일은 캘린더의 약 절반
    };
    let base_day = chrono::Local::now().date_naive()
        - chrono::Duration::days(count * factor);
    let base_date = base_day.format("%Y%m%d").to_string();

    let client = s.rest_client.read().await.clone();
    match client.get_overseas_chart_data(&symbol, &exchange, period, &base_date).await {
        Ok(candles) => Json(serde_json::to_value(candles).unwrap_or_default()),
        Err(e)      => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ── 자동매매 제어 핸들러 ────────────────────────────────────────────

/// GET /api/trading/status
async fn trading_status_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let is_running = *s.is_trading.lock().await;
    let active_strategies: Vec<String> = s.strategy_manager.lock().await.active_names();
    let (position_count, total_unrealized_pnl) = {
        let tracker = s.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let (buy_suspended, buy_suspended_reason) = {
        let om = s.order_manager.lock().await;
        (om.buy_suspended, om.buy_suspended_reason.clone())
    };
    Json(serde_json::json!({
        "isRunning":           is_running,
        "activeStrategies":    active_strategies,
        "positionCount":       position_count,
        "totalUnrealizedPnl":  total_unrealized_pnl,
        "wsConnected":         false,
        "tradingProfileId":    null,
        "buySuspended":        buy_suspended,
        "buySuspendedReason":  buy_suspended_reason,
    }))
}

/// POST /api/trading/start — is_trading = true (폴링 데몬이 자동으로 재개)
async fn trading_start_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mut flag = s.is_trading.lock().await;
    if *flag {
        return Json(serde_json::json!({ "ok": false, "message": "이미 실행 중입니다." }));
    }
    *flag = true;
    drop(flag);
    tracing::info!("자동매매 시작 (웹 API 요청)");
    Json(serde_json::json!({ "ok": true, "message": "자동매매 시작됨" }))
}

/// POST /api/trading/stop — is_trading = false (폴링 데몬 자동 일시 정지)
async fn trading_stop_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    *s.is_trading.lock().await = false;
    tracing::info!("자동매매 정지 (웹 API 요청)");
    Json(serde_json::json!({ "ok": true, "message": "자동매매 정지됨" }))
}

/// GET /api/strategies — 전략 목록 (이름, 활성 여부, 대상 종목)
async fn strategies_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mgr = s.strategy_manager.lock().await;
    let configs = mgr.all_configs();
    Json(serde_json::to_value(configs).unwrap_or_default())
}

// ── 전략 업데이트 ─────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateStrategyBody {
    enabled:         Option<bool>,
    target_symbols:  Option<Vec<String>>,
    order_quantity:  Option<u64>,
    params:          Option<serde_json::Value>,
}

/// POST /api/strategies/:id — 전략 파라미터 업데이트
async fn update_strategy_handler(
    State(s): State<ServerState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateStrategyBody>,
) -> Json<serde_json::Value> {
    let target_symbols_snapshot = {
        let mut mgr = s.strategy_manager.lock().await;
        let cfg = match mgr.get_config_mut(&id) {
            Some(c) => c,
            None => return Json(serde_json::json!({ "error": format!("전략을 찾을 수 없습니다: {}", id) })),
        };
        if let Some(en) = body.enabled         { cfg.enabled = en; }
        if let Some(sym) = body.target_symbols { cfg.target_symbols = sym; }
        if let Some(qty) = body.order_quantity  { cfg.order_quantity = qty; }
        if let Some(p) = body.params            { cfg.params = p; }
        cfg.target_symbols.clone()
    };

    let mut symbol_names = std::collections::HashMap::new();
    for code in &target_symbols_snapshot {
        let name = s.stock_store.get_name(code).await.unwrap_or_else(|| code.clone());
        symbol_names.insert(code.clone(), name);
    }

    // 디스크에 영구 저장
    let profile_id = s.profiles.read().await.active_id.clone();
    if let Some(pid) = &profile_id {
        let all_configs: Vec<crate::trading::strategy::StrategyConfig> = {
            let mgr = s.strategy_manager.lock().await;
            mgr.all_configs().into_iter().cloned().collect()
        };
        if let Err(e) = s.strategy_store.save(pid, &all_configs).await {
            tracing::warn!("전략 설정 저장 실패: {}", e);
        }
    }

    let mgr = s.strategy_manager.lock().await;
    match mgr.all_configs().into_iter().find(|c| c.id == id) {
        Some(cfg) => Json(serde_json::json!({
            "id":              cfg.id,
            "name":            cfg.name,
            "enabled":         cfg.enabled,
            "targetSymbols":   cfg.target_symbols,
            "targetSymbolNames": symbol_names,
            "orderQuantity":   cfg.order_quantity,
            "params":          cfg.params,
        })),
        None => Json(serde_json::json!({ "error": "전략을 찾을 수 없습니다" })),
    }
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
    let (active_id, active_name) = {
        let profiles = s.profiles.read().await;
        match profiles.get_active() {
            Some(p) => (Some(p.id.clone()), Some(p.name.clone())),
            None    => (None, None),
        }
    };
    Json(serde_json::json!({
        "kisAppKeyMasked":   masked_key,
        "kisAccountNo":      cfg.kis_account_no,
        "kisIsPaperTrading": cfg.kis_is_paper_trading,
        "kisConfigured":     cfg.is_kis_configured(),
        "discordEnabled":    cfg.discord_bot_token.is_some(),
        "activeProfileId":   active_id,
        "activeProfileName": active_name,
    }))
}

/// GET /api/profiles
async fn profiles_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let profiles = s.profiles.read().await;
    let views: Vec<serde_json::Value> = profiles.profiles.iter().map(|p| {
        let masked = if p.app_key.len() > 6 {
            format!("{}****", &p.app_key[..6])
        } else {
            "****".into()
        };
        serde_json::json!({
            "id":             p.id,
            "name":           p.name,
            "isPaperTrading": p.is_paper_trading,
            "appKeyMasked":   masked,
            "accountNo":      p.account_no,
            "isActive":       profiles.active_id.as_deref() == Some(&p.id),
            "isConfigured":   p.is_configured(),
        })
    }).collect();
    Json(serde_json::to_value(views).unwrap_or_default())
}

// ── 포지션 / 통계 / 체결 ──────────────────────────────────────────

/// GET /api/positions
async fn positions_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let tracker = s.position_tracker.lock().await;
    let mut positions: Vec<serde_json::Value> = tracker.all().iter().map(|p| {
        serde_json::json!({
            "symbol":             p.symbol,
            "symbolName":         p.symbol_name,
            "quantity":           p.quantity,
            "avgPrice":           p.avg_price,
            "currentPrice":       p.current_price,
            "unrealizedPnl":      p.unrealized_pnl(),
            "unrealizedPnlRate":  p.unrealized_pnl_rate(),
        })
    }).collect();
    positions.sort_by(|a, b| {
        let qa = a.get("quantity").and_then(|v| v.as_u64()).unwrap_or(0);
        let qb = b.get("quantity").and_then(|v| v.as_u64()).unwrap_or(0);
        qb.cmp(&qa)
    });
    Json(serde_json::to_value(positions).unwrap_or_default())
}

/// GET /api/today-stats
async fn today_stats_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let today = chrono::Local::now().date_naive();
    match s.stats_store.get_by_date(today).await {
        Ok(stats) => Json(serde_json::to_value(stats).unwrap_or_default()),
        Err(e)    => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct DateRangeQuery {
    from: Option<String>,
    to:   Option<String>,
}

/// GET /api/stats?from=YYYY-MM-DD&to=YYYY-MM-DD
async fn stats_by_range_handler(
    State(s): State<ServerState>,
    Query(params): Query<DateRangeQuery>,
) -> Json<serde_json::Value> {
    use chrono::NaiveDate;
    let from_str = params.from.as_deref().unwrap_or("2020-01-01");
    let to_str   = params.to.as_deref().unwrap_or_else(|| {
        // 최대 범위 — 호출 시점 날짜 (아래에서 직접 계산)
        ""
    });
    let today = chrono::Local::now().date_naive().to_string();
    let to_str = if to_str.is_empty() { today.as_str() } else { to_str };

    let from = match NaiveDate::parse_from_str(from_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(serde_json::json!({ "error": format!("from 날짜 오류: {}", e) })),
    };
    let to = match NaiveDate::parse_from_str(to_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(serde_json::json!({ "error": format!("to 날짜 오류: {}", e) })),
    };
    match s.stats_store.get_by_range(from, to).await {
        Ok(stats) => Json(serde_json::to_value(stats).unwrap_or_default()),
        Err(e)    => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/trades?from=YYYY-MM-DD&to=YYYY-MM-DD
async fn trades_by_range_handler(
    State(s): State<ServerState>,
    Query(params): Query<DateRangeQuery>,
) -> Json<serde_json::Value> {
    use chrono::NaiveDate;
    let from_str = params.from.as_deref().unwrap_or("2020-01-01");
    let today = chrono::Local::now().date_naive().to_string();
    let to_str = params.to.as_deref().unwrap_or(today.as_str());

    let from = match NaiveDate::parse_from_str(from_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(serde_json::json!({ "error": format!("from 날짜 오류: {}", e) })),
    };
    let to = match NaiveDate::parse_from_str(to_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => return Json(serde_json::json!({ "error": format!("to 날짜 오류: {}", e) })),
    };
    match s.trade_store.get_by_range(from, to).await {
        Ok(trades) => Json(serde_json::to_value(trades).unwrap_or_default()),
        Err(e)     => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/kis-executed?from=YYYY-MM-DD&to=YYYY-MM-DD
async fn kis_executed_handler(
    State(s): State<ServerState>,
    Query(params): Query<DateRangeQuery>,
) -> Json<serde_json::Value> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let from_fmt = params.from.as_deref()
        .map(|d| d.replace('-', ""))
        .unwrap_or_else(|| today.clone());
    let to_fmt = params.to.as_deref()
        .map(|d| d.replace('-', ""))
        .unwrap_or(today);
    let client = s.rest_client.read().await.clone();
    match client.get_executed_orders_range(&from_fmt, &to_fmt).await {
        Ok(orders) => Json(serde_json::to_value(orders).unwrap_or_default()),
        Err(e)     => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/pending-orders
async fn pending_orders_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mgr = s.order_manager.lock().await;
    let views: Vec<serde_json::Value> = mgr.pending_orders().iter().map(|p| {
        serde_json::json!({
            "odno":         p.record.kis_order_id.clone().unwrap_or_default(),
            "symbol":       p.record.symbol,
            "symbolName":   p.record.symbol_name,
            "side":         match &p.record.side {
                crate::storage::order_store::OrderSide::Buy  => "buy",
                crate::storage::order_store::OrderSide::Sell => "sell",
            },
            "quantity":     p.record.quantity,
            "timestamp":    p.record.timestamp,
            "signalReason": p.signal_reason,
        })
    }).collect();
    Json(serde_json::to_value(views).unwrap_or_default())
}

// ── 로그 설정 ─────────────────────────────────────────────────────

/// GET /api/log-config
async fn log_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let cfg = s.log_config.read().await.clone();
    Json(serde_json::to_value(cfg).unwrap_or_default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetLogConfigBody {
    retention_days: Option<u32>,
    max_size_mb:    Option<u64>,
    api_debug:      Option<bool>,
}

/// POST /api/log-config
async fn set_log_config_handler(
    State(s): State<ServerState>,
    Json(body): Json<SetLogConfigBody>,
) -> Json<serde_json::Value> {
    let current = s.log_config.read().await.clone();
    let new_cfg = LogConfig {
        retention_days: body.retention_days.unwrap_or(current.retention_days).clamp(1, 365),
        max_size_mb:    body.max_size_mb.unwrap_or(current.max_size_mb).clamp(10, 10240),
        api_debug:      body.api_debug.unwrap_or(current.api_debug),
    };
    *s.log_config.write().await = new_cfg.clone();
    s.rest_client.read().await.set_api_debug(new_cfg.api_debug);
    new_cfg.save_sync(&s.log_dir).ok();
    crate::logging::cleanup(&s.log_dir, &new_cfg);
    Json(serde_json::to_value(new_cfg).unwrap_or_default())
}

/// GET /api/recent-logs?count=100
async fn recent_logs_handler(
    State(s): State<ServerState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let count = params.get("count")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100);
    let entries = crate::logging::read_recent_entries(&s.log_dir, count);
    Json(serde_json::to_value(entries).unwrap_or_default())
}

// ── 체결 기록 보관 설정 ───────────────────────────────────────────

/// GET /api/archive-config
async fn archive_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let cfg = s.trade_archive_config.read().await.clone();
    Json(serde_json::to_value(cfg).unwrap_or_default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetArchiveConfigBody {
    retention_days: u32,
    max_size_mb:    u64,
}

/// POST /api/archive-config
async fn set_archive_config_handler(
    State(s): State<ServerState>,
    Json(body): Json<SetArchiveConfigBody>,
) -> Json<serde_json::Value> {
    let new_cfg = TradeArchiveConfig {
        retention_days: body.retention_days.clamp(1, 3650),
        max_size_mb:    body.max_size_mb.clamp(50, 102400),
    };
    *s.trade_archive_config.write().await = new_cfg.clone();
    new_cfg.save_sync(&s.data_dir).ok();
    tracing::info!("체결 기록 보관 설정 변경 (웹 API): 보관 {}일, 최대 {}MB",
        new_cfg.retention_days, new_cfg.max_size_mb);
    Json(serde_json::to_value(new_cfg).unwrap_or_default())
}

/// GET /api/archive-stats
async fn archive_stats_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let data_dir = s.data_dir.clone();
    let result = tokio::task::spawn_blocking(move || {
        let trades_dir = data_dir.join("trades");
        let mut total_files: u64 = 0;
        let mut size_bytes: u64 = 0;
        let mut dates: Vec<String> = Vec::new();
        if trades_dir.exists() {
            if let Ok(years) = std::fs::read_dir(&trades_dir) {
                for year in years.flatten() {
                    if let Ok(months) = std::fs::read_dir(year.path()) {
                        for month in months.flatten() {
                            if let Ok(days) = std::fs::read_dir(month.path()) {
                                for day in days.flatten() {
                                    dates.push(format!("{}-{}-{}",
                                        year.file_name().to_string_lossy(),
                                        month.file_name().to_string_lossy(),
                                        day.file_name().to_string_lossy()));
                                    if let Ok(files) = std::fs::read_dir(day.path()) {
                                        for f in files.flatten() {
                                            if f.path().is_file() {
                                                total_files += 1;
                                                size_bytes += f.path().metadata()
                                                    .map(|m| m.len()).unwrap_or(0);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        dates.sort();
        serde_json::json!({
            "totalFiles": total_files,
            "sizeBytes":  size_bytes,
            "oldestDate": dates.first(),
            "newestDate": dates.last(),
        })
    }).await;
    match result {
        Ok(val) => Json(val),
        Err(e)  => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ── 리스크 관리 ───────────────────────────────────────────────────

/// GET /api/risk-config
async fn risk_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mut risk = s.risk_manager.lock().await;
    risk.reset_if_new_day();
    Json(serde_json::json!({
        "enabled":          risk.is_enabled(),
        "dailyLossLimit":   risk.daily_loss_limit,
        "maxPositionRatio": risk.max_position_ratio,
        "currentLoss":      risk.current_loss(),
        "dailyProfit":      risk.daily_profit(),
        "netLoss":          risk.net_loss(),
        "lossRatio":        risk.loss_ratio(),
        "isEmergencyStop":  risk.is_emergency_stop(),
        "canTrade":         risk.can_trade(),
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateRiskConfigBody {
    enabled:            Option<bool>,
    daily_loss_limit:   Option<i64>,
    max_position_ratio: Option<f64>,
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
        if limit >= 0 { risk.daily_loss_limit = limit; }
    }
    if let Some(ratio) = body.max_position_ratio {
        if (0.0..=1.0).contains(&ratio) { risk.max_position_ratio = ratio; }
    }
    Json(serde_json::json!({
        "enabled":          risk.is_enabled(),
        "dailyLossLimit":   risk.daily_loss_limit,
        "maxPositionRatio": risk.max_position_ratio,
        "currentLoss":      risk.current_loss(),
        "dailyProfit":      risk.daily_profit(),
        "netLoss":          risk.net_loss(),
        "lossRatio":        risk.loss_ratio(),
        "isEmergencyStop":  risk.is_emergency_stop(),
        "canTrade":         risk.can_trade(),
    }))
}

/// POST /api/risk-config/clear-emergency
async fn clear_emergency_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let mut risk = s.risk_manager.lock().await;
    risk.clear_emergency_stop();
    Json(serde_json::json!({
        "enabled":          risk.is_enabled(),
        "dailyLossLimit":   risk.daily_loss_limit,
        "maxPositionRatio": risk.max_position_ratio,
        "currentLoss":      risk.current_loss(),
        "dailyProfit":      risk.daily_profit(),
        "netLoss":          risk.net_loss(),
        "lossRatio":        risk.loss_ratio(),
        "isEmergencyStop":  risk.is_emergency_stop(),
        "canTrade":         risk.can_trade(),
    }))
}

// ── 웹 설정 ───────────────────────────────────────────────────────

/// GET /api/web-config
async fn web_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "runningPort": s.web_port,
        "accessUrl":   format!("http://localhost:{}", s.web_port),
    }))
}

// ── 설정 진단 ─────────────────────────────────────────────────────

/// GET /api/check-config
async fn check_config_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let cfg = s.config.read().await.clone();
    let mut issues: Vec<String> = Vec::new();
    if cfg.kis_app_key.is_empty()    { issues.push("KIS APP KEY가 설정되지 않았습니다.".into()); }
    if cfg.kis_app_secret.is_empty() { issues.push("KIS APP SECRET이 설정되지 않았습니다.".into()); }
    if cfg.kis_account_no.is_empty() { issues.push("KIS 계좌번호가 설정되지 않았습니다.".into()); }
    let paper_available = {
        let p = s.profiles.read().await;
        p.profiles.iter().any(|p| p.is_paper_trading && p.is_configured())
    };
    Json(serde_json::json!({
        "realKeySet":        !cfg.kis_app_key.is_empty(),
        "realAccountSet":    !cfg.kis_account_no.is_empty(),
        "paperKeySet":       paper_available,
        "activeMode":        if cfg.kis_is_paper_trading { "모의투자" } else { "실전투자" },
        "isReady":           cfg.is_kis_configured(),
        "discordConfigured": cfg.discord_bot_token.is_some(),
        "baseUrl":           cfg.kis_base_url(),
        "issues":            issues,
    }))
}

// ── 프로파일 관리 helper ──────────────────────────────────────────

fn profile_json(p: &AccountProfile, active_id: &Option<String>) -> serde_json::Value {
    let masked = if p.app_key.len() > 6 {
        format!("{}****", &p.app_key[..6])
    } else {
        "****".into()
    };
    serde_json::json!({
        "id":             p.id,
        "name":           p.name,
        "isPaperTrading": p.is_paper_trading,
        "appKeyMasked":   masked,
        "accountNo":      p.account_no,
        "isActive":       active_id.as_deref() == Some(&p.id),
        "isConfigured":   p.is_configured(),
    })
}

/// 활성 프로파일 변경 시 config + rest_client 갱신 (웹 서버 내부용)
async fn apply_profile_change(s: &ServerState) {
    use crate::api::token::TokenManager;
    let new_config = {
        let profiles = s.profiles.read().await;
        let existing = s.config.read().await.clone();
        match profiles.get_active() {
            Some(p) => Arc::new(AppConfig {
                kis_app_key:          p.app_key.clone(),
                kis_app_secret:       p.app_secret.clone(),
                kis_account_no:       p.account_no.clone(),
                kis_is_paper_trading: p.is_paper_trading,
                discord_bot_token:    existing.discord_bot_token.clone(),
                discord_channel_id:   existing.discord_channel_id.clone(),
                notification_levels:  existing.notification_levels.clone(),
            }),
            None => Arc::new(AppConfig {
                kis_app_key:          String::new(),
                kis_app_secret:       String::new(),
                kis_account_no:       String::new(),
                kis_is_paper_trading: false,
                discord_bot_token:    existing.discord_bot_token.clone(),
                discord_channel_id:   existing.discord_channel_id.clone(),
                notification_levels:  existing.notification_levels.clone(),
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

// ── 프로파일 CRUD 핸들러 ──────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddProfileBody {
    name: String,
    is_paper_trading: bool,
    app_key: String,
    app_secret: String,
    account_no: String,
}

/// POST /api/profiles/add
async fn add_profile_handler(
    State(s): State<ServerState>,
    Json(body): Json<AddProfileBody>,
) -> Json<serde_json::Value> {
    let profile = AccountProfile::new(
        body.name, body.is_paper_trading,
        body.app_key, body.app_secret, body.account_no,
    );
    let (view, is_first) = {
        let mut profiles = s.profiles.write().await;
        let was_empty = profiles.profiles.is_empty();
        let added = profiles.add(profile);
        let view = profile_json(&added, &profiles.active_id);
        (view, was_empty)
    };
    if is_first {
        apply_profile_change(&s).await;
    }
    save_profiles_server(&s).await;
    Json(view)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProfileBody {
    id: String,
    name: Option<String>,
    is_paper_trading: Option<bool>,
    app_key: Option<String>,
    app_secret: Option<String>,
    account_no: Option<String>,
}

/// POST /api/profiles/update
async fn update_profile_handler(
    State(s): State<ServerState>,
    Json(body): Json<UpdateProfileBody>,
) -> Json<serde_json::Value> {
    let (view, is_active) = {
        let mut profiles = s.profiles.write().await;
        match profiles.update(
            &body.id, body.name, body.is_paper_trading,
            body.app_key, body.app_secret, body.account_no,
        ) {
            Some(p) => {
                let active = profiles.active_id.as_deref() == Some(&body.id);
                (profile_json(&p, &profiles.active_id), active)
            }
            None => return Json(serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", body.id) })),
        }
    };
    if is_active {
        apply_profile_change(&s).await;
    }
    save_profiles_server(&s).await;
    Json(view)
}

#[derive(Deserialize)]
struct DeleteProfileBody {
    id: String,
}

/// POST /api/profiles/delete
async fn delete_profile_handler(
    State(s): State<ServerState>,
    Json(body): Json<DeleteProfileBody>,
) -> Json<serde_json::Value> {
    let deleted = {
        let mut profiles = s.profiles.write().await;
        profiles.delete(&body.id)
    };
    if !deleted {
        return Json(serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", body.id) }));
    }
    apply_profile_change(&s).await;
    save_profiles_server(&s).await;
    Json(serde_json::json!({ "ok": true }))
}

/// POST /api/profiles/:id/set-active
async fn set_active_profile_handler(
    State(s): State<ServerState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let ok = {
        let mut profiles = s.profiles.write().await;
        profiles.set_active(&id)
    };
    if !ok {
        return Json(serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", id) }));
    }
    if !*s.is_trading.lock().await {
        apply_profile_change(&s).await;
    }
    save_profiles_server(&s).await;
    app_config_handler(State(s)).await
}

// ── 실전/모의 자동 감지 ───────────────────────────────────────────

#[derive(serde::Serialize)]
struct DetectTokenReq {
    grant_type: String,
    appkey: String,
    appsecret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetectTradingTypeBody {
    app_key: String,
    app_secret: String,
}

async fn try_detect_token(client: &reqwest::Client, url: &str, key: &str, secret: &str) -> bool {
    let Ok(resp) = client
        .post(url)
        .header("content-type", "application/json; charset=utf-8")
        .json(&DetectTokenReq {
            grant_type: "client_credentials".into(),
            appkey: key.to_string(),
            appsecret: secret.to_string(),
        })
        .send()
        .await
    else {
        return false;
    };
    if !resp.status().is_success() { return false; }
    let Ok(val) = resp.json::<serde_json::Value>().await else { return false; };
    val.get("access_token").and_then(|v| v.as_str()).map(|t| !t.is_empty()).unwrap_or(false)
}

/// POST /api/detect-trading-type
async fn detect_trading_type_handler(
    State(_s): State<ServerState>,
    Json(body): Json<DetectTradingTypeBody>,
) -> Json<serde_json::Value> {
    const REAL_URL: &str  = "https://openapi.koreainvestment.com:9443/oauth2/tokenP";
    const PAPER_URL: &str = "https://openapivts.koreainvestment.com:29443/oauth2/tokenP";
    if body.app_key.trim().is_empty() || body.app_secret.trim().is_empty() {
        return Json(serde_json::json!({ "error": "APP KEY와 APP SECRET을 모두 입력하세요." }));
    }
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build() {
        Ok(c) => c,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };
    if try_detect_token(&client, REAL_URL, &body.app_key, &body.app_secret).await {
        return Json(serde_json::json!({ "isPaperTrading": false, "message": "실전투자 키로 확인되었습니다." }));
    }
    if try_detect_token(&client, PAPER_URL, &body.app_key, &body.app_secret).await {
        return Json(serde_json::json!({ "isPaperTrading": true, "message": "모의투자 키로 확인되었습니다." }));
    }
    Json(serde_json::json!({ "error": "실전/모의 키를 자동 감지하지 못했습니다." }))
}

/// POST /api/profiles/:id/detect
async fn detect_profile_handler(
    State(s): State<ServerState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let (app_key, app_secret) = {
        let profiles = s.profiles.read().await;
        match profiles.profiles.iter().find(|p| p.id == id) {
            Some(p) if !p.app_key.is_empty() && !p.app_secret.is_empty() =>
                (p.app_key.clone(), p.app_secret.clone()),
            Some(_) =>
                return Json(serde_json::json!({ "error": "APP KEY 또는 APP SECRET이 설정되지 않았습니다." })),
            None =>
                return Json(serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", id) })),
        }
    };
    const REAL_URL: &str  = "https://openapi.koreainvestment.com:9443/oauth2/tokenP";
    const PAPER_URL: &str = "https://openapivts.koreainvestment.com:29443/oauth2/tokenP";
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build() {
        Ok(c) => c,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };
    let is_paper = if try_detect_token(&client, REAL_URL, &app_key, &app_secret).await {
        false
    } else if try_detect_token(&client, PAPER_URL, &app_key, &app_secret).await {
        true
    } else {
        return Json(serde_json::json!({ "error": "실전/모의 키를 자동 감지하지 못했습니다." }));
    };
    let view = {
        let mut profiles = s.profiles.write().await;
        match profiles.update(&id, None, Some(is_paper), None, None, None) {
            Some(p) => profile_json(&p, &profiles.active_id),
            None    => return Json(serde_json::json!({ "error": format!("프로파일을 찾을 수 없습니다: {}", id) })),
        }
    };
    let is_active = s.profiles.read().await.active_id.as_deref() == Some(&id);
    if is_active { apply_profile_change(&s).await; }
    save_profiles_server(&s).await;
    Json(view)
}

// ── 종목 목록 ─────────────────────────────────────────────────────

/// GET /api/stock-list-stats
async fn stock_list_stats_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let count                = s.stock_store.size().await;
    let last_updated_at      = s.stock_store.last_updated_at().await;
    let update_interval_hours = s.stock_store.get_interval_hours().await;
    Json(serde_json::json!({
        "count":               count,
        "lastUpdatedAt":       last_updated_at,
        "filePath":            "",
        "updateIntervalHours": update_interval_hours,
    }))
}

#[derive(Deserialize)]
struct StockIntervalBody { hours: u32 }

/// POST /api/stock-update-interval
async fn set_stock_update_interval_handler(
    State(s): State<ServerState>,
    Json(body): Json<StockIntervalBody>,
) -> Json<serde_json::Value> {
    match s.stock_store.set_interval_hours(body.hours).await {
        Ok(())  => Json(serde_json::json!({ "ok": true })),
        Err(e)  => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// POST /api/refresh-stock-list
async fn refresh_stock_list_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let items = match crate::market::StockList::fetch_from_krx().await {
        Ok(items) if !items.is_empty() => items,
        Ok(_)  => return Json(serde_json::json!({ "error": "KRX에서 종목 목록을 가져오지 못했습니다 (0개)." })),
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };
    let count = items.len();
    *s.stock_list.write().await = items.clone();
    let cache_path = s.data_dir.join("stock_list.json");
    if let Some(dir) = cache_path.parent() { let _ = tokio::fs::create_dir_all(dir).await; }
    if let Ok(json) = serde_json::to_string_pretty(&items) {
        let _ = tokio::fs::write(&cache_path, json).await;
    }
    s.stock_store.upsert_many(items.iter().map(|i| (i.pdno.clone(), i.prdt_name.clone()))).await;
    tracing::info!("종목 목록 웹 API 갱신 완료: {}개", count);
    Json(serde_json::json!(count))
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
                Ok(_)  => Json(serde_json::json!({ "ok": true, "message": "Discord 테스트 알림 전송 완료" })),
                Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
            }
        }
        None => Json(serde_json::json!({ "error": "Discord 봇이 설정되지 않았습니다." })),
    }
}

// ── 당일 체결 기록 ────────────────────────────────────────────────

/// GET /api/today-trades
async fn today_trades_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let today = chrono::Local::now().date_naive();
    match s.trade_store.get_by_date(today).await {
        Ok(trades) => Json(serde_json::to_value(trades).unwrap_or_default()),
        Err(e)     => Json(serde_json::json!({ "error": e.to_string() })),
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
struct SaveWebConfigBody { new_port: u16 }

/// POST /api/web-config/save — .env WEB_PORT 저장 (재시작 후 반영)
async fn save_web_config_handler(
    State(_s): State<ServerState>,
    Json(body): Json<SaveWebConfigBody>,
) -> Json<serde_json::Value> {
    use std::io::Write;
    let env_path = std::env::current_dir().unwrap_or_default().join(".env");
    let content = format!("WEB_PORT={}\n", body.new_port);
    match std::fs::OpenOptions::new()
        .create(true).write(true).truncate(true)
        .open(&env_path)
        .and_then(|mut f| f.write_all(content.as_bytes()))
    {
        Ok(_)  => Json(serde_json::json!({ "ok": true, "message": format!(".env 저장 완료: WEB_PORT={}", body.new_port) })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ── 환율 / 갱신 주기 ─────────────────────────────────────────────

/// GET /api/exchange-rate
async fn exchange_rate_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!(*s.exchange_rate_krw.read().await))
}

/// GET /api/refresh-interval
async fn refresh_interval_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!(s.refresh_interval_sec))
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
    Json(serde_json::json!({
        "enabled":          risk.is_enabled(),
        "dailyLossLimit":   risk.daily_loss_limit,
        "maxPositionRatio": risk.max_position_ratio,
        "currentLoss":      risk.current_loss(),
        "dailyProfit":      risk.daily_profit(),
        "netLoss":          risk.net_loss(),
        "lossRatio":        risk.loss_ratio(),
        "isEmergencyStop":  risk.is_emergency_stop(),
        "canTrade":         risk.can_trade(),
    }))
}

// ── 체결 기록 / 통계 저장 ─────────────────────────────────────────

/// POST /api/save-trade
async fn save_trade_handler(
    State(s): State<ServerState>,
    Json(record): Json<crate::storage::trade_store::TradeRecord>,
) -> Json<serde_json::Value> {
    let saved = serde_json::to_value(&record).unwrap_or_default();
    match s.trade_store.append(record).await {
        Ok(_)  => Json(saved),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// POST /api/upsert-stats
async fn upsert_stats_handler(
    State(s): State<ServerState>,
    Json(stats): Json<crate::storage::stats_store::DailyStats>,
) -> Json<serde_json::Value> {
    match s.stats_store.upsert(stats).await {
        Ok(_)  => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ── 프론트엔드 로그 ───────────────────────────────────────────────

#[derive(Deserialize)]
struct FrontendLogBody {
    level:   Option<String>,
    message: String,
    context: Option<String>,
}

/// POST /api/frontend-log
async fn frontend_log_handler(Json(body): Json<FrontendLogBody>) -> Json<serde_json::Value> {
    let ctx = body.context.as_deref().unwrap_or("ui");
    match body.level.as_deref().unwrap_or("INFO") {
        "ERROR" => tracing::error!("[Frontend:{}] {}", ctx, body.message),
        "WARN"  => tracing::warn!("[Frontend:{}] {}", ctx, body.message),
        _       => tracing::info!("[Frontend:{}] {}", ctx, body.message),
    }
    Json(serde_json::json!({ "ok": true }))
}

// ── 모바일 대시보드 HTML (dist/index.html 없을 때 폴백) ────────────

static MOBILE_HTML: &str = r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>KISAutoTrade</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, sans-serif; background: #121212; color: #e0e0e0; padding: 16px; }
    h1 { font-size: 18px; color: #90caf9; margin-bottom: 4px; }
    .subtitle { font-size: 12px; color: #757575; margin-bottom: 16px; }
    h2 { font-size: 14px; color: #90caf9; margin-bottom: 8px; }
    .card { background: #1e1e1e; border-radius: 8px; padding: 14px; margin-bottom: 12px; }
    .row { display: flex; justify-content: space-between; padding: 6px 0; border-bottom: 1px solid #2a2a2a; font-size: 13px; }
    .row:last-child { border-bottom: none; }
    .label { color: #9e9e9e; }
    .value { font-weight: 600; }
    .green { color: #4caf50; }
    .red { color: #ef5350; }
    table { width: 100%; border-collapse: collapse; font-size: 12px; }
    th { color: #9e9e9e; text-align: left; padding: 4px 0; border-bottom: 1px solid #333; }
    td { padding: 6px 0; border-bottom: 1px solid #2a2a2a; vertical-align: top; }
    .chip { display: inline-block; padding: 2px 8px; border-radius: 12px; font-size: 11px; font-weight: 600; }
    .chip-buy { background: #1565c0; color: #fff; }
    .chip-sell { background: #b71c1c; color: #fff; }
    .chip-running { background: #2e7d32; color: #fff; }
    .chip-stopped { background: #424242; color: #9e9e9e; }
    .sub { font-size: 11px; color: #757575; }
    .refresh { margin-top: 8px; text-align: right; font-size: 11px; color: #757575; }
    .btn { border: none; padding: 10px 18px; border-radius: 8px; font-size: 13px; cursor: pointer; font-weight: 600; }
    .btn-primary { background: #1565c0; color: #fff; }
    .btn-danger  { background: #b71c1c; color: #fff; }
    .btn-refresh { background: #37474f; color: #ccc; }
    .btn-row { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 8px; }
    .status-row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
  </style>
</head>
<body>
  <h1>KISAutoTrade</h1>
  <div class="subtitle">모바일 대시보드</div>

  <!-- 자동매매 제어 -->
  <div class="card">
    <h2>자동매매</h2>
    <div class="status-row">
      <span class="label">상태</span>
      <span id="trading-status-chip" class="chip chip-stopped">대기 중</span>
    </div>
    <div id="trading-strategies" class="sub" style="margin-bottom:8px;color:#757575"></div>
    <div class="btn-row">
      <button class="btn btn-primary" onclick="tradingStart()">▶ 시작</button>
      <button class="btn btn-danger"  onclick="tradingStop()">■ 정지</button>
    </div>
    <div id="trading-msg" class="sub" style="margin-top:6px;color:#ffb300"></div>
  </div>

  <!-- 잔고 -->
  <div class="card">
    <h2>잔고</h2>
    <div id="balance-body"><div class="label">로딩 중...</div></div>
  </div>

  <!-- 보유 종목 -->
  <div class="card">
    <h2>보유 종목</h2>
    <table>
      <thead><tr><th>종목</th><th style="text-align:right">수량</th><th style="text-align:right">손익률</th></tr></thead>
      <tbody id="holdings"></tbody>
    </table>
  </div>

  <!-- 당일 체결 -->
  <div class="card">
    <h2>당일 체결</h2>
    <table>
      <thead><tr><th>종목</th><th>구분</th><th style="text-align:right">체결가</th></tr></thead>
      <tbody id="executed"></tbody>
    </table>
  </div>

  <div class="refresh" id="refresh-time"></div>
  <br>
  <button class="btn btn-refresh" onclick="load()">↺ 새로고침</button>

  <script>
    function fmt(n) { return Number(n || 0).toLocaleString('ko-KR'); }

    async function tradingStart() {
      document.getElementById('trading-msg').textContent = '요청 중...';
      try {
        const r = await fetch('/api/trading/start', { method: 'POST' }).then(r => r.json());
        document.getElementById('trading-msg').textContent = r.message || '';
        await loadTradingStatus();
      } catch(e) { document.getElementById('trading-msg').textContent = '오류: ' + e.message; }
    }
    async function tradingStop() {
      document.getElementById('trading-msg').textContent = '요청 중...';
      try {
        const r = await fetch('/api/trading/stop', { method: 'POST' }).then(r => r.json());
        document.getElementById('trading-msg').textContent = r.message || '';
        await loadTradingStatus();
      } catch(e) { document.getElementById('trading-msg').textContent = '오류: ' + e.message; }
    }
    async function loadTradingStatus() {
      try {
        const t = await fetch('/api/trading/status').then(r => r.json());
        const chip = document.getElementById('trading-status-chip');
        if (t.isRunning) {
          chip.className = 'chip chip-running'; chip.textContent = '● 자동매매 중';
        } else {
          chip.className = 'chip chip-stopped'; chip.textContent = '대기 중';
        }
        document.getElementById('trading-strategies').textContent =
          t.activeStrategies && t.activeStrategies.length
            ? '전략: ' + t.activeStrategies.join(', ')
            : '활성 전략 없음';
      } catch(e) { console.error('trading status', e); }
    }

    async function load() {
      await loadTradingStatus();
      try {
        const bal = await fetch('/api/balance').then(r => r.json());
        const s = bal.summary || {};
        document.getElementById('balance-body').innerHTML =
          `<div class="row"><span class="label">예수금</span><span class="value">${fmt(s.dnca_tot_amt)}원</span></div>` +
          `<div class="row"><span class="label">총평가금액</span><span class="value">${fmt(s.tot_evlu_amt)}원</span></div>` +
          `<div class="row"><span class="label">순자산</span><span class="value">${fmt(s.nass_amt)}원</span></div>`;

        const tbody = document.getElementById('holdings');
        tbody.innerHTML = (bal.items || []).map(i => {
          const pf = parseFloat(i.evlu_pfls_rt || 0);
          const cls = pf >= 0 ? 'green' : 'red';
          const sign = pf >= 0 ? '+' : '';
          return `<tr>
            <td>${i.prdt_name}<div class="sub">${i.pdno}</div></td>
            <td style="text-align:right">${fmt(i.hldg_qty)}주</td>
            <td style="text-align:right" class="${cls}">${sign}${pf.toFixed(2)}%<div class="sub ${cls}">${sign}${fmt(i.evlu_pfls_amt)}원</div></td>
          </tr>`;
        }).join('') || '<tr><td colspan="3" class="label" style="text-align:center;padding:8px">보유 종목 없음</td></tr>';
      } catch(e) { console.error('balance', e); }

      try {
        const ex = await fetch('/api/executed').then(r => r.json());
        const etbody = document.getElementById('executed');
        etbody.innerHTML = (Array.isArray(ex) ? ex : []).map(o => {
          const isSell = o.sll_buy_dvsn_cd === '01';
          return `<tr>
            <td>${o.prdt_name}<div class="sub">${o.pdno}</div></td>
            <td><span class="chip ${isSell ? 'chip-sell' : 'chip-buy'}">${isSell ? '매도' : '매수'}</span></td>
            <td style="text-align:right">${fmt(o.ord_unpr)}원<div class="sub">${fmt(o.tot_ccld_qty)}주</div></td>
          </tr>`;
        }).join('') || '<tr><td colspan="3" class="label" style="text-align:center;padding:8px">체결 내역 없음</td></tr>';
      } catch(e) { console.error('executed', e); }

      document.getElementById('refresh-time').textContent =
        '최종 업데이트: ' + new Date().toLocaleTimeString('ko-KR');
    }

    load();
    setInterval(load, 30000);
  </script>
</body>
</html>"#;
