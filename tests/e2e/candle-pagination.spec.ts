import { expect, test, type Locator, type Page } from '@playwright/test'
import type { CandlePageView, ChartCandle } from '../../src/api/types'
import { historyFixture, mockApi, readyPreparation } from './fixtures/strategyResearchFixture'

async function genericCard(page: Page, count = 200): Promise<Locator> {
  await page.goto('/strategy')
  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  if (count !== 50) {
    await card.getByRole('combobox', { name: '분석 구간' }).click()
    await page.getByRole('option', { name: `최근 ${count}봉`, exact: true }).click()
  }
  return card
}

for (const broker of ['kis', 'toss'] as const) {
  test(`${broker} pagination preserves 200 evaluation + 252 prior bars and encoded REST cursors`, async ({ page }) => {
    const requests: string[] = []
    const previews: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[] }> = []
    const ordered = historyFixture(452)
    await mockApi(page, { activeBroker: broker, chartRequests: requests, genericPreviewRequests: previews,
      chartCandles: ordered, genericPreparation: { ...readyPreparation, evaluatedBars: 200 } })
    const card = await genericCard(page)
    await card.getByRole('button', { name: '미리보기 계산' }).click()
    await expect(card.getByTestId('strategy-evaluation-range')).toHaveText('요청 분석 200봉 · 실제 평가 200봉 · 사전 자료 252봉')
    await expect(card.getByTestId('candle-collection-status')).toContainText('목표 확보')
    expect(requests).toHaveLength(broker === 'kis' ? 5 : 3)
    expect(previews[0].candles).toEqual(ordered.slice(-200))
    expect(previews[0].historyCandles).toEqual(ordered.slice(0, 252))
    const first = new URL(requests[0])
    expect(first.pathname).toBe('/api/candles/page')
    expect(Object.fromEntries(first.searchParams)).toMatchObject({
      brokerId: broker, expectedProfileId: broker === 'toss' ? 'toss-live' : 'paper',
      expectedBrokerAccountId: broker === 'toss' ? '1' : '12345678-01',
      interval: 'D', count: '200', isOverseas: 'false', symbol: '000002',
    })
    expect(requests[1]).toContain('%7B')
    expect(JSON.parse(new URL(requests[1]).searchParams.get('cursor')!).token).toBe('a+/=&한글')
    expect(previews[0].historyCandles.at(-1)!.date < previews[0].candles[0].date).toBe(true)
  })
}

test('inclusive page overlaps are deduplicated chronologically without changing the evaluation window', async ({ page }) => {
  const ordered = historyFixture(452)
  const previews: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[] }> = []
  await mockApi(page, { activeBroker: 'toss', genericPreviewRequests: previews, candlePages: [
    { candles: ordered.slice(252).reverse(), excludedUnfinishedBars: 1 },
    { candles: ordered.slice(53, 253).reverse(), excludedInvalidBars: 2 },
    { candles: ordered.slice(0, 54).reverse() },
  ], genericPreparation: { ...readyPreparation, evaluatedBars: 200 } })
  const card = await genericCard(page)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  const status = card.getByTestId('candle-collection-status')
  await expect(status).toContainText('3페이지 · 목표 확보')
  await expect(status).toContainText('미완료 제외 1봉 · 무효 제외 2봉 · 중복 제외 2봉')
  expect(previews[0].candles).toEqual(ordered.slice(-200))
  expect(previews[0].historyCandles).toEqual(ordered.slice(0, 252))
})

for (const scenario of ['exhausted', 'noProgress', 'repeatedCursor', 'pageLimit', 'partialError'] as const) {
  test(`bounded collector reports ${scenario} and retains the requested evaluation bars`, async ({ page }) => {
    const ordered = historyFixture(scenario === 'pageLimit' ? 240 : 220)
    const requests: string[] = []
    const previews: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[] }> = []
    const pages: Array<Partial<CandlePageView>> = scenario === 'pageLimit'
      ? Array.from({ length: 10 }, (_, index) => ({ candles: ordered.slice(210 - index * 30, 240 - index * 30) }))
      : scenario === 'noProgress' || scenario === 'repeatedCursor'
        ? [{ candles: ordered.slice(-200) }, { candles: ordered.slice(-200), ...(scenario === 'repeatedCursor' ? { nextCursor: JSON.stringify({ offset: 1, token: 'a+/=&한글' }) } : {}) }, { candles: [] }]
        : [{ candles: ordered.slice(-200) }, { candles: ordered.slice(0, 20) }]
    await mockApi(page, { activeBroker: 'toss', chartRequests: requests, genericPreviewRequests: previews,
      candlePages: pages, ...(scenario === 'partialError' ? { candlePageError: { page: 1, code: 'CANDLE_PAGE_FAILED' } } : {}) })
    const card = await genericCard(page)
    await card.getByRole('button', { name: '미리보기 계산' }).click()
    const status = card.getByTestId('candle-collection-status')
    const labels = { exhausted: '제공 자료 끝', noProgress: '추가 봉 없음', repeatedCursor: '커서 반복', pageLimit: '최대 8페이지 도달', partialError: '일부 페이지 조회 실패' }
    await expect(status).toContainText(labels[scenario])
    await expect(status).toContainText('자료 부족')
    await expect(card.getByTestId('strategy-evaluation-range')).toContainText('실제 평가 200봉')
    expect(requests).toHaveLength(scenario === 'pageLimit' ? 8 : 2)
    expect(previews[0].candles).toEqual(ordered.slice(-200))
    expect(previews[0].historyCandles.length).toBeLessThan(252)
  })
}

