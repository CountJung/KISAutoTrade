import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle, EntryFilterPreparation } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { threshold_pct: 2, stop_loss_pct: 2 }
const policyParams = { long_trend_period: 200, profit_activation_pct: 5, profit_trailing_pct: 3, max_hold_observations: 63 }
const ready: EntryFilterPreparation = { enabled: true, ready: true, status: 'ready', cadence: 'priorCompletedDailyOhlc', requiredCompletedBars: 200, availableCompletedBars: 252, trueRangeSamples: 14,
  candidates: 9, unreadyCandidates: 1, minMoveRejections: 0, atrRejections: 0, trendRejections: 4, passedCandidates: 4 }
function response(body: any, continuous: boolean, diagnostic?: EntryFilterPreparation) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-22T12:00:00+09:00', message: 'strong_close fixture',
    ...(diagnostic ? { entryFilterPreparation: diagnostic } : {}),
    replay: { engineVersion: 'strategy-replay-v20', strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null,
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'StrongCloseStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void; saveDelay?: Promise<void>; saveStarted?: () => void; diagnostic?: EntryFilterPreparation; toss?: boolean } = {}) {
  const requests: any[] = [], updates: any[] = [], unsafe: string[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { activeBroker: options.toss ? 'toss' : 'kis', chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  page.on('request', (request) => { if (/\/api\/(?:order|trading\/(?:start|stop))/.test(new URL(request.url()).pathname) && request.method() !== 'GET') unsafe.push(request.url()) })
  const strategy = () => ({ id: 'strong_close_default', name: 'StrongCloseStrategy', enabled: options.enabled ?? false, brokerId: options.toss ? 'toss' : 'kis', brokerAccountId: options.toss ? '1' : '12345678-01', targetSymbols: ['000005'], targetSymbolNames: { '000005': 'Mock strong_close' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/strong_close_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    options.saveStarted?.(); if (options.saveDelay) await options.saveDelay
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: '강한 종가 설정 저장 실패' } }); return }
    stored = body.params; await route.fulfill({ json: strategy() })
  })
  await page.route('**/api/strategy/preview{,/continuous}', async (route) => {
    const body = route.request().postDataJSON(), continuous = route.request().url().endsWith('/continuous')
    requests.push(body); options.started?.()
    if (options.delay) await options.delay
    await route.fulfill({ json: response(body, continuous, options.diagnostic) })
  })
  await page.goto('/strategy')
  return { requests, updates, unsafe }
}
const labels = ['강한 종가 진입 완료 일봉 추세 MA 기간', '강한 종가 수익 보호 시작 (%)', '강한 종가 수익 보호 되밀림 (%)', '강한 종가 최대 보유 관측 수'] as const
const runButton = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
const saveButton = (page: Page) => card(page).getByRole('button', { name: '변경사항 저장', exact: true })
async function setPolicy(page: Page) {
  for (const [index, value] of ['200', '5', '3', '63'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
}
async function continuous(page: Page) {
  await card(page).getByRole('combobox', { name: /^계산 범위 모드/ }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await card(page).getByLabel('연속 시작일').fill('2025-01-01')
  await card(page).getByLabel('연속 종료일').fill('2025-12-31')
}
async function saveAndWait(page: Page) {
  const response = page.waitForResponse((res) => res.url().endsWith('/api/strategies/strong_close_default'))
  await saveButton(page).click(); await (await response).finished(); await expect(saveButton(page)).toHaveCount(0)
}

test('custom legacy settings preserve omitted options and explain owned protection and publication', async ({ page }) => {
  const { requests, updates, unsafe } = await setup(page)
  const panel = card(page), controls = panel.getByTestId('strong-close-policy-controls')
  await expect(panel.getByLabel('강한 종가 기준 (%)', { exact: true })).toHaveValue('2')
  await expect(panel.getByLabel('손절 기준 (%)', { exact: true })).toHaveValue('2')
  for (const label of labels) await expect(panel.getByLabel(label, { exact: true })).toHaveValue('0')
  for (const text of ['(고가−종가)/고가×100 ≤ 기준', '양봉·상승률 조건이 아닙니다', '고가=종가인 유효한 봉도 통과', '첫 적격 가격관측', 'D-1 완료 고가·종가 조건 → D 종가 관측 신호 → D+1 다음 관측 시가 체결', '실제 보유 수량·종목 통화 기준 평균 매수가', '손절0도 평균가 이하 청산', '현재 관측을 제외', '실제 보유 중 관측 최고가', '기존 손절 → 수익 보호 → 보유 한도', '주문 거절·부분 체결·설정 변경', '나이가 미확인이면 활성 보유 한도', '일수나 실제 보유시간 보장이 아닙니다', '같은 날 자료 갱신이 이미 소비한 조건을 다시 만들지는']) await expect(controls).toContainText(text)
  await runButton(page).click()
  await expect(panel.getByTestId('strong-close-policy-summary')).toContainText('≤ 2% · 손절 2%')
  await expect(panel.getByTestId('strong-close-preparation')).toContainText('진입 추세 비활성')
  expect(requests[0].params).toMatchObject(legacyParams)
  await panel.getByLabel('1회 수량', { exact: true }).fill('2'); await saveAndWait(page)
  expect(updates[0].params).toMatchObject(legacyParams)
  for (const key of Object.keys(policyParams)) expect(updates[0].params).not.toHaveProperty(key)
  await page.reload(); await expect(panel.getByLabel('강한 종가 기준 (%)', { exact: true })).toHaveValue('2')
  expect(unsafe).toEqual([])
})

test('standard settings and explicit zero options preserve original active stop zero', async ({ page }) => {
  const { requests, updates } = await setup(page, { params: { threshold_pct: 3, stop_loss_pct: 3, long_trend_period: 0, profit_activation_pct: 0, profit_trailing_pct: 0, max_hold_observations: 0 } })
  await runButton(page).click(); await expect(card(page).getByTestId('strong-close-policy-summary')).toContainText('≤ 3% · 손절 3%')
  expect(requests[0].params).toMatchObject({ threshold_pct: 3, stop_loss_pct: 3, long_trend_period: 0, profit_activation_pct: 0, profit_trailing_pct: 0, max_hold_observations: 0 })
  await card(page).getByLabel('손절 기준 (%)', { exact: true }).fill('0'); await saveAndWait(page)
  expect(updates[0].params.stop_loss_pct).toBe(0)
  await runButton(page).click(); await expect(card(page).getByTestId('strong-close-policy-summary')).toContainText('손절 0% (0도 평균가 이하 청산)')
})

test('missing legacy core renders original defaults without writing option defaults', async ({ page }) => {
  const { requests } = await setup(page, { params: {} })
  await expect(card(page).getByLabel('강한 종가 기준 (%)', { exact: true })).toHaveValue('3')
  await expect(card(page).getByLabel('손절 기준 (%)', { exact: true })).toHaveValue('3')
  await runButton(page).click(); await expect(card(page).getByTestId('strong-close-policy-summary')).toContainText('≤ 3 (기본)% · 손절 3 (기본)%')
  expect(requests[0].params).not.toHaveProperty('long_trend_period')
})

test('four optional settings survive scoped save and reload', async ({ page }) => {
  const { updates, unsafe } = await setup(page)
  await setPolicy(page); await saveAndWait(page)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...policyParams } })
  await page.reload()
  for (const [index, value] of ['200', '5', '3', '63'].entries()) await expect(card(page).getByLabel(labels[index], { exact: true })).toHaveValue(value)
  expect(unsafe).toEqual([])
})

