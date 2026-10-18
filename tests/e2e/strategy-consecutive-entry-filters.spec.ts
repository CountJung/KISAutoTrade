import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle, EntryFilterPreparation } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { buy_days: 2, sell_days: 4 }
const filterParams = { min_cumulative_move_pct: 1.2, min_move_atr_multiplier: 0.5, long_trend_period: 200 }
function response(body: any, continuous: boolean, diagnostic?: EntryFilterPreparation, sizingPreparation?: unknown) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-18T12:00:00+09:00', message: 'consecutive fixture',
    replay: { engineVersion: 'strategy-replay-v13', strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null, ...(sizingPreparation ? { atrPreparation: sizingPreparation } : {}),
    },
    ...(diagnostic ? { entryFilterPreparation: diagnostic } : {}),
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'ConsecutiveMoveStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void; diagnostic?: EntryFilterPreparation; sizingPreparation?: unknown } = {}) {
  const requests: any[] = [], updates: any[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  const strategy = () => ({ id: 'consecutive_move_default', name: 'ConsecutiveMoveStrategy', enabled: options.enabled ?? false, brokerId: 'kis', brokerAccountId: '12345678-01', targetSymbols: ['000007'], targetSymbolNames: { '000007': 'Mock consecutive' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/consecutive_move_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: '연속 전략 저장 실패' } }); return }
    stored = body.params; await route.fulfill({ json: strategy() })
  })
  await page.route('**/api/strategy/preview{,/continuous}', async (route) => {
    const body = route.request().postDataJSON(), continuous = route.request().url().endsWith('/continuous')
    requests.push(body); options.started?.()
    if (options.delay) await options.delay
    await route.fulfill({ json: response(body, continuous, options.diagnostic, options.sizingPreparation) })
  })
  await page.goto('/strategy')
  return { requests, updates }
}
async function fillFilters(page: Page) {
  const panel = card(page)
  await panel.getByLabel('최소 누적 상승률 (%)', { exact: true }).fill('1.2')
  await panel.getByLabel('최소 상승폭 ATR 배수', { exact: true }).fill('0.5')
  await panel.getByLabel('진입 장기 MA 기간', { exact: true }).fill('200')
}

test('legacy custom observation counts survive new disabled filters and scoped save/reload', async ({ page }) => {
  const { updates } = await setup(page)
  const panel = card(page)
  await expect(panel.getByLabel('연속 상승 관측 횟수', { exact: true })).toHaveValue('2')
  await expect(panel.getByLabel('연속 하락 관측 횟수', { exact: true })).toHaveValue('4')
  for (const label of ['최소 누적 상승률 (%)', '최소 상승폭 ATR 배수', '진입 장기 MA 기간']) await expect(panel.getByLabel(label, { exact: true })).toHaveValue('0')
  await expect(panel.getByTestId('consecutive-entry-controls')).toContainText('약10초 가격관측')
  await expect(panel.getByTestId('consecutive-entry-controls')).toContainText('기존 연속 하락 청산을 막지 않습니다')
  await fillFilters(page)
  await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...filterParams } })
  await page.reload()
  await expect(panel.getByLabel('최소 상승폭 ATR 배수', { exact: true })).toHaveValue('0.5')
  await expect(panel.getByLabel('연속 하락 관측 횟수', { exact: true })).toHaveValue('4')
})

test('filter parameters reach both bounded and continuous previews and A/B retains conditions', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await fillFilters(page)
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  expect(requests[0].params).toMatchObject({ ...legacyParams, ...filterParams })
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await panel.getByLabel('연속 시작일').fill('2025-01-01')
  await panel.getByLabel('연속 종료일').fill('2025-12-31')
  await panel.getByLabel('최소 누적 상승률 (%)', { exact: true }).fill('2')
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...filterParams, min_cumulative_move_pct: 2 })
  expect(requests[1]).not.toHaveProperty('candles')
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  await expect(panel.getByText('A와 B의 전략 파라미터가 다릅니다. 진입 필터 조건을 함께 확인하세요.')).toBeVisible()
  const snapshots = await page.evaluate(() => Object.values(localStorage).filter((value) => value.includes('consecutive_move_default')).map((value) => JSON.parse(value)))
  expect(JSON.stringify(snapshots)).toContain('"min_move_atr_multiplier":0.5')
  await expect(panel.getByText(/신규 진입 필터 · 누적 상승률 1.2%/)).toBeVisible()
})

