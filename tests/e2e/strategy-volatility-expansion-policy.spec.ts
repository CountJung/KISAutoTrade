import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle, VolatilityExpansionPreparation } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { lookback_days: 7, expansion_factor: 1.7, stop_loss_pct: 2 }
const policyParams = { profit_activation_pct: 5, profit_trailing_pct: 3, max_hold_observations: 63 }
const ready: VolatilityExpansionPreparation = { ready: true, status: 'ready', cadence: 'dailyCloseRange', requiredCompletedBars: 7, availableCompletedBars: 252, evaluatedObservations: 20, ownedObservations: 8, candidateObservations: 12, unreadyCandidates: 1, rangeRejections: 4, directionRejections: 3, passedCandidates: 4 }
function response(body: any, continuous: boolean, diagnostic?: VolatilityExpansionPreparation) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-22T12:00:00+09:00', message: 'volatility_expansion fixture',
    ...(diagnostic ? { volatilityExpansionPreparation: diagnostic } : {}),
    replay: { engineVersion: 'strategy-replay-v21', strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null,
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'VolatilityExpansionStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; failPreview?: boolean; delay?: Promise<void>; started?: () => void; saveDelay?: Promise<void>; saveStarted?: () => void; diagnostic?: VolatilityExpansionPreparation; toss?: boolean } = {}) {
  const requests: any[] = [], updates: any[] = [], unsafe: string[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { activeBroker: options.toss ? 'toss' : 'kis', chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  page.on('request', (request) => { if (/\/api\/(?:order|trading\/(?:start|stop))/.test(new URL(request.url()).pathname) && request.method() !== 'GET') unsafe.push(request.url()) })
  const strategy = () => ({ id: 'volatility_expansion_default', name: 'VolatilityExpansionStrategy', enabled: options.enabled ?? false, brokerId: options.toss ? 'toss' : 'kis', brokerAccountId: options.toss ? '1' : '12345678-01', targetSymbols: ['000005'], targetSymbolNames: { '000005': 'Mock volatility_expansion' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/volatility_expansion_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    options.saveStarted?.(); if (options.saveDelay) await options.saveDelay
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: '변동성 확장 설정 저장 실패' } }); return }
    stored = body.params; await route.fulfill({ json: strategy() })
  })
  await page.route('**/api/strategy/preview{,/continuous}', async (route) => {
    const body = route.request().postDataJSON(), continuous = route.request().url().endsWith('/continuous')
    requests.push(body); options.started?.()
    if (options.delay) await options.delay
    if (options.failPreview) { await route.fulfill({ status: 500, json: { code: 'SIMULATION_ERROR', error: '변동성 확장 계산 실패' } }); return }
    await route.fulfill({ json: response(body, continuous, options.diagnostic) })
  })
  await page.goto('/strategy')
  return { requests, updates, unsafe }
}
const coreLabels = ['평균 기간 (완료 일봉)', '확장 배율', '변동성 손절 기준 (%)'] as const
const labels = ['변동성 수익 보호 시작 (%)', '변동성 수익 보호 되밀림 (%)', '변동성 최대 보유 관측 수'] as const
const runButton = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
const saveButton = (page: Page) => card(page).getByRole('button', { name: '변경사항 저장', exact: true })
async function setPolicy(page: Page) {
  for (const [index, value] of ['5', '3', '63'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
}
async function continuous(page: Page) {
  await card(page).getByRole('combobox', { name: /^계산 범위 모드/ }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await card(page).getByLabel('연속 시작일').fill('2025-01-01')
  await card(page).getByLabel('연속 종료일').fill('2025-12-31')
}
async function saveAndWait(page: Page) {
  const response = page.waitForResponse((res) => res.url().endsWith('/api/strategies/volatility_expansion_default'))
  await saveButton(page).click(); await (await response).finished(); await expect(saveButton(page)).toHaveCount(0)
}

test('custom legacy settings preserve omitted options and explain owned protection and publication', async ({ page }) => {
  const { requests, updates, unsafe } = await setup(page)
  const panel = card(page), controls = panel.getByTestId('volatility-expansion-controls')
  await expect(panel.getByLabel('평균 기간 (완료 일봉)', { exact: true })).toHaveValue('7')
  await expect(panel.getByLabel('변동성 손절 기준 (%)', { exact: true })).toHaveValue('2')
  for (const label of labels) await expect(panel.getByLabel(label, { exact: true })).toHaveValue('0')
  for (const text of ['이전 N완료 일봉 범위 산술평균 × 배수', '두 조건 모두 엄격한 초과', '0범위도 평균 일수에 포함', '배수0도 양의 당일 범위', 'D 종가에 완성 OHLC', '다음 관측 시가에 체결', '첫 관측 가격을 시가 대용', '거래소 시가·당일 전체 고저와 다릅니다', '실제 보유 수량·종목 통화 기준 평균 매수가', '손절0도 평균가 이하 청산', '실제 보유 중 관측 최고가', '기존 손절 → 수익 보호 → 보유 한도', '주문 거절·부분 체결·설정 변경', '나이가 미확인이면 활성 보유 한도', '일수나 실제 보유시간 보장이 아닙니다', '같은 날 자료 갱신으로 관측 범위를 초기화하지']) await expect(controls).toContainText(text)
  await runButton(page).click()
  await expect(panel.getByTestId('volatility-expansion-summary')).toContainText('이전 7완료봉 범위 평균 × 1.7배 초과')
  await expect(panel.getByTestId('volatility-expansion-preparation')).toContainText('준비 정보 없음')
  expect(requests[0].params).toMatchObject(legacyParams)
  await panel.getByLabel('1회 수량', { exact: true }).fill('2'); await saveAndWait(page)
  expect(updates[0].params).toMatchObject(legacyParams)
  for (const key of Object.keys(policyParams)) expect(updates[0].params).not.toHaveProperty(key)
  await page.reload(); await expect(panel.getByLabel('평균 기간 (완료 일봉)', { exact: true })).toHaveValue('7')
  expect(unsafe).toEqual([])
})

test('standard settings and explicit zero options preserve original active stop zero', async ({ page }) => {
  const { requests, updates } = await setup(page, { params: { lookback_days: 10, expansion_factor: 2, stop_loss_pct: 3, profit_activation_pct: 0, profit_trailing_pct: 0, max_hold_observations: 0 } })
  await runButton(page).click(); await expect(card(page).getByTestId('volatility-expansion-summary')).toContainText('이전 10완료봉 범위 평균 × 2배 초과')
  expect(requests[0].params).toMatchObject({ lookback_days: 10, expansion_factor: 2, stop_loss_pct: 3, profit_activation_pct: 0, profit_trailing_pct: 0, max_hold_observations: 0 })
  await card(page).getByLabel('변동성 손절 기준 (%)', { exact: true }).fill('0'); await saveAndWait(page)
  expect(updates[0].params.stop_loss_pct).toBe(0)
  await runButton(page).click(); await expect(card(page).getByTestId('volatility-expansion-summary')).toContainText('손절 0% (0도 평균가 이하 청산)')
})

test('missing legacy core renders original defaults without writing option defaults', async ({ page }) => {
  const { requests } = await setup(page, { params: {} })
  await expect(card(page).getByLabel('평균 기간 (완료 일봉)', { exact: true })).toHaveValue('10')
  await expect(card(page).getByLabel('변동성 손절 기준 (%)', { exact: true })).toHaveValue('3')
  await runButton(page).click(); await expect(card(page).getByTestId('volatility-expansion-summary')).toContainText('이전 10완료봉 (누락 · 기본) 범위 평균 × 2배 (누락 · 기본) 초과')
  expect(requests[0].params).not.toHaveProperty('profit_activation_pct')
})

test('three optional settings survive scoped save and reload', async ({ page }) => {
  const { updates, unsafe } = await setup(page)
  await setPolicy(page); await saveAndWait(page)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...policyParams } })
  await page.reload()
  for (const [index, value] of ['5', '3', '63'].entries()) await expect(card(page).getByLabel(labels[index], { exact: true })).toHaveValue(value)
  expect(unsafe).toEqual([])
})

