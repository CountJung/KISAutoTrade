import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { period: 10, oversold: 25, overbought: 75 }
const policyParams = { long_trend_period: 200, stop_loss_pct: 3, max_hold_observations: 63 }
function response(body: any, continuous: boolean) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-18T12:00:00+09:00', message: 'rsi fixture',
    replay: { engineVersion: 'strategy-replay-v18', strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null,
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'RSIStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void; saveDelay?: Promise<void>; saveStarted?: () => void } = {}) {
  const requests: any[] = [], updates: any[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  const strategy = () => ({ id: 'rsi_default', name: 'RSIStrategy', enabled: options.enabled ?? false, brokerId: 'kis', brokerAccountId: '12345678-01', targetSymbols: ['000005'], targetSymbolNames: { '000005': 'Mock rsi' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/rsi_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    options.saveStarted?.(); if (options.saveDelay) await options.saveDelay
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: 'RSI 저장 실패' } }); return }
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
const labels = ['RSI 진입 추세 MA 관측 수', 'RSI 보유 손절 기준 (%)', 'RSI 최대 보유 관측 수'] as const
const runButton = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
const saveButton = (page: Page) => card(page).getByRole('button', { name: '변경사항 저장', exact: true })
async function setPolicy(page: Page) {
  for (const [index, value] of ['200', '3', '63'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
}
async function continuous(page: Page) {
  await card(page).getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await card(page).getByLabel('연속 시작일').fill('2025-01-01')
  await card(page).getByLabel('연속 종료일').fill('2025-12-31')
}

test('custom legacy RSI settings preserve omitted options through save and reload', async ({ page }) => {
  const { requests, updates } = await setup(page)
  const panel = card(page), controls = panel.getByTestId('rsi-policy-controls')
  await expect(panel.getByLabel('RSI 기간', { exact: true })).toHaveValue('10')
  await expect(panel.getByLabel('과매도 기준', { exact: true })).toHaveValue('25')
  await expect(panel.getByLabel('과매수 기준', { exact: true })).toHaveValue('75')
  for (const label of labels) await expect(panel.getByLabel(label, { exact: true })).toHaveValue('0')
  for (const text of ['단순평균 RSI이며 Wilder 방식이 아닙니다', '횡보를 포함해 RSI 100', '실제 보유 수량·종목 통화 기준 평균 매수가', '현재 관측은 제외', 'N관측은 N일이 아닙니다', '손절 → 최대 보유 → 기존 RSI 하락 교차', '다음 관측을 1로', '나이를 0으로 가정하지', '잔량 전량', '거절한 교차를 조건 지속만으로 재시도하지', '손실 상한을 보장하지', '운영에 자동 적용하지']) await expect(controls).toContainText(text)
  await runButton(page).click()
  await expect(panel.getByTestId('rsi-policy-summary')).toContainText('기간 10관측 · 과매도 25 / 과매수 75')
  await expect(panel.getByTestId('rsi-policy-summary')).toContainText('보유 손절 0% (누락 · 기본 비활성)')
  expect(requests[0].params).toMatchObject(legacyParams)
  expect(requests[0].params).not.toHaveProperty('stop_loss_pct')
  await panel.getByLabel('1회 수량', { exact: true }).fill('2')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params).toMatchObject(legacyParams)
  await page.reload()
  await expect(panel.getByLabel('RSI 기간', { exact: true })).toHaveValue('10')
  expect(updates[0].params).not.toHaveProperty('long_trend_period')
})

test('default RSI 14/30/70 and explicit zero protections stay zero', async ({ page }) => {
  const { requests } = await setup(page, { params: { period: 14, oversold: 30, overbought: 70, long_trend_period: 0, stop_loss_pct: 0, max_hold_observations: 0 } })
  await runButton(page).click()
  await expect(card(page).getByTestId('rsi-policy-summary')).toContainText('기간 14관측 · 과매도 30 / 과매수 70')
  await expect(card(page).getByTestId('rsi-policy-summary')).toContainText('진입 추세 0관측 · 보유 손절 0% · 최대 보유 0관측')
  expect(requests[0].params.stop_loss_pct).toBe(0)
})

test('optional policies survive scoped save/reload without changing custom RSI core', async ({ page }) => {
  const { updates } = await setup(page)
  await setPolicy(page); await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...policyParams } })
  await page.reload()
  for (const [index, value] of ['200', '3', '63'].entries()) await expect(card(page).getByLabel(labels[index], { exact: true })).toHaveValue(value)
  await expect(card(page).getByLabel('RSI 기간', { exact: true })).toHaveValue('10')
})

test('short/continuous payloads and A/B preserve each RSI policy and its units', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await runButton(page).click(); await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await setPolicy(page); await continuous(page); await runButton(page).click()
  await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...policyParams }); expect(requests[1]).not.toHaveProperty('candles')
  await expect(panel.getByTestId('rsi-policy-summary').first()).toContainText('진입 추세 200관측 · 보유 손절 3% · 최대 보유 63관측')
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('A와 B의 RSI 파라미터가 다릅니다')
  await expect(ab).toContainText('보유 손절 0% (누락 · 기본 비활성)'); await expect(ab).toContainText('보유 손절 3%')
  const snapshots = await page.evaluate(() => Object.values(localStorage).filter((value) => value.includes('rsi_default')).map((value) => JSON.parse(value)))
  expect(snapshots[0].B.params).toMatchObject({ ...legacyParams, ...policyParams })
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '단기 구간 · 최근 50/100/200봉', exact: true }).click()
  await runButton(page).click(); await expect.poll(() => requests.length).toBe(3)
  expect(requests[2].params).toMatchObject({ ...legacyParams, ...policyParams })
})

