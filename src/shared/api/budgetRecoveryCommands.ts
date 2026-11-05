import { invoke } from './transport'
import type { AutoTradingBudgetInput } from './types'
import type { ApproveBudgetRecoveryInput, BudgetRecoveryView } from './budgetRecoveryTypes'

// Desktop IPC only. The unauthenticated web server has no recovery routes.
export const getBudgetRecovery = (input: AutoTradingBudgetInput) =>
  invoke<BudgetRecoveryView>('get_budget_recovery', { input })

export const approveBudgetRecovery = (input: ApproveBudgetRecoveryInput) =>
  invoke<BudgetRecoveryView>('approve_budget_recovery', { input })