test('bounded and continuous previews preserve params, sizing and A/B preparation', async ({ page }) => {
  const options = { diagnostic: undefined as VolatilityExpansionPreparation | undefined }
  const { requests } = await setup(page, options), panel = card(page)
  await runButton(page).click(); await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  options.diagnostic = ready
  await setPolicy(page); await continuous(page)
  await panel.getByLabel('백테스트 ATR 기반 수량 산정', { exact: true }).check()
  await runButton(page).click(); await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...policyParams }); expect(requests[1]).not.toHaveProperty('candles')
  expect(requests[1].assumptions.volatilitySizingEnabled).toBe(true)
  const diagnostic = panel.getByTestId('volatility-expansion-preparation').first()
  for (const text of ['준비 완료', '필요 완료봉 7 · 확보 252', '평가 관측 20', '보유 관측 8', '미보유 후보 12', '준비 미확인 보류 1', '범위 거부 4', '방향 거부 3', '후보 통과 4', '수량 산정용 ATR·다른 전략의 진입 필터와 별도']) await expect(diagnostic).toContainText(text)
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('A와 B의 변동성 확장 조건 또는 관측 주기가 다릅니다')
  await expect(ab).toContainText('누락 · 기본 비활성'); await expect(ab).toContainText('수익 보호 시작 5% / 되밀림 3%')
  const snapshots = await page.evaluate(() => Object.values(localStorage).map((value) => { try { return JSON.parse(value) } catch { return null } }).find((value) => value?.B?.strategyId === 'volatility_expansion_default'))
  expect(snapshots.B.params).toMatchObject(policyParams); expect(snapshots.B.volatilityExpansionPreparation).toMatchObject({ cadence: 'dailyCloseRange', requiredCompletedBars: 7, rangeRejections: 4, directionRejections: 3 })
  await panel.getByRole('combobox', { name: /^계산 범위 모드/ }).click(); await page.getByRole('option', { name: '단기 구간 · 최근 50/100/200봉', exact: true }).click()
  await runButton(page).click(); await expect.poll(() => requests.length).toBe(3)
  expect(requests[2].params).toMatchObject(policyParams)
  await page.reload(); await runButton(page).click()
  await expect(panel.getByTestId('strategy-ab-comparison')).toContainText('범위 거부 4 · 방향 거부 3 · 후보 통과 4')
})

