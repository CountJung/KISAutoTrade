import { useMutation, useQuery, useQueryClient, type UseQueryOptions } from '@tanstack/react-query'

import { POLL_INTERVALS } from '../scheduler'
import * as cmd from './commands'
import { KEYS } from './queryKeys'
import type {
  DailyStats,
  ExecutedOrder,
  OverseasExecutedOrder,
  TradeRecord,
  TradeArchiveConfig,
  SetTradeArchiveConfigInput,
  TradeArchiveStats,
} from './types'

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

export function useTodayOverseasExecuted(
  options?: Partial<UseQueryOptions<OverseasExecutedOrder[]>>
) {
  return useQuery({
    queryKey: KEYS.todayOverseasExecuted,
    queryFn: cmd.getTodayOverseasExecuted,
    staleTime: 15_000,
    refetchInterval: POLL_INTERVALS.NORMAL,
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
  // 오늘 날짜 범위이면 NORMAL(30s) 주기로 자동 갱신, 과거 기간은 staleTime 무제한
  const todayStr = new Date().toISOString().slice(0, 10)
  const isToday = to === todayStr
  return useQuery({
    queryKey: KEYS.tradeRange(from, to),
    queryFn: () => cmd.getTradesByRange(from, to),
    enabled: !!from && !!to,
    staleTime: isToday ? 20_000 : Infinity,
    refetchInterval: isToday ? POLL_INTERVALS.NORMAL : false,
    refetchOnWindowFocus: isToday,
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

export function useOverseasExecutedByRange(
  from: string,
  to: string,
  options?: Partial<UseQueryOptions<OverseasExecutedOrder[]>>
) {
  return useQuery({
    queryKey: KEYS.overseasExecuted(from, to),
    queryFn: () => cmd.getOverseasExecutedByRange(from, to),
    enabled: !!from && !!to,
    staleTime: 30_000,
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
