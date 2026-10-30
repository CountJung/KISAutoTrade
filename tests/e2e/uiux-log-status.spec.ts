import { expect, test, type Page } from '@playwright/test'
import type { AppLogEntry } from '../../src/api/types'
import { mockUx } from './fixtures/uxReviewFixture'

const entries = (count: number): AppLogEntry[] => Array.from({ length: count }, (_, index) => ({ timestamp: `2026-10-30T10:${String(Math.floor(index / 60)).padStart(2, '0')}:${String(index % 60).padStart(2, '0')}`, level: index % 2 ? 'WARN' : 'INFO', target: 'synthetic-log', message: `Synthetic log row ${index}` }))
const traceRow: AppLogEntry = { timestamp: '2026-10-30T10:00:00', level: 'INFO', target: 'synthetic-trace', message: 'Synthetic trace provider=kis tr_id=SYN-TR odno=SYN-ORDER request_id=SYN-REQ' }
const panel = (page: Page) => page.getByTestId('log-resizable-panel')
const status = (page: Page) => page.getByTestId('log-query-status').getByRole('status')
const search = (page: Page) => page.getByRole('textbox', { name: '로그 검색', exact: true })
const retry = (page: Page) => page.getByRole('button', { name: '로그 다시 불러오기', exact: true })
const count = (page: Page) => page.getByTestId('log-result-count')
function gate() { let release!: () => void; const promise = new Promise<void>(resolve => { release = resolve }); return { promise, release } }

async function setup(page: Page, rows: AppLogEntry[] = []) {
  const audit = await mockUx(page)
  const control: { status: number; rows: AppLogEntry[]; hold?: Promise<void> } = { status: 200, rows }
  const requestedCounts: Array<string | null> = []
  audit.responses.set('/api/recent-logs', async route => {
    const request = route.request()
    const url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
    if (url.pathname !== '/api/recent-logs' || request.method() !== 'GET') return route.abort()
    requestedCounts.push(url.searchParams.get('count'))
    const response = { ...control }
    if (response.hold) await response.hold
    return route.fulfill({ status: response.status, json: response.status === 200 ? response.rows : { message: 'Synthetic log unavailable' } })
  })
  return { audit, control, requestedCounts }
}
function safe(state: Awaited<ReturnType<typeof setup>>) {
  expect(state.audit.writes).toEqual([])
  expect(state.audit.external).toEqual([])
  expect(state.audit.unknown).toEqual([])
  expect(state.audit.errors).toEqual([])
  expect(state.requestedCounts.every(value => value === '300')).toBe(true)
}
async function pauseClock(page: Page) {
  await page.clock.pauseAt(await page.evaluate(() => Date.now() + 100))
}
async function repeatReads(page: Page, state: Awaited<ReturnType<typeof setup>>) {
  // The clock is paused here: only user reads are counted, without changing poll policy.
  for (let repeat = 0; repeat < 2; repeat++) {
    const before = state.audit.counts.get('/api/recent-logs') ?? 0
    const response = page.waitForResponse(res => new URL(res.url()).pathname === '/api/recent-logs')
    await retry(page).focus()
    await page.keyboard.press('Enter')
    const completed = await response
    expect(completed.ok()).toBe(true)
    await completed.finished()
    await page.evaluate(() => Promise.resolve())
    await page.clock.runFor(1)
    await expect(retry(page)).toBeEnabled()
    await expect(status(page)).toHaveText('로그 조회 완료')
    expect(state.audit.counts.get('/api/recent-logs')).toBe(before + 1)
  }
}

