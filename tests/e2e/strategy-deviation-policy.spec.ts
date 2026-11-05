import { expect, test, type Page } from '@playwright/test'
import type { ChartCandle } from '../../src/api/types'
import { historyFixture, mockApi } from './fixtures/strategyResearchFixture'

type Params = Record<string, unknown>
const legacyParams = { ma_period: 30, buy_threshold_pct: -6, sell_threshold_pct: 7 }
const crossingParams = { entry_mode: 'crossing', long_trend_period: 200, max_hold_observations: 63, stop_loss_pct: 8 }
function response(body: any, continuous: boolean) {
  const candles: ChartCandle[] = continuous ? body.chunks.flatMap((chunk: any) => chunk.candles) : body.candles
  const first = candles[0].date, last = candles[candles.length - 1].date
  return {
    strategyId: body.strategyId, symbol: body.symbol, candles, signals: [], generatedAt: '2026-10-18T12:00:00+09:00', message: 'deviation fixture',
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
function card(page: Page) { return page.getByTestId('strategy-card-grid').filter({ hasText: 'DeviationStrategy' }) }
async function setup(page: Page, options: { params?: Params; enabled?: boolean; failSave?: boolean; delay?: Promise<void>; started?: () => void; saveDelay?: Promise<void>; saveStarted?: () => void } = {}) {
  const requests: any[] = [], updates: any[] = []
  let stored = options.params ?? { ...legacyParams }
  await mockApi(page, { chartCandles: historyFixture(731).map((candle, index) => ({ ...candle, date: new Date(Date.UTC(2024, 0, 1 + index)).toISOString().slice(0, 10).replace(/-/g, '') })) })
  const strategy = () => ({ id: 'deviation_default', name: 'DeviationStrategy', enabled: options.enabled ?? false, brokerId: 'kis', brokerAccountId: '12345678-01', targetSymbols: ['000005'], targetSymbolNames: { '000005': 'Mock deviation' }, orderQuantity: 1, params: stored })
  await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
  await page.route('**/api/strategies/deviation_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    options.saveStarted?.(); if (options.saveDelay) await options.saveDelay
    if (options.failSave) { await route.fulfill({ status: 500, json: { code: 'PERSISTENCE_ERROR', error: 'Deviation 저장 실패' } }); return }
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
  await card(page).getByRole('combobox', { name: /^Deviation 진입 모드/ }).click()
  await page.getByRole('option', { name: mode === 'level' ? 'level · 이격 상태 (기본)' : 'crossing · 하향 교차', exact: true }).click()
}

const labels = ['진입 추세 MA 관측 수', '최대 보유 관측 수', '보유 손절 기준 (%)'] as const
const runButton = (page: Page) => card(page).getByRole('button', { name: '미리보기 계산', exact: true })
const saveButton = (page: Page) => card(page).getByRole('button', { name: '변경사항 저장', exact: true })
async function setPolicy(page: Page) {
  await chooseMode(page, 'crossing')
  for (const [index, value] of ['200', '63', '8'].entries()) await card(page).getByLabel(labels[index], { exact: true }).fill(value)
}
async function continuous(page: Page) {
  await card(page).getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '연속 일봉 · 날짜 범위', exact: true }).click()
  await card(page).getByLabel('연속 시작일').fill('2025-01-01')
  await card(page).getByLabel('연속 종료일').fill('2025-12-31')
}

test('legacy custom deviation settings keep omitted optional defaults through save and reload', async ({ page }) => {
  const { requests, updates } = await setup(page)
  const panel = card(page), controls = panel.getByTestId('deviation-policy-controls')
  await expect(panel.getByLabel('MA 기간', { exact: true })).toHaveValue('30')
  await expect(panel.getByLabel('매수 이격 (%)', { exact: true })).toHaveValue('-6')
  await expect(panel.getByLabel('매도 이격 (%)', { exact: true })).toHaveValue('7')
  await expect(panel.getByRole('combobox', { name: /^Deviation 진입 모드/ })).toContainText('level · 이격 상태 (기본)')
  for (const label of labels) await expect(panel.getByLabel(label, { exact: true })).toHaveValue('0')
  await expect(controls).toContainText('이전 이격도 > 매수 기준, 현재 이격도 ≤ 매수 기준')
  await expect(controls).toContainText('실시간은 시작 후 실제 가격 N관측을 모아 다음 관측부터')
  await expect(controls).toContainText('옵션이 꺼져 있어도 실제 양수 보유 이후 누적')
  await expect(controls).toContainText('복원 보유의 age 정보가 없고')
  await expect(controls).toContainText('손절 → 최대 보유 → 고이격')
  await expect(controls).toContainText('실제 소유 잔량 전부')
  await expect(controls).toContainText('다음 시가 체결·갭')
  await runButton(page).click()
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  expect(requests[0].params).toMatchObject(legacyParams)
  expect(requests[0].params).not.toHaveProperty('entry_mode')
  await panel.getByLabel('1회 수량', { exact: true }).fill('2')
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params).toMatchObject(legacyParams)
  await page.reload()
  await expect(panel.getByLabel('MA 기간', { exact: true })).toHaveValue('30')
})

