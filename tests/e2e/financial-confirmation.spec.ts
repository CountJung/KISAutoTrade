import { expect, test, type Page } from '@playwright/test'
import { isDeepStrictEqual } from 'node:util'
import { mockUx } from './fixtures/uxReviewFixture'

type ScopeSnapshot = {
  active_broker_id: 'kis' | 'toss'; active_profile_id: string; active_profile_name: string
  active_broker_account_id: string; kis_is_paper_trading: boolean
}
type WireRequest = { origin: string; method: string; path: string; query: string; body: unknown; hasBody: boolean }
type ExpectedMutation = WireRequest & { cacheScope: ScopeSnapshot }
const snapshot = (config: ScopeSnapshot): ScopeSnapshot => ({
  active_broker_id: config.active_broker_id, active_profile_id: config.active_profile_id,
  active_profile_name: config.active_profile_name, active_broker_account_id: config.active_broker_account_id,
  kis_is_paper_trading: config.kis_is_paper_trading,
})
const currentScope = (page: Page) => page.evaluate(() => {
  const config = (Reflect.get(window, '__ux07') as { current: () => ScopeSnapshot }).current()
  return config
}).then(snapshot)

// Source bridge retains the production router, QueryClient, StrictMode and UI.
// This is browser HTTP mock coverage, not Rust IPC or provider execution.
async function setup(page: Page, broker: 'kis' | 'toss' = 'kis', paper = false) {
  const audit = await mockUx(page, { activeBroker: broker,
    appConfigOverride: { kis_is_paper_trading: paper } })
  const origin = new URL(String(test.info().project.use.baseURL)).origin
  // cacheScope is browser observation metadata, never an identity in the HTTP body.
  const writes: ExpectedMutation[] = []
  const rejected: ExpectedMutation[] = []
  const control = { fail: false, hold: false, release: () => {},
    expected: [] as ExpectedMutation[], expectedRejected: [] as ExpectedMutation[] }

  const risk = { enabled: true, dailyLossLimit: 500000, maxPositionRatio: 0.2,
    maxDailyBuyOrdersPerSymbol: 0, maxDailySellOrdersPerSymbol: 1, maxConsecutiveLossesPerStrategySymbol: 3,
    volatilitySizingEnabled: false, riskPerTradeBps: 100, atrStopMultiplier: 2,
    currentLoss: 0, dailyProfit: 0, netLoss: 0, isEmergencyStop: false, canTrade: true,
    lossRatio: 0, blockedStrategySymbolCount: 0, atrReadySymbolCount: 0 }
  audit.responses.set('/api/risk-config', { json: risk })
  await page.route('**/src/main.tsx*', async route => {
    const response = await route.fetch(), body = await response.text()
    expect(body).toContain('queryClient')
    await route.fulfill({ response, body: body + `\nwindow.__ux07 = { navigate: async to => (await import('/src/router/index.ts')).router.navigate({ to }), change: (patch) => queryClient.setQueryData(['appConfig'], old => ({...old, ...patch})), fault: (kind) => { if (kind === 'publishing') { const m = queryClient.getMutationCache().build(queryClient, { mutationKey: ['account-profile-publication'], mutationFn: () => new Promise(() => {}) }); void m.execute(); } else { const q = queryClient.getQueryCache().find({queryKey:['appConfig']}); q.setState(kind === 'fetching' ? {fetchStatus:'fetching'} : kind === 'error' ? {status:'error'} : {isInvalidated:true}); } }, state: key => queryClient.getQueryData([key]), seed: (key, patch) => queryClient.setQueryData([key], old => ({...old, ...patch})), current: () => queryClient.getQueryData(['appConfig']) };\n` })
  })
  const mutationPaths = new Set(['/api/trading/start', '/api/trading/stop', '/api/activate-emergency',
    '/api/risk-config/clear-emergency', '/api/buy-suspension/clear', '/api/order', '/api/overseas-order'])
  await page.route('**/api/**', async route => {
    const req = route.request(), url = new URL(req.url()), path = url.pathname
    if (mutationPaths.has(path)) {
      let body: unknown = req.postData()
      try { body = req.postData() === null ? null : req.postDataJSON() } catch { /* malformed JSON is denied below */ }
      const observed = { origin: url.origin, method: req.method(), path, query: url.search,
        body, hasBody: req.postData() !== null, cacheScope: await currentScope(page) }
      const expected = control.expected[writes.length]
      if (!isDeepStrictEqual(observed, expected)) {
        rejected.push(observed)
        return route.fulfill({ status: 403, headers: { 'access-control-allow-origin': origin }, json: { code: 'UNEXPECTED_FINANCIAL_REQUEST', message: 'Synthetic contract denied request' } })
      }
      writes.push(observed)
      if (control.hold) await new Promise<void>(resolve => { control.release = resolve })
      if (control.fail) { control.fail = false; return route.fulfill({ status: 400, json: { code: 'SYNTHETIC_REJECT', message: '합성 요청 거절' } }) }
      const key = path.includes('emergency') ? 'riskConfig' : 'tradingStatus'
      const previous = await page.evaluate(key => (Reflect.get(window, '__ux07') as { state: (key: string) => object }).state(key), key)
      const result = path.includes('order') ? { odno: 'SYNTHETIC', ord_tmd: '120000', krx_fwdg_ord_orgno: 'MOCK' }
        : path.includes('emergency') ? { ...risk, ...previous, isEmergencyStop: path === '/api/activate-emergency' }
        : path === '/api/buy-suspension/clear' ? { ...previous, buySuspended: false, buySuspendedReason: null }
        : { ...previous, isRunning: path === '/api/trading/start' }
      // Keep subsequent mock reads consistent; this is not backend policy coverage.
      if (!path.includes('order')) audit.responses.set(key === 'riskConfig' ? '/api/risk-config' : '/api/trading/status', { json: result })
      return route.fulfill({ json: result })
    }
    expect(url.origin).toBe(origin)
    if (path === '/api/toss-open-orders') {
      expect(url.search).toBe('')
      expect(req.method()).toBe('POST'); expect(req.postDataJSON()).toEqual({ symbol: null })
      return route.fulfill({ json: [] })
    }
    if (path === '/api/toss-order-preflight') {
      expect(url.search).toBe('')
      expect(req.method()).toBe('POST')
      const input = req.postDataJSON()
      const expected = [
        { symbol: '005930', side: 'Buy', quantity: '2', price: '10000' },
        { symbol: '005930', side: 'Buy', quantity: '3', price: '10000' },
        { symbol: '005930', side: 'Sell', quantity: '2', price: '10000' },
        { symbol: 'AAPL', side: 'Buy', quantity: '2', price: '100.25' },
        { symbol: 'AAPL', side: 'Sell', quantity: '2', price: '100.25' },
      ]
      expect(expected).toContainEqual(input)
      const currency = input.symbol === 'AAPL' ? 'USD' : 'KRW'
      const money = { amount: String(Number(input.quantity) * Number(input.price)), currency }
      return route.fulfill({ json: { brokerId: 'toss', accountSeq: '1', symbol: input.symbol, market: input.symbol === 'AAPL' ? 'us' : 'kr', side: input.side.toLowerCase(), quantity: input.quantity, price: { amount: input.price, currency }, priceSource: 'input', buyingPower: money, sellableQuantity: '2', commissionRate: '0.00015', grossAmount: money, estimatedCommission: money, requiredCash: money, liquidityOk: true, safetyOk: true, orderAdapterSupported: true, canSubmit: true, blockedReasons: [], warnings: [] } })
    }
    if (['/api/price/005930', '/api/overseas-price/NAS/AAPL', '/api/toss-stock-safety/005930', '/api/toss-market-snapshot/005930', '/api/toss-stock-safety/AAPL', '/api/toss-market-snapshot/AAPL'].includes(path)) {
      expect(req.method()).toBe('GET')
      return route.fulfill({ json: path.includes('/price/') ? { stck_prpr: '10000', hts_kor_isnm: '합성 종목' }
        : path.includes('overseas-price') ? { last: '100.25', name: 'Synthetic AAPL' }
        : path.includes('safety') ? { brokerId: 'toss', symbol: path.split('/').pop(), warnings: [], hasBlockingWarning: false, buyBlocked: false }
        : { brokerId: 'toss', market: path.endsWith('AAPL') ? 'us' : 'kr', price: { amount: path.endsWith('AAPL') ? '100.25' : '10000', currency: path.endsWith('AAPL') ? 'USD' : 'KRW' }, orderbook: { asks: [], bids: [], currency: path.endsWith('AAPL') ? 'USD' : 'KRW' }, trades: [], priceLimits: {} } })
    }
    if (['/api/chart/005930', '/api/overseas-chart/NAS/AAPL', '/api/toss-chart/005930', '/api/toss-chart/AAPL'].includes(path)) {
      expect(req.method()).toBe('GET'); return route.fulfill({ json: [] })
    }
    return route.fallback()
  })
  await page.routeWebSocket('**/*', socket => {
    const url = new URL(socket.url())
    if (url.origin.replace(/^ws/, 'http') !== origin || url.pathname !== '/') audit.unknown.push('WS ' + socket.url())
    socket.close()
  })
  // Enforce exact origin/method even for the existing audited read fixtures.
  await page.route('**/*', route => {
    const req = route.request(), url = new URL(req.url())
    if ((!mutationPaths.has(url.pathname) && url.origin !== origin) || (req.method() !== 'GET' && !mutationPaths.has(url.pathname)
      && !['/api/toss-open-orders', '/api/toss-order-preflight'].includes(url.pathname))) {
      audit.unknown.push(req.method() + ' ' + url.href); return route.abort()
    }
    return route.fallback()
  })
  await page.goto('/trading')
  await expect(page.getByLabel('현재 broker scope').first()).toBeVisible()
  await expect.poll(() => page.evaluate(() => (Reflect.get(window, '__ux07') as { current: () => { active_profile_id?: string } | undefined }).current()?.active_profile_id)).toBeTruthy()
  const scope = await currentScope(page)
  expect(scope.active_broker_id).toBe(broker)
  expect(scope.kis_is_paper_trading).toBe(paper)
  expect(scope.active_profile_id).toBeTruthy(); expect(scope.active_broker_account_id).toBeTruthy()
  const contract = (path: string, body: unknown = null): ExpectedMutation => ({
    origin, method: 'POST', path, query: '', body, hasBody: body !== null, cacheScope: { ...scope },
  })
  return { audit, writes, rejected, control, scope, contract,
    // Declare the complete allowed sequence before any financial action.
    allow: (...requests: ExpectedMutation[]) => { expect(writes).toEqual([]); control.expected = requests },
    change: (patch: object) => page.evaluate(patch => (Reflect.get(window, '__ux07') as { change: (p: object) => void }).change(patch), patch),
  }
}
type State = Awaited<ReturnType<typeof setup>>
const states = new WeakMap<Page, State>()
test.afterEach(async ({ page }) => {
  const s = states.get(page); if (!s) return
  s.control.release()
  expect(s.writes).toEqual(s.control.expected)
  expect(s.rejected).toEqual(s.control.expectedRejected)
  expect(s.audit.writes).toEqual([]); expect(s.audit.external).toEqual([])
  expect(s.audit.unknown).toEqual([]); expect(s.audit.errors).toEqual([])
})
const final = (page: Page) => page.getByRole('button', { name: '최종 확인', exact: true })
const start = (page: Page) => page.locator('button[aria-label="자동매매 시작"]')
const seed = (page: Page, key: string, patch: object) => page.evaluate(({ key, patch }) =>
  (Reflect.get(window, '__ux07') as { seed: (k: string, p: object) => void }).seed(key, patch), { key, patch })
