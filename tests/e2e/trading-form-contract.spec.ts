import { expect, test, type Page } from '@playwright/test'
import { mockApi } from './fixtures/strategyResearchFixture'

async function setup(page: Page, broker: 'kis' | 'toss' = 'kis') {
  const orders: Array<{ path: string; input: Record<string, unknown> }> = []
  const control = { canSubmit: false, failNext: false }
  await mockApi(page, { activeBroker: broker, appConfigOverride: { kis_is_paper_trading: broker === 'kis' } })
  await page.route('**/api/**', async route => {
    const path = new URL(route.request().url()).pathname
    if (!path.startsWith('/api/')) return route.fallback()
    if (path === '/api/order' || path === '/api/overseas-order') {
      orders.push({ path, input: route.request().postDataJSON() })
      if (control.failNext) {
        control.failNext = false
        return route.fulfill({ status: 400, json: { code: 'MOCK_REJECT', message: '합성 주문 거절' } })
      }
      return route.fulfill({ json: { odno: 'MOCK-ONLY', ord_tmd: '120000', krx_fwdg_ord_orgno: 'MOCK' } })
    }
    if (path.startsWith('/api/overseas-chart/')) return route.fulfill({ json: [] })
    if (path.startsWith('/api/price/')) return route.fulfill({ json: { stck_prpr: '10000', hts_kor_isnm: '합성 종목', prdy_vrss: '0', prdy_ctrt: '0', acml_vol: '100' } })
    if (path.startsWith('/api/overseas-price/')) return route.fulfill({ json: { last: '100.25', name: 'Synthetic ETF', diff: '0', rate: '0', tvol: '100', base: '100', open: '100', high: '101', low: '99' } })
    if (path.startsWith('/api/toss-market-snapshot/')) return route.fulfill({ json: {
      brokerId: 'toss', market: 'us', price: { amount: '100.25', currency: 'USD' },
      orderbook: { asks: [], bids: [], currency: 'USD' }, trades: [], priceLimits: { upperLimitPrice: null, lowerLimitPrice: null, currency: 'USD' },
    } })
    if (path.startsWith('/api/toss-stock-safety/')) return route.fulfill({ json: { brokerId: 'toss', symbol: 'MOCK', stockInfo: null, warnings: [], hasBlockingWarning: false, buyBlocked: false, buyBlockReason: null } })
    if (path === '/api/toss-market-calendar') {
      const day = { date: '2026-10-29', daySession: null, preSession: null, regularSession: { startTime: '22:30', endTime: '05:00' }, afterSession: null, isDayOpen: false, isPreOpen: false, isRegularOpen: false, isAfterOpen: false }
      return route.fulfill({ json: { brokerId: 'toss', kr: day, us: day, summary: '합성 휴장 세션' } })
    }
    if (path === '/api/toss-open-orders') return route.fulfill({ json: [] })
    if (path === '/api/toss-order-preflight') {
      const input = route.request().postDataJSON()
      const money = { amount: '100.25', currency: 'USD' }
      return route.fulfill({ json: { brokerId: 'toss', accountSeq: 'synthetic', symbol: input.symbol, market: 'us', side: 'buy', quantity: input.quantity, price: money, priceSource: 'input', buyingPower: money, sellableQuantity: null, commissionRate: '0.00015', grossAmount: money, estimatedCommission: money, requiredCash: money, liquidityOk: control.canSubmit, safetyOk: true, orderAdapterSupported: true, canSubmit: control.canSubmit, blockedReasons: control.canSubmit ? [] : ['합성 사전검증 보류'], warnings: [] } })
    }
    return route.fallback()
  })
  return { orders, control }
}
const domestic = (page: Page) => page.getByRole('textbox', { name: '6자리 종목코드 (예: 005930, 0005A0)' })
const quantity = (page: Page) => page.getByRole('textbox', { name: '주문수량', exact: true })

test('KIS domestic limit validates quantity, keeps rejection draft and clears after mock success', async ({ page }) => {
  const { orders, control } = await setup(page)
  await page.goto('/trading')
  await domestic(page).fill('0005A0')
  const submit = page.getByRole('button', { name: '매수 주문', exact: true })
  await submit.click()
  await expect(page.getByText('수량을 입력하세요.', { exact: true })).toBeVisible()
  expect(orders).toHaveLength(0)
  await quantity(page).fill('2')
  await page.getByRole('button', { name: '현재가', exact: true }).click()
  await expect(page.getByRole('textbox', { name: '주문가격', exact: true })).toHaveValue('10000')
  control.failNext = true
  await submit.click()
  await expect(page.getByText('합성 주문 거절', { exact: true })).toBeVisible()
  await expect(quantity(page)).toHaveValue('2')
  await submit.click()
  await expect(page.getByText('주문 완료 — 주문번호: MOCK-ONLY', { exact: true })).toBeVisible()
  expect(orders.map(o => o.input)).toEqual(Array(2).fill({ symbol: '0005A0', side: 'Buy', order_type: 'Limit', quantity: 2, price: 10000 }))
  await expect(quantity(page)).toHaveValue('')
})

