import { expect, test, type Locator, type Page } from '@playwright/test'
import type { BalanceResult, OverseasBalanceResult } from '../../src/api/types'
import { mockUx } from './fixtures/uxReviewFixture'

const queryText = '합성검색'
const first = { pdno: '0005A0', prdt_name: '합성 첫 종목' }
const second = { pdno: '0005A1', prdt_name: `합성 긴 이름 ${'접근성 표시 '.repeat(12)}`.trim() }
const input = (page: Page) => page.getByRole('textbox', { name: '6자리 종목코드 (예: 005930, 0005A0)' })
const selectedButton = (page: Page, item: typeof first) => page.getByRole('button', { name: `${item.prdt_name} (${item.pdno}) 선택`, exact: true })
const qty = (page: Page) => page.getByRole('textbox', { name: '주문수량', exact: true })

async function setup(page: Page, broker: 'kis' | 'toss' = 'kis') {
  const audit = await mockUx(page, { activeBroker: broker })
  for (const term of [queryText, first.pdno]) audit.responses.set(`/api/search/${encodeURIComponent(term)}`, { json: [first, second] })
  for (const item of [first, second]) {
    audit.responses.set(`/api/price/${item.pdno}`, { json: { stck_prpr: '10000', hts_kor_isnm: item.prdt_name, prdy_vrss: '0', prdy_ctrt: '0', acml_vol: '100' } })
    audit.responses.set(`/api/chart/${item.pdno}`, { json: [] })
  }
  audit.responses.set('/api/overseas-price/NYS/SYNTH', { json: { last: '102', name: 'Synthetic overseas holding', diff: '0', rate: '0', tvol: '100' } })
  audit.responses.set('/api/overseas-chart/NYS/SYNTH', { json: [] })
  const readPosts: string[] = []
  // Only these existing semantic reads use POST; order/update routes remain denied.
  if (broker === 'toss') {
    await page.route('**/api/**', async route => {
      const request = route.request()
      const url = new URL(request.url())
      if (!['127.0.0.1', 'localhost'].includes(url.hostname)) {
        audit.external.push(url.origin)
        return route.abort()
      }
      if (!['/api/toss-open-orders', '/api/toss-order-preflight'].includes(url.pathname) || request.method() !== 'POST') return route.fallback()
      readPosts.push(url.pathname)
      if (url.pathname === '/api/toss-open-orders') return route.fulfill({ json: [] })
      const body = request.postDataJSON()
      const money = { amount: '10000', currency: body.symbol === 'SYNTH' ? 'USD' : 'KRW' }
      return route.fulfill({ json: {
        brokerId: 'toss', accountSeq: 'synthetic', symbol: body.symbol, market: body.symbol === 'SYNTH' ? 'us' : 'kr', side: 'buy', quantity: body.quantity,
        price: money, priceSource: 'input', buyingPower: money, sellableQuantity: null, commissionRate: '0.00015', grossAmount: money,
        estimatedCommission: money, requiredCash: money, liquidityOk: false, safetyOk: true, orderAdapterSupported: true,
        canSubmit: false, blockedReasons: ['Synthetic read-only preflight'], warnings: [],
      } })
    })
  }
  for (const symbol of [first.pdno, 'SYNTH']) {
    audit.responses.set(`/api/toss-market-snapshot/${symbol}`, { json: {
      brokerId: 'toss', market: symbol === 'SYNTH' ? 'us' : 'kr', price: { amount: '10000', currency: symbol === 'SYNTH' ? 'USD' : 'KRW' },
      orderbook: { asks: [], bids: [], currency: 'KRW' }, trades: [], priceLimits: { upperLimitPrice: null, lowerLimitPrice: null, currency: 'KRW' },
    } })
    audit.responses.set(`/api/toss-stock-safety/${symbol}`, { json: { brokerId: 'toss', symbol, stockInfo: null, warnings: [], hasBlockingWarning: false, buyBlocked: false, buyBlockReason: null } })
    audit.responses.set(`/api/toss-chart/${symbol}`, { json: [] })
  }
  return { audit, readPosts }
}
function safe(audit: Awaited<ReturnType<typeof mockUx>>) {
  expect(audit.writes).toEqual([])
  expect(audit.external).toEqual([])
  expect(audit.unknown).toEqual([])
  expect(audit.errors).toEqual([])
}
async function focusRing(button: Locator) {
  await expect(button).toBeFocused()
  expect(await button.evaluate(element => ({ style: getComputedStyle(element).outlineStyle, width: getComputedStyle(element).outlineWidth }))).toEqual({ style: 'solid', width: '2px' })
}
async function openWithKeyboard(page: Page, route: string) {
  await input(page).fill(route === '/trading' ? queryText : first.pdno)
  await input(page).press('Tab')
  await expect(page.getByRole('button', { name: '국내 종목 검색', exact: true })).toBeFocused()
  await page.keyboard.press('Enter')
  await expect(selectedButton(page, first)).toBeVisible()
  await page.keyboard.press('Tab')
  await focusRing(selectedButton(page, first))
}

