import { expect, test, type Page } from '@playwright/test'
import { mkdir, writeFile } from 'node:fs/promises'
import { mockUx } from './fixtures/uxReviewFixture'

// Synthetic, read-only navigation; no native shell or real settings service runs.
const sections = [
  ['settings-theme', '테마'], ['settings-refresh', '데이터 갱신 주기'], ['settings-logs', '로그 설정'],
  ['settings-archive', '체결 기록 보관'], ['settings-database', '데이터베이스 및 데이터 이관'],
  ['settings-rate-limit', 'Broker 요청 상태 (rate limit)'], ['settings-active-profile', '활성 증권사 프로파일'],
  ['settings-kis-profiles', '한국투자증권 계좌 프로파일'], ['settings-toss-profiles', '토스증권 계좌 프로파일'],
  ['settings-web', '웹 접속 설정'], ['settings-stocks', '종목 목록 관리'], ['settings-budget', '자동매매 전용 예산'],
  ['settings-recovery', '응답 불명 주문 대조·복구'], ['settings-risk', '리스크 관리'], ['settings-discord', 'Discord 알림'],
] as const
const risk = { enabled: true, dailyLossLimit: 500000, maxPositionRatio: 0.2, maxDailyBuyOrdersPerSymbol: 0,
  maxDailySellOrdersPerSymbol: 1, maxConsecutiveLossesPerStrategySymbol: 3, volatilitySizingEnabled: false,
  riskPerTradeBps: 100, atrStopMultiplier: 2, currentLoss: 0, dailyProfit: 0, netLoss: 0,
  emergencyStop: false, canTrade: true, lossRatio: 0, blockedStrategySymbolCount: 0, atrReadySymbolCount: 0 }
const currency = { allocatedAmount: 0, cashAmount: 0, reservedAmount: 0, availableAmount: 0, ownedPositionCount: 0, blockedReason: null, feeBufferBps: 100 }
const toc = (page: Page) => page.getByRole('navigation', { name: '설정 목차', exact: true })
const main = (page: Page) => page.getByTestId('app-main-scroll')
function gate() { let release!: () => void; const promise = new Promise<void>(resolve => { release = resolve }); return { promise, release } }
async function setup(page: Page) {
  const audit = await mockUx(page)
  const consoleMessages: string[] = []
  page.on('console', message => { if (['warning', 'error'].includes(message.type())) consoleMessages.push(message.text()) })
  const reads: Array<{ path: string; method: string; query: Record<string, string> }> = []
  const control: { status: number; hold?: Promise<void> } = { status: 200 }
  for (const path of ['/api/risk-config', '/api/auto-trading-budget']) {
    audit.responses.set(path, async route => {
      const request = route.request(), url = new URL(request.url())
      if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
      if (url.pathname !== path || request.method() !== 'GET') return route.abort()
      reads.push({ path, method: request.method(), query: Object.fromEntries(url.searchParams) })
      if (path === '/api/risk-config') {
        const response = { ...control }
        if (response.hold) await response.hold
        return route.fulfill({ status: response.status, json: response.status === 200 ? risk : { message: 'Synthetic risk unavailable' } })
      }
      return route.fulfill({ json: { scope: { brokerId: 'kis', accountId: '12345678-01' }, krw: currency, usd: currency } })
    })
  }
  return { audit, reads, control, consoleMessages }
}
async function safe(state: Awaited<ReturnType<typeof setup>>) {
  expect(state.audit.writes).toEqual([])
  expect(state.audit.external).toEqual([])
  expect(state.audit.unknown).toEqual([])
  expect(state.audit.errors).toEqual([])
  expect(state.reads.filter(read => read.path === '/api/auto-trading-budget').every(read =>
    read.method === 'GET' && JSON.stringify(read.query) === JSON.stringify({ brokerId: 'kis', brokerAccountId: '12345678-01' }))).toBe(true)
  expect(state.reads.some(read => read.path === '/api/auto-trading-budget')).toBe(true)
  expect(state.audit.counts.get('/api/budget-recovery') ?? 0).toBe(0)
  expect(state.audit.counts.get('/api/database-config') ?? 0).toBe(0)
  expect(state.consoleMessages.filter(message => !/Failed to load resource: the server responded with a status of 500/.test(message))).toEqual([])
  const directory = '.cache/uiux-implementation-20261030/round8-api-audit'
  await mkdir(directory, { recursive: true })
  await writeFile(`${directory}/${test.info().title.replace(/[^a-zA-Z0-9-]/g, '_')}.json`, JSON.stringify({
    reads: state.reads, counts: Object.fromEntries(state.audit.counts), writes: state.audit.writes,
    external: state.audit.external, unknown: state.audit.unknown, pageErrors: state.audit.errors, consoleMessages: state.consoleMessages,
  }, null, 2))
}
async function destination(page: Page, id: string, label: string) {
  const link = toc(page).getByRole('link', { name: label, exact: true })
  await link.focus()
  await page.keyboard.press('Enter')
  const heading = page.getByRole('heading', { name: label, exact: true })
  await expect(heading).toHaveAttribute('id', id)
  await expect(heading).toBeFocused()
  await expect.poll(() => heading.evaluate(element => {
    const rect = element.getBoundingClientRect(), container = document.querySelector('[data-testid="app-main-scroll"]')!.getBoundingClientRect()
    return rect.top >= container.top && rect.bottom <= container.bottom && rect.left >= 0 && rect.right <= innerWidth
  })).toBe(true)
  await expect.poll(() => heading.evaluate(element => getComputedStyle(element).outlineStyle)).toBe('solid')
  expect(new URL(page.url()).hash).toBe('')
  return heading
}
async function returnToContents(page: Page, label: string) {
  await page.getByRole('link', { name: `${label}에서 설정 목차로`, exact: true }).focus()
  await page.keyboard.press('Enter')
  await expect(page.getByRole('heading', { name: '설정 목차', exact: true })).toBeFocused()
  await expect.poll(() => toc(page).evaluate(element => {
    const heading = element.querySelector('h2')!.getBoundingClientRect()
    const link = element.querySelector('a')!.getBoundingClientRect()
    const container = document.querySelector('[data-testid="app-main-scroll"]')!.getBoundingClientRect()
    return heading.top >= container.top && heading.bottom <= container.bottom && link.top >= container.top && link.bottom <= container.bottom
  })).toBe(true)
}
async function screenshot(page: Page, name: string) {
  const directory = '.cache/uiux-implementation-20261030/round8-screenshots'
  await mkdir(directory, { recursive: true })
  await page.screenshot({ path: `${directory}/${name}.png` })
}