async function assertDialogScope(page: Page, s: State) {
  expect(await currentScope(page)).toEqual(s.scope)
  const dialog = page.getByRole('dialog')
  const indicator = dialog.getByLabel('현재 broker scope')
  await expect(indicator.getByText(s.scope.active_broker_id === 'kis' ? 'KIS' : 'Toss', { exact: true })).toBeVisible()
  await expect(indicator.getByText(s.scope.active_profile_name, { exact: true })).toBeVisible()
  await expect(indicator.getByText(s.scope.active_broker_account_id, { exact: true })).toBeVisible()
  await expect(dialog).toContainText(s.scope.active_broker_id === 'kis' && s.scope.kis_is_paper_trading ? '모의투자' : '실전투자')
  // The profile ID is compared in cacheScope, the UI displays its name.
}
async function focusedOpener(page: Page) {
  const opener = start(page)
  await expect(opener).toHaveCount(1)
  await expect(opener).toBeVisible(); await expect(opener).toBeEnabled()
  await opener.focus(); await expect(opener).toBeFocused()
  return opener
}
const cacheState = (page: Page, key: string) => page.evaluate(key =>
  (Reflect.get(window, '__ux07') as { state: (key: string) => Record<string, unknown> }).state(key), key)
async function draft(page: Page) {
  await page.getByRole('textbox', { name: '6자리 종목코드 (예: 005930, 0005A0)' }).fill('005930')
  await page.getByRole('textbox', { name: '주문가격', exact: true }).fill('10000')
  await page.getByRole('textbox', { name: '주문수량', exact: true }).fill('2')
}