test('four optional settings retain exact types through scoped save and reloaded configuration', async ({ page }) => {
  const { updates } = await setup(page)
  await setPolicy(page)
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0]).toMatchObject({ expectedProfileId: 'paper', expectedBrokerId: 'kis', expectedBrokerAccountId: '12345678-01', params: { ...legacyParams, ...crossingParams } })
  await page.reload()
  await expect(card(page).getByRole('combobox', { name: /^Deviation 진입 모드/ })).toContainText('crossing · 하향 교차')
  for (const [index, value] of ['200', '63', '8'].entries()) await expect(card(page).getByLabel(labels[index], { exact: true })).toHaveValue(value)
})

test('short and continuous replay send all settings and A/B preserves each assumption summary', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await runButton(page).click()
  await panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await setPolicy(page)
  await continuous(page)
  await runButton(page).click()
  await expect(panel.getByTestId('continuous-replay-continuity')).toBeVisible()
  expect(requests[0].params).toMatchObject(legacyParams)
  expect(requests[0].params).not.toHaveProperty('entry_mode')
  expect(requests[1].params).toMatchObject({ ...legacyParams, ...crossingParams })
  expect(requests[1]).not.toHaveProperty('candles')
  await expect(panel).toContainText('Deviation 진입 · crossing · 추세 MA 200관측 · 최대 보유 63관측 · 손절 8%')
  await panel.getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('A와 B의 Deviation 파라미터가 다릅니다')
  await expect(ab).toContainText('level (기본 level) · 추세 MA 0관측 · 최대 보유 0관측 · 손절 0%')
  await expect(ab).toContainText('crossing · 추세 MA 200관측 · 최대 보유 63관측 · 손절 8%')
  const snapshots = await page.evaluate(() => Object.values(localStorage).filter((value) => value.includes('deviation_default')).map((value) => JSON.parse(value)))
  expect(snapshots[0].B.params).toMatchObject({ ...legacyParams, ...crossingParams })
  // Exercise the short endpoint with enabled settings as well.
  await panel.getByRole('combobox', { name: '계산 범위 모드' }).click()
  await page.getByRole('option', { name: '단기 구간 · 최근 50/100/200봉', exact: true }).click()
  await runButton(page).click()
  await expect.poll(() => requests.length).toBe(3)
  expect(requests[2].params).toMatchObject({ ...legacyParams, ...crossingParams })
})

