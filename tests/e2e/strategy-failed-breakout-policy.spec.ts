import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle, EntryFilterPreparation } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { lookback_days: 30, buffer_pct: 0.7 }
const policyParams = { breakout_basis: 'completedHigh', atr_buffer_multiplier: 0.5, min_volume_ratio: 1.5, profit_activation_pct: 5, profit_trailing_pct: 3, max_hold_observations: 63 }
const ready: EntryFilterPreparation = { enabled: true, ready: true, status: 'ready', cadence: 'priorCompletedDailyOhlcv', requiredCompletedBars: 30, availableCompletedBars: 252, trueRangeSamples: 14,
  candidates: 9, unreadyCandidates: 1, minMoveRejections: 0, atrRejections: 2, trendRejections: 0, volumeRejections: 3, passedCandidates: 4, breakoutBasis: 'completedHigh', priorAtr: 0, priorVolumeRatio: 0 }
function response(body: any, continuous: boolean, diagnostic?: EntryFilterPreparation) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-22T12:00:00+09:00', message: 'failed_breakout fixture',
    ...(diagnostic ? { entryFilterPreparation: diagnostic } : {}),
    replay: { engineVersion: 'strategy-replay-v19', strategyVersion: 'fixture', sourceInterval: body.interval, replayCadence: 'dailyCloseWithDayBoundary', liveCadenceSeconds: 10, warmupCount: body.historyCandles.length, dataStart: first, dataEnd: last, dataSource: body.dataSource, deterministic: true, lookAheadSafe: false, inputHash: JSON.stringify(body.params) },
    backtest: { assumptions: body.assumptions,
      summary: { initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0, mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0, winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0, signalCount: 0, orderEligibleCount: 0, filledOrderCount: 0, blockedOrderCount: 0 },
      trades: [], equityCurve: [], phases: [], overfitWarning: null,
    },
    ...(continuous ? { continuity: { mode: 'continuous', chunkCount: body.chunks.length, evaluationBars: candles.length, historyBars: body.historyCandles.length,
      chunks: body.chunks.map((chunk: any, index: number) => ({ index, start: chunk.candles[0].date, end: chunk.candles[chunk.candles.length - 1].date, bars: chunk.candles.length, startQuantity: 0, endQuantity: 0, pendingAtEnd: false })),
    } } : {}),
  }
}
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'FailedBreakoutStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void; saveDelay?: Promise<void>; saveStarted?: () => void; diagnostic?: EntryFilterPreparation; toss?: boolean } = {}) {
  const requests: any[] = [], updates: any[] = [], unsafe: string[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { activeBroker: options.toss ? 'toss' : 'kis', chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  page.on('request', (request) => { if (/\/api\/(?:order|trading\/(?:start|stop))/.test(new URL(request.url()).pathname) && request.method() !== 'GET') unsafe.push(request.url()) })
  const strategy = () => ({ id: 'failed_breakout_default', name: 'FailedBreakoutStrategy', enabled: options.enabled ?? false, brokerId: options.toss ? 'toss' : 'kis', brokerAccountId: options.toss ? '1' : '12345678-01', targetSymbols: ['000005'], targetSymbolNames: { '000005': 'Mock failed_breakout' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/failed_breakout_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    options.saveStarted?.(); if (options.saveDelay) await options.saveDelay
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: '돌파 설정 저장 실패' } }); return }
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
const labels = ['돌파 일봉 ATR 버퍼 배수', '최소 완료 일봉 거래량 비율', '돌파 수익 보호 시작 (%)', '돌파 수익 보호 되밀림 (%)', '돌파 최대 보유 관측 수'] as const
const runButton = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
const saveButton = (page: Page) => card(page).getByRole('button', { name: '변경사항 저장', exact: true })
async function basis(page: Page, high: boolean) {
  await card(page).getByRole('combobox', { name: /^돌파 기준/ }).click()
  await page.getByRole('option', { name: high ? '이전 완료 일봉 고가' : '이전 가격관측 최대값 (기본)', exact: true }).click()
}
async function setPolicy(page: Page) {
  await basis(page, true)
  for (const [index, value] of ['0.5', '1.5', '5', '3', '63'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
}
async function continuous(page: Page) {
  await card(page).getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await card(page).getByLabel('연속 시작일').fill('2025-01-01')
  await card(page).getByLabel('연속 종료일').fill('2025-12-31')
}

test('custom legacy settings preserve omitted options and disclose actual holdings and timing', async ({ page }) => {
  const { requests, updates, unsafe } = await setup(page)
  const panel = card(page), controls = panel.getByTestId('failed-breakout-policy-controls')
  await expect(panel.getByLabel('전고점 기간', { exact: true })).toHaveValue('30')
  await expect(panel.getByLabel('돌파 버퍼 (%)', { exact: true })).toHaveValue('0.7')
  for (const label of labels) await expect(panel.getByLabel(label, { exact: true })).toHaveValue('0')
  await expect(panel.getByRole('combobox', { name: /^돌파 기준/ })).toContainText('이전 가격관측 최대값 (기본)')
  for (const text of ['N관측은 N일이 아닙니다', '현재 봉 고가를 돌파 신호로 쓰지', '실제 보유 수량·종목 통화 기준 평균 매수가', '버퍼 전 진입 기준 고점 미만', '주문 거절·부분 체결·설정 변경', '신호는 체결이 아니', '이전 완료 15일봉', '총 21개 이전 봉', '현재 봉 거래량·실시간 누적량을 사용하지', '실제 보유 중 관측 최고가', '실패 → 수익 보호 → 보유 한도', '나이가 미확인이면 다음 보유 관측', '손익이나 실제 보유시간을 보장하지']) await expect(controls).toContainText(text)
  await runButton(page).click()
  await expect(panel.getByTestId('failed-breakout-policy-summary')).toContainText('기간 30관측/봉 · 버퍼 0.7%')
  await expect(panel.getByTestId('failed-breakout-preparation')).toContainText('완료 일봉 필터 비활성')
  expect(requests[0].params).toMatchObject(legacyParams)
  expect(requests[0].params).not.toHaveProperty('breakout_basis')
  await panel.getByLabel('1회 수량', { exact: true }).fill('2'); await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1); await page.reload()
  await expect(panel.getByLabel('돌파 버퍼 (%)', { exact: true })).toHaveValue('0.7')
  for (const key of Object.keys(policyParams)) expect(updates[0].params).not.toHaveProperty(key)
  expect(unsafe).toEqual([])
})

test('standard 20 and 0.5 settings retain explicit zero options without promotion', async ({ page }) => {
  const { requests } = await setup(page, { params: { lookback_days: 20, buffer_pct: 0.5, breakout_basis: 'priceObservations', atr_buffer_multiplier: 0, min_volume_ratio: 0, profit_activation_pct: 0, profit_trailing_pct: 0, max_hold_observations: 0 } })
  await runButton(page).click()
  await expect(card(page).getByTestId('failed-breakout-policy-summary')).toContainText('기간 20관측/봉 · 버퍼 0.5%')
  expect(requests[0].params).toMatchObject({ atr_buffer_multiplier: 0, min_volume_ratio: 0, profit_activation_pct: 0, profit_trailing_pct: 0, max_hold_observations: 0 })
})

test('optional basis and protections survive scoped save and reload', async ({ page }) => {
  const { updates } = await setup(page)
  await setPolicy(page); await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...policyParams } })
  await page.reload()
  for (const [index, value] of ['0.5', '1.5', '5', '3', '63'].entries()) await expect(card(page).getByLabel(labels[index], { exact: true })).toHaveValue(value)
  await expect(card(page).getByRole('combobox', { name: /^돌파 기준/ })).toContainText('이전 완료 일봉 고가')
})

