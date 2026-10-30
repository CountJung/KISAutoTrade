import { expect, test, type Page } from '@playwright/test'
import type { BalanceResult, OverseasBalanceResult } from '../../src/api/types'
import { mockUx } from './fixtures/uxReviewFixture'

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