test('invalid drafts retain raw text, clear prior results, and block save and both preview modes', async ({ page }) => {
  const { requests } = await setup(page)
  const panel = card(page)
  await runButton(page).click()
  await expect(panel.getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toBeVisible()
  for (const [label, invalids] of [
    [labels[0], ['', 'NaN', 'Infinity', '-1', '513', '1.5']],
    [labels[1], ['1000001', '2.5']], [labels[2], ['101', '-0.1']],
  ] as const) {
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
  ['unknown enum', { entry_mode: 'bogus' }], ['null enum', { entry_mode: null }],
  ['stored numeric string', { long_trend_period: '200' }], ['stored null', { max_hold_observations: null }],
  ['stored fractional integer', { max_hold_observations: 1.5 }], ['stored stop string', { stop_loss_pct: '8' }],
] as const) {
  test(`${name} remains invalid until a real edit repairs its stored type`, async ({ page }) => {
    const { updates } = await setup(page, { params: { ...legacyParams, ...invalid } })
    await expect(runButton(page)).toBeDisabled()
    await card(page).getByLabel('1회 수량', { exact: true }).fill('2')
    await expect(saveButton(page)).toBeDisabled()
    // Clear before retyping: a stored string can have the same visible text as a valid number.
    for (const label of labels) await card(page).getByLabel(label, { exact: true }).fill('')
    await setPolicy(page)
    await saveButton(page).click()
    await expect.poll(() => updates.length).toBe(1)
    expect(updates[0].params).toMatchObject({ ...legacyParams, ...crossingParams })
  })
}

test('save failure preserves optional drafts and permits an identical scoped retry', async ({ page }) => {
  const { updates } = await setup(page, { failSave: true })
  await setPolicy(page)
  await saveButton(page).click()
  await expect(page.getByRole('alert').filter({ hasText: 'Deviation 저장 실패' })).toBeVisible()
  await expect(card(page).getByLabel(labels[1], { exact: true })).toHaveValue('63')
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toEqual(updates[0].params)
})

test('invalid edits discard an in-flight response and mobile controls have no horizontal overflow', async ({ page }) => {
  let release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  await page.setViewportSize({ width: 390, height: 844 })
  await setup(page, { delay, started })
  const finished = page.waitForResponse((res) => res.url().endsWith('/api/strategy/preview'))
  await runButton(page).click()
  await entered
  await card(page).getByLabel(labels[2], { exact: true }).fill('NaN')
  release(); await (await finished).finished()
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))))
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
  expect(await card(page).getByTestId('deviation-policy-controls').evaluate((el) => el.scrollWidth > el.clientWidth + 1)).toBe(false)
})

test('enabled strategies lock the mode and all three optional numeric controls', async ({ page }) => {
  await setup(page, { enabled: true })
  await expect(card(page).getByRole('combobox', { name: /^Deviation 진입 모드/ })).toBeDisabled()
  for (const label of labels) await expect(card(page).getByLabel(label, { exact: true })).toBeDisabled()
})

test('keyboard decimal drafts preserve the dot, while integer limits and stop limit remain inclusive', async ({ page }) => {
  const { updates } = await setup(page)
  const panel = card(page), input = panel.getByLabel(labels[2], { exact: true })
  await input.fill('')
  await input.pressSequentially('1.')
  await expect(input).toHaveValue('1.')
  await input.pressSequentially('2')
  await expect(input).toHaveValue('1.2')
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(1)
  expect(updates[0].params.stop_loss_pct).toBe(1.2)
  await expect(saveButton(page)).toHaveCount(0)
  await panel.getByLabel(labels[0], { exact: true }).fill('512')
  await panel.getByLabel(labels[1], { exact: true }).fill('1000000')
  await input.fill('100')
  await expect(runButton(page)).toBeEnabled()
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[1].params).toMatchObject({ long_trend_period: 512, max_hold_observations: 1000000, stop_loss_pct: 100 })
})

test('a delayed successful save preserves newer Deviation mode and numeric drafts for a second save', async ({ page }) => {
  let release!: () => void, started!: () => void
  const saveDelay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const { updates } = await setup(page, { saveDelay, saveStarted: started })
  await setPolicy(page)
  await saveButton(page).click()
  await entered
  await chooseMode(page, 'level')
  await card(page).getByLabel(labels[2], { exact: true }).fill('9')
  const completed = page.waitForResponse((res) => res.url().endsWith('/api/strategies/deviation_default'))
  release(); await (await completed).finished()
  await expect(saveButton(page)).toBeEnabled()
  await expect(card(page).getByRole('combobox', { name: /^Deviation 진입 모드/ })).toContainText('level · 이격 상태 (기본)')
  await expect(card(page).getByLabel(labels[2], { exact: true })).toHaveValue('9')
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[0].params).toMatchObject(crossingParams)
  expect(updates[1].params).toMatchObject({ ...crossingParams, entry_mode: 'level', stop_loss_pct: 9 })
  await expect(saveButton(page)).toHaveCount(0)
})

