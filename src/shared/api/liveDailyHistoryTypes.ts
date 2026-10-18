import type { BrokerId } from './types'

/** Read-only live daily context preparation. Ready does not imply an eligible order. */
export interface LiveDailyHistoryStatus {
  strategyId: string
  symbol: string
  brokerId: BrokerId
  profileId: string
  marketDate: string
  requiredBars: number
  availableBars: number
  state: string
  source: string | null
  lastCompletedDate: string | null
  lastError: string | null
  nextRetryAt: string | null
}