for (const [name, diagnostic, expected] of [
  ['insufficient history', { ...ready, ready: false, status: 'insufficientHistory', availableCompletedBars: 6 }, '이전 완료 자료 부족'],
  ['invalid params', { ...ready, ready: false, status: 'invalidParams' }, '잘못된 설정'],
  ['unsupported observation', { ...ready, ready: false, status: 'unsupportedObservation' }, '관측 형식 미지원'],
  ['unknown future status', { ...ready, status: 'futureStatus' }, '준비 정보 확인 필요'],
  ['unknown cadence', { ...ready, cadence: 'priceObservations' }, '준비 정보 확인 필요'],
  ['live cadence', { ...ready, cadence: 'observedLiveQuotes' }, '실시간 관측 가격 범위'],
  ['live cadence before verification', { ...ready, ready: false, status: 'unsupportedObservation', cadence: 'observedLiveQuotes', availableCompletedBars: 0 }, '관측 형식 미지원'],
  ['unverified cadence', { ...ready, ready: false, status: 'insufficientHistory', cadence: 'unverified', availableCompletedBars: 0 }, '관측 주기 미확인'],
  ['negative count', { ...ready, rangeRejections: -1 }, '준비 정보 확인 필요'],
  ['numeric string count', { ...ready, candidateObservations: '12' }, '준비 정보 확인 필요'],
  ['missing count', { ...ready, ownedObservations: undefined }, '준비 정보 확인 필요'],
  ['inconsistent classification', { ...ready, passedCandidates: 5 }, '준비 정보 확인 필요'],
  ['inconsistent readiness', { ...ready, ready: false }, '준비 정보 확인 필요'],
  ['ready without sufficient history', { ...ready, availableCompletedBars: 6 }, '준비 정보 확인 필요'],
  ['ready without verified cadence', { ...ready, cadence: 'unverified' }, '준비 정보 확인 필요'],
  ['available history exceeds bounded context', { ...ready, availableCompletedBars: 513 }, '준비 정보 확인 필요'],
  ['unready available history exceeds bounded context', { ...ready, ready: false, status: 'insufficientHistory', availableCompletedBars: 513 }, '준비 정보 확인 필요'],
  ['required history exceeds bounded context', { ...ready, ready: false, status: 'insufficientHistory', requiredCompletedBars: 513 }, '준비 정보 확인 필요'],
  ['required history cannot be zero even when invalid', { ...ready, ready: false, status: 'invalidParams', requiredCompletedBars: 0 }, '준비 정보 확인 필요'],
  ['full bounded history is valid', { ...ready, requiredCompletedBars: 512, availableCompletedBars: 512 }, '필요 완료봉 512 · 확보 512'],
  ['empty history remains insufficient', { ...ready, ready: false, status: 'insufficientHistory', requiredCompletedBars: 1, availableCompletedBars: 0 }, '필요 완료봉 1 · 확보 0'],
] as const) {
  test(`preparation ${name} remains distinct in current and saved results`, async ({ page }) => {
    await setup(page, { params: { ...legacyParams, ...policyParams }, diagnostic: diagnostic as VolatilityExpansionPreparation })
    await runButton(page).click(); await expect(card(page).getByTestId('volatility-expansion-preparation')).toContainText(expected)
    if (name === 'live cadence before verification') {
      await expect(card(page).getByTestId('volatility-expansion-preparation')).toContainText('실시간 관측 가격 범위')
      await expect(card(page).getByTestId('volatility-expansion-preparation')).not.toContainText('검증된 시장일')
      await expect(card(page).getByTestId('volatility-expansion-preparation')).not.toContainText('준비 완료')
    }
    await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
    await expect(card(page).getByTestId('strategy-ab-comparison')).toContainText(expected)
  })
}

