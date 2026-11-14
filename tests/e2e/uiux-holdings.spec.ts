import { expect, test, type Page } from '@playwright/test'
import type { AppConfigView, BalanceResult, OverseasBalanceResult } from '../../src/api/types'
import { mockUx } from './fixtures/uxReviewFixture'
import { fakeProfiles } from './fixtures/keyboardProfileFixture'

// Synthetic holdings only: every API response stays in the browser route fixture.
const domestic: BalanceResult = {
  items: [{ pdno: '0005A0', prdt_name: '합성 국내 보유', hldg_qty: '7', pchs_avg_pric: '10000', prpr: '11000', evlu_pfls_amt: '7000', evlu_pfls_rt: '10' }],
  summary: null,
}
const overseas: OverseasBalanceResult = {
  items: [{ ovrs_pdno: 'SYNTH', ovrs_item_name: 'Synthetic US holding', ovrs_cblc_qty: '4', pchs_avg_pric: '100', now_pric2: '102', ovrs_stck_evlu_amt: '408', frcr_evlu_pfls_amt: '8', evlu_pfls_rt: '2', ovrs_excg_cd: 'NYSE', tr_mket_name: 'Synthetic NYSE' }],
  summary: null,
}
const markets = [
  { name: 'domestic', label: '국내 잔고', path: '/api/balance', other: '/api/overseas-balance', otherLabel: '해외 잔고', data: domestic, holding: '합성 국내 보유', quantity: '7주', empty: '보유 종목 없음' },
  { name: 'overseas', label: '해외 잔고', path: '/api/overseas-balance', other: '/api/balance', otherLabel: '국내 잔고', data: overseas, holding: 'Synthetic US holding', quantity: '4', empty: '해외 보유 종목 없음' },
] as const
const section = (page: Page, market: typeof markets[number]) => page.getByTestId(`kis-${market.name}-holdings`)
const retry = (page: Page, market: typeof markets[number]) => section(page, market).getByRole('button', { name: `${market.label} 다시 확인` })
const failure = { status: 503, json: { message: 'Synthetic holdings unavailable' } }
const empty = { items: [], summary: null }

function assertReadOnly(audit: Awaited<ReturnType<typeof mockUx>>) {
  expect(audit.writes).toEqual([])
  expect(audit.external).toEqual([])
  expect(audit.errors).toEqual([])
  expect(audit.unknown).toEqual([])
}

