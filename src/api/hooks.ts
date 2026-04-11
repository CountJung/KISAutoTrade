/**
 * TanStack Query + Tauri IPC 훅
 *
 * 사용 예시:
 *   const { data: balance } = useBalance()
 *   const { mutate: order } = usePlaceOrder()
 */
import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseQueryOptions,
} from '@tanstack/react-query'

import { POLL_INTERVALS, ORDER_REFETCH_DELAY_MS } from '../scheduler'
import * as cmd from './commands'
import type {
  AccountProfileView,
  AddProfileInput,
  AppConfigView,
  BalanceResult,
  OverseasBalanceResult,
  ChartCandle,
  ConfigDiagnostic,
  DailyStats,
  ExecutedOrder,
  FrontendLogInput,
  LogConfig,
  OverseasPriceResponse,
  PlaceOrderInput,
  PlaceOverseasOrderInput,
  PositionView,
  PriceResponse,
  SetLogConfigInput,
  StockSearchItem,
  AppLogEntry,
  StrategyView,
  TradeRecord,
  TradingStatus,
  UpdateInfo,
  UpdateProfileInput,
  UpdateStrategyInput,
  WebConfig,
  DetectTradingTypeResult,
  RiskConfigView,
  UpdateRiskConfigInput,
  PendingOrderView,
  CmdError,
  TradeArchiveConfig,
  SetTradeArchiveConfigInput,
  TradeArchiveStats,
} from './types'

// ─── Query Keys ────────────────────────────────────────────────────
export const KEYS = {
  appConfig: ['appConfig'] as const,
  checkConfig: ['checkConfig'] as const,
  profiles: ['profiles'] as const,
  tradingStatus: ['tradingStatus'] as const,
  positions: ['positions'] as const,
  strategies: ['strategies'] as const,
  balance: ['balance'] as const,
  overseasBalance: ['overseasBalance'] as const,
  price: (symbol: string) => ['price', symbol] as const,
  todayExecuted: ['todayExecuted'] as const,
  todayTrades: ['todayTrades'] as const,
  tradeRange: (from: string, to: string) => ['trades', from, to] as const,
  todayStats: ['todayStats'] as const,
  statsRange: (from: string, to: string) => ['stats', from, to] as const,
  logConfig: ['logConfig'] as const,
  chartData: (symbol: string, presetKey: string) => ['chartData', symbol, presetKey] as const,
  stockSearch: (q: string) => ['stockSearch', q] as const,
  kisExecuted: (from: string, to: string) => ['kisExecuted', from, to] as const,
  recentLogs: ['recentLogs'] as const,
  updateCheck: ['updateCheck'] as const,
  webConfig: ['webConfig'] as const,
  overseasPrice: (exchange: string, symbol: string) => ['overseasPrice', exchange, symbol] as const,
  overseasChart: (exchange: string, symbol: string, presetKey: string) => ['overseasChart', exchange, symbol, presetKey] as const,
  riskConfig: ['riskConfig'] as const,
  pendingOrders: ['pendingOrders'] as const,
  tradeArchiveConfig: ['tradeArchiveConfig'] as const,
  tradeArchiveStats: ['tradeArchiveStats'] as const,
  exchangeRate: ['exchangeRate'] as const,
  refreshInterval: ['refreshInterval'] as const,
}

// ─── 앱 설정 ───────────────────────────────────────────────────────
export function useAppConfig(
  options?: Partial<UseQueryOptions<AppConfigView>>
) {
  return useQuery({
    queryKey: KEYS.appConfig,
    queryFn: cmd.getAppConfig,
    staleTime: Infinity,
    ...options,
  })
}

// ─── 잔고 ──────────────────────────────────────────────────────────
export function useBalance(
  options?: Partial<UseQueryOptions<BalanceResult>>
) {
  return useQuery({
    queryKey: KEYS.balance,
    queryFn: cmd.getBalance,
    staleTime: 30_000,
    refetchInterval: POLL_INTERVALS.SLOW, // 60s — 스케쥴러 기준
    ...options,
  })
}

export function useOverseasBalance(
  options?: Partial<UseQueryOptions<OverseasBalanceResult>>
) {
  return useQuery({
    queryKey: KEYS.overseasBalance,
    queryFn: cmd.getOverseasBalance,
    staleTime: 30_000,
    refetchInterval: POLL_INTERVALS.SLOW, // 60s
    ...options,
  })
}