test('active legacy response without preparation and damaged old A/B remain unverified', async ({ page }) => {
  await setup(page, { params: { ...legacyParams, ...policyParams } })
  await runButton(page).click(); await expect(card(page).getByTestId('volatility-expansion-preparation')).toContainText('준비 정보 없음')
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await page.evaluate(() => { for (const [key, raw] of Object.entries(localStorage)) { if (!raw.includes('volatility_expansion_default')) continue; const saved = JSON.parse(raw); if (saved.A) { saved.A.params = { profit_trailing_pct: null }; saved.A.replay.engineVersion = 'strategy-replay-v20'; localStorage.setItem(key, JSON.stringify(saved)) } } })
  await page.reload(); await runButton(page).click()
  await card(page).getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = card(page).getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('이전 10완료봉 (누락 · 기본) 범위 평균 × 2배 (누락 · 기본) 초과'); await expect(ab).toContainText('되밀림 확인 필요 (null)')
  await expect(ab).toContainText('이전 엔진의 새 보호 적용·장중 최초 발동·실제 체결을')
  await expect(ab).toContainText('준비 정보 없음'); await expect(ab).toContainText('A와 B의 엔진 또는 체결 가정이 다릅니다')
})

test('strict numeric drafts and paired protection block save and both preview modes', async ({ page }) => {
  test.setTimeout(60_000)
  const { requests } = await setup(page)
  await runButton(page).click(); await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const [label, invalids, valid] of [[coreLabels[0], ['', 'NaN', 'Infinity', '0', '-1', '1.5', '513'], '7'], [coreLabels[1], ['', 'NaN', '-1', '1000.1'], '1.7'], [coreLabels[2], ['-1', '99.91'], '2'], [labels[0], ['1000.1'], '0'], [labels[1], ['99.91'], '0'], [labels[2], ['1.5', '1000001'], '0']] as const) {
    for (const raw of invalids) {
      const input = card(page).getByLabel(label, { exact: true })
      await input.fill(raw); await expect(input).toHaveValue(raw); await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
      await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
      await input.fill(valid); await expect(runButton(page)).toBeEnabled()
    }
  }
  await card(page).getByLabel(labels[0], { exact: true }).fill('5'); await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
  await expect(card(page)).toContainText('둘 다 양수여야')
  await card(page).getByLabel(labels[1], { exact: true }).fill('3'); await expect(runButton(page)).toBeEnabled()
  await continuous(page); await card(page).getByLabel(labels[2], { exact: true }).fill('2.5')
  await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled(); expect(requests).toHaveLength(1)
})