async function scopeSettings(page: Page) {
  const profiles = structuredClone(fakeProfiles.slice(0, 3))
  // Same account, different mode: the editor must not carry its paper draft into live.
  profiles[1].broker_account_id = profiles[0].broker_account_id
  profiles[1].account_no = profiles[0].account_no
  profiles[1].is_paper_trading = false
  const origin = new URL(String(test.info().project.use.baseURL)).origin
  const rejected: string[] = []
  const saves: unknown[] = []
  let allocated = 1000, budgetFails = false, holdBudget = false
  const budgetReleases: (() => void)[] = []
  let active = 0
  let configFails = false
  const config = (): AppConfigView => ({ active_profile_id: profiles[active].id, active_profile_name: profiles[active].name,
    active_broker_id: profiles[active].broker_id, active_broker_account_id: profiles[active].broker_account_id,
    kis_is_paper_trading: profiles[active].is_paper_trading, kis_app_key_masked: 'SYNTHETIC-***', kis_account_no: '',
    kis_configured: true, active_broker_configured: true, discord_enabled: false, notification_levels: [] })
  const audit = await mockUx(page, { appConfigOverride: config() })
  audit.responses.set('/api/app-config', route => route.fulfill(configFails
    ? { status: 503, json: { error: 'Synthetic config unavailable' } } : { json: config() }))
  audit.responses.set('/api/profiles', route => route.fulfill({ json: profiles.map((p, i) => ({ ...p, is_active: i === active })) }))
  const currency = { allocatedAmount: 1000, cashAmount: 1000, reservedAmount: 0, availableAmount: 1000,
    ownedPositionCount: 0, blockedReason: null, feeBufferBps: 100 }
  audit.responses.set('/api/auto-trading-budget', async route => {
    const url = new URL(route.request().url()), selected = profiles[active]
    if (url.origin !== origin || route.request().method() !== 'GET' || url.searchParams.get('brokerId') !== selected.broker_id || url.searchParams.get('brokerAccountId') !== selected.broker_account_id) {
      rejected.push('budget read scope/method'); return route.fulfill({ status: 400, json: { message: 'Synthetic budget scope denied' } })
    }
    const response = budgetFails ? { status: 503, json: { message: '합성 저장 후 재조회 실패' } } : active === 1
      ? { status: 400, json: { message: '합성 예산 투자 모드 불일치' } }
      : { json: { scope: { brokerId: selected.broker_id, accountId: selected.broker_account_id }, krw: { ...currency, allocatedAmount: allocated, cashAmount: allocated, availableAmount: allocated }, usd: currency } }
    if (holdBudget) await new Promise<void>(resolve => budgetReleases.push(resolve))
    return route.fulfill(response)
  })
  await page.route('**/api/auto-trading-budget', async route => {
    if (route.request().method() === 'GET') return route.fallback()
    const input = route.request().postDataJSON(), selected = profiles[active]
    if (new URL(route.request().url()).origin !== origin || route.request().method() !== 'POST' ||
      input.brokerId !== selected.broker_id || input.brokerAccountId !== selected.broker_account_id ||
      input.expectedProfileId !== selected.id || input.expectedIsPaper !== selected.is_paper_trading) {
      rejected.push('budget write scope/method'); return route.fulfill({ status: 400, json: { message: 'Synthetic budget scope denied' } })
    }
    saves.push(input); allocated = input.krwAmount
    return route.fulfill({ json: { scope: { brokerId: selected.broker_id, accountId: selected.broker_account_id }, krw: { ...currency, allocatedAmount: allocated }, usd: currency } })
  })
  const selections: string[] = []
  let hold = false, reject = false
  let release!: () => void
  await page.route('**/api/profiles/*/set-active', async route => {
    const request = route.request(), url = new URL(request.url())
    if (url.origin !== origin || request.method() !== 'POST') { rejected.push('selection origin/method'); return route.abort() }
    const id = url.pathname.split('/')[3]
    const next = profiles.findIndex(p => p.id === id)
    if (next < 0) throw new Error('Unknown synthetic selection')
    selections.push(id)
    if (hold) await new Promise<void>(resolve => { release = resolve })
    if (reject) return route.fulfill({ status: 409, json: { message: '합성 계좌 전환 거부' } })
    active = next
    await route.fulfill({ json: config() })
  })
  await page.route('**/*', route => {
    const request = route.request(), url = new URL(request.url())
    const staticPath = ['/', '/settings', '/@vite/client', '/@react-refresh', '/favicon.ico'].includes(url.pathname) || ['/src/', '/node_modules/', '/@fs/', '/assets/'].some(prefix => url.pathname.startsWith(prefix))
    if (url.origin !== origin || (!url.pathname.startsWith('/api/') && (request.method() !== 'GET' || !staticPath))) {
      rejected.push('network ' + request.method() + ' ' + url.href); return route.abort()
    }
    return route.fallback()
  })
  await page.routeWebSocket('**/*', socket => {
    const url = new URL(socket.url())
    if (url.origin.replace(/^ws/, 'http') !== origin || url.pathname !== '/') rejected.push('WS ' + socket.url())
    socket.close()
  })
  return { audit, profiles, selections, saves, rejected,
    holdBudget: () => { holdBudget = true }, budgetWaiters: () => budgetReleases.length,
    releaseBudget: () => { holdBudget = false; budgetReleases.splice(0).forEach(release => release()) },
    failBudget: (value: boolean) => { budgetFails = value },
    hold: () => { hold = true }, release: () => { hold = false; release() },
    reject: (value: boolean) => { reject = value }, configFails: (value: boolean) => { configFails = value } }
}

