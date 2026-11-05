import { expect, test, type Page } from '@playwright/test'
import { mockUx } from './fixtures/uxReviewFixture'
const summaries = (page: Page) => [page.getByTestId('sidebar-trading-status'), page.getByTestId('dashboard-trading-status')]

test('status loading and first failure are distinct from a confirmed stop; keyboard retry recovers', async ({ page }) => {
  const audit = await mockUx(page)
  let fail = false
  let release!: () => void
  const gate = new Promise<void>(resolve => { release = resolve })
  audit.responses.set('/api/trading/status', async route => {
    await gate
    if (fail) return route.fulfill({ status: 503, json: { message: 'Synthetic unavailable' } })
    return route.fallback()
  })
  try {
    await page.goto('/')
    for (const summary of summaries(page)) await expect(summary).toContainText('\uC2E4\uD589 \uC0C1\uD0DC \uD655\uC778 \uC911')
    fail = true
    release()
    for (const summary of summaries(page)) await expect(summary).toContainText('\uC2E4\uD589 \uC0C1\uD0DC \uD655\uC778 \uC2E4\uD328', { timeout: 12000 })
    await page.screenshot({ path: '.cache/uiux-implementation-20261030/status-first-error.png', animations: 'disabled' })
    fail = false
    const retry = summaries(page)[0].getByRole('button', { name: '\uC2E4\uD589 \uC0C1\uD0DC \uB2E4\uC2DC \uD655\uC778' })
    await retry.focus()
    await page.keyboard.press('Enter')
    for (const summary of summaries(page)) await expect(summary).toContainText('\uC790\uB3D9\uB9E4\uB9E4 \uC815\uC9C0 \uC0C1\uD0DC')
    expect(audit.writes).toEqual([])
    expect(audit.external).toEqual([])
    expect(audit.errors).toEqual([])
  } finally { release() }
})

for (const running of [false, true]) {
  test(`previous ${running ? 'running' : 'stopped'} status survives refetch failure and repeated read retries`, async ({ page }) => {
    const audit = await mockUx(page, { tradingStatusOverride: () => ({ isRunning: running }) })
    await page.goto('/')
    const state = running ? '\uC790\uB3D9\uB9E4\uB9E4 \uC2E4\uD589 \uC911' : '\uC790\uB3D9\uB9E4\uB9E4 \uC815\uC9C0 \uC0C1\uD0DC'
    for (const summary of summaries(page)) {
      await expect(summary).toContainText(state)
      await expect(summary).toContainText('\uB9C8\uC9C0\uB9C9 \uC815\uC0C1 \uD655\uC778:')
    }
    audit.responses.set('/api/trading/status', { status: 503, json: { message: 'Synthetic unavailable' } })
    await summaries(page)[0].getByRole('button', { name: '\uC2E4\uD589 \uC0C1\uD0DC \uB2E4\uC2DC \uD655\uC778' }).click()
    for (const summary of summaries(page)) {
      await expect(summary).toContainText('\uC0C1\uD0DC \uC7AC\uC870\uD68C \uC2E4\uD328', { timeout: 12000 })
      await expect(summary).toContainText(state)
      await expect(summary).toContainText('\uC774\uC804 \uC0C1\uD0DC')
    }
    audit.responses.delete('/api/trading/status')
    for (let i = 0; i < 2; i++) {
      await summaries(page)[1].getByRole('button', { name: '\uC2E4\uD589 \uC0C1\uD0DC \uB2E4\uC2DC \uD655\uC778' }).click()
      for (const summary of summaries(page)) await expect(summary).not.toContainText('\uC0C1\uD0DC \uC7AC\uC870\uD68C \uC2E4\uD328')
    }
    expect(audit.writes).toEqual([])
    expect(audit.external).toEqual([])
    expect(audit.errors).toEqual([])
  })
}

test('unknown status wraps at narrow width and remains reachable in the mobile menu', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await page.emulateMedia({ colorScheme: 'dark' })
  const audit = await mockUx(page)
  audit.responses.set('/api/trading/status', { status: 503, json: { message: 'Synthetic unavailable' } })
  await page.goto('/')
  await expect(page.getByTestId('dashboard-trading-status')).toContainText('\uC2E4\uD589 \uC0C1\uD0DC \uD655\uC778 \uC2E4\uD328', { timeout: 12000 })
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
  await page.getByRole('button', { name: '\uBA54\uB274 \uC5F4\uAE30' }).click()
  const summary = page.getByTestId('sidebar-trading-status')
  await expect(summary).toBeVisible()
  await expect(summary).toContainText('\uC2E4\uD589 \uC0C1\uD0DC \uD655\uC778 \uC2E4\uD328')
  await page.screenshot({ path: '.cache/uiux-implementation-20261030/status-narrow-error.png', animations: 'disabled' })
  expect(audit.writes).toEqual([])
  expect(audit.external).toEqual([])
  expect(audit.errors).toEqual([])
})

test('previous status and retry remain inside the minimum 160px sidebar', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 })
  await page.addInitScript(() => localStorage.setItem('act:panel:sidebar:width', '160'))
  const audit = await mockUx(page, { tradingStatusOverride: () => ({ isRunning: true }) })
  await page.goto('/')
  const summary = page.getByTestId('sidebar-trading-status')
  await expect(summary).toContainText('\uC790\uB3D9\uB9E4\uB9E4 \uC2E4\uD589 \uC911')
  audit.responses.set('/api/trading/status', { status: 503, json: { message: 'Synthetic unavailable' } })
  await summary.getByRole('button', { name: '\uC2E4\uD589 \uC0C1\uD0DC \uB2E4\uC2DC \uD655\uC778' }).click()
  await expect(summary).toContainText('\uC0C1\uD0DC \uC7AC\uC870\uD68C \uC2E4\uD328', { timeout: 12000 })
  const nav = (await page.getByRole('navigation').boundingBox())!
  const box = (await summary.boundingBox())!
  expect(box.x).toBeGreaterThanOrEqual(nav.x)
  expect(box.x + box.width).toBeLessThanOrEqual(nav.x + nav.width + 1)
  await expect(summary.getByRole('button', { name: '\uC2E4\uD589 \uC0C1\uD0DC \uB2E4\uC2DC \uD655\uC778' })).toBeVisible()
  await page.screenshot({ path: '.cache/uiux-implementation-20261030/status-minimum-sidebar.png', animations: 'disabled' })
  expect(audit.writes).toEqual([])
  expect(audit.external).toEqual([])
  expect(audit.errors).toEqual([])
})
