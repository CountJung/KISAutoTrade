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

import * as cmd from './commands'
import type {
  AccountProfileView,
  AddProfileInput,
  AppConfigView,
  BalanceResult,
  ChartCandle,
  ConfigDiagnostic,
  DailyStats,
  ExecutedOrder,
  FrontendLogInput,
  LogConfig,
  PlaceOrderInput,
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
    refetchInterval: 60_000, // 1분마다 자동 갱신
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
    refetchInterval: 10_000, // 10초마다 자동 갱신
    ...options,
  })
}

// ─── 주문 ──────────────────────────────────────────────────────────
export function usePlaceOrder() {
  const qc = useQueryClient()

  return useMutation({
    mutationFn: (input: PlaceOrderInput) => cmd.placeOrder(input),
    onSuccess: () => {
      // 주문 성공 시 잔고 및 체결 내역 갱신
      void qc.invalidateQueries({ queryKey: KEYS.balance })
      void qc.invalidateQueries({ queryKey: KEYS.todayExecuted })
      void qc.invalidateQueries({ queryKey: KEYS.todayTrades })
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
    refetchInterval: 30_000,
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
    refetchInterval: 10_000,
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
    refetchInterval: 30_000,
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
/** query 2자 이상 & 숫자가 아닌 경우에만 KIS 검색 API 호출 */
export function useStockSearch(query: string) {
  return useQuery<StockSearchItem[]>({
    queryKey: KEYS.stockSearch(query),
    queryFn: () => cmd.searchStock(query),
    enabled: query.length >= 2 && !/^\d+$/.test(query),
    staleTime: 30_000,
    placeholderData: [],
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