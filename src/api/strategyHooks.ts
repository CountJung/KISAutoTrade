import { useMutation, useQuery, useQueryClient, type UseQueryOptions } from '@tanstack/react-query'

import * as cmd from './commands'
import { KEYS } from './queryKeys'
import type {
  LeveragedTrendHoldPreviewInput,
  LeveragedTrendHoldPreviewView,
  StrategyPreviewInput,
  ContinuousStrategyPreviewInput,
  ContinuousStrategyPreviewView,
  StrategyPreviewView,
  StrategyView,
  UpdateStrategyInput,
  CmdError,
} from './types'

// ─── 전략 ──────────────────────────────────────────────────────────
export function useStrategies(
  scopeKey?: string | null,
  options?: Partial<UseQueryOptions<StrategyView[]>>
) {
  return useQuery({
    queryKey: KEYS.strategies(scopeKey),
    queryFn: cmd.getStrategies,
    enabled: Boolean(scopeKey),
    staleTime: Infinity,
    ...options,
  })
}

export function useUpdateStrategy() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: UpdateStrategyInput) => cmd.updateStrategy(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['strategies'] })
    },
  })
}

export function usePreviewLeveragedTrendHold() {
  return useMutation<LeveragedTrendHoldPreviewView, CmdError, LeveragedTrendHoldPreviewInput>({
    mutationFn: cmd.previewLeveragedTrendHold,
  })
}

export function usePreviewStrategy() {
  return useMutation<StrategyPreviewView, CmdError, StrategyPreviewInput>({
    mutationFn: cmd.previewStrategy,
  })
}


export function usePreviewStrategyContinuous() {
  return useMutation<ContinuousStrategyPreviewView, CmdError, ContinuousStrategyPreviewInput>({
    mutationFn: cmd.previewStrategyContinuous,
  })
}
