import { expect, type Page } from '@playwright/test'
import { mkdir, writeFile } from 'node:fs/promises'
import { mockUx } from './uxReviewFixture'

export type SearchSurface = 'Trading' | 'Strategy' | 'LTH'
export type SearchScope = { broker: 'kis' | 'toss'; profile: string; account: string; paper: boolean }
export interface SearchReply { status: number; json: unknown }
export interface SearchTicket {
  id: number
  path: string
  scope: SearchScope
  held: boolean
  released: boolean
  settled: boolean
  browserFinished: boolean
  browserError?: string
  responseStatus?: number
  reply: SearchReply
  browserDone: Promise<void>
  complete: (reply?: Partial<SearchReply>) => void
}
const ticketHeader = 'x-kbd02-ticket'
const symbols = ['A', 'B', 'Z']
const price = (symbol: string) => ({ last: '100.25', name: `Synthetic ${symbol}`, diff: '0', rate: '0', tvol: '10' })
function safety(symbol: string) {
  return { brokerId: 'toss', symbol,
    stockInfo: { symbol, name: `Synthetic ${symbol}`, englishName: `Synthetic ${symbol}`, isinCode: 'SYNTHETIC', market: 'us', securityType: 'stock', isCommonShare: true, status: 'normal', currency: 'USD', listDate: null, delistDate: null, sharesOutstanding: '1', leverageFactor: null },
    warnings: [], hasBlockingWarning: false, buyBlocked: false, buyBlockReason: null }
}
export function invalidSearchReply(broker: 'kis' | 'toss', symbol: string): SearchReply {
  return { status: 200, json: broker === 'kis' ? { ...price(symbol), last: '0', name: '' } : { ...safety(symbol), stockInfo: null } }
}
export const failedSearchReply = (): SearchReply => ({ status: 503, json: { code: 'SYNTHETIC', message: 'Synthetic lookup unavailable' } })

