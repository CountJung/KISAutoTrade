import type { BrokerChartInput } from '../../../shared/api/brokerChartTypes'
import type { SectorPortfolioInput } from '../../../shared/api/sectorPortfolioTypes'
import { SECTOR_INPUT_MAX_BYTES, syntheticSectorPortfolioInput } from './sectorPortfolioInput'

/** Structural/transport bounds only. Rust owns OHLC, session and financial validation. */
export function parseBrokerChartInput(text: string): BrokerChartInput {
  if (new TextEncoder().encode(text).byteLength > SECTOR_INPUT_MAX_BYTES) throw new Error('차트 JSON은 1.5MB 이내로 입력하세요.')
  const value: unknown = JSON.parse(text)
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('차트 자료는 JSON 객체여야 합니다.')
  const input = value as Partial<BrokerChartInput>
  const source = input.source
  if (input.schemaVersion !== 1 || !source || typeof source !== 'object' || Array.isArray(source)
    || !['kis', 'toss'].includes(source.brokerId) || typeof source.name !== 'string' || typeof source.retrievedAt !== 'string'
    || typeof source.synthetic !== 'boolean' || source.adjustment !== 'unadjusted' || source.currency !== 'USD' || source.interval !== '1d'
    || !['observedSpySessions', 'brokerCalendar'].includes(source.sessionBasis) || typeof source.calendarEvidence !== 'string'
    || !Array.isArray(input.history) || !Array.isArray(input.evaluation) || !Array.isArray(input.expectedSessionDates)
    || !Array.isArray(input.knownCorporateActions) || typeof input.acknowledgedPriceOnly !== 'boolean'
    || typeof input.initialCashUsd !== 'number' || !Number.isFinite(input.initialCashUsd) || ![10, 20].includes(input.costBps ?? 0)) {
    throw new Error('schemaVersion 1, 비조정 USD 일봉 source, 사전/평가자료, 거래일, 기업행위 목록, 초기자금·비용·한계 확인 값이 필요합니다.')
  }
  if (input.history.length < 22 || !input.evaluation.length || input.history.length + input.evaluation.length > 500) {
    throw new Error('사전자료 최소22봉과 평가자료를 분리하고 합계500봉 이내로 입력하세요.')
  }
  return input as BrokerChartInput
}

export function syntheticBrokerChartInput(): BrokerChartInput {
  const fixture = syntheticSectorPortfolioInput()
  const convert = (days: typeof fixture.history) => days.map(day => ({ date: day.date,
    prices: Object.fromEntries(Object.entries(day.prices).map(([symbol, price]) => [symbol, {
      open: price.open, high: price.close + 1, low: price.open - 1, close: price.close, volume: 100_000, currency: 'USD' as const,
    }])),
  }))
  return {
    schemaVersion: 1, initialCashUsd: 10_000, costBps: 10,
    source: { brokerId: 'kis', name: '합성 차트 예제 · 실제 브로커 시세 아님', retrievedAt: '2026-01-01T00:00:00Z', synthetic: true,
      adjustment: 'unadjusted', currency: 'USD', interval: '1d', sessionBasis: 'observedSpySessions',
      calendarEvidence: '합성 평일 fixture · 공식 거래소 달력 아님' },
    history: convert(fixture.history), evaluation: convert(fixture.evaluation.slice(0, 20)),
    expectedSessionDates: fixture.expectedSessionDates.slice(0, 42), acknowledgedPriceOnly: false, knownCorporateActions: [],
  }
}

/** Results-view adapter only: raw close fills the signal slot without asserting adjustment. */
export function brokerChartResultsView(input: BrokerChartInput): SectorPortfolioInput {
  const convert = (days: BrokerChartInput['history']) => days.map(day => ({ date: day.date,
    prices: Object.fromEntries(Object.entries(day.prices).map(([symbol, price]) => [symbol, { open: price.open, close: price.close, adjustedClose: price.close }])),
  }))
  return { initialCashUsd: input.initialCashUsd, costBps: input.costBps,
    source: { name: input.source.name, retrievedAt: input.source.retrievedAt, synthetic: input.source.synthetic,
      adjustmentPolicy: 'unadjustedSignalRawExecution', calendarSource: input.source.calendarEvidence,
      coverageVerified: false, corporateActionsVerified: false },
    history: convert(input.history), evaluation: convert(input.evaluation), expectedSessionDates: input.expectedSessionDates, corporateActions: [],
  }
}
