import { expect, test, type Locator, type Page } from '@playwright/test'
import { mockUx } from './fixtures/uxReviewFixture'

const sidebarKey = 'act:panel:sidebar:width'
const logKey = 'act:panel:log:height'
const unrelatedKey = 'synthetic:unchanged-preference'
const sidebar = (page: Page) => page.getByRole('separator', { name: '사이드바 너비 조절', exact: true })
const logHandle = (page: Page) => page.getByRole('separator', { name: '로그 패널 높이 조절', exact: true })

async function setup(page: Page, width = 220, height = 480) {
  await page.setViewportSize({ width: 1440, height: 1400 })
  await page.addInitScript(({ sidebarKey, logKey, unrelatedKey, width, height }) => {
    if (localStorage.getItem('synthetic:resizer-seeded')) return
    localStorage.setItem(sidebarKey, String(width))
    localStorage.setItem(logKey, String(height))
    localStorage.setItem(unrelatedKey, 'keep-this-value')
    localStorage.setItem('synthetic:resizer-seeded', '1')
  }, { sidebarKey, logKey, unrelatedKey, width, height })
  const audit = await mockUx(page)
  audit.responses.set('/api/recent-logs', { json: Array.from({ length: 40 }, (_, index) => ({ timestamp: `2026-10-30T10:00:${String(index).padStart(2, '0')}`, level: 'INFO', target: 'synthetic-resizer', message: `Synthetic log row ${index}` })) })
  return audit
}
function safe(audit: Awaited<ReturnType<typeof mockUx>>) {
  expect(audit.writes).toEqual([])
  expect(audit.external).toEqual([])
  expect(audit.unknown).toEqual([])
  expect(audit.errors).toEqual([])
}
async function value(page: Page, handle: Locator, key: string, expected: number) {
  await expect(handle).toHaveAttribute('aria-valuenow', String(expected))
  await expect(handle).toHaveAttribute('aria-valuetext', `${expected}픽셀`)
  await expect.poll(() => page.evaluate(key => localStorage.getItem(key), key)).toBe(String(expected))
}
async function otherPreferences(page: Page, key: string, expected: number) {
  expect(await page.evaluate(key => localStorage.getItem(key), key)).toBe(String(expected))
  expect(await page.evaluate(key => localStorage.getItem(key), unrelatedKey)).toBe('keep-this-value')
}
async function metadata(handle: Locator, orientation: 'vertical' | 'horizontal', min: number, max: number, initial: number) {
  await expect(handle).toHaveAttribute('tabindex', '0')
  await expect(handle).toHaveAttribute('aria-orientation', orientation)
  await expect(handle).toHaveAttribute('aria-valuemin', String(min))
  await expect(handle).toHaveAttribute('aria-valuemax', String(max))
  await expect(handle).toHaveAttribute('aria-valuenow', String(initial))
  await expect(handle).toHaveAccessibleDescription(new RegExp(`Home: 최소 ${min}픽셀.*End: 최대 ${max}픽셀.*Enter: 기본`))
}
async function focusRing(handle: Locator) {
  await expect(handle).toBeFocused()
  expect(await handle.evaluate(element => [getComputedStyle(element).outlineStyle, getComputedStyle(element).outlineWidth])).toEqual(['solid', '2px'])
}
async function drag(page: Page, handle: Locator, dx: number, dy: number) {
  const box = (await handle.boundingBox())!
  expect(box).toBeTruthy()
  const x = box.x + box.width / 2
  const y = box.y + box.height / 2
  await page.mouse.move(x, y)
  await page.mouse.down()
  await page.mouse.move(x + dx, y + dy, { steps: 6 })
  await page.mouse.up()
  expect(await page.evaluate(() => [document.body.style.cursor, document.body.style.userSelect])).toEqual(['', ''])
}

