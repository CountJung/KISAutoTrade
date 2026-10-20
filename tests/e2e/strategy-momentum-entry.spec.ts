import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { lookback_period: 12, threshold_pct: 5 }
const crossingParams = { entry_mode: 'crossing', entry_hysteresis_pct: 1 }
function response(body: any, continuous: boolean) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-18T12:00:00+09:00', message: 'momentum fixture',
    replay: { engineVersion: 'strategy-replay-v13', strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null,
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'MomentumStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void } = {}) {
  const requests: any[] = [], updates: any[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  const strategy = () => ({ id: 'momentum_default', name: 'MomentumStrategy', enabled: options.enabled ?? false, brokerId: 'kis', brokerAccountId: '12345678-01', targetSymbols: ['000004'], targetSymbolNames: { '000004': 'Mock momentum' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/momentum_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: 'Momentum 저장 실패' } }); return }
    stored = body.params; await route.fulfill({ json: strategy() })
  })
  await page.route('**/api/strategy/preview{,/continuous}', async (route) => {
    const body = route.request().postDataJSON(), continuous = route.request().url().endsWith('/continuous')
    requests.push(body); options.started?.()
    if (options.delay) await options.delay
    await route.fulfill({ json: response(body, continuous) })
  })
  await page.goto('/strategy')
  return { requests, updates }
}
async function chooseMode(page: Page, mode: 'level' | 'crossing') {
  await card(page).getByRole('combobox', { name: /^Momentum 진입 모드/ }).click()
  await page.getByRole('option', { name: mode === 'level' ? 'level · 임계값 상태 (기본)' : 'crossing · 상향 돌파', exact: true }).click()
}

test('legacy custom parameters retain default level mode and zero unused hysteresis', async ({ page }) => {
  const { requests, updates } = await setup(page)
  const panel = card(page)
  await expect(panel.getByLabel('비교 기간', { exact: true })).toHaveValue('12')
  await expect(panel.getByLabel('임계값 (%)', { exact: true })).toHaveValue('5')
  await expect(panel.getByRole('combobox', { name: /^Momentum 진입 모드/ })).toContainText('level · 임계값 상태 (기본)')
  await expect(panel.getByLabel('재무장 간격 (%p)', { exact: true })).toHaveValue('0')
  await expect(panel.getByTestId('momentum-entry-controls')).toContainText('이전 모멘텀 < 임계값, 현재 모멘텀 ≥ 임계값')
  await expect(panel.getByTestId('momentum-entry-controls')).toContainText('음의 임계값 이하 청산')
  await expect(panel.getByTestId('momentum-entry-controls')).toContainText('실시간은 약10초 가격관측')
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  expect(requests[0].params).toMatchObject(legacyParams)
  expect(requests[0].params).not.toHaveProperty('entry_mode')
  await panel.getByLabel('1회 수량', { exact: true }).fill('2')
  await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params).toMatchObject(legacyParams)
  await page.reload()
  await expect(panel.getByLabel('비교 기간', { exact: true })).toHaveValue('12')
})

test('crossing mode and hysteresis survive scoped save and reloaded params without string coercion', async ({ page }) => {
  const { updates } = await setup(page)
  const panel = card(page)
  await chooseMode(page, 'crossing')
  await panel.getByLabel('재무장 간격 (%p)', { exact: true }).fill('1')
  await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...crossingParams } })
  await page.reload()
  await expect(panel.getByRole('combobox', { name: /^Momentum 진입 모드/ })).toContainText('crossing · 상향 돌파')
  await expect(panel.getByLabel('재무장 간격 (%p)', { exact: true })).toHaveValue('1')
})

test('both preview modes send entry params and A/B clearly retains level-unused versus crossing assumptions', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await panel.getByLabel('재무장 간격 (%p)', { exact: true }).fill('1')
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  expect(requests[0].params).toMatchObject({ ...legacyParams, entry_hysteresis_pct: 1 })
  await chooseMode(page, 'crossing')
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await panel.getByLabel('연속 시작일').fill('2025-01-01')
  await panel.getByLabel('연속 종료일').fill('2025-12-31')
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...crossingParams })
  expect(requests[1]).not.toHaveProperty('candles')
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('A와 B의 Momentum 파라미터가 다릅니다')
  await expect(ab).toContainText('level (기본 level) · 재무장 간격 1%p · level에서 미사용')
  await expect(ab).toContainText('Momentum 진입 모드 · crossing · 재무장 간격 1%p')
  const snapshots = await page.evaluate(() => Object.values(localStorage).filter((value) => value.includes('momentum_default')).map((value) => JSON.parse(value)))
  expect(snapshots[0].B.params.entry_mode).toBe('crossing')
  expect(snapshots[0].B.params.entry_hysteresis_pct).toBe(1)
})