test('scope Settings blocks pending selection, exposes unknown failure and restores mode-specific budget drafts', async ({ page }) => {
  const state = await scopeSettings(page)
  const summary = page.getByRole('region', { name: '활성 증권사 프로파일', exact: true })
  const choose = (i: number) => page.getByRole('button', { name: `${state.profiles[i].name} 선택`, exact: true })
  const budget = page.getByRole('region', { name: '자동매매 전용 예산', exact: true })
  const draft = budget.getByRole('textbox', { name: '자동매매 원화 배정액 (원)', exact: true })
  await page.goto('/settings')
  await expect(summary).toContainText(`활성: ${state.profiles[0].name}`)
  await expect(summary).toContainText('모의투자 모드')
  await expect(choose(1)).toBeEnabled()
  await draft.fill('1111')
  state.hold(); state.reject(true); state.configFails(true)
  await choose(1).click()
  await expect.poll(() => state.selections.length).toBe(1)
  await expect(choose(1)).toBeDisabled(); await expect(choose(2)).toBeDisabled()
  await expect(summary).toContainText('계좌 선택 상태를 확인하는 중')
  await expect(draft).toHaveCount(0)
  state.release()
  await expect(summary).toContainText('계좌 선택 상태 확인 실패', { timeout: 12000 })
  await expect(summary).toContainText('합성 계좌 전환 거부')
  await expect(summary).not.toContainText(`활성: ${state.profiles[0].name}`)
  await expect(draft).toHaveCount(0)
  state.configFails(false); state.reject(false); state.holdBudget()
  await page.getByRole('button', { name: '설정 재점검', exact: true }).click()
  await expect.poll(() => state.budgetWaiters()).toBeGreaterThan(0)
  await expect(draft).toHaveCount(0)
  state.releaseBudget()
  await expect(draft).toHaveValue('1000')
  await draft.fill('1111')
  await choose(1).click()
  await expect(summary).toContainText('실전투자 모드')
  await expect(summary).toContainText(`계좌: ${state.profiles[1].broker_account_id}`)
  await expect(budget).toContainText('합성 예산 투자 모드 불일치', { timeout: 12000 })
  await expect(draft).toHaveCount(0)
  await choose(0).click()
  await expect(summary).toContainText('모의투자 모드')
  await expect(draft).toHaveValue('1000')
  await choose(2).click()
  await expect(summary).toContainText('Toss 실거래 모드')
  await expect(summary).toContainText(`계좌: ${state.profiles[2].broker_account_id}`)
  expect(state.selections).toEqual([state.profiles[1].id, state.profiles[1].id, state.profiles[0].id, state.profiles[2].id])
  expect(state.rejected).toEqual([])
  expect(state.saves).toHaveLength(0)
  assertReadOnly(state.audit) // Only the explicitly mocked profile selections above are allowed writes.
})

for (const fails of [false, true]) {
  test(`scope Settings budget save waits for same-scope GET and retains draft on failure=${fails}`, async ({ page }) => {
    const state = await scopeSettings(page)
    await page.goto('/settings')
    const draft = page.getByRole('textbox', { name: '자동매매 원화 배정액 (원)', exact: true })
    const save = page.getByRole('button', { name: '자동매매 예산 저장', exact: true })
    await expect(draft).toHaveValue('1000')
    await draft.fill('2345')
    state.holdBudget(); state.failBudget(fails)
    await save.click()
    await expect.poll(() => state.saves.length).toBe(1)
    await expect.poll(() => state.budgetWaiters()).toBeGreaterThan(0)
    await expect(page.getByText('자동매매 예산을 저장했습니다.', { exact: true })).toHaveCount(0)
    await expect(draft).toHaveValue('2345')
    await expect(page.getByRole('button', { name: '저장 중…', exact: true })).toBeDisabled()
    state.releaseBudget()
    if (fails) {
      await expect(page.getByText(/예산은 저장됐지만 재조회에 실패/)).toBeVisible({ timeout: 15000 })
      await expect(draft).toHaveValue('2345')
      await expect(save).toBeEnabled()
    } else {
      await expect(page.getByText('자동매매 예산을 저장했습니다.', { exact: true })).toBeVisible()
      await expect(draft).toHaveValue('2345')
      await expect(save).toBeDisabled()
    }
    expect(state.saves).toHaveLength(1)
    expect(state.rejected).toEqual([])
    assertReadOnly(state.audit)
  })
}