test('P2-UX-07 same-tick paper handler cannot bypass newly published live confirmation', async ({ page }) => {
  const s = await setup(page, 'kis', true); states.set(page, s)
  s.allow()
  const attempted: string[] = []
  // Observe attempts without bypassing the deny-by-default contract handler.
  page.on('request', request => { if (new URL(request.url()).pathname === '/api/trading/start') attempted.push(request.method()) })
  await page.evaluate(() => {
    const bridge = Reflect.get(window, '__ux07') as { change: (patch: object) => void }
    bridge.change({ kis_is_paper_trading: false })
    const button = document.querySelector<HTMLButtonElement>('button[aria-label="자동매매 시작"]')
    if (!button) throw new Error('Missing existing paper-rendered start handler')
    button.click() // same task, before React receives QueryClient's notification
  })
  await assertDialogScope(page, { ...s, scope: { ...s.scope, kis_is_paper_trading: false } })
  await expect(final(page)).toBeEnabled()
  expect(attempted).toEqual([])
  await page.keyboard.press('Escape')
  expect(attempted).toEqual([])
})

test('P2-UX-07 browser navigation ABA permanently invalidates sidebar approval', async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow()
  const navigate = (to: string) => page.evaluate(to =>
    (Reflect.get(window, '__ux07') as { navigate: (to: string) => Promise<void> }).navigate(to), to)
  await navigate('/log')
  await navigate('/trading')
  await start(page).click()
  await expect(final(page)).toBeEnabled()
  await page.goBack()
  await expect(page).toHaveURL(/\/log$/)
  await expect(final(page)).toBeDisabled()
  await page.goForward()
  await expect(page).toHaveURL(/\/trading$/)
  await expect(final(page)).toBeDisabled()
  await page.keyboard.press('Escape')
  expect(s.writes).toEqual([])
})