// ─── 차트 데이터 ──────────────────────────────────────────────────────
/** symbol 6자리 + 프리셋 키 보유 시 자동 페치 */
export function useChartData(
  symbol: string,
  periodCode: string,
  startDate: string,
  endDate: string,
  presetKey: string,
  options?: Partial<UseQueryOptions<ChartCandle[]>>
) {
  return useQuery({
    queryKey: KEYS.chartData(symbol, presetKey),
    queryFn: () =>
      cmd.getChartData({
        symbol,
        period_code: periodCode,
        start_date: startDate,
        end_date: endDate,
      }),
    enabled: symbol.length === 6,
    staleTime: 60_000,
    gcTime: 5 * 60_000,
    ...options,
  })
}

// ─── 현재가 ────────────────────────────────────────────────────────
export function usePrice(
  symbol: string,
  options?: Partial<UseQueryOptions<PriceResponse>>
) {
  return useQuery({
    queryKey: KEYS.price(symbol),
    queryFn: () => cmd.getPrice(symbol),
    enabled: !!symbol,
    staleTime: 5_000,
    refetchInterval: POLL_INTERVALS.FAST, // 10s — 스케쥴러 기준
    ...options,
  })
}

// ─── 주문 ──────────────────────────────────────────────────────────
export function usePlaceOrder() {
  const qc = useQueryClient()

  return useMutation({
    mutationFn: (input: PlaceOrderInput) => cmd.placeOrder(input),
    onSuccess: () => {
      // 잔고·로컬 체결 기록은 즉시 무효화
      void qc.invalidateQueries({ queryKey: KEYS.balance })
      void qc.invalidateQueries({ queryKey: KEYS.todayTrades })
      // KIS 서버 처리 딜레이 후 체결 내역 갱신 (즉시 재조회 시 미반영 가능)
      setTimeout(
        () => void qc.invalidateQueries({ queryKey: KEYS.todayExecuted }),
        ORDER_REFETCH_DELAY_MS.REAL,
      )
    },
  })
}

// ─── 당일 체결 내역 (KIS 서버) ────────────────────────────────────
export function useTodayExecuted(
  options?: Partial<UseQueryOptions<ExecutedOrder[]>>
) {
  return useQuery({
    queryKey: KEYS.todayExecuted,
    queryFn: cmd.getTodayExecuted,
    staleTime: 15_000,
    refetchInterval: POLL_INTERVALS.NORMAL, // 30s — 스케쥴러 기준
    refetchOnWindowFocus: true,
    placeholderData: [],
    ...options,
  })
}

// ─── 로컬 체결 기록 ────────────────────────────────────────────────
export function useTodayTrades(
  options?: Partial<UseQueryOptions<TradeRecord[]>>
) {
  return useQuery({
    queryKey: KEYS.todayTrades,
    queryFn: cmd.getTodayTrades,
    staleTime: 15_000,
    ...options,
  })
}

export function useTradesByRange(
  from: string,
  to: string,
  options?: Partial<UseQueryOptions<TradeRecord[]>>
) {
  return useQuery({
    queryKey: KEYS.tradeRange(from, to),
    queryFn: () => cmd.getTradesByRange(from, to),
    enabled: !!from && !!to,
    staleTime: 60_000,
    ...options,
  })
}

// ─── 통계 ──────────────────────────────────────────────────────────
export function useTodayStats(
  options?: Partial<UseQueryOptions<DailyStats>>
) {
  return useQuery({
    queryKey: KEYS.todayStats,
    queryFn: cmd.getTodayStats,
    staleTime: 30_000,
    refetchInterval: 60_000,
    ...options,
  })
}

export function useStatsByRange(
  from: string,
  to: string,
  options?: Partial<UseQueryOptions<DailyStats[]>>
) {
  return useQuery({
    queryKey: KEYS.statsRange(from, to),
    queryFn: () => cmd.getStatsByRange(from, to),
    enabled: !!from && !!to,
    staleTime: 300_000,
    ...options,
  })
}

// ─── Discord 테스트 ────────────────────────────────────────────────
export function useSendTestDiscord() {
  return useMutation({
    mutationFn: cmd.sendTestDiscord,
  })
}

