import type { AutoTradingBudgetInput } from './types'

/** Operator-attested binding; it is not a broker-confirmed fill or an instruction to trade. */
export interface BudgetRecoveryItem {
  reservationId: string
  currency: 'KRW' | 'USD'
  market: 'kr' | 'us' | null
  exchange: string | null
  symbol: string
  side: 'buy' | 'sell'
  quantity: number
  orderPrice: number
  filledQuantity: number
  filledNotional: number
  reservedAmount: number
  submittedAt: string | null
  profileId: string | null
  isPaper: boolean | null
  clientOrderId: string | null
  providerOrderId: string | null
  evidenceReference: string | null
  snapshotRevision: string | null
  state: 'unresolved' | 'awaitingReconciliation' | 'resolved'
  blockedReason: string | null
}

export interface BudgetRecoveryView {
  scope: { brokerId: 'kis' | 'toss'; accountId: string | null }
  items: BudgetRecoveryItem[]
  totalCount: number
}

export interface ApproveBudgetRecoveryInput extends AutoTradingBudgetInput {
  reservationId: string
  snapshotRevision: string
  providerOrderId: string
  clientOrderId: string | null
  evidenceReference: string
  confirmed: boolean
}
