import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle } from '../../src/api/types'
import { mockApi } from './fixtures/strategyResearchFixture'

function dailySeries(from = '2022-01-01', to = '2025-12-31'): ChartCandle[] {
  const rows: ChartCandle[] = []
  for (let instant = Date.parse(`${from}T00:00:00Z`); instant <= Date.parse(`${to}T00:00:00Z`); instant += 86_400_000) {
    rows.push({ date: new Date(instant).toISOString().slice(0, 10).replace(/-/g, ''), open: '100', high: '102', low: '99', close: '101', volume: '1000' })
  }
  return rows
}

type SetupOptions = {
  data?: ChartCandle[]
  requests?: Record<string, any>[]
  flatRequests?: Record<string, any>[]
  pages?: string[]
  delayMs?: number
  pageError?: { page: number; code: string }
  mismatchScope?: boolean
}

function researchResponse(body: Record<string, any>, continuous: boolean) {
  const candles = continuous ? body.chunks.flatMap((chunk: { candles: ChartCandle[] }) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  const midpoint = Math.max(1, Math.floor(candles.length * 0.7))
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-18T12:00:00+09:00',
    message: continuous ? 'fixture continuous single ledger' : 'fixture bounded window',
    replay: {
      engineVersion: 'strategy-replay-v12', strategyVersion: 'fixture', sourceInterval: 'D', replayCadence: 'dailyCloseWithDayBoundary',
      liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last,
      dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: continuous ? 'fixture-continuous-hash' : 'fixture-bounded-hash',
    },
    backtest: {
      assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0, unfilledOrderCount: 0, ambiguousBarCount: 0 },
      trades: [], equityCurve: [], overfitWarning: null, executionWarnings: [],
      phases: [
        { phase: 'inSample', start: first, end: candles[midpoint - 1].date, returnPct: 0, mddPct: 0, completedTrades: 0, winRatePct: 0 },
        { phase: 'outOfSample', start: candles[midpoint]?.date ?? last, end: last, returnPct: 0, mddPct: 0, completedTrades: 0, winRatePct: 0 },
      ],
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: { candles: ChartCandle[] }, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}

async function setup(page: Page, options: SetupOptions = {}) {
  await mockApi(page, {
    chartCandles: options.data ?? dailySeries(), chartRequests: options.pages,
    candlePageError: options.pageError,
    ...(options.mismatchScope ? { candlePages: [{}, { scopeKey: 'other-scope' }] } : {}),
  })
  await page.route('**/api/strategy/preview/continuous', async (route) => {
    const body = route.request().postDataJSON()
    options.requests?.push(body)
    if (options.delayMs) await new Promise((resolve) => setTimeout(resolve, options.delayMs))
    await route.fulfill({ json: researchResponse(body, true) })
  })
  await page.route('**/api/strategy/preview', async (route) => {
    const body = route.request().postDataJSON()
    options.flatRequests?.push(body)
    await route.fulfill({ json: researchResponse(body, false) })
  })
}
function card(page: Page) { return page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first() }
async function chooseContinuous(page: Page, start = '2023-01-01', end = '2025-12-31') {
  const panel = card(page)
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await panel.getByLabel('연속 시작일').fill(start)
  await panel.getByLabel('연속 종료일').fill(end)
  return panel
}

test('continuous daily range collects more than the old eight pages and submits bounded chronological chunks once', async ({ page }) => {
  const requests: Record<string, any>[] = [], flatRequests: Record<string, any>[] = [], pages: string[] = []
  await setup(page, { requests, flatRequests, pages })
  await page.goto('/strategy')
  const panel = await chooseContinuous(page)
  await expect(panel.getByRole('combobox', { name: '봉 단위' })).toBeDisabled()
  await expect(panel.getByRole('combobox', { name: '분석 구간' })).toBeDisabled()
  await expect(panel.getByRole('spinbutton', { name: '앞 구간 비율(%)' })).toHaveValue('70')
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests).toHaveLength(1)
  expect(flatRequests).toHaveLength(0)
  expect(pages.length).toBeGreaterThan(8)
  expect(pages.length).toBeLessThanOrEqual(48)
  const body = requests[0]
  expect(body.interval).toBe('D')
  expect(body.historyCandles).toHaveLength(252)
  expect(body.historyCandles.every((row: ChartCandle) => row.date < '20230101')).toBe(true)
  expect(body.chunks.map((chunk: { candles: ChartCandle[] }) => chunk.candles.length)).toEqual([500, 500, 96])
  const evaluation = body.chunks.flatMap((chunk: { candles: ChartCandle[] }) => chunk.candles)
  expect(evaluation[0].date).toBe('20230101')
  expect(evaluation[evaluation.length - 1].date).toBe('20251231')
  expect(evaluation.every((row: ChartCandle, index: number) => index === 0 || row.date > evaluation[index - 1].date)).toBe(true)
  expect(body).not.toHaveProperty('warmupCount')
  expect(body).not.toHaveProperty('candles')
  await expect(panel.getByTestId('phase-policy')).toContainText('독립 검증 성과로 해석하지 않습니다')
  await expect(panel.getByRole('table', { name: '동일 장부 구간 성과' })).toContainText('앞 구간 · 동일 장부')
  await expect(panel.getByTestId('continuous-collection-status')).toContainText('coverageVerified=false')
})