test('invalid raw drafts clear the preview and block both save and calculation until corrected', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page), run = panel.getByRole('button', { name: '미리보기 계산', exact: true })
  await run.click()
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const [label, raw] of [['최소 누적 상승률 (%)', 'NaN'], ['최소 누적 상승률 (%)', '101'], ['최소 상승폭 ATR 배수', '21'], ['최소 상승폭 ATR 배수', ''], ['진입 장기 MA 기간', '2.5'], ['진입 장기 MA 기간', '513']]) {
    const input = panel.getByLabel(label, { exact: true })
    await input.fill(raw)
    await expect(input).toHaveValue(raw)
    await expect(run).toBeDisabled()
    await expect(panel.getByRole('button', { name: '변경사항 저장', exact: true })).toBeDisabled()
    await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
    await input.fill('0')
    await expect(run).toBeEnabled()
  }
  expect(requests).toHaveLength(1)
})

test('stored invalid filters fail closed and save failure keeps a valid edited draft', async ({ page }) => {
  const { updates } = await setup(page, { params: { ...legacyParams, min_move_atr_multiplier: '0.5' }, failSave: true })
  const panel = card(page)
  await expect(panel.getByRole('button', { name: '미리보기 계산', exact: true })).toBeDisabled()
  await panel.getByLabel('최소 상승폭 ATR 배수', { exact: true }).fill('')
  await panel.getByLabel('최소 상승폭 ATR 배수', { exact: true }).fill('0.5')
  await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect(page.getByRole('alert').filter({ hasText: '연속 전략 저장 실패' })).toBeVisible()
  await expect(panel.getByLabel('최소 상승폭 ATR 배수', { exact: true })).toHaveValue('0.5')
  await expect(panel.getByRole('button', { name: '변경사항 저장', exact: true })).toBeEnabled()
  await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params.min_move_atr_multiplier).toBe(0.5)
})

test('ATR and MA require daily preview while zero disabled permits other observation intervals', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await panel.getByRole('combobox', { name: /^봉 단위/ }).click()
  await page.getByRole('option', { name: '주봉', exact: true }).click()
  await expect(panel.getByRole('button', { name: '미리보기 계산', exact: true })).toBeEnabled()
  await panel.getByLabel('최소 상승폭 ATR 배수', { exact: true }).fill('0.5')
  await expect(panel.getByRole('alert').filter({ hasText: '일봉을 선택하세요.' })).toBeVisible()
  await expect(panel.getByRole('button', { name: '미리보기 계산', exact: true })).toBeDisabled()
  await panel.getByLabel('최소 상승폭 ATR 배수', { exact: true }).fill('0')
  await panel.getByLabel('진입 장기 MA 기간', { exact: true }).fill('20')
  await expect(panel.getByRole('button', { name: '미리보기 계산', exact: true })).toBeDisabled()
  await panel.getByLabel('진입 장기 MA 기간', { exact: true }).fill('0')
  await expect(panel.getByRole('button', { name: '미리보기 계산', exact: true })).toBeEnabled()
  expect(requests).toHaveLength(0)
})

test('invalid edit invalidates an in-flight response and mobile controls stay within the card', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delayed = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 390, height: 844 })
  await setup(page, { delay: delayed, started })
  const panel = card(page)
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await entered
  await panel.getByLabel('진입 장기 MA 기간', { exact: true }).fill('1.5')
  release(); await (await finished).finished()
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  const overflow = await panel.getByTestId('consecutive-entry-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)
  expect(overflow).toBe(false)
})

test('enabled strategy filters stay locked', async ({ page }) => {
  await setup(page, { enabled: true })
  for (const label of ['최소 누적 상승률 (%)', '최소 상승폭 ATR 배수', '진입 장기 MA 기간']) await expect(card(page).getByLabel(label, { exact: true })).toBeDisabled()
})


