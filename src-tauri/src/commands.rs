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

mod accounts;
pub use accounts::*;
mod toss;
pub use toss::*;
mod toss_market;
pub use toss_market::*;
mod trading;
pub use trading::*;
mod strategy_preview;
pub use strategy_preview::*;
mod strategy;
pub use strategy::*;
mod risk;
pub use risk::*;
mod orders;
pub use orders::*;
mod market;
pub use market::*;
mod records;
pub use records::*;
mod settings;
pub use settings::*;
mod archive;
pub use archive::*;
mod database;
pub use database::*;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::{watch, Mutex, RwLock};

use crate::broker::toss::{
    TossCommission, TossExchangeRateResponse, TossKrMarketCalendarResponse, TossMarketSession,
    TossOrderbookEntry, TossOrderbookResponse, TossPriceLimitResponse, TossStockInfo,
    TossStockWarning, TossTrade, TossUsMarketCalendarResponse,
};
use crate::{
    api::{
        rest::{
            BalanceItem, BalanceSummary, ChartCandle, ExecutedOrder, KisRestClient, OrderRequest,
            OrderResponse, OverseasBalanceItem, OverseasBalanceSummary, OverseasExecutedOrder,
            PriceResponse, StockSearchItem,
        },
        token::TokenManager,
    },
    broker::{
        BrokerAccountId, BrokerAdapter, BrokerCandle, BrokerCurrency, BrokerHolding, BrokerId,
        BrokerMarket, BrokerMoney, BrokerOrderSide, BrokerQuantity, BrokerScope, BrokerSymbol,
        KisBrokerAdapter, TossBrokerAdapter,
    },
    config::{AccountProfile, AppConfig, DiscordConfig, ProfilesConfig},
    logging::LogConfig,
    market_hours::{
        is_domestic_symbol, is_market_open_for_with_calendar,
        is_market_open_for_with_calendar_policy, open_markets_summary_with_calendar,
        MarketCalendarOverride, MarketDayCalendar, MarketSessionWindow, UsMarketSessionCalendar,
        UsTradingSessionPolicy,
    },
    notifications::{discord::DiscordNotifier, types::NotificationEvent},
    storage::{
        database::DatabaseManager,
        stats_store::DailyStats,
        stock_store::{StockListStats, StockStore},
        strategy_store::StrategyStore,
        trade_store::TradeRecord,
        OrderStore, StatsStore, TradeStore,
    },
    trading::{
        order::OrderManager,
        position::{OverseasPositionTracker, Position, PositionTracker},
        preflight::{
            evaluate_order_preflight, format_money_amount, parse_decimal_amount,
            OrderPreflightConstraints, OrderPreflightInput,
        },
        risk::RiskManager,
        strategy::{
            BrokerPositionSnapshot, ConsecutiveMoveParams, ConsecutiveMoveStrategy,
            DeviationParams, DeviationStrategy, FailedBreakoutParams, FailedBreakoutStrategy,
            FiftyTwoWeekHighParams, FiftyTwoWeekHighStrategy, LeveragedTrendHoldParams,
            LeveragedTrendHoldStrategy, MaCrossParams, MeanReversionParams, MeanReversionStrategy,
            MomentumParams, MomentumStrategy, MovingAverageCrossStrategy, OhlcCandle,
            PriceConditionParams, PriceConditionStrategy, RsiParams, RsiStrategy, StrategyConfig,
            StrategyManager, StrongCloseParams, StrongCloseStrategy, TrendFilterParams,
            TrendFilterStrategy, VolatilityExpansionParams, VolatilityExpansionStrategy,
        },
        views::{build_strategy_view, StrategyView},
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
    /// 저장 backend 관리와 자동매매 시작을 직렬화해 전환 중 신규 거래를 막는다.
    pub storage_maintenance: Arc<Mutex<()>>,
    /// 전략 관리자
    pub strategy_manager: Arc<Mutex<StrategyManager>>,
    /// 전략 메모리 변경과 전체 설정 문서 저장을 IPC/REST 사이에서 직렬화한다.
    pub strategy_update_lock: Arc<Mutex<()>>,
    /// 포지션 트래커
    pub position_tracker: Arc<Mutex<PositionTracker>>,
    /// 해외 포지션 트래커 (USD cents, 거래소 코드 보존)
    pub overseas_position_tracker: Arc<Mutex<OverseasPositionTracker>>,
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
    /// 자동매매가 시작된 시점의 broker ID (프로파일 전환 중에도 유지)
    pub trading_broker_id: Arc<RwLock<Option<BrokerId>>>,
    /// 자동매매가 시작된 시점의 broker account ID (프로파일 전환 중에도 유지)
    pub trading_account_id: Arc<RwLock<Option<String>>>,
    /// USD/KRW 환율 캐시 (refresh_interval_sec마다 백그라운드 갱신, 기본 1450원)
    pub exchange_rate_krw: Arc<RwLock<f64>>,
    /// USD/KRW 환율 출처/유효시간 메타데이터
    pub exchange_rate_status: Arc<RwLock<ExchangeRateView>>,
    /// 데이터 갱신 주기 설정 (UI에서 변경 가능, JSON 영구 저장)
    pub refresh_config: Arc<RwLock<RefreshConfig>>,
    /// 갱신 주기 변경을 백그라운드 데몬에 전달하는 채널 (설정 저장 시 즉시 적용)
    pub refresh_interval_tx: Arc<watch::Sender<u64>>,
    /// JSON/PostgreSQL/MariaDB 저장 backend와 관리 작업을 공유한다.
    pub database_manager: Arc<DatabaseManager>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        config: Arc<AppConfig>,
        discord_config: Arc<DiscordConfig>,
        profiles: ProfilesConfig,
        profiles_path: PathBuf,
        data_dir: PathBuf,
        log_dir: PathBuf,
        log_config: LogConfig,
        web_port: u16,
        refresh_config: RefreshConfig,
        refresh_interval_tx: watch::Sender<u64>,
        database_manager: Arc<DatabaseManager>,
    ) -> Self {
        let rest_client = make_rest_client(&config);

        let discord = match (&discord_config.bot_token, &discord_config.channel_id) {
            (Some(token), Some(channel)) if !token.is_empty() && !channel.is_empty() => Some(
                Arc::new(DiscordNotifier::new(token.clone(), channel.clone())),
            ),
            _ => None,
        };

        let trade_store = Arc::new(TradeStore::new(data_dir.clone()));
        let stats_store = Arc::new(StatsStore::new(data_dir.clone()));
        let order_store = Arc::new(OrderStore::new(data_dir.clone()));
        let risk_manager = Arc::new(Mutex::new(RiskManager::default()));
        let position_tracker = Arc::new(Mutex::new(PositionTracker::new()));
        let overseas_position_tracker = Arc::new(Mutex::new(OverseasPositionTracker::new()));
        let exchange_rate_krw = Arc::new(RwLock::new(1450.0_f64));
        let exchange_rate_status = Arc::new(RwLock::new(ExchangeRateView::default_krw()));
        let initial_active_profile_id = profiles.active_id.clone();
        let profiles = Arc::new(RwLock::new(profiles));

        // rest_client를 RwLock으로 감싸서 OrderManager와 공유
        let rest_client_rw = Arc::new(RwLock::new(rest_client));

        let order_manager = Arc::new(Mutex::new(OrderManager::new(
            Arc::clone(&rest_client_rw),
            Arc::clone(&profiles),
            Arc::clone(&order_store),
            Arc::clone(&trade_store),
            Arc::clone(&position_tracker),
            Arc::clone(&overseas_position_tracker),
            Arc::clone(&stats_store),
            Arc::clone(&exchange_rate_krw),
            Arc::clone(&risk_manager),
            discord.clone(),
        )));

        // 기본 MA 크로스 전략 등록
        let mut strategy_manager = StrategyManager::new();
        let strategy_broker_id = config.broker_id;
        let strategy_account_id = if config.broker_account_id.is_empty() {
            None
        } else {
            Some(config.broker_account_id.clone())
        };
        let scoped_strategy =
            |cfg: StrategyConfig| cfg.with_scope(strategy_broker_id, strategy_account_id.clone());
        let default_strategy = scoped_strategy(StrategyConfig::new(
            "ma_cross_default",
            "이동평균 교차 전략",
            false,
            vec![],
            1,
            serde_json::to_value(MaCrossParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(MovingAverageCrossStrategy::new(default_strategy)));

        // RSI 전략 (기본 등록, 비활성)
        let rsi_strategy = scoped_strategy(StrategyConfig::new(
            "rsi_default",
            "RSI 전략",
            false,
            vec![],
            1,
            serde_json::to_value(RsiParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(RsiStrategy::new(rsi_strategy)));

        // 모멘텀 전략 (기본 등록, 비활성)
        let momentum_strategy = scoped_strategy(StrategyConfig::new(
            "momentum_default",
            "모멘텀 전략",
            false,
            vec![],
            1,
            serde_json::to_value(MomentumParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(MomentumStrategy::new(momentum_strategy)));

        // 이격도 전략 (기본 등록, 비활성)
        let deviation_strategy = scoped_strategy(StrategyConfig::new(
            "deviation_default",
            "이격도 전략",
            false,
            vec![],
            1,
            serde_json::to_value(DeviationParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(DeviationStrategy::new(deviation_strategy)));

        // 52주 신고가 전략 (기본 등록, 비활성)
        let fifty_two_week_high_strategy = scoped_strategy(StrategyConfig::new(
            "fifty_two_week_high_default",
            "52주 신고가 전략",
            false,
            vec![],
            1,
            serde_json::to_value(FiftyTwoWeekHighParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(FiftyTwoWeekHighStrategy::new(
            fifty_two_week_high_strategy,
        )));

        // 연속 상승/하락 전략 (기본 등록, 비활성)
        let consecutive_move_strategy = scoped_strategy(StrategyConfig::new(
            "consecutive_move_default",
            "연속 상승/하락 전략",
            false,
            vec![],
            1,
            serde_json::to_value(ConsecutiveMoveParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(ConsecutiveMoveStrategy::new(
            consecutive_move_strategy,
        )));

        // 돌파 실패 전략 (기본 등록, 비활성)
        let failed_breakout_strategy = scoped_strategy(StrategyConfig::new(
            "failed_breakout_default",
            "돌파 실패 전략",
            false,
            vec![],
            1,
            serde_json::to_value(FailedBreakoutParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(FailedBreakoutStrategy::new(
            failed_breakout_strategy,
        )));

        // 강한 종가 전략 (기본 등록, 비활성)
        let strong_close_strategy = scoped_strategy(StrategyConfig::new(
            "strong_close_default",
            "강한 종가 전략",
            false,
            vec![],
            1,
            serde_json::to_value(StrongCloseParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(StrongCloseStrategy::new(strong_close_strategy)));

        // 변동성 확장 전략 (기본 등록, 비활성)
        let volatility_expansion_strategy = scoped_strategy(StrategyConfig::new(
            "volatility_expansion_default",
            "변동성 확장 전략",
            false,
            vec![],
            1,
            serde_json::to_value(VolatilityExpansionParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(VolatilityExpansionStrategy::new(
            volatility_expansion_strategy,
        )));

        // 평균회귀 전략 (기본 등록, 비활성)
        let mean_reversion_strategy = scoped_strategy(StrategyConfig::new(
            "mean_reversion_default",
            "평균회귀 전략 (볼린저 밴드)",
            false,
            vec![],
            1,
            serde_json::to_value(MeanReversionParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(MeanReversionStrategy::new(
            mean_reversion_strategy,
        )));

        // 추세 필터 전략 (기본 등록, 비활성)
        let trend_filter_strategy = scoped_strategy(StrategyConfig::new(
            "trend_filter_default",
            "추세 필터 전략",
            false,
            vec![],
            1,
            serde_json::to_value(TrendFilterParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(TrendFilterStrategy::new(trend_filter_strategy)));

        // 레버리지 추세 보유 전략 (기본 등록, 비활성)
        let leveraged_trend_hold_strategy = scoped_strategy(StrategyConfig::new(
            "leveraged_trend_hold_default",
            "LeveragedTrendHoldStrategy",
            false,
            vec![],
            1,
            serde_json::to_value(LeveragedTrendHoldParams::default()).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(LeveragedTrendHoldStrategy::new(
            leveraged_trend_hold_strategy,
        )));

        // 가격 조건 매매 전략 (기본 등록, 비활성)
        let price_condition_strategy = scoped_strategy(StrategyConfig::new(
            "price_condition_default",
            "가격 조건 매매",
            false,
            vec![],
            1,
            serde_json::to_value(PriceConditionParams { symbols: vec![] }).unwrap_or_default(),
        ));
        strategy_manager.add(Box::new(PriceConditionStrategy::new(
            price_condition_strategy,
        )));

        // 전략 설정 영구 저장소
        let strategy_store = Arc::new(StrategyStore::new(&data_dir));

        // 저장된 전략 설정 로드 (프로파일별, 프로그램 재시작 후 복원)
        if let Some(profile_id) = initial_active_profile_id.as_deref() {
            let saved = strategy_store
                .load(profile_id)
                .await
                .unwrap_or_else(|error| {
                    tracing::error!("전략 설정 복원 실패 (프로파일 {}): {}", profile_id, error);
                    Vec::new()
                });
            strategy_manager.apply_saved_configs_for_scope(
                &saved,
                strategy_broker_id,
                strategy_account_id.clone(),
            );
            tracing::info!(
                "전략 설정 복원: 프로파일 '{}', {}개 전략",
                profile_id,
                saved.len()
            );
        }

        Self {
            config: Arc::new(RwLock::new(config)),
            rest_client: rest_client_rw,
            discord,
            discord_config,
            profiles,
            profiles_path,
            trade_store,
            stats_store,
            order_store,
            is_trading: Arc::new(Mutex::new(false)),
            storage_maintenance: Arc::new(Mutex::new(())),
            strategy_manager: Arc::new(Mutex::new(strategy_manager)),
            strategy_update_lock: Arc::new(Mutex::new(())),
            position_tracker,
            overseas_position_tracker,
            order_manager,
            risk_manager,
            log_dir,
            log_config: Arc::new(RwLock::new(log_config)),
            trade_archive_config: Arc::new(RwLock::new(
                TradeArchiveConfig::load_or_default(&data_dir).await,
            )),
            data_dir: data_dir.clone(),
            stock_list: Arc::new(RwLock::new(vec![])),
            stock_store: Arc::new(StockStore::new(&data_dir).await),
            strategy_store,
            web_port,
            ws_connected: Arc::new(AtomicBool::new(false)),
            trading_profile_id: Arc::new(RwLock::new(None)),
            trading_broker_id: Arc::new(RwLock::new(None)),
            trading_account_id: Arc::new(RwLock::new(None)),
            exchange_rate_krw,
            exchange_rate_status,
            refresh_config: Arc::new(RwLock::new(refresh_config)),
            refresh_interval_tx: Arc::new(refresh_interval_tx),
            database_manager,
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

fn usd_to_cents(value: &str) -> u64 {
    value
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .map(|v| (v * 100.0).round() as u64)
        .unwrap_or(0)
}

fn decimal_quantity_to_position_units(value: &str) -> u64 {
    let parsed = parse_amount_f64(value);
    if parsed <= 0.0 {
        0
    } else {
        parsed.floor().max(1.0) as u64
    }
}

fn broker_money_to_strategy_price_units(money: &BrokerMoney) -> u64 {
    match money.currency {
        BrokerCurrency::Krw => parse_amount_f64(&money.amount).round().max(0.0) as u64,
        BrokerCurrency::Usd => usd_to_cents(&money.amount),
    }
}

fn normalize_overseas_order_exchange(code: &str) -> String {
    match code.trim().to_uppercase().as_str() {
        "NAS" | "NASD" | "NASDAQ" => "NASD".to_string(),
        "NYS" | "NYSE" => "NYSE".to_string(),
        "AMS" | "AMEX" => "AMEX".to_string(),
        other => other.to_string(),
    }
}

fn parse_amount_i64(value: &str) -> i64 {
    value.trim().replace(',', "").parse::<i64>().unwrap_or(0)
}

fn parse_amount_f64(value: &str) -> f64 {
    value.trim().replace(',', "").parse::<f64>().unwrap_or(0.0)
}

fn balance_summary_total_krw(summary: Option<&BalanceSummary>) -> i64 {
    let Some(summary) = summary else {
        return 0;
    };
    let net_asset = parse_amount_i64(&summary.nass_amt);
    if net_asset > 0 {
        return net_asset;
    }
    parse_amount_i64(&summary.tot_evlu_amt)
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