test('P2-UX-07 Escape cancels live start and restores the same Sidebar opener', async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow()
  const opener = await focusedOpener(page)
  const element = await opener.elementHandle()
  await opener.press('Space')
  await assertDialogScope(page, s)
  await expect(page.getByRole('button', { name: '취소', exact: true })).toBeFocused()
  await page.keyboard.press('Escape')
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await expect(opener).toBeEnabled(); await expect(opener).toBeFocused()
  expect(await element!.evaluate(button => button === document.activeElement)).toBe(true)
  expect(s.writes).toHaveLength(0)
})

test('P2-UX-07 synthetic IME 229 repeat modifiers prevent default without approval', async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow()
  await start(page).click(); await assertDialogScope(page, s)
  const prevented = await final(page).evaluate(button => [
    { key: 'Enter', isComposing: true }, { key: 'Enter', keyCode: 229 },
    { key: 'Enter', repeat: true }, { key: ' ', ctrlKey: true },
    { key: 'Enter', metaKey: true }, { key: ' ', altKey: true }, { key: 'Enter', shiftKey: true },
  ].map(init => { const event = new KeyboardEvent('keydown', { ...init, bubbles: true, cancelable: true }); button.dispatchEvent(event); return event.defaultPrevented }))
  expect(prevented).toEqual(Array(7).fill(true))
  expect(s.writes).toHaveLength(0)
  await page.keyboard.press('Escape')
})

test('P2-UX-07 browser modifier and held repeat keys never approve live start', async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow()
  await start(page).press('Enter'); await assertDialogScope(page, s)
  await final(page).focus(); await expect(final(page)).toBeFocused()
  for (const key of ['Control+Enter', 'Meta+Enter', 'Alt+Enter', 'Shift+Space']) {
    await page.keyboard.press(key)
    expect(s.writes).toHaveLength(0)
    await expect(page.getByRole('dialog')).toBeVisible()
  }
  // First keydown is modified; subsequent real Playwright keydown has repeat=true.
  await page.keyboard.down('Control'); await page.keyboard.down('Enter')
  await page.keyboard.up('Control'); await page.keyboard.down('Enter'); await page.keyboard.up('Enter')
  expect(s.writes).toHaveLength(0)
  await page.keyboard.press('Escape')
})

test('P2-UX-07 input Enter and same-scope publication never approve live start', async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow()
  await page.getByRole('textbox', { name: '주문수량', exact: true }).press('Enter')
  await page.keyboard.press('Control+Enter'); expect(s.writes).toHaveLength(0)
  await start(page).click(); await assertDialogScope(page, s)
  await s.change({}) // same scope, new public generation
  await expect(final(page)).toBeDisabled()
  await page.getByRole('button', { name: '취소', exact: true }).click()
  expect(s.writes).toHaveLength(0)
})