for (const layout of [{ name: 'desktop', width: 1280, dark: false }, { name: 'narrow-dark', width: 375, dark: true }]) {
  test(`Settings navigation: all stable destinations and keyboard return, ${layout.name}`, async ({ page }) => {
    await page.setViewportSize({ width: layout.width, height: 900 })
    await page.emulateMedia({ colorScheme: layout.dark ? 'dark' : 'light' })
    const state = await setup(page)
    await page.goto('/settings')
    await expect(page.getByRole('button', { name: '자동매매 예산 저장', exact: true })).toBeDisabled()
    await expect(page.getByRole('heading', { level: 1, name: 'Settings', exact: true })).toBeVisible()
    await expect(toc(page).getByRole('link')).toHaveCount(15)
    await expect(toc(page)).toContainText('조작 완료 시 저장')
    await toc(page).getByRole('link').first().focus()
    for (let index = 0; index < sections.length; index++) {
      await expect(toc(page).getByRole('link', { name: sections[index][1], exact: true })).toBeFocused()
      if (index < sections.length - 1) await page.keyboard.press('Tab')
    }
    for (const [id, label] of sections) {
      const heading = await destination(page, id, label)
      expect(await heading.evaluate(element => element.tagName)).toBe(id === 'settings-recovery' ? 'H3' : 'H2')
      if (id === 'settings-budget') await screenshot(page, `${layout.name}-budget`)
      await returnToContents(page, label)
    }
    await page.keyboard.press('Tab')
    await expect(toc(page).getByRole('link', { name: '테마', exact: true })).toBeFocused()
    await screenshot(page, `${layout.name}-contents`)
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
    expect(await main(page).evaluate(element => element.scrollWidth <= element.clientWidth + 1)).toBe(true)
    await safe(state)
  })
}