test('P2-UX-02: sidebar keyboard values, clamps, default restore and new-value persistence survive route/reload', async ({ page }) => {
  const audit = await setup(page, 220, 360)
  await page.goto('/')
  const handle = sidebar(page)
  await metadata(handle, 'vertical', 160, 400, 220)
  await page.getByRole('navigation').getByRole('button', { name: 'Settings', exact: true }).focus()
  await page.keyboard.press('Tab')
  await focusRing(handle)
  for (const next of [230, 240, 250]) {
    await page.keyboard.press('ArrowRight')
    await value(page, handle, sidebarKey, next)
  }
  await page.keyboard.press('ArrowLeft')
  await value(page, handle, sidebarKey, 240)
  await page.keyboard.press('ArrowUp')
  await value(page, handle, sidebarKey, 240)
  await page.keyboard.press('Home')
  await value(page, handle, sidebarKey, 160)
  for (let i = 0; i < 2; i++) await page.keyboard.press('ArrowLeft')
  await value(page, handle, sidebarKey, 160)
  await page.keyboard.press('End')
  await value(page, handle, sidebarKey, 400)
  for (let i = 0; i < 2; i++) await page.keyboard.press('ArrowRight')
  await value(page, handle, sidebarKey, 400)
  await page.keyboard.press('Enter')
  await value(page, handle, sidebarKey, 220)
  await page.keyboard.press('ArrowRight')
  await value(page, handle, sidebarKey, 230)
  expect((await page.getByRole('navigation').boundingBox())!.width).toBe(230)
  await page.screenshot({ path: '.cache/uiux-implementation-20261030/resizer-sidebar-focus.png', animations: 'disabled' })
  await otherPreferences(page, logKey, 360)
  await page.getByRole('navigation').getByRole('button', { name: 'Log', exact: true }).press('Enter')
  await expect(page.getByRole('heading', { name: 'Log', exact: true })).toBeVisible()
  await value(page, handle, sidebarKey, 230)
  await page.getByRole('navigation').getByRole('button', { name: 'Dashboard', exact: true }).press('Enter')
  await value(page, handle, sidebarKey, 230)
  await page.reload()
  await value(page, sidebar(page), sidebarKey, 230)
  await otherPreferences(page, logKey, 360)
  safe(audit)
})

test('P2-UX-02: log keyboard values, clamps, default restore and new-value persistence survive route/reload', async ({ page }) => {
  const audit = await setup(page, 270, 480)
  await page.setViewportSize({ width: 1440, height: 900 })
  await page.goto('/log')
  const handle = logHandle(page)
  await metadata(handle, 'horizontal', 160, 900, 480)
  await page.getByPlaceholder('검색...').focus()
  await page.keyboard.press('Tab')
  await expect(page.getByTestId('log-resizable-panel')).toBeFocused()
  await page.keyboard.press('Tab')
  await focusRing(handle)
  for (const next of [490, 500, 510]) {
    await page.keyboard.press('ArrowDown')
    await value(page, handle, logKey, next)
  }
  await page.keyboard.press('ArrowUp')
  await value(page, handle, logKey, 500)
  await page.keyboard.press('ArrowLeft')
  await value(page, handle, logKey, 500)
  await page.keyboard.press('Home')
  await value(page, handle, logKey, 160)
  for (let i = 0; i < 2; i++) await page.keyboard.press('ArrowUp')
  await value(page, handle, logKey, 160)
  await page.keyboard.press('End')
  await value(page, handle, logKey, 900)
  const endViewport = await handle.evaluate(element => {
    const rect = element.getBoundingClientRect()
    return { top: rect.top, bottom: rect.bottom, height: rect.height, viewportHeight: innerHeight, intersects: rect.top < innerHeight && rect.bottom > 0 }
  })
  const endPanel = await page.getByTestId('log-resizable-panel').evaluate(element => ({ rectHeight: element.getBoundingClientRect().height, computedHeight: getComputedStyle(element).height, clientHeight: element.clientHeight }))
  console.log('Synthetic Log End viewport:', JSON.stringify(endViewport))
  console.log('Synthetic Log End panel geometry:', JSON.stringify({ ariaValue: await handle.getAttribute('aria-valuenow'), ...endPanel }))
  await page.screenshot({ path: '.cache/uiux-implementation-20261030/resizer-log-end.png', animations: 'disabled' })
  expect(endViewport.intersects).toBe(true)
  for (let i = 0; i < 2; i++) await page.keyboard.press('ArrowDown')
  await value(page, handle, logKey, 900)
  await page.keyboard.press('Enter')
  await value(page, handle, logKey, 480)
  await page.keyboard.press('ArrowDown')
  await value(page, handle, logKey, 490)
  expect((await page.getByTestId('log-resizable-panel').boundingBox())!.height).toBe(490)
  await page.screenshot({ path: '.cache/uiux-implementation-20261030/resizer-log-focus.png', animations: 'disabled' })
  await otherPreferences(page, sidebarKey, 270)
  await page.getByRole('navigation').getByRole('button', { name: 'Dashboard', exact: true }).press('Enter')
  await expect(page.getByRole('heading', { name: 'Dashboard', exact: true })).toBeVisible()
  await page.getByRole('navigation').getByRole('button', { name: 'Log', exact: true }).press('Enter')
  await value(page, logHandle(page), logKey, 490)
  await page.reload()
  await value(page, logHandle(page), logKey, 490)
  await otherPreferences(page, sidebarKey, 270)
  safe(audit)
})

