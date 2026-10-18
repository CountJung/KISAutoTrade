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

// A separate bounded collection policy; the short preview's 8/500 limits above
// remain unchanged. Chunks are formed only after all backwards pages are sorted.
const CONTINUOUS_MAX_ATTEMPTS = 48
const CONTINUOUS_MAX_EVALUATION = 4096
const CONTINUOUS_MAX_RETAINED = 4608
export type ContinuousCollectionEnd = CandleCollectionEnd | 'retainedLimit'
export interface ContinuousCollectionProgress { pages: number; evaluationBars: number; historyBars: number }
export interface ContinuousCandleCollection extends ContinuousCollectionProgress {
  candles: ChartCandle[]
  historyCandles: ChartCandle[]
  chunks: Array<{ candles: ChartCandle[] }>
  requestedStart: string
  requestedEnd: string
  actualStart: string | null
  actualEnd: string | null
  asOf: string | null
  ready: boolean
  end: ContinuousCollectionEnd
  issues: string[]
  excludedUnfinishedBars: number
  excludedInvalidBars: number
  duplicateBars: number
  maximumCalendarGapDays: number
  coverageVerified: false
  sourceLabel: string
  dataSource: string
  isOverseas: boolean
}

function calendarLabel(value: string): string {
  const compact = value.replace(/-/g, '')
  if (!/^\d{8}$/.test(compact)) throw new Error('연속 일봉 날짜는 YYYY-MM-DD 또는 YYYYMMDD여야 합니다.')
  const iso = `${compact.slice(0, 4)}-${compact.slice(4, 6)}-${compact.slice(6, 8)}`
  const parsed = new Date(`${iso}T00:00:00Z`)
  if (!Number.isFinite(parsed.getTime()) || parsed.toISOString().slice(0, 10) !== iso) throw new Error('유효한 연속 분석 날짜를 입력하세요.')
  return compact
}

export function continuousRangeError(start: string, end: string): string | null {
  try {
    if (calendarLabel(start) >= calendarLabel(end)) return '연속 분석 시작일은 종료일보다 이전이어야 합니다.'
  } catch (error) { return (error as Error).message }
  return null
}

