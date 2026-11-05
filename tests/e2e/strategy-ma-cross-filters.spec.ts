import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { short_period: 7, long_period: 30 }
const filterParams = { long_trend_period: 200, min_long_ma_slope_pct: 0.1, min_atr_pct: 0.5, min_bandwidth_pct: 2 }
function response(body: any, continuous: boolean, diagnostic = true) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-18T12:00:00+09:00', message: 'ma_cross fixture',
    ...(diagnostic ? { entryFilterPreparation: { enabled: true, ready: true, status: 'ready', cadence: 'priorCompletedDailyOhlc', requiredCompletedBars: 200, availableCompletedBars: 252, trueRangeSamples: 14, candidates: 9, unreadyCandidates: 1, trendRejections: 2, slopeRejections: 3, atrRejections: 4, bandwidthRejections: 5, minMoveRejections: 0, passedCandidates: 2 } } : {}),
    replay: { engineVersion: 'strategy-replay-v17', strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null,
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'MaCrossStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void; saveDelay?: Promise<void>; saveStarted?: () => void; diagnostic?: boolean; toss?: boolean } = {}) {
  const requests: any[] = [], updates: any[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { activeBroker: options.toss ? 'toss' : 'kis', chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  const strategy = () => ({ id: 'ma_cross_default', name: 'MaCrossStrategy', enabled: options.enabled ?? false, brokerId: options.toss ? 'toss' : 'kis', brokerAccountId: options.toss ? '1' : '12345678-01', targetSymbols: ['000005'], targetSymbolNames: { '000005': 'Mock ma_cross' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/ma_cross_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    options.saveStarted?.(); if (options.saveDelay) await options.saveDelay
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: 'MA 교차 저장 실패' } }); return }
    stored = body.params; await route.fulfill({ json: strategy() })
  })
  await page.route('**/api/strategy/preview{,/continuous}', async (route) => {
    const body = route.request().postDataJSON(), continuous = route.request().url().endsWith('/continuous')
    requests.push(body); options.started?.()
    if (options.delay) await options.delay
    await route.fulfill({ json: response(body, continuous, options.diagnostic ?? true) })
  })
  await page.goto('/strategy')
  return { requests, updates }
}
const labels = ['진입 완료 일봉 추세 MA 기간', '최소 장기 일봉 MA 기울기 (%)', '최소 일봉 ATR 비율 (%)', '최소 일봉 볼린저 밴드폭 (%)'] as const
const runButton = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
const saveButton = (page: Page) => card(page).getByRole('button', { name: '변경사항 저장', exact: true })
async function setFilters(page: Page) {
  for (const [index, value] of ['200', '0.1', '0.5', '2'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
}
async function continuous(page: Page) {
  await card(page).getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await card(page).getByLabel('연속 시작일').fill('2025-01-01')
  await card(page).getByLabel('연속 종료일').fill('2025-12-31')
}

test('legacy custom cross settings preserve omitted filters and disclose separate daily context through save/reload', async ({ page }) => {
  const { requests, updates } = await setup(page, { diagnostic: false })
  const panel = card(page), controls = panel.getByTestId('ma-cross-entry-controls')
  await expect(panel.getByLabel('단기 MA', { exact: true })).toHaveValue('7')
  await expect(panel.getByLabel('장기 MA', { exact: true })).toHaveValue('30')
  for (const label of labels) await expect(panel.getByLabel(label, { exact: true })).toHaveValue('0')
  for (const text of ['골든크로스 매수 후보에만', '거절한 교차를 나중에 다시 살리지', '5/20일을 뜻하지', '현재 봉은 판단 후 추가', '가격 tick이나 분봉으로 일봉을 합성하지', 'long_period+1봉', 'TR14 산술평균', '모집단 표준편차', '준비 완료지만 양수 임계값에서는 거부', '통과는 체결이 아닙니다']) await expect(controls).toContainText(text)
  await runButton(page).click()
  await expect(panel.getByTestId('ma-cross-entry-preparation')).toContainText('네 조건이 모두 누락/0')
  expect(requests[0].params).toMatchObject(legacyParams)
  expect(requests[0].params).not.toHaveProperty('min_atr_pct')
  await panel.getByLabel('1회 수량', { exact: true }).fill('2')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params).toMatchObject(legacyParams)
  await page.reload()
  await expect(panel.getByLabel('장기 MA', { exact: true })).toHaveValue('30')
  expect(updates[0].params).not.toHaveProperty('min_bandwidth_pct')
})

test('four optional filters survive scoped save and reload with numeric types', async ({ page }) => {
  const { updates } = await setup(page)
  await setFilters(page); await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...filterParams } })
  await page.reload()
  for (const [index, value] of ['200', '0.1', '0.5', '2'].entries()) await expect(card(page).getByLabel(labels[index], { exact: true })).toHaveValue(value)
})