for (const kind of ['price_condition', 'leveraged_trend_hold'] as const) {
  test(`${kind} delayed save preserves newer immutable drafts and saves them on retry`, async ({ page }) => {
    await mockApi(page)
    const id = `${kind}_default`, updates: any[] = []
    let release!: () => void, started!: () => void
    const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
    const entry = { leveraged_symbol: 'SOXL', leveraged_symbol_name: 'Mock ETF', base_symbols: [], base_symbol_names: {}, base_symbol_roles: {}, quantity: 3, is_overseas: true }
    let params: Params = kind === 'price_condition'
      ? { symbols: [{ symbol: '005930', symbol_name: 'Mock stock', is_overseas: false, quantity: 1, buy_trigger_price: 100, sell_trigger_price: 0, take_profit_pct: 0, stop_loss_pct: 0 }] }
      : { entries: [entry], initial_stop_loss_pct: 1 }
    const strategy = () => ({ id, name: kind, enabled: false, brokerId: 'kis', brokerAccountId: '12345678-01', targetSymbols: kind === 'price_condition' ? ['005930'] : ['SOXL'], targetSymbolNames: {}, orderQuantity: 1, params })
    await page.route('**/api/strategies', (route) => route.fulfill({ json: [strategy()] }))
    await page.route(`**/api/strategies/${id}`, async (route) => {
      const body = route.request().postDataJSON(); updates.push(body); started(); await delay
      params = body.params; await route.fulfill({ json: strategy() })
    })
    await page.goto('/strategy')
    const panel = page.getByTestId('strategy-card-grid'), save = panel.getByRole('button', { name: '변경사항 저장', exact: true })
    const input = panel.getByLabel(kind === 'price_condition' ? '005930 매수가' : '초기 손절(%)', { exact: true })
    await input.fill(kind === 'price_condition' ? '200' : '2')
    if (kind === 'leveraged_trend_hold') await panel.getByLabel('SOXL 수량', { exact: true }).fill('4')
    await save.click(); await entered
    await input.fill(kind === 'price_condition' ? '300' : '3')
    if (kind === 'leveraged_trend_hold') await panel.getByLabel('SOXL 수량', { exact: true }).fill('5')
    const completed = page.waitForResponse((res) => res.url().endsWith(`/api/strategies/${id}`))
    release(); await (await completed).finished()
    await expect(save).toBeEnabled()
    await expect(input).toHaveValue(kind === 'price_condition' ? '300' : '3')
    await save.click(); await expect.poll(() => updates.length).toBe(2)
    if (kind === 'price_condition') expect(updates[1].params.symbols[0].buy_trigger_price).toBe(300)
    else {
      expect(updates[1].params.initial_stop_loss_pct).toBe(3)
      expect(updates[1].params.entries[0].quantity).toBe(5)
    }
    await expect(save).toHaveCount(0)
  })
}


