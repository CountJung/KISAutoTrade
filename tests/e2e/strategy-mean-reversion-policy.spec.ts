import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { period: 30, std_dev: 2.5, stop_loss_pct: 4 }
const policyParams = { exit_mode: 'middleBand', long_trend_period: 200, max_hold_observations: 63 }
function response(body: any, continuous: boolean) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-18T12:00:00+09:00', message: 'mean_reversion fixture',
    replay: { engineVersion: 'strategy-replay-v15', strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null,
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'MeanReversionStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void; saveDelay?: Promise<void>; saveStarted?: () => void } = {}) {
  const requests: any[] = [], updates: any[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  const strategy = () => ({ id: 'mean_reversion_default', name: 'MeanReversionStrategy', enabled: options.enabled ?? false, brokerId: 'kis', brokerAccountId: '12345678-01', targetSymbols: ['000005'], targetSymbolNames: { '000005': 'Mock mean_reversion' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/mean_reversion_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    options.saveStarted?.(); if (options.saveDelay) await options.saveDelay
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: 'MeanReversion 저장 실패' } }); return }
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
async function chooseMode(page: Page, mode: 'upperBand' | 'middleBand') {
  await card(page).getByRole('combobox', { name: /^MeanReversion 청산 모드/ }).click()
  await page.getByRole('option', { name: mode === 'upperBand' ? 'upperBand · 상단 초과 (기본)' : 'middleBand · 중심선 초과', exact: true }).click()
}

const labels = ['진입 추세 MA 관측 수', '최대 보유 관측 수'] as const
const runButton = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
const saveButton = (page: Page) => card(page).getByRole('button', { name: '변경사항 저장', exact: true })
async function setPolicy(page: Page) {
  await chooseMode(page, 'middleBand')
  await card(page).getByLabel(labels[0], { exact: true }).fill('200')
  await card(page).getByLabel(labels[1], { exact: true }).fill('63')
}
async function continuous(page: Page) {
  await card(page).getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await card(page).getByLabel('연속 시작일').fill('2025-01-01')
  await card(page).getByLabel('연속 종료일').fill('2025-12-31')
}

test('legacy custom Bollinger settings retain omitted upper-band and zero new options through save and reload', async ({ page }) => {
  const { requests, updates } = await setup(page)
  const panel = card(page), controls = panel.getByTestId('mean-reversion-policy-controls')
  await expect(panel.getByLabel('볼린저 밴드 기간', { exact: true })).toHaveValue('30')
  await expect(panel.getByLabel('표준편차 배율', { exact: true })).toHaveValue('2.5')
  await expect(panel.getByLabel('손절 기준 (%)', { exact: true })).toHaveValue('4')
  await expect(panel.getByRole('combobox', { name: /^MeanReversion 청산 모드/ })).toContainText('upperBand · 상단 초과 (기본)')
  for (const label of labels) await expect(panel.getByLabel(label, { exact: true })).toHaveValue('0')
  for (const text of ['현재 관측을 포함한 볼린저 평균·모집단 분산', '하단 미만일 때만 매수', '경계와 같으면 밴드 매매 조건이 아닙니다', '실제 양수 수량·native 평균가', '부분 체결 뒤에도 실제 잔량', '손절 0은 비활성이 아니라 원가 이하 손절', '손절·밴드 매도는 설정 수량, 최대 보유만 실제 잔량 전량', '손절 → 최대 보유 → 선택 밴드', '일봉/1분봉 warmup과 섞지', '옵션이 꺼져 있어도', '장중 손실 상한을 보장하지']) await expect(controls).toContainText(text)
  await runButton(page).click()
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  expect(requests[0].params).toMatchObject(legacyParams)
  expect(requests[0].params).not.toHaveProperty('exit_mode')
  await panel.getByLabel('1회 수량', { exact: true }).fill('2')
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params).toMatchObject(legacyParams)
  await page.reload()
  await expect(panel.getByLabel('볼린저 밴드 기간', { exact: true })).toHaveValue('30')
})

test('optional exit mode and integer observations survive scoped save and reload without coercion', async ({ page }) => {
  const { updates } = await setup(page)
  await setPolicy(page)
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...policyParams } })
  await page.reload()
  await expect(card(page).getByRole('combobox', { name: /^MeanReversion 청산 모드/ })).toContainText('middleBand · 중심선 초과')
  await expect(card(page).getByLabel(labels[0], { exact: true })).toHaveValue('200')
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('63')
})

test('short and continuous payloads retain policy and A/B displays each saved exit and observation assumption', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await runButton(page).click()
  await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await setPolicy(page)
  await continuous(page)
  await runButton(page).click()
  await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[0].params).toMatchObject(legacyParams)
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...policyParams })
  expect(requests[1]).not.toHaveProperty('candles')
  await expect(panel).toContainText('MeanReversion 청산 · middleBand · 추세 MA 200관측 · 최대 보유 63관측')
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('A와 B의 MeanReversion 파라미터가 다릅니다')
  await expect(ab).toContainText('upperBand (기본 upperBand) · 추세 MA 0관측 · 최대 보유 0관측')
  await expect(ab).toContainText('middleBand · 추세 MA 200관측 · 최대 보유 63관측')
  const snapshots = await page.evaluate(() => Object.values(localStorage).filter((value) => value.includes('mean_reversion_default')).map((value) => JSON.parse(value)))
  expect(snapshots[0].B.params).toMatchObject(policyParams)
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '단기 구간 · 최근 50/100/200봉', exact: true }).click()
  await runButton(page).click()
  await expect.poll(() => requests.length).toBe(3)
  expect(requests[2].params).toMatchObject({ ...legacyParams, ...policyParams })
})