for (const scenario of ['scopeKey', 'profileId', 'asOf', 'scopeError'] as const) {
  test(`collector rejects ${scenario} change without replaying partial data`, async ({ page }) => {
    const previews: unknown[] = []
    const requests: string[] = []
    await mockApi(page, { activeBroker: 'toss', chartRequests: requests, genericPreviewRequests: previews, candlePages: [
      { candles: historyFixture(200) }, { candles: historyFixture(50), ...(scenario === 'scopeError' ? {} : { [scenario]: 'changed' }) },
    ], ...(scenario === 'scopeError' ? { candlePageError: { page: 1, code: 'SCOPE_MISMATCH' } } : {}) })
    const card = await genericCard(page)
    await card.getByRole('button', { name: '미리보기 계산' }).click()
    await expect(card.getByText(scenario === 'scopeError' ? 'mock page failure' : '캔들 페이지의 프로파일·계좌·조회 기준이 변경되어 결과를 폐기했습니다.')).toBeVisible()
    expect(requests).toHaveLength(2)
    expect(previews).toHaveLength(0)
    await expect(card.getByTestId('candle-collection-status')).toHaveCount(0)
  })
}

test('cancel during a page request prevents later pages and replay', async ({ page }) => {
  const requests: string[] = []
  const previews: unknown[] = []
  let releasePage!: () => void
  const firstPageResponse = new Promise<void>((resolve) => { releasePage = resolve })
  await mockApi(page, { chartRequests: requests, genericPreviewRequests: previews, chartCandles: historyFixture(452), candlePageGate: firstPageResponse })
  const card = await genericCard(page)
  // Hold the first response until the UI action completes, independently of runner speed.
  try {
    await card.getByRole('button', { name: '미리보기 계산' }).click()
    await expect.poll(() => requests.length).toBe(1)
    await card.getByRole('button', { name: '취소', exact: true }).click()
    await expect(card.getByText('미리보기 계산을 취소했습니다.')).toBeVisible()
  } finally {
    releasePage()
  }
  await page.waitForTimeout(350)
  expect(requests).toHaveLength(1)
  expect(previews).toHaveLength(0)
})

test('parameter change during collection stops subsequent pages and discards the response', async ({ page }) => {
  const requests: string[] = []
  const previews: unknown[] = []
  let releasePage!: () => void
  const firstPageResponse = new Promise<void>((resolve) => { releasePage = resolve })
  await mockApi(page, { chartRequests: requests, genericPreviewRequests: previews, chartCandles: historyFixture(452), candlePageGate: firstPageResponse })
  const card = await genericCard(page)
  // Hold the first response until the UI action completes, independently of runner speed.
  try {
    await card.getByRole('button', { name: '미리보기 계산' }).click()
    await expect.poll(() => requests.length).toBe(1)
    await card.getByRole('spinbutton', { name: '단기 MA', exact: true }).fill('7')
  } finally {
    releasePage()
  }
  await page.waitForTimeout(350)
  expect(requests).toHaveLength(1)
  expect(previews).toHaveLength(0)
  await expect(card.getByTestId('candle-collection-status')).toHaveCount(0)
})

test('minute pages use instant order and deduplicate timezone aliases while chart labels preserve offsets', async ({ page }) => {
  const previews: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[] }> = []
  const ordered = Array.from({ length: 302 }, (_, index) => ({
    date: new Date(Date.UTC(2026, 0, 1, 0, index)).toISOString(),
    open: '100', high: '105', low: '99', close: '104', volume: '1000',
  }))
  const kst = (candle: ChartCandle): ChartCandle => ({ ...candle,
    date: new Date(Date.parse(candle.date) + 9 * 60 * 60 * 1000).toISOString().replace('Z', '+09:00'),
  })
  await mockApi(page, { activeBroker: 'toss', genericPreviewRequests: previews, candlePages: [
    { candles: ordered.slice(102).reverse() }, { candles: ordered.slice(0, 103).map(kst).reverse() },
  ], genericSignals: [
    { time: ordered[252].date, side: 'buy', price: 104, quantity: 1, reason: 'UTC source' },
    { time: '2026-01-01T04:13:00-04:00', side: 'sell', price: 104, quantity: 1, reason: 'US offset source' },
  ] })
  const card = await genericCard(page, 50)
  await card.getByRole('combobox', { name: '봉 단위' }).click()
  await page.getByRole('option', { name: '1분봉', exact: true }).click()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-evaluation-range')).toContainText('실제 평가 50봉 · 사전 자료 252봉')
  await expect(card.getByTestId('candle-collection-status')).toContainText('중복 제외 1봉')
  expect(previews[0].candles).toEqual(ordered.slice(-50))
  expect(previews[0].historyCandles.map((candle) => Date.parse(candle.date))).toEqual(ordered.slice(0, 252).map((candle) => Date.parse(candle.date)))
  await expect(card.getByText('매수 01/01 13:12 · 104.00', { exact: true })).toBeVisible()
  await expect(card.getByText('매도 01/01 17:13 · 104.00', { exact: true })).toBeVisible()
})