test('scope Settings budget fixture negative control rejects wrong account and method', async ({ page }) => {
  const state = await scopeSettings(page)
  await page.goto('/settings')
  await expect(page.getByRole('textbox', { name: '자동매매 원화 배정액 (원)', exact: true })).toBeVisible()
  const statuses = await page.evaluate(async () => Promise.all([
    fetch('/api/auto-trading-budget?brokerId=kis&brokerAccountId=WRONG').then(r => r.status),
    fetch('/api/auto-trading-budget', { method: 'PUT', headers: { 'content-type': 'application/json' }, body: '{}' }).then(r => r.status),
    fetch('/api/auto-trading-budget', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ brokerId: 'kis', brokerAccountId: 'WRONG', expectedProfileId: 'WRONG', expectedIsPaper: false }) }).then(r => r.status),
  ]))
  expect(statuses).toEqual([400, 400, 400])
  expect(state.rejected).toHaveLength(3)
  expect(state.saves).toHaveLength(0)
  assertReadOnly(state.audit)
})

test('scope Settings disables selection while trading state is unknown or running without claiming unknown is running', async ({ page }) => {
  const state = await scopeSettings(page)
  let release!: () => void
  const gate = new Promise<void>(resolve => { release = resolve })
  state.audit.responses.set('/api/trading/status', async route => {
    await gate
    await route.fulfill({ json: { isRunning: true, activeStrategies: [], positionCount: 0, totalUnrealizedPnl: 0,
      wsConnected: false, tradingProfileId: state.profiles[0].id, tradingBrokerId: 'kis', tradingAccountId: state.profiles[0].broker_account_id } })
  })
  try {
    await page.goto('/settings')
    const choose = page.getByRole('button', { name: `${state.profiles[1].name} 선택`, exact: true })
    await expect(choose).toBeDisabled()
    await expect(page.getByText('매매 대기', { exact: true })).toHaveCount(0)
    release()
    await expect(page.getByText('● 자동매매 실행 중', { exact: true })).toBeVisible()
    await expect(choose).toBeDisabled()
    expect(state.selections).toEqual([])
    assertReadOnly(state.audit)
  } finally { release() }
})

