import type { SectorCorporateAction, SectorPortfolioInput } from './sectorPortfolioTypes'

export interface SectorQuoteScope { profileId: string; provider: string; paper: boolean }
export interface SectorExchangeMapping { symbol: string; exchange: string; evidence: string }
export interface SectorDataManifest {
  sessionDates: string[]
  endDate: string
  calendarSource: string
  calendarEvidence: string
  exchangeMappings: SectorExchangeMapping[]
  synthetic: boolean
}
export interface SectorDataSeries {
  symbol: string
  exchange: string
  raw: { date: string; open: number; close: number }[]
  adjusted: { date: string; close: number }[]
}
export interface SectorPageReceipt { symbol: string; exchange: string; modp: string; anchor: string; rowCount: number; rowsHash: string }
export interface SectorDataBundle { schemaVersion: number; manifest: SectorDataManifest; retrievedAt: string; series: SectorDataSeries[]; pages?: SectorPageReceipt[] }
export interface SectorDataVerification {
  coverageVerified: boolean
  adjustmentVerified: boolean
  corporateActionsVerified: boolean
  adjustmentEvidence: string
  corporateActionsEvidence: string
  corporateActions?: SectorCorporateAction[]
}
export interface CollectSectorDataInput { expectedProfileId: string; authenticatedQuotesAcknowledged: boolean; manifest: SectorDataManifest }
export interface ValidateSectorDataInput { bundle: SectorDataBundle; verification: SectorDataVerification; initialCashUsd: number; costBps: number }
export interface SectorDataValidation {
  ready: boolean
  diagnostics: { code: string; message: string }[]
  warnings: string[]
  input: SectorPortfolioInput | null
  bundleHash: string
  verificationHash: string
}
