import { expect, type Page } from '@playwright/test'
import { mockUx } from './uxReviewFixture'

export const keyboardStock = { pdno: '0005A0', prdt_name: '합성 키보드 종목' }
export const enterTerm = '합성엔터'
export const buttonTerm = '합성버튼'
export const oneLetterTicker = 'Z'
function gate() { let release!: () => void; const promise = new Promise<void>(resolve => { release = resolve }); return { promise, release } }
export async function keyboardSearchFixture(page: Page, broker: 'kis' | 'toss') {
  await page.clock.install()
  const audit = await mockUx(page, { activeBroker: broker, appConfigOverride: { kis_is_paper_trading: broker === 'kis' } })
  const reads: Array<{ path: string; method: string; settled: boolean }> = []
  const readPosts: string[] = []
  const notes: unknown[] = []
  const held = gate()
  const control: { hold?: Promise<void> } = { hold: held.promise }
  const name = 'Synthetic Z'
  const safety = { brokerId: 'toss', symbol: oneLetterTicker,
    stockInfo: { symbol: oneLetterTicker, name, englishName: name, isinCode: 'SYNTHETIC', market: 'us', securityType: 'stock', isCommonShare: true, status: 'normal', currency: 'USD', listDate: null, delistDate: null, sharesOutstanding: '1', leverageFactor: null },
    warnings: [], hasBlockingWarning: false, buyBlocked: false, buyBlockReason: null }
  const payloads: Record<string, unknown> = {
    [`/api/search/${encodeURIComponent(enterTerm)}`]: [keyboardStock],
    [`/api/search/${encodeURIComponent(buttonTerm)}`]: [keyboardStock],
    [`/api/search/${keyboardStock.pdno}`]: [keyboardStock],
    [`/api/price/${keyboardStock.pdno}`]: { stck_prpr: '10000', hts_kor_isnm: keyboardStock.prdt_name, prdy_vrss: '0', prdy_ctrt: '0', acml_vol: '10' },
    [`/api/chart/${keyboardStock.pdno}`]: [],
    [`/api/overseas-price/NAS/${oneLetterTicker}`]: { last: '100.25', name, diff: '0', rate: '0', tvol: '10' },
    [`/api/overseas-chart/NAS/${oneLetterTicker}`]: [],
    [`/api/toss-chart/${oneLetterTicker}`]: [],
    [`/api/toss-chart/${keyboardStock.pdno}`]: [],
    [`/api/toss-stock-safety/${oneLetterTicker}`]: safety,
    [`/api/toss-stock-safety/${keyboardStock.pdno}`]: { ...safety, symbol: keyboardStock.pdno, stockInfo: null },
  }
  for (const symbol of [keyboardStock.pdno, oneLetterTicker]) payloads[`/api/toss-market-snapshot/${symbol}`] = {
    brokerId: 'toss', market: symbol === oneLetterTicker ? 'us' : 'kr', price: { amount: '100.25', currency: symbol === oneLetterTicker ? 'USD' : 'KRW' },
    orderbook: { asks: [], bids: [], currency: 'USD' }, trades: [], priceLimits: { upperLimitPrice: null, lowerLimitPrice: null, currency: 'USD' },
  }
  const logicalPath = broker === 'kis' ? `/api/overseas-price/NAS/${oneLetterTicker}` : `/api/toss-stock-safety/${oneLetterTicker}`
  for (const [path, json] of Object.entries(payloads)) audit.responses.set(path, async route => {
    const request = route.request(), url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
    if (url.pathname !== path || request.method() !== 'GET') return route.abort()
    const read = { path, method: request.method(), settled: false }
    reads.push(read)
    const hold = path === logicalPath ? control.hold : undefined
    try { if (hold) await hold; await route.fulfill({ json }) } finally { read.settled = true }
  })
  if (broker === 'toss') await page.route('**/api/toss-open-orders', async route => {
    const request = route.request(), url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
    if (url.pathname !== '/api/toss-open-orders' || request.method() !== 'POST') return route.fallback()
    readPosts.push(url.pathname)
    return route.fulfill({ json: [] }) // Existing semantic read only; no order/preflight mutation exception.
  })
  return {
    audit, reads, readPosts, notes, broker, logicalPath,
    logicalReads: () => reads.filter(read => read.path === logicalPath).length,
    release: async () => {
      control.hold = undefined; held.release()
      await expect.poll(() => reads.filter(read => !read.settled).length).toBe(0)
      await page.clock.runFor(1)
      await page.evaluate(() => Promise.resolve())
    },
  }
}
export type KeyboardSearchFixture = Awaited<ReturnType<typeof keyboardSearchFixture>>
export function safeKeyboardSearch(state: KeyboardSearchFixture) {
  expect(state.audit.writes).toEqual([])
  expect(state.audit.external).toEqual([])
  expect(state.audit.unknown).toEqual([])
  expect(state.audit.errors).toEqual([])
}
export async function keyboardObservationWindow(page: Page) {
  // Covers the existing 250/400ms debounces without changing the production timers.
  await page.clock.runFor(600)
  await page.evaluate(() => Promise.resolve())
}