for (const market of markets) {
  test(`KIS ${market.name}: initial delay and failure are not empty; independent keyboard retry confirms empty`, async ({ page }) => {
    const audit = await mockUx(page)
    let release!: () => void
    const gate = new Promise<void>(resolve => { release = resolve })
    audit.responses.set(market.path, async route => {
      await gate
      return route.fulfill(failure)
    })
    try {
      await page.goto('/trading')
      const panel = section(page, market)
      await expect(panel).toContainText(`${market.label} 확인 중`)
      await expect(panel.getByText(market.empty, { exact: true })).toHaveCount(0)
      await expect(panel.getByTestId(`kis-${market.name}-holdings-status-checked-at`)).toHaveText('마지막 정상 확인: 없음')
      await expect(page.getByText(`${market.otherLabel} 조회 완료`, { exact: true })).toBeVisible()
      release()
      await expect(panel).toContainText(`${market.label} 확인 실패`, { timeout: 12000 })
      await expect(panel.getByText(market.empty, { exact: true })).toHaveCount(0)
      await expect(panel.locator('time')).toHaveCount(0)
      const otherReads = audit.counts.get(market.other)
      audit.responses.set(market.path, { json: empty })
      await retry(page, market).focus()
      await page.keyboard.press('Enter')
      await expect(panel).toContainText(`${market.label} 조회 완료`)
      await expect(panel.getByText(market.empty, { exact: true })).toBeVisible()
      await expect(panel.locator('time')).toHaveAttribute('datetime', /T/)
      expect(audit.counts.get(market.other)).toBe(otherReads)
      assertReadOnly(audit)
    } finally { release() }
  })

  test(`KIS ${market.name}: refetch failure preserves holdings and success timestamp; recovery updates only its query`, async ({ page }) => {
    if (market.name === 'overseas') {
      await page.setViewportSize({ width: 375, height: 812 })
      await page.emulateMedia({ colorScheme: 'dark' })
    }
    const audit = await mockUx(page)
    audit.responses.set(market.path, { json: market.data })
    await page.goto('/trading')
    const panel = section(page, market)
    await expect(panel.getByText(market.holding, { exact: true })).toBeVisible()
    await expect(panel.getByRole('cell', { name: market.quantity, exact: true })).toBeVisible()
    const checkedAt = await panel.locator('time').getAttribute('datetime')
    expect(checkedAt).toBeTruthy()
    const otherReads = audit.counts.get(market.other)
    audit.responses.set(market.path, failure)
    await retry(page, market).click()
    await expect(panel).toContainText(`이전 상태: ${market.label} 조회 완료`, { timeout: 12000 })
    await expect(panel).toContainText('이전 자료: 마지막 정상 조회의 보유 수량·가격입니다.')
    await expect(panel.getByText(market.holding, { exact: true })).toBeVisible()
    await expect(panel.getByRole('cell', { name: market.quantity, exact: true })).toBeVisible()
    await expect(panel.locator('time')).toHaveAttribute('datetime', checkedAt!)
    await expect(page.getByText(`${market.otherLabel} 조회 완료`, { exact: true })).toBeVisible()
    if (market.name === 'overseas') {
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
      await expect(retry(page, market)).toBeVisible()
      await page.screenshot({ path: '.cache/uiux-implementation-20261030/holdings-narrow-stale.png', animations: 'disabled' })
    }
    audit.responses.set(market.path, { json: empty })
    await retry(page, market).click()
    await expect(panel.getByText(market.empty, { exact: true })).toBeVisible()
    await expect(panel).not.toContainText('이전 자료:')
    await expect(panel.getByText(market.holding, { exact: true })).toHaveCount(0)
    expect(audit.counts.get(market.other)).toBe(otherReads)
    expect(await panel.locator('time').getAttribute('datetime')).not.toBe(checkedAt)
    for (let attempt = 0; attempt < 2; attempt++) {
      const readsBefore = audit.counts.get(market.path) ?? 0
      const response = page.waitForResponse(res => new URL(res.url()).pathname === market.path && res.request().method() === 'GET')
      await retry(page, market).click()
      const completed = await response
      expect(completed.ok()).toBe(true)
      expect(await completed.json()).toEqual(empty)
      await expect(retry(page, market)).toBeEnabled()
      await expect(panel).toContainText(`${market.label} 조회 완료`)
      await expect(panel).not.toContainText('상태 재조회 실패')
      await expect(panel).not.toContainText('이전 자료:')
      await expect(panel.getByText(market.empty, { exact: true })).toBeVisible()
      expect(audit.counts.get(market.path)).toBe(readsBefore + 1)
      expect(audit.counts.get(market.other)).toBe(otherReads)
    }
    assertReadOnly(audit)
  })

  test(`KIS ${market.name}: a successful empty snapshot remains explicitly previous after failure`, async ({ page }) => {
    const audit = await mockUx(page)
    await page.goto('/trading')
    const panel = section(page, market)
    await expect(panel.getByText(market.empty, { exact: true })).toBeVisible()
    const checkedAt = await panel.locator('time').getAttribute('datetime')
    audit.responses.set(market.path, failure)
    await retry(page, market).click()
    await expect(panel).toContainText('상태 재조회 실패', { timeout: 12000 })
    await expect(panel).toContainText('이전 자료:')
    await expect(panel.getByText(market.empty, { exact: true })).toBeVisible()
    await expect(panel.locator('time')).toHaveAttribute('datetime', checkedAt!)
    assertReadOnly(audit)
  })
}

