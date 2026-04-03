/**
 * Tauri IPC invoke 래퍼
 * 모든 커맨드 호출 시 에러를 타입 안전하게 처리합니다.
 */
import { invoke } from '@tauri-apps/api/core'

import type {
  AccountProfileView,
  AddProfileInput,
  AppConfigView,
  AppLogEntry,
  BalanceResult,
  ChartCandle,
  ChartDataInput,
  ConfigDiagnostic,
  DailyStats,
  ExecutedOrder,
  FrontendLogInput,
  LogConfig,
  OrderResponse,
  PlaceOrderInput,
  PositionView,
  PriceResponse,
  SetLogConfigInput,
  StockSearchItem,
  StrategyView,
  TradeRecord,
  TradingStatus,
  UpdateProfileInput,
  UpdateStrategyInput,
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
