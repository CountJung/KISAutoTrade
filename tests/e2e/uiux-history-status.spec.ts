import { expect, test, type Page } from '@playwright/test'
import type { DailyStats, TradeRecord } from '../../src/api/types'
import { mockUx } from './fixtures/uxReviewFixture'

type Kind = 'stats' | 'trades'
type Control = { status: number; json: unknown; hold?: Promise<void> }
const paths: Record<Kind, string> = { stats: '/api/stats', trades: '/api/trades' }
const labels: Record<Kind, string> = { stats: '통계', trades: '체결 목록' }
const rangeA = { from: '2026-09-01', to: '2026-09-07' }
const rangeB = { from: '2026-09-08', to: '2026-09-14' }
const zero: DailyStats = {
  date: rangeA.from, total_trades: 0, winning_trades: 0, losing_trades: 0,
  gross_profit: 0, gross_loss: 0, net_profit: 0, fees_paid: 0,
  win_rate: 0, profit_factor: 0, starting_balance: 100000, ending_balance: 100000,
}
const profit: DailyStats = { ...zero, total_trades: 2, winning_trades: 1, losing_trades: 1, net_profit: 10000, win_rate: 0.5 }
const trade: TradeRecord = {
  id: 'SYNTHETIC-KRW', timestamp: '2026-09-01T10:00:00', symbol: '0005A0', symbol_name: '합성 체결 자료',
  side: 'buy', quantity: 2, price: 11000, total_amount: 22000, fee: 11,
  strategy_id: 'SYNTHETIC-STRATEGY', order_id: 'SYNTHETIC-ORDER', status: 'filled', signal_reason: 'synthetic fixture',
  market: 'domestic', currency: 'KRW', slippage: 100, slippage_bps: 5,
  provider: 'kis', provider_order_id: 'SYNTHETIC-ORDER', provider_request_id: 'SYNTHETIC-REQUEST', provider_tr_id: 'SYNTHETIC-TR',
}
const panel = (page: Page, kind: Kind) => page.getByTestId(`history-${kind}-panel`)
const retry = (page: Page, kind: Kind) => panel(page, kind).getByRole('button', { name: `${labels[kind]} 다시 확인` })
const emptyTrades = '해당 기간에 체결 기록이 없습니다. (자동매매 실행 시 기록됩니다)'

function gate() {
  let release!: () => void
  const promise = new Promise<void>(resolve => { release = resolve })
  return { promise, release }
}

async function setup(page: Page) {
  const audit = await mockUx(page)
  const controls: Record<Kind, Control> = { stats: { status: 200, json: [zero] }, trades: { status: 200, json: [] } }
  const reads: Array<{ kind: Kind; from: string | null; to: string | null }> = []
  for (const kind of ['stats', 'trades'] as const) {
    // The response Map is already behind mockUx's hostname/method gate. Keep
    // the same explicit boundary in this dynamic handler, with no live fallback.
    audit.responses.set(paths[kind], async route => {
      const request = route.request()
      const url = new URL(request.url())
      if (!['127.0.0.1', 'localhost'].includes(url.hostname)) {
        audit.external.push(url.origin)
        return route.abort()
      }
      if (url.pathname !== paths[kind] || request.method() !== 'GET') return route.abort()
      reads.push({ kind, from: url.searchParams.get('from'), to: url.searchParams.get('to') })
      const response = { ...controls[kind] }
      if (response.hold) await response.hold
      return route.fulfill({ status: response.status, json: response.json })
    })
  }
  return { audit, controls, reads }
}