test('missing core fields in an old A/B show defaults but malformed optional values remain unknown', async ({ page }) => {
  await setup(page)
  await runButton(page).click(); await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await page.evaluate(() => {
    for (const [key, raw] of Object.entries(localStorage)) {
      if (!raw.includes('rsi_default')) continue
      const saved = JSON.parse(raw)
      if (saved.A) { saved.A.params = { stop_loss_pct: null }; localStorage.setItem(key, JSON.stringify(saved)) }
    }
  })
  await page.reload(); await runButton(page).click()
  const ab = card(page).getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('기간 14 (기본)관측 · 과매도 30 (기본) / 과매수 70 (기본)')
  await expect(ab).toContainText('보유 손절 확인 필요 (null)')
  await expect(ab).toContainText('이전 엔진의 보호 적용·실제 보유·체결을 증명하지 않습니다')
})

test('invalid raw policy drafts invalidate old results and block save and both previews', async ({ page }) => {
  const { requests } = await setup(page)
  await runButton(page).click(); await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const [label, invalids] of [[labels[0], ['', 'NaN', 'Infinity', '-1', '513', '1.5']], [labels[1], ['-0.1', '100', '99.91']], [labels[2], ['1000001', '2.5']]] as const) {
    const input = card(page).getByLabel(label, { exact: true })
    for (const raw of invalids) {
      await input.fill(raw); await expect(input).toHaveValue(raw)
      await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
      await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
      await input.fill('0'); await expect(runButton(page)).toBeEnabled()
    }
  }
  await continuous(page); await card(page).getByLabel(labels[0], { exact: true }).fill('0.5')
  await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled(); expect(requests).toHaveLength(1)
})

for (const [name, invalid] of [
  ['stored numeric string', { stop_loss_pct: '3' }], ['stored null', { max_hold_observations: null }],
  ['stored fractional integer', { long_trend_period: 1.5 }], ['stored boolean', { long_trend_period: false }],
] as const) {
  test(`${name} fails closed until real edits repair optional types while core values survive`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...legacyParams, ...invalid } })
    await expect(runButton(page)).toBeDisabled()
    await card(page).getByLabel('1회 수량', { exact: true }).fill('2'); await expect(saveButton(page)).toBeDisabled()
    await expect(card(page).getByLabel('과매도 기준', { exact: true })).toHaveValue('25')
    for (const label of labels) await card(page).getByLabel(label, { exact: true }).fill('')
    await setPolicy(page); await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
    expect(updates[0].params).toMatchObject({ ...legacyParams, ...policyParams })
  })
}

test('save failure retains RSI policies for an identical scoped retry', async ({ page }) => {
  const { updates } = await setup(page, { failSave: true })
  await setPolicy(page); await saveButton(page).click()
  await expect(page.getByRole('alert').filter({ hasText: 'RSI 저장 실패' })).toBeVisible()
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('3')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toEqual(updates[0].params)
})

test('invalid edits discard a late response and mobile controls have no horizontal overflow', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 390, height: 844 }); await setup(page, { delay, started })
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await runButton(page).click(); await entered
  await card(page).getByLabel(labels[1], { exact: true }).fill('NaN')
  release(); await (await finished).finished()
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  expect(await card(page).getByTestId('rsi-policy-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
})

test('enabled RSI strategy locks all three optional fields', async ({ page }) => {
  await setup(page, { enabled: true })
  for (const label of labels) await expect(card(page).getByLabel(label, { exact: true })).toBeDisabled()
})

test('keyboard decimal drafts and inclusive bounds reach save as numbers', async ({ page }) => {
  const { updates } = await setup(page)
  const input = card(page).getByLabel(labels[1], { exact: true })
  await input.fill(''); await input.pressSequentially('3.'); await expect(input).toHaveValue('3.')
  await input.pressSequentially('5'); await expect(input).toHaveValue('3.5')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params.stop_loss_pct).toBe(3.5); await expect(saveButton(page)).toHaveCount(0)
  for (const [index, value] of ['512', '99.9', '1000000'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toMatchObject({ long_trend_period: 512, stop_loss_pct: 99.9, max_hold_observations: 1000000 })
})

test('a delayed save preserves newer RSI drafts for the next save', async ({ page }) => {
  let release!: () => void, started!: () => void
  const saveDelay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const { updates } = await setup(page, { saveDelay, saveStarted: started })
  await setPolicy(page); await saveButton(page).click(); await entered
  await card(page).getByLabel(labels[1], { exact: true }).fill('4.5')
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategies/rsi_default'))
  release(); await (await finished).finished(); await expect(saveButton(page)).toBeEnabled()
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('4.5')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2)
  expect(updates[0].params).toMatchObject(policyParams); expect(updates[1].params).toMatchObject({ ...policyParams, stop_loss_pct: 4.5 })
  await expect(saveButton(page)).toHaveCount(0)
})