test('bounded and continuous requests and A/B retain params, sizing and diagnostic counts', async ({ page }) => {
  const options = { diagnostic: undefined as EntryFilterPreparation | undefined }
  const { requests } = await setup(page, options)
  const panel = card(page)
  await runButton(page).click(); await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  options.diagnostic = ready
  await setPolicy(page); await continuous(page)
  await panel.getByLabel('백테스트 ATR 기반 수량 산정', { exact: true }).check()
  await runButton(page).click(); await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...policyParams }); expect(requests[1]).not.toHaveProperty('candles')
  expect(requests[1].assumptions.volatilitySizingEnabled).toBe(true)
  const diagnostic = panel.getByTestId('failed-breakout-preparation').first()
  for (const text of ['준비 완료', '필요 완료봉 30 · 확보 252 · 실제 TR 14개', '이전 ATR 0', '0배 (검증된 0)', '돌파 후보 9', '준비 미확인 보류 1', 'ATR 거부 2', '거래량 거부 3', '후보 통과 4', '수량 산정용 ATR 준비와 별도']) await expect(diagnostic).toContainText(text)
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('A와 B의 돌파 파라미터가 다릅니다')
  await expect(ab).toContainText('누락 · 기본 비활성'); await expect(ab).toContainText('수익 보호 시작 5% / 되밀림 3%')
  const snapshots = await page.evaluate(() => Object.values(localStorage).filter((value) => value.includes('failed_breakout_default')).map((value) => JSON.parse(value)))
  expect(snapshots[0].B.params).toMatchObject(policyParams); expect(snapshots[0].B.entryFilterPreparation).toMatchObject({ priorAtr: 0, priorVolumeRatio: 0, volumeRejections: 3 })
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click(); await page.getByRole('option', { name: '단기 구간 · 최근 50/100/200봉', exact: true }).click()
  await runButton(page).click(); await expect.poll(() => requests.length).toBe(3)
  expect(requests[2].params).toMatchObject(policyParams)
})