for (const kind of ['fetching', 'error', 'invalidated', 'publishing']) test(`P2-UX-07 untrusted config ${kind} invalidates approval`, async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow()
  await start(page).click()
  await page.evaluate(kind => (Reflect.get(window, '__ux07') as { fault: (kind: string) => void }).fault(kind), kind)
  await expect(final(page)).toBeDisabled()
  await page.keyboard.press('Escape'); expect(s.writes).toHaveLength(0)
})

test('P2-UX-07 trusted backdrop hit cancels without mutation and restores trigger focus', async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow()
  const opener = await focusedOpener(page)
  const element = await opener.elementHandle()
  await opener.click()
  await assertDialogScope(page, s)
  await expect(page.getByRole('button', { name: '취소', exact: true })).toBeFocused()
  const point = await page.locator('.MuiDialog-container').evaluate(container => {
    const box = container.getBoundingClientRect(), point = { x: box.left + 2, y: box.top + 2 }
    if (document.elementFromPoint(point.x, point.y) !== container) throw new Error('Backdrop target is not the dialog container')
    return point
  })
  await page.mouse.click(point.x, point.y)
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await expect(opener).toBeEnabled(); await expect(opener).toBeFocused()
  expect(await element!.evaluate(button => button === document.activeElement)).toBe(true)
  expect(s.writes).toHaveLength(0)
})

test('P2-UX-07 Toss US sell market confirms unpriced payload with session', async ({ page }) => {
  const s = await setup(page, 'toss'); states.set(page, s)
  s.allow(s.contract('/api/order', { symbol: 'AAPL', side: 'Sell', order_type: 'Market', quantity: 2, price: 0, toss_session: 'auto' }))
  await page.getByRole('button', { name: '시장가', exact: true }).click()
  await page.getByRole('button', { name: '미국', exact: true }).click()
  const ticker = page.getByRole('textbox', { name: '티커 (AAPL, TSLA …)' })
  await ticker.fill('AAPL'); await ticker.press('Enter')
  await page.getByRole('textbox', { name: '주문가격 (USD)' }).fill('100.25')
  await page.getByRole('textbox', { name: '주문수량', exact: true }).fill('2')
  await page.getByRole('button', { name: '매도', exact: true }).click()
  await page.getByRole('button', { name: '매도 주문 (USD 시장가)', exact: true }).click()
  await expect(page.getByRole('dialog')).toContainText('매도')
  await expect(page.getByRole('dialog')).toContainText('가격 미지정 (시장가)')
  await expect(page.getByRole('dialog')).toContainText('세션 자동')
  await assertDialogScope(page, s)
  expect(s.writes).toHaveLength(0)
  await final(page).click()
  await expect.poll(() => s.writes.map(w => w.path)).toEqual(['/api/order'])
  await expect(page.getByRole('dialog')).toHaveCount(0)
})

test('P2-UX-07 live start final exactly once pending and explicit retry', async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow(s.contract('/api/trading/start'), s.contract('/api/trading/start'))
  s.control.hold = true; s.control.fail = true
  await start(page).click(); await assertDialogScope(page, s)
  expect(s.writes).toHaveLength(0)
  await final(page).focus(); await page.keyboard.press('Enter')
  await expect.poll(() => s.writes.length).toBe(1)
  await page.keyboard.down('Enter'); await page.keyboard.down('Enter'); await page.keyboard.up('Enter')
  expect(s.writes).toHaveLength(1); await expect(final(page)).toBeDisabled()
  s.control.release(); await expect(page.getByRole('dialog')).toContainText('합성 요청 거절')
  await page.getByRole('button', { name: '닫기', exact: true }).click()
  s.control.hold = false
  await start(page).click(); await assertDialogScope(page, s)
  expect(s.writes).toHaveLength(1)
  await final(page).click()
  await expect.poll(() => s.writes.length).toBe(2)
  await expect(page.getByRole('dialog')).toHaveCount(0)
})