test('short and continuous params and A/B snapshots retain filters and server preparation counters', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await runButton(page).click(); await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await setFilters(page); await continuous(page); await runButton(page).click()
  await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...filterParams })
  expect(requests[1]).not.toHaveProperty('candles')
  const diagnostic = panel.getByTestId('ma-cross-entry-preparation').first()
  for (const text of ['골든크로스 후보 9', '준비 미확인 보류 1', '추세 거부 2', '기울기 거부 3', 'ATR 거부 4', '밴드폭 거부 5', '후보 통과 2', '실제 보유 검사 전']) await expect(diagnostic).toContainText(text)
  await expect(diagnostic).not.toContainText('미보유 후보')
  await expect(diagnostic).not.toContainText('누적 상승률')
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('A와 B의 MA 교차 파라미터가 다릅니다')
  await expect(ab).toContainText('이전 일봉 추세 0봉'); await expect(ab).toContainText('이전 일봉 추세 200봉')
  await expect(ab).toContainText('기울기 거부 3')
  const snapshots = await page.evaluate(() => Object.values(localStorage).filter((value) => value.includes('ma_cross_default')).map((value) => JSON.parse(value)))
  expect(snapshots[0].B.params).toMatchObject(filterParams)
  expect(snapshots[0].B.entryFilterPreparation).toMatchObject({ slopeRejections: 3, bandwidthRejections: 5 })
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '단기 구간 · 최근 50/100/200봉', exact: true }).click()
  await runButton(page).click(); await expect.poll(() => requests.length).toBe(3)
  expect(requests[2].params).toMatchObject(filterParams)
})

test('active legacy response without diagnostics remains unknown in current and A/B', async ({ page }) => {
  await setup(page, { params: { ...legacyParams, ...filterParams }, diagnostic: false })
  await runButton(page).click()
  await expect(card(page).getByTestId('ma-cross-entry-preparation')).toContainText('준비 정보 없음')
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await expect(card(page).getByTestId('strategy-ab-comparison')).toContainText('준비 정보 없음')
})

test('invalid raw drafts block saving and both previews and invalidate an earlier result', async ({ page }) => {
  const { requests } = await setup(page)
  await runButton(page).click(); await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const [label, invalids] of [[labels[0], ['', 'NaN', 'Infinity', '-1', '513', '1.5']], [labels[1], ['101']], [labels[2], ['101']], [labels[3], ['1001']]] as const) {
    const input = card(page).getByLabel(label, { exact: true })
    for (const raw of invalids) {
      await input.fill(raw); await expect(input).toHaveValue(raw)
      await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
      await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
      await input.fill('0'); await expect(runButton(page)).toBeEnabled()
    }
  }
  await continuous(page); await card(page).getByLabel(labels[0], { exact: true }).fill('0.5')
  await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
  expect(requests).toHaveLength(1)
})

for (const [name, invalid] of [
  ['stored numeric string', { min_atr_pct: '0.5' }], ['stored null', { min_bandwidth_pct: null }],
  ['stored fractional integer', { long_trend_period: 1.5 }], ['stored boolean', { min_long_ma_slope_pct: false }],
] as const) {
  test(`${name} fails closed until a real edit repairs stored optional types`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...legacyParams, ...invalid } })
    await expect(runButton(page)).toBeDisabled()
    await card(page).getByLabel('1회 수량', { exact: true }).fill('2'); await expect(saveButton(page)).toBeDisabled()
    for (const label of labels) await card(page).getByLabel(label, { exact: true }).fill('')
    await setFilters(page); await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
    expect(updates[0].params).toMatchObject({ ...legacyParams, ...filterParams })
  })
}

test('slope requires integer legacy long period within 1..511 only while active', async ({ page }) => {
  await setup(page)
  const period = card(page).getByLabel('장기 MA', { exact: true })
  await card(page).getByLabel(labels[1], { exact: true }).fill('0.1')
  for (const value of ['0', '512', '1.5']) {
    await period.fill(value); await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
    await expect(card(page)).toContainText('1~511 정수여야')
  }
  for (const value of ['1', '511']) { await period.fill(value); await expect(runButton(page)).toBeEnabled() }
  await period.fill('512'); await card(page).getByLabel(labels[1], { exact: true }).fill('0')
  await expect(runButton(page)).toBeEnabled()
})