// ─── 진단 모드 ───────────────────────────────────────────────
export function useCheckConfig(
  options?: Partial<UseQueryOptions<ConfigDiagnostic>>
) {
  return useQuery({
    queryKey: KEYS.checkConfig,
    queryFn: cmd.checkConfig,
    staleTime: 30_000,
    ...options,
  })
}
// ─── 자동 매매 상태 ─────────────────────────────────────────────────
export function useTradingStatus(
  options?: Partial<UseQueryOptions<TradingStatus>>
) {
  return useQuery({
    queryKey: KEYS.tradingStatus,
    queryFn: cmd.getTradingStatus,
    staleTime: 5_000,
    refetchInterval: POLL_INTERVALS.FAST, // 10s — 스케쥴러 기준
    ...options,
  })
}

export function useStartTrading() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: cmd.startTrading,
    onSuccess: (data) => {
      qc.setQueryData(KEYS.tradingStatus, data)
    },
  })
}

export function useStopTrading() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: cmd.stopTrading,
    onSuccess: (data) => {
      qc.setQueryData(KEYS.tradingStatus, data)
    },
  })
}

// ─── 포지션 ────────────────────────────────────────────────────────
export function usePositions(
  options?: Partial<UseQueryOptions<PositionView[]>>
) {
  return useQuery({
    queryKey: KEYS.positions,
    queryFn: cmd.getPositions,
    staleTime: 10_000,
    refetchInterval: POLL_INTERVALS.NORMAL, // 30s — 스케쥴러 기준
    ...options,
  })
}

// ─── 전략 ──────────────────────────────────────────────────────────
export function useStrategies(
  options?: Partial<UseQueryOptions<StrategyView[]>>
) {
  return useQuery({
    queryKey: KEYS.strategies,
    queryFn: cmd.getStrategies,
    staleTime: Infinity,
    ...options,
  })
}

export function useUpdateStrategy() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: UpdateStrategyInput) => cmd.updateStrategy(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: KEYS.strategies })
    },
  })
}

// ─── 계좌 프로파일 ──────────────────────────────────────────────────
export function useProfiles(
  options?: Partial<UseQueryOptions<AccountProfileView[]>>
) {
  return useQuery({
    queryKey: KEYS.profiles,
    queryFn: cmd.listProfiles,
    staleTime: Infinity,
    ...options,
  })
}

export function useAddProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: AddProfileInput) => cmd.addProfile(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.appConfig })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
    },
  })
}

export function useUpdateProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: UpdateProfileInput) => cmd.updateProfile(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.appConfig })
    },
  })
}

export function useDeleteProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => cmd.deleteProfile(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.appConfig })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
    },
  })
}

export function useSetActiveProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => cmd.setActiveProfile(id),
    onSuccess: (newConfig) => {
      qc.setQueryData(KEYS.appConfig, newConfig)
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
      // 프로파일 전환 시 잔고 캠시 무효화
      void qc.invalidateQueries({ queryKey: KEYS.balance })
    },
  })
}

/** APP KEY + APP SECRET으로 실전/모의투자를 자동 감지합니다. */
export function useDetectTradingType() {
  return useMutation<DetectTradingTypeResult, Error, { appKey: string; appSecret: string }>({
    mutationFn: ({ appKey, appSecret }) => cmd.detectTradingType(appKey, appSecret),
  })
}

/** 저장된 프로파일 키로 실전/모의 감지 후 즉시 저장합니다. */
export function useDetectProfileTradingType() {
  const qc = useQueryClient()
  return useMutation<AccountProfileView, Error, string>({
    mutationFn: (profileId) => cmd.detectProfileTradingType(profileId),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.appConfig })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
    },
  })
}

// ─── 로그 설정 ─────────────────────────────────────────────────────
export function useLogConfig(
  options?: Partial<UseQueryOptions<LogConfig>>
) {
  return useQuery({
    queryKey: KEYS.logConfig,
    queryFn: cmd.getLogConfig,
    staleTime: Infinity,
    ...options,
  })
}

export function useSetLogConfig() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: SetLogConfigInput) => cmd.setLogConfig(input),
    onSuccess: (newCfg) => {
      qc.setQueryData(KEYS.logConfig, newCfg)
    },
  })
}