test('invalid observation drafts retain text and block saving and both previews while clearing earlier results', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await runButton(page).click()
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const [label, invalids] of [[labels[0], ['', 'NaN', 'Infinity', '-1', '513', '1.5']], [labels[1], ['1000001', '2.5']]] as const) {
    const input = panel.getByLabel(label, { exact: true })
    for (const raw of invalids) {
      await input.fill(raw)
      await expect(input).toHaveValue(raw)
      await expect(runButton(page)).toBeDisabled()
      await expect(saveButton(page)).toBeDisabled()
      await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
      await input.fill('0')
      await expect(runButton(page)).toBeEnabled()
    }
  }
  await continuous(page)
  await panel.getByLabel(labels[0], { exact: true }).fill('0.5')
  await expect(runButton(page)).toBeDisabled()
  await expect(saveButton(page)).toBeDisabled()
  expect(requests).toHaveLength(1)
})

for (const [name, invalid] of [
  ['unknown enum', { exit_mode: 'bogus' }], ['null enum', { exit_mode: null }],
  ['stored numeric string', { long_trend_period: '200' }], ['stored null', { max_hold_observations: null }],
  ['stored fractional integer', { max_hold_observations: 1.5 }], ['stored boolean', { long_trend_period: false }],
] as const) {
  test(`${name} fails closed until a real edit repairs the optional stored type`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...legacyParams, ...invalid } })
    await expect(runButton(page)).toBeDisabled()
    await card(page).getByLabel('1회 수량', { exact: true }).fill('2')
    await expect(saveButton(page)).toBeDisabled()
    for (const label of labels) await card(page).getByLabel(label, { exact: true }).fill('')
    await setPolicy(page)
    await saveButton(page).click()
    await expect.poll(() => updates.length).toBe(1)
    expect(updates[0].params).toMatchObject({ ...legacyParams, ...policyParams })
  })
}

test('save failure retains edited policy and permits an identical scoped retry', async ({ page }) => {
  const { updates } = await setup(page, { failSave: true })
  await setPolicy(page)
  await saveButton(page).click()
  await expect(page.getByRole('alert').filter({ hasText: 'MeanReversion 저장 실패' })).toBeVisible()
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('63')
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toEqual(updates[0].params)
})

test('invalid edits discard a late response and mobile controls have no horizontal overflow', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 390, height: 844 })
  await setup(page, { delay, started })
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await runButton(page).click(); await entered
  await card(page).getByLabel(labels[0], { exact: true }).fill('NaN')
  release(); await (await finished).finished()
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  expect(await card(page).getByTestId('mean-reversion-policy-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
})

test('enabled strategy locks exit mode and both new integer fields', async ({ page }) => {
  await setup(page, { enabled: true })
  await expect(card(page).getByRole('combobox', { name: /^MeanReversion 청산 모드/ })).toBeDisabled()
  for (const label of labels) await expect(card(page).getByLabel(label, { exact: true })).toBeDisabled()
})

test('keyboard dot drafts remain visible and inclusive integer limits plus legacy zero stop reach save', async ({ page }) => {
  const { updates } = await setup(page)
  const panel = card(page), input = panel.getByLabel(labels[0], { exact: true })
  await input.fill(''); await input.pressSequentially('2.')
  await expect(input).toHaveValue('2.')
  await input.pressSequentially('0')
  await expect(input).toHaveValue('2.0')
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params.long_trend_period).toBe(2)
  await expect(saveButton(page)).toHaveCount(0)
  await input.fill('512')
  await panel.getByLabel(labels[1], { exact: true }).fill('1000000')
  await panel.getByLabel('손절 기준 (%)', { exact: true }).fill('0')
  await expect(runButton(page)).toBeEnabled()
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toMatchObject({ long_trend_period: 512, max_hold_observations: 1000000, stop_loss_pct: 0 })
})

test('a delayed save preserves newer mean-reversion drafts for the next save', async ({ page }) => {
  let release!: () => void, started!: () => void
  const saveDelay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const { updates } = await setup(page, { saveDelay, saveStarted: started })
  await setPolicy(page)
  await saveButton(page).click(); await entered
  await chooseMode(page, 'upperBand')
  await card(page).getByLabel(labels[1], { exact: true }).fill('99')
  const completed = page.waitForResponse((res) => res.url().endsWith('/api/strategies/mean_reversion_default'))
  release(); await (await completed).finished()
  await expect(saveButton(page)).toBeEnabled()
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('99')
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[0].params).toMatchObject(policyParams)
  expect(updates[1].params).toMatchObject({ ...policyParams, exit_mode: 'upperBand', max_hold_observations: 99 })
  await expect(saveButton(page)).toHaveCount(0)
})