export async function keyboardSearchRaceFixture(page: Page, broker: 'kis' | 'toss' = 'kis') {
  await page.clock.install()
  const audit = await mockUx(page, { activeBroker: broker, appConfigOverride: { kis_is_paper_trading: broker === 'kis' } })
  const scope: SearchScope = { broker, profile: broker === 'kis' ? 'paper' : 'toss-live', account: broker === 'kis' ? '12345678-01' : '1', paper: broker === 'kis' }
  const heldPaths = new Set<string>()
  const defaults = new Map<string, SearchReply>()
  const tickets: SearchTicket[] = []
  const notes: unknown[] = [], readPosts: string[] = []
  const strategyState = { enabled: false } // Synthetic GET state only; no strategy toggle/write request.
  const appConfig = () => ({ active_broker_id: scope.broker, active_broker_account_id: scope.account,
    active_profile_id: scope.profile, active_profile_name: `Synthetic ${scope.profile}`, kis_app_key_masked: '***', kis_account_no: scope.account,
    kis_is_paper_trading: scope.paper, kis_configured: true, active_broker_configured: true, discord_enabled: false, notification_levels: [] })
  audit.responses.set('/api/app-config', async route => { await route.fulfill({ json: appConfig() }) })
  audit.responses.set('/api/check-config', async route => { await route.fulfill({ json: { broker_id: scope.broker, broker_account_id: scope.account,
    real_key_set: true, real_account_set: true, paper_key_set: true, active_mode: scope.paper ? '모의투자' : '실전', is_ready: true, discord_configured: false, base_url: 'synthetic', issues: [] } }) })
  audit.responses.set('/api/profiles', async route => { await route.fulfill({ json: [{ id: scope.profile, broker_id: scope.broker, broker_account_id: scope.account,
    name: `Synthetic ${scope.profile}`, is_paper_trading: scope.paper, live_trading_consent: false, app_key_masked: '***', account_no: scope.account, is_active: true, is_configured: true }] }) })
  audit.responses.set('/api/strategies', async route => { await route.fulfill({ json: [{ id: 'leveraged_trend_hold_default', name: 'LeveragedTrendHoldStrategy',
    enabled: strategyState.enabled, brokerId: scope.broker, brokerAccountId: scope.account, targetSymbols: ['SOXL'], targetSymbolNames: { SOXL: 'Synthetic existing target' }, orderQuantity: 1,
    params: { entries: [{ leveraged_symbol: 'SOXL', leveraged_symbol_name: 'Synthetic existing target', inverse_leveraged_symbol: '', inverse_leveraged_symbol_name: '', base_symbols: [], base_symbol_names: {}, base_symbol_roles: {}, quantity: 1, inverse_quantity: 1, is_overseas: true }] } }] }) })
  for (const symbol of symbols) {
    for (const exchange of ['NAS', 'NYS', 'AMS']) {
      defaults.set(`/api/overseas-price/${exchange}/${symbol}`, { status: 200, json: price(symbol) })
      defaults.set(`/api/overseas-chart/${exchange}/${symbol}`, { status: 200, json: [] })
    }
    defaults.set(`/api/toss-stock-safety/${symbol}`, { status: 200, json: safety(symbol) })
    defaults.set(`/api/toss-chart/${symbol}`, { status: 200, json: [] })
    defaults.set(`/api/toss-market-snapshot/${symbol}`, { status: 200, json: { brokerId: 'toss', market: 'us', price: { amount: '100.25', currency: 'USD' }, orderbook: { asks: [], bids: [], currency: 'USD' }, trades: [], priceLimits: { upperLimitPrice: null, lowerLimitPrice: null, currency: 'USD' } } })
  }
  defaults.set('/api/price/0005A0', { status: 200, json: { stck_prpr: '10000', hts_kor_isnm: 'Synthetic KR', prdy_vrss: '0', prdy_ctrt: '0', acml_vol: '10' } })
  defaults.set('/api/chart/0005A0', { status: 200, json: [] })
  for (const path of defaults.keys()) audit.responses.set(path, async route => {
    const request = route.request(), url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
    if (url.pathname !== path || request.method() !== 'GET') return route.abort()
    let release!: () => void
    const gate = new Promise<void>(resolve => { release = resolve })
    const ticket: SearchTicket = { id: tickets.length + 1, path, scope: { ...scope }, held: heldPaths.has(path), released: false,
      settled: false, browserFinished: false, reply: { ...defaults.get(path)! }, browserDone: Promise.resolve(),
      complete: reply => { if (!ticket.released) { Object.assign(ticket.reply, reply); ticket.released = true; release() } } }
    tickets.push(ticket)
    // Installed before fulfill, including automatic reads. A path alone cannot distinguish scope ABA.
    ticket.browserDone = page.waitForResponse(response => response.headers()[ticketHeader] === String(ticket.id)).then(async response => {
      ticket.responseStatus = response.status()
      const failure = await response.finished()
      if (failure) throw failure
      ticket.browserFinished = true
    }).catch(error => { ticket.browserError = String(error) })
    try {
      if (ticket.held) await gate
      await route.fulfill({ status: ticket.reply.status, json: ticket.reply.json, headers: { [ticketHeader]: String(ticket.id) } })
    } finally { ticket.settled = true }
  })
  await page.route('**/api/toss-open-orders', async route => {
    const request = route.request(), url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
    if (scope.broker !== 'toss' || url.pathname !== '/api/toss-open-orders' || request.method() !== 'POST') return route.fallback()
    readPosts.push(url.pathname); await route.fulfill({ json: [] })
  })
  const lookupPath = (symbol: string, selectedBroker = scope.broker, exchange = 'NAS') => selectedBroker === 'kis' ? `/api/overseas-price/${exchange}/${symbol}` : `/api/toss-stock-safety/${symbol}`
  return { audit, scope, tickets, notes, readPosts, strategyState, lookupPath, defaults,
    hold: (symbol: string, selectedBroker = scope.broker, exchange = 'NAS') => { heldPaths.add(lookupPath(symbol, selectedBroker, exchange)) },
    auto: (symbol: string, selectedBroker = scope.broker, exchange = 'NAS') => { heldPaths.delete(lookupPath(symbol, selectedBroker, exchange)) },
    reads: (symbol: string, selectedBroker = scope.broker, exchange = 'NAS') => tickets.filter(ticket => ticket.path === lookupPath(symbol, selectedBroker, exchange)),
    drain: async () => {
      heldPaths.clear()
      // Finite original NAS/NYS/AMS fallback plus post-selection query notifications.
      for (let pass = 0; pass < 8; pass++) {
        const snapshot = [...tickets]
        for (const ticket of snapshot) if (ticket.held && !ticket.released) ticket.complete()
        await Promise.all(snapshot.map(ticket => ticket.browserDone))
        await page.clock.runFor(1); await page.evaluate(() => Promise.resolve())
        if (tickets.length === snapshot.length && tickets.every(ticket => ticket.settled)) return
      }
      throw new Error('Synthetic search cleanup did not quiesce')
    },
  }
}
export type KeyboardRaceFixture = Awaited<ReturnType<typeof keyboardSearchRaceFixture>>
export async function openSearchSurface(page: Page, surface: SearchSurface, spa = false) {
  const destination = surface === 'Trading' ? 'Trading' : 'Strategy'
  if (spa) await page.getByRole('button', { name: destination, exact: true }).click()
  else await page.goto(surface === 'Trading' ? '/trading' : '/strategy')
  if (surface === 'LTH') {
    const card = page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' })
    const input = card.getByRole('textbox', { name: 'ETF 티커', exact: true })
    await expect(input).toBeVisible()
    return { input, button: card.getByRole('button', { name: '레버리지 대상 검색', exact: true }), card }
  }
  await page.getByRole('button', { name: surface === 'Trading' ? '미국' : '해외 (미국)', exact: true }).click()
  return { input: page.getByRole('textbox', { name: surface === 'Trading' ? '티커 (AAPL, TSLA …)' : '티커 (예: AAPL, SPYM, QQQ)', exact: true }),
    button: page.getByRole('button', { name: '미국 종목 검색', exact: true }), card: page.locator('body') }
}
export type KeyboardSearchView = Awaited<ReturnType<typeof openSearchSurface>>
export async function switchSearchMarket(page: Page, surface: SearchSurface, market: 'KR' | 'US') {
  const scope = surface === 'LTH' ? page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' }) : page.locator('body')
  const usName = surface === 'Strategy' ? '해외 (미국)' : '미국'
  const group = scope.getByRole('button', { name: usName, exact: true }).locator('..')
  await group.getByRole('button', { name: market === 'US' ? usName : '국내', exact: true }).click()
}
export async function expectSearchSelection(page: Page, surface: SearchSurface, symbol: string) {
  if (surface === 'Trading') await expect(page.getByTestId('candle-data-summary')).toHaveAccessibleName(`${symbol} 차트 자료`)
  else if (surface === 'Strategy') await expect(page.getByText(`선택됨: Synthetic ${symbol} (${symbol})`, { exact: true })).toBeVisible()
  else await expect(page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' }).getByText(`Synthetic ${symbol} (${symbol})`, { exact: true })).toBeVisible()
}
export async function expectNoSearchSelection(page: Page, surface: SearchSurface, symbol: string) {
  if (surface === 'Trading') await expect(page.getByTestId('candle-data-summary')).not.toHaveAccessibleName(`${symbol} 차트 자료`)
  else if (surface === 'Strategy') await expect(page.getByText(new RegExp(`^선택됨: .*\\(${symbol}\\)$`))).toHaveCount(0)
  else await expect(page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' }).getByText(new RegExp(`^.*\\(${symbol}\\)$`))).toHaveCount(0)
}
export async function expectNoSearchError(page: Page, surface: SearchSurface, symbol: string) {
  const scope = surface === 'LTH' ? page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' }) : page.locator('body')
  await expect(scope).not.toContainText(`"${symbol}"`)
}
export async function waitSearchReply(state: KeyboardRaceFixture, symbol: string, ordinal = 1, broker = state.scope.broker, exchange = 'NAS') {
  await expect.poll(() => state.reads(symbol, broker, exchange).filter(ticket => ticket.held).length).toBeGreaterThanOrEqual(ordinal)
  return state.reads(symbol, broker, exchange).filter(ticket => ticket.held)[ordinal - 1]
}
export async function settleSearchReply(page: Page, ticket: SearchTicket, reply?: Partial<SearchReply>) {
  ticket.complete(reply)
  await ticket.browserDone
  expect(ticket.browserError).toBeUndefined()
  expect(ticket.browserFinished).toBe(true)
  expect(ticket.responseStatus).toBe(ticket.reply.status)
  await expect.poll(() => ticket.settled).toBe(true)
  await page.clock.runFor(1); await page.evaluate(() => Promise.resolve())
}
export function safeSearchRace(state: KeyboardRaceFixture) {
  expect(state.audit.writes).toEqual([]); expect(state.audit.external).toEqual([])
  expect(state.audit.unknown).toEqual([]); expect(state.audit.errors).toEqual([])
  expect(state.tickets.map(ticket => ticket.browserError).filter(Boolean)).toEqual([])
}
export async function captureSearchRace(page: Page, state: KeyboardRaceFixture, title: string, status: string | undefined) {
  await state.drain()
  const directory = '.cache/kbd02-implementation-20261031/evidence'
  await mkdir(directory, { recursive: true })
  const name = title.replace(/[^a-zA-Z0-9-]/g, '_')
  await writeFile(`${directory}/${name}.json`, JSON.stringify({ status, notes: state.notes,
    requests: state.tickets.map(({ complete: _complete, browserDone: _browserDone, ...ticket }) => ticket), readPosts: state.readPosts,
    counts: Object.fromEntries(state.audit.counts), writes: state.audit.writes, external: state.audit.external, unknown: state.audit.unknown, pageErrors: state.audit.errors }, null, 2))
  await page.screenshot({ path: `${directory}/${name}.png`, animations: 'disabled' })
}

// Source-only bridge: preserves the actual router, hooks, QueryClient and StrictMode.
export async function installKeyboardScopeHarness(page: Page, state: KeyboardRaceFixture) {
  let intercepted = false
  await page.route('**/src/main.tsx*', async route => {
    const url = new URL(route.request().url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { state.audit.external.push(url.origin); return route.abort() }
    if (url.pathname !== '/src/main.tsx' || route.request().method() !== 'GET') return route.fallback()
    const response = await route.fetch(), original = await response.text()
    if (!original.includes('queryClient') || !original.includes('RouterProvider')) throw new Error('Production entry QueryClient/router source no longer matches')
    intercepted = true
    await mkdir('.cache/kbd02-implementation-20261031', { recursive: true })
    await writeFile('.cache/kbd02-implementation-20261031/source-entry-transformed.txt', original)
    await route.fulfill({ response, body: original + `\nwindow.__kbd02Scope = { identity: Math.random().toString(36), current: () => queryClient.getQueryData(['appConfig']), refresh: async () => { await queryClient.invalidateQueries({ queryKey: ['appConfig'] }); await queryClient.invalidateQueries({ queryKey: ['strategies'] }); } };\n` })
  })
  return { intercepted: () => intercepted,
    identity: () => page.evaluate(() => (Reflect.get(window, '__kbd02Scope') as { identity: string }).identity),
    current: () => page.evaluate(() => (Reflect.get(window, '__kbd02Scope') as { current: () => unknown }).current()),
    change: async (next: Partial<SearchScope>) => {
      Object.assign(state.scope, next)
      await page.evaluate(() => (Reflect.get(window, '__kbd02Scope') as { refresh: () => Promise<void> }).refresh())
      await page.clock.runFor(1); await page.evaluate(() => Promise.resolve())
    },
  }
}
