/**
 * Tauri IPC invoke 래퍼
 * 모든 커맨드 호출 시 에러를 타입 안전하게 처리합니다.
 */
import { invoke } from './transport'

import type {
  AccountProfileView,
  AddProfileInput,
  AppConfigView,
  AppLogEntry,
  BalanceResult,
  OverseasBalanceResult,
  ChartCandle,
  ChartDataInput,
  ConfigDiagnostic,
  DailyStats,
  DetectTradingTypeResult,
  ExecutedOrder,
  FrontendLogInput,
  LogConfig,
  OrderResponse,
  OverseasPriceResponse,
  PendingOrderView,
  PlaceOrderInput,
  PlaceOverseasOrderInput,
  PositionView,
  PriceResponse,
  RiskConfigView,
  SetLogConfigInput,
  SetTradeArchiveConfigInput,
  StockListStats,
  StockSearchItem,
  StrategyView,
  TradeArchiveConfig,
  TradeArchiveStats,
  TradeRecord,
  TradingStatus,
  UpdateInfo,
  UpdateProfileInput,
  UpdateRiskConfigInput,
  UpdateStrategyInput,
  WebConfig,
} from './types'

// ─── 앱 설정 ───────────────────────────────────────────────────────
export const getAppConfig = (): Promise<AppConfigView> =>
  invoke('get_app_config')

// ─── 계좌 프로파일 ─────────────────────────────────────────────────
export const listProfiles = (): Promise<AccountProfileView[]> =>
  invoke('list_profiles')

export const addProfile = (input: AddProfileInput): Promise<AccountProfileView> =>
  invoke('add_profile', { input })

export const updateProfile = (input: UpdateProfileInput): Promise<AccountProfileView> =>
  invoke('update_profile', { input })

export const deleteProfile = (id: string): Promise<void> =>
  invoke('delete_profile', { id })

export const setActiveProfile = (id: string): Promise<AppConfigView> =>
  invoke('set_active_profile', { id })

// ─── 잔고 ──────────────────────────────────────────────────────────
export const getBalance = (): Promise<BalanceResult> =>
  invoke('get_balance')

export const getOverseasBalance = (): Promise<OverseasBalanceResult> =>
  invoke('get_overseas_balance')

// ─── 현재가 ────────────────────────────────────────────────────────
export const getPrice = (symbol: string): Promise<PriceResponse> =>
  invoke('get_price', { symbol })

// ─── 차트 ───────────────────────────────────────────────────────
export const getChartData = (input: ChartDataInput): Promise<ChartCandle[]> =>
  invoke('get_chart_data', { input })

// ─── 주문 ──────────────────────────────────────────────────────────
export const placeOrder = (input: PlaceOrderInput): Promise<OrderResponse> =>
  invoke('place_order', { input })

// ─── 종목 검색 ────────────────────────────────────────────────────
export const searchStock = (query: string): Promise<StockSearchItem[]> =>
  invoke('search_stock', { query })

export const refreshStockList = (): Promise<number> =>
  invoke('refresh_stock_list')

export const getStockListStats = (): Promise<StockListStats> =>
  invoke('get_stock_list_stats')

export const setStockUpdateInterval = (hours: number): Promise<void> =>
  invoke('set_stock_update_interval', { hours })

// ─── KIS 기간별 체결 내역 ──────────────────────────────────────────
export const getKisExecutedByRange = (from: string, to: string): Promise<ExecutedOrder[]> =>
  invoke('get_kis_executed_by_range', { from, to })

// ─── 최근 앱 로그 (파일 기반) ─────────────────────────────────────
export const getRecentLogs = (count: number): Promise<AppLogEntry[]> =>
  invoke('get_recent_logs', { count })

// ─── 당일 체결 내역 (KIS) ─────────────────────────────────────────
export const getTodayExecuted = (): Promise<ExecutedOrder[]> =>
  invoke('get_today_executed')

// ─── 로컬 체결 기록 ────────────────────────────────────────────────
export const getTodayTrades = (): Promise<TradeRecord[]> =>
  invoke('get_today_trades')

export const getTradesByRange = (from: string, to: string): Promise<TradeRecord[]> =>
  invoke('get_trades_by_range', { input: { from, to } })

// ─── 통계 ──────────────────────────────────────────────────────────
export const getTodayStats = (): Promise<DailyStats> =>
  invoke('get_today_stats')

export const getStatsByRange = (from: string, to: string): Promise<DailyStats[]> =>
  invoke('get_stats_by_range', { input: { from, to } })

// ─── Discord 테스트 ────────────────────────────────────────────────
export const sendTestDiscord = (): Promise<string> =>
  invoke('send_test_discord')

// ─── 체결 기록 저장 ────────────────────────────────────────────────
export const saveTrade = (input: TradeRecord): Promise<TradeRecord> =>
  invoke('save_trade', { input })

// ─── 일별 통계 저장/갱신 ──────────────────────────────────────────
export const upsertDailyStats = (stats: DailyStats): Promise<void> =>
  invoke('upsert_daily_stats', { stats })

