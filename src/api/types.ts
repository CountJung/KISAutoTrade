/// Rust commands.rs 와 1:1 대응하는 TypeScript 타입 정의

// ─── 공통 오류 ──────────────────────────────────────────────────────
export interface CmdError {
  code: string
  message: string
}

// ─── 앱 설정 뷰 ────────────────────────────────────────────────────
export interface AppConfigView {
  kis_app_key_masked: string
  kis_account_no: string
  kis_is_paper_trading: boolean
  kis_configured: boolean
  discord_enabled: boolean
  notification_levels: string[]
  active_profile_id: string | null
  active_profile_name: string | null
}

// ─── 계좌 프로파일 ───────────────────────────────────────────────────
export interface AccountProfileView {
  id: string
  name: string
  is_paper_trading: boolean
  app_key_masked: string
  account_no: string
  is_active: boolean
  is_configured: boolean
}

export interface AddProfileInput {
  name: string
  is_paper_trading: boolean
  app_key: string
  app_secret: string
  account_no: string
}

export interface UpdateProfileInput {
  id: string
  name?: string
  is_paper_trading?: boolean
  /** 빈 문자열 = 변경 안 함 */
  app_key?: string
  /** 빈 문자열 = 변경 안 함 */
  app_secret?: string
  account_no?: string
}

// ─── 실전/모의투자 자동 감지 결과 ─────────────────────────────────
export interface DetectTradingTypeResult {
  /** true = 모의투자, false = 실전투자 */
  is_paper_trading: boolean
  message: string
}

// ─── 진단 모드 ────────────────────────────────────────────────
export interface ConfigDiagnostic {
  real_key_set: boolean
  real_account_set: boolean
  paper_key_set: boolean
  active_mode: string
  is_ready: boolean
  discord_configured: boolean
  base_url: string
  issues: string[]
}

// ─── 잔고 ──────────────────────────────────────────────────────────
export interface BalanceItem {
  pdno: string
  prdt_name: string
  hldg_qty: string
  pchs_avg_pric: string
  prpr: string
  evlu_pfls_amt: string
  evlu_pfls_rt: string
}

export interface BalanceSummary {
  dnca_tot_amt: string
  tot_evlu_amt: string
  nass_amt: string
  tot_evlu_pfls_rt: string
}

export interface BalanceResult {
  items: BalanceItem[]
  summary: BalanceSummary | null
}

// ─── 현재가 ────────────────────────────────────────────────────────
export interface PriceResponse {
  stck_prpr: string
  prdy_vrss: string
  prdy_ctrt: string
  acml_vol: string
  hts_kor_isnm: string
  stck_oprc?: string   // 시가
  stck_hgpr?: string   // 고가
  stck_lwpr?: string   // 저가
  stck_mxpr?: string   // 상한가
  stck_llam?: string   // 하한가
  w52_hgpr?: string    // 52주 최고가
  w52_lwpr?: string    // 52주 최저가
}

// ─── 앱 로그 아1트리 ────────────────────────────────────────────
export interface AppLogEntry {
  timestamp: string
  level: 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'TRACE'
  target: string
  message: string
}

// ─── 종목 검색 ─────────────────────────────────────────────────────
export interface StockSearchItem {
  pdno: string
  prdt_name: string
}

// ─── 주문 ──────────────────────────────────────────────────────────
export type OrderSide = 'Buy' | 'Sell'
export type OrderType = 'Limit' | 'Market'

export interface PlaceOrderInput {
  symbol: string
  side: OrderSide
  order_type: OrderType
  quantity: number
  price: number
}

export interface OrderResponse {
  odno: string
  ord_tmd: string
  rt_cd: string
  msg1: string
}

// ─── 체결 내역 ─────────────────────────────────────────────────────
export interface ExecutedOrder {
  pdno: string
  prdt_name: string
  sll_buy_dvsn_cd: string
  ord_qty: string
  ord_unpr: string
  tot_ccld_qty: string
  tot_ccld_amt: string
  odno: string
  ord_dt: string
  ord_tmd: string
}

// ─── 로컬 체결 기록 ────────────────────────────────────────────────
export type TradeSide = 'buy' | 'sell'
export type TradeStatus = 'filled' | 'partially_filled' | 'cancelled'

export interface TradeRecord {
  id: string
  timestamp: string
  symbol: string
  symbol_name: string
  side: TradeSide
  quantity: number
  price: number
  total_amount: number
  fee: number
  strategy_id: string | null
  order_id: string
  status: TradeStatus
}

// ─── 자동 매매 상태 ─────────────────────────────────────────────────
export interface TradingStatus {
  isRunning: boolean
  activeStrategies: string[]
  positionCount: number
  totalUnrealizedPnl: number
  /** WebSocket 실시간 시세 연결 여부 */
  wsConnected: boolean
}