for (const market of markets) {
  test(`KIS ${market.name}: holding selection keeps the original form mapping without submitting`, async ({ page }) => {
    const audit = await mockUx(page)
    audit.responses.set(market.path, { json: market.data })
    audit.responses.set('/api/price/0005A0', { json: { stck_prpr: '11000', hts_kor_isnm: '합성 국내 보유', prdy_vrss: '0', prdy_ctrt: '0', acml_vol: '100' } })
    audit.responses.set('/api/chart/0005A0', { json: [] })
    audit.responses.set('/api/overseas-price/NYS/SYNTH', { json: { last: '102', name: 'Synthetic US holding', diff: '0', rate: '0', tvol: '100' } })
    audit.responses.set('/api/overseas-chart/NYS/SYNTH', { json: [] })
    await page.goto('/trading')
    if (market.name === 'overseas') await page.getByRole('button', { name: '미국', exact: true }).click()
    const qty = page.getByRole('textbox', { name: '주문수량', exact: true })
    await qty.fill('3')
    await section(page, market).getByRole('row').filter({ hasText: market.holding }).click()
    const input = page.getByRole('textbox', { name: market.name === 'domestic' ? '6자리 종목코드 (예: 005930, 0005A0)' : '티커 (AAPL, TSLA …)' })
    await expect(input).toHaveValue(market.holding)
    await expect(qty).toHaveValue('3')
    if (market.name === 'overseas') await expect(page.getByText('NYS', { exact: true })).toBeVisible()
    await expect(page.getByRole('button', { name: market.name === 'domestic' ? '매수 주문' : '매수 주문 (USD 지정가)', exact: true })).toBeVisible()
    assertReadOnly(audit)
  })
}

async function mockToss(page: Page) {
  const audit = await mockUx(page, { activeBroker: 'toss' })
  const readPosts: string[] = []
  // This existing API is a read despite its POST transport. No order route is allowed.
  await page.route('**/api/toss-open-orders', async route => {
    const request = route.request()
    const url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) {
      audit.external.push(url.origin)
      return route.abort()
    }
    if (url.pathname !== '/api/toss-open-orders' || request.method() !== 'POST') return route.fallback()
    readPosts.push('/api/toss-open-orders')
    return route.fulfill({ json: [] })
  })
  return { audit, readPosts }
}

function assertTossGates(audit: Awaited<ReturnType<typeof mockUx>>) {
  expect(audit.counts.get('/api/balance') ?? 0).toBe(0)
  expect(audit.counts.get('/api/overseas-balance') ?? 0).toBe(0)
  assertReadOnly(audit)
}

test('Toss keeps loading, failure and successful empty handling while KIS reads stay disabled', async ({ page }) => {
  const { audit, readPosts } = await mockToss(page)
  let release!: () => void
  const gate = new Promise<void>(resolve => { release = resolve })
  audit.responses.set('/api/broker-holdings', async route => {
    await gate
    return route.fulfill(failure)
  })
  try {
    await page.goto('/trading')
    await expect(page.getByText('보유한 Toss 종목이 없습니다.', { exact: true })).toHaveCount(0)
    const holdings = page.locator('.MuiPaper-root').filter({ has: page.getByRole('heading', { name: '보유 종목', exact: true }) })
    // Wait for the requested broker scope before inspecting its loading UI.
    // During config loading the conservative KIS placeholder has two spinners.
    await expect(holdings.getByText(/^Toss(?: · accountSeq .+)?$/)).toBeVisible()
    await expect(holdings.getByRole('progressbar')).toBeVisible()
    release()
    await expect(page.getByText(/Toss 보유종목 조회 실패/)).toBeVisible({ timeout: 12000 })
    await expect(page.getByTestId('kis-domestic-holdings')).toHaveCount(0)
    audit.responses.set('/api/broker-holdings', { json: [] })
    await page.reload()
    await expect(page.getByText('보유한 Toss 종목이 없습니다.', { exact: true })).toBeVisible()
    expect(readPosts.length).toBeGreaterThan(0)
    assertTossGates(audit)
  } finally { release() }
})

test('Toss successful holdings remain visible with original quantities and no KIS holdings requests', async ({ page }) => {
  const { audit, readPosts } = await mockToss(page)
  audit.responses.set('/api/broker-holdings', { json: [{ brokerId: 'toss', market: 'kr', symbol: '0005A0', symbolName: '합성 Toss 보유', quantity: '2', averagePrice: { amount: '10000', currency: 'KRW' }, currentPrice: { amount: '11000', currency: 'KRW' }, unrealizedPnl: { amount: '2000', currency: 'KRW' } }] })
  await page.goto('/trading')
  const row = page.getByRole('row').filter({ hasText: '합성 Toss 보유' })
  await expect(row).toBeVisible()
  await expect(row.getByRole('cell', { name: '2', exact: true })).toBeVisible()
  expect(readPosts.length).toBeGreaterThan(0)
  assertTossGates(audit)
})