for (const broker of ['kis', 'toss'] as const) test(`P2-UX-07 ${broker} manual order summary invalidation cancel and final`, async ({ page }) => {
  const s = await setup(page, broker, broker === 'kis'); states.set(page, s)
  s.allow(s.contract('/api/order', { symbol: '005930', side: 'Buy', order_type: 'Limit', quantity: 2, price: 10000,
    ...(broker === 'toss' ? { toss_session: null } : {}) }))
  await draft(page)
  const submit = page.getByRole('button', { name: '매수 주문', exact: true })
  await submit.click()
  await expect(page.getByRole('dialog')).toContainText('005930')
  await expect(page.getByRole('dialog')).toContainText('10,000')
  await page.getByRole('button', { name: '취소', exact: true }).click()
  expect(s.writes).toHaveLength(0)
  await submit.click()
  await page.getByRole('textbox', { name: '주문수량', exact: true, includeHidden: true }).fill('3')
  await expect(final(page)).toBeDisabled()
  await page.getByRole('button', { name: '취소', exact: true }).click()
  await page.getByRole('textbox', { name: '주문수량', exact: true }).fill('2')
  await submit.click(); await assertDialogScope(page, s)
  expect(s.writes).toHaveLength(0)
  await final(page).press('Space')
  await expect.poll(() => s.writes.length).toBe(1)
  await expect(page.getByRole('dialog')).toHaveCount(0)
})

for (const patch of [{ active_broker_id: 'toss' }, { active_broker_account_id: 'SYNTHETIC-B' },
  { active_profile_id: 'synthetic-profile-b' }, { kis_is_paper_trading: true }]) {
  test(`P2-UX-07 scope ABA invalidates ${Object.keys(patch)[0]}`, async ({ page }) => {
    const s = await setup(page); states.set(page, s)
  s.allow()
    const original = await page.evaluate(() => (Reflect.get(window, '__ux07') as { current: () => object }).current())
    await start(page).click()
    await s.change(patch); await s.change(original)
    await expect(final(page)).toBeDisabled()
    await page.keyboard.press('Escape'); expect(s.writes).toHaveLength(0)
    await start(page).click(); await expect(final(page)).toBeEnabled()
    await page.getByRole('button', { name: '취소', exact: true }).click()
  })
}

test('P2-UX-07 two start entries and final double activation share one pending lock; stop remains immediate', async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  await page.getByRole('button', { name: 'Dashboard', exact: true }).click()
  s.allow(s.contract('/api/trading/start'), s.contract('/api/trading/stop'))
  const entries = page.getByRole('button', { name: '자동매매 시작', exact: true })
  await expect(entries).toHaveCount(2)
  await entries.evaluateAll(buttons => buttons.forEach(button => (button as HTMLButtonElement).click()))
  await expect(page.getByRole('dialog')).toHaveCount(1)
  await assertDialogScope(page, s); expect(s.writes).toHaveLength(0)
  s.control.hold = true
  await final(page).evaluate(button => { (button as HTMLButtonElement).click(); (button as HTMLButtonElement).click() })
  await expect.poll(() => s.writes.length).toBe(1)
  await expect(final(page)).toBeDisabled()
  await expect(page.getByRole('button', { name: '자동매매 즉시 정지', exact: true })).toBeEnabled()
  s.control.release(); s.control.hold = false
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await page.getByRole('button', { name: '자동매매 정지', exact: true }).first().click()
  await expect.poll(() => s.writes.map(w => w.path)).toEqual(['/api/trading/start', '/api/trading/stop'])
  await expect(page.getByRole('dialog')).toHaveCount(0)
})

