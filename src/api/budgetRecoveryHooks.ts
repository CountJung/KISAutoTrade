import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { approveBudgetRecovery, getBudgetRecovery } from '../shared/api/budgetRecoveryCommands'
import type { ApproveBudgetRecoveryInput, BudgetRecoveryView } from '../shared/api/budgetRecoveryTypes'
import type { AutoTradingBudgetInput, CmdError } from './types'
import { KEYS } from './queryKeys'

export interface RecoveryIdentity {
  profileId: string | null
  isPaper: boolean
}

export function useBudgetRecovery(scope: AutoTradingBudgetInput | undefined, identity: RecoveryIdentity, enabled: boolean) {
  return useQuery<BudgetRecoveryView, CmdError>({
    queryKey: KEYS.budgetRecovery(scope?.brokerId, scope?.brokerAccountId, identity.profileId, identity.isPaper),
    queryFn: () => getBudgetRecovery(scope!),
    enabled: enabled && Boolean(scope?.brokerAccountId),
    staleTime: 0,
    // Review explicitly; a background refresh must not silently replace an approval draft.
    refetchOnWindowFocus: false,
  })
}

export function useApproveBudgetRecovery() {
  const qc = useQueryClient()
  return useMutation<BudgetRecoveryView, CmdError, ApproveBudgetRecoveryInput>({
    mutationFn: approveBudgetRecovery,
    onSuccess: (_data, input) => {
      // Invalidate the original scope only. Never publish a late result into the selected profile.
      void qc.invalidateQueries({ queryKey: ['budgetRecovery', input.brokerId, input.brokerAccountId] })
      void qc.invalidateQueries({ queryKey: KEYS.autoTradingBudget(input.brokerId, input.brokerAccountId) })
      void qc.invalidateQueries({ queryKey: KEYS.pendingOrders })
    },
  })
}