for (const route of ['/trading', '/strategy']) {
  test(`${route}: keyboard results survive internal blur, Escape and selection return focus without reopening`, async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 })
    await page.emulateMedia({ colorScheme: 'dark' })
    await page.clock.install()
    const { audit } = await setup(page)
    await page.goto(route)
    if (route === '/trading') await qty(page).fill('3')
    await expect(input(page)).toBeVisible()
    if (route === '/strategy') await expect(page.getByRole('spinbutton', { name: '1회 수량', exact: true }).first()).toHaveValue('1')
    const strategyNumbers = await page.locator('input[type="number"]').evaluateAll(elements => elements.map(element => (element as HTMLInputElement).value))
    await openWithKeyboard(page, route)
    await page.clock.fastForward(250)
    await focusRing(selectedButton(page, first))
    await page.keyboard.press('Tab')
    await focusRing(selectedButton(page, second))
    await page.screenshot({ path: `.cache/uiux-implementation-20261030/keyboard-${route.slice(1)}-focus.png`, animations: 'disabled' })
    await page.keyboard.press('Escape')
    await expect(input(page)).toBeFocused()
    await expect(selectedButton(page, first)).toHaveCount(0)
    await page.clock.fastForward(250)
    await expect(selectedButton(page, first)).toHaveCount(0)
    await input(page).press('Tab')
    await page.keyboard.press('Enter')
    await expect(selectedButton(page, first)).toBeVisible()
    await page.keyboard.press('Tab')
    await page.keyboard.press('Tab')
    await focusRing(selectedButton(page, second))
    await page.keyboard.press('Enter')
    await expect(input(page)).toHaveValue(second.prdt_name)
    await expect(input(page)).toBeFocused()
    await expect(selectedButton(page, second)).toHaveCount(0)
    await page.clock.fastForward(250)
    await expect(selectedButton(page, second)).toHaveCount(0)
    if (route === '/trading') {
      await expect(page.getByText(`선택됨: ${second.pdno}`, { exact: true })).toBeVisible()
      await expect(qty(page)).toHaveValue('3')
    } else {
      await expect(page.getByText(`선택됨: ${second.prdt_name} (${second.pdno})`, { exact: true })).toBeVisible()
      expect(await page.locator('input[type="number"]').evaluateAll(elements => elements.map(element => (element as HTMLInputElement).value))).toEqual(strategyNumbers)
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
    safe(audit)
  })

  test(`${route}: focus leaving results closes them after the original delay and pointer row selection still works`, async ({ page }) => {
    await page.clock.install()
    const { audit } = await setup(page)
    await page.goto(route)
    await openWithKeyboard(page, route)
    await page.keyboard.press('Tab')
    await page.keyboard.press('Tab')
    await expect(selectedButton(page, second)).not.toBeFocused()
    await page.clock.fastForward(250)
    await expect(selectedButton(page, first)).toHaveCount(0)
    await openWithKeyboard(page, route)
    // Clicking the code cell exercises the retained row action, not its new button.
    await page.getByRole('row').filter({ has: selectedButton(page, first) }).getByRole('cell').last().click()
    await expect(input(page)).toHaveValue(first.prdt_name)
    await expect(input(page)).toBeFocused()
    await expect(selectedButton(page, first)).toHaveCount(0)
    safe(audit)
  })

  test(`${route}: pointer blur race keeps the selected value, returned focus and closed results`, async ({ page }) => {
    await page.clock.install()
    const { audit } = await setup(page)
    const raceItem = { pdno: first.pdno, prdt_name: 'RACE01' }
    // A six-character name is valid for Strategy's existing automatic search.
    // Serve it normally rather than assuming selection must produce zero searches.
    for (const term of [queryText, first.pdno, raceItem.prdt_name]) audit.responses.set(`/api/search/${encodeURIComponent(term)}`, { json: [raceItem] })
    await page.goto(route)
    await input(page).fill(route === '/trading' ? queryText : first.pdno)
    await input(page).press('Tab')
    await page.keyboard.press('Enter')
    const button = selectedButton(page, raceItem)
    await expect(button).toBeVisible()
    await page.clock.pauseAt(await page.evaluate(() => Date.now() + 1000))
    const outside = route === '/trading' ? qty(page) : page.getByRole('spinbutton', { name: '1회 수량', exact: true }).first()
    await outside.focus()
    await expect(outside).toBeFocused()
    await page.getByRole('row').filter({ has: button }).getByRole('cell').last().click()
    await expect(input(page)).toHaveValue(raceItem.prdt_name)
    await expect(input(page)).toBeFocused()
    await expect(button).toHaveCount(0)
    await page.clock.fastForward(250)
    await expect(button).toHaveCount(0)
    await page.clock.fastForward(450)
    await expect(button).toHaveCount(0)
    safe(audit)
  })
}