export async function collectContinuousCandles(
  input: Omit<CandlePageInput, 'count' | 'cursor' | 'interval'>,
  range: { start: string; end: string },
  isCancelled: () => boolean = () => false,
  onProgress: (value: ContinuousCollectionProgress) => void = () => {},
  fetchPage: (input: CandlePageInput) => Promise<CandlePageView> = getCandlePage,
): Promise<ContinuousCandleCollection> {
  const rangeError = continuousRangeError(range.start, range.end)
  if (rangeError || !input.expectedProfileId || !input.symbol) throw new Error(rangeError ?? '연속 조회 프로파일과 종목을 확인하세요.')
  const start = calendarLabel(range.start), end = calendarLabel(range.end)
  const collected = new Map<string, ChartCandle>()
  const seenCursors = new Set<string>()
  const exchanges = input.brokerId === 'kis' && input.isOverseas && !input.exchange ? [...OVERSEAS_EXCHANGES] : [input.exchange]
  let exchangeIndex = 0, attempts = 0, pages = 0, duplicateBars = 0, excludedUnfinishedBars = 0, excludedInvalidBars = 0
  let first: CandlePageView | null = null
  let cursor: string | undefined
  let previousOldest: string | null = null
  let stop: ContinuousCollectionEnd = 'pageLimit'
  let providerReason: string | null = null
  let evaluationBars = 0, historyBars = 0
  while (attempts < CONTINUOUS_MAX_ATTEMPTS) {
    checkCancelled(isCancelled)
    const request: CandlePageInput = { ...input, interval: 'D', exchange: exchanges[exchangeIndex], cursor, count: 200 }
    let page: CandlePageView
    attempts += 1
    try { page = await fetchPage(request) } catch (error) {
      checkCancelled(isCancelled)
      if (!isProviderError(error)) throw error
      if (!first && exchangeIndex + 1 < exchanges.length && attempts < CONTINUOUS_MAX_ATTEMPTS) { exchangeIndex += 1; continue }
      if (!first) throw error
      stop = 'partialError'
      providerReason = typeof error === 'object' && error !== null && 'message' in error ? String(error.message) : '페이지 조회 실패'
      break
    }
    checkCancelled(isCancelled)
    assertPageScope(page, request, first)
    if (page.timestampSemantics !== 'tradingDate' || !Array.isArray(page.candles) || page.candles.length > 200) throw new Error('연속 일봉 페이지의 시각/최대 200봉 계약을 확인할 수 없습니다.')
    first ??= page
    pages += 1
    excludedUnfinishedBars += page.excludedUnfinishedBars
    excludedInvalidBars += page.excludedInvalidBars
    let oldest: string | null = null
    for (const candle of page.candles) {
      const day = calendarLabel(candle.date)
      if (!oldest || day < oldest) oldest = day
      if (day > end) continue
      if (collected.has(day)) duplicateBars += 1
      else collected.set(day, candle)
    }
    const ordered = [...collected.keys()].sort()
    evaluationBars = ordered.filter((day) => day >= start).length
    historyBars = Math.min(HISTORY_BARS, ordered.filter((day) => day < start).length)
    const exceeded = collected.size > CONTINUOUS_MAX_RETAINED || evaluationBars > CONTINUOUS_MAX_EVALUATION
    if (collected.size > CONTINUOUS_MAX_RETAINED) {
      for (const day of ordered.slice(0, ordered.length - CONTINUOUS_MAX_RETAINED)) collected.delete(day)
    }
    onProgress({ pages, evaluationBars, historyBars })
    providerReason = page.endReason
    if (exceeded) { stop = 'retainedLimit'; break }
    if (evaluationBars > 0 && historyBars === HISTORY_BARS) { stop = 'targetReached'; break }
    if (!page.nextCursor) { stop = 'providerEnd'; break }
    if (seenCursors.has(page.nextCursor) || page.nextCursor === cursor) { stop = 'repeatedCursor'; break }
    if (oldest && previousOldest && oldest >= previousOldest) { stop = 'noProgress'; break }
    if (oldest) previousOldest = oldest
    seenCursors.add(page.nextCursor)
    cursor = page.nextCursor
  }
  checkCancelled(isCancelled)
  const ordered = [...collected.entries()].sort(([left], [right]) => left.localeCompare(right))
  const candles = ordered.filter(([day]) => day >= start && day <= end).map(([, candle]) => candle)
  const historyCandles = ordered.filter(([day]) => day < start).slice(-HISTORY_BARS).map(([, candle]) => candle)
  const issues: string[] = []
  if (!candles.length) issues.push('요청 기간의 완료 평가 봉을 확보하지 못했습니다.')
  if (historyCandles.length < HISTORY_BARS) issues.push(`시작일보다 엄격히 이전인 사전자료 ${historyCandles.length}/${HISTORY_BARS}봉: 요청 시작 경계와 준비 자료를 모두 확보해야 합니다.`)
  const stopLabel: Record<ContinuousCollectionEnd, string> = { targetReached: '목표 확보', providerEnd: '제공 자료 끝', noProgress: '추가 과거 봉 없음', repeatedCursor: '커서 반복', pageLimit: '최대 48페이지 도달', partialError: '일부 페이지 조회 실패', retainedLimit: '평가/보관 봉 상한 도달' }
  if (stop !== 'targetReached') issues.push(`수집을 완료하지 못했습니다: ${stopLabel[stop]}${providerReason ? ` (${providerReason})` : ''}`)
  if (candles.length > CONTINUOUS_MAX_EVALUATION) issues.push('연속 평가 4096봉 상한을 초과했습니다. 기간을 줄이세요.')
  let maximumCalendarGapDays = 0
  for (let index = 1; index < candles.length; index += 1) {
    const previous = calendarLabel(candles[index - 1].date), current = calendarLabel(candles[index].date)
    const instant = (day: string) => Date.parse(`${day.slice(0, 4)}-${day.slice(4, 6)}-${day.slice(6, 8)}T00:00:00Z`)
    maximumCalendarGapDays = Math.max(maximumCalendarGapDays, (instant(current) - instant(previous)) / 86_400_000)
  }
  const dayInstant = (label: string) => {
    const day = calendarLabel(label)
    return Date.parse(`${day.slice(0, 4)}-${day.slice(4, 6)}-${day.slice(6, 8)}T00:00:00Z`)
  }
  if (candles.length && ((dayInstant(candles[0].date) - dayInstant(start)) / 86_400_000 > 7 || (dayInstant(end) - dayInstant(candles[candles.length - 1].date)) / 86_400_000 > 7)) issues.push('요청 시작/종료와 실제 관측 경계의 차이가 보수적 7달력일 상한을 초과해 연속기간 준비를 확인하지 못했습니다.')
  if (maximumCalendarGapDays > 7) issues.push('평가 관측 사이의 간격이 보수적 7달력일 상한을 초과합니다. 거래소 휴장·누락을 구분해 확인해야 합니다.')
  const chunks: Array<{ candles: ChartCandle[] }> = []
  if (issues.length === 0) for (let offset = 0; offset < candles.length; offset += 500) chunks.push({ candles: candles.slice(offset, offset + 500) })
  const dataSource = `${input.brokerId === 'toss' ? 'Toss' : 'KIS'}${first?.exchange ? ` ${first.exchange}` : ''} 연속 일봉 요청 ${start}~${end}`
  return {
    candles, historyCandles, chunks, requestedStart: start, requestedEnd: end,
    actualStart: candles[0]?.date ?? null, actualEnd: candles[candles.length - 1]?.date ?? null,
    asOf: first?.asOf ?? null, ready: issues.length === 0, end: stop, issues,
    pages, evaluationBars: candles.length, historyBars: historyCandles.length,
    excludedUnfinishedBars, excludedInvalidBars, duplicateBars, maximumCalendarGapDays, coverageVerified: false,
    sourceLabel: `${dataSource} · 평가 ${candles.length}봉 · 사전 ${historyCandles.length}봉 · ${pages}페이지 · ${stopLabel[stop]}`,
    dataSource, isOverseas: input.isOverseas,
  }
}
