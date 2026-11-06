import type { ValidateSectorDataInput } from '../../../shared/api/sectorDataTypes'

export const SECTOR_DATA_MAX_BYTES = 1_500_000
export const SECTOR_DATA_SYMBOLS = ['SPY', 'XLB', 'XLC', 'XLE', 'XLF', 'XLI', 'XLK', 'XLP', 'XLRE', 'XLU', 'XLV', 'XLY']

/** Intentionally incomplete: a template never asserts real calendar, exchange, adjustment or events. */
export function sectorDataTemplate() {
  return {
    bundle: {
      schemaVersion: 1,
      manifest: {
        sessionDates: [], endDate: '', calendarSource: '', calendarEvidence: '', synthetic: false,
        exchangeMappings: SECTOR_DATA_SYMBOLS.map(symbol => ({ symbol, exchange: '', evidence: '' })),
      },
      retrievedAt: '', series: [], pages: [],
    },
    verification: {
      coverageVerified: false, adjustmentVerified: false, corporateActionsVerified: false,
      adjustmentEvidence: '', corporateActionsEvidence: '', corporateActions: [],
    },
    initialCashUsd: 10_000, costBps: 10,
  }
}

/** Bounds only; the Rust validator owns calendar, price, mapping and financial semantics. */
export function parseSectorDataPreparation(text: string): ValidateSectorDataInput {
  if (new TextEncoder().encode(text).byteLength > SECTOR_DATA_MAX_BYTES) throw new Error('자료 준비 JSON은 1.5MB 이내여야 합니다.')
  const document: unknown = JSON.parse(text)
  // Reimport our evidence export without trusting its previous validation report.
  const value: unknown = document && typeof document === 'object' && !Array.isArray(document) && 'request' in document
    ? document.request : document
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('자료 준비 JSON 객체가 필요합니다.')
  const data = value as Partial<ValidateSectorDataInput>
  if (!data.bundle || typeof data.bundle !== 'object' || Array.isArray(data.bundle) || !data.bundle.manifest || typeof data.bundle.manifest !== 'object' || Array.isArray(data.bundle.manifest)
    || !Array.isArray(data.bundle.series) || data.bundle.series.length > 12 || !Array.isArray(data.bundle.manifest.sessionDates)
    || data.bundle.manifest.sessionDates.length > 500 || !data.verification || typeof data.verification !== 'object' || Array.isArray(data.verification)) {
    throw new Error('bundle.manifest, bundle.series(최대12개), verification이 필요합니다. 기간은 최대500봉입니다.')
  }
  const manifest = data.bundle.manifest
  if (typeof manifest.endDate !== 'string' || typeof manifest.calendarSource !== 'string' || typeof manifest.calendarEvidence !== 'string'
    || typeof manifest.synthetic !== 'boolean' || typeof data.bundle.retrievedAt !== 'string'
    || (data.bundle.pages != null && (!Array.isArray(data.bundle.pages) || data.bundle.pages.length > 2000))) {
    throw new Error('기준일·달력 출처/근거·수집시각은 문자열, synthetic은 boolean, pages는 배열이어야 합니다.')
  }
  return data as ValidateSectorDataInput
}