for (const action of ['emergency', 'buy'] as const) test(`P2-UX-07 ${action} release requires a new confirmation and preserves meaning`, async ({ page }) => {
  const s = await setup(page); states.set(page, s)
  s.allow(s.contract(action === 'emergency' ? '/api/risk-config/clear-emergency' : '/api/buy-suspension/clear'))
  await page.getByRole('button', { name: 'Dashboard', exact: true }).click()
  await expect(page.getByText('리스크 관리', { exact: true })).toBeVisible()
  await seed(page, 'riskConfig', { isEmergencyStop: true, currentLoss: 700000, dailyProfit: 100000,
    netLoss: 600000, lossRatio: 1.2, canTrade: false, blockedStrategySymbolCount: 2 })
  await seed(page, 'tradingStatus', { isRunning: true, buySuspended: true, buySuspendedReason: '합성 잔고 부족' })
  const beforeRisk = await cacheState(page, 'riskConfig'), beforeTrading = await cacheState(page, 'tradingStatus')
  s.audit.responses.set('/api/risk-config', { json: beforeRisk })
  s.audit.responses.set('/api/trading/status', { json: beforeTrading })
  const button = page.getByRole('button', { name: action === 'emergency' ? '비상정지 해제' : '매수 재개', exact: true })
  const meaning = action === 'emergency'
    ? '비상정지 flag를 해제합니다. 자동매매 시작이나 손실 카운터 초기화가 아닙니다. 손실 한도·매수 정지와 기존 guard는 유지됩니다.'
    : '매수 정지와 사유를 해제합니다. 입금이나 잔고 검증이 아니며 비상정지를 해제하거나 자동매매를 시작하지 않습니다. 기존 guard는 유지됩니다.'
  await button.click(); await assertDialogScope(page, s)
  await expect(page.getByRole('dialog')).toContainText(meaning)
  expect(s.writes).toHaveLength(0)
  await page.keyboard.press('Escape')
  await expect(page.getByRole('dialog')).toHaveCount(0)
  expect(s.writes).toHaveLength(0)
  expect(await cacheState(page, 'riskConfig')).toEqual(beforeRisk)
  expect(await cacheState(page, 'tradingStatus')).toEqual(beforeTrading)
  await button.press('Enter'); await assertDialogScope(page, s)
  await expect(page.getByRole('dialog')).toContainText(meaning)
  expect(s.writes).toHaveLength(0)
  await final(page).press('Space')
  await expect(page.getByRole('dialog')).toHaveCount(0)
  expect(s.writes).toEqual(s.control.expected)
  await expect.poll(() => cacheState(page, 'riskConfig')).toEqual(action === 'emergency'
    ? { ...beforeRisk, isEmergencyStop: false } : beforeRisk)
  await expect.poll(() => cacheState(page, 'tradingStatus')).toEqual(action === 'buy'
    ? { ...beforeTrading, buySuspended: false, buySuspendedReason: null } : beforeTrading)
  // This asserts frontend state with a preserving HTTP fake, not real risk policy.
})

for (const kind of ['account', 'path', 'method', 'body', 'query', 'origin'] as const) {
  test(`P2-UX-07 contract rejects wrong ${kind} with HTTP 403 and records exact attempt`, async ({ page }) => {
    const s = await setup(page); states.set(page, s)
    const valid = s.contract('/api/trading/start')
    const invalid = { ...valid, cacheScope: { ...valid.cacheScope } }
    if (kind === 'account') invalid.cacheScope.active_broker_account_id = 'SYNTHETIC-WRONG-ACCOUNT'
    if (kind === 'path') invalid.path = '/api/trading/stop' // also bodyless; cannot substitute for start
    if (kind === 'method') invalid.method = 'PUT'
    if (kind === 'body') { invalid.body = { unexpected: true }; invalid.hasBody = true }
    if (kind === 'query') invalid.query = '?unexpected=1'
    if (kind === 'origin') invalid.origin = 'http://127.0.0.1:1439'
    s.allow(valid); s.control.expectedRejected = [invalid]
    if (kind === 'account') await s.change(invalid.cacheScope)
    const response = await page.evaluate(async input => {
      const response = await fetch(input.origin + input.path + input.query, {
        method: input.method, ...(input.hasBody ? { body: JSON.stringify(input.body) } : {}),
      })
      return { status: response.status, body: await response.json() }
    }, invalid)
    expect(response).toEqual({ status: 403, body: { code: 'UNEXPECTED_FINANCIAL_REQUEST', message: 'Synthetic contract denied request' } })
    expect(s.rejected).toEqual([invalid]); expect(s.writes).toHaveLength(0)
    if (kind === 'account') await s.change(s.scope)
    await start(page).click(); await assertDialogScope(page, s)
    expect(s.writes).toHaveLength(0)
    await final(page).click()
    await expect(page.getByRole('dialog')).toHaveCount(0)
    expect(s.writes).toEqual([valid])
  })
}