async function dates(page: Page, range: typeof rangeA) {
  await page.getByLabel('시작일', { exact: true }).fill(range.from)
  await page.getByLabel('종료일', { exact: true }).fill(range.to)
}
async function apply(page: Page, range = rangeA) {
  await dates(page, range)
  await page.getByRole('button', { name: '조회', exact: true }).click()
}
async function expectZero(page: Page) {
  for (const value of ['+0원', '0건', '0.0%']) await expect(panel(page, 'stats').getByText(value, { exact: true })).toBeVisible()
}
async function noSummary(page: Page) {
  for (const value of ['기간 순손익', '+0원', '0건', '0.0%']) await expect(panel(page, 'stats').getByText(value, { exact: true })).toHaveCount(0)
}
function safe(audit: Awaited<ReturnType<typeof mockUx>>) {
  expect(audit.writes).toEqual([])
  expect(audit.external).toEqual([])
  expect(audit.unknown).toEqual([])
  expect(audit.errors).toEqual([])
}
async function repeatReads(page: Page, state: Awaited<ReturnType<typeof setup>>, kind: Kind) {
  const other: Kind = kind === 'stats' ? 'trades' : 'stats'
  const otherBefore = state.audit.counts.get(paths[other])
  for (let i = 0; i < 2; i++) {
    const ownBefore = state.audit.counts.get(paths[kind]) ?? 0
    const response = page.waitForResponse(res => new URL(res.url()).pathname === paths[kind] && res.request().method() === 'GET')
    await retry(page, kind).focus()
    await page.keyboard.press('Enter')
    const completed = await response
    expect(completed.ok()).toBe(true)
    expect(await completed.json()).toEqual(state.controls[kind].json)
    await expect(retry(page, kind)).toBeEnabled()
    await expect(panel(page, kind)).toContainText(`${labels[kind]} 조회 완료`)
    await expect(panel(page, kind)).not.toContainText('상태 재조회 실패')
    expect(state.audit.counts.get(paths[kind])).toBe(ownBefore + 1)
    expect(state.audit.counts.get(paths[other])).toBe(otherBefore)
  }
}

test('before search and delayed first queries have no invented zero; stats failure leaves successful trades visible', async ({ page }) => {
  const state = await setup(page)
  const held = gate()
  state.controls.stats = { status: 503, json: { message: 'Synthetic stats unavailable' }, hold: held.promise }
  state.controls.trades = { status: 200, json: [trade], hold: held.promise }
  try {
    await page.goto('/history')
    for (const kind of ['stats', 'trades'] as const) {
      await expect(panel(page, kind)).toContainText(`${labels[kind]} 조회 전`)
      await expect(retry(page, kind)).toHaveCount(0)
    }
    expect(state.reads).toEqual([])
    await noSummary(page)
    await apply(page)
    for (const kind of ['stats', 'trades'] as const) await expect(panel(page, kind)).toContainText(`${labels[kind]} 확인 중`)
    await noSummary(page)
    held.release()
    await expect(panel(page, 'stats')).toContainText('통계 확인 실패', { timeout: 12000 })
    await expect(panel(page, 'trades').getByText(trade.symbol_name, { exact: true })).toBeVisible()
    await noSummary(page)
    await expect(panel(page, 'stats').locator('time')).toHaveCount(0)
    const tradeReads = state.audit.counts.get(paths.trades)
    state.controls.stats = { status: 200, json: [zero] }
    await retry(page, 'stats').focus()
    await page.keyboard.press('Enter')
    await expectZero(page)
    expect(state.audit.counts.get(paths.trades)).toBe(tradeReads)
    await repeatReads(page, state, 'stats')
    await expectZero(page)
    expect(state.reads.every(read => read.from === rangeA.from && read.to === rangeA.to)).toBe(true)
    safe(state.audit)
  } finally { held.release() }
})

test('successful zero statistics and empty trades are explicitly confirmed for the applied range', async ({ page }) => {
  const state = await setup(page)
  await page.goto('/history')
  await apply(page)
  await expectZero(page)
  await expect(panel(page, 'trades').getByText(emptyTrades, { exact: true })).toBeVisible()
  for (const kind of ['stats', 'trades'] as const) {
    await expect(panel(page, kind)).toContainText(`${labels[kind]} 조회 완료`)
    await expect(page.getByTestId(`history-${kind}-status-range`)).toHaveText(`적용 기간: ${rangeA.from} ~ ${rangeA.to}`)
    await expect(panel(page, kind).locator('time')).toHaveAttribute('datetime', /T/)
  }
  safe(state.audit)
})