// ─── 프론트엔드 로그 기록 ─────────────────────────────────────────
export function useWriteFrontendLog() {
  return useMutation({
    mutationFn: (input: FrontendLogInput) => cmd.writeFrontendLog(input),
  })
}

// ─── 종목 검색 ─────────────────────────────────────────────────────
/** query 2자 이상인 경우 KIS/로컬 검색 (stock_list 비어있으면 STOCK_LIST_EMPTY 에러) */
export function useStockSearch(query: string) {
  return useQuery<StockSearchItem[], CmdError>({
    queryKey: KEYS.stockSearch(query),
    queryFn: () => cmd.searchStock(query),
    enabled: query.length >= 2,
    staleTime: 30_000,
    placeholderData: [],
    // STOCK_LIST_EMPTY는 재시도 불필요 (사용자가 수동으로 새로고침 필요)
    retry: (count, err) => {
      if ((err as CmdError | null)?.code === 'STOCK_LIST_EMPTY') return false
      return count < 2
    },
  })
}

// ─── 종목 목록 새로고침 ──────────────────────────────────────────────
export function useRefreshStockList() {
  const qc = useQueryClient()
  return useMutation<number, CmdError>({
    mutationFn: () => cmd.refreshStockList(),
    onSuccess: () => {
      // 검색 캐시 + 통계 캐시 전체 무효화
      qc.invalidateQueries({ queryKey: ['stockSearch'] })
      qc.invalidateQueries({ queryKey: ['stockListStats'] })
    },
    onError: (err) => {
      // KRX_EMPTY: KRX 다운로드가 0개 반환 — NAVER 실시간 검색으로 폴백 동작 중
      if (err.code !== 'KRX_EMPTY') {
        console.warn('[useRefreshStockList]', err.code, err.message)
      }
    },
  })
}

// ─── 종목 목록 통계 ──────────────────────────────────────────────────
export function useStockListStats() {
  return useQuery({
    queryKey: ['stockListStats'],
    queryFn: () => cmd.getStockListStats(),
    staleTime: 30_000,
  })
}

// ─── 종목 목록 갱신 간격 변경 ──────────────────────────────────────
export function useSetStockUpdateInterval() {
  const qc = useQueryClient()
  return useMutation<void, CmdError, number>({
    mutationFn: (hours: number) => cmd.setStockUpdateInterval(hours),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['stockListStats'] }),
  })
}

// ─── KIS 기간별 체결 내역 ──────────────────────────────────────────
export function useKisExecutedByRange(
  from: string,
  to: string,
  options?: Partial<UseQueryOptions<ExecutedOrder[]>>
) {
  return useQuery({
    queryKey: KEYS.kisExecuted(from, to),
    queryFn: () => cmd.getKisExecutedByRange(from, to),
    enabled: !!from && !!to,
    staleTime: 30_000,
    ...options,
  })
}

// ─── 최근 앱 로그 (파일 기반) ─────────────────────────────────────
export function useRecentLogs(count = 200) {
  return useQuery<AppLogEntry[]>({
    queryKey: KEYS.recentLogs,
    queryFn: () => cmd.getRecentLogs(count),
    refetchInterval: 3_000,
    staleTime: 0,
    placeholderData: [],
  })
}

/**
 * 프론트엔드 로거 — 일반 함수 형태 (훅 외부에서 직접 호출 가능)
 * Tauri 환경에서만 동작하며, 실패해도 콘솔만 출력하고 에러 전파 안 함
 */
export const frontendLogger = {
  error: (message: string, context?: string) => {
    console.error(`[${context ?? 'frontend'}] ${message}`)
    cmd.writeFrontendLog({ level: 'error', message, context }).catch(() => {})
  },
  warn: (message: string, context?: string) => {
    console.warn(`[${context ?? 'frontend'}] ${message}`)
    cmd.writeFrontendLog({ level: 'warn', message, context }).catch(() => {})
  },
  info: (message: string, context?: string) => {
    console.info(`[${context ?? 'frontend'}] ${message}`)
    cmd.writeFrontendLog({ level: 'info', message, context }).catch(() => {})
  },
  debug: (message: string, context?: string) => {
    console.debug(`[${context ?? 'frontend'}] ${message}`)
    cmd.writeFrontendLog({ level: 'debug', message, context }).catch(() => {})
  },
}