for (const [name, invalid] of [
  ['stored numeric string', { lookback_days: '7' }], ['stored null', { profit_trailing_pct: null }],
  ['stored fractional integer', { max_hold_observations: 1.5 }], ['stored boolean', { expansion_factor: false }], ['stored unpaired protection', { profit_activation_pct: 5 }],
  ['stored core null', { stop_loss_pct: null }], ['stored fractional lookback', { lookback_days: 2.5 }],
] as const) {
  test(`${name} remains invalid until deliberate edits repair it`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...legacyParams, ...invalid } })
    await expect(runButton(page)).toBeDisabled(); await card(page).getByLabel('1회 수량', { exact: true }).fill('2'); await expect(saveButton(page)).toBeDisabled()
    await expect(card(page).getByLabel('평균 기간 (완료 일봉)', { exact: true })).toHaveValue(String(({ ...legacyParams, ...invalid } as Params).lookback_days))
    await expect(card(page).getByLabel('변동성 손절 기준 (%)', { exact: true })).toHaveValue(String(({ ...legacyParams, ...invalid } as Params).stop_loss_pct))
    for (const label of labels) await card(page).getByLabel(label, { exact: true }).fill('')
    for (const [index, value] of ['7', '1.7', '2'].entries()) { const input = card(page).getByLabel(coreLabels[index], { exact: true }); await input.fill(''); await input.fill(value) }
    await setPolicy(page); await saveAndWait(page)
    expect(updates[0].params).toMatchObject({ ...legacyParams, ...policyParams })
  })
}

test('generic VolatilityExpansion remains daily only even with only profit and cap', async ({ page }) => {
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
  await setPolicy(page); await saveButton(page).click(); await expect(page.getByRole('alert').filter({ hasText: '변동성 확장 설정 저장 실패' })).toBeVisible()
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('3')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2); expect(updates[1]).toEqual(updates[0])
})

test('late preview cannot publish after invalid edits and mobile stays inside card', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 375, height: 844 }); await setup(page, { delay, started })
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await runButton(page).click(); await entered; await card(page).getByLabel(coreLabels[0], { exact: true }).fill('NaN')
  release(); await (await finished).finished(); await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  expect(await card(page).getByTestId('volatility-expansion-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
  await page.screenshot({ path: '.cache/volatility-expansion-mobile.png', fullPage: true })
})

test('active strategy locks every optional control', async ({ page }) => {
  await setup(page, { enabled: true })
  for (const label of [...coreLabels, ...labels]) await expect(card(page).getByLabel(label, { exact: true })).toBeDisabled()
})

