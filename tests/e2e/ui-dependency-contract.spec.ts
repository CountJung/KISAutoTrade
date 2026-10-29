import { expect, test, type Locator, type Page, type TestInfo } from '@playwright/test'
import { mkdir } from 'node:fs/promises'
import { mockApi } from './fixtures/strategyResearchFixture'

// Production UI contracts only: all financial data and writes stay in mock routes.
async function setup(page: Page, broker: 'kis' | 'toss' = 'kis') {
  const errors: string[] = []
  const external: string[] = []
  const writes: Array<{ path: string; body: unknown }> = []
  page.on('pageerror', error => errors.push(error.message))
  page.on('console', message => {
    if (message.type() === 'warning' || message.type() === 'error') errors.push(message.text())
  })
  await page.route('**/*', route => {
    const url = new URL(route.request().url())
    if (['127.0.0.1', 'localhost'].includes(url.hostname)) return route.fallback()
    external.push(url.origin)
    return route.abort()
  })
  await mockApi(page, { activeBroker: broker })
  let interval = 24
  await page.route('**/api/**', async route => {
    const request = route.request()
    const path = new URL(request.url()).pathname
    // Do not capture Vite's /src/api modules when running this spec in dev mode.
    if (!path.startsWith('/api/')) return route.fallback()
    if (request.method() !== 'GET') writes.push({ path, body: request.postDataJSON() })
    if (path === '/api/profiles') return route.fulfill({ json: [] })
    if (path === '/api/stock-update-interval') {
      interval = request.postDataJSON().hours
      return route.fulfill({ json: null })
    }
    if (path === '/api/stock-list-stats') {
      return route.fulfill({ json: { count: 0, lastUpdatedAt: null, filePath: 'synthetic-fixture', updateIntervalHours: interval } })
    }
    if (path === '/api/trades') {
      return route.fulfill({ json: Array.from({ length: 31 }, (_, index) => ({
        id: `synthetic-${index}`, strategy_id: 'fixture', symbol: `TEST${index}`,
        symbol_name: `합성 종목 ${index}`, side: index % 2 ? 'sell' : 'buy', quantity: 1,
        price: 1000, total_amount: 1000, fee: 0, tax: 0, realized_pnl: 0,
        timestamp: `2026-10-29T10:${String(index).padStart(2, '0')}:00`, signal_reason: 'UI fixture',
      })) })
    }
    return route.fallback()
  })
  return { errors, external, writes }
}

async function capture(page: Page, info: TestInfo, name: string) {
  const directory = process.env.UI_MIGRATION_EVIDENCE ?? '.cache/react-mui-migration-20261029/screenshots/current'
  await mkdir(directory, { recursive: true })
  const image = await page.screenshot({ path: `${directory}/${name}.png`, animations: 'disabled' })
  await info.attach(name, { body: image, contentType: 'image/png' })
}

async function navigate(page: Page, name: 'Dashboard' | 'Settings' | 'Strategy') {
  // A closing temporary Drawer hides the background from the accessibility
  // tree. Its transient aria-hidden state must not imply a desktop viewport.
  const mobile = await page.getByRole('button', { name: '메뉴 열기', includeHidden: true }).isVisible()
  const menu = page.getByRole('button', { name: '메뉴 열기' })
  if (mobile) {
    await expect(menu).toBeVisible()
    await menu.click()
  }
  await page.getByRole('button', { name, exact: true }).click()
  const path = name === 'Dashboard' ? '/' : `/${name.toLowerCase()}`
  await expect(page).toHaveURL(new RegExp(`${path}$`))
  // Wait until the modal releases the route content before navigating again.
  if (mobile) await expect(menu).toBeVisible()
}

async function paintedChart(chart: Locator) {
  await expect(chart.locator('canvas').first()).toBeVisible()
  await expect.poll(() => chart.locator('canvas').evaluateAll(elements => elements.some(element => {
    const canvas = element as HTMLCanvasElement
    if (canvas.width === 0 || canvas.height === 0) return false
    const sample = document.createElement('canvas')
    sample.width = canvas.width
    sample.height = canvas.height
    const context = sample.getContext('2d', { willReadFrequently: true })
    if (!context) return false
    context.drawImage(canvas, 0, 0)
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data
    const colors = new Set<string>()
    // Sample rendered content, not a library class name or an exact screenshot.
    for (let index = 0; index < pixels.length; index += 64) {
      if (pixels[index + 3]) colors.add(`${pixels[index]},${pixels[index + 1]},${pixels[index + 2]}`)
      if (colors.size > 3) return true
    }
    return false
  }))).toBe(true)
}