// ─── 업데이트 확인 ────────────────────────────────────────────────
export function useUpdateCheck() {
  return useQuery<UpdateInfo>({
    queryKey: KEYS.updateCheck,
    queryFn: cmd.checkForUpdate,
    // 앱 시작 시 한 번만 확인. 성공하면 24시간 동안 재요청 안 함
    staleTime: 1000 * 60 * 60 * 24,
    retry: 1,
    // 실패해도 UI 먹스에 영향 없음
    throwOnError: false,
  })
}

// ─── 웹 접속 설정 ─────────────────────────────────────────────────
export function useWebConfig() {
  return useQuery<WebConfig>({
    queryKey: KEYS.webConfig,
    queryFn: cmd.getWebConfig,
    staleTime: Infinity,
  })
}

export function useSaveWebConfig() {
  const qc = useQueryClient()
  return useMutation<string, Error, number>({
    mutationFn: (newPort) => cmd.saveWebConfig(newPort),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: KEYS.webConfig })
    },
  })
}

// ─── 해외(미국) 차트 데이터 ──────────────────────────────────────────
const OVERSEAS_CHART_PRESETS: Record<string, { periodCode: string; months: number }> = {
  '1M': { periodCode: 'D', months: 1  },
  '3M': { periodCode: 'D', months: 3  },
  '6M': { periodCode: 'W', months: 6  },
  '1Y': { periodCode: 'W', months: 12 },
  '5Y': { periodCode: 'M', months: 60 },
}

export function useOverseasChartData(
  symbol: string,
  exchange: string,
  presetKey: string = '3M',
  options?: Partial<UseQueryOptions<ChartCandle[]>>
) {
  const cfg = OVERSEAS_CHART_PRESETS[presetKey] ?? OVERSEAS_CHART_PRESETS['3M']
  const baseDate = (() => {
    const d = new Date()
    d.setMonth(d.getMonth() - cfg.months)
    return d.toISOString().slice(0, 10).replace(/-/g, '')
  })()

  return useQuery({
    queryKey: KEYS.overseasChart(exchange, symbol, presetKey),
    queryFn: () =>
      cmd.getOverseasChartData(symbol, exchange, cfg.periodCode, baseDate),
    enabled: !!symbol && !!exchange,
    staleTime: 3 * 60_000,
    gcTime: 10 * 60_000,
    retry: false,
    ...options,
  })
}

// ─── 해외(미국) 현재가 ───────────────────────────────────────────────
export function useOverseasPrice(
  symbol: string,
  exchange: string,
  options?: Partial<UseQueryOptions<OverseasPriceResponse>>
) {
  return useQuery({
    queryKey: KEYS.overseasPrice(exchange, symbol),
    queryFn: () => cmd.getOverseasPrice(symbol, exchange),
    enabled: !!symbol && !!exchange,
    staleTime: 10_000,
    refetchInterval: 15_000,
    retry: false,
    ...options,
  })
}

// ─── 해외 주문 (지정가 한정) ────────────────────────────────────────
/** 해외 주문 — 지정가만 지원, USD 단위 가격 입력 */
export function usePlaceOverseasOrder() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: PlaceOverseasOrderInput) => cmd.placeOverseasOrder(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.balance })
      // 해외 주문 체결 후 해외 잔고도 갱신
      void qc.invalidateQueries({ queryKey: KEYS.overseasBalance })
      // KIS 서버 처리 딜레이 후 체결 내역 갱신
      setTimeout(
        () => void qc.invalidateQueries({ queryKey: KEYS.todayExecuted }),
        ORDER_REFETCH_DELAY_MS.REAL,
      )
    },
  })
}

// ─── 리스크 관리 ───────────────────────────────────────────────────
export function useRiskConfig(options?: Partial<UseQueryOptions<RiskConfigView>>) {
  return useQuery<RiskConfigView>({
    queryKey: KEYS.riskConfig,
    queryFn: cmd.getRiskConfig,
    staleTime: 10_000,
    refetchInterval: POLL_INTERVALS.FAST, // 10s — 실시간 리스크 반영
    ...options,
  })
}