// ─── 진단 모드 ────────────────────────────────────────────────────
export const checkConfig = (): Promise<ConfigDiagnostic> =>
  invoke('check_config')

// ─── 자동 매매 ────────────────────────────────────────────────────
export const getTradingStatus = (): Promise<TradingStatus> =>
  invoke('get_trading_status')

export const startTrading = (): Promise<TradingStatus> =>
  invoke('start_trading')

export const stopTrading = (): Promise<TradingStatus> =>
  invoke('stop_trading')

// ─── 포지션 ────────────────────────────────────────────────────────
export const getPositions = (): Promise<PositionView[]> =>
  invoke('get_positions')

// ─── 전략 ──────────────────────────────────────────────────────────
export const getStrategies = (): Promise<StrategyView[]> =>
  invoke('get_strategies')

export const updateStrategy = (input: UpdateStrategyInput): Promise<StrategyView> =>
  invoke('update_strategy', { input })

// ─── 로그 설정 ────────────────────────────────────────────────────
export const getLogConfig = (): Promise<LogConfig> =>
  invoke('get_log_config')

export const setLogConfig = (input: SetLogConfigInput): Promise<LogConfig> =>
  invoke('set_log_config', { input })

// ─── 프론트엔드 로그 기록 ─────────────────────────────────────────
export const writeFrontendLog = (input: FrontendLogInput): Promise<void> =>
  invoke('write_frontend_log', { input })

// ─── 업데이트 확인 ────────────────────────────────────────────────
export const checkForUpdate = (): Promise<UpdateInfo> =>
  invoke('check_for_update')
// ─── 웹 접속 설정 ─────────────────────────────────────────────────
export const getWebConfig = (): Promise<WebConfig> =>
  invoke('get_web_config')

export const saveWebConfig = (newPort: number): Promise<string> =>
  invoke('save_web_config', { newPort })

// ─── 실전/모의투자 자동 감지 ──────────────────────────────────────
export const detectTradingType = (appKey: string, appSecret: string): Promise<DetectTradingTypeResult> =>
  invoke('detect_trading_type', { appKey, appSecret })

/** 저장된 프로파일의 키로 직접 감지 후 is_paper_trading 자동 업데이트 */
export const detectProfileTradingType = (profileId: string): Promise<AccountProfileView> =>
  invoke('detect_profile_trading_type', { profileId })

// ─── 해외(미국) 주식 ───────────────────────────────────────────────
/** 해외 현재가 조회 (NAS/NYS/AMS) */
export const getOverseasPrice = (symbol: string, exchange: string): Promise<OverseasPriceResponse> =>
  invoke('get_overseas_price', { symbol, exchange })

/** 해외 주식 기간별 차트 데이터 (일/주/월봉) */
export const getOverseasChartData = (
  symbol: string,
  exchange: string,
  periodCode: string,
  baseDate: string,
): Promise<ChartCandle[]> =>
  invoke('get_overseas_chart_data', { symbol, exchange, periodCode, baseDate })

/** 해외 주식 주문 (지정가 한정) */
export const placeOverseasOrder = (input: PlaceOverseasOrderInput): Promise<OrderResponse> =>
  invoke('place_overseas_order', { input })

// ─── 리스크 관리 ───────────────────────────────────────────────────
export const getRiskConfig = (): Promise<RiskConfigView> =>
  invoke('get_risk_config')

export const updateRiskConfig = (input: UpdateRiskConfigInput): Promise<RiskConfigView> =>
  invoke('update_risk_config', { input })

export const clearEmergencyStop = (): Promise<RiskConfigView> =>
  invoke('clear_emergency_stop')

export const activateEmergencyStop = (): Promise<RiskConfigView> =>
  invoke('activate_emergency_stop')

/** 잔고 부족 매수 정지를 수동으로 해제 */
export const clearBuySuspension = (): Promise<TradingStatus> =>
  invoke('clear_buy_suspension')

// ─── 미체결 주문 목록 ──────────────────────────────────────────
export const getPendingOrders = (): Promise<PendingOrderView[]> =>
  invoke('get_pending_orders')

// ─── 환율 / 공통 갱신 주기 ──────────────────────────────────────
/** 현재 USD/KRW 환율 조회 (캐시, REFRESH_INTERVAL_SEC마다 갱신) */
export const getExchangeRate = (): Promise<number> =>
  invoke('get_exchange_rate')

/** 공통 데이터 갱신 주기 조회 (초) — REFRESH_INTERVAL_SEC 환경변수 */
export const getRefreshInterval = (): Promise<number> =>
  invoke('get_refresh_interval')

// ─── 체결 기록 보관 설정 ──────────────────────────────────────────
export const getTradeArchiveConfig = (): Promise<TradeArchiveConfig> =>
  invoke('get_trade_archive_config')

export const setTradeArchiveConfig = (input: SetTradeArchiveConfigInput): Promise<TradeArchiveConfig> =>
  invoke('set_trade_archive_config', { input })

export const getTradeArchiveStats = (): Promise<TradeArchiveStats> =>
  invoke('get_trade_archive_stats')
