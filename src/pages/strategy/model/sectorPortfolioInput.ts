import type { SectorDay, SectorPortfolioInput } from '../../../shared/api/sectorPortfolioTypes'

export const SECTOR_INPUT_MAX_BYTES = 1_500_000
const symbols = ['SPY', 'XLC', 'XLY', 'XLP', 'XLE', 'XLF', 'XLV', 'XLI', 'XLB', 'XLRE', 'XLK', 'XLU']

/** A UI/transport bound only; financial and calendar validation belongs to Rust. */
export function parseSectorPortfolioInput(text: string): SectorPortfolioInput {
  if (new TextEncoder().encode(text).byteLength > SECTOR_INPUT_MAX_BYTES) {
    throw new Error('JSON 자료는 1.5MB 이내로 입력하세요.')
  }
  const value: unknown = JSON.parse(text)
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('자료는 JSON 객체여야 합니다.')
  const input = value as Partial<SectorPortfolioInput>
  if (!input.source || typeof input.source !== 'object' || Array.isArray(input.source)
    || !Array.isArray(input.history) || !Array.isArray(input.evaluation)
    || !Array.isArray(input.expectedSessionDates)) {
    throw new Error('source, history, evaluation, expectedSessionDates가 필요합니다.')
  }
  if (!input.history.length || !input.evaluation.length || input.history.length + input.evaluation.length > 500) {
    throw new Error('사전자료와 평가자료를 분리하고 합계 500거래일 이내로 입력하세요.')
  }
  return input as SectorPortfolioInput
}

/** Deliberately synthetic weekday calendar and prices, never market history. */
export function syntheticSectorPortfolioInput(): SectorPortfolioInput {
  const dates: string[] = []
  const date = new Date('2025-01-06T00:00:00Z')
  while (dates.length < 52) {
    if (date.getUTCDay() !== 0 && date.getUTCDay() !== 6) dates.push(date.toISOString().slice(0, 10))
    date.setUTCDate(date.getUTCDate() + 1)
  }
  const days: SectorDay[] = dates.map((day, index) => ({
    date: day,
    prices: Object.fromEntries(symbols.map((symbol, rank) => {
      const price = Math.round((100 + index * (0.05 + rank * 0.03)) * 100) / 100
      return [symbol, { adjustedClose: price, open: price, close: price }]
    })),
  }))
  return {
    initialCashUsd: 10_000,
    costBps: 10,
    source: {
      name: '합성 예제 · 실제 시장자료 아님',
      retrievedAt: '2026-01-01T00:00:00Z',
      synthetic: true,
      adjustmentPolicy: 'adjustedSignalRawExecution',
      calendarSource: '합성 fixture 내부 완전성·기업행위 없음 선언 · 실제 거래소 검증 아님',
      coverageVerified: true,
      corporateActionsVerified: true,
    },
    history: days.slice(0, 22),
    evaluation: days.slice(22),
    expectedSessionDates: dates,
    corporateActions: [],
  }
}