test('insufficient strict prior history blocks calculation without stealing evaluation bars', async ({ page }) => {
  const requests: Record<string, any>[] = []
  await setup(page, { data: dailySeries('2022-12-20'), requests })
  await page.goto('/strategy')
  const panel = await chooseContinuous(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  const notice = panel.getByTestId('continuous-collection-status')
  await expect(notice).toContainText('계산 보류')
  await expect(notice).toContainText('사전자료 12/252봉')
  await expect(notice).toContainText('평가 1096봉')
  expect(requests).toHaveLength(0)
})

for (const scenario of ['stale end', 'internal gap'] as const) {
  test(`conservative seven-day coverage gate blocks ${scenario} without synthesizing missing candles`, async ({ page }) => {
    const data = scenario === 'stale end' ? dailySeries('2022-01-01', '2025-03-31') : dailySeries().filter((row) => row.date < '20240101' || row.date > '20240120')
    const requests: Record<string, any>[] = []
    await setup(page, { data, requests })
    await page.goto('/strategy')
    const panel = await chooseContinuous(page)
    await panel.getByRole('button', { name: '미리보기 계산' }).click()
    await expect(panel.getByTestId('continuous-collection-status')).toContainText('계산 보류')
    await expect(panel.getByTestId('continuous-collection-status')).toContainText('7달력일 상한')
    await expect(panel.getByTestId('continuous-collection-status')).toContainText('긴 휴장도 계산이 보류')
    expect(requests).toHaveLength(0)
  })
}

test('incomplete provider pages and changed scope cannot publish a continuous performance result', async ({ page }) => {
  const requests: Record<string, any>[] = []
  await setup(page, { requests, pageError: { page: 1, code: 'CANDLE_PAGE_FAILED' } })
  await page.goto('/strategy')
  const panel = await chooseContinuous(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('continuous-collection-status')).toContainText('계산 보류')
  expect(requests).toHaveLength(0)
  await setup(page, { requests, mismatchScope: true })
  await page.reload()
  await chooseContinuous(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByText('캔들 페이지의 프로파일·계좌·조회 기준이 변경되어 결과를 폐기했습니다.', { exact: true })).toBeVisible()
  await expect(panel.getByTestId('strategy-backtest-results')).toHaveCount(0)
  expect(requests).toHaveLength(0)
})

test('cancelled continuous calculation cannot replace a later short-window result', async ({ page }) => {
  const requests: Record<string, any>[] = []
  await setup(page, { requests, delayMs: 800 })
  const lateResponse = page.waitForResponse('**/api/strategy/preview/continuous')
  await page.goto('/strategy')
  const panel = await chooseContinuous(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect.poll(() => requests.length).toBe(1)
  await panel.getByRole('button', { name: '취소', exact: true }).click()
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '단기 구간 · 최근 50/100/200봉', exact: true }).click()
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByText('fixture bounded window', { exact: true })).toBeVisible()
  await (await lateResponse).finished()
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => resolve())))
  await expect(panel.getByText('fixture bounded window', { exact: true })).toBeVisible()
  await expect(panel.getByTestId('continuous-replay-continuity')).toHaveCount(0)
})