const insufficientEntry: EntryFilterPreparation = {
  enabled: true, ready: false, status: 'insufficientHistory', cadence: 'priorCompletedDailyOhlc',
  requiredCompletedBars: 200, availableCompletedBars: 12, trueRangeSamples: 11,
  candidates: 7, unreadyCandidates: 3, minMoveRejections: 2, atrRejections: 1, trendRejections: 2, passedCandidates: 2,
}

test('entry OHLC readiness and candidate diagnostics stay separate from sizing ATR and survive A/B storage', async ({ page }) => {
  await setup(page, {
    params: { ...legacyParams, ...filterParams }, diagnostic: insufficientEntry,
    sizingPreparation: { period: 14, historyBars: 252, readyAtStart: true, readyAtEnd: true, firstReadyTime: '20250101', unreadyEvaluationBars: 0, rangePolicy: 'observedBarsWithPreviousCloseNoInterpolation' },
  })
  const panel = card(page)
  await panel.getByRole('checkbox', { name: '백테스트 ATR 기반 수량 산정', exact: true }).check()
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  const entry = panel.getByTestId('entry-filter-preparation').first()
  await expect(entry).toContainText('신규 진입 필터 준비 · 완료 자료 부족')
  await expect(entry).toContainText('필요 완료봉 200 · 확보 12 · 실제 TR 11개')
  await expect(entry).toContainText('준비 미확인 보류 3')
  await expect(entry).toContainText('ATR 거부 1')
  await expect(entry).toContainText('거부 사유는 중복 집계될 수 있고 후보 통과는 실제 체결이 아닙니다')
  await expect(panel.getByTestId('backtest-atr-preparation')).toContainText('시작 준비 · 종료 준비')
  await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  const saved = await page.evaluate(() => Object.entries(localStorage).filter(([key]) => key.includes('consecutive_move_default')).map(([, value]) => JSON.parse(value)))
  expect(saved[0].A.entryFilterPreparation).toEqual(insufficientEntry)
  await page.evaluate(() => {
    for (const key of Object.keys(localStorage)) {
      if (!key.includes('consecutive_move_default')) continue
      const saved = JSON.parse(localStorage.getItem(key)!)
      delete saved.A.entryFilterPreparation
      localStorage.setItem(key, JSON.stringify(saved))
    }
  })
  await page.reload()
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await expect(panel.getByTestId('entry-filter-preparation').filter({ hasText: '진입 필터 준비 정보 없음' })).toBeVisible()
})

test('legacy response without entry metadata never borrows sizing ATR readiness', async ({ page }) => {
  await setup(page, { params: { ...legacyParams, min_move_atr_multiplier: 0.5 } })
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  const entry = panel.getByTestId('entry-filter-preparation')
  await expect(entry).toContainText('진입 필터 준비 정보 없음')
  await expect(entry).toContainText('수량 산정 ATR 정보로 대신 확인하지 않습니다')
  await expect(entry).not.toContainText('준비 완료')
})


test('decimal keyboard drafts preserve the dot while only finite numeric values reach save', async ({ page }) => {
  const { updates } = await setup(page)
  const panel = card(page), percent = panel.getByLabel('최소 누적 상승률 (%)', { exact: true })
  await percent.fill('')
  await percent.pressSequentially('1.')
  await expect(percent).toHaveValue('1.')
  await percent.pressSequentially('2')
  await expect(percent).toHaveValue('1.2')
  const atr = panel.getByLabel('최소 상승폭 ATR 배수', { exact: true })
  await atr.fill('')
  await atr.pressSequentially('.5')
  await expect(atr).toHaveValue('.5')
  await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params).toMatchObject({ ...legacyParams, min_cumulative_move_pct: 1.2, min_move_atr_multiplier: 0.5 })
})


test('known disabled legacy defaults omit unused preparation without an unknown warning in current and saved results', async ({ page }) => {
  await setup(page)
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  const notices = panel.getByTestId('entry-filter-preparation')
  await expect(notices).toHaveCount(1)
  await expect(notices).toContainText('신규 진입 필터 비활성')
  await expect(notices).toHaveClass(/MuiAlert-standardInfo/)
  await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await expect(notices).toHaveCount(2)
  await expect(notices.nth(1)).toContainText('신규 진입 필터 비활성')
  await expect(panel.getByText(/활성 필터의 이전\/생략 응답입니다/)).toHaveCount(0)
})