test('an unfinished-only empty page advances to earlier valid pages', async ({ page }) => {
  const requests: string[] = []
  const ordered = historyFixture(302)
  const previews: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[] }> = []
  await mockApi(page, { activeBroker: 'toss', chartRequests: requests, genericPreviewRequests: previews, candlePages: [
    { candles: [], excludedUnfinishedBars: 20 }, { candles: ordered.slice(102) }, { candles: ordered.slice(0, 102) },
  ] })
  const card = await genericCard(page, 50)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('candle-collection-status')).toContainText('3페이지 · 목표 확보')
  await expect(card.getByTestId('candle-collection-status')).toContainText('미완료 제외 20봉')
  expect(requests).toHaveLength(3)
  expect(previews[0].candles).toEqual(ordered.slice(-50))
  expect(previews[0].historyCandles).toEqual(ordered.slice(0, 252))
})

test('oversized provider response is rejected before collection or replay', async ({ page }) => {
  const requests: string[] = []
  const previews: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', chartRequests: requests, genericPreviewRequests: previews,
    candlePages: [{ candles: historyFixture(201) }, { candles: [] }] })
  const card = await genericCard(page, 50)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByText('캔들 페이지가 최대 200봉 계약을 초과하여 결과를 폐기했습니다.')).toBeVisible()
  expect(requests).toHaveLength(1)
  expect(previews).toHaveLength(0)
})

test('KIS overseas fallback occurs only on the first provider failure and locks subsequent pages to that exchange', async ({ page }) => {
  const requests: string[] = []
  const previews: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[] }> = []
  const ordered = historyFixture(302)
  await mockApi(page, { genericSymbol: 'SOXL', chartRequests: requests, genericPreviewRequests: previews,
    candlePageError: { page: 0, code: 'CANDLE_PAGE_FAILED' }, candlePages: [
      {}, { candles: ordered.slice(202) }, { candles: ordered.slice(102, 202) },
      { candles: ordered.slice(2, 102) }, { candles: ordered.slice(0, 2) },
    ] })
  const card = await genericCard(page, 50)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('candle-collection-status')).toContainText('KIS NYS 일봉')
  await expect(card.getByTestId('candle-collection-status')).toContainText('4페이지 · 목표 확보')
  expect(requests.map((request) => new URL(request).searchParams.get('exchange'))).toEqual(['NAS', 'NYS', 'NYS', 'NYS', 'NYS'])
  expect(previews[0].candles).toEqual(ordered.slice(-50))
  expect(previews[0].historyCandles).toEqual(ordered.slice(0, 252))
})

for (const code of ['SCOPE_MISMATCH', 'INVALID_CURSOR', 'CONFIG_NOT_READY']) {
  test(`KIS overseas ${code} does not trigger an exchange fallback`, async ({ page }) => {
    const requests: string[] = []
    const previews: unknown[] = []
    await mockApi(page, { genericSymbol: 'SOXL', chartRequests: requests, genericPreviewRequests: previews,
      candlePageError: { page: 0, code } })
    const card = await genericCard(page, 50)
    await card.getByRole('button', { name: '미리보기 계산' }).click()
    await expect(card.getByText('mock page failure')).toBeVisible()
    expect(requests).toHaveLength(1)
    expect(previews).toHaveLength(0)
  })
}

test('KIS overseas provider failure after a successful page remains a partial result on the original exchange', async ({ page }) => {
  const requests: string[] = []
  const previews: unknown[] = []
  await mockApi(page, { genericSymbol: 'SOXL', chartRequests: requests, genericPreviewRequests: previews,
    candlePages: [{ candles: historyFixture(100) }, { candles: [] }],
    candlePageError: { page: 1, code: 'CANDLE_PAGE_FAILED' } })
  const card = await genericCard(page, 50)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('candle-collection-status')).toContainText('KIS NAS 일봉')
  await expect(card.getByTestId('candle-collection-status')).toContainText('일부 페이지 조회 실패')
  await expect(card.getByTestId('strategy-evaluation-range')).toContainText('실제 평가 50봉 · 사전 자료 50봉')
  expect(requests.map((request) => new URL(request).searchParams.get('exchange'))).toEqual(['NAS', 'NAS'])
  expect(previews).toHaveLength(1)
})