test('bounded and continuous previews preserve params, sizing and A/B preparation', async ({ page }) => {
  const options = { diagnostic: undefined as EntryFilterPreparation | undefined }
  const { requests } = await setup(page, options), panel = card(page)
  await runButton(page).click(); await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  options.diagnostic = ready
  await setPolicy(page); await continuous(page)
  await panel.getByLabel('백테스트 ATR 기반 수량 산정', { exact: true }).check()
  await runButton(page).click(); await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...policyParams }); expect(requests[1]).not.toHaveProperty('candles')
  expect(requests[1].assumptions.volatilitySizingEnabled).toBe(true)
  const diagnostic = panel.getByTestId('strong-close-preparation').first()
  for (const text of ['준비 완료', '필요 완료봉 200 · 확보 252', '전일 조건 후보 9', '준비 미확인 보류 1', '추세 거부 4', '후보 통과 4', '수량 산정용 ATR 준비와 별도']) await expect(diagnostic).toContainText(text)
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('A와 B의 강한 종가 파라미터가 다릅니다')
  await expect(ab).toContainText('누락 · 기본 비활성'); await expect(ab).toContainText('수익 보호 시작 5% / 되밀림 3%')
  const snapshots = await page.evaluate(() => Object.values(localStorage).filter((value) => value.includes('strong_close_default')).map((value) => JSON.parse(value)))
  expect(snapshots[0].B.params).toMatchObject(policyParams); expect(snapshots[0].B.entryFilterPreparation).toMatchObject({ cadence: 'priorCompletedDailyOhlc', requiredCompletedBars: 200, trendRejections: 4 })
  await panel.getByRole('combobox', { name: /^계산 범위 모드/ }).click(); await page.getByRole('option', { name: '단기 구간 · 최근 50/100/200봉', exact: true }).click()
  await runButton(page).click(); await expect.poll(() => requests.length).toBe(3)
  expect(requests[2].params).toMatchObject(policyParams)
})