for (const layout of [
  { name: 'desktop-light', width: 1280, height: 900, scheme: 'light' as const },
  { name: 'narrow-dark', width: 375, height: 812, scheme: 'dark' as const },
]) {
  test(`Settings profile dialog traps keyboard focus and restores its trigger: ${layout.name}`, async ({ page }, info) => {
    await page.setViewportSize({ width: layout.width, height: layout.height })
    await page.emulateMedia({ colorScheme: layout.scheme })
    const audit = await setup(page)
    await page.goto('/settings')
    const trigger = page.getByRole('button', { name: '한국투자증권 계좌 추가', exact: true })
    await trigger.scrollIntoViewIfNeeded()
    await trigger.focus()
    await page.keyboard.press('Enter')
    const dialog = page.getByRole('dialog', { name: '한국투자증권 계좌 프로파일 추가' })
    await expect(dialog).toBeVisible()
    await dialog.getByRole('textbox', { name: '프로파일 이름' }).fill('합성 UI 프로파일')
    for (let index = 0; index < 12; index += 1) {
      await page.keyboard.press('Tab')
      expect(await dialog.evaluate(element => element.contains(document.activeElement))).toBe(true)
    }
    const box = (await dialog.boundingBox())!
    expect(box.x).toBeGreaterThanOrEqual(0)
    expect(box.x + box.width).toBeLessThanOrEqual(layout.width + 1)
    await capture(page, info, `settings-dialog-${layout.name}`)
    await page.keyboard.press('Escape')
    await expect(dialog).toHaveCount(0)
    await expect(trigger).toBeFocused()
    expect(audit.writes).toEqual([])
    expect(audit.errors).toEqual([])
    expect(audit.external).toEqual([])
  })
}

test('Settings Select keyboard saves one mock change and Escape restores focus without saving', async ({ page }, info) => {
  const audit = await setup(page)
  await page.goto('/settings')
  const interval = page.getByRole('combobox').filter({ hasText: '매일 (24시간)' })
  await interval.scrollIntoViewIfNeeded()
  await interval.focus()
  await page.keyboard.press('Enter')
  await expect(page.getByRole('listbox')).toBeVisible()
  await page.keyboard.press('Home')
  await page.keyboard.press('ArrowDown')
  await expect(page.getByRole('option', { name: '6시간마다', exact: true })).toBeFocused()
  await page.keyboard.press('Enter')
  const updated = page.getByRole('combobox').filter({ hasText: '6시간마다' })
  await expect(updated).toBeFocused()
  await expect.poll(() => audit.writes).toEqual([{ path: '/api/stock-update-interval', body: { hours: 6 } }])
  await updated.press('Enter')
  await page.keyboard.press('End')
  await expect(page.getByRole('option', { name: '매주 (7일)' })).toBeFocused()
  await capture(page, info, 'settings-select-keyboard')
  await page.keyboard.press('Escape')
  await expect(page.getByRole('listbox')).toHaveCount(0)
  await expect(updated).toBeFocused()
  expect(audit.writes).toHaveLength(1)
  expect(audit.errors).toEqual([])
  expect(audit.external).toEqual([])
})

test('Dashboard Select keyboard changes pagination and returns focus to the select', async ({ page }, info) => {
  const audit = await setup(page)
  await page.goto('/')
  const size = page.getByRole('combobox', { name: '표시 건수' })
  await expect(page.getByText('1 / 2 페이지 · 31건')).toBeVisible()
  await size.focus()
  await size.press('Enter')
  await page.keyboard.press('End')
  await expect(page.getByRole('option', { name: '100건', exact: true })).toBeFocused()
  await page.keyboard.press('Enter')
  await expect(size).toContainText('100건')
  await expect(size).toBeFocused()
  await expect(page.getByText('1 / 1 페이지 · 31건')).toBeVisible()
  await page.setViewportSize({ width: 375, height: 812 })
  await size.scrollIntoViewIfNeeded()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
  await capture(page, info, 'dashboard-narrow-pagination')
  expect(audit.writes).toEqual([])
  expect(audit.errors).toEqual([])
  expect(audit.external).toEqual([])
})

test('Painted chart survives light/dark settings, narrow resize and SPA unmount/remount', async ({ page }, info) => {
  await page.emulateMedia({ colorScheme: 'light' })
  const audit = await setup(page, 'toss')
  await page.goto('/settings')
  await page.getByRole('button', { name: '라이트', exact: true }).click()
  const light = await page.locator('body').evaluate(element => getComputedStyle(element).backgroundColor)
  await navigate(page, 'Strategy')
  const card = page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' })
  const chart = card.getByTestId('lth-preview-chart')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await paintedChart(chart)
  await chart.scrollIntoViewIfNeeded()
  const wide = (await chart.boundingBox())!.width
  await capture(page, info, 'chart-light-painted')
  await navigate(page, 'Settings')
  await expect(page.getByTestId('lth-preview-chart')).toHaveCount(0)
  await page.getByRole('button', { name: '다크', exact: true }).click()
  await expect.poll(() => page.locator('body').evaluate(element => getComputedStyle(element).backgroundColor)).not.toBe(light)
  await navigate(page, 'Strategy')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await paintedChart(chart)
  await page.setViewportSize({ width: 375, height: 812 })
  await chart.scrollIntoViewIfNeeded()
  await expect.poll(async () => (await chart.boundingBox())!.width).toBeLessThan(wide)
  await paintedChart(chart)
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
  await capture(page, info, 'chart-dark-narrow-painted')
  await navigate(page, 'Dashboard')
  await expect(page.getByTestId('lth-preview-chart')).toHaveCount(0)
  await navigate(page, 'Strategy')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await paintedChart(chart)
  expect(audit.errors).toEqual([])
  expect(audit.external).toEqual([])
})