test('A/B persists calculation mode and continuity independently of legacy saved snapshots', async ({ page }) => {
  await setup(page)
  await page.goto('/strategy')
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await panel.getByRole('button', { name: '현재 결과를 A로 저장' }).click()
  await page.evaluate(() => {
    for (const key of Object.keys(localStorage)) if (key.startsWith('act:strategy-experiments:v1:') && !key.endsWith(':index')) {
      const slots = JSON.parse(localStorage.getItem(key)!)
      if (slots.A) { delete slots.A.calculationMode; delete slots.A.continuity; localStorage.setItem(key, JSON.stringify(slots)) }
    }
  })
  await page.reload()
  await chooseContinuous(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await panel.getByRole('button', { name: '현재 결과를 B로 저장' }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('계산 범위 정보 없음 · 이전 결과')
  await expect(ab).toContainText('연속 일봉 장부 · 공급 3묶음 / 평가 1096봉 / 사전 252봉 1회')
  const saved = await page.evaluate(() => Object.keys(localStorage).filter((key) => key.startsWith('act:strategy-experiments:v1:') && !key.endsWith(':index')).map((key) => JSON.parse(localStorage.getItem(key)!)))
  expect(saved[0].B.calculationMode).toBe('continuous')
  expect(saved[0].B.continuity.chunks).toHaveLength(3)
  await page.reload()
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('strategy-ab-comparison')).toContainText('연속 일봉 장부')
})

test('invalid date range disables calculation and mobile continuous controls stay within the card', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await setup(page)
  await page.goto('/strategy')
  const panel = await chooseContinuous(page)
  await panel.getByLabel('연속 시작일').fill('2026-01-01')
  await expect(panel.getByRole('button', { name: '미리보기 계산' })).toBeDisabled()
  await panel.getByLabel('연속 시작일').fill('2023-01-01')
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  const metrics = await panel.evaluate((element) => ({ width: element.clientWidth, scroll: element.scrollWidth }))
  expect(metrics.scroll).toBeLessThanOrEqual(metrics.width + 1)
})


for (const policy of ['page cap', 'evaluation cap'] as const) {
  test(`continuous collector enforces the hard ${policy} and never sends partial oversized input`, async ({ page }) => {
    const requests: Record<string, any>[] = [], pages: string[] = []
    await setup(page, { data: dailySeries('1990-01-01', '2025-12-31'), requests, pages })
    await page.goto('/strategy')
    const panel = await chooseContinuous(page, '1990-01-01', policy === 'page cap' ? '2005-12-31' : '2025-12-31')
    await panel.getByRole('button', { name: '미리보기 계산' }).click()
    await expect(panel.getByTestId('continuous-collection-status')).toContainText('계산 보류')
    expect(requests).toHaveLength(0)
    expect(pages.length).toBeLessThanOrEqual(48)
    if (policy === 'page cap') {
      expect(pages.length).toBe(48)
      await expect(panel.getByTestId('continuous-collection-status')).toContainText('최대 48페이지 도달')
    } else await expect(panel.getByTestId('continuous-collection-status')).toContainText('연속 평가 4096봉 상한')
  })
}