test('Log: 375px dark placeholder delay and first failure never report a confirmed zero; keyboard retries recover', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 })
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.clock.install()
  const state = await setup(page)
  const held = gate()
  state.control.status = 500
  state.control.hold = held.promise
  try {
    await page.goto('/log')
    await expect(status(page)).toHaveText('로그 불러오는 중')
    await expect(count(page)).toHaveText('조회 미확인')
    await expect(page.getByTestId('log-checked-at')).toHaveText('마지막 정상 확인: 없음')
    await expect(panel(page).getByText('조회한 최근 로그가 없습니다.', { exact: true })).toHaveCount(0)
    await expect(search(page)).toBeVisible()
    held.release()
    await expect(status(page)).toHaveText('로그 불러오기 실패', { timeout: 12000 })
    await expect(count(page)).toHaveText('조회 미확인')
    await expect(panel(page)).toContainText('로그를 불러오지 못했습니다.')
    await page.screenshot({ path: '.cache/uiux-implementation-20261030/log-first-error-375.png', animations: 'disabled' })
    await pauseClock(page)
    state.control.status = 200
    state.control.rows = [traceRow]
    state.control.hold = undefined
    await repeatReads(page, state)
    await expect(count(page)).toHaveText('표시 1건 / 조회 1건')
    await expect(page.getByTestId('log-checked-at').locator('time')).toHaveAttribute('datetime', /T/)
    await expect(panel(page)).toContainText(traceRow.message)
    await expect(panel(page).getByText('Order SYN-ORDER', { exact: true })).toBeVisible()
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
    safe(state)
  } finally { held.release() }
})

test('Log: successful empty data is distinct from first failure and remains previous zero after refetch failure', async ({ page }) => {
  await page.clock.install()
  const state = await setup(page)
  await page.goto('/log')
  await expect(status(page)).toHaveText('로그 조회 완료')
  await expect(count(page)).toHaveText('표시 0건 / 조회 0건')
  await expect(panel(page)).toContainText('조회한 최근 로그가 없습니다.')
  const checkedAt = await page.getByTestId('log-checked-at').locator('time').getAttribute('datetime')
  state.control.status = 500
  await retry(page).click()
  await expect(status(page)).toHaveText('로그 재조회 실패 — 이전 자료 표시', { timeout: 12000 })
  await expect(count(page)).toHaveText('표시 0건 / 조회 0건')
  await expect(page.getByTestId('log-checked-at').locator('time')).toHaveAttribute('datetime', checkedAt!)
  await pauseClock(page)
  state.control.status = 200
  await repeatReads(page, state)
  await expect(panel(page)).toContainText('조회한 최근 로그가 없습니다.')
  safe(state)
})

test('Log: filter zero and repeated keyboard clearing remain separate from raw zero and search stays usable', async ({ page }) => {
  await page.clock.install()
  await pauseClock(page)
  const state = await setup(page, [traceRow, ...entries(3)])
  await page.goto('/log')
  await expect(count(page)).toHaveText('표시 4건 / 조회 4건')
  const before = state.audit.counts.get('/api/recent-logs')
  for (let repeat = 0; repeat < 2; repeat++) {
    await page.getByRole('button', { name: 'ERROR', exact: true }).press('Enter')
    await search(page).fill('no synthetic match')
    await expect(count(page)).toHaveText('표시 0건 / 조회 4건')
    await expect(panel(page)).toContainText('현재 필터와 일치하는 로그가 없습니다.')
    await expect(panel(page).getByText('조회한 최근 로그가 없습니다.', { exact: true })).toHaveCount(0)
    await search(page).press('Shift+Tab')
    const clear = page.getByRole('button', { name: '필터 지우기', exact: true })
    await expect(clear).toBeFocused()
    await page.keyboard.press('Enter')
    await expect(search(page)).toHaveValue('')
    await expect(page.getByRole('button', { name: 'ALL', exact: true })).toHaveAttribute('aria-pressed', 'true')
    await expect(count(page)).toHaveText('표시 4건 / 조회 4건')
    await page.keyboard.press('Tab')
    await expect(search(page)).toBeFocused()
    await search(page).fill('synthetic-trace')
    await expect(count(page)).toHaveText('표시 1건 / 조회 4건')
    await expect(panel(page)).toContainText(traceRow.message)
  }
  expect(state.audit.counts.get('/api/recent-logs')).toBe(before)
  safe(state)
})

