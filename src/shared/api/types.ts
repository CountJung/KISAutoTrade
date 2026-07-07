/// Rust commands.rs 와 1:1 대응하는 TypeScript 타입 정의

// ─── 공통 오류 ──────────────────────────────────────────────────────
export interface CmdError {
  code: string
  message: string
}

export type BrokerId = 'kis' | 'toss'
export type BrokerMarket = 'kr' | 'us'
export type BrokerCurrency = 'KRW' | 'USD'

// ─── 앱 설정 뷰 ────────────────────────────────────────────────────
export interface AppConfigView {
  active_broker_id: BrokerId
  active_broker_account_id: string | null
  kis_app_key_masked: string
  kis_account_no: string
  kis_is_paper_trading: boolean
  kis_configured: boolean
  active_broker_configured: boolean
  discord_enabled: boolean
  notification_levels: string[]
  active_profile_id: string | null
  active_profile_name: string | null
}

// ─── 계좌 프로파일 ───────────────────────────────────────────────────
export interface AccountProfileView {
  id: string
  broker_id: BrokerId
  broker_account_id: string
  name: string
  is_paper_trading: boolean
  live_trading_consent: boolean
  app_key_masked: string
  account_no: string
  is_active: boolean
  is_configured: boolean
}

export interface AddProfileInput {
  broker_id?: BrokerId
  name: string
  is_paper_trading: boolean
  live_trading_consent: boolean
  app_key: string
  app_secret: string
  account_no: string
}

