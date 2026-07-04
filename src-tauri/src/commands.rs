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
        BrokerAccountId, BrokerAdapter, BrokerCurrency, BrokerHolding, BrokerId, BrokerMarket,
        BrokerMoney, BrokerOrderSide, BrokerQuantity, BrokerScope, BrokerSymbol, KisBrokerAdapter,
        TossBrokerAdapter,
    },
    config::{AccountProfile, AppConfig, DiscordConfig, ProfilesConfig},
    logging::LogConfig,
    market_hours::{
        is_domestic_symbol, is_market_open_for_with_calendar, open_markets_summary_with_calendar,
        MarketCalendarOverride, MarketDayCalendar, MarketSessionWindow,
    },
    notifications::{discord::DiscordNotifier, types::NotificationEvent},
    storage::{
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
    },
};

// ────────────────────────────────────────────────────────────────────
// 체결 기록 보관 설정
// ────────────────────────────────────────────────────────────────────

/// 체결 기록 보관 설정 (보관 기간, 최대 저장 용량)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeArchiveConfig {
    pub retention_days: u32, // 보관 기간 (일), 기본 90
    pub max_size_mb: u64,    // 최대 저장 용량 (MB), 기본 500
}

impl Default for TradeArchiveConfig {
    fn default() -> Self {
        Self {
            retention_days: 90,
            max_size_mb: 500,
        }
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
    fn default_krw() -> Self {
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
    let Ok(year_entries) = std::fs::read_dir(&trades_dir) else {
        return result;
    };
    for year_entry in year_entries.flatten() {
        let year_path = year_entry.path();
        if !year_path.is_dir() {
            continue;
        }
        let Some(year_str) = year_entry.file_name().into_string().ok() else {
            continue;
        };
        let Ok(year) = year_str.parse::<i32>() else {
            continue;
        };
        let Ok(month_entries) = std::fs::read_dir(&year_path) else {
            continue;
        };
        for month_entry in month_entries.flatten() {
            let month_path = month_entry.path();
            if !month_path.is_dir() {
                continue;
            }
            let Some(month_str) = month_entry.file_name().into_string().ok() else {
                continue;
            };
            let Ok(month) = month_str.parse::<u32>() else {
                continue;
            };
            let Ok(day_entries) = std::fs::read_dir(&month_path) else {
                continue;
            };
            for day_entry in day_entries.flatten() {
                let day_path = day_entry.path();
                if !day_path.is_dir() {
                    continue;
                }
                let Some(day_str) = day_entry.file_name().into_string().ok() else {
                    continue;
                };
                let Ok(day) = day_str.parse::<u32>() else {
                    continue;
                };
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
/// lib.rs 시작 시 및 일일 정리 데몬에서 호출 가능하도록 pub
pub fn purge_old_trade_files(data_dir: &Path, cfg: &TradeArchiveConfig) {
    let cutoff =
        chrono::Local::now().date_naive() - chrono::Duration::days(cfg.retention_days as i64);

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
        if total_size <= max_bytes {
            break;
        }
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
        refresh_config: RefreshConfig,
        refresh_interval_tx: watch::Sender<u64>,
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

        // rest_client를 RwLock으로 감싸서 OrderManager와 공유
        let rest_client_rw = Arc::new(RwLock::new(rest_client));

        let order_manager = Arc::new(Mutex::new(OrderManager::new(
            Arc::clone(&rest_client_rw),
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
        if let Some(profile_id) = profiles.active_id.as_deref() {
            let saved = strategy_store.load_sync(profile_id);
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
            profiles: Arc::new(RwLock::new(profiles)),
            profiles_path,
            trade_store,
            stats_store,
            order_store,
            is_trading: Arc::new(Mutex::new(false)),
            strategy_manager: Arc::new(Mutex::new(strategy_manager)),
            position_tracker,
            overseas_position_tracker,
            order_manager,
            risk_manager,
            log_dir,
            log_config: Arc::new(RwLock::new(log_config)),
            trade_archive_config: Arc::new(RwLock::new(TradeArchiveConfig::load_or_default(
                &data_dir,
            ))),
            data_dir: data_dir.clone(),
            stock_list: Arc::new(RwLock::new(vec![])),
            stock_store: Arc::new(StockStore::new(&data_dir)),
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

async fn fetch_account_risk_balance_krw(
    rest: &Arc<KisRestClient>,
    include_overseas: bool,
    exchange_rate_krw: f64,
) -> i64 {
    let domestic_total = match rest.get_balance().await {
        Ok(resp) => balance_summary_total_krw(resp.summary.as_ref()),
        Err(e) => {
            tracing::warn!("리스크 총잔고 조회 실패(국내): {}", e);
            0
        }
    };

    if !include_overseas {
        return domestic_total;
    }

    let overseas_total = match rest.get_overseas_balance().await {
        Ok(resp) => resp
            .summary
            .as_ref()
            .map(|s| (parse_amount_f64(&s.frcr_evlu_tota) * exchange_rate_krw).round() as i64)
            .unwrap_or(0),
        Err(e) => {
            tracing::warn!("리스크 총잔고 조회 실패(해외): {}", e);
            0
        }
    };

    domestic_total.saturating_add(overseas_total.max(0))
}

fn calculate_atr(candles: &[OhlcCandle], period: usize) -> Option<u64> {
    if candles.len() < 2 || period == 0 {
        return None;
    }
    let start = candles.len().saturating_sub(period).max(1);
    let mut ranges = Vec::with_capacity(candles.len().saturating_sub(start));
    for idx in start..candles.len() {
        let candle = candles[idx];
        let prev_close = candles[idx - 1].close;
        let high_low = candle.high.saturating_sub(candle.low);
        let high_prev = candle.high.abs_diff(prev_close);
        let low_prev = candle.low.abs_diff(prev_close);
        ranges.push(high_low.max(high_prev).max(low_prev));
    }
    if ranges.is_empty() {
        return None;
    }
    Some((ranges.iter().sum::<u64>() as f64 / ranges.len() as f64).round() as u64)
        .filter(|atr| *atr > 0)
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
            BrokerId::Toss => "read-only".into(),
        },
        is_ready: cfg.is_active_broker_configured(),
        discord_configured: cfg.discord_bot_token.is_some(),
        base_url: cfg.kis_base_url().to_string(),
        issues,
    })
}

#[derive(Debug, Serialize)]
pub struct TossConnectionStep {
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TossAccountOptionView {
    pub account_seq: String,
    pub account_no_masked: String,
    pub account_type: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct TossAccountLookupInput {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
pub struct TossConnectionDiagnostic {
    pub profile_id: String,
    pub profile_name: String,
    pub broker_id: BrokerId,
    pub account_seq: String,
    pub openapi_title: Option<String>,
    pub openapi_version: Option<String>,
    pub openapi_server: Option<String>,
    pub openapi_paths_count: Option<usize>,
    pub token_type: Option<String>,
    pub token_expires_at: Option<String>,
    pub accounts_count: Option<usize>,
    pub matched_account_no: Option<String>,
    pub holdings_count: Option<usize>,
    pub buying_power_krw: Option<String>,
    pub buying_power_usd: Option<String>,
    pub commissions_count: Option<usize>,
    pub sellable_quantity_symbol: Option<String>,
    pub sellable_quantity: Option<String>,
    pub is_ready: bool,
    pub steps: Vec<TossConnectionStep>,
    pub issues: Vec<String>,
}

fn toss_diag_step(
    id: impl Into<String>,
    label: impl Into<String>,
    ok: bool,
    message: impl Into<String>,
) -> TossConnectionStep {
    TossConnectionStep {
        id: id.into(),
        label: label.into(),
        ok,
        message: message.into(),
    }
}

fn mask_toss_account_no(account_no: &str) -> String {
    let suffix: String = account_no
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if suffix.is_empty() {
        "(계좌번호 없음)".to_string()
    } else {
        format!("****{suffix}")
    }
}

fn toss_account_option(account: crate::broker::toss::TossAccount) -> TossAccountOptionView {
    let account_seq = account.account_seq.to_string();
    let account_no_masked = mask_toss_account_no(&account.account_no);
    let label = format!(
        "accountSeq {} · {} · {}",
        account_seq, account_no_masked, account.account_type
    );
    TossAccountOptionView {
        account_seq,
        account_no_masked,
        account_type: account.account_type,
        label,
    }
}

pub(crate) async fn lookup_toss_accounts_with_credentials(
    client_id: String,
    client_secret: String,
) -> CmdResult<Vec<TossAccountOptionView>> {
    let client_id = client_id.trim();
    let client_secret = client_secret.trim();
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(CmdError {
            code: "MISSING_CREDENTIALS".into(),
            message: "토스증권 Client ID와 Client Secret을 모두 입력하세요.".into(),
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        client_id.to_string(),
        client_secret.to_string(),
        None::<String>,
    );
    adapter
        .list_accounts()
        .await
        .map(|accounts| accounts.into_iter().map(toss_account_option).collect())
        .map_err(|e| CmdError {
            code: "TOSS_ACCOUNTS_ERROR".into(),
            message: e.to_string(),
        })
}

#[tauri::command]
pub async fn list_toss_accounts(
    input: TossAccountLookupInput,
) -> CmdResult<Vec<TossAccountOptionView>> {
    lookup_toss_accounts_with_credentials(input.client_id, input.client_secret).await
}

#[tauri::command]
pub async fn list_toss_profile_accounts(
    profile_id: String,
    state: State<'_, AppState>,
) -> CmdResult<Vec<TossAccountOptionView>> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .cloned()
            .ok_or_else(|| CmdError {
                code: "NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {profile_id}"),
            })?
    };

    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_MISMATCH".into(),
            message: "토스증권 프로파일만 accountSeq 목록을 조회할 수 있습니다.".into(),
        });
    }

    lookup_toss_accounts_with_credentials(profile.app_key, profile.app_secret).await
}

pub(crate) async fn run_toss_connection_diagnostic(
    profile: AccountProfile,
) -> TossConnectionDiagnostic {
    let account_seq = profile.broker_account_id();
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(account_seq.clone()),
    );

    let mut steps = Vec::new();
    let mut issues = Vec::new();
    let mut openapi_title = None;
    let mut openapi_version = None;
    let mut openapi_server = None;
    let mut openapi_paths_count = None;
    let mut token_type = None;
    let mut token_expires_at = None;
    let mut accounts_count = None;
    let mut matched_account_no = None;
    let mut holdings_count = None;
    let mut first_holding_symbol = None;
    let mut buying_power_krw = None;
    let mut buying_power_usd = None;
    let mut commissions_count = None;
    let mut sellable_quantity_symbol = None;
    let mut sellable_quantity = None;

    match adapter.openapi_overview().await {
        Ok(overview) => {
            let ok = overview.server == TossBrokerAdapter::DEFAULT_BASE_URL
                && !overview.version.is_empty()
                && overview.paths_count > 0;
            if !ok {
                issues.push("토스증권 OpenAPI 스펙 메타데이터가 예상과 다릅니다.".into());
            }
            steps.push(toss_diag_step(
                "openapi",
                "OpenAPI 스펙",
                ok,
                format!(
                    "{} v{} · paths {}",
                    overview.title, overview.version, overview.paths_count
                ),
            ));
            openapi_title = Some(overview.title);
            openapi_version = Some(overview.version);
            openapi_server = Some(overview.server);
            openapi_paths_count = Some(overview.paths_count);
        }
        Err(e) => {
            let message = e.to_string();
            issues.push(message.clone());
            steps.push(toss_diag_step("openapi", "OpenAPI 스펙", false, message));
        }
    }

    let credentials_present =
        !profile.app_key.trim().is_empty() && !profile.app_secret.trim().is_empty();
    let account_seq_valid = account_seq.trim().parse::<i64>().is_ok();

    if !credentials_present {
        let message = "토스증권 client_id/client_secret이 설정되지 않았습니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("token", "토큰 발급", false, message));
    } else {
        match adapter.check_token().await {
            Ok(token) => {
                token_type = Some(token.token_type.clone());
                token_expires_at = Some(token.expires_at.to_rfc3339());
                steps.push(toss_diag_step(
                    "token",
                    "토큰 발급",
                    true,
                    format!("{} token · 만료 {}", token.token_type, token.expires_at),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("token", "토큰 발급", false, message));
            }
        }
    }

    if credentials_present {
        match adapter.list_accounts().await {
            Ok(accounts) => {
                accounts_count = Some(accounts.len());
                matched_account_no = accounts
                    .iter()
                    .find(|account| account.account_seq.to_string() == account_seq)
                    .map(|account| account.account_no.clone());
                let ok = account_seq.trim().is_empty()
                    || matched_account_no.is_some()
                    || !account_seq_valid;
                if !ok {
                    issues.push(format!(
                        "저장된 accountSeq({account_seq})와 일치하는 토스 계좌를 찾지 못했습니다."
                    ));
                }
                let message = match &matched_account_no {
                    Some(account_no) => {
                        format!("{}개 계좌 조회 · 저장 계좌 {}", accounts.len(), account_no)
                    }
                    None if account_seq.trim().is_empty() => {
                        format!("{}개 계좌 조회 · accountSeq를 저장하세요", accounts.len())
                    }
                    None if !account_seq_valid => {
                        format!(
                            "{}개 계좌 조회 · accountSeq는 숫자여야 합니다",
                            accounts.len()
                        )
                    }
                    None => format!("{}개 계좌 조회 · 저장 계좌 불일치", accounts.len()),
                };
                steps.push(toss_diag_step("accounts", "계좌 조회", ok, message));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("accounts", "계좌 조회", false, message));
            }
        }
    } else {
        steps.push(toss_diag_step(
            "accounts",
            "계좌 조회",
            false,
            "토큰 발급 전이라 계좌 조회를 건너뛰었습니다.",
        ));
    }

    if account_seq.trim().is_empty() {
        let message = "토스증권 accountSeq가 설정되지 않았습니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
    } else if !account_seq_valid {
        let message = "토스증권 accountSeq는 숫자여야 합니다.".to_string();
        issues.push(message.clone());
        steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
    } else if credentials_present {
        let account_id = BrokerAccountId(account_seq.clone());
        match adapter.list_holdings(Some(&account_id)).await {
            Ok(holdings) => {
                holdings_count = Some(holdings.len());
                first_holding_symbol = holdings.first().map(|holding| holding.symbol.0.clone());
                steps.push(toss_diag_step(
                    "holdings",
                    "잔고 조회",
                    true,
                    format!("{}개 보유 종목 조회", holdings.len()),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("holdings", "잔고 조회", false, message));
            }
        }
    } else {
        steps.push(toss_diag_step(
            "holdings",
            "잔고 조회",
            false,
            "토큰 발급 전이라 잔고 조회를 건너뛰었습니다.",
        ));
    }

    if credentials_present && account_seq_valid && !account_seq.trim().is_empty() {
        match adapter
            .get_buying_power(Some(&account_seq), BrokerCurrency::Krw)
            .await
        {
            Ok(power) => {
                buying_power_krw = Some(power.cash_buying_power.clone());
                steps.push(toss_diag_step(
                    "buyingPowerKrw",
                    "매수가능금액(KRW)",
                    true,
                    format!("{} {}", power.cash_buying_power, power.currency),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step(
                    "buyingPowerKrw",
                    "매수가능금액(KRW)",
                    false,
                    message,
                ));
            }
        }

        match adapter
            .get_buying_power(Some(&account_seq), BrokerCurrency::Usd)
            .await
        {
            Ok(power) => {
                buying_power_usd = Some(power.cash_buying_power.clone());
                steps.push(toss_diag_step(
                    "buyingPowerUsd",
                    "매수가능금액(USD)",
                    true,
                    format!("{} {}", power.cash_buying_power, power.currency),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step(
                    "buyingPowerUsd",
                    "매수가능금액(USD)",
                    false,
                    message,
                ));
            }
        }

        match adapter.list_commissions(Some(&account_seq)).await {
            Ok(commissions) => {
                commissions_count = Some(commissions.len());
                steps.push(toss_diag_step(
                    "commissions",
                    "수수료 조회",
                    true,
                    format!("{}개 수수료 정책 조회", commissions.len()),
                ));
            }
            Err(e) => {
                let message = e.to_string();
                issues.push(message.clone());
                steps.push(toss_diag_step("commissions", "수수료 조회", false, message));
            }
        }

        if let Some(symbol) = &first_holding_symbol {
            let broker_symbol = BrokerSymbol(symbol.clone());
            match adapter
                .get_sellable_quantity(Some(&account_seq), &broker_symbol)
                .await
            {
                Ok(quantity) => {
                    sellable_quantity_symbol = Some(symbol.clone());
                    sellable_quantity = Some(quantity.sellable_quantity.clone());
                    steps.push(toss_diag_step(
                        "sellableQuantity",
                        "매도가능수량",
                        true,
                        format!("{}: {}", symbol, quantity.sellable_quantity),
                    ));
                }
                Err(e) => {
                    let message = e.to_string();
                    issues.push(message.clone());
                    steps.push(toss_diag_step(
                        "sellableQuantity",
                        "매도가능수량",
                        false,
                        message,
                    ));
                }
            }
        } else {
            steps.push(toss_diag_step(
                "sellableQuantity",
                "매도가능수량",
                true,
                "보유 종목이 없어 매도가능수량 조회를 건너뛰었습니다.",
            ));
        }
    }

    let is_ready = issues.is_empty() && steps.iter().all(|step| step.ok);

    TossConnectionDiagnostic {
        profile_id: profile.id,
        profile_name: profile.name,
        broker_id: BrokerId::Toss,
        account_seq,
        openapi_title,
        openapi_version,
        openapi_server,
        openapi_paths_count,
        token_type,
        token_expires_at,
        accounts_count,
        matched_account_no,
        holdings_count,
        buying_power_krw,
        buying_power_usd,
        commissions_count,
        sellable_quantity_symbol,
        sellable_quantity,
        is_ready,
        steps,
        issues,
    }
}

#[tauri::command]
pub async fn check_toss_profile_connection(
    profile_id: String,
    state: State<'_, AppState>,
) -> CmdResult<TossConnectionDiagnostic> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .cloned()
            .ok_or_else(|| CmdError {
                code: "NOT_FOUND".into(),
                message: format!("프로파일을 찾을 수 없습니다: {profile_id}"),
            })?
    };

    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_MISMATCH".into(),
            message: "토스증권 프로파일만 Toss 연결 진단을 실행할 수 있습니다.".into(),
        });
    }

    Ok(run_toss_connection_diagnostic(profile).await)
}

// ────────────────────────────────────────────────────────────────────
// 계좌 프로파일 관리
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProfileView {
    pub id: String,
    pub broker_id: BrokerId,
    pub broker_account_id: String,
    pub name: String,
    pub is_paper_trading: bool,
    pub live_trading_consent: bool,
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
        broker_id: p.broker_id,
        broker_account_id: p.broker_account_id(),
        name: p.name.clone(),
        is_paper_trading: p.is_paper_trading,
        live_trading_consent: p.live_trading_consent,
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
    #[serde(default = "default_input_broker_id")]
    pub broker_id: BrokerId,
    pub name: String,
    pub is_paper_trading: bool,
    #[serde(default)]
    pub live_trading_consent: bool,
    pub app_key: String,
    pub app_secret: String,
    pub account_no: String,
}

fn default_input_broker_id() -> BrokerId {
    BrokerId::Kis
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
    let profile = AccountProfile {
        broker_id: input.broker_id,
        live_trading_consent: input.live_trading_consent,
        ..profile
    };

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
    pub broker_id: Option<BrokerId>,
    pub name: Option<String>,
    pub is_paper_trading: Option<bool>,
    pub live_trading_consent: Option<bool>,
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
                input.broker_id,
                input.name,
                input.is_paper_trading,
                input.live_trading_consent,
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
pub async fn delete_profile(id: String, state: State<'_, AppState>) -> CmdResult<()> {
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
    let active_scope = {
        let cfg = state.config.read().await.clone();
        let account_id = if cfg.broker_account_id.is_empty() {
            None
        } else {
            Some(cfg.broker_account_id.clone())
        };
        (cfg.broker_id, account_id)
    };
    if let Some(pid) = &active_id {
        let saved = state.strategy_store.load_sync(pid);
        let mut mgr = state.strategy_manager.lock().await;
        mgr.apply_saved_configs_for_scope(&saved, active_scope.0, active_scope.1.clone());
        tracing::info!(
            "프로파일 전환 — 전략 설정 복원: 프로파일 '{}', {}개 전략",
            pid,
            saved.len()
        );
    } else {
        let mut mgr = state.strategy_manager.lock().await;
        mgr.apply_saved_configs_for_scope(&[], active_scope.0, active_scope.1.clone());
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
                resp.summary
                    .as_ref()
                    .map(|s| s.tot_evlu_amt.as_str())
                    .unwrap_or("미제공")
            );
            // 잔고 응답의 종목코드+이름 데이터 자동 수집
            state
                .stock_store
                .upsert_many(
                    resp.items
                        .iter()
                        .map(|i| (i.pdno.clone(), i.prdt_name.clone())),
                )
                .await;
            // 앱 재시작 후 position_tracker가 비어있으면 잔고 응답으로 복원
            {
                let mut tracker = state.position_tracker.lock().await;
                tracker.load_if_empty(resp.items.iter().map(|i| {
                    (
                        i.pdno.clone(),
                        i.prdt_name.clone(),
                        i.hldg_qty.parse::<u64>().unwrap_or(0),
                        i.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64,
                        i.prpr.parse::<u64>().unwrap_or(0),
                    )
                }));
            }
            Ok(BalanceResult {
                items: resp.items,
                summary: resp.summary,
            })
        }
        Err(e) => {
            tracing::error!("잔고 조회 실패: {}", e);
            Err(CmdError::from(e))
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 해외 잔고 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OverseasBalanceResult {
    pub items: Vec<OverseasBalanceItem>,
    pub summary: Option<OverseasBalanceSummary>,
}

#[tauri::command]
pub async fn get_overseas_balance(state: State<'_, AppState>) -> CmdResult<OverseasBalanceResult> {
    let client = state.rest_client.read().await.clone();
    match client.get_overseas_balance().await {
        Ok(resp) => {
            tracing::info!("해외 잔고 조회 성공: 보유종목 {}개", resp.items.len());
            // 해외 잔고는 국내 position_tracker에 혼입하지 않고 별도 tracker에만 복원한다.
            {
                let mut tracker = state.overseas_position_tracker.lock().await;
                tracker.load_if_empty(resp.items.iter().map(|i| {
                    (
                        i.ovrs_pdno.clone(),
                        i.ovrs_item_name.clone(),
                        normalize_overseas_order_exchange(&i.ovrs_excg_cd),
                        i.ovrs_cblc_qty.parse::<u64>().unwrap_or(0),
                        usd_to_cents(&i.pchs_avg_pric),
                        usd_to_cents(&i.now_pric2),
                    )
                }));
            }
            Ok(OverseasBalanceResult {
                items: resp.items,
                summary: resp.summary,
            })
        }
        Err(e) => {
            tracing::error!("해외 잔고 조회 실패: {}", e);
            Err(CmdError::from(e))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerMoneyView {
    pub amount: String,
    pub currency: BrokerCurrency,
}

impl From<BrokerMoney> for BrokerMoneyView {
    fn from(money: BrokerMoney) -> Self {
        Self {
            amount: money.amount,
            currency: money.currency,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerHoldingView {
    pub broker_id: BrokerId,
    pub account_id: Option<String>,
    pub market: BrokerMarket,
    pub symbol: String,
    pub symbol_name: String,
    pub quantity: String,
    pub average_price: BrokerMoneyView,
    pub current_price: BrokerMoneyView,
    pub unrealized_pnl: Option<BrokerMoneyView>,
}

impl From<BrokerHolding> for BrokerHoldingView {
    fn from(holding: BrokerHolding) -> Self {
        Self {
            broker_id: holding.broker,
            account_id: holding.account_id.map(|id| id.0),
            market: holding.market,
            symbol: holding.symbol.0,
            symbol_name: holding.symbol_name,
            quantity: holding.quantity.0,
            average_price: holding.average_price.into(),
            current_price: holding.current_price.into(),
            unrealized_pnl: holding.unrealized_pnl.map(Into::into),
        }
    }
}

pub async fn list_broker_holdings_for_profile(
    profile: AccountProfile,
    rest_client: Arc<KisRestClient>,
) -> Result<Vec<BrokerHoldingView>, CmdError> {
    let account_id = BrokerAccountId(profile.broker_account_id());
    let holdings = match profile.broker_id {
        BrokerId::Kis => {
            let adapter = KisBrokerAdapter::new(rest_client);
            adapter.list_holdings(Some(&account_id)).await
        }
        BrokerId::Toss => {
            let adapter = TossBrokerAdapter::with_credentials(
                TossBrokerAdapter::DEFAULT_BASE_URL,
                profile.app_key,
                profile.app_secret,
                Some(profile.account_no),
            );
            adapter.list_holdings(Some(&account_id)).await
        }
    }
    .map_err(|e| CmdError {
        code: "BROKER_HOLDINGS_ERROR".into(),
        message: e.to_string(),
    })?;

    let mut views: Vec<BrokerHoldingView> =
        holdings.into_iter().map(BrokerHoldingView::from).collect();
    views.sort_by(|a, b| {
        broker_market_sort_key(a.market)
            .cmp(&broker_market_sort_key(b.market))
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    Ok(views)
}

fn broker_market_sort_key(market: BrokerMarket) -> u8 {
    match market {
        BrokerMarket::Kr => 0,
        BrokerMarket::Us => 1,
    }
}

#[tauri::command]
pub async fn get_broker_holdings(state: State<'_, AppState>) -> CmdResult<Vec<BrokerHoldingView>> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    let Some(profile) = profile else {
        return Ok(Vec::new());
    };
    let rest_client = state.rest_client.read().await.clone();
    list_broker_holdings_for_profile(profile, rest_client).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderbookView {
    pub timestamp: Option<String>,
    pub currency: String,
    pub asks: Vec<TossOrderbookEntry>,
    pub bids: Vec<TossOrderbookEntry>,
}

impl From<TossOrderbookResponse> for TossOrderbookView {
    fn from(value: TossOrderbookResponse) -> Self {
        Self {
            timestamp: value.timestamp,
            currency: value.currency,
            asks: value.asks,
            bids: value.bids,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossTradeView {
    pub price: String,
    pub volume: String,
    pub timestamp: String,
    pub currency: String,
}

impl From<TossTrade> for TossTradeView {
    fn from(value: TossTrade) -> Self {
        Self {
            price: value.price,
            volume: value.volume,
            timestamp: value.timestamp,
            currency: value.currency,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossPriceLimitView {
    pub timestamp: String,
    pub upper_limit_price: Option<String>,
    pub lower_limit_price: Option<String>,
    pub currency: String,
}

impl From<TossPriceLimitResponse> for TossPriceLimitView {
    fn from(value: TossPriceLimitResponse) -> Self {
        Self {
            timestamp: value.timestamp,
            upper_limit_price: value.upper_limit_price,
            lower_limit_price: value.lower_limit_price,
            currency: value.currency,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketSnapshotView {
    pub broker_id: BrokerId,
    pub market: BrokerMarket,
    pub symbol: String,
    pub timestamp: Option<String>,
    pub price: BrokerMoneyView,
    pub orderbook: TossOrderbookView,
    pub trades: Vec<TossTradeView>,
    pub price_limits: TossPriceLimitView,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossStockInfoView {
    pub symbol: String,
    pub name: String,
    pub english_name: String,
    pub isin_code: String,
    pub market: String,
    pub security_type: String,
    pub is_common_share: bool,
    pub status: String,
    pub currency: String,
    pub list_date: Option<String>,
    pub delist_date: Option<String>,
    pub shares_outstanding: String,
    pub leverage_factor: Option<String>,
}

impl From<TossStockInfo> for TossStockInfoView {
    fn from(value: TossStockInfo) -> Self {
        Self {
            symbol: value.symbol,
            name: value.name,
            english_name: value.english_name,
            isin_code: value.isin_code,
            market: value.market,
            security_type: value.security_type,
            is_common_share: value.is_common_share,
            status: value.status,
            currency: value.currency,
            list_date: value.list_date,
            delist_date: value.delist_date,
            shares_outstanding: value.shares_outstanding,
            leverage_factor: value.leverage_factor,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossStockWarningView {
    pub warning_type: String,
    pub label: String,
    pub exchange: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub blocking_for_buy: bool,
}

impl From<TossStockWarning> for TossStockWarningView {
    fn from(value: TossStockWarning) -> Self {
        let blocking_for_buy = value.is_blocking_for_buy();
        let label = toss_warning_label(&value.warning_type).to_string();
        Self {
            warning_type: value.warning_type,
            label,
            exchange: value.exchange,
            start_date: value.start_date,
            end_date: value.end_date,
            blocking_for_buy,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossStockSafetyView {
    pub broker_id: BrokerId,
    pub symbol: String,
    pub stock_info: Option<TossStockInfoView>,
    pub warnings: Vec<TossStockWarningView>,
    pub has_blocking_warning: bool,
    pub buy_blocked: bool,
    pub buy_block_reason: Option<String>,
}

fn toss_warning_label(warning_type: &str) -> &str {
    match warning_type {
        "LIQUIDATION_TRADING" => "정리매매",
        "OVERHEATED" => "단기과열",
        "INVESTMENT_WARNING" => "투자경고",
        "INVESTMENT_RISK" => "투자위험",
        "VI_STATIC_AND_DYNAMIC" => "VI 정적+동적",
        "VI_STATIC" => "VI 정적",
        "VI_DYNAMIC" => "VI 동적",
        "STOCK_WARRANTS" => "신주인수권",
        _ => warning_type,
    }
}

fn normalize_toss_symbol(symbol: String) -> CmdResult<String> {
    let symbol = symbol.trim().to_uppercase();
    if symbol.is_empty() || symbol.len() > 32 {
        return Err(CmdError {
            code: "INVALID_SYMBOL".into(),
            message: format!("Toss symbol은 1~32자여야 합니다: {symbol}"),
        });
    }
    if !symbol
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(CmdError {
            code: "INVALID_SYMBOL".into(),
            message: format!("Toss symbol은 영문/숫자/./- 문자만 허용합니다: {symbol}"),
        });
    }
    Ok(symbol)
}

pub async fn get_toss_stock_safety_for_profile(
    symbol: String,
    profile: AccountProfile,
) -> CmdResult<TossStockSafetyView> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 종목 유의사항은 Toss 활성 프로파일에서만 조회할 수 있습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(symbol)?;
    let broker_symbol = BrokerSymbol(symbol.clone());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );

    let (stocks, warnings) = tokio::try_join!(
        adapter.list_stocks(std::slice::from_ref(&broker_symbol)),
        adapter.list_warnings(&broker_symbol),
    )
    .map_err(|e| CmdError {
        code: "TOSS_STOCK_SAFETY_ERROR".into(),
        message: e.to_string(),
    })?;

    let stock_info = stocks
        .into_iter()
        .find(|item| item.symbol.eq_ignore_ascii_case(&symbol))
        .map(TossStockInfoView::from);
    let status_block_reason = stock_info.as_ref().and_then(|info| {
        (info.status != "ACTIVE").then(|| format!("상장 상태가 ACTIVE가 아닙니다: {}", info.status))
    });
    let warnings: Vec<TossStockWarningView> = warnings
        .into_iter()
        .map(TossStockWarningView::from)
        .collect();
    let blocking_labels = warnings
        .iter()
        .filter(|warning| warning.blocking_for_buy)
        .map(|warning| warning.label.as_str())
        .collect::<Vec<_>>();
    let has_blocking_warning = !blocking_labels.is_empty();
    let warning_block_reason =
        has_blocking_warning.then(|| format!("매수 유의사항: {}", blocking_labels.join(", ")));
    let buy_block_reason = match (status_block_reason, warning_block_reason) {
        (Some(status), Some(warnings)) => Some(format!("{status}; {warnings}")),
        (Some(status), None) => Some(status),
        (None, Some(warnings)) => Some(warnings),
        (None, None) => None,
    };
    let buy_blocked = buy_block_reason.is_some();

    Ok(TossStockSafetyView {
        broker_id: BrokerId::Toss,
        symbol,
        stock_info,
        warnings,
        has_blocking_warning,
        buy_blocked,
        buy_block_reason,
    })
}

#[tauri::command]
pub async fn get_toss_stock_safety(
    symbol: String,
    state: State<'_, AppState>,
) -> CmdResult<TossStockSafetyView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    get_toss_stock_safety_for_profile(symbol, profile).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketSessionView {
    pub start_time: String,
    pub end_time: String,
}

impl From<&TossMarketSession> for TossMarketSessionView {
    fn from(value: &TossMarketSession) -> Self {
        Self {
            start_time: value.start_time.clone(),
            end_time: value.end_time.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketDayView {
    pub date: String,
    pub regular_session: Option<TossMarketSessionView>,
    pub is_regular_open: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossMarketCalendarView {
    pub broker_id: BrokerId,
    pub kr: TossMarketDayView,
    pub us: TossMarketDayView,
    pub summary: String,
}

fn toss_market_day_calendar(session: Option<&TossMarketSession>) -> CmdResult<MarketDayCalendar> {
    match session {
        Some(session) => {
            let regular_session =
                MarketSessionWindow::parse(&session.start_time, &session.end_time).ok_or_else(
                    || CmdError {
                        code: "TOSS_MARKET_CALENDAR_MAPPING_ERROR".into(),
                        message: format!(
                            "Toss market-calendar 세션 시간을 해석할 수 없습니다: {} ~ {}",
                            session.start_time, session.end_time
                        ),
                    },
                )?;
            Ok(MarketDayCalendar {
                regular_session: Some(regular_session),
            })
        }
        None => Ok(MarketDayCalendar::closed()),
    }
}

fn toss_calendar_override(
    kr: &TossKrMarketCalendarResponse,
    us: &TossUsMarketCalendarResponse,
) -> CmdResult<MarketCalendarOverride> {
    let kr_regular = kr
        .today
        .integrated
        .as_ref()
        .and_then(|integrated| integrated.regular_market.as_ref());
    let us_regular = us.today.regular_market.as_ref();
    Ok(MarketCalendarOverride {
        kr: Some(toss_market_day_calendar(kr_regular)?),
        us: Some(toss_market_day_calendar(us_regular)?),
    })
}

fn toss_calendar_view(
    kr: TossKrMarketCalendarResponse,
    us: TossUsMarketCalendarResponse,
) -> CmdResult<TossMarketCalendarView> {
    let override_calendar = toss_calendar_override(&kr, &us)?;
    let kst = chrono::FixedOffset::east_opt(9 * 3600).expect("KST FixedOffset 생성 실패");
    let now = chrono::Utc::now().with_timezone(&kst);
    let kr_regular = kr
        .today
        .integrated
        .as_ref()
        .and_then(|integrated| integrated.regular_market.as_ref());
    let us_regular = us.today.regular_market.as_ref();

    Ok(TossMarketCalendarView {
        broker_id: BrokerId::Toss,
        kr: TossMarketDayView {
            date: kr.today.date,
            regular_session: kr_regular.map(TossMarketSessionView::from),
            is_regular_open: override_calendar
                .kr
                .as_ref()
                .is_some_and(|day| day.is_open_at(now)),
        },
        us: TossMarketDayView {
            date: us.today.date,
            regular_session: us_regular.map(TossMarketSessionView::from),
            is_regular_open: override_calendar
                .us
                .as_ref()
                .is_some_and(|day| day.is_open_at(now)),
        },
        summary: open_markets_summary_with_calendar(Some(&override_calendar)),
    })
}

pub async fn get_toss_market_calendar_for_profile(
    profile: AccountProfile,
) -> CmdResult<TossMarketCalendarView> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss market-calendar는 Toss 활성 프로파일에서만 조회할 수 있습니다.".into(),
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let (kr, us) = tokio::try_join!(
        adapter.get_kr_market_calendar(None),
        adapter.get_us_market_calendar(None),
    )
    .map_err(|e| CmdError {
        code: "TOSS_MARKET_CALENDAR_ERROR".into(),
        message: e.to_string(),
    })?;
    toss_calendar_view(kr, us)
}

async fn get_active_toss_calendar_override(
    profiles: &Arc<RwLock<ProfilesConfig>>,
) -> Option<MarketCalendarOverride> {
    let profile = {
        let profiles = profiles.read().await;
        profiles.get_active().cloned()
    }?;
    if profile.broker_id != BrokerId::Toss {
        return None;
    }
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let (kr, us) = tokio::try_join!(
        adapter.get_kr_market_calendar(None),
        adapter.get_us_market_calendar(None),
    )
    .ok()?;
    match toss_calendar_override(&kr, &us) {
        Ok(calendar) => Some(calendar),
        Err(e) => {
            tracing::warn!(
                "Toss market-calendar 매핑 실패 — 기존 장 시간 fallback 사용: {}",
                e.message
            );
            None
        }
    }
}

#[tauri::command]
pub async fn get_toss_market_calendar(
    state: State<'_, AppState>,
) -> CmdResult<TossMarketCalendarView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    get_toss_market_calendar_for_profile(profile).await
}

pub async fn get_toss_market_snapshot_for_profile(
    symbol: String,
    profile: AccountProfile,
) -> Result<TossMarketSnapshotView, CmdError> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 시세 snapshot은 Toss 활성 프로파일에서만 조회할 수 있습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(symbol)?;

    let broker_symbol = BrokerSymbol(symbol.clone());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );

    let (prices, orderbook, trades, limits) = tokio::try_join!(
        adapter.list_prices(std::slice::from_ref(&broker_symbol)),
        adapter.get_orderbook(&broker_symbol),
        adapter.list_trades(&broker_symbol, Some(10)),
        adapter.get_price_limits(&broker_symbol),
    )
    .map_err(|e| CmdError {
        code: "TOSS_MARKET_DATA_ERROR".into(),
        message: e.to_string(),
    })?;

    let price = prices
        .into_iter()
        .find(|item| item.symbol.eq_ignore_ascii_case(&symbol))
        .ok_or_else(|| CmdError {
            code: "TOSS_PRICE_NOT_FOUND".into(),
            message: format!("Toss 현재가 응답에 요청 symbol이 없습니다: {symbol}"),
        })?;
    let quote = price.to_broker_price_quote().map_err(|e| CmdError {
        code: "TOSS_PRICE_MAPPING_ERROR".into(),
        message: e.to_string(),
    })?;

    Ok(TossMarketSnapshotView {
        broker_id: BrokerId::Toss,
        market: quote.market,
        symbol,
        timestamp: price.timestamp,
        price: quote.last.into(),
        orderbook: orderbook.into(),
        trades: trades.into_iter().map(TossTradeView::from).collect(),
        price_limits: limits.into(),
    })
}

#[tauri::command]
pub async fn get_toss_market_snapshot(
    symbol: String,
    state: State<'_, AppState>,
) -> CmdResult<TossMarketSnapshotView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    get_toss_market_snapshot_for_profile(symbol, profile).await
}

#[derive(Debug, Clone, Deserialize)]
pub struct TossOrderPreflightInput {
    pub symbol: String,
    pub side: String,
    pub quantity: String,
    pub price: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderPreflightView {
    pub broker_id: BrokerId,
    pub account_seq: String,
    pub symbol: String,
    pub market: BrokerMarket,
    pub side: BrokerOrderSide,
    pub quantity: String,
    pub price: BrokerMoneyView,
    pub price_source: String,
    pub buying_power: Option<BrokerMoneyView>,
    pub sellable_quantity: Option<String>,
    pub commission_rate: Option<String>,
    pub gross_amount: BrokerMoneyView,
    pub estimated_commission: Option<BrokerMoneyView>,
    pub required_cash: Option<BrokerMoneyView>,
    pub liquidity_ok: bool,
    pub safety_ok: bool,
    pub order_adapter_supported: bool,
    pub can_submit: bool,
    pub blocked_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

fn parse_toss_order_side(side: &str) -> CmdResult<BrokerOrderSide> {
    match side.trim().to_ascii_lowercase().as_str() {
        "buy" => Ok(BrokerOrderSide::Buy),
        "sell" => Ok(BrokerOrderSide::Sell),
        other => Err(CmdError {
            code: "INVALID_SIDE".into(),
            message: format!("알 수 없는 Toss 주문 방향: {other}"),
        }),
    }
}

fn toss_currency_from_view(money: &BrokerMoneyView) -> BrokerCurrency {
    money.currency
}

fn toss_market_country(market: BrokerMarket) -> &'static str {
    match market {
        BrokerMarket::Kr => "KR",
        BrokerMarket::Us => "US",
    }
}

fn select_toss_commission<'a>(
    commissions: &'a [TossCommission],
    market: BrokerMarket,
) -> Option<&'a TossCommission> {
    let country = toss_market_country(market);
    commissions
        .iter()
        .find(|commission| commission.market_country.eq_ignore_ascii_case(country))
        .or_else(|| commissions.first())
}

pub async fn check_toss_order_preflight_for_profile(
    input: TossOrderPreflightInput,
    profile: AccountProfile,
) -> CmdResult<TossOrderPreflightView> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 주문 전 검증은 Toss 활성 프로파일에서만 사용할 수 있습니다.".into(),
        });
    }

    let account_seq = profile.broker_account_id();
    if account_seq.trim().is_empty() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "토스증권 accountSeq가 설정되지 않았습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(input.symbol)?;
    let side = parse_toss_order_side(&input.side)?;
    let quantity = input.quantity.trim().replace(',', "");
    if parse_decimal_amount(&quantity).unwrap_or(0.0) <= 0.0 {
        return Err(CmdError {
            code: "INVALID_QUANTITY".into(),
            message: "Toss 주문 전 검증 수량은 0보다 커야 합니다.".into(),
        });
    }

    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(account_seq.clone()),
    );

    let snapshot = get_toss_market_snapshot_for_profile(symbol.clone(), profile.clone()).await?;
    let safety = get_toss_stock_safety_for_profile(symbol.clone(), profile).await?;
    let currency = toss_currency_from_view(&snapshot.price);
    let input_price = input.price.as_deref().and_then(parse_decimal_amount);
    let snapshot_price = parse_decimal_amount(&snapshot.price.amount).unwrap_or(0.0);
    let (price_amount, price_source) = match input_price.filter(|value| *value > 0.0) {
        Some(value) => (format_money_amount(value, currency), "input".to_string()),
        None => (
            format_money_amount(snapshot_price, currency),
            "snapshot".to_string(),
        ),
    };
    let price = BrokerMoney {
        amount: price_amount,
        currency,
    };

    let commissions = adapter
        .list_commissions(Some(&account_seq))
        .await
        .map_err(|e| CmdError {
            code: "TOSS_PREFLIGHT_COMMISSIONS_ERROR".into(),
            message: e.to_string(),
        })?;
    let commission_rate = select_toss_commission(&commissions, snapshot.market)
        .map(|commission| commission.commission_rate.clone());

    let (buying_power, sellable_quantity) = match side {
        BrokerOrderSide::Buy => {
            let power = adapter
                .get_buying_power(Some(&account_seq), currency)
                .await
                .map_err(|e| CmdError {
                    code: "TOSS_PREFLIGHT_BUYING_POWER_ERROR".into(),
                    message: e.to_string(),
                })?
                .money()
                .map_err(|e| CmdError {
                    code: "TOSS_PREFLIGHT_BUYING_POWER_MAPPING_ERROR".into(),
                    message: e.to_string(),
                })?;
            (Some(power), None)
        }
        BrokerOrderSide::Sell => {
            let qty = adapter
                .get_sellable_quantity(Some(&account_seq), &BrokerSymbol(symbol.clone()))
                .await
                .map_err(|e| CmdError {
                    code: "TOSS_PREFLIGHT_SELLABLE_ERROR".into(),
                    message: e.to_string(),
                })?
                .quantity();
            (None, Some(qty))
        }
    };

    let decision = evaluate_order_preflight(
        &OrderPreflightInput {
            side,
            quantity: BrokerQuantity(quantity.clone()),
            price: price.clone(),
        },
        &OrderPreflightConstraints {
            buying_power: buying_power.clone(),
            sellable_quantity: sellable_quantity.clone(),
            commission_rate_percent: commission_rate.clone(),
        },
    );

    let mut blocked_reasons = decision.blocked_reasons;
    if let Some(reason) = safety.buy_block_reason.as_ref() {
        if side == BrokerOrderSide::Buy {
            blocked_reasons.push(reason.clone());
        }
    }

    let mut warnings = Vec::new();
    if commission_rate.is_none() {
        warnings
            .push("시장과 일치하는 Toss 수수료 정책을 찾지 못해 수수료 0으로 추정했습니다.".into());
    }
    warnings.push("Toss 주문 생성 adapter는 아직 소액 검증 gate 전이라 제출이 차단됩니다.".into());

    let safety_ok = !(side == BrokerOrderSide::Buy && safety.buy_blocked);
    let order_adapter_supported = false;
    let liquidity_ok = decision.liquidity_ok;
    let can_submit = liquidity_ok && safety_ok && order_adapter_supported;

    Ok(TossOrderPreflightView {
        broker_id: BrokerId::Toss,
        account_seq,
        symbol,
        market: snapshot.market,
        side,
        quantity,
        price: price.into(),
        price_source,
        buying_power: buying_power.map(Into::into),
        sellable_quantity: sellable_quantity.map(|quantity| quantity.0),
        commission_rate,
        gross_amount: decision.gross_amount.into(),
        estimated_commission: decision.estimated_commission.map(Into::into),
        required_cash: decision.required_cash.map(Into::into),
        liquidity_ok,
        safety_ok,
        order_adapter_supported,
        can_submit,
        blocked_reasons,
        warnings,
    })
}

#[tauri::command]
pub async fn check_toss_order_preflight(
    input: TossOrderPreflightInput,
    state: State<'_, AppState>,
) -> CmdResult<TossOrderPreflightView> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    check_toss_order_preflight_for_profile(input, profile).await
}

pub async fn get_toss_chart_data_for_profile(
    symbol: String,
    interval: String,
    count: Option<u16>,
    profile: AccountProfile,
) -> CmdResult<Vec<ChartCandle>> {
    if profile.broker_id != BrokerId::Toss {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss 캔들 조회는 Toss 활성 프로파일에서만 사용할 수 있습니다.".into(),
        });
    }

    let symbol = normalize_toss_symbol(symbol)?;

    let interval = match interval.as_str() {
        "1m" | "M1" | "m" => "1m",
        "1d" | "D" | "d" => "1d",
        other => {
            return Err(CmdError {
                code: "INVALID_INTERVAL".into(),
                message: format!("Toss candles interval은 1m 또는 1d만 지원합니다: {other}"),
            });
        }
    };
    let count = count.unwrap_or(200);
    if !(1..=200).contains(&count) {
        return Err(CmdError {
            code: "INVALID_COUNT".into(),
            message: format!("Toss candles count는 1~200 범위여야 합니다: {count}"),
        });
    }

    let symbol = BrokerSymbol(symbol);
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let candles = adapter
        .get_candles(&symbol, interval, "", "")
        .await
        .map_err(|e| CmdError {
            code: "TOSS_CANDLES_ERROR".into(),
            message: e.to_string(),
        })?;

    let mut chart: Vec<ChartCandle> = candles
        .into_iter()
        .take(count as usize)
        .map(|candle| ChartCandle {
            date: normalize_toss_chart_time(&candle.date, interval),
            open: candle.open.amount,
            high: candle.high.amount,
            low: candle.low.amount,
            close: candle.close.amount,
            volume: candle.volume.0,
        })
        .collect();
    chart.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(chart)
}

fn normalize_toss_chart_time(value: &str, interval: &str) -> String {
    if interval == "1d" {
        return value
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(8)
            .collect();
    }
    value.to_string()
}

#[tauri::command]
pub async fn get_toss_chart_data(
    symbol: String,
    interval: String,
    count: Option<u16>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<ChartCandle>> {
    let profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    }
    .ok_or_else(|| CmdError {
        code: "CONFIG_NOT_READY".into(),
        message: "활성 프로파일이 없습니다.".into(),
    })?;

    get_toss_chart_data_for_profile(symbol, interval, count, profile).await
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
        .get_chart_data(
            &input.symbol,
            &input.period_code,
            &input.start_date,
            &input.end_date,
        )
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
        state
            .stock_store
            .upsert(&symbol, &result.hts_kor_isnm)
            .await;
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
        other => {
            return Err(CmdError {
                code: "INVALID_SIDE".into(),
                message: format!("알 수 없는 주문 방향: {}", other),
            })
        }
    };

    let order_type = match input.order_type.as_str() {
        "limit" | "Limit" => OrderType::Limit,
        "market" | "Market" => OrderType::Market,
        other => {
            return Err(CmdError {
                code: "INVALID_ORDER_TYPE".into(),
                message: format!("알 수 없는 주문 유형: {}", other),
            })
        }
    };

    let req = OrderRequest {
        symbol: input.symbol,
        side,
        order_type,
        quantity: input.quantity,
        price: input.price,
    };
    let client = state.rest_client.read().await.clone();
    client.place_order(&req).await.map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 당일 체결 내역 (KIS 실시간)
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_executed(state: State<'_, AppState>) -> CmdResult<Vec<ExecutedOrder>> {
    let client = state.rest_client.read().await.clone();
    client
        .get_today_executed_orders()
        .await
        .map_err(CmdError::from)
}

#[tauri::command]
pub async fn get_today_overseas_executed(
    state: State<'_, AppState>,
) -> CmdResult<Vec<OverseasExecutedOrder>> {
    let client = state.rest_client.read().await.clone();
    client
        .get_today_overseas_executed_orders()
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 로컬 체결 기록 (JSON 저장소)
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_trades(state: State<'_, AppState>) -> CmdResult<Vec<TradeRecord>> {
    let today = chrono::Local::now().date_naive();
    state
        .trade_store
        .get_by_date(today)
        .await
        .map_err(CmdError::from)
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
    state
        .trade_store
        .get_by_range(from, to)
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 일별 통계
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_today_stats(state: State<'_, AppState>) -> CmdResult<DailyStats> {
    let today = chrono::Local::now().date_naive();
    state
        .stats_store
        .get_by_date(today)
        .await
        .map_err(CmdError::from)
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
    state
        .stats_store
        .get_by_range(from, to)
        .await
        .map_err(CmdError::from)
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
        other => {
            return Err(CmdError {
                code: "INVALID_SIDE".into(),
                message: format!("알 수 없는 방향: {}", other),
            })
        }
    };

    let record = TradeRecord::new(
        input.symbol,
        input.symbol_name,
        side.clone(),
        input.quantity,
        input.price,
        input.fee,
        input.order_id,
        input.strategy_id,
        String::new(), // 수동 저장 시 signal_reason 없음
    );

    state
        .trade_store
        .append(record.clone())
        .await
        .map_err(CmdError::from)?;

    if let Some(notifier) = &state.discord {
        let side_label = if side == TradeSide::Buy {
            "매수"
        } else {
            "매도"
        };
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
pub async fn upsert_daily_stats(stats: DailyStats, state: State<'_, AppState>) -> CmdResult<()> {
    state
        .stats_store
        .upsert(stats)
        .await
        .map_err(CmdError::from)
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
    /// 자동매매가 실행 중인 broker ID (미실행 시 None)
    pub trading_broker_id: Option<BrokerId>,
    /// 자동매매가 실행 중인 broker account ID (미실행 시 None)
    pub trading_account_id: Option<String>,
    /// 잔고 부족으로 매수 정지 여부
    pub buy_suspended: bool,
    /// 매수 정지 사유 (KIS 응답 msg1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_suspended_reason: Option<String>,
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
    let trading_broker_id = state.trading_broker_id.read().await.clone();
    let trading_account_id = state.trading_account_id.read().await.clone();
    let (buy_suspended, buy_suspended_reason) = {
        let om = state.order_manager.lock().await;
        (om.buy_suspended, om.buy_suspended_reason.clone())
    };
    Ok(TradingStatus {
        is_running,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended,
        buy_suspended_reason,
    })
}

async fn sync_strategy_positions_from_active_broker(
    state: &AppState,
    current_cfg: &AppConfig,
) -> usize {
    let active_profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    match active_profile
        .as_ref()
        .map(|profile| profile.broker_id)
        .unwrap_or(current_cfg.broker_id)
    {
        BrokerId::Kis => sync_kis_strategy_positions(state).await,
        BrokerId::Toss => match active_profile {
            Some(profile) => sync_toss_strategy_positions(state, profile).await,
            None => {
                tracing::warn!("Toss holdings 동기화 건너뜀: 활성 Toss 프로파일이 없습니다.");
                0
            }
        },
    }
}

async fn sync_kis_strategy_positions(state: &AppState) -> usize {
    let rest = state.rest_client.read().await.clone();
    let mut synced = 0usize;
    match rest.get_balance().await {
        Ok(resp) => {
            state
                .stock_store
                .upsert_many(
                    resp.items
                        .iter()
                        .map(|i| (i.pdno.clone(), i.prdt_name.clone())),
                )
                .await;
            {
                let mut tracker = state.position_tracker.lock().await;
                tracker.load_if_empty(resp.items.iter().map(|i| {
                    (
                        i.pdno.clone(),
                        i.prdt_name.clone(),
                        i.hldg_qty.parse::<u64>().unwrap_or(0),
                        i.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64,
                        i.prpr.parse::<u64>().unwrap_or(0),
                    )
                }));
            }
            {
                let mut mgr = state.strategy_manager.lock().await;
                for item in &resp.items {
                    let qty = item.hldg_qty.parse::<u64>().unwrap_or(0);
                    let avg = item.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64;
                    if qty > 0 {
                        synced += 1;
                    }
                    mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                        broker_id: BrokerId::Kis,
                        market: BrokerMarket::Kr,
                        symbol: item.pdno.clone(),
                        quantity: qty,
                        avg_price: avg,
                    });
                }
            }
        }
        Err(e) => tracing::warn!("자동매매 시작 전 국내 잔고 동기화 실패: {}", e),
    }

    match rest.get_overseas_balance().await {
        Ok(resp) => {
            {
                let mut tracker = state.overseas_position_tracker.lock().await;
                tracker.load_if_empty(resp.items.iter().map(|i| {
                    (
                        i.ovrs_pdno.clone(),
                        i.ovrs_item_name.clone(),
                        normalize_overseas_order_exchange(&i.ovrs_excg_cd),
                        i.ovrs_cblc_qty.parse::<u64>().unwrap_or(0),
                        usd_to_cents(&i.pchs_avg_pric),
                        usd_to_cents(&i.now_pric2),
                    )
                }));
            }
            let mut mgr = state.strategy_manager.lock().await;
            for item in &resp.items {
                let qty = item.ovrs_cblc_qty.parse::<u64>().unwrap_or(0);
                let avg = usd_to_cents(&item.pchs_avg_pric);
                if qty > 0 {
                    synced += 1;
                }
                mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                    broker_id: BrokerId::Kis,
                    market: BrokerMarket::Us,
                    symbol: item.ovrs_pdno.clone(),
                    quantity: qty,
                    avg_price: avg,
                });
            }
        }
        Err(e) => tracing::warn!("자동매매 시작 전 해외 잔고 동기화 실패: {}", e),
    }

    synced
}

async fn sync_toss_strategy_positions(state: &AppState, profile: AccountProfile) -> usize {
    if !profile.is_configured() {
        tracing::warn!("Toss holdings 동기화 건너뜀: 활성 Toss 프로파일 설정이 미완료입니다.");
        return 0;
    }

    let account_id = BrokerAccountId(profile.broker_account_id());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let holdings = match adapter.list_holdings(Some(&account_id)).await {
        Ok(holdings) => holdings,
        Err(e) => {
            tracing::warn!("자동매매 시작 전 Toss holdings 동기화 실패: {}", e);
            return 0;
        }
    };

    state
        .stock_store
        .upsert_many(
            holdings
                .iter()
                .map(|holding| (holding.symbol.0.clone(), holding.symbol_name.clone())),
        )
        .await;

    {
        let mut domestic_tracker = state.position_tracker.lock().await;
        domestic_tracker.load_if_empty(holdings.iter().filter_map(|holding| {
            (holding.market == BrokerMarket::Kr).then(|| {
                (
                    holding.symbol.0.clone(),
                    holding.symbol_name.clone(),
                    decimal_quantity_to_position_units(&holding.quantity.0),
                    broker_money_to_strategy_price_units(&holding.average_price),
                    broker_money_to_strategy_price_units(&holding.current_price),
                )
            })
        }));
    }
    {
        let mut overseas_tracker = state.overseas_position_tracker.lock().await;
        overseas_tracker.load_if_empty(holdings.iter().filter_map(|holding| {
            (holding.market == BrokerMarket::Us).then(|| {
                (
                    holding.symbol.0.clone(),
                    holding.symbol_name.clone(),
                    "TOSS_US".to_string(),
                    decimal_quantity_to_position_units(&holding.quantity.0),
                    broker_money_to_strategy_price_units(&holding.average_price),
                    broker_money_to_strategy_price_units(&holding.current_price),
                )
            })
        }));
    }

    let mut synced = 0usize;
    let mut mgr = state.strategy_manager.lock().await;
    for holding in &holdings {
        let quantity = decimal_quantity_to_position_units(&holding.quantity.0);
        if quantity > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Toss,
            market: holding.market,
            symbol: holding.symbol.0.clone(),
            quantity,
            avg_price: broker_money_to_strategy_price_units(&holding.average_price),
        });
    }
    tracing::info!(
        "Toss holdings 기반 전략 포지션 동기화 완료: {}개 보유 종목",
        synced
    );
    synced
}

#[tauri::command]
pub async fn start_trading(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<TradingStatus> {
    let current_cfg = state.config.read().await.clone();
    if current_cfg.broker_id == BrokerId::Kis && !current_cfg.is_kis_configured() {
        return Err(CmdError {
            code: "CONFIG_NOT_READY".into(),
            message: "KIS API 설정이 완료되지 않았습니다. Settings에서 API 키를 확인하세요.".into(),
        });
    }

    if *state.is_trading.lock().await {
        return Err(CmdError {
            code: "ALREADY_RUNNING".into(),
            message: "자동 매매가 이미 실행 중입니다.".into(),
        });
    }

    // 자동매매 시작 전 실제 잔고를 전략 내부 포지션 상태와 동기화한다.
    // 재시작 직후 내부 in_position=false 상태로 같은 종목을 재매수하는 위험을 줄인다.
    let synced_positions = sync_strategy_positions_from_active_broker(&state, &current_cfg).await;
    tracing::info!(
        "자동매매 시작 전 broker-aware 포지션 동기화 완료: {}개",
        synced_positions
    );

    if current_cfg.broker_id != BrokerId::Kis {
        return Err(CmdError {
            code: "BROKER_NOT_SUPPORTED".into(),
            message: "Toss holdings 기반 전략 포지션 동기화는 완료했지만, 현재 자동매매 주문 실행 경로는 KIS broker만 지원합니다. Toss 주문/체결 adapter와 소액 검증 gate 이후 연결하세요."
                .into(),
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
        let (active_id, broker_id, account_id) = {
            let profiles = state.profiles.read().await;
            match profiles.get_active() {
                Some(profile) => (
                    Some(profile.id.clone()),
                    Some(profile.broker_id),
                    Some(profile.broker_account_id()),
                ),
                None => (
                    profiles.active_id.clone(),
                    Some(current_cfg.broker_id),
                    Some(current_cfg.broker_account_id.clone())
                        .filter(|account_id| !account_id.is_empty()),
                ),
            }
        };
        let execution_scope = BrokerScope::new(
            broker_id.unwrap_or(BrokerId::Kis),
            account_id.clone().map(BrokerAccountId),
        );
        state
            .order_manager
            .lock()
            .await
            .set_execution_scope(execution_scope);
        *state.trading_profile_id.write().await = active_id;
        *state.trading_broker_id.write().await = broker_id;
        *state.trading_account_id.write().await = account_id;
    }

    if let Some(notifier) = &state.discord {
        let _ = notifier
            .send(NotificationEvent::info(
                "자동 매매 시작".to_string(),
                "AutoConditionTrade 자동 매매가 시작되었습니다.".to_string(),
            ))
            .await;
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
            let start_date = (today - chrono::Duration::days(400))
                .format("%Y%m%d")
                .to_string();

            for symbol in &active_symbols {
                if is_domestic_symbol(symbol) {
                    // ── 국내 종목 초기화 ──
                    match rest
                        .get_chart_data(symbol, "D", &start_date, &end_date)
                        .await
                    {
                        Ok(candles) if !candles.is_empty() => {
                            let highs: Vec<u64> = candles
                                .iter()
                                .filter_map(|c| c.high.parse::<u64>().ok())
                                .collect();
                            if !highs.is_empty() {
                                state
                                    .strategy_manager
                                    .lock()
                                    .await
                                    .initialize_historical(symbol, &highs);
                                tracing::info!(
                                    "전략 히스토리 초기화 완료: {} ({}봉)",
                                    symbol,
                                    highs.len()
                                );
                            }
                            let high_close: Vec<(u64, u64)> = candles
                                .iter()
                                .filter_map(|c| {
                                    let h = c.high.parse::<u64>().ok()?;
                                    let cl = c.close.parse::<u64>().ok()?;
                                    Some((h, cl))
                                })
                                .collect();
                            if !high_close.is_empty() {
                                state
                                    .strategy_manager
                                    .lock()
                                    .await
                                    .initialize_candles(symbol, &high_close);
                            }
                            let ohlc: Vec<OhlcCandle> = candles
                                .iter()
                                .filter_map(|c| {
                                    Some(OhlcCandle {
                                        open: c.open.parse::<u64>().ok()?,
                                        high: c.high.parse::<u64>().ok()?,
                                        low: c.low.parse::<u64>().ok()?,
                                        close: c.close.parse::<u64>().ok()?,
                                    })
                                })
                                .collect();
                            if !ohlc.is_empty() {
                                state
                                    .strategy_manager
                                    .lock()
                                    .await
                                    .initialize_ohlc(symbol, &ohlc);
                                if let Some(atr) = calculate_atr(&ohlc, 14) {
                                    state.risk_manager.lock().await.set_symbol_atr(symbol, atr);
                                    tracing::info!(
                                        "리스크 ATR 초기화 완료: {} ATR14={}",
                                        symbol,
                                        atr
                                    );
                                }
                            }
                            let ranges: Vec<u64> = candles
                                .iter()
                                .filter_map(|c| {
                                    let h = c.high.parse::<u64>().ok()?;
                                    let l = c.low.parse::<u64>().ok()?;
                                    Some(h.saturating_sub(l))
                                })
                                .collect();
                            if !ranges.is_empty() {
                                state
                                    .strategy_manager
                                    .lock()
                                    .await
                                    .initialize_range_data(symbol, &ranges);
                            }
                        }
                        Ok(_) => {
                            tracing::debug!("차트 데이터 없음 (히스토리 초기화 건너뜀): {}", symbol)
                        }
                        Err(e) => tracing::warn!(
                            "차트 데이터 조회 실패 (히스토리 초기화 건너뜀): {} — {}",
                            symbol,
                            e
                        ),
                    }
                } else {
                    // ── 해외 종목 초기화 (NAS → NYS → AMS 순 시도) ──
                    let mut initialized = false;
                    for exchange in &["NAS", "NYS", "AMS"] {
                        match rest
                            .get_overseas_chart_data(symbol, exchange, "D", &end_date)
                            .await
                        {
                            Ok(candles) if !candles.is_empty() => {
                                // USD float 문자열 → ×100 센트(u64)로 변환하여 전략 히스토리 초기화
                                let highs: Vec<u64> = candles
                                    .iter()
                                    .filter_map(|c| {
                                        c.high
                                            .parse::<f64>()
                                            .ok()
                                            .map(|v| (v * 100.0).round() as u64)
                                    })
                                    .filter(|&v| v > 0)
                                    .collect();
                                if !highs.is_empty() {
                                    state
                                        .strategy_manager
                                        .lock()
                                        .await
                                        .initialize_historical(symbol, &highs);
                                    tracing::info!(
                                        "해외 전략 히스토리 초기화: {} @ {} ({}봉, 센트 단위)",
                                        symbol,
                                        exchange,
                                        highs.len()
                                    );
                                }
                                let high_close: Vec<(u64, u64)> = candles
                                    .iter()
                                    .filter_map(|c| {
                                        let h = c
                                            .high
                                            .parse::<f64>()
                                            .ok()
                                            .map(|v| (v * 100.0).round() as u64)?;
                                        let cl = c
                                            .close
                                            .parse::<f64>()
                                            .ok()
                                            .map(|v| (v * 100.0).round() as u64)?;
                                        if h > 0 && cl > 0 {
                                            Some((h, cl))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                if !high_close.is_empty() {
                                    state
                                        .strategy_manager
                                        .lock()
                                        .await
                                        .initialize_candles(symbol, &high_close);
                                }
                                let ohlc: Vec<OhlcCandle> = candles
                                    .iter()
                                    .filter_map(|c| {
                                        Some(OhlcCandle {
                                            open: c
                                                .open
                                                .parse::<f64>()
                                                .ok()
                                                .map(|v| (v * 100.0).round() as u64)?,
                                            high: c
                                                .high
                                                .parse::<f64>()
                                                .ok()
                                                .map(|v| (v * 100.0).round() as u64)?,
                                            low: c
                                                .low
                                                .parse::<f64>()
                                                .ok()
                                                .map(|v| (v * 100.0).round() as u64)?,
                                            close: c
                                                .close
                                                .parse::<f64>()
                                                .ok()
                                                .map(|v| (v * 100.0).round() as u64)?,
                                        })
                                    })
                                    .filter(|c| {
                                        c.open > 0 && c.high > 0 && c.low > 0 && c.close > 0
                                    })
                                    .collect();
                                if !ohlc.is_empty() {
                                    state
                                        .strategy_manager
                                        .lock()
                                        .await
                                        .initialize_ohlc(symbol, &ohlc);
                                    if let Some(atr) = calculate_atr(&ohlc, 14) {
                                        state.risk_manager.lock().await.set_symbol_atr(symbol, atr);
                                        tracing::info!(
                                            "해외 리스크 ATR 초기화: {} @ {} ATR14={} cents",
                                            symbol,
                                            exchange,
                                            atr
                                        );
                                    }
                                }
                                let ranges: Vec<u64> = candles
                                    .iter()
                                    .filter_map(|c| {
                                        let h = c.high.parse::<f64>().ok()?;
                                        let l = c.low.parse::<f64>().ok()?;
                                        let diff = ((h - l) * 100.0).round() as u64;
                                        if diff > 0 {
                                            Some(diff)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                if !ranges.is_empty() {
                                    state
                                        .strategy_manager
                                        .lock()
                                        .await
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
        let symbols: Vec<String> = state.strategy_manager.lock().await.active_symbols();

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
    let trading_broker_id = state.trading_broker_id.read().await.clone();
    let trading_account_id = state.trading_account_id.read().await.clone();
    let (buy_suspended, buy_suspended_reason) = {
        let om = state.order_manager.lock().await;
        (om.buy_suspended, om.buy_suspended_reason.clone())
    };
    Ok(TradingStatus {
        is_running: true,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended,
        buy_suspended_reason,
    })
}

#[tauri::command]
pub async fn stop_trading(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    let mut is_running = state.is_trading.lock().await;
    *is_running = false;
    tracing::info!("자동 매매 정지");

    // 자동매매 종료 시 트레이딩 프로파일 ID 클리어
    *state.trading_profile_id.write().await = None;
    *state.trading_broker_id.write().await = None;
    *state.trading_account_id.write().await = None;

    if let Some(notifier) = &state.discord {
        let _ = notifier
            .send(NotificationEvent::info(
                "자동 매매 정지".to_string(),
                "AutoConditionTrade 자동 매매가 정지되었습니다.".to_string(),
            ))
            .await;
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
        trading_broker_id: None,
        trading_account_id: None,
        buy_suspended: false,
        buy_suspended_reason: None,
    })
}

/// 잔고 부족 매수 정지를 수동으로 해제합니다.
/// 계좌에 자금을 입금한 경우 또는 오판 시 사용.
#[tauri::command]
pub async fn clear_buy_suspension(state: State<'_, AppState>) -> CmdResult<TradingStatus> {
    state.order_manager.lock().await.clear_buy_suspension();

    let is_running = *state.is_trading.lock().await;
    let strategies = state.strategy_manager.lock().await.active_names();
    let (position_count, total_pnl) = {
        let tracker = state.position_tracker.lock().await;
        (tracker.count(), tracker.total_pnl())
    };
    let ws_connected = state.ws_connected.load(Ordering::Relaxed);
    let trading_profile_id = state.trading_profile_id.read().await.clone();
    let trading_broker_id = state.trading_broker_id.read().await.clone();
    let trading_account_id = state.trading_account_id.read().await.clone();
    Ok(TradingStatus {
        is_running,
        active_strategies: strategies,
        position_count,
        total_unrealized_pnl: total_pnl,
        ws_connected,
        trading_profile_id,
        trading_broker_id,
        trading_account_id,
        buy_suspended: false,
        buy_suspended_reason: None,
    })
}

// ────────────────────────────────────────────────────────────────────
// 자동매매 폴링 데몬 (lib.rs 에서 앱 시작 시 영구 spawn)
//
// is_trading 플래그가 false 이면 5초마다 재확인하며 대기.
// true 로 바뀌면 즉시 폴링 재개. start_trading / web API 모두 이 방식으로 제어.
//
// 설계 원칙:
//   - 레이블 루프(labeled loop)·goto 유사 패턴 사용 금지
//   - 제어 흐름은 항상 위→아래 순차 실행
//   - 내부 루프에서 외부 루프로 점프하는 break 'label / continue 'label 금지
//   - 내부 루프 조기 탈출이 필요한 경우 별도 함수로 추출 후 return 사용
// ────────────────────────────────────────────────────────────────────

/// 종목 폴링 한 사이클(틱)의 처리 결과
///
/// `poll_symbols_tick` 반환값으로 사용. 호출자가 결과에 따라
/// 시장 대기 여부를 결정한다.
#[derive(Debug, PartialEq)]
enum TickCycleResult {
    /// 모든 종목 정상 처리 완료
    Done,
    /// 장 마감 / 장외 시간 감지 → 호출자는 market_pause_until 설정 후 다음 이터레이션으로
    MarketClosed,
    /// is_trading 플래그가 false 로 바뀜 → 이번 사이클 조기 종료
    Stopped,
}

/// 종목 목록을 순차적으로 순회하여 현재가 조회 + 전략 신호 처리
///
/// `break 'label` / `continue 'label` 없이 `return`으로 조기 탈출한다.
/// 장 마감 감지 시 `TickCycleResult::MarketClosed` 를 반환하고,
/// 호출자(run_trading_daemon)가 market_pause_until 을 설정한다.
async fn poll_symbols_tick(
    symbols: &[String],
    is_trading: &Arc<Mutex<bool>>,
    strategy_mgr: &Arc<Mutex<crate::trading::strategy::StrategyManager>>,
    order_mgr: &Arc<Mutex<crate::trading::order::OrderManager>>,
    stock_store: &Arc<crate::storage::stock_store::StockStore>,
    rest: &Arc<KisRestClient>,
    delay_ms: u64,
    total_balance_krw: i64,
    fills_pending: &mut Vec<(String, u64)>,
    market_calendar: Option<&MarketCalendarOverride>,
) -> TickCycleResult {
    for symbol in symbols {
        // 종료 플래그 선행 확인
        if !*is_trading.lock().await {
            return TickCycleResult::Stopped;
        }

        // 해당 종목 시장 개장 여부 확인 (폐장이면 건너뜀)
        if !is_market_open_for_with_calendar(symbol, market_calendar) {
            tracing::debug!(
                "시장 폐장 — 건너뜀: {} ({})",
                symbol,
                if is_domestic_symbol(symbol) {
                    "KRX"
                } else {
                    "US"
                }
            );
            continue;
        }

        // 현재가 조회 + 해외 주문용 거래소 코드 캡처
        let (tick, exchange_opt): (Result<(u64, u64), String>, Option<String>) =
            if is_domestic_symbol(symbol) {
                let t = rest
                    .get_price(symbol)
                    .await
                    .map(|p| {
                        let price = p.stck_prpr.parse::<u64>().unwrap_or(0);
                        let volume = p.acml_vol.parse::<u64>().unwrap_or(0);
                        (price, volume)
                    })
                    .map_err(|e| e.to_string());
                (t, None)
            } else {
                match fetch_overseas_tick(rest, symbol).await {
                    Ok((price, volume, exch)) => (Ok((price, volume)), Some(exch)),
                    Err(e) => (Err(e.to_string()), None),
                }
            };

        // 틱 처리: 현재가 기반 전략 신호 생성 → 주문
        match tick {
            Ok((price, volume)) if price > 0 => {
                let signals = strategy_mgr.lock().await.on_tick(symbol, price, volume);
                for strategy_signal in signals {
                    let symbol_name = stock_store
                        .get_name(symbol)
                        .await
                        .unwrap_or_else(|| symbol.clone());
                    let submit_result = order_mgr
                        .lock()
                        .await
                        .submit_signal(
                            Some(strategy_signal.strategy_id),
                            strategy_signal.signal,
                            &symbol_name,
                            total_balance_krw,
                            exchange_opt.clone(),
                            price,
                        )
                        .await;
                    match submit_result {
                        Ok(()) => {
                            fills_pending.push((symbol.clone(), price));
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            if is_market_closed_error(&msg) {
                                tracing::info!(
                                    "장 마감/장외 시간 감지 (주문, {}) — 5분 대기: {}",
                                    symbol,
                                    msg
                                );
                                return TickCycleResult::MarketClosed;
                            }
                            tracing::warn!("신호 처리 실패 ({}): {}", symbol, msg);
                        }
                    }
                }
            }
            Ok(_) => {
                tracing::debug!("현재가 0 — 건너뜀: {}", symbol);
            }
            Err(e) => {
                if is_market_closed_error(&e) {
                    tracing::info!(
                        "장 마감/장외 시간 감지 (현재가, {}) — 5분 대기: {}",
                        symbol,
                        e
                    );
                    return TickCycleResult::MarketClosed;
                }
                tracing::warn!("현재가 조회 실패 ({}): {}", symbol, e);
            }
        }

        // 종목 간 API 호출 딜레이
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        // 딜레이 후 종료 플래그 재확인
        if !*is_trading.lock().await {
            return TickCycleResult::Stopped;
        }
    }
    TickCycleResult::Done
}

pub async fn run_trading_daemon(
    is_trading: Arc<Mutex<bool>>,
    strategy_mgr: Arc<Mutex<crate::trading::strategy::StrategyManager>>,
    order_mgr: Arc<Mutex<crate::trading::order::OrderManager>>,
    risk_mgr: Arc<Mutex<crate::trading::risk::RiskManager>>,
    rest_arc: Arc<RwLock<Arc<KisRestClient>>>,
    stock_store: Arc<crate::storage::stock_store::StockStore>,
    profiles: Arc<RwLock<ProfilesConfig>>,
) {
    tracing::info!("자동매매 폴링 데몬 시작 (is_trading=false 대기 중)");
    let mut last_reset_date = chrono::Local::now().date_naive();
    let mut market_pause_until: Option<tokio::time::Instant> = None;
    let mut fills_pending: Vec<(String, u64)> = Vec::new();
    let mut was_running = false;

    // 레이블 없는 단순 루프 — 제어 흐름은 위→아래 순차 실행
    loop {
        let is_running = *is_trading.lock().await;

        // ── Phase 1: 자동매매 비활성 → 5초 대기 후 재확인 ──────────
        if !is_running {
            if was_running {
                fills_pending.clear();
                market_pause_until = None;
                tracing::info!("자동매매 폴링 데몬 일시 정지 (is_trading=false)");
            }
            was_running = false;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        // ── Phase 2: 방금 활성화됨 → 로컬 상태 초기화 ──────────────
        if !was_running {
            was_running = true;
            fills_pending.clear();
            market_pause_until = None;
            last_reset_date = chrono::Local::now().date_naive();
            tracing::info!("자동매매 폴링 데몬 활성화");
        }

        // ── Phase 3: 장 마감으로 대기 중 → 30초 슬립 후 재확인 ─────
        if let Some(pause_until) = market_pause_until {
            if tokio::time::Instant::now() < pause_until {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                continue;
            }
            tracing::info!("장 마감 대기 완료 — 폴링 재개");
            market_pause_until = None;
        }

        // ── Phase 4: 이전 틱 시장가 주문 자동 체결 확인 ────────────
        if !fills_pending.is_empty() {
            if let Err(e) = order_mgr
                .lock()
                .await
                .confirm_pending_fills_from_broker()
                .await
            {
                tracing::debug!(
                    "주문번호 기반 체결 확인 실패 — 다음 틱 가격 확인으로 보완: {}",
                    e
                );
            }
            let fills = std::mem::take(&mut fills_pending);
            for (sym, fill_price) in fills {
                if let Err(e) = order_mgr
                    .lock()
                    .await
                    .confirm_fill_by_symbol(&sym, fill_price)
                    .await
                {
                    tracing::warn!("자동 체결 확인 실패 ({}): {}", sym, e);
                }
            }
        }

        // ── Phase 5: 날짜 변경 시 일별 초기화 ──────────────────────
        let today = chrono::Local::now().date_naive();
        if today != last_reset_date {
            last_reset_date = today;
            risk_mgr.lock().await.reset_if_new_day();
            order_mgr.lock().await.reset_day();
            tracing::info!("자동매매 일별 초기화 완료 ({})", today);
        }

        // ── Phase 6: 활성 전략의 종목 수집 ─────────────────────────
        let symbols: Vec<String> = strategy_mgr.lock().await.active_symbols();
        if symbols.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            continue;
        }

        let rest = rest_arc.read().await.clone();
        let delay_ms: u64 = if rest.is_paper() { 700 } else { 150 };

        // ── Phase 7: 전체 시장 폐장 여부 사전 체크 ─────────────────
        let market_calendar = get_active_toss_calendar_override(&profiles).await;

        if symbols
            .iter()
            .all(|s| !is_market_open_for_with_calendar(s, market_calendar.as_ref()))
        {
            tracing::info!(
                "모든 시장 폐장 ({}) — 5분 대기 후 재확인",
                open_markets_summary_with_calendar(market_calendar.as_ref())
            );
            market_pause_until =
                Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(300));
            continue;
        }
        tracing::debug!(
            "시장 상태: {}",
            open_markets_summary_with_calendar(market_calendar.as_ref())
        );

        let exchange_rate = order_mgr.lock().await.current_exchange_rate_krw().await;
        let total_balance_krw = fetch_account_risk_balance_krw(
            &rest,
            symbols.iter().any(|s| !is_domestic_symbol(s)),
            exchange_rate,
        )
        .await;
        if total_balance_krw <= 0 {
            tracing::warn!(
                "리스크 총잔고가 0원으로 조회되어 ATR 수량 산정과 포지션 비중 검사를 건너뜁니다."
            );
        }

        // ── Phase 8: 종목별 현재가 조회 + 전략 신호 처리 ───────────
        //   내부 루프는 poll_symbols_tick() 으로 분리 — goto 유사 패턴 없음
        let tick_result = poll_symbols_tick(
            &symbols,
            &is_trading,
            &strategy_mgr,
            &order_mgr,
            &stock_store,
            &rest,
            delay_ms,
            total_balance_krw,
            &mut fills_pending,
            market_calendar.as_ref(),
        )
        .await;

        if tick_result == TickCycleResult::MarketClosed {
            market_pause_until =
                Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(300));
            continue;
        }

        // ── Phase 9: 다음 틱까지 10초 대기 (100ms 단위 — 종료 신호 즉시 반응) ──
        for _ in 0u32..100 {
            if !*is_trading.lock().await {
                break;
            }
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
    let mut positions: Vec<PositionView> = tracker
        .all()
        .iter()
        .map(|p| PositionView::from(*p))
        .collect();
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
    pub broker_id: BrokerId,
    pub broker_account_id: Option<String>,
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
            let name = state
                .stock_store
                .get_name(code)
                .await
                .unwrap_or_else(|| code.clone());
            symbol_names.insert(code.clone(), name);
        }
        views.push(StrategyView {
            id: c.id.clone(),
            name: c.name.clone(),
            enabled: c.enabled,
            broker_id: c.broker_id,
            broker_account_id: c.broker_account_id.clone(),
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
    let active_scope = {
        let cfg = state.config.read().await.clone();
        let account_id = if cfg.broker_account_id.is_empty() {
            None
        } else {
            Some(cfg.broker_account_id.clone())
        };
        (cfg.broker_id, account_id)
    };
    let target_symbols_snapshot = {
        let mut mgr = state.strategy_manager.lock().await;
        let cfg = mgr.get_config_mut(&input.id).ok_or_else(|| CmdError {
            code: "STRATEGY_NOT_FOUND".into(),
            message: format!("전략을 찾을 수 없습니다: {}", input.id),
        })?;

        if let Some(enabled) = input.enabled {
            cfg.enabled = enabled;
        }
        if let Some(symbols) = input.target_symbols {
            cfg.target_symbols = symbols;
        }
        if let Some(qty) = input.order_quantity {
            cfg.order_quantity = qty;
        }
        if let Some(params) = input.params {
            cfg.params = params;
        }
        cfg.set_scope(active_scope.0, active_scope.1.clone());

        cfg.target_symbols.clone()
    };

    // StockStore에서 종목명 조회
    let mut symbol_names = std::collections::HashMap::new();
    for code in &target_symbols_snapshot {
        let name = state
            .stock_store
            .get_name(code)
            .await
            .unwrap_or_else(|| code.clone());
        symbol_names.insert(code.clone(), name);
    }

    let view = {
        let mgr = state.strategy_manager.lock().await;
        let cfg = mgr
            .all_configs()
            .into_iter()
            .find(|c| c.id == input.id)
            .ok_or_else(|| CmdError {
                code: "STRATEGY_NOT_FOUND".into(),
                message: format!("전략을 찾을 수 없습니다: {}", input.id),
            })?;
        StrategyView {
            id: cfg.id.clone(),
            name: cfg.name.clone(),
            enabled: cfg.enabled,
            broker_id: cfg.broker_id,
            broker_account_id: cfg.broker_account_id.clone(),
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
    /// 리스크 관리 활성화 여부
    pub enabled: bool,
    /// 일일 최대 순손실 한도 (원, 양수)
    pub daily_loss_limit: i64,
    /// 단일 종목 최대 비중 (0.0~1.0)
    pub max_position_ratio: f64,
    /// 전략/종목별 일일 매수 주문 제한. 0이면 제한 없음.
    pub max_daily_buy_orders_per_symbol: u32,
    /// 전략/종목별 일일 매도 주문 제한. 0이면 제한 없음.
    pub max_daily_sell_orders_per_symbol: u32,
    /// 전략/종목별 연속 손실 차단 기준. 0이면 제한 없음.
    pub max_consecutive_losses_per_strategy_symbol: u32,
    /// ATR 기반 주문 수량 산정 활성화 여부
    pub volatility_sizing_enabled: bool,
    /// 거래당 허용 위험 한도(bps). 100 = 1%.
    pub risk_per_trade_bps: u32,
    /// ATR 손절폭 배수
    pub atr_stop_multiplier: f64,
    /// ATR이 준비된 종목 수
    pub atr_symbol_count: usize,
    /// 현재 연속 손실로 신규 진입이 차단된 전략/종목 조합 수
    pub blocked_strategy_symbol_count: usize,
    /// 오늘 누적 총 손실 (음수)
    pub current_loss: i64,
    /// 오늘 누적 총 수익 (양수)
    pub daily_profit: i64,
    /// 순손실 = 총손실 - 당일수익 (양수 = 순손실)
    pub net_loss: i64,
    /// 순손실 한도 소진율 (0.0 ~ 1.0+)
    pub loss_ratio: f64,
    /// 비상 정지 여부
    pub is_emergency_stop: bool,
    /// 추가 거래 가능 여부
    pub can_trade: bool,
}

fn build_risk_view(risk: &crate::trading::risk::RiskManager) -> RiskConfigView {
    RiskConfigView {
        enabled: risk.is_enabled(),
        daily_loss_limit: risk.daily_loss_limit,
        max_position_ratio: risk.max_position_ratio,
        max_daily_buy_orders_per_symbol: risk.max_daily_buy_orders_per_symbol,
        max_daily_sell_orders_per_symbol: risk.max_daily_sell_orders_per_symbol,
        max_consecutive_losses_per_strategy_symbol: risk.max_consecutive_losses_per_strategy_symbol,
        volatility_sizing_enabled: risk.volatility_sizing_enabled,
        risk_per_trade_bps: risk.risk_per_trade_bps,
        atr_stop_multiplier: risk.atr_stop_multiplier,
        atr_symbol_count: risk.atr_symbol_count(),
        blocked_strategy_symbol_count: risk.blocked_strategy_symbol_count(),
        current_loss: risk.current_loss(),
        daily_profit: risk.daily_profit(),
        net_loss: risk.net_loss(),
        loss_ratio: risk.loss_ratio(),
        is_emergency_stop: risk.is_emergency_stop(),
        can_trade: risk.can_trade(),
    }
}

#[tauri::command]
pub async fn get_risk_config(state: State<'_, AppState>) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    // 날짜가 바뀌면 자동으로 당일 손익 초기화
    risk.reset_if_new_day();
    Ok(build_risk_view(&risk))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRiskConfigInput {
    /// 리스크 관리 활성화 여부
    pub enabled: Option<bool>,
    pub daily_loss_limit: Option<i64>,
    /// 0.01 ~ 1.0 (1% ~ 100%)
    pub max_position_ratio: Option<f64>,
    /// 전략/종목별 일일 매수 주문 제한. 0이면 제한 없음.
    pub max_daily_buy_orders_per_symbol: Option<u32>,
    /// 전략/종목별 일일 매도 주문 제한. 0이면 제한 없음.
    pub max_daily_sell_orders_per_symbol: Option<u32>,
    /// 전략/종목별 연속 손실 차단 기준. 0이면 제한 없음.
    pub max_consecutive_losses_per_strategy_symbol: Option<u32>,
    /// ATR 기반 주문 수량 산정 활성화 여부
    pub volatility_sizing_enabled: Option<bool>,
    /// 거래당 허용 위험 한도(bps). 0이면 고정 수량 유지.
    pub risk_per_trade_bps: Option<u32>,
    /// ATR 손절폭 배수
    pub atr_stop_multiplier: Option<f64>,
}

#[tauri::command]
pub async fn update_risk_config(
    input: UpdateRiskConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    if let Some(en) = input.enabled {
        risk.set_enabled(en);
    }
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
    if let Some(limit) = input.max_daily_buy_orders_per_symbol {
        risk.max_daily_buy_orders_per_symbol = limit;
    }
    if let Some(limit) = input.max_daily_sell_orders_per_symbol {
        risk.max_daily_sell_orders_per_symbol = limit;
    }
    if let Some(limit) = input.max_consecutive_losses_per_strategy_symbol {
        risk.max_consecutive_losses_per_strategy_symbol = limit;
    }
    if let Some(enabled) = input.volatility_sizing_enabled {
        risk.volatility_sizing_enabled = enabled;
    }
    if let Some(bps) = input.risk_per_trade_bps {
        if bps > 10_000 {
            return Err(CmdError {
                code: "INVALID_PARAM".into(),
                message: "거래당 위험 한도는 0~10000bps 범위여야 합니다.".into(),
            });
        }
        risk.risk_per_trade_bps = bps;
    }
    if let Some(multiplier) = input.atr_stop_multiplier {
        if !(0.1..=20.0).contains(&multiplier) {
            return Err(CmdError {
                code: "INVALID_PARAM".into(),
                message: "ATR 손절 배수는 0.1~20.0 범위여야 합니다.".into(),
            });
        }
        risk.atr_stop_multiplier = multiplier;
    }
    tracing::info!(
        "리스크 설정 변경: 활성={}, 일일손실한도={}원, 종목비중={:.0}%, 일일주문제한(매수/매도)={}/{}, 연속손실차단={}회, ATR수량산정={}, 거래당위험={}bps, ATR배수={:.2}",
        risk.is_enabled(),
        risk.daily_loss_limit,
        risk.max_position_ratio * 100.0,
        risk.max_daily_buy_orders_per_symbol,
        risk.max_daily_sell_orders_per_symbol,
        risk.max_consecutive_losses_per_strategy_symbol,
        risk.volatility_sizing_enabled,
        risk.risk_per_trade_bps,
        risk.atr_stop_multiplier
    );
    Ok(build_risk_view(&risk))
}

/// 비상 정지 수동 해제
#[tauri::command]
pub async fn clear_emergency_stop(state: State<'_, AppState>) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    risk.clear_emergency_stop();
    Ok(build_risk_view(&risk))
}

/// 비상 정지 수동 발동 (사용자가 직접 자동매매를 중단시킬 때)
#[tauri::command]
pub async fn activate_emergency_stop(state: State<'_, AppState>) -> CmdResult<RiskConfigView> {
    let mut risk = state.risk_manager.lock().await;
    risk.trigger_emergency_stop();
    Ok(build_risk_view(&risk))
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
    /// "pending" | "partially_filled" | "filled" | "cancelled" | "failed"
    pub status: String,
    pub quantity: u64,
    pub filled_quantity: u64,
    pub remaining_quantity: u64,
    pub timestamp: String,
    pub signal_reason: String,
    pub provider: Option<String>,
    pub provider_order_id: Option<String>,
    pub provider_request_id: Option<String>,
    pub provider_tr_id: Option<String>,
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
            status: match &p.record.status {
                crate::storage::order_store::OrderStatus::Pending => "pending".into(),
                crate::storage::order_store::OrderStatus::Filled => "filled".into(),
                crate::storage::order_store::OrderStatus::PartiallyFilled => {
                    "partially_filled".into()
                }
                crate::storage::order_store::OrderStatus::Cancelled => "cancelled".into(),
                crate::storage::order_store::OrderStatus::Failed => "failed".into(),
            },
            quantity: p.record.quantity,
            filled_quantity: p.filled_quantity,
            remaining_quantity: p.record.quantity.saturating_sub(p.filled_quantity),
            timestamp: p.record.timestamp.clone(),
            signal_reason: p.signal_reason.clone(),
            provider: p.record.provider.clone(),
            provider_order_id: p.record.provider_order_id.clone(),
            provider_request_id: p.record.provider_request_id.clone(),
            provider_tr_id: p.record.provider_tr_id.clone(),
        })
        .collect();
    Ok(views)
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
    new_cfg.save_sync(&state.data_dir).map_err(|e| CmdError {
        code: "SAVE_ERR".into(),
        message: e,
    })?;

    // 즉시 정리 실행
    let data_dir = state.data_dir.clone();
    let cfg_clone = new_cfg.clone();
    tokio::task::spawn_blocking(move || purge_old_trade_files(&data_dir, &cfg_clone));

    tracing::info!(
        "체결 기록 보관 설정 변경: 보관 {}일, 최대 {}MB",
        new_cfg.retention_days,
        new_cfg.max_size_mb
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
        let oldest_date = day_dirs
            .first()
            .map(|(d, _)| d.format("%Y-%m-%d").to_string());
        let newest_date = day_dirs
            .last()
            .map(|(d, _)| d.format("%Y-%m-%d").to_string());
        TradeArchiveStats {
            total_files,
            size_bytes,
            oldest_date,
            newest_date,
        }
    })
    .await
    .map_err(|e| CmdError {
        code: "TASK_ERR".into(),
        message: e.to_string(),
    })?;

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
        "warn" => tracing::warn!(target: "frontend", "{}", msg),
        "debug" => tracing::debug!(target: "frontend", "{}", msg),
        _ => tracing::info!(target: "frontend", "{}", msg),
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
            return Ok(vec![StockSearchItem {
                pdno: code,
                prdt_name: name,
                market: None,
            }]);
        }
        // 없으면 KIS get_price로 확인
        let client = state.rest_client.read().await.clone();
        if let Ok(p) = client.get_price(&code).await {
            if !p.hts_kor_isnm.is_empty() {
                state.stock_store.upsert(&code, &p.hts_kor_isnm).await;
                return Ok(vec![StockSearchItem {
                    pdno: code,
                    prdt_name: p.hts_kor_isnm,
                    market: None,
                }]);
            }
        }
        // KIS 실패 시 Yahoo Finance로 이름 조회 (설정 없이도 동작)
        tracing::debug!("KIS 현재가 실패 → Yahoo Finance로 종목명 조회: {}", code);
        match crate::market::lookup_name_by_code(&code).await {
            Ok(name) => {
                tracing::info!("Yahoo Finance 이름 조회 성공: {} → {}", code, name);
                state.stock_store.upsert(&code, &name).await;
                return Ok(vec![StockSearchItem {
                    pdno: code,
                    prdt_name: name,
                    market: None,
                }]);
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
        tracing::debug!(
            "StockStore 검색: query={:?}, {}개 결과",
            query,
            local_results.len()
        );
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

    // ④ KRX 프록시 검색 (k-skill-proxy — 공식 KRX 데이터, API 키 불필요, 시장구분 포함)
    tracing::info!(
        "search_stock: 로컬 캐시 miss → KRX 프록시 검색 (query={:?})",
        query
    );
    match crate::market::search_krx_proxy(&query, 20).await {
        Ok(results) if !results.is_empty() => {
            tracing::info!(
                "KRX 프록시 검색 성공: {}개 결과 (query={:?})",
                results.len(),
                query
            );
            // 결과를 StockStore에 캐시
            state
                .stock_store
                .upsert_many(
                    results
                        .iter()
                        .map(|r| (r.pdno.clone(), r.prdt_name.clone())),
                )
                .await;
            return Ok(results);
        }
        Ok(_) => tracing::debug!("KRX 프록시 결과 없음 (query={:?}), NAVER 폴백 시도", query),
        Err(e) => tracing::warn!(
            "KRX 프록시 검색 실패: {} (query={:?}), NAVER 폴백 시도",
            e,
            query
        ),
    }

    // ⑤ NAVER Finance 실시간 검색 폴백 (최후 수단)
    tracing::info!(
        "search_stock: KRX 프록시 miss → NAVER 실시간 검색 (query={:?})",
        query
    );
    match crate::market::search_naver_live(&query).await {
        Ok(results) if !results.is_empty() => {
            tracing::info!(
                "NAVER 검색 성공: {}개 결과 (query={:?})",
                results.len(),
                query
            );
            // NAVER 결과도 StockStore에 캐시
            state
                .stock_store
                .upsert_many(
                    results
                        .iter()
                        .map(|r| (r.pdno.clone(), r.prdt_name.clone())),
                )
                .await;
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
    let file_path = state
        .data_dir
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
pub async fn set_stock_update_interval(hours: u32, state: State<'_, AppState>) -> CmdResult<()> {
    state
        .stock_store
        .set_interval_hours(hours)
        .await
        .map_err(CmdError::from)?;
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

#[tauri::command]
pub async fn get_overseas_executed_by_range(
    from: String, // YYYY-MM-DD
    to: String,   // YYYY-MM-DD
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::api::rest::OverseasExecutedOrder>> {
    let from_fmt = from.replace('-', "");
    let to_fmt = to.replace('-', "");
    let client = state.rest_client.read().await.clone();
    client
        .get_overseas_executed_orders_range(&from_fmt, &to_fmt)
        .await
        .map_err(CmdError::from)
}

// ── 최근 로그 엔트리 (파일 기반) ──────────────────────────────────
#[tauri::command]
pub async fn get_recent_logs(
    count: u32,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::logging::LogEntry>> {
    Ok(crate::logging::read_recent_entries(
        &state.log_dir,
        count as usize,
    ))
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
        name: resp.name,
        last: resp.last,
        diff: resp.diff,
        rate: resp.rate,
        open: resp.open,
        high: resp.high,
        low: resp.low,
        h52p: resp.h52p,
        l52p: resp.l52p,
        tvol: resp.tvol,
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
        input.exchange,
        input.symbol,
        input.side,
        input.quantity,
        input.price
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
                input.exchange,
                input.symbol,
                resp.odno,
                resp.ord_tmd
            );
            Ok(resp)
        }
        Err(e) => {
            tracing::error!(
                "해외 주문 실패: {} {} 수량={} 가격={} — {}",
                input.exchange,
                input.symbol,
                input.quantity,
                input.price,
                e
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
/// 반환값: (price_cents: u64, volume: u64, exchange: String)
/// - price_cents = USD 현재가 × 100 (정수화하여 on_tick에 전달)
/// - exchange = 성공한 거래소 코드 ("NAS" / "NYS" / "AMS")
async fn fetch_overseas_tick(
    rest: &std::sync::Arc<crate::api::rest::KisRestClient>,
    symbol: &str,
) -> anyhow::Result<(u64, u64, String)> {
    for exchange in &["NAS", "NYS", "AMS"] {
        match rest.get_overseas_price(symbol, exchange).await {
            Ok(p) => {
                let price_f: f64 = p.last.parse().unwrap_or(0.0);
                if price_f > 0.0 {
                    // USD → 센트(×100) 변환으로 u64 정수화
                    let price_cents = (price_f * 100.0).round() as u64;
                    let volume = p.tvol.parse::<u64>().unwrap_or(0);
                    return Ok((price_cents, volume, exchange.to_string()));
                }
            }
            Err(_) => continue,
        }
    }
    anyhow::bail!("해외 현재가 조회 실패: {} (NAS/NYS/AMS 모두 실패)", symbol)
}