test('Settings navigation: delayed and failed risk keep a real focus destination and error visibility', async ({ page }) => {
  const state = await setup(page), held = gate()
  state.control.hold = held.promise
  state.control.status = 500
  state.audit.responses.set('/api/broker-rate-limit-status', { status: 500, json: { message: 'Synthetic rate limit unavailable' } })
  try {
    await page.goto('/settings')
    await destination(page, 'settings-risk', '리스크 관리')
    await expect(page.getByText('리스크 설정을 불러오는 중…', { exact: true })).toBeVisible()
    held.release()
    await expect(page.getByText('리스크 설정을 불러오지 못했습니다.', { exact: true })).toBeVisible()
    await expect(page.getByRole('heading', { name: '리스크 관리', exact: true })).toBeFocused()
    await returnToContents(page, '리스크 관리')
    await destination(page, 'settings-rate-limit', 'Broker 요청 상태 (rate limit)')
    await expect(page.getByText('rate limit 상태를 불러오지 못했습니다.', { exact: true })).toBeVisible()
    await returnToContents(page, 'Broker 요청 상태 (rate limit)')
    await destination(page, 'settings-database', '데이터베이스 및 데이터 이관')
    await expect(page.getByText(/DB password와 테이블 삭제 기능은 보안을 위해/)).toBeVisible()
    await expect(page.getByLabel('Password')).toHaveCount(0)
    await safe(state)
  } finally { held.release() }
})

test('Settings navigation: unsaved budget values and existing save gates survive local jumps without writes', async ({ page }) => {
  const state = await setup(page)
  await page.goto('/settings')
  await destination(page, 'settings-budget', '자동매매 전용 예산')
  const amount = page.getByRole('textbox', { name: '자동매매 원화 배정액 (원)', exact: true })
  await amount.fill('100000')
  const save = page.getByRole('button', { name: '자동매매 예산 저장', exact: true })
  await expect(save).toBeEnabled()
  await returnToContents(page, '자동매매 전용 예산')
  await destination(page, 'settings-recovery', '응답 불명 주문 대조·복구')
  await expect(page.getByText('불명 주문 복구는 Tauri 데스크톱 앱에서만 사용할 수 있습니다.', { exact: true })).toBeVisible()
  await returnToContents(page, '응답 불명 주문 대조·복구')
  await destination(page, 'settings-budget', '자동매매 전용 예산')
  await expect(amount).toHaveValue('100000')
  await expect(save).toBeEnabled()
  await safe(state)
})

test('Settings navigation: route back restores its scroll and reload starts at the original route top', async ({ page }) => {
  const state = await setup(page)
  await page.goto('/settings')
  await expect(page.getByRole('button', { name: '자동매매 예산 저장', exact: true })).toBeDisabled()
  await destination(page, 'settings-risk', '리스크 관리')
  const top = await main(page).evaluate(element => element.scrollTop)
  await page.getByRole('button', { name: 'History', exact: true }).click()
  await expect(page).toHaveURL(/\/history$/)
  await page.goBack()
  await expect(page).toHaveURL(/\/settings$/)
  await expect.poll(() => main(page).evaluate(element => element.scrollTop)).toBeCloseTo(top, 0)
  await returnToContents(page, '리스크 관리')
  await destination(page, 'settings-risk', '리스크 관리')
  const held = gate()
  state.control.hold = held.promise
  try {
    await page.reload()
    await expect(toc(page)).toBeVisible()
    await expect.poll(() => main(page).evaluate(element => element.scrollTop)).toBe(0)
    await destination(page, 'settings-risk', '리스크 관리')
    await expect(page.getByText('리스크 설정을 불러오는 중…', { exact: true })).toBeVisible()
    held.release()
    await expect(page.getByText('리스크 설정을 불러오는 중…', { exact: true })).toHaveCount(0)
    await expect(page.getByRole('heading', { name: '리스크 관리', exact: true })).toBeFocused()
    await returnToContents(page, '리스크 관리')
    await safe(state)
  } finally { held.release() }
})


