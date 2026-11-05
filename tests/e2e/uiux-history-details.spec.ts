import { chromium, expect, test, type BrowserContext, type Page } from '@playwright/test'
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import type { TradeRecord } from '../../src/api/types'
import { mockUx } from './fixtures/uxReviewFixture'

const krw: TradeRecord = {
  id: 'SYN-KRW-1', timestamp: '2026-09-01T10:00:01.123+09:00', symbol: '0005A0', symbol_name: '합성 동일 종목',
  side: 'buy', quantity: 2, price: 11000, total_amount: 22000, fee: 11,
  strategy_id: 'SYNTHETIC-STRATEGY-A', order_id: 'SYN-ORDER-1', status: 'filled', signal_reason: 'synthetic fixture',
  market: 'domestic', currency: 'KRW', slippage: 100, slippage_bps: 5, provider: 'kis', provider_order_id: 'SYN-ORDER-1', provider_request_id: 'SYN-REQUEST-1', provider_tr_id: 'SYN-TR-1',
}
const krwSecond: TradeRecord = { ...krw, id: 'SYN-KRW-2', timestamp: '2026-09-01T10:00:02.456+09:00', price: 12000, total_amount: 24000, fee: 12, strategy_id: null, order_id: '', provider: null, provider_order_id: null, provider_request_id: null, provider_tr_id: null, slippage: null, slippage_bps: null }
const usd: TradeRecord = {
  ...krw, id: `SYN-USD-${'LONG-ID-'.repeat(20)}`, timestamp: '2026-09-01T14:01:02.789Z', symbol: 'SYNTH', symbol_name: 'Synthetic USD trade',
  market: 'overseas', currency: 'USD', exchange: 'NAS', price: 10225, total_amount: 20450, fee: 15,
  total_amount_krw: 300000, fee_krw: 220, strategy_id: 'SYNTHETIC-USD-STRATEGY', slippage: -5, slippage_bps: -2,
  provider_order_id: null, order_id: 'SYN-LEGACY-ORDER', provider_request_id: `SYN-REQUEST-${'LONG-'.repeat(40)}`,
}
const usdExplicit: TradeRecord = { ...usd, id: 'SYN-USD-EXPLICIT', timestamp: '2026-09-01T14:02:03Z', price_usd: 103.5, total_amount_usd: 207, fee_usd: 0.2, total_amount_krw: null, fee_krw: null, slippage: 0, slippage_bps: 0, provider_request_id: null }
const records = [krw, krwSecond, usd, usdExplicit]
const expected = [
  { price: '11,000원', amount: '22,000원', fee: '11원', slippage: '+100원 (+5bps)', currency: 'KRW', strategy: 'SYNTHETIC-STRATEGY-A', order: 'SYN-ORDER-1', request: 'SYN-REQUEST-1' },
  { price: '12,000원', amount: '24,000원', fee: '12원', slippage: '-', currency: 'KRW', strategy: '-', order: '-', request: '-' },
  { price: '$102.25', amount: '$204.50 / 300,000원', fee: '$0.15 / 220원', slippage: '$-0.05 (-2bps)', currency: 'USD', strategy: 'SYNTHETIC-USD-STRATEGY', order: 'SYN-LEGACY-ORDER', request: usd.provider_request_id! },
  { price: '$103.50', amount: '$207.00', fee: '$0.20', slippage: '$0.00 (0bps)', currency: 'USD', strategy: 'SYNTHETIC-USD-STRATEGY', order: 'SYN-LEGACY-ORDER', request: '-' },
]
const trigger = (page: Page, record: TradeRecord) => page.getByRole('button', { name: `${record.symbol_name} (${record.symbol}) ${record.timestamp} 체결 ${record.id} 상세 보기`, exact: true })
const field = (page: Page, label: string) => page.getByRole('dialog').getByText(label, { exact: true }).locator('..').locator('dd')