for (const [name, diagnostic, expected] of [
  ['insufficient history', { ...ready, ready: false, status: 'insufficientHistory', availableCompletedBars: 199 }, '이전 완료 자료 부족'],
  ['invalid params', { ...ready, ready: false, status: 'invalidParams' }, '잘못된 설정'],
  ['unsupported observation', { ...ready, ready: false, status: 'unsupportedObservation' }, '관측 형식 미지원'],
  ['unknown future status', { ...ready, status: 'futureStatus' }, '진입 준비 형식 미지원'],
  ['unknown cadence', { ...ready, cadence: 'priceObservations' }, '진입 준비 형식 미지원'],
] as const) {
  test(`preparation ${name} remains distinct in current and saved results`, async ({ page }) => {
    await setup(page, { params: { ...legacyParams, ...policyParams }, diagnostic: diagnostic as EntryFilterPreparation })
    await runButton(page).click(); await expect(card(page).getByTestId('strong-close-preparation')).toContainText(expected)
    await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
    await expect(card(page).getByTestId('strategy-ab-comparison')).toContainText(expected)
  })
}

test('active legacy response without preparation and damaged old A/B remain unverified', async ({ page }) => {
  await setup(page, { params: { ...legacyParams, ...policyParams } })
  await runButton(page).click(); await expect(card(page).getByTestId('strong-close-preparation')).toContainText('진입 준비 정보 없음')
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await page.evaluate(() => { for (const [key, raw] of Object.entries(localStorage)) { if (!raw.includes('strong_close_default')) continue; const saved = JSON.parse(raw); if (saved.A) { saved.A.params = { profit_trailing_pct: null }; saved.A.replay.engineVersion = 'strategy-replay-v19'; localStorage.setItem(key, JSON.stringify(saved)) } } })
  await page.reload(); await runButton(page).click()
  await card(page).getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = card(page).getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('≤ 3 (기본)% · 손절 3 (기본)%'); await expect(ab).toContainText('되밀림 확인 필요 (null)')
  await expect(ab).toContainText('이전 엔진의 새 보호 적용·실제 보유·체결을')
  await expect(ab).toContainText('진입 준비 정보 없음'); await expect(ab).toContainText('A와 B의 엔진 또는 체결 가정이 다릅니다')
})

test('strict numeric drafts and paired protection block save and both preview modes', async ({ page }) => {
  const { requests } = await setup(page)
  await runButton(page).click(); await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const [label, invalids] of [[labels[0], ['', 'NaN', 'Infinity', '-1', '1.5', '513']], [labels[1], ['1000.1']], [labels[2], ['99.91']], [labels[3], ['1.5', '1000001']]] as const) {
    for (const raw of invalids) {
      const input = card(page).getByLabel(label, { exact: true })
      await input.fill(raw); await expect(input).toHaveValue(raw); await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
      await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
      await input.fill('0'); await expect(runButton(page)).toBeEnabled()
    }
  }
  await card(page).getByLabel(labels[1], { exact: true }).fill('5'); await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
  await expect(card(page)).toContainText('둘 다 양수여야')
  await card(page).getByLabel(labels[2], { exact: true }).fill('3'); await expect(runButton(page)).toBeEnabled()
  await continuous(page); await card(page).getByLabel(labels[3], { exact: true }).fill('2.5')
  await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled(); expect(requests).toHaveLength(1)
})

for (const [name, invalid] of [
  ['stored numeric string', { long_trend_period: '200' }], ['stored null', { profit_trailing_pct: null }],
  ['stored fractional integer', { max_hold_observations: 1.5 }], ['stored boolean', { profit_activation_pct: false }], ['stored unpaired protection', { profit_activation_pct: 5 }],
] as const) {
  test(`${name} remains invalid until deliberate edits repair it`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...legacyParams, ...invalid } })
    await expect(runButton(page)).toBeDisabled(); await card(page).getByLabel('1회 수량', { exact: true }).fill('2'); await expect(saveButton(page)).toBeDisabled()
    await expect(card(page).getByLabel('강한 종가 기준 (%)', { exact: true })).toHaveValue('2')
    await expect(card(page).getByLabel('손절 기준 (%)', { exact: true })).toHaveValue('2')
    for (const label of labels) await card(page).getByLabel(label, { exact: true }).fill('')
    await setPolicy(page); await saveAndWait(page)
    expect(updates[0].params).toMatchObject({ ...legacyParams, ...policyParams })
  })
}