export interface UpdateProfileInput {
  id: string
  broker_id?: BrokerId
  name?: string
  is_paper_trading?: boolean
  live_trading_consent?: boolean
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

export type ExchangeRateSource = 'toss' | 'externalPublic' | 'cachedFallback'

export interface ExchangeRateView {
  rate: number
  source: ExchangeRateSource
  fallbackUsed: boolean
  baseCurrency: 'USD' | 'KRW' | string
  quoteCurrency: 'USD' | 'KRW' | string
  rateText: string
  midRate: string | null
  basisPoint: string | null
  rateChangeType: string | null
  validFrom: string | null
  validUntil: string | null
  updatedAt: string
  message: string
}

// ─── 진단 모드 ────────────────────────────────────────────────
export interface ConfigDiagnostic {
  broker_id: BrokerId
  broker_account_id: string | null
  real_key_set: boolean
  real_account_set: boolean
  paper_key_set: boolean
  active_mode: string
  is_ready: boolean
  discord_configured: boolean
  base_url: string
  issues: string[]
}

export interface TossConnectionStep {
  id: string
  label: string
  ok: boolean
  message: string
}

export interface TossAccountLookupInput {
  client_id: string
  client_secret: string
}

export interface TossAccountOptionView {
  account_seq: string
  account_no_masked: string
  account_type: string
  label: string
}

export interface TossConnectionDiagnostic {
  profile_id: string
  profile_name: string
  broker_id: 'toss'
  account_seq: string
  openapi_title: string | null
  openapi_version: string | null
  openapi_server: string | null
  openapi_paths_count: number | null
  token_type: string | null
  token_expires_at: string | null
  accounts_count: number | null
  matched_account_no: string | null
  holdings_count: number | null
  buying_power_krw: string | null
  buying_power_usd: string | null
  commissions_count: number | null
  sellable_quantity_symbol: string | null
  sellable_quantity: string | null
  is_ready: boolean
  steps: TossConnectionStep[]
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
  /** 예수금총금액 (D+0, 매수 당일 결제 전 음수 가능) */
  dnca_tot_amt: string
  /** 익일정산금액 (D+1 예수금) */
  nxdy_excc_amt: string
  /** 가수도정산금액 (D+2 예수금, 실제 인출·매매 가능 현금) */
  prvs_rcdl_excc_amt: string
  tot_evlu_amt: string
  nass_amt: string
  tot_evlu_pfls_rt: string
}

export interface BalanceResult {
  items: BalanceItem[]
  summary: BalanceSummary | null
}

// ─── 해외 잔고 ─────────────────────────────────────────────────────
export interface OverseasBalanceItem {
  /** 종목코드 (티커) */
  ovrs_pdno: string
  /** 종목명 */
  ovrs_item_name: string
  /** 해외잔고수량 */
  ovrs_cblc_qty: string
  /** 매입평균가격 (USD) */
  pchs_avg_pric: string
  /** 현재가 (USD) */
  now_pric2: string
  /** 해외주식평가금액 (USD) */
  ovrs_stck_evlu_amt: string
  /** 외화평가손익금액 (USD) */
  frcr_evlu_pfls_amt: string
  /** 평가손익율 (%) */
  evlu_pfls_rt: string
  /** 거래소코드 (NAS/NYS/AMS 등) */
  ovrs_excg_cd: string
  /** 시장명 */
  tr_mket_name: string
}

export interface OverseasBalanceSummary {
  /** 외화매입금액합계 (USD) */
  frcr_pchs_amt1: string
  /** 해외총손익 (USD) */
  ovrs_tot_pfls: string
  /** 외화평가금액합계 (USD) */
  frcr_evlu_tota: string
  /** 총수익률 (%) */
  tot_pftrt: string
}

export interface OverseasBalanceResult {
  items: OverseasBalanceItem[]
  summary: OverseasBalanceSummary | null
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
  /** 시장구분: "KOSPI" | "KOSDAQ" | "KONEX" | "US" */
  market?: string
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
  toss_session?: TossManualSession | null
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

export interface OverseasExecutedOrder {
  pdno: string
  prdt_name: string
  ovrs_excg_cd: string
  sll_buy_dvsn: string
  ccld_nccs_dvsn: string
  ft_ord_qty: string
  ft_ord_unpr3: string
  ft_ccld_qty: string
  ft_ccld_unpr3: string
  ft_ccld_amt3: string
  odno: string
  ord_dt: string
  ord_tmd: string
  ord_gno_brno: string
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
  /** 국내는 KRW, 해외는 USD cents */
  price: number
  /** 국내는 KRW, 해외는 USD cents */
  total_amount: number
  /** 국내는 KRW, 해외는 USD cents */
  fee: number
  strategy_id: string | null
  order_id: string
  status: TradeStatus
  /** 체결 원인 — 어떤 전략 신호에 의해 매매됩는지 */
  signal_reason: string
  market?: 'domestic' | 'overseas'
  currency?: 'KRW' | 'USD'
  exchange?: string | null
  price_usd?: number | null
  total_amount_usd?: number | null
  fee_usd?: number | null
  exchange_rate_krw?: number | null
  total_amount_krw?: number | null
  fee_krw?: number | null
  realized_pnl_usd?: number | null
  realized_pnl_krw?: number | null
  /** 신호 발생 시점 가격. 국내는 KRW, 해외는 USD cents */
  signal_price?: number | null
  /** 주문 제출 가격. 국내 시장가는 0, 해외는 USD cents */
  order_price?: number | null
  /** 슬리피지 비용. 국내는 KRW, 해외는 USD cents. 양수면 불리, 음수면 유리 */
  slippage?: number | null
  /** 신호가 대비 슬리피지 비용(bps). 양수면 불리, 음수면 유리 */
  slippage_bps?: number | null
  /** 원천 provider 식별자 (예: kis, toss) */
  provider?: string | null
  /** provider가 반환한 주문 ID (KIS odno, Toss order id 등) */
  provider_order_id?: string | null
  /** provider 원본 요청 추적 ID (Toss requestId, X-Request-Id 등) */
  provider_request_id?: string | null
  /** provider 요청 TR-ID (KIS TR-ID 등) */
  provider_tr_id?: string | null
}

// ─── 자동 매매 상태 ─────────────────────────────────────────────────
export interface TradingStatus {
  isRunning: boolean
  activeStrategies: string[]
  positionCount: number
  totalUnrealizedPnl: number
  /** WebSocket 실시간 시세 연결 여부 */
  wsConnected: boolean
  /** 자동매매가 실행 중인 프로파일 ID (미실행 시 null) */
  tradingProfileId: string | null
  /** 자동매매가 실행 중인 broker ID (미실행 시 null) */
  tradingBrokerId: BrokerId | null
  /** 자동매매가 실행 중인 broker account ID (미실행 시 null) */
  tradingAccountId: string | null
  /** 잔고 부족으로 매수가 정지된 여부 */
  buySuspended: boolean
  /** 매수 정지 사유 (KIS msg1, 없으면 null) */
  buySuspendedReason?: string | null
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

export interface BrokerMoneyView {
  amount: string
  currency: BrokerCurrency
}

export interface BrokerHoldingView {
  brokerId: BrokerId
  accountId: string | null
  market: BrokerMarket
  symbol: string
  symbolName: string
  quantity: string
  averagePrice: BrokerMoneyView
  currentPrice: BrokerMoneyView
  unrealizedPnl: BrokerMoneyView | null
}

export interface TossOrderbookEntryView {
  price: string
  volume: string
}

export interface TossOrderbookView {
  timestamp: string | null
  currency: string
  asks: TossOrderbookEntryView[]
  bids: TossOrderbookEntryView[]
}

export interface TossTradeView {
  price: string
  volume: string
  timestamp: string
  currency: string
}

export interface TossPriceLimitView {
  timestamp: string
  upperLimitPrice: string | null
  lowerLimitPrice: string | null
  currency: string
}

export interface TossMarketSnapshotView {
  brokerId: 'toss'
  market: BrokerMarket
  symbol: string
  timestamp: string | null
  price: BrokerMoneyView
  orderbook: TossOrderbookView
  trades: TossTradeView[]
  priceLimits: TossPriceLimitView
}

export interface TossStockInfoView {
  symbol: string
  name: string
  englishName: string
  isinCode: string
  market: string
  securityType: string
  isCommonShare: boolean
  status: string
  currency: string
  listDate: string | null
  delistDate: string | null
  sharesOutstanding: string
  leverageFactor: string | null
}

export interface TossStockWarningView {
  warningType: string
  label: string
  exchange: string | null
  startDate: string | null
  endDate: string | null
  blockingForBuy: boolean
}

export interface TossStockSafetyView {
  brokerId: 'toss'
  symbol: string
  stockInfo: TossStockInfoView | null
  warnings: TossStockWarningView[]
  hasBlockingWarning: boolean
  buyBlocked: boolean
  buyBlockReason: string | null
}

export interface TossOrderPreflightInput {
  symbol: string
  side: OrderSide
  quantity: string
  price?: string | null
}

export interface TossOrderPreflightView {
  brokerId: 'toss'
  accountSeq: string
  symbol: string
  market: 'kr' | 'us'
  side: 'buy' | 'sell'
  quantity: string
  price: BrokerMoneyView
  priceSource: 'input' | 'snapshot'
  buyingPower: BrokerMoneyView | null
  sellableQuantity: string | null
  commissionRate: string | null
  grossAmount: BrokerMoneyView
  estimatedCommission: BrokerMoneyView | null
  requiredCash: BrokerMoneyView | null
  liquidityOk: boolean
  safetyOk: boolean
  orderAdapterSupported: boolean
  canSubmit: boolean
  blockedReasons: string[]
  warnings: string[]
}

export interface TossOpenOrdersInput {
  symbol?: string | null
}

export interface TossOpenOrderView {
  brokerId: 'toss'
  accountSeq: string
  orderId: string
  symbol: string
  side: 'BUY' | 'SELL' | string
  orderType: 'MARKET' | 'LIMIT' | string
  status: string
  price: string | null
  quantity: string
  currency: BrokerCurrency | string
  orderedAt: string
  canceledAt: string | null
  filledQuantity: string
  averageFilledPrice: string | null
  filledAmount: string | null
  commission: string | null
  tax: string | null
}

export interface TossModifyOrderInput {
  orderId: string
  orderType: 'MARKET' | 'LIMIT' | string
  quantity?: string | null
  price?: string | null
  confirmHighValueOrder?: boolean | null
}

export interface TossOrderOperationView {
  brokerId: 'toss'
  accountSeq: string
  orderId: string
  message: string
}

export interface TossSmallBuyVerificationInput {
  symbol: string
  symbolName?: string | null
  expectedAccountSeq: string
  maxNotionalAmount: string
  confirmed: boolean
}

export interface TossSmallBuyVerificationView {
  brokerId: 'toss'
  accountSeq: string
  symbol: string
  symbolName: string
  market: 'kr' | 'us'
  side: 'buy'
  orderType: 'MARKET' | string
  quantity: string
  estimatedGrossAmount: BrokerMoneyView
  requiredCash: BrokerMoneyView | null
  orderId: string
  clientOrderId: string | null
  status: string
  filledQuantity: string
  averageFilledPrice: BrokerMoneyView | null
  filledAmount: BrokerMoneyView | null
  orderRecordId: string
  tradeRecorded: boolean
  message: string
}

export interface TossMarketSessionView {
  startTime: string
  endTime: string
}

export type TossManualSession = 'auto' | 'day' | 'pre' | 'regular' | 'after'

export interface TossMarketDayView {
  date: string
  daySession: TossMarketSessionView | null
  preSession: TossMarketSessionView | null
  regularSession: TossMarketSessionView | null
  afterSession: TossMarketSessionView | null
  isDayOpen: boolean
  isPreOpen: boolean
  isRegularOpen: boolean
  isAfterOpen: boolean
}

export interface TossMarketCalendarView {
  brokerId: 'toss'
  kr: TossMarketDayView
  us: TossMarketDayView
  summary: string
}

// ─── 가격 조건 매매 종목별 설정 ──────────────────────────────────────
/** PriceConditionStrategy params.symbols 내 개별 종목 설정 (snake_case: Rust params JSON과 1:1) */
export interface PriceConditionSymbolConfig {
  symbol: string
  symbol_name: string
  quantity: number
  buy_trigger_price: number
  sell_trigger_price: number
  take_profit_pct: number
  stop_loss_pct: number
  /** 해외 주식 여부. true이면 가격 단위 = USD */
  is_overseas: boolean
}

// ─── 레버리지 추세 보유 전략 설정 ─────────────────────────────────
export type LeveragedTrendHoldBaseRole = 'underlying' | 'proxy'

export interface LeveragedTrendHoldEntry {
  leveraged_symbol: string
  leveraged_symbol_name: string
  /** legacy compatibility: no longer used by the single-ticker strategy model */
  inverse_leveraged_symbol: string
  /** legacy compatibility: no longer used by the single-ticker strategy model */
  inverse_leveraged_symbol_name: string
  /** legacy compatibility: no longer used by the single-ticker strategy model */
  base_symbols: string[]
  /** legacy compatibility: no longer used by the single-ticker strategy model */
  base_symbol_names: Record<string, string>
  /** legacy compatibility: no longer used by the single-ticker strategy model */
  base_symbol_roles?: Record<string, LeveragedTrendHoldBaseRole>
  quantity: number
  inverse_quantity: number
  /** 해외 주식 여부. true이면 가격 단위 = USD */
  is_overseas: boolean
}

// ─── 전략 ──────────────────────────────────────────────────────────
export interface StrategyView {
  id: string
  name: string
  enabled: boolean
  brokerId: BrokerId
  brokerAccountId: string | null
  targetSymbols: string[]
  /** 종목코드 → 종목명 맵 (StockStore 조회 결과) */
  targetSymbolNames: Record<string, string>
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
  /** KIS API 진단 로그: true 시 요청 파라미터·응답 전체를 로그에 기록 */
  api_debug: boolean
}

export interface SetLogConfigInput {
  retention_days: number
  max_size_mb: number
  api_debug: boolean
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
  date: string   // YYYYMMDD 또는 provider intraday timestamp
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
  /** 서버가 탐색한 dist/ 디렉터리 절대 경로 */
  distPath?: string
  /** dist/index.html 존재 여부 */
  distFound?: boolean
}

// ─── 리스크 관리 ───────────────────────────────────────────────────
export interface RiskConfigView {
  /** 리스크 관리 활성화 여부 */
  enabled: boolean
  /** 일일 최대 순손실 한도 (원) */
  dailyLossLimit: number
  /** 단일 종목 최대 비중 (0.0~1.0) */
  maxPositionRatio: number
  /** 전략/종목별 일일 매수 주문 제한. 0이면 제한 없음. */
  maxDailyBuyOrdersPerSymbol: number
  /** 전략/종목별 일일 매도 주문 제한. 0이면 제한 없음. */
  maxDailySellOrdersPerSymbol: number
  /** 전략/종목별 연속 손실 차단 기준. 0이면 제한 없음. */
  maxConsecutiveLossesPerStrategySymbol: number
  /** ATR 기반 주문 수량 산정 활성화 여부 */
  volatilitySizingEnabled: boolean
  /** 거래당 허용 위험 한도(bps). 100 = 1%. */
  riskPerTradeBps: number
  /** ATR 손절폭 배수 */
  atrStopMultiplier: number
  /** ATR이 준비된 종목 수 */
  atrSymbolCount: number
  /** 현재 연속 손실로 신규 진입이 차단된 전략/종목 조합 수 */
  blockedStrategySymbolCount: number
  /** 오늘 누적 총 손실 (음수) */
  currentLoss: number
  /** 오늘 누적 총 수익 (양수) */
  dailyProfit: number
  /** 순손실 = 총손실 - 당일수익 (양수 = 순손실) */
  netLoss: number
  /** 순손실 소진율 (0.0~1.0+) */
  lossRatio: number
  /** 비상 정지 여부 */
  isEmergencyStop: boolean
  /** 추가 거래 가능 여부 */
  canTrade: boolean
}

export interface UpdateRiskConfigInput {
  /** 리스크 관리 활성화 여부 */
  enabled?: boolean
  dailyLossLimit?: number
  /** 0.01 ~ 1.0 */
  maxPositionRatio?: number
  /** 전략/종목별 일일 매수 주문 제한. 0이면 제한 없음. */
  maxDailyBuyOrdersPerSymbol?: number
  /** 전략/종목별 일일 매도 주문 제한. 0이면 제한 없음. */
  maxDailySellOrdersPerSymbol?: number
  /** 전략/종목별 연속 손실 차단 기준. 0이면 제한 없음. */
  maxConsecutiveLossesPerStrategySymbol?: number
  /** ATR 기반 주문 수량 산정 활성화 여부 */
  volatilitySizingEnabled?: boolean
  /** 거래당 허용 위험 한도(bps). 0이면 고정 수량 유지. */
  riskPerTradeBps?: number
  /** ATR 손절폭 배수 */
  atrStopMultiplier?: number
}

// ─── 미체결 주문 ───────────────────────────────────────────────────
export interface PendingOrderView {
  odno: string
  symbol: string
  symbolName: string
  /** "buy" | "sell" */
  side: string
  status: 'pending' | 'partially_filled' | 'filled' | 'cancelled' | 'failed'
  quantity: number
  filledQuantity: number
  remainingQuantity: number
  timestamp: string
  signalReason: string
  provider?: string | null
  providerOrderId?: string | null
  providerRequestId?: string | null
  providerTrId?: string | null
}

// ─── 종목 목록 통계 ──────────────────────────────────────────────
export interface StockListStats {
  /** 저장된 종목 수 */
  count: number
  /** 마지막 upsert 시각 (ISO8601), 없으면 null */
  lastUpdatedAt: string | null
  /** stocklist.json 절대 경로 (디버그용) */
  filePath: string
  /** 자동 갱신 간격 (시간, 0 = 수동 전용) */
  updateIntervalHours: number
}

// ─── 체결 기록 보관 설정 ─────────────────────────────────────────
export interface TradeArchiveConfig {
  retention_days: number
  max_size_mb: number
}

export interface SetTradeArchiveConfigInput {
  retention_days: number
  max_size_mb: number
}

export interface TradeArchiveStats {
  /** 전체 trades.json 파일 수 */
  total_files: number
  /** 전체 저장 용량 (바이트) */
  size_bytes: number
  /** 가장 오래된 날짜 (YYYY-MM-DD), 없으면 null */
  oldest_date: string | null
  /** 가장 최근 날짜 (YYYY-MM-DD), 없으면 null */
  newest_date: string | null
}

// ─── 데이터 갱신 주기 설정 ──────────────────────────────────────────────
export interface RefreshConfig {
  /** 갱신 주기(초), 기본 30, 최소 5, 최대 3600 */
  interval_sec: number
}