export function useUpdateRiskConfig() {
  const qc = useQueryClient()
  return useMutation<RiskConfigView, Error, UpdateRiskConfigInput>({
    mutationFn: (input) => cmd.updateRiskConfig(input),
    onSuccess: (data) => {
      qc.setQueryData(KEYS.riskConfig, data)
    },
  })
}

export function useClearEmergencyStop() {
  const qc = useQueryClient()
  return useMutation<RiskConfigView, Error, void>({
    mutationFn: () => cmd.clearEmergencyStop(),
    onSuccess: (data) => {
      qc.setQueryData(KEYS.riskConfig, data)
    },
  })
}

export function useActivateEmergencyStop() {
  const qc = useQueryClient()
  return useMutation<RiskConfigView, Error, void>({
    mutationFn: () => cmd.activateEmergencyStop(),
    onSuccess: (data) => {
      qc.setQueryData(KEYS.riskConfig, data)
    },
  })
}

/** 잔고 부족 매수 정지를 수동으로 해제 */
export function useClearBuySuspension() {
  const qc = useQueryClient()
  return useMutation<TradingStatus, Error, void>({
    mutationFn: () => cmd.clearBuySuspension(),
    onSuccess: (data) => {
      qc.setQueryData(KEYS.tradingStatus, data)
    },
  })
}

// ─── 환율 / 공통 갱신 주기 ──────────────────────────────────────
/**
 * 공통 데이터 갱신 주기 조회 (초)
 * REFRESH_INTERVAL_SEC 환경변수를 Rust에서 읽어 반환합니다.
 * 시작 시 1회만 페치하여 다음 지점까지 캐시(staleTime=Infinity)합니다.
 */
export function useRefreshInterval() {
  return useQuery<number>({
    queryKey: KEYS.refreshInterval,
    queryFn: cmd.getRefreshInterval,
    staleTime: Infinity,
    placeholderData: 30,
  })
}

/**
 * USD/KRW 환율 조회
 * REFRESH_INTERVAL_SEC마다 Rust 백그라운드에서 자동 갱신된 캐시 값을 반환합니다.
 * KRW 모드 없으면 기본값 1450원 사용합니다.
 */
export function useExchangeRate() {
  const { data: intervalSec = 30 } = useRefreshInterval()
  return useQuery<number>({
    queryKey: KEYS.exchangeRate,
    queryFn: cmd.getExchangeRate,
    staleTime: intervalSec * 900,  // 90% 주기도다 짧게 stale 표시
    refetchInterval: intervalSec * 1000,
    placeholderData: 1450,
  })
}

export function usePendingOrders(options?: Partial<UseQueryOptions<PendingOrderView[]>>) {
  return useQuery<PendingOrderView[]>({
    queryKey: KEYS.pendingOrders,
    queryFn: cmd.getPendingOrders,
    staleTime: 5_000,
    refetchInterval: POLL_INTERVALS.PENDING, // 5s — 스케쥴러 기준
    placeholderData: [],
    ...options,
  })
}
// ─── 체결 기록 보관 설정 ────────────────────────────────────────────────────
export function useTradeArchiveConfig(
  options?: Partial<UseQueryOptions<TradeArchiveConfig>>
) {
  return useQuery<TradeArchiveConfig>({
    queryKey: KEYS.tradeArchiveConfig,
    queryFn: cmd.getTradeArchiveConfig,
    staleTime: Infinity,
    ...options,
  })
}

export function useSetTradeArchiveConfig() {
  const qc = useQueryClient()
  return useMutation<TradeArchiveConfig, Error, SetTradeArchiveConfigInput>({
    mutationFn: (input) => cmd.setTradeArchiveConfig(input),
    onSuccess: (newCfg) => {
      qc.setQueryData(KEYS.tradeArchiveConfig, newCfg)
      // 설정 변경 후 통계 새로고침
      qc.invalidateQueries({ queryKey: KEYS.tradeArchiveStats })
    },
  })
}

export function useTradeArchiveStats(
  options?: Partial<UseQueryOptions<TradeArchiveStats>>
) {
  return useQuery<TradeArchiveStats>({
    queryKey: KEYS.tradeArchiveStats,
    queryFn: cmd.getTradeArchiveStats,
    staleTime: 30_000,
    ...options,
  })
}