test('Settings navigation: delayed route-back restoration rejects other-route cancellation, with a same-route control and two focus returns', async ({ page }) => {
  await page.clock.install()
  const state = await setup(page)
  await page.goto('/settings')
  await expect(page.getByRole('region', { name: '리스크 관리', exact: true }).getByRole('switch').first()).toBeVisible()
  // Isolate AppShell's restore from browser scroll anchoring in both comparison cycles.
  await main(page).evaluate(element => { element.style.overflowAnchor = 'none' })
  await page.clock.pauseAt(await page.evaluate(() => Date.now() + 100))
  const measurements: Array<Record<string, string | number>> = []
  for (const eventRoute of ['/history', '/settings']) {
    const held = gate()
    try {
      await destination(page, 'settings-discord', 'Discord 알림')
      await page.clock.runFor(32)
      const stored = await main(page).evaluate(element => ({ top: element.scrollTop, height: element.scrollHeight }))
      await page.getByRole('button', { name: 'History', exact: true }).click()
      await expect(page).toHaveURL(/\/history$/)
      state.control.hold = held.promise
      await page.clock.fastForward(300001)
      await page.goBack()
      await expect(page).toHaveURL(/\/settings$/)
      await page.clock.runFor(32)
      await expect(page.getByText('리스크 설정을 불러오는 중…', { exact: true })).toBeVisible()
      const delayed = await main(page).evaluate(element => ({ top: element.scrollTop, height: element.scrollHeight }))
      expect(delayed.top).toBeLessThan(stored.top - 100)
      await page.evaluate(routeKey => window.dispatchEvent(new CustomEvent('act:page-focus-navigation', { detail: { routeKey } })), eventRoute)
      const response = page.waitForResponse(res => new URL(res.url()).pathname === '/api/risk-config')
      held.release()
      await (await response).finished()
      await page.evaluate(() => Promise.resolve())
      await page.clock.runFor(1)
      await expect(page.getByText('리스크 설정을 불러오는 중…', { exact: true })).toHaveCount(0)
      await expect.poll(async () => { await page.clock.runFor(32); return main(page).evaluate(element => element.scrollHeight) }).toBe(stored.height)
      if (eventRoute === '/history') {
        // A wrong-route event must leave the pending target intact until height recovers.
        await expect.poll(async () => { await page.clock.runFor(32); return main(page).evaluate(element => element.scrollTop) }).toBeCloseTo(stored.top, 0)
      } else {
        // Counterfactual control: cancelling that pending target must make this observation differ.
        await page.clock.runFor(64)
        expect(await main(page).evaluate(element => element.scrollTop)).toBeCloseTo(delayed.top, 0)
        expect(await main(page).evaluate(element => element.scrollTop)).toBeLessThan(stored.top - 100)
      }
      const recovered = await main(page).evaluate(element => ({ top: element.scrollTop, height: element.scrollHeight }))
      measurements.push({ eventRoute, storedTop: stored.top, delayedTop: delayed.top, recoveredTop: recovered.top,
        storedHeight: stored.height, delayedHeight: delayed.height, recoveredHeight: recovered.height })
      state.control.hold = undefined
    } finally { held.release() }
  }
  await writeFile('.cache/uiux-implementation-20261030/round8-route-guard-control.json', JSON.stringify(measurements, null, 2))
  for (let repeat = 0; repeat < 2; repeat++) {
    const held = gate()
    try {
      await destination(page, 'settings-discord', 'Discord 알림')
      await page.clock.runFor(32) // Flush the original scroll/animation-frame position observer.
      await page.getByRole('button', { name: 'History', exact: true }).click()
      await expect(page).toHaveURL(/\/history$/)
      state.control.hold = held.promise
      // Existing cache default GC elapses away from Settings, without changing its query policy.
      await page.clock.fastForward(300001)
      await page.goBack()
      await expect(page).toHaveURL(/\/settings$/)
      await page.clock.runFor(32)
      await expect(page.getByText('리스크 설정을 불러오는 중…', { exact: true })).toBeVisible()
      const pendingPosition = await main(page).evaluate(element => element.scrollTop)
      expect(pendingPosition).toBeGreaterThan(1000)
      await returnToContents(page, 'Discord 알림')
      await page.clock.runFor(200)
      await expect(page.getByRole('heading', { name: '설정 목차', exact: true })).toBeFocused()
      expect(await main(page).evaluate(element => element.scrollTop)).toBeLessThan(100)
      const response = page.waitForResponse(res => new URL(res.url()).pathname === '/api/risk-config')
      held.release()
      await (await response).finished()
      await page.evaluate(() => Promise.resolve())
      await page.clock.runFor(1)
      await expect(page.getByText('리스크 설정을 불러오는 중…', { exact: true })).toHaveCount(0)
      state.control.hold = undefined
      await safe(state)
    } finally { held.release() }
  }
})