const domestic: BalanceResult = { items: [{ pdno: first.pdno, prdt_name: first.prdt_name, hldg_qty: '7', pchs_avg_pric: '9000', prpr: '10000', evlu_pfls_amt: '7000', evlu_pfls_rt: '10' }], summary: null }
const overseas: OverseasBalanceResult = { items: [{ ovrs_pdno: 'SYNTH', ovrs_item_name: 'Synthetic overseas holding', ovrs_cblc_qty: '4', pchs_avg_pric: '100', now_pric2: '102', ovrs_stck_evlu_amt: '408', frcr_evlu_pfls_amt: '8', evlu_pfls_rt: '2', ovrs_excg_cd: 'NYSE', tr_mket_name: 'Synthetic NYSE' }], summary: null }

for (const broker of ['kis', 'toss'] as const) {
  for (const market of ['kr', 'us'] as const) {
    test(`${broker} ${market}: Tab/Enter selects a holding with the original market, symbol and quantity contract`, async ({ page }) => {
      const { audit, readPosts } = await setup(page, broker)
      const name = market === 'kr' ? first.prdt_name : 'Synthetic overseas holding'
      const symbol = market === 'kr' ? first.pdno : 'SYNTH'
      if (broker === 'kis') {
        audit.responses.set('/api/balance', { json: domestic })
        audit.responses.set('/api/overseas-balance', { json: overseas })
      } else {
        audit.responses.set('/api/broker-holdings', { json: [{ brokerId: 'toss', market, symbol, symbolName: name, quantity: '4', averagePrice: { amount: '100', currency: market === 'kr' ? 'KRW' : 'USD' }, currentPrice: { amount: '102', currency: market === 'kr' ? 'KRW' : 'USD' }, unrealizedPnl: { amount: '8', currency: market === 'kr' ? 'KRW' : 'USD' } }] })
      }
      await page.goto('/trading')
      if (market === 'us') await page.getByRole('button', { name: '미국', exact: true }).press('Enter')
      await qty(page).fill('3')
      const button = page.getByRole('button', { name: `${name} (${symbol}) 선택`, exact: true })
      // Start immediately before the selected row's only native control, then Tab into it.
      await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur())
      for (let tabs = 0; tabs < 45 && !(await button.evaluate(element => document.activeElement === element)); tabs++) await page.keyboard.press('Tab')
      await focusRing(button)
      await page.keyboard.press('Enter')
      const field = market === 'kr' ? input(page) : page.getByRole('textbox', { name: '티커 (AAPL, TSLA …)' })
      await expect(field).toHaveValue(name)
      await expect(qty(page)).toHaveValue('3')
      await expect(page.getByRole('button', { name: '매수', exact: true })).toHaveAttribute('aria-pressed', 'true')
      if (market === 'us' && broker === 'kis') await expect(page.getByText('NYS', { exact: true })).toBeVisible()
      await page.getByRole('row').filter({ has: button }).getByRole('cell', { name: broker === 'kis' && market === 'kr' ? '7주' : '4', exact: true }).click()
      await expect(field).toHaveValue(name)
      await expect(qty(page)).toHaveValue('3')
      if (broker === 'toss') {
        expect(audit.counts.get('/api/balance') ?? 0).toBe(0)
        expect(audit.counts.get('/api/overseas-balance') ?? 0).toBe(0)
        expect(readPosts.length).toBeGreaterThan(0)
        await expect(page.getByRole('button', { name: market === 'kr' ? '매수 주문' : '매수 주문 (USD 지정가)', exact: true })).toBeDisabled()
      }
      safe(audit)
    })
  }
}
