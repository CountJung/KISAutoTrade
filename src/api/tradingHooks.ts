import { useMutation, useQuery, useQueryClient, type UseQueryOptions } from '@tanstack/react-query'

import { POLL_INTERVALS } from '../scheduler'
import * as cmd from './commands'
import { KEYS } from './queryKeys'
import type {
  PositionView,
  TradingStatus,
  RiskConfigView,
  UpdateRiskConfigInput,
} from './types'

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