for (const [name, diagnostic, expected] of [
  ['unverified volume', { ...ready, ready: false, status: 'unverifiedVolume', priorAtr: undefined, priorVolumeRatio: undefined }, '완료 거래량 미확인'],
  ['zero reference', { ...ready, ready: false, status: 'zeroReferenceVolume', priorVolumeRatio: undefined }, '기준 거래량 평균 0'],
  ['unsupported cadence', { ...ready, ready: false, status: 'unsupportedObservation' }, '관측 형식 미지원'],
  ['unknown future status', { ...ready, status: 'futureStatus' }, '진입 준비 형식 미지원'],
] as const) {
  test(`preparation ${name} remains distinct in current and saved results`, async ({ page }) => {
    await setup(page, { params: { ...legacyParams, ...policyParams }, diagnostic: diagnostic as EntryFilterPreparation })
    await runButton(page).click()
    const preparation = card(page).getByTestId('failed-breakout-preparation')
    await expect(preparation).toContainText(expected)
    if (name === 'unverified volume' || name === 'zero reference') { await expect(preparation).toContainText('거래량 비율 미확인'); await expect(preparation).not.toContainText('0배 (검증된 0)') }
    await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
    await expect(card(page).getByTestId('strategy-ab-comparison')).toContainText(expected)
  })
}