test('KIS domestic market order retains zero price contract', async ({ page }) => {
  const { orders } = await setup(page)
  await page.goto('/trading')
  await domestic(page).fill('005930')
  await page.getByRole('button', { name: '시장가', exact: true }).click()
  await expect(page.getByRole('textbox', { name: '주문가격', exact: true })).toHaveCount(0)
  await quantity(page).fill('3')
  await page.getByRole('button', { name: '매수 주문', exact: true }).click()
  await expect(page.getByText('주문 완료 — 주문번호: MOCK-ONLY', { exact: true })).toBeVisible()
  expect(orders[0]).toEqual({ path: '/api/order', input: { symbol: '005930', side: 'Buy', order_type: 'Market', quantity: 3, price: 0 } })
})

test('KIS US maps exchange and keeps paper sell restriction', async ({ page }) => {
  const { orders } = await setup(page)
  await page.goto('/trading')
  await page.getByRole('button', { name: '미국', exact: true }).click()
  const ticker = page.getByRole('textbox', { name: '티커 (AAPL, TSLA …)' })
  await ticker.fill('AAPL')
  await ticker.press('Enter')
  await page.getByRole('button', { name: '현재가', exact: true }).click()
  await quantity(page).fill('2')
  await page.getByRole('button', { name: '매수 주문 (USD 지정가)', exact: true }).click()
  await expect(page.getByText('주문 완료 — 주문번호: MOCK-ONLY', { exact: true })).toBeVisible()
  expect(orders[0]).toEqual({ path: '/api/overseas-order', input: { symbol: 'AAPL', exchange: 'NASD', side: 'Buy', quantity: 2, price: 100.25 } })
  await ticker.fill('SPY')
  await ticker.press('Enter')
  await page.getByRole('button', { name: '매도', exact: true }).click()
  await expect(page.getByRole('button', { name: '매도 주문 (USD 지정가)' })).toBeDisabled()
  expect(orders).toHaveLength(1)
})

test('Toss preflight blocks submission, closed explicit session blocks, auto preserves mock payload', async ({ page }) => {
  const { orders, control } = await setup(page, 'toss')
  await page.goto('/trading')
  await page.getByRole('button', { name: '미국', exact: true }).click()
  const ticker = page.getByRole('textbox', { name: '티커 (AAPL, TSLA …)' })
  await ticker.fill('AAPL')
  await ticker.press('Enter')
  await page.getByRole('textbox', { name: '주문가격 (USD)' }).fill('100.25')
  await quantity(page).fill('1')
  const submit = page.getByRole('button', { name: '매수 주문 (USD 지정가)' })
  await expect(page.getByText('합성 사전검증 보류', { exact: true })).toBeVisible()
  await expect(submit).toBeDisabled()
  control.canSubmit = true
  await quantity(page).fill('2')
  await expect(submit).toBeEnabled()
  await page.getByRole('button', { name: '정규', exact: true }).click()
  await submit.click()
  await expect(page.getByText(/현재 시간은 선택한 Toss 미국 정규 세션이 아닙니다/)).toBeVisible()
  expect(orders).toHaveLength(0)
  await page.getByRole('button', { name: '자동', exact: true }).click()
  await submit.click()
  await expect(page.getByText('Toss 주문 완료 — 주문번호: MOCK-ONLY')).toBeVisible()
  expect(orders[0]).toEqual({ path: '/api/order', input: { symbol: 'AAPL', side: 'Buy', order_type: 'Limit', quantity: 2, price: 100.25, toss_session: 'auto' } })
})

test('market switch clears draft and mobile form remains inside viewport', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await setup(page)
  await page.goto('/trading')
  await domestic(page).fill('005930')
  await quantity(page).fill('2')
  await page.getByRole('textbox', { name: '주문가격', exact: true }).fill('10000')
  await page.getByRole('button', { name: '미국', exact: true }).click()
  await expect(quantity(page)).toHaveValue('')
  await expect(page.getByRole('textbox', { name: '주문가격 (USD)' })).toHaveValue('')
  await expect(page.getByRole('button', { name: '매수 주문 (USD 지정가)' })).toBeDisabled()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBeTruthy()
})