test('both first queries can fail and only the trades retry recovers its result with repeated keyboard reads', async ({ page }) => {
  const state = await setup(page)
  for (const kind of ['stats', 'trades'] as const) state.controls[kind] = { status: 503, json: { message: 'Synthetic unavailable' } }
  await page.goto('/history')
  await apply(page)
  for (const kind of ['stats', 'trades'] as const) await expect(panel(page, kind)).toContainText(`${labels[kind]} 확인 실패`, { timeout: 12000 })
  await noSummary(page)
  await expect(panel(page, 'trades').getByText(emptyTrades, { exact: true })).toHaveCount(0)
  const statsReads = state.audit.counts.get(paths.stats)
  state.controls.trades = { status: 200, json: [] }
  await retry(page, 'trades').focus()
  await page.keyboard.press('Enter')
  await expect(panel(page, 'trades').getByText(emptyTrades, { exact: true })).toBeVisible()
  await repeatReads(page, state, 'trades')
  await expect(panel(page, 'stats')).toContainText('통계 확인 실패')
  expect(state.audit.counts.get(paths.stats)).toBe(statsReads)
  await noSummary(page)
  safe(state.audit)
})

test('same-query refetch failures retain each successful value and timestamp; reads recover independently', async ({ page }) => {
  const state = await setup(page)
  state.controls.stats.json = [profit]
  state.controls.trades.json = [trade]
  await page.goto('/history')
  await apply(page)
  await expect(panel(page, 'stats').getByText('+10,000원', { exact: true })).toBeVisible()
  await expect(panel(page, 'trades').getByText(trade.symbol_name, { exact: true })).toBeVisible()
  const times = { stats: await panel(page, 'stats').locator('time').getAttribute('datetime'), trades: await panel(page, 'trades').locator('time').getAttribute('datetime') }
  for (const kind of ['stats', 'trades'] as const) {
    state.controls[kind] = { status: 503, json: { message: 'Synthetic refetch unavailable' } }
    await retry(page, kind).click()
    await expect(panel(page, kind)).toContainText('상태 재조회 실패', { timeout: 12000 })
    await expect(panel(page, kind)).toContainText('이전 자료: 이 적용 기간의 마지막 정상 조회 결과입니다.')
    await expect(panel(page, kind).locator('time')).toHaveAttribute('datetime', times[kind]!)
  }
  await expect(panel(page, 'stats').getByText('+10,000원', { exact: true })).toBeVisible()
  await expect(panel(page, 'trades').getByText(trade.symbol_name, { exact: true })).toBeVisible()
  const tradeReads = state.audit.counts.get(paths.trades)
  state.controls.stats = { status: 200, json: [zero] }
  await retry(page, 'stats').click()
  await expectZero(page)
  await expect(panel(page, 'stats')).not.toContainText('이전 자료:')
  await expect(panel(page, 'trades')).toContainText('상태 재조회 실패')
  expect(state.audit.counts.get(paths.trades)).toBe(tradeReads)
  state.controls.trades = { status: 200, json: [] }
  await retry(page, 'trades').click()
  await expect(panel(page, 'trades').getByText(emptyTrades, { exact: true })).toBeVisible()
  await expect(panel(page, 'trades')).not.toContainText('이전 자료:')
  safe(state.audit)
})

