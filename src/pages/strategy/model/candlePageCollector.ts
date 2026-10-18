import { getCandlePage } from '../../../shared/api/commands'
import type { CandlePageInput, CandlePageView, ChartCandle } from '../../../shared/api/types'

const MAX_PAGES = 8
const MAX_RETAINED = 500
const HISTORY_BARS = 252
const OVERSEAS_EXCHANGES = ['NAS', 'NYS', 'AMS'] as const

export type CandleCollectionEnd = 'targetReached' | 'providerEnd' | 'noProgress' | 'repeatedCursor' | 'pageLimit' | 'partialError'

export interface CandleCollection {
  candles: ChartCandle[]
  historyCandles: ChartCandle[]
  pages: number
  end: CandleCollectionEnd
  providerEndReason: string | null
  excludedUnfinishedBars: number
  excludedInvalidBars: number
  duplicateBars: number
  sourceLabel: string
  dataSource: string
  isOverseas: boolean
}

function isProviderError(error: unknown): boolean {
  return typeof error === 'object' && error !== null && 'code' in error && error.code === 'CANDLE_PAGE_FAILED'
}

function checkCancelled(isCancelled: () => boolean) {
  if (isCancelled()) throw new Error('캔들 조회가 취소되었거나 입력 범위가 변경되었습니다.')
}

function candleTimestamp(date: string): number {
  const compact = /^(\d{4})(\d{2})(\d{2})(?:(\d{2})(\d{2})(\d{2})?)?$/.exec(date)
  const timestamp = compact
    ? Date.parse(`${compact[1]}-${compact[2]}-${compact[3]}T${compact[4] ?? '00'}:${compact[5] ?? '00'}:${compact[6] ?? '00'}+09:00`)
    : Date.parse(date)
  if (!Number.isFinite(timestamp)) throw new Error('캔들 페이지에 유효하지 않은 시간이 포함되어 결과를 폐기했습니다.')
  return timestamp
}

function assertPageScope(page: CandlePageView, input: CandlePageInput, first: CandlePageView | null) {
  if (page.profileId !== input.expectedProfileId || page.brokerId !== input.brokerId ||
      page.symbol !== input.symbol || page.interval !== input.interval ||
      page.exchange !== (input.exchange ?? null) || !page.scopeKey || !page.asOf ||
      (first && (page.scopeKey !== first.scopeKey || page.asOf !== first.asOf ||
        page.source !== first.source || page.timestampSemantics !== first.timestampSemantics ||
        page.completionPolicy !== first.completionPolicy))) {
    throw new Error('캔들 페이지의 프로파일·계좌·조회 기준이 변경되어 결과를 폐기했습니다.')
  }
}