test('decimal keyboard drafts and inclusive limits save as numeric values', async ({ page }) => {
  const { updates } = await setup(page, { params: { ...legacyParams, profit_activation_pct: 5, profit_trailing_pct: 3 } })
  const input = card(page).getByLabel(labels[1], { exact: true })
  await input.fill(''); await input.pressSequentially('0.'); await expect(input).toHaveValue('0.'); await input.pressSequentially('5'); await expect(input).toHaveValue('0.5')
  await saveAndWait(page); expect(updates[0].params.profit_trailing_pct).toBe(0.5)
  for (const [index, value] of ['1000', '99.9', '1000000'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
  await saveAndWait(page)
  expect(updates[1].params).toMatchObject({ profit_activation_pct: 1000, profit_trailing_pct: 99.9, max_hold_observations: 1000000 })
  for (const [index, value] of ['512', '1000', '99.9'].entries()) await card(page).getByLabel(coreLabels[index], { exact: true }).fill(value)
  await saveAndWait(page)
  expect(updates[2].params).toMatchObject({ lookback_days: 512, expansion_factor: 1000, stop_loss_pct: 99.9 })
  for (const [index, value] of ['1', '0', '0'].entries()) await card(page).getByLabel(coreLabels[index], { exact: true }).fill(value)
  await saveAndWait(page)
  expect(updates[3].params).toMatchObject({ lookback_days: 1, expansion_factor: 0, stop_loss_pct: 0 })
})

test('save completion preserves a newer draft for a second scoped save', async ({ page }) => {
  let release!: () => void, started!: () => void
  const saveDelay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const { updates } = await setup(page, { saveDelay, saveStarted: started })
  await setPolicy(page); await saveButton(page).click(); await entered
  await card(page).getByLabel(labels[1], { exact: true }).fill('4.5')
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategies/volatility_expansion_default'))
  release(); await (await finished).finished(); await expect(saveButton(page)).toBeEnabled()
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('4.5')
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
  await card(page).getByLabel(coreLabels[0], { exact: true }).fill('100')
  await runButton(page).click(); await expect(card(page).getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests).toHaveLength(2); expect(requests[1].params.lookback_days).toBe(100)
})

test('mobile result and saved diagnostics remain readable without horizontal overflow', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 844 })
  await setup(page, { params: { ...legacyParams, ...policyParams }, diagnostic: ready })
  await runButton(page).click()
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  for (const id of ['volatility-expansion-controls', 'volatility-expansion-summary', 'volatility-expansion-preparation', 'strategy-ab-comparison']) {
    for (const element of await card(page).getByTestId(id).all()) expect(await element.evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
  }
  await card(page).getByTestId('volatility-expansion-preparation').first().scrollIntoViewIfNeeded()
  await page.screenshot({ path: '.cache/volatility-expansion-result-mobile.png', fullPage: true })
})

test('preview failure retains numeric settings and retry returns verified diagnostics', async ({ page }) => {
  const options = { params: { ...legacyParams, ...policyParams }, diagnostic: ready, failPreview: true }
  const { requests, unsafe } = await setup(page, options)
  await runButton(page).click()
  await expect(card(page)).toContainText('변동성 확장 계산 실패')
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('3')
  options.failPreview = false
  await runButton(page).click()
  await expect(card(page).getByTestId('volatility-expansion-preparation')).toContainText('준비 완료')
  expect(requests[1]).toEqual(requests[0]); expect(unsafe).toEqual([])
})

for (const [model, option, currentLabel] of [
  ['sameClose', '신호 봉 종가 체결 가정', '체결 신호 봉 종가'],
  ['nextOpen', '다음 관측 봉 시가 체결 가정', '체결 다음 관측 봉 시가'],
] as const) {
  test(`${model} current and restored A/B preserve actual execution assumption with conditional policy text`, async ({ page }) => {
    const { requests } = await setup(page, { diagnostic: ready })
    const panel = card(page)
    await panel.getByRole('combobox', { name: /^체결 모델/ }).click()
    await page.getByRole('option', { name: option, exact: true }).click()
    const controls = panel.getByTestId('volatility-expansion-controls')
    await expect(controls).toContainText('nextOpen 선택 시 다음 관측 시가에 체결을 가정')
    await expect(controls).toContainText('sameClose 선택 시 신호 봉 종가에 체결을 가정')
    await runButton(page).click()
    await expect(panel.getByText(currentLabel, { exact: true })).toBeVisible()
    expect(requests[0].assumptions.executionModel).toBe(model)
    await expect(panel.getByTestId('volatility-expansion-summary')).toContainText('sameClose 선택 시 신호 봉 종가, nextOpen 선택 시 다음 관측 시가')
    await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
    await page.reload(); await runButton(page).click()
    const ab = panel.getByTestId('strategy-ab-comparison')
    await expect(ab).toContainText(model === 'sameClose' ? '신호 봉 종가' : '다음 관측 봉 시가')
    await expect(ab.getByTestId('volatility-expansion-summary')).toContainText('sameClose 선택 시 신호 봉 종가, nextOpen 선택 시 다음 관측 시가')
    const savedModel = await page.evaluate(() => Object.values(localStorage).map((value) => { try { return JSON.parse(value) } catch { return null } }).find((value) => value?.A?.strategyId === 'volatility_expansion_default')?.A?.backtest.assumptions.executionModel)
    expect(savedModel).toBe(model)
  })
}