test('active old responses without diagnostics and old parameters remain unverified', async ({ page }) => {
  await setup(page, { params: { ...legacyParams, ...policyParams } })
  await runButton(page).click(); await expect(card(page).getByTestId('failed-breakout-preparation')).toContainText('진입 준비 정보 없음')
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await page.evaluate(() => { for (const [key, raw] of Object.entries(localStorage)) { if (!raw.includes('failed_breakout_default')) continue; const saved = JSON.parse(raw); if (saved.A) { saved.A.params = { profit_trailing_pct: null }; saved.A.replay.engineVersion = 'strategy-replay-v18'; localStorage.setItem(key, JSON.stringify(saved)) } } })
  await page.reload(); await runButton(page).click()
  await card(page).getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = card(page).getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('기간 20 (기본)관측/봉 · 버퍼 0.5 (기본)%')
  await expect(ab).toContainText('되밀림 확인 필요 (null)'); await expect(ab).toContainText('이전 엔진·v1 복원에서 새 보호 적용')
  await expect(ab).toContainText('A와 B의 엔진 또는 체결 가정이 다릅니다')
})

test('strict numeric drafts and paired protection invalidate results and block save and both previews', async ({ page }) => {
  const { requests } = await setup(page)
  await runButton(page).click()
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const [label, invalids] of [[labels[0], ['', 'NaN', 'Infinity', '-1', '20.1']], [labels[1], ['100.1']], [labels[4], ['1.5', '1000001']]] as const) {
    const input = card(page).getByLabel(label, { exact: true })
    for (const raw of invalids) {
      await input.fill(raw); await expect(input).toHaveValue(raw); await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
      await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
      await input.fill('0'); await expect(runButton(page)).toBeEnabled()
    }
  }
  await card(page).getByLabel(labels[2], { exact: true }).fill('5')
  await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled()
  await expect(card(page)).toContainText('둘 다 양수여야')
  await card(page).getByLabel(labels[3], { exact: true }).fill('3'); await expect(runButton(page)).toBeEnabled()
  for (const [label, raw, repair] of [[labels[2], '1000.1', '5'], [labels[3], '99.91', '3']] as const) {
    await card(page).getByLabel(label, { exact: true }).fill(raw); await expect(runButton(page)).toBeDisabled()
    await card(page).getByLabel(label, { exact: true }).fill(repair)
  }
  await continuous(page); await card(page).getByLabel(labels[4], { exact: true }).fill('2.5')
  await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeDisabled(); expect(requests).toHaveLength(1)
})

for (const [name, invalid] of [
  ['stored numeric string', { min_volume_ratio: '1.5' }], ['stored null', { atr_buffer_multiplier: null }],
  ['stored fractional integer', { max_hold_observations: 1.5 }], ['stored boolean', { profit_activation_pct: false }], ['stored invalid basis', { breakout_basis: 'currentHigh' }],
] as const) {
  test(`${name} remains invalid until deliberate edits repair it`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...legacyParams, ...invalid } })
    await expect(runButton(page)).toBeDisabled(); await card(page).getByLabel('1회 수량', { exact: true }).fill('2'); await expect(saveButton(page)).toBeDisabled()
    await expect(card(page).getByLabel('돌파 버퍼 (%)', { exact: true })).toHaveValue('0.7')
    for (const label of labels) await card(page).getByLabel(label, { exact: true }).fill('')
    await setPolicy(page); await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
    expect(updates[0].params).toMatchObject({ ...legacyParams, ...policyParams })
  })
}

test('daily inputs block minute preview while profit and cap alone allow it and save', async ({ page }) => {
  const { requests, updates } = await setup(page, { toss: true })
  const panel = card(page)
  await panel.getByRole('combobox', { name: /^봉 단위/ }).click(); await page.getByRole('option', { name: '1분봉', exact: true }).click()
  await expect(runButton(page)).toBeEnabled()
  await basis(page, true); await expect(runButton(page)).toBeDisabled(); await expect(saveButton(page)).toBeEnabled()
  await expect(panel).toContainText('이전 완료 일봉이 필요합니다. 일봉을 선택하세요. 실시간 저장은 지원합니다.')
  await basis(page, false)
  for (const label of labels.slice(0, 2)) { await panel.getByLabel(label, { exact: true }).fill('1'); await expect(runButton(page)).toBeDisabled(); await panel.getByLabel(label, { exact: true }).fill('0') }
  await panel.getByLabel(labels[2], { exact: true }).fill('5'); await panel.getByLabel(labels[3], { exact: true }).fill('3'); await panel.getByLabel(labels[4], { exact: true }).fill('63')
  await expect(runButton(page)).toBeEnabled(); await saveButton(page).click(); await expect.poll(() => updates.length).toBe(1)
  await expect(saveButton(page)).toHaveCount(0)
  await runButton(page).click(); await expect.poll(() => requests.length).toBe(1)
  expect(requests[0].interval).toBe('1m'); expect(requests[0].params).toMatchObject({ profit_activation_pct: 5, profit_trailing_pct: 3, max_hold_observations: 63 })
})

