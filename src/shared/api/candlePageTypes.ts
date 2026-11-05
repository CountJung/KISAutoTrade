import type { BrokerId, ChartCandle } from './types'

export type CandleInterval = 'D' | 'W' | 'M' | '1m'

export interface CandlePageInput {
  symbol: string
  brokerId: BrokerId
  isOverseas: boolean
  interval: CandleInterval
  count?: number
  exchange?: string
  cursor?: string
  expectedProfileId: string
  expectedBrokerAccountId?: string
}

/** One scoped provider page. Cursor is opaque and must be passed unchanged. */
export interface CandlePageView {
  candles: ChartCandle[]
  nextCursor: string | null
  endReason: string | null
  brokerId: BrokerId
  profileId: string
  scopeKey: string
  symbol: string
  interval: CandleInterval
  exchange: string | null
  source: string
  timestampSemantics: 'tradingDate' | 'barClose'
  completionPolicy: 'strictlyBeforeCurrentMarketPeriod' | 'barCloseAtOrBeforeAsOf'
  asOf: string
  excludedUnfinishedBars: number
  excludedInvalidBars: number
}