// ─── WebSocket 연결 상태 이벤트 (Tauri 'ws-status' 이벤트 페이로드) ──
export interface WsStatusEvent {
  connected: boolean
  message: string
}

// ─── 포지션 ────────────────────────────────────────────────────────
export interface PositionView {
  symbol: string
  symbolName: string
  quantity: number
  avgPrice: number
  currentPrice: number
  unrealizedPnl: number
  unrealizedPnlRate: number
}

// ─── 전략 ──────────────────────────────────────────────────────────
export interface StrategyView {
  id: string
  name: string
  enabled: boolean
  targetSymbols: string[]
  orderQuantity: number
  params: Record<string, unknown>
}

export interface UpdateStrategyInput {
  id: string
  enabled?: boolean
  targetSymbols?: string[]
  orderQuantity?: number
  params?: Record<string, unknown>
}

// ─── 일별 통계 ─────────────────────────────────────────────────────
export interface DailyStats {
  date: string
  total_trades: number
  winning_trades: number
  losing_trades: number
  gross_profit: number
  gross_loss: number
  net_profit: number
  fees_paid: number
  win_rate: number
  profit_factor: number
  starting_balance: number
  ending_balance: number
}

// ─── 로그 설정 ─────────────────────────────────────────────────────
export interface LogConfig {
  retention_days: number
  max_size_mb: number
}

export interface SetLogConfigInput {
  retention_days: number
  max_size_mb: number
}

export type FrontendLogLevel = 'error' | 'warn' | 'info' | 'debug'

export interface FrontendLogInput {
  level: FrontendLogLevel
  message: string
  context?: string
}

// ─── 차트 ────────────────────────────────────────────────────────
/// Rust ChartCandle 의 TypeScript 미러
export interface ChartCandle {
  date: string   // YYYYMMDD
  open: string
  high: string
  low: string
  close: string
  volume: string
}

export interface ChartDataInput {
  symbol: string
  period_code: string   // 'D' | 'W' | 'M'
  start_date: string    // YYYYMMDD
  end_date: string      // YYYYMMDD
}

// ─── 업데이트 정보 ────────────────────────────────────────────────
export interface UpdateInfo {
  hasUpdate: boolean
  currentVersion: string
  latestVersion: string
  releaseUrl: string
  releaseNotes: string | null
}
// ─── 해외(미국) 주식 ───────────────────────────────────────────────
/** KIS 해외 거래소 코드 */
export type OverseasExchange = 'NAS' | 'NYS' | 'AMS'
/** KIS 해외 주문용 거래소 코드 (TR-ID OVRS_EXCG_CD) */
export type OverseasOrderExchange = 'NASD' | 'NYSE' | 'AMEX'

export interface OverseasPriceResponse {
  /** 현지 현재가 (USD 등) */
  last: string
  /** 전일대비 */
  diff: string
  /** 등락률 (%) */
  rate: string
  /** 거래량 */
  tvol: string
  /** 종목명 */
  name: string
  /** 시가 */
  open: string
  /** 고가 */
  high: string
  /** 저가 */
  low: string
  /** 52주 최고 */
  h52p: string
  /** 52주 최저 */
  l52p: string
  /** 거래소 코드 (NAS/NYS/AMS) */
  exchange: string
  /** 티커 (AAPL 등) */
  symbol: string
}

export interface PlaceOverseasOrderInput {
  symbol: string
  exchange: OverseasOrderExchange
  side: OrderSide
  /** 해외는 지정가만 지원 */
  price: number
  quantity: number
}

// ─── 웹 접속 설정 ─────────────────────────────────────────────────
export interface WebConfig {
  runningPort: number
  accessUrl: string
}

// ─── 리스크 관리 ───────────────────────────────────────────────────
export interface RiskConfigView {
  /** 일일 최대 손실 한도 (원) */
  dailyLossLimit: number
  /** 단일 종목 최대 비중 (0.0~1.0) */
  maxPositionRatio: number
  /** 오늘 누적 손실 (음수) */
  currentLoss: number
  /** 손실 소진율 (0.0~1.0+) */
  lossRatio: number
  /** 비상 정지 여부 */
  isEmergencyStop: boolean
  /** 추가 거래 가능 여부 */
  canTrade: boolean
}

export interface UpdateRiskConfigInput {
  dailyLossLimit?: number
  /** 0.01 ~ 1.0 */
  maxPositionRatio?: number
}

// ─── 미체결 주문 ───────────────────────────────────────────────────
export interface PendingOrderView {
  odno: string
  symbol: string
  symbolName: string
  /** "buy" | "sell" */
  side: string
  quantity: number
  timestamp: string
  signalReason: string
}