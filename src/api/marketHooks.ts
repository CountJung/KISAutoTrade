import { useMutation, useQuery, useQueryClient, type UseQueryOptions } from '@tanstack/react-query'

import { POLL_INTERVALS } from '../scheduler'
import * as cmd from './commands'
import { KEYS } from './queryKeys'
import { useRefreshConfig } from './settingsHooks'
import type {
  ChartCandle,
  OverseasPriceResponse,
  PriceResponse,
  StockSearchItem,
  ExchangeRateView,
  CmdError,
  TossMarketCalendarView,
  TossMarketSnapshotView,
  TossStockSafetyView,
} from './types'

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

export function useTossChartData(
  symbol: string,
  interval: '1d' | '1m',
  presetKey: string,
  options?: Partial<UseQueryOptions<ChartCandle[]>>
) {
  return useQuery({
    queryKey: KEYS.tossChartData(symbol, interval, presetKey),
    queryFn: () => cmd.getTossChartData(symbol, interval, 200),
    enabled: !!symbol,
    staleTime: 60_000,
    gcTime: 5 * 60_000,
    retry: false,
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

// ─── Toss read-only 시세 snapshot ─────────────────────────────────
export function useTossMarketSnapshot(
  symbol: string,
  options?: Partial<UseQueryOptions<TossMarketSnapshotView>>
) {
  return useQuery({
    queryKey: KEYS.tossMarketSnapshot(symbol),
    queryFn: () => cmd.getTossMarketSnapshot(symbol),
    enabled: !!symbol,
    staleTime: 5_000,
    refetchInterval: POLL_INTERVALS.FAST,
    retry: false,
    ...options,
  })
}

export function useTossStockSafety(
  symbol: string,
  options?: Partial<UseQueryOptions<TossStockSafetyView>>
) {
  return useQuery({
    queryKey: KEYS.tossStockSafety(symbol),
    queryFn: () => cmd.getTossStockSafety(symbol),
    enabled: !!symbol,
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    retry: false,
    ...options,
  })
}

export function useTossMarketCalendar(
  options?: Partial<UseQueryOptions<TossMarketCalendarView>>
) {
  return useQuery({
    queryKey: KEYS.tossMarketCalendar,
    queryFn: cmd.getTossMarketCalendar,
    staleTime: 60_000,
    gcTime: 5 * 60_000,
    retry: false,
    ...options,
  })
}

/**
 * USD/KRW 환율 조회
 * refresh_config.interval_sec마다 Rust 백그라운드에서 자동 갱신된 캐시 값을 반환합니다.
 * KRW 모드 없으면 기본값 1450원 사용합니다.
 */
export function useExchangeRate() {
  const { data: cfg } = useRefreshConfig()
  const intervalSec = cfg?.interval_sec ?? 30
  return useQuery<number>({
    queryKey: KEYS.exchangeRate,
    queryFn: cmd.getExchangeRate,
    staleTime: intervalSec * 900,  // 90% 주기도다 짧게 stale 표시
    refetchInterval: intervalSec * 1000,
    placeholderData: 1450,
  })
}

/**
 * USD/KRW 환율 출처와 fallback 상태 조회
 */
export function useExchangeRateStatus() {
  const { data: cfg } = useRefreshConfig()
  const intervalSec = cfg?.interval_sec ?? 30
  return useQuery<ExchangeRateView>({
    queryKey: KEYS.exchangeRateStatus,
    queryFn: cmd.getExchangeRateStatus,
    staleTime: intervalSec * 900,
    refetchInterval: intervalSec * 1000,
  })
}