for (const market of ['KIS US', 'Toss domestic'] as const) {
  test(`P2-UX-07 ${market} sell summary cancel zero and fresh confirmation exact payload one`, async ({ page }) => {
    const kis = market === 'KIS US'
    const s = await setup(page, kis ? 'kis' : 'toss'); states.set(page, s)
    s.allow(kis ? s.contract('/api/overseas-order', { symbol: 'AAPL', exchange: 'NASD', side: 'Sell', quantity: 2, price: 100.25 })
      : s.contract('/api/order', { symbol: '005930', side: 'Sell', order_type: 'Limit', quantity: 2, price: 10000, toss_session: null }))
    if (kis) {
      await page.getByRole('button', { name: '미국', exact: true }).click()
      const ticker = page.getByRole('textbox', { name: '티커 (AAPL, TSLA …)' })
      await ticker.fill('AAPL'); await ticker.press('Enter')
      await page.getByRole('textbox', { name: '주문가격 (USD)' }).fill('100.25')
      await page.getByRole('textbox', { name: '주문수량', exact: true }).fill('2')
    } else await draft(page)
    await page.getByRole('button', { name: '매도', exact: true }).click()
    const submit = page.getByRole('button', { name: kis ? '매도 주문 (USD 지정가)' : '매도 주문', exact: true })
    const summary = async () => {
      await assertDialogScope(page, s)
      for (const text of [kis ? 'AAPL' : '005930', '매도', '수량 2주', '지정가', kis ? '100.25 USD' : '10,000 KRW']) {
        await expect(page.getByRole('dialog')).toContainText(text)
      }
      if (kis) await expect(page.getByRole('dialog')).toContainText('거래소 NASD')
      else await expect(page.getByRole('dialog')).not.toContainText('세션 ')
    }
    await submit.click(); await summary()
    expect(s.writes).toHaveLength(0)
    await page.getByRole('button', { name: '취소', exact: true }).click()
    await expect(page.getByRole('dialog')).toHaveCount(0)
    expect(s.writes).toHaveLength(0)
    await submit.click(); await summary()
    expect(s.writes).toHaveLength(0)
    await final(page).click()
    await expect(page.getByRole('dialog')).toHaveCount(0)
    expect(s.writes).toEqual(s.control.expected)
  })
}

test('P2-UX-07 KIS US uses confirmed NASD USD limit payload', async ({ page }) => {
  const s = await setup(page, 'kis', true); states.set(page, s)
  s.allow(s.contract('/api/overseas-order', { symbol: 'AAPL', exchange: 'NASD', side: 'Buy', quantity: 2, price: 100.25 }))
  await page.getByRole('button', { name: '미국', exact: true }).click()
  const ticker = page.getByRole('textbox', { name: '티커 (AAPL, TSLA …)' })
  await ticker.fill('AAPL'); await ticker.press('Enter')
  await page.getByRole('textbox', { name: '주문가격 (USD)' }).fill('100.25')
  await page.getByRole('textbox', { name: '주문수량', exact: true }).fill('2')
  await page.getByRole('button', { name: '매수 주문 (USD 지정가)', exact: true }).click()
  await expect(page.getByRole('dialog')).toContainText('100.25 USD')
  await expect(page.getByRole('dialog')).toContainText('NASD')
  await assertDialogScope(page, s); expect(s.writes).toHaveLength(0)
  await final(page).click()
  await expect.poll(() => s.writes.map(w => w.path)).toEqual(['/api/overseas-order'])
  await expect(page.getByRole('dialog')).toHaveCount(0)
})

test('P2-UX-07 375px dark dialog reflows and emergency stop is immediate inside confirmation', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.addInitScript(() => localStorage.setItem('act-settings', JSON.stringify({ state: { theme: 'dark' }, version: 0 })))
  const s = await setup(page); states.set(page, s)
  s.allow(s.contract('/api/activate-emergency'))
  await draft(page)
  await page.getByRole('button', { name: '매수 주문', exact: true }).click()
  const dialog = page.getByRole('dialog')
  const box = await dialog.boundingBox()
  expect(box).not.toBeNull(); expect(box!.x).toBeGreaterThanOrEqual(0); expect(box!.x + box!.width).toBeLessThanOrEqual(375)
  const visual = await dialog.evaluate(element => {
    const style = getComputedStyle(element)
    const luminance = (color: string) => {
      const channels = (color.match(/[\d.]+/g) ?? []).slice(0, 3).map(Number).map(value => {
        const srgb = value / 255
        return srgb <= 0.04045 ? srgb / 12.92 : ((srgb + 0.055) / 1.055) ** 2.4
      })
      return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722
    }
    const foreground = luminance(style.color), background = luminance(style.backgroundColor)
    return { overflow: element.scrollWidth > element.clientWidth,
      background, contrast: (Math.max(foreground, background) + 0.05) / (Math.min(foreground, background) + 0.05) }
  })
  expect(visual.overflow).toBe(false); expect(visual.background).toBeLessThan(0.2); expect(visual.contrast).toBeGreaterThanOrEqual(4.5)
  await expect(dialog).toContainText('접수 주문 취소나 포지션 청산이 아닙니다')
  await assertDialogScope(page, s); expect(s.writes).toHaveLength(0)
  await page.getByRole('button', { name: '비상정지 즉시 발동', exact: true }).click()
  await expect.poll(() => s.writes.map(w => w.path)).toEqual(['/api/activate-emergency'])
  await expect(final(page)).toBeDisabled()
  await page.keyboard.press('Escape')
})