test('a late successful A save cannot clear new A drafts after an A to B to A scope transition', async ({ page }) => {
  await mockApi(page, { activeBroker: 'toss' })
  // Keep the production router, page and QueryClient mounted throughout both
  // transitions; capture its real per-call cleanup for adversarial late delivery.
  // A scope reset can detach React Query's observer before that delivery.
  let hookInstrumented = false
  const moduleRequests: string[] = []
  page.on('request', (request) => { if (request.url().includes('strategyHooks')) moduleRequests.push(request.url()) })
  await page.route('**/src/api/strategyHooks.ts*', async (route) => {
    hookInstrumented = true
    const response = await route.fetch(), source = await response.text()
    const original = source.match(/function useUpdateStrategy\(\) \{[\s\S]*?\n\}/)?.[0]
    if (!original || !original.includes('return useMutation(')) throw new Error('Production update hook instrumentation no longer matches')
    const instrumented = original.replace('return useMutation(', 'const mutation = useMutation(').replace(/\n\}$/, `
      return { ...mutation, mutate(input, options) {
        if (options?.onSuccess) window.__deviationQueuedSuccess = () => options.onSuccess();
        return mutation.mutate(input, options);
      } };
    }` )
    await route.fulfill({ response, body: source.replace(original, instrumented) })
  })
  await page.route('**/src/main.tsx', async (route) => {
    const response = await route.fetch()
    await route.fulfill({ response, body: await response.text() + `
      window.__deviationScope = {
        refresh: async () => {
          await queryClient.invalidateQueries({ queryKey: ['appConfig'] });
          await queryClient.invalidateQueries({ queryKey: ['strategies'] });
        },
        completed: () => queryClient.getMutationCache().getAll().filter(m => m.state.status === 'success').length,
        deliverQueued: () => {
          if (!window.__deviationQueuedSuccess) throw new Error('No captured production cleanup');
          window.__deviationQueuedSuccess();
        },
      };
    ` })
  })
  let active: 'A' | 'B' = 'A', release!: () => void, started!: () => void
  const delay = new Promise<void>((resolve) => { release = resolve }), entered = new Promise<void>((resolve) => { started = resolve })
  const updates: any[] = [], reads: string[] = []
  const config = () => ({ active_profile_id: active === 'A' ? 'toss-live' : 'profile-b', active_profile_name: active,
    active_broker_id: 'toss', active_broker_account_id: active === 'A' ? '1' : '2', kis_is_paper_trading: true,
    kis_configured: true, active_broker_configured: true, discord_enabled: false, notification_levels: [] })
  const strategy = () => ({ id: 'deviation_default', name: 'DeviationStrategy', enabled: false, brokerId: 'toss',
    brokerAccountId: active === 'A' ? '1' : '2', targetSymbols: ['SOXL'], targetSymbolNames: { SOXL: 'Synthetic ETF' },
    orderQuantity: active === 'A' ? 1 : 9, params: { ...legacyParams, toss_us_session: 'auto' } })
  await page.route('**/api/app-config', (route) => route.fulfill({ json: config() }))
  await page.route('**/api/strategies', (route) => { reads.push(active); return route.fulfill({ json: [strategy()] }) })
  await page.route('**/api/strategies/deviation_default', async (route) => {
    const body = route.request().postDataJSON(); updates.push(body)
    if (updates.length === 1) { started(); await delay }
    // A successful mutation acknowledgement, never an account-fence rejection.
    await route.fulfill({ json: { ...strategy(), params: body.params } })
  })
  await page.goto('/strategy')
  await setPolicy(page)
  expect(hookInstrumented, JSON.stringify(moduleRequests)).toBe(true)
  await card(page).getByRole('button', { name: '프리', exact: true }).click()
  await saveButton(page).click(); await entered
  expect(updates[0]).toMatchObject({ expectedProfileId: 'toss-live', expectedBrokerAccountId: '1', params: { toss_us_session: 'pre' } })
  type Harness = { refresh: () => Promise<void>; completed: () => number; deliverQueued: () => void }
  const refresh = () => page.evaluate(() => (window as unknown as { __deviationScope: Harness }).__deviationScope.refresh())
  active = 'B'; await refresh()
  await expect(card(page).getByLabel('1회 수량', { exact: true })).toHaveValue('9')
  active = 'A'; await refresh()
  await expect(card(page).getByLabel('1회 수량', { exact: true })).toHaveValue('1')
  await expect(card(page).getByLabel(labels[2], { exact: true })).toHaveValue('0')
  expect(reads).toContain('B')
  await chooseMode(page, 'crossing')
  await card(page).getByLabel(labels[2], { exact: true }).fill('9')
  // The same primitive session as the old submission makes epoch protection
  // observable independently of the immutable numeric draft identity check.
  await card(page).getByRole('button', { name: '프리', exact: true }).click()
  const completed = page.waitForResponse((res) => res.url().endsWith('/api/strategies/deviation_default'))
  release()
  const success = await completed
  expect(success.status()).toBe(200)
  await success.finished()
  await expect.poll(() => page.evaluate(() => (window as unknown as { __deviationScope: Harness }).__deviationScope.completed())).toBe(1)
  // Explicitly deliver the captured production Page onSuccess after 200 success;
  // observer reset alone must not be what makes the assertions below pass.
  await page.evaluate(() => (window as unknown as { __deviationScope: Harness }).__deviationScope.deliverQueued())
  await expect(card(page).getByRole('button', { name: '프리', exact: true })).toHaveAttribute('aria-pressed', 'true')
  await expect(card(page).getByLabel(labels[2], { exact: true })).toHaveValue('9')
  await expect(saveButton(page)).toBeEnabled()
  await saveButton(page).click()
  await expect.poll(() => updates.length).toBe(2)
  expect(updates[1]).toMatchObject({ expectedProfileId: 'toss-live', expectedBrokerAccountId: '1', params: { entry_mode: 'crossing', stop_loss_pct: 9, toss_us_session: 'pre' } })
})