test('P2-UX-02: sidebar pointer drag retains delta, bounds and saved width', async ({ page }) => {
  const audit = await setup(page, 220, 360)
  await page.goto('/')
  await metadata(sidebar(page), 'vertical', 160, 400, 220)
  await drag(page, sidebar(page), 60, 0)
  await value(page, sidebar(page), sidebarKey, 280)
  await drag(page, sidebar(page), 300, 0)
  await value(page, sidebar(page), sidebarKey, 400)
  await drag(page, sidebar(page), -350, 0)
  await value(page, sidebar(page), sidebarKey, 160)
  await page.reload()
  await value(page, sidebar(page), sidebarKey, 160)
  await otherPreferences(page, logKey, 360)
  safe(audit)
})

test('P2-UX-02: log pointer drag retains delta, bounds and saved height', async ({ page }) => {
  const audit = await setup(page, 270, 480)
  await page.goto('/log')
  await metadata(logHandle(page), 'horizontal', 160, 900, 480)
  await drag(page, logHandle(page), 0, 60)
  await value(page, logHandle(page), logKey, 540)
  await drag(page, logHandle(page), 0, 600)
  await value(page, logHandle(page), logKey, 900)
  await drag(page, logHandle(page), 0, -800)
  await value(page, logHandle(page), logKey, 160)
  await page.reload()
  await value(page, logHandle(page), logKey, 160)
  await otherPreferences(page, sidebarKey, 270)
  safe(audit)
})

test('P2-UX-02: existing responsive handle visibility is preserved and the medium-width log handle stays keyboard reachable', async ({ page }) => {
  const audit = await setup(page)
  await page.setViewportSize({ width: 375, height: 900 })
  await page.goto('/log')
  await expect(page.getByRole('heading', { name: 'Log', exact: true })).toBeVisible()
  await expect(sidebar(page)).toHaveCount(0)
  await expect(logHandle(page)).toHaveCount(0)
  await page.setViewportSize({ width: 800, height: 1000 })
  await expect(sidebar(page)).toHaveCount(0)
  await expect(logHandle(page)).toBeVisible()
  await page.getByPlaceholder('검색...').focus()
  await page.keyboard.press('Tab')
  await expect(page.getByTestId('log-resizable-panel')).toBeFocused()
  await page.keyboard.press('Tab')
  await focusRing(logHandle(page))
  await page.keyboard.press('ArrowDown')
  await value(page, logHandle(page), logKey, 490)
  await page.setViewportSize({ width: 1440, height: 1400 })
  await expect(sidebar(page)).toBeVisible()
  await value(page, logHandle(page), logKey, 490)
  await otherPreferences(page, sidebarKey, 220)
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
  safe(audit)
})