test('invalid numeric drafts preserve raw text and clear old previews even when level ignores hysteresis economically', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page), input = panel.getByLabel('재무장 간격 (%p)', { exact: true }), run = panel.getByRole('button', { name: '미리보기 계산', exact: true })
  await run.click()
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const raw of ['', 'NaN', 'Infinity', '-1', '101']) {
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

test('level preserves an unused draft and crossing alone enforces hysteresis no greater than threshold', async ({ page }) => {
  await setup(page)
  const panel = card(page), input = panel.getByLabel('재무장 간격 (%p)', { exact: true }), run = panel.getByRole('button', { name: '미리보기 계산', exact: true })
  await input.fill('6')
  await expect(run).toBeEnabled()
  await expect(panel.getByTestId('momentum-entry-controls')).toContainText('level에서는 미사용')
  await chooseMode(page, 'crossing')
  await expect(input).toHaveValue('6')
  await expect(run).toBeDisabled()
  await chooseMode(page, 'level')
  await expect(input).toHaveValue('6')
  await expect(run).toBeEnabled()
  await chooseMode(page, 'crossing')
  await input.fill('5')
  await expect(run).toBeEnabled()
  await panel.getByLabel('임계값 (%)', { exact: true }).fill('4')
  await expect(run).toBeDisabled()
  await input.fill('4')
  await expect(run).toBeEnabled()
})

for (const [name, invalid] of [
  ['unknown enum', { entry_mode: 'bogus' }], ['null enum', { entry_mode: null }],
  ['stored numeric string', { entry_hysteresis_pct: '1' }], ['stored null', { entry_hysteresis_pct: null }],
] as const) {
  test(`${name} fails closed until a real edit repairs the stored type`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...legacyParams, ...invalid } })
    const panel = card(page)
    await expect(panel.getByRole('button', { name: '미리보기 계산', exact: true })).toBeDisabled()
    await chooseMode(page, 'crossing')
    const input = panel.getByLabel('재무장 간격 (%p)', { exact: true })
    await input.fill('')
    await input.fill('1')
    await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
    await expect.poll(() => updates.length).toBe(1)
    expect(updates[0].params).toMatchObject({ ...legacyParams, ...crossingParams })
  })
}

test('save failure keeps crossing draft and allows an identical scoped retry', async ({ page }) => {
  const { updates } = await setup(page, { failSave: true })
  const panel = card(page)
  await chooseMode(page, 'crossing')
  await panel.getByLabel('재무장 간격 (%p)', { exact: true }).fill('1')
  await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect(page.getByRole('alert').filter({ hasText: 'Momentum 저장 실패' })).toBeVisible()
  await expect(panel.getByLabel('재무장 간격 (%p)', { exact: true })).toHaveValue('1')
  await panel.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toEqual(updates[0].params)
})

test('invalid edit discards in-flight replay and mobile controls stay inside the card', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 390, height: 844 })
  await setup(page, { delay, started })
  const panel = card(page), finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await panel.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await entered
  await panel.getByLabel('재무장 간격 (%p)', { exact: true }).fill('NaN')
  release(); await (await finished).finished()
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  expect(await panel.getByTestId('momentum-entry-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
})

test('active strategies lock the mode and hysteresis inputs', async ({ page }) => {
  await setup(page, { enabled: true })
  await expect(card(page).getByRole('combobox', { name: /^Momentum 진입 모드/ })).toBeDisabled()
  await expect(card(page).getByLabel('재무장 간격 (%p)', { exact: true })).toBeDisabled()
})

test('sequential decimal keyboard input preserves the dot and sends a finite number', async ({ page }) => {
  const { updates } = await setup(page)
  const input = card(page).getByLabel('재무장 간격 (%p)', { exact: true })
  await chooseMode(page, 'crossing')
  await input.fill('')
  await input.pressSequentially('1.')
  await expect(input).toHaveValue('1.')
  await input.pressSequentially('2')
  await expect(input).toHaveValue('1.2')
  await card(page).getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params.entry_hysteresis_pct).toBe(1.2)
})