async function setup(page: Page) {
  const audit = await mockUx(page)
  audit.responses.set('/api/stats', { json: [] })
  audit.responses.set('/api/trades', { json: records })
  return audit
}
async function search(page: Page, from = '2026-09-01', to = '2026-09-07') {
  await page.getByLabel('시작일', { exact: true }).fill(from)
  await page.getByLabel('종료일', { exact: true }).fill(to)
  await page.getByRole('button', { name: '조회', exact: true }).click()
  await expect(page.getByTestId('history-stats-panel')).toContainText('통계 조회 완료')
  await expect(trigger(page, krw)).toBeVisible()
}
function safe(audit: Awaited<ReturnType<typeof mockUx>>) {
  expect(audit.writes).toEqual([])
  expect(audit.external).toEqual([])
  expect(audit.unknown).toEqual([])
  expect(audit.errors).toEqual([])
}
async function readAllDetails(page: Page, evidence: string) {
  for (let index = 0; index < records.length; index++) {
    const record = records[index]
    const values = expected[index]
    const button = trigger(page, record)
    await page.getByTestId('history-trades-panel').getByRole('button', { name: '체결 목록 다시 확인' }).focus()
    for (let tabs = 0; tabs < 20 && !(await button.evaluate(element => document.activeElement === element)); tabs++) await page.keyboard.press('Tab')
    await expect(button).toBeFocused()
    await page.keyboard.press('Enter')
    const dialog = page.getByRole('dialog')
    await expect(dialog).toHaveAccessibleName(`거래 상세 — ${record.symbol_name} (${record.symbol})`)
    await expect(dialog.getByRole('button', { name: '닫기', exact: true })).toBeFocused()
    for (const [label, value] of Object.entries({ '체결 ID': record.id, '일시 (기록값)': record.timestamp, '종목': `${record.symbol_name} (${record.symbol})`, '구분': '매수', '수량': '2', '통화': values.currency, '가격': values.price, '금액': values.amount, '수수료': values.fee, '슬리피지': values.slippage, '전략': values.strategy, 'Provider': record.provider || '-', 'TR-ID': record.provider_tr_id || '-', 'Provider order ID': values.order, 'Provider request ID': values.request })) await expect(field(page, label)).toHaveText(value)
    const geometry = await dialog.evaluate(element => {
      const rect = element.getBoundingClientRect()
      const content = element.querySelector('[data-testid="trade-details-content"]')!
      return { fits: rect.left >= 0 && rect.right <= innerWidth + 1 && rect.top >= 0 && rect.bottom <= innerHeight + 1, contentFits: content.scrollWidth <= content.clientWidth + 1, documentFits: document.documentElement.scrollWidth <= innerWidth + 1 }
    })
    expect(geometry).toEqual({ fits: true, contentFits: true, documentFits: true })
    if (index === 2) await page.screenshot({ path: `.cache/uiux-implementation-20261030/history-detail-${evidence}-top.png`, animations: 'disabled' })
    await dialog.getByTestId('trade-details-content').focus()
    await page.keyboard.press('End')
    await expect(field(page, 'Provider request ID')).toBeInViewport()
    if (index === 2) await page.screenshot({ path: `.cache/uiux-implementation-20261030/history-detail-${evidence}-trace.png`, animations: 'disabled' })
    for (let tabs = 0; tabs < 3; tabs++) {
      await page.keyboard.press('Tab')
      expect(await dialog.evaluate(element => element.contains(document.activeElement))).toBe(true)
    }
    if (index === 1) await dialog.getByRole('button', { name: '닫기', exact: true }).press('Enter')
    else await page.keyboard.press('Escape')
    await expect(dialog).toHaveCount(0)
    await expect(button).toBeFocused()
  }
}

for (const width of [375, 1280]) {
  test(`History details: ${width}px exposes every original value and returns keyboard focus to the exact trade`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 })
    await page.emulateMedia({ colorScheme: width === 375 ? 'dark' : 'light' })
    const audit = await setup(page)
    await page.goto('/history')
    await search(page)
    const readsBefore = [audit.counts.get('/api/trades'), audit.counts.get('/api/stats')]
    await readAllDetails(page, String(width))
    expect([audit.counts.get('/api/trades'), audit.counts.get('/api/stats')]).toEqual(readsBefore)
    safe(audit)
  })
}

test('History details: an empty poll closes selection and the same id returning cannot reopen a dismissed record', async ({ page }) => {
  await page.clock.install({ time: new Date('2026-10-30T00:00:00Z') })
  const audit = await setup(page)
  await page.goto('/history')
  await search(page, '2026-10-30', '2026-10-30')
  await trigger(page, krw).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  audit.responses.set('/api/trades', { json: [] })
  await page.clock.fastForward(30_100)
  await expect(page.getByRole('dialog')).toHaveCount(0)
  audit.responses.set('/api/trades', { json: records })
  await page.clock.fastForward(30_100)
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await expect(trigger(page, krw)).toBeVisible()
  safe(audit)
})

test('History details: isolated Chromium tab browser zoom 200% performs CSS reflow and preserves every detail', async ({ baseURL }) => {
  const root = await mkdtemp(join(tmpdir(), 'kis-history-zoom-'))
  const extension = join(root, 'zoom-extension')
  let context: BrowserContext | undefined
  try {
    await mkdir(extension)
    // No tabs/host permissions, content scripts, network, external URLs or user profile access.
    await writeFile(join(extension, 'manifest.json'), JSON.stringify({ manifest_version: 3, name: 'Synthetic tab zoom helper', version: '1.0.0', background: { service_worker: 'worker.js' } }))
    await writeFile(join(extension, 'worker.js'), 'chrome.runtime.onInstalled.addListener(() => {});')
    context = await chromium.launchPersistentContext(join(root, 'isolated-profile'), { channel: 'chromium', headless: true, viewport: { width: 1280, height: 1000 }, args: [`--disable-extensions-except=${extension}`, `--load-extension=${extension}`] })
    const worker = context.serviceWorkers()[0] ?? await context.waitForEvent('serviceworker')
    const page = await context.newPage()
    const audit = await setup(page)
    await page.goto(`${baseURL}/history`)
    const before = await page.evaluate(() => ({ innerWidth, innerHeight, dpr: devicePixelRatio, visualWidth: visualViewport?.width, visualScale: visualViewport?.scale }))
    const zoom = await worker.evaluate(`(async () => {
      const [tab] = await chrome.tabs.query({active:true,currentWindow:true});
      await chrome.tabs.setZoomSettings(tab.id,{mode:'automatic',scope:'per-tab'});
      await chrome.tabs.setZoom(tab.id,2);
      return chrome.tabs.getZoom(tab.id);
    })()`)
    expect(zoom).toBe(2)
    await expect.poll(() => page.evaluate(() => innerWidth)).toBeCloseTo(before.innerWidth / 2, 0)
    const after = await page.evaluate(() => ({ innerWidth, innerHeight, dpr: devicePixelRatio, visualWidth: visualViewport?.width, visualScale: visualViewport?.scale, desktopSidebar: matchMedia('(min-width:900px)').matches }))
    console.log('Synthetic browser 200% zoom:', JSON.stringify({ zoom, before, after }))
    await search(page)
    await readAllDetails(page, 'browser-200-percent')
    safe(audit)
  } finally {
    await context?.close()
    await rm(root, { recursive: true, force: true })
  }
})