/** Sequential cursor collection: retain the full evaluation window before assigning older history. */
export async function collectPreviewCandles(
  input: Omit<CandlePageInput, 'count' | 'cursor'>,
  evaluationCount: number,
  isCancelled: () => boolean = () => false,
  fetchPage: (input: CandlePageInput) => Promise<CandlePageView> = getCandlePage,
): Promise<CandleCollection> {
  if (!input.expectedProfileId || !input.symbol || !Number.isInteger(evaluationCount) || evaluationCount < 1 || evaluationCount > 248) {
    throw new Error('캔들 조회 프로파일과 분석 구간을 확인하세요.')
  }
  const target = Math.min(MAX_RETAINED, evaluationCount + HISTORY_BARS)
  const collected = new Map<number, ChartCandle>()
  const seenCursors = new Set<string>()
  const exchanges = input.brokerId === 'kis' && input.isOverseas && !input.exchange
    ? [...OVERSEAS_EXCHANGES] : [input.exchange]
  let exchangeIndex = 0
  let cursor: string | undefined
  let first: CandlePageView | null = null
  let pages = 0
  let attempts = 0
  let end: CandleCollectionEnd = 'pageLimit'
  let providerEndReason: string | null = null
  let excludedUnfinishedBars = 0
  let excludedInvalidBars = 0
  let duplicateBars = 0

  while (attempts < MAX_PAGES) {
    checkCancelled(isCancelled)
    const request = { ...input, exchange: exchanges[exchangeIndex], cursor, count: 200 }
    let page: CandlePageView
    attempts += 1
    try {
      page = await fetchPage(request)
    } catch (error) {
      checkCancelled(isCancelled)
      if (!isProviderError(error)) throw error
      if (!first && exchangeIndex + 1 < exchanges.length && attempts < MAX_PAGES) {
        exchangeIndex += 1
        continue
      }
      if (!first) throw error
      end = 'partialError'
      providerEndReason = typeof error === 'object' && error !== null && 'message' in error ? String(error.message) : '페이지 조회 실패'
      break
    }
    checkCancelled(isCancelled)
    assertPageScope(page, request, first)
    if (!Array.isArray(page.candles) || page.candles.length > request.count) {
      throw new Error('캔들 페이지가 최대 200봉 계약을 초과하여 결과를 폐기했습니다.')
    }
    first ??= page
    pages += 1
    excludedUnfinishedBars += page.excludedUnfinishedBars
    excludedInvalidBars += page.excludedInvalidBars
    const previousSize = collected.size
    for (const candle of page.candles) {
      const timestamp = candleTimestamp(candle.date)
      if (collected.has(timestamp)) duplicateBars += 1
      else collected.set(timestamp, candle)
    }
    const retained = [...collected.entries()].sort(([left], [right]) => left - right).slice(-MAX_RETAINED)
    collected.clear()
    for (const [timestamp, candle] of retained) collected.set(timestamp, candle)
    providerEndReason = page.endReason
    if (collected.size >= target) { end = 'targetReached'; break }
    if (!page.nextCursor) { end = 'providerEnd'; break }
    if (seenCursors.has(page.nextCursor) || page.nextCursor === cursor) { end = 'repeatedCursor'; break }
    // Filtered unfinished/invalid-only pages can still advance the provider cursor.
    if (page.candles.length > 0 && collected.size === previousSize) { end = 'noProgress'; break }
    seenCursors.add(page.nextCursor)
    cursor = page.nextCursor
  }
  checkCancelled(isCancelled)
  const ordered = [...collected.entries()].sort(([left], [right]) => left - right).slice(-target).map(([, candle]) => candle)
  const evaluationStart = Math.max(0, ordered.length - evaluationCount)
  const candles = ordered.slice(evaluationStart)
  const historyCandles = ordered.slice(0, evaluationStart)
  const intervalLabel = input.interval === '1m' ? '1분봉' : input.interval === 'D' ? '일봉' : input.interval === 'W' ? '주봉' : '월봉'
  const dataSource = `${input.brokerId === 'toss' ? 'Toss' : 'KIS'}${first?.exchange ? ` ${first.exchange}` : ''} ${intervalLabel}`
  const endLabel: Record<CandleCollectionEnd, string> = {
    targetReached: '목표 확보', providerEnd: '제공 자료 끝', noProgress: '추가 봉 없음',
    repeatedCursor: '커서 반복', pageLimit: '최대 8페이지 도달', partialError: '일부 페이지 조회 실패',
  }
  const range = candles.length === evaluationCount ? `최근 ${evaluationCount}봉` : `실제 ${candles.length}봉 / 요청 ${evaluationCount}봉`
  const sourceLabel = `${dataSource} · ${range} · 사전 자료 ${historyCandles.length}봉 · ${pages}페이지 · ${endLabel[end]}` +
    `${providerEndReason ? ` (${providerEndReason})` : ''} · 미완료 제외 ${excludedUnfinishedBars}봉 · 무효 제외 ${excludedInvalidBars}봉 · 중복 제외 ${duplicateBars}봉`
  return { candles, historyCandles, pages, end, providerEndReason, excludedUnfinishedBars, excludedInvalidBars, duplicateBars, sourceLabel, dataSource, isOverseas: input.isOverseas }
}