test('save failure preserves filters and permits identical scoped retry', async ({ page }) => {
  const { updates } = await setup(page, { failSave: true })
  await setFilters(page); await saveButton(page).click()
  await expect(page.getByRole('alert').filter({ hasText: 'MA 교차 저장 실패' })).toBeVisible()
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('0.1')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toEqual(updates[0].params)
})

test('late response is discarded after invalid editing and mobile fields do not overflow', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 390, height: 844 }); await setup(page, { delay, started })
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await runButton(page).click(); await entered
  await card(page).getByLabel(labels[2], { exact: true }).fill('NaN')
  release(); await (await finished).finished()
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  expect(await card(page).getByTestId('ma-cross-entry-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
})

test('enabled strategy locks all four new fields', async ({ page }) => {
  await setup(page, { enabled: true })
  for (const label of labels) await expect(card(page).getByLabel(label, { exact: true })).toBeDisabled()
})

test('keyboard decimal drafts and inclusive maximum boundaries reach save', async ({ page }) => {
  const { updates } = await setup(page)
  const input = card(page).getByLabel(labels[1], { exact: true })
  await input.fill(''); await input.pressSequentially('0.'); await expect(input).toHaveValue('0.')
  await input.pressSequentially('1'); await expect(input).toHaveValue('0.1')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params.min_long_ma_slope_pct).toBe(0.1); await expect(saveButton(page)).toHaveCount(0)
  for (const [index, value] of ['512', '100', '100', '1000'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toMatchObject({ long_trend_period: 512, min_long_ma_slope_pct: 100, min_atr_pct: 100, min_bandwidth_pct: 1000 })
})

test('a delayed save preserves newer filter drafts for a second save', async ({ page }) => {
  let release!: () => void, started!: () => void
  const saveDelay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const { updates } = await setup(page, { saveDelay, saveStarted: started })
  await setFilters(page); await saveButton(page).click(); await entered
  await card(page).getByLabel(labels[2], { exact: true }).fill('0.7')
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategies/ma_cross_default'))
  release(); await (await finished).finished(); await expect(saveButton(page)).toBeEnabled()
  await expect(card(page).getByLabel(labels[2], { exact: true })).toHaveValue('0.7')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2)
  expect(updates[0].params).toMatchObject(filterParams); expect(updates[1].params).toMatchObject({ ...filterParams, min_atr_pct: 0.7 })
  await expect(saveButton(page)).toHaveCount(0)
})

test('Toss minute preview is blocked only with active daily filters', async ({ page }) => {
  const { requests } = await setup(page, { toss: true })
  const panel = card(page)
  await panel.getByRole('combobox', { name: /^봉 단위/ }).click()
  await page.getByRole('option', { name: '1분봉', exact: true }).click()
  await expect(runButton(page)).toBeEnabled()
  await panel.getByLabel(labels[0], { exact: true }).fill('200')
  await expect(runButton(page)).toBeDisabled(); await expect(panel).toContainText('MA 교차 진입 필터는 이전 완료 일봉이 필요합니다')
  await panel.getByLabel(labels[0], { exact: true }).fill('0'); await expect(runButton(page)).toBeEnabled()
  await runButton(page).click(); await expect.poll(() => requests.length).toBe(1)
  expect(requests[0].interval).toBe('1m')
})

for (const [name, params] of [
  ['raw null short', { short_period: null, long_period: 20 }],
  ['raw string long', { short_period: 5, long_period: '20' }],
  ['fractional short', { short_period: 1.5, long_period: 20 }],
  ['partial nondefault long', { long_period: 8 }],
] as const) {
  test(`slope ${name} preserves stored type and blocks inconsistent legacy fallback`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...params, min_long_ma_slope_pct: 0.1 } })
    await expect(runButton(page)).toBeDisabled()
    await card(page).getByLabel('1회 수량', { exact: true }).fill('2')
    await expect(saveButton(page)).toBeDisabled()
    await card(page).getByLabel('단기 MA', { exact: true }).fill('5')
    await card(page).getByLabel('장기 MA', { exact: true }).fill('')
    await card(page).getByLabel('장기 MA', { exact: true }).fill('20')
    await expect(runButton(page)).toBeEnabled(); await saveButton(page).click()
    await expect.poll(() => updates.length).toBe(1)
    expect(updates[0].params).toMatchObject({ short_period: 5, long_period: 20, min_long_ma_slope_pct: 0.1 })
  })
}