test('Log: retained rows, success time and user scroll survive failure and recovery', async ({ page }) => {
  await page.clock.install()
  const state = await setup(page, entries(150))
  await page.goto('/log')
  await expect(count(page)).toHaveText('표시 150건 / 조회 150건')
  await panel(page).hover()
  await page.mouse.wheel(0, -10000)
  await expect.poll(() => panel(page).evaluate(element => element.scrollTop)).toBe(0)
  const checkedAt = await page.getByTestId('log-checked-at').locator('time').getAttribute('datetime')
  state.control.status = 500
  await retry(page).click()
  await expect(status(page)).toHaveText('로그 재조회 실패 — 이전 자료 표시', { timeout: 12000 })
  await expect(count(page)).toHaveText('표시 150건 / 조회 150건')
  await expect(page.getByTestId('log-checked-at').locator('time')).toHaveAttribute('datetime', checkedAt!)
  expect(await panel(page).evaluate(element => element.scrollTop)).toBe(0)
  await page.screenshot({ path: '.cache/uiux-implementation-20261030/log-previous-data-desktop.png', animations: 'disabled' })
  await pauseClock(page)
  state.control.status = 200
  state.control.rows = entries(160)
  await repeatReads(page, state)
  await expect(count(page)).toHaveText('표시 160건 / 조회 160건')
  expect(await panel(page).evaluate(element => element.scrollTop)).toBe(0)
  expect(await page.getByTestId('log-checked-at').locator('time').getAttribute('datetime')).not.toBe(checkedAt)
  safe(state)
})

test('Log: original five-second polling respects upward scroll and bottom follow without repeated live announcements', async ({ page }) => {
  await page.clock.install()
  await pauseClock(page)
  const state = await setup(page, entries(150))
  await page.goto('/log')
  await expect(count(page)).toHaveText('표시 150건 / 조회 150건')
  await expect.poll(() => panel(page).evaluate(element => element.scrollHeight - element.scrollTop - element.clientHeight)).toBeLessThan(48)
  await expect(panel(page).getByText('Synthetic log row 149', { exact: true })).toBeInViewport()
  await page.evaluate(() => {
    const live = document.querySelector('[data-testid="log-query-status"] [role="status"]')!
    const seen: string[] = []
    Object.assign(window, { syntheticLogAnnouncements: seen })
    new MutationObserver(() => seen.push(live.textContent ?? '')).observe(live, { childList: true, characterData: true, subtree: true })
  })
  await panel(page).hover()
  await page.mouse.wheel(0, -10000)
  await expect.poll(() => panel(page).evaluate(element => element.scrollTop)).toBe(0)
  const readsBefore = state.audit.counts.get('/api/recent-logs') ?? 0
  state.control.rows = entries(160)
  const firstPoll = page.waitForResponse(res => new URL(res.url()).pathname === '/api/recent-logs')
  await page.clock.fastForward(5000)
  await (await firstPoll).finished()
  await page.evaluate(() => Promise.resolve())
  await page.clock.runFor(1)
  await expect(count(page)).toHaveText('표시 160건 / 조회 160건')
  expect(state.audit.counts.get('/api/recent-logs')).toBe(readsBefore + 1)
  expect(await panel(page).evaluate(element => element.scrollTop)).toBe(0)
  await panel(page).hover()
  await page.mouse.wheel(0, 10000)
  await expect.poll(() => panel(page).evaluate(element => element.scrollHeight - element.scrollTop - element.clientHeight)).toBeLessThan(48)
  state.control.rows = entries(170)
  const secondPoll = page.waitForResponse(res => new URL(res.url()).pathname === '/api/recent-logs')
  await page.clock.fastForward(5000)
  await (await secondPoll).finished()
  await page.evaluate(() => Promise.resolve())
  await page.clock.runFor(1)
  await expect(count(page)).toHaveText('표시 170건 / 조회 170건')
  await expect.poll(() => panel(page).evaluate(element => element.scrollHeight - element.scrollTop - element.clientHeight)).toBeLessThan(48)
  await expect(panel(page).getByText('Synthetic log row 169', { exact: true })).toBeInViewport()
  expect(state.audit.counts.get('/api/recent-logs')).toBe(readsBefore + 2)
  const announcements = await page.evaluate(() => Reflect.get(window, 'syntheticLogAnnouncements'))
  expect(announcements).toEqual([])
  await expect(status(page)).toHaveText('로그 조회 완료')
  safe(state)
})