test('save failure retains identical scoped settings for retry', async ({ page }) => {
  const { updates } = await setup(page, { failSave: true })
  await setPolicy(page); await saveButton(page).click(); await expect(page.getByRole('alert').filter({ hasText: '돌파 설정 저장 실패' })).toBeVisible()
  await expect(card(page).getByLabel(labels[3], { exact: true })).toHaveValue('3')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2); expect(updates[1]).toEqual(updates[0])
})

test('late preview response cannot publish after invalid edits and mobile has no overflow', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 375, height: 844 }); await setup(page, { delay, started })
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await runButton(page).click(); await entered; await card(page).getByLabel(labels[0], { exact: true }).fill('NaN')
  release(); await (await finished).finished(); await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  expect(await card(page).getByTestId('failed-breakout-policy-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
  await page.screenshot({ path: '.cache/failed-breakout-mobile.png', fullPage: true })
})

test('active strategy locks basis and every optional control', async ({ page }) => {
  await setup(page, { enabled: true })
  for (const label of labels) await expect(card(page).getByLabel(label, { exact: true })).toBeDisabled()
  await expect(card(page).getByRole('combobox', { name: /^돌파 기준/ })).toHaveAttribute('aria-disabled', 'true')
})

test('decimal keyboard drafts and inclusive limits save as numeric values', async ({ page }) => {
  const { updates } = await setup(page)
  const input = card(page).getByLabel(labels[0], { exact: true })
  await input.fill(''); await input.pressSequentially('0.'); await expect(input).toHaveValue('0.'); await input.pressSequentially('5'); await expect(input).toHaveValue('0.5')
  // 경계 입력 검증은 첫 저장의 canonical 응답까지 완료한 상태에서 시작한다.
  const saved = page.waitForResponse((res) => res.url().endsWith('/api/strategies/failed_breakout_default'))
  await saveButton(page).click(); await (await saved).finished(); await expect(saveButton(page)).toHaveCount(0)
  expect(updates[0].params.atr_buffer_multiplier).toBe(0.5)
  for (const [index, value] of ['20', '100', '1000', '99.9', '1000000'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toMatchObject({ atr_buffer_multiplier: 20, min_volume_ratio: 100, profit_activation_pct: 1000, profit_trailing_pct: 99.9, max_hold_observations: 1000000 })
})

test('save completion preserves newer draft for a second scoped save', async ({ page }) => {
  let release!: () => void, started!: () => void
  const saveDelay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const { updates } = await setup(page, { saveDelay, saveStarted: started })
  await setPolicy(page); await saveButton(page).click(); await entered
  await card(page).getByLabel(labels[3], { exact: true }).fill('4.5')
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategies/failed_breakout_default'))
  release(); await (await finished).finished(); await expect(saveButton(page)).toBeEnabled()
  await expect(card(page).getByLabel(labels[3], { exact: true })).toHaveValue('4.5')
  await saveButton(page).click(); await expect.poll(() => updates.length).toBe(2)
  expect(updates[0].params).toMatchObject(policyParams); expect(updates[1].params).toMatchObject({ ...policyParams, profit_trailing_pct: 4.5 })
  await expect(saveButton(page)).toHaveCount(0)
})
