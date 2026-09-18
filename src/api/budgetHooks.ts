import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { getAutoTradingBudget, updateAutoTradingBudget } from './commands'
import { KEYS } from './queryKeys'
import { POLL_INTERVALS } from '../scheduler'
import type { AutoTradingBudgetInput, AutoTradingBudgetView, CmdError, UpdateAutoTradingBudgetInput } from './types'

export function useAutoTradingBudget(input: AutoTradingBudgetInput | undefined) {
  return useQuery<AutoTradingBudgetView, CmdError>({
    queryKey: KEYS.autoTradingBudget(input?.brokerId, input?.brokerAccountId),
    queryFn: () => getAutoTradingBudget(input!),
    enabled: Boolean(input?.brokerAccountId),
    staleTime: 5_000,
    refetchInterval: POLL_INTERVALS.FAST,
  })
}

export function useUpdateAutoTradingBudget() {
  const qc = useQueryClient()
  return useMutation<AutoTradingBudgetView, CmdError, UpdateAutoTradingBudgetInput>({
    mutationFn: updateAutoTradingBudget,
    onSuccess: (data, input) => {
      qc.setQueryData(KEYS.autoTradingBudget(input.brokerId, input.brokerAccountId), data)
    },
  })
}