test('375px dark: unapplied dates keep old results and retries use the applied range until Search', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await page.emulateMedia({ colorScheme: 'dark' })
  const state = await setup(page)
  state.controls.stats.json = [profit]
  state.controls.trades.json = [trade]
  await page.goto('/history')
  await apply(page)
  await expect(panel(page, 'stats').getByText('+10,000원', { exact: true })).toBeVisible()
  await expect(panel(page, 'trades').getByText(trade.symbol_name, { exact: true })).toBeVisible()
  const beforeDraft = state.reads.length
  await dates(page, rangeB)
  await expect(page.getByTestId('history-unapplied-dates')).toBeVisible()
  for (const kind of ['stats', 'trades'] as const) await expect(page.getByTestId(`history-${kind}-status-range`)).toHaveText(`적용 기간: ${rangeA.from} ~ ${rangeA.to}`)
  expect(state.reads.length).toBe(beforeDraft)
  await repeatReads(page, state, 'stats')
  expect(state.reads.every(read => read.from === rangeA.from && read.to === rangeA.to)).toBe(true)
  const held = gate()
  state.controls.stats = { status: 200, json: [], hold: held.promise }
  state.controls.trades = { status: 200, json: [], hold: held.promise }
  try {
    await page.getByRole('button', { name: '조회', exact: true }).click()
    for (const kind of ['stats', 'trades'] as const) {
      await expect(page.getByTestId(`history-${kind}-status-range`)).toHaveText(`적용 기간: ${rangeB.from} ~ ${rangeB.to}`)
      await expect(panel(page, kind)).toContainText(`${labels[kind]} 확인 중`)
    }
    await noSummary(page)
    await expect(panel(page, 'trades').getByText(trade.symbol_name, { exact: true })).toHaveCount(0)
    held.release()
    await expectZero(page)
    await expect(panel(page, 'trades').getByText(emptyTrades, { exact: true })).toBeVisible()
    for (const kind of ['stats', 'trades'] as const) expect(state.reads.filter(read => read.kind === kind).at(-1)).toEqual({ kind, ...rangeB })
    await expect(page.getByTestId('history-unapplied-dates')).toHaveCount(0)
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
    await panel(page, 'stats').screenshot({ path: '.cache/uiux-implementation-20261030/history-narrow-stats.png', animations: 'disabled' })
    await panel(page, 'trades').screenshot({ path: '.cache/uiux-implementation-20261030/history-narrow-trades.png', animations: 'disabled' })
    safe(state.audit)
  } finally { held.release() }
})

test('summary arithmetic, KRW/USD rendering and provider trace remain unchanged for successful data', async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 1000 })
  const state = await setup(page)
  state.controls.stats.json = [profit, { ...zero, date: rangeA.to, net_profit: -500, total_trades: 1, win_rate: 1 }]
  const usd: TradeRecord = { ...trade, id: 'SYNTHETIC-USD', symbol: 'SYNTH', symbol_name: 'Synthetic USD trade', market: 'overseas', currency: 'USD', exchange: 'NAS', price: 10225, price_usd: 102.25, total_amount: 40900, total_amount_usd: 409, total_amount_krw: 600000, fee: 15, fee_usd: 0.15, fee_krw: 220, slippage: -5, slippage_bps: -2 }
  state.controls.trades.json = [trade, usd]
  await page.goto('/history')
  await apply(page)
  for (const value of ['+9,500원', '3건', '75.0%']) await expect(panel(page, 'stats').getByText(value, { exact: true })).toBeVisible()
  const krwRow = panel(page, 'trades').getByRole('row').filter({ hasText: trade.symbol_name })
  const usdRow = panel(page, 'trades').getByRole('row').filter({ hasText: usd.symbol_name })
  for (const value of ['11,000원', '22,000원', '11원', '+100원 (+5bps)']) await expect(krwRow.getByRole('cell', { name: value, exact: true })).toBeVisible()
  for (const value of ['$102.25', '$409.00 / 600,000원', '$0.15 / 220원', '$-0.05 (-2bps)']) await expect(usdRow.getByRole('cell', { name: value, exact: true })).toBeVisible()
  await expect(krwRow).toContainText('SYNTHETIC-TR')
  safe(state.audit)
})
