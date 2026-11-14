import type { SectorCorporateAction, SectorPortfolioReport } from './sectorPortfolioTypes'

/** Separate price-only contract; never a strict verified sector input. */
export interface BrokerChartPrice { open: number; high: number; low: number; close: number; volume: number; currency: 'USD' }
export interface BrokerChartDay { date: string; prices: Record<string, BrokerChartPrice> }
export interface BrokerChartInput {
  schemaVersion: 1
  initialCashUsd: number
  costBps: 10 | 20
  source: {
    brokerId: 'kis' | 'toss'; name: string; retrievedAt: string; synthetic: boolean
    adjustment: 'unadjusted'; currency: 'USD'; interval: '1d'
    sessionBasis: 'observedSpySessions' | 'brokerCalendar'; calendarEvidence: string
  }
  history: BrokerChartDay[]
  evaluation: BrokerChartDay[]
  expectedSessionDates: string[]
  acknowledgedPriceOnly: boolean
  knownCorporateActions: SectorCorporateAction[]
}
export interface BrokerChartReport {
  mode: 'brokerChartPriceApproximation'
  signalBasis: 'unadjustedClose'
  dividendReturnIncluded: false
  economicReturnVerified: false
  inputHash: string
  warnings: string[]
  result: SectorPortfolioReport
}
export interface BrokerChartScope { profileId: string; provider: 'kis' | 'toss'; paper: boolean }
export interface BrokerChartExchangeMapping { symbol: string; exchange: string; evidence: string }
export interface CollectBrokerChartInput {
  expectedProfileId: string
  authenticatedQuotesAcknowledged: boolean
  evaluationBars: number
  endDate?: string
  exchangeMappings?: BrokerChartExchangeMapping[]
}
/** Identity HTTP body fingerprint; receivedAt is UTC body receipt completion, not publication/PIT. */
export interface BrokerChartResponseSource { sha256: string; byteLength: number; receivedAt: string }
export interface BrokerChartPageReceipt {
  symbol: string; exchange: string | null; page: number; rowCount: number
  oldestDate: string; newestDate: string; rowsHash: string; requestAdjustment: 'unadjusted'
  /** Missing for unavailable/unknown provenance, including KIS and legacy collections. */
  responseSource?: BrokerChartResponseSource
}
export interface BrokerChartCollection {
  input: BrokerChartInput
  diagnostics: string[]
  receipts: BrokerChartPageReceipt[]
  collectionHash: string
  exchangeMappings?: BrokerChartExchangeMapping[]
}
