/** Pure research DTOs mirrored from trading::sector_portfolio; no broker scope. */
export interface SectorPrice {
  adjustedClose: number
  open: number
  close: number
}

export interface SectorDay {
  date: string
  prices: Record<string, SectorPrice>
}

export interface SectorSource {
  name: string
  retrievedAt: string
  synthetic: boolean
  adjustmentPolicy: string
  calendarSource: string
  coverageVerified: boolean
  corporateActionsVerified: boolean
}

export interface SectorCorporateAction {
  symbol: string
  date: string
  kind: string
  payDate?: string | null
  dividendPerShare?: number | null
  splitRatio?: number | null
}

export interface SectorPortfolioInput {
  initialCashUsd?: number
  costBps?: number
  source: SectorSource
  history: SectorDay[]
  evaluation: SectorDay[]
  expectedSessionDates: string[]
  corporateActions?: SectorCorporateAction[]
}

export type SectorStrategy = 'base' | 'defensive' | 'aggressive' | 'spy' | 'cash'
export type DefensiveState = 'normal' | 'defensive'

export interface SectorHolding {
  symbol: string
  quantity: number
  averageCostUsd: number
  remainingBuyCostUsd: number
}

export interface SectorRank {
  symbol: string
  r5: number
  r20: number
  rs5: number
  rs20: number
  percentile5: number
  percentile20: number
  score: number
  eligible: boolean
}

export interface SectorTarget {
  symbol: string
  weight: number
  budgetUsd: number
  referenceWeight: number
}

export interface SectorPlan {
  strategy: SectorStrategy
  signalDate: string
  defensiveState: DefensiveState
  selectedSymbols: string[]
  targetWeight: number
  referenceEquityUsd: number
  rebalance: boolean
  reasons: string[]
  targets: SectorTarget[]
}

export interface SectorRankingDay {
  date: string
  signalDate: string
  spyR20: number
  previousSpyR20: number
  rankings: SectorRank[]
}

export interface SectorTrade {
  date: string
  symbol: string
  side: string
  requestedQuantity: number
  quantity: number
  priceUsd: number
  costUsd: number
  cashAfterUsd: number
  status: string
}

export interface SectorDaily {
  date: string
  plan: SectorPlan
  cashUsd: number
  equityUsd: number
  dailyReturnPct: number
  drawdownPct: number
  exposurePct: number
  holdings: SectorHolding[]
}

export interface SectorSummary {
  initialCashUsd: number
  endingEquityUsd: number
  netReturnPct: number
  samePathReturnBeforeCostsPct: number
  maxDrawdownPct: number
  maxDailyLossPct: number
  dailyVolatilityPct: number
  totalCostUsd: number
  realizedPnlUsd: number
  unrealizedPnlUsd: number
  averageExposurePct: number
  turnoverPct: number
  filledTradeCount: number
  cashLimitedCount: number
  endingCashUsd: number
}

export interface SectorPortfolioResult {
  strategy: SectorStrategy
  summary: SectorSummary
  daily: SectorDaily[]
  trades: SectorTrade[]
  endingHoldings: SectorHolding[]
}

export interface SectorPortfolioReport {
  engineVersion: string
  inputHash: string
  source: SectorSource
  historyBars: number
  evaluationBars: number
  startDate: string
  endDate: string
  executionModel: string
  warnings: string[]
  rankings: SectorRankingDay[]
  portfolios: SectorPortfolioResult[]
}