test('generic StrongClose remains daily only even with only profit and cap', async ({ page }) => {
  await setup(page, { toss: true, params: { ...legacyParams, profit_activation_pct: 5, profit_trailing_pct: 3, max_hold_observations: 63 } })
  const panel = card(page)
  await panel.getByRole('combobox', { name: /^봉 단위/ }).click()
  await expect(page.getByRole('option', { name: '일봉', exact: true })).toBeVisible()
  await expect(page.getByRole('option', { name: '1분봉', exact: true })).toHaveCount(0)
  await page.keyboard.press('Escape'); await expect(panel.getByTestId('daily-event-replay-note')).toContainText('일봉만 지원')
  await expect(runButton(page)).toBeEnabled()
})

test('save failure retains identical scoped settings for retry', async ({ page }) => {
  const { updates } = await setup(page, { failSave: true })
  await setPolicy(page); await saveButton(page).click(); await expect(page.getByRole('alert').filter({ hasText: '강한 종가 설정 저장 실패' })).toBeVisible()
  await expect(card(page).getByLabel(labels[2], { exact: true })).toHaveValue('3')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2); expect(updates[1]).toEqual(updates[0])
})

test('late preview cannot publish after invalid edits and mobile stays inside card', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 375, height: 844 }); await setup(page, { delay, started })
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await runButton(page).click(); await entered; await card(page).getByLabel(labels[0], { exact: true }).fill('NaN')
  release(); await (await finished).finished(); await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  expect(await card(page).getByTestId('strong-close-policy-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
  await page.screenshot({ path: '.cache/strong-close-mobile.png', fullPage: true })
})

test('active strategy locks every optional control', async ({ page }) => {
  await setup(page, { enabled: true })
  for (const label of labels) await expect(card(page).getByLabel(label, { exact: true })).toBeDisabled()
})

test('decimal keyboard drafts and inclusive limits save as numeric values', async ({ page }) => {
  const { updates } = await setup(page, { params: { ...legacyParams, profit_activation_pct: 5, profit_trailing_pct: 3 } })
  const input = card(page).getByLabel(labels[2], { exact: true })
  await input.fill(''); await input.pressSequentially('0.'); await expect(input).toHaveValue('0.'); await input.pressSequentially('5'); await expect(input).toHaveValue('0.5')
  await saveAndWait(page); expect(updates[0].params.profit_trailing_pct).toBe(0.5)
  for (const [index, value] of ['512', '1000', '99.9', '1000000'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
  await saveAndWait(page)
  expect(updates[1].params).toMatchObject({ long_trend_period: 512, profit_activation_pct: 1000, profit_trailing_pct: 99.9, max_hold_observations: 1000000 })
})

test('save completion preserves a newer draft for a second scoped save', async ({ page }) => {
  let release!: () => void, started!: () => void
  const saveDelay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const { updates } = await setup(page, { saveDelay, saveStarted: started })
  await setPolicy(page); await saveButton(page).click(); await entered
  await card(page).getByLabel(labels[2], { exact: true }).fill('4.5')
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategies/strong_close_default'))
  release(); await (await finished).finished(); await expect(saveButton(page)).toBeEnabled()
  await expect(card(page).getByLabel(labels[2], { exact: true })).toHaveValue('4.5')
  await saveAndWait(page)
  expect(updates[0].params).toMatchObject(policyParams); expect(updates[1].params).toMatchObject({ ...policyParams, profit_trailing_pct: 4.5 })
})

test('cancelled continuous result cannot publish and a later preview can complete', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const { requests } = await setup(page, { delay, started, params: { ...legacyParams, ...policyParams }, diagnostic: ready })
  await continuous(page)
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview/continuous'))
  await runButton(page).click(); await entered
  await card(page).getByRole('button', { name: '취소', exact: true }).click()
  await expect(card(page)).toContainText('미리보기 계산을 취소했습니다.')
  release(); await (await finished).finished(); await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  await card(page).getByLabel(labels[0], { exact: true }).fill('100')
  await runButton(page).click(); await expect(card(page).getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests).toHaveLength(2); expect(requests[1].params.long_trend_period).toBe(100)
})
