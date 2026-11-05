import { expect, test, type Locator, type Page } from '@playwright/test'
import { mkdir, writeFile } from 'node:fs/promises'
import type { ChartCandle } from '../../src/api/types'
import { mockUx } from './fixtures/uxReviewFixture'

test.use({ timezoneId: 'UTC' })
const domestic = (page: Page) => page.getByRole('textbox', { name: '6자리 종목코드 (예: 005930, 0005A0)' })
const data = (page: Page) => page.getByTestId('candle-data-summary')
const chart = (page: Page) => data(page).locator('..')
const stateText = (page: Page) => data(page).getByTestId('candle-query-state')
const latest = (page: Page) => data(page).getByTestId('candle-latest')
const selected = (page: Page) => data(page).getByTestId('candle-selected')
const daily: ChartCandle[] = Array.from({ length: 22 }, (_, index) => ({ date: `202610${String(index + 1).padStart(2, '0')}`,
  open: String(10000 + index * 100), high: String(10050 + index * 100), low: String(9950 + index * 100), close: String(10025 + index * 100), volume: String(1000 + index + 0.125) }))
const usd: ChartCandle[] = [20, 21, 22].map((day, index) => ({ date: `202610${day}`, open: String(100.125 + index), high: String(101.75 + index), low: String(99.875 + index), close: String(100.625 + index), volume: String(2000 + index) }))
const minutes: ChartCandle[] = [0, 1, 2].map((minute, index) => ({ date: `2026103009${String(minute).padStart(2, '0')}00`, open: String(300.125 + index), high: String(301.75 + index), low: String(299.875 + index), close: String(300.625 + index), volume: String(30 + index) }))
function gate() { let release!: () => void; const promise = new Promise<void>(resolve => { release = resolve }); return { promise, release } }
type Control = { rows: ChartCandle[]; status: number; hold?: Promise<void> }
async function setup(page: Page, broker: 'kis' | 'toss' = 'kis') {
  const audit = await mockUx(page, { activeBroker: broker })
  const currency = { allocatedAmount: 0, cashAmount: 0, reservedAmount: 0, availableAmount: 0, ownedPositionCount: 0, blockedReason: null, feeBufferBps: 100 }
  audit.responses.set('/api/auto-trading-budget', { json: { scope: { brokerId: 'kis', accountId: '12345678-01' }, krw: currency, usd: currency } })
  const controls = new Map<string, Control>()
  const reads: Array<{ path: string; method: string; query: Record<string, string> }> = []
  const readPosts: string[] = [], consoleMessages: string[] = []
  page.on('console', message => { if (['warning', 'error'].includes(message.type())) consoleMessages.push(message.text()) })
  for (const [path, rows] of [
    ['/api/chart/005930', daily], ['/api/chart/000660', usd], ['/api/overseas-chart/NAS/AAPL', usd],
    // Synthetic transition only: the existing KR symbol briefly survives the market change.
    ['/api/overseas-chart/NAS/005930', []], ['/api/toss-chart/005930', daily], ['/api/toss-chart/AAPL', minutes],
  ] as const) {
    const control: Control = { rows: [...rows], status: 200 }
    controls.set(path, control)
    audit.responses.set(path, async route => {
      const request = route.request(), url = new URL(request.url())
      if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
      if (url.pathname !== path || request.method() !== 'GET') return route.abort()
      reads.push({ path, method: request.method(), query: Object.fromEntries(url.searchParams) })
      const response = { ...control }
      if (response.hold) await response.hold
      return route.fulfill({ status: response.status, json: response.status === 200 ? (path === '/api/toss-chart/AAPL' && url.searchParams.get('interval') === '1d' ? usd : response.rows) : { message: 'Synthetic chart unavailable' } })
    })
  }
  for (const symbol of ['005930', '000660']) audit.responses.set(`/api/price/${symbol}`, { json: { stck_prpr: '12125', hts_kor_isnm: `합성 ${symbol}`, prdy_vrss: '0', prdy_ctrt: '0', acml_vol: '2345' } })
  for (const symbol of ['AAPL', '005930']) audit.responses.set(`/api/overseas-price/NAS/${symbol}`, { json: { last: '102.625', name: symbol === 'AAPL' ? 'Synthetic USD' : 'Synthetic transition only', diff: '0', rate: '0', tvol: '2345' } })
  for (const symbol of ['005930', 'AAPL']) {
    audit.responses.set(`/api/toss-market-snapshot/${symbol}`, { json: {
      brokerId: 'toss', market: symbol === 'AAPL' ? 'us' : 'kr', price: { amount: '302.625', currency: symbol === 'AAPL' ? 'USD' : 'KRW' },
      orderbook: { asks: [], bids: [], currency: 'KRW' }, trades: [], priceLimits: { upperLimitPrice: null, lowerLimitPrice: null, currency: 'KRW' },
    } })
    audit.responses.set(`/api/toss-stock-safety/${symbol}`, { json: { brokerId: 'toss', symbol, stockInfo: null, warnings: [], hasBlockingWarning: false, buyBlocked: false, buyBlockReason: null } })
  }
  if (broker === 'toss') await page.route('**/api/toss-open-orders', async route => {
    const request = route.request(), url = new URL(request.url())
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) { audit.external.push(url.origin); return route.abort() }
    if (url.pathname !== '/api/toss-open-orders' || request.method() !== 'POST') return route.fallback()
    readPosts.push(url.pathname)
    return route.fulfill({ json: [] }) // Existing semantic read, never an order/write.
  })
  return { audit, controls, reads, readPosts, consoleMessages }
}
async function safe(s: Awaited<ReturnType<typeof setup>>) {
  expect(s.audit.writes).toEqual([])
  expect(s.audit.external).toEqual([])
  expect(s.audit.unknown).toEqual([])
  expect(s.audit.errors).toEqual([])
  expect(s.consoleMessages.filter(message => !/Failed to load resource: the server responded with a status of 503/.test(message))).toEqual([])
  expect(s.reads.every(read => read.method === 'GET' && (read.path.includes('toss-chart')
    ? read.query.count === '200' && ['1d', '1m'].includes(read.query.interval)
    : read.query.count === '100' && ['D', 'W', 'M'].includes(read.query.period)))).toBe(true)
  const directory = '.cache/uiux-implementation-20261030/round9-api-audit'
  await mkdir(directory, { recursive: true })
  await writeFile(`${directory}/${test.info().title.replace(/[^a-zA-Z0-9-]/g, '_')}.json`, JSON.stringify({ reads: s.reads,
    counts: Object.fromEntries(s.audit.counts), readPosts: s.readPosts, writes: s.audit.writes, external: s.audit.external,
    unknown: s.audit.unknown, pageErrors: s.audit.errors, consoleMessages: s.consoleMessages }, null, 2))
}
async function enter(page: Page, market: 'KR' | 'US', symbol = market === 'KR' ? '005930' : 'AAPL') {
  if (market === 'KR') await domestic(page).fill(symbol)
  else {
    await page.getByRole('button', { name: '미국', exact: true }).click()
    const ticker = page.getByRole('textbox', { name: '티커 (AAPL, TSLA …)' })
    await ticker.fill(symbol)
    await ticker.press('Enter')
  }
}
async function openTable(page: Page) {
  const summary = data(page).locator('summary')
  await summary.focus()
  await page.keyboard.press('Enter')
  await expect(data(page).getByRole('table')).toBeVisible()
}
async function paintMetrics(root: Locator) {
  return root.locator('canvas').evaluateAll(elements => elements.map(element => {
    const canvas = element as HTMLCanvasElement
    const sample = document.createElement('canvas')
    sample.width = canvas.width; sample.height = canvas.height
    const context = sample.getContext('2d', { willReadFrequently: true })
    if (!context || !canvas.width || !canvas.height) return { width: canvas.width, height: canvas.height, chromaticPixels: 0 }
    context.drawImage(canvas, 0, 0)
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data
    let chromaticPixels = 0
    for (let index = 0; index < pixels.length; index += 4) {
      const range = Math.max(pixels[index], pixels[index + 1], pixels[index + 2]) - Math.min(pixels[index], pixels[index + 1], pixels[index + 2])
      if (pixels[index + 3] > 20 && range > 40) chromaticPixels++
    }
    return { width: canvas.width, height: canvas.height, chromaticPixels }
  }))
}
const paints = new Map<string, Awaited<ReturnType<typeof paintMetrics>>[]>()
async function painted(root: Locator) {
  await expect(root.locator('canvas').first()).toBeVisible()
  await expect.poll(async () => (await paintMetrics(root)).reduce((sum, canvas) => sum + canvas.chromaticPixels, 0)).toBeGreaterThan(100)
  const evidence = paints.get(test.info().title) ?? []
  evidence.push(await paintMetrics(root)); paints.set(test.info().title, evidence)
  const directory = '.cache/uiux-implementation-20261030/round9-paint-audit'
  await mkdir(directory, { recursive: true })
  await writeFile(`${directory}/${test.info().title.replace(/[^a-zA-Z0-9-]/g, '_')}.json`, JSON.stringify(evidence, null, 2))
}
async function capture(page: Page, name: string) {
  await mkdir('.cache/uiux-implementation-20261030/round9-screenshots', { recursive: true })
  await page.screenshot({ path: `.cache/uiux-implementation-20261030/round9-screenshots/${name}.png` })
}
async function refresh(page: Page, path: string) {
  const response = page.waitForResponse(res => new URL(res.url()).pathname === path)
  await page.clock.fastForward(61000)
  await page.evaluate(() => window.dispatchEvent(new Event('visibilitychange')))
  return response
}

test('Chart KRW: normalized values, 20-row bound, period Arrow/Enter and row Arrow/Home/End match the painted canvas', async ({ page }) => {
  const s = await setup(page)
  s.controls.get('/api/chart/005930')!.rows.push({ ...daily[0], date: '', close: '999999' }, { ...daily[0], date: '20261023', open: '0', close: '999999' })
  await page.goto('/trading'); await enter(page, 'KR')
  await expect(latest(page)).toContainText('2026-10-22')
  await expect(data(page)).toContainText('1개월 설정 · 일봉')
  await expect(data(page)).toContainText('가격: KRW')
  await expect(latest(page)).toContainText('시가 12,100 · 고가 12,150 · 저가 12,050 · 종가 12,125 · 거래량 1,021.125')
  await expect(data(page)).toContainText('22개')
  await painted(chart(page))
  await chart(page).getByRole('button', { name: '일', exact: true }).focus()
  await page.keyboard.press('ArrowRight')
  await expect(chart(page).getByRole('button', { name: '3월', exact: true })).toBeFocused()
  await page.keyboard.press('Enter')
  await expect(data(page)).toContainText('3개월 설정 · 일봉')
  await openTable(page)
  const rows = data(page).getByRole('table').getByRole('row')
  await expect(rows).toHaveCount(21)
  await expect(rows.nth(1)).toContainText('2026-10-03')
  const first = data(page).getByRole('button', { name: '2026-10-03 봉 선택', exact: true })
  await first.focus(); await page.keyboard.press('Enter')
  await expect(selected(page)).toContainText('종가 10,225')
  await page.keyboard.press('ArrowDown')
  await expect(data(page).getByRole('button', { name: '2026-10-04 봉 선택', exact: true })).toBeFocused()
  await page.keyboard.press('End')
  await expect(selected(page)).toHaveText((await latest(page).innerText()).replace('최신 봉:', '선택 봉:'))
  await page.keyboard.press('ArrowDown')
  await expect(data(page).getByRole('button', { name: '2026-10-22 봉 선택', exact: true })).toBeFocused()
  await page.keyboard.press('Home'); await page.keyboard.press('ArrowUp')
  await expect(first).toBeFocused()
  await capture(page, 'krw-table-light')
  await safe(s)
})

test('Chart USD: overseas date filtering and unscaled decimal values agree between latest, selected row and canvas', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' })
  const s = await setup(page)
  s.controls.get('/api/overseas-chart/NAS/AAPL')!.rows.push({ ...usd[0], date: '20261022103000', close: '999999' })
  await page.goto('/trading'); await enter(page, 'US')
  await expect(data(page)).toContainText('선택 프리셋: 3월 · 일봉')
  await expect(data(page)).toContainText('가격: USD')
  await expect(latest(page)).toContainText('시가 102.125 · 고가 103.75 · 저가 101.875 · 종가 102.625 · 거래량 2,002')
  await expect(data(page)).toContainText('3개')
  await painted(chart(page))
  await openTable(page)
  const row = data(page).getByRole('row').filter({ has: page.getByRole('button', { name: '2026-10-22 봉 선택', exact: true }) })
  await expect(row.getByRole('cell')).toHaveText(['102.125', '103.75', '101.875', '102.625', '2,002'])
  await row.getByRole('button').focus(); await page.keyboard.press('Enter')
  await expect(selected(page)).toHaveText((await latest(page).innerText()).replace('최신 봉:', '선택 봉:'))
  await chart(page).getByRole('button', { name: '캔들 차트', exact: true }).focus()
  await page.keyboard.press('ArrowRight')
  await expect(chart(page).getByRole('button', { name: '라인 차트', exact: true })).toBeFocused()
  await page.keyboard.press('Enter')
  await expect(chart(page).getByRole('button', { name: '라인 차트', exact: true })).toHaveAttribute('aria-pressed', 'true')
  for (const name of ['확대', '축소', '전체 맞춤']) { await chart(page).getByRole('button', { name, exact: true }).focus(); await page.keyboard.press('Enter') }
  await painted(chart(page)); await capture(page, 'usd-table-dark')
  await safe(s)
})

test('Chart Toss US: selected market stays distinct from unknown candle currency and minute time reuses the existing normalized instant', async ({ page }) => {
  const s = await setup(page, 'toss')
  await page.goto('/trading'); await enter(page, 'US')
  await chart(page).getByRole('button', { name: '1분', exact: true }).focus(); await page.keyboard.press('Enter')
  await expect(data(page)).toContainText('선택 시장: 미국 · 1분봉 · 요청 최대 200봉')
  await expect(data(page)).toContainText('가격: 통화 미확인')
  await expect(data(page)).toContainText('차트 응답에는 가격 통화 정보가 없습니다')
  await expect(latest(page)).toContainText('종가 302.625')
  await expect(latest(page).locator('time')).toHaveAttribute('datetime', '2026-10-30T00:02:00.000Z')
  await painted(chart(page)); await openTable(page)
  await data(page).getByRole('button', { name: /봉 선택$/ }).first().focus(); await page.keyboard.press('Enter')
  await page.keyboard.press('End')
  await expect(selected(page).locator('time')).toHaveAttribute('datetime', '2026-10-30T00:02:00.000Z')
  await expect(selected(page)).toContainText('종가 302.625')
  expect(s.reads.filter(read => read.path === '/api/toss-chart/AAPL').some(read => read.query.interval === '1m' && read.query.count === '200')).toBe(true)
  expect(s.reads.filter(read => !read.path.includes('toss-chart'))).toEqual([])
  await safe(s)
})

test('Chart status: initial delay, failure, successful empty and filtered-out data never claim a zero-price candle', async ({ page }) => {
  test.setTimeout(45000)
  const s = await setup(page), held = gate(), control = s.controls.get('/api/chart/005930')!
  control.hold = held.promise; control.status = 503
  try {
    await page.goto('/trading'); await enter(page, 'KR')
    await expect(stateText(page)).toHaveText('차트 자료를 불러오는 중…')
    await expect(latest(page)).toHaveCount(0)
    held.release()
    await expect(stateText(page)).toHaveText('차트 조회 실패', { timeout: 15000 })
    await expect(chart(page).getByRole('alert')).toContainText('Synthetic chart unavailable')
    control.hold = undefined; control.status = 200; control.rows = []
    await chart(page).getByRole('button', { name: '3월', exact: true }).click()
    await expect(stateText(page)).toHaveText('표시할 유효한 봉이 없습니다')
    await expect(latest(page)).toHaveCount(0)
    await expect.poll(async () => (await paintMetrics(chart(page))).reduce((sum, canvas) => sum + canvas.chromaticPixels, 0)).toBe(0)
    await writeFile('.cache/uiux-implementation-20261030/round9-empty-paint-control.json', JSON.stringify(await paintMetrics(chart(page)), null, 2))
    await openTable(page); await expect(data(page).getByRole('row')).toHaveCount(1)
    control.rows = [{ ...daily[0], open: '0' }]
    await chart(page).getByRole('button', { name: '6월', exact: true }).click()
    await expect(stateText(page)).toHaveText('표시할 유효한 봉이 없습니다')
    await expect(latest(page)).toHaveCount(0)
    await safe(s)
  } finally { held.release() }
})

test('Chart refresh: previous confirmed values stay labelled during refresh/failure, new projection clears selection without live announcements', async ({ page }) => {
  await page.clock.install()
  const s = await setup(page), control = s.controls.get('/api/chart/005930')!
  await page.goto('/trading'); await enter(page, 'KR')
  await expect(latest(page)).toContainText('12,125')
  await openTable(page)
  await data(page).getByRole('button', { name: /봉 선택$/ }).first().click()
  await expect(selected(page)).toContainText('10,225')
  const successTime = await data(page).getByText('마지막 조회 성공:', { exact: false }).locator('time').getAttribute('datetime')
  const held = gate(); control.hold = held.promise; control.status = 503
  try {
    const response = refresh(page, '/api/chart/005930')
    await expect(stateText(page)).toHaveText('갱신 중 · 이전 확인 자료')
    await expect(latest(page)).toContainText('12,125')
    held.release(); await (await response).finished()
    await expect(stateText(page)).toHaveText('재조회 실패 · 이전 확인 자료', { timeout: 15000 })
    await expect(data(page).getByText('마지막 조회 성공:', { exact: false }).locator('time')).toHaveAttribute('datetime', successTime!)
    await expect(latest(page)).toContainText('12,125')
    control.hold = undefined; control.status = 200
    control.rows = [...daily, { ...daily[21], date: '20261023', close: '12345.5', volume: '2222.75' }]
    await (await refresh(page, '/api/chart/005930')).finished()
    await expect(stateText(page)).toHaveText('차트 자료 확인 완료')
    await expect(latest(page)).toContainText('종가 12,345.5 · 거래량 2,222.75')
    await expect(selected(page)).toHaveText('선택 봉: 표에서 봉을 선택하세요.')
    await expect(data(page)).toHaveAttribute('aria-live', 'off')
    await expect(data(page).locator('[role="status"], [role="alert"], [aria-live="polite"], [aria-live="assertive"]')).toHaveCount(0)
    await safe(s)
  } finally { held.release() }
})

test('Chart scope: symbol and period changes hide old selected/latest data while the new request is delayed', async ({ page }) => {
  const s = await setup(page)
  await page.goto('/trading'); await enter(page, 'KR')
  await expect(latest(page)).toContainText('12,125'); await openTable(page)
  await data(page).getByRole('button', { name: /봉 선택$/ }).first().click()
  const held = gate(), periodHeld = gate(), control = s.controls.get('/api/chart/000660')!
  control.hold = held.promise
  try {
    await domestic(page).fill('000660')
    await expect(data(page)).toHaveAccessibleName('000660 차트 자료')
    await expect(stateText(page)).toHaveText('차트 자료를 불러오는 중…')
    await expect(latest(page)).toHaveCount(0)
    held.release(); await expect(latest(page)).toContainText('102.625')
    control.hold = periodHeld.promise
    await chart(page).getByRole('button', { name: '6월', exact: true }).focus(); await page.keyboard.press('Enter')
    await expect(data(page)).toContainText('6개월 설정 · 주봉')
    await expect(latest(page)).toHaveCount(0)
    await openTable(page); await expect(selected(page)).toHaveText('선택 봉: 표에서 봉을 선택하세요.')
    await expect(data(page).getByRole('button', { name: /봉 선택$/ })).toHaveCount(0)
    await safe(s)
  } finally { held.release(); periodHeld.release() }
})

for (const market of ['KR', 'US'] as const) test(`Chart lifecycle ${market}: own painted canvas survives theme, narrow resize and unmount/remount with an accessible table`, async ({ page }) => {
  const s = await setup(page)
  await page.goto('/settings'); await page.getByRole('button', { name: '라이트', exact: true }).click()
  await page.getByRole('button', { name: 'Trading', exact: true }).click(); await enter(page, market)
  await expect(latest(page)).toBeVisible(); await painted(chart(page))
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(data(page)).toHaveCount(0)
  await page.getByRole('button', { name: '다크', exact: true }).click()
  await page.getByRole('button', { name: 'Trading', exact: true }).click(); await enter(page, market)
  await expect(latest(page)).toBeVisible(); await painted(chart(page))
  const wideCanvas = await chart(page).locator('canvas').first().evaluate(element => ({ backingWidth: (element as HTMLCanvasElement).width, cssWidth: element.getBoundingClientRect().width }))
  await page.setViewportSize({ width: 375, height: 900 })
  await expect.poll(() => chart(page).locator('canvas').first().evaluate(element => (element as HTMLCanvasElement).width)).toBeLessThan(wideCanvas.backingWidth)
  const narrowCanvas = await chart(page).locator('canvas').first().evaluate(element => ({ backingWidth: (element as HTMLCanvasElement).width, cssWidth: element.getBoundingClientRect().width }))
  expect(narrowCanvas.cssWidth).toBeLessThan(wideCanvas.cssWidth)
  await writeFile(`.cache/uiux-implementation-20261030/round9-resize-${market}.json`, JSON.stringify({ viewport: 375, wideCanvas, narrowCanvas }, null, 2))
  await painted(chart(page)); await openTable(page)
  const scroller = data(page).getByLabel('최근 봉 표 가로 스크롤', { exact: true })
  await scroller.focus()
  const horizontalBefore = await scroller.evaluate(element => element.scrollLeft)
  const maxHorizontal = await scroller.evaluate(element => element.scrollWidth - element.clientWidth)
  expect(maxHorizontal).toBeGreaterThan(0)
  for (let step = 0; step < 24; step++) {
    const previous = await scroller.evaluate(element => element.scrollLeft)
    if (previous >= maxHorizontal - 1) break
    await page.keyboard.press('ArrowRight')
    await expect.poll(() => scroller.evaluate(element => element.scrollLeft)).toBeGreaterThan(previous)
  }
  const horizontalAfter = await scroller.evaluate(element => element.scrollLeft)
  expect(horizontalAfter).toBeGreaterThan(horizontalBefore)
  expect(horizontalAfter).toBeGreaterThanOrEqual(maxHorizontal - 1)
  const rightmostHeader = data(page).getByRole('columnheader', { name: '거래량', exact: true })
  const headerRect = (await rightmostHeader.boundingBox())!, scrollRect = (await scroller.boundingBox())!
  expect(headerRect.x).toBeGreaterThanOrEqual(scrollRect.x)
  expect(headerRect.x + headerRect.width).toBeLessThanOrEqual(scrollRect.x + scrollRect.width + 1)
  const visibleRightValues = await data(page).locator('tbody tr td:last-child').evaluateAll(elements => {
    const viewport = document.querySelector('[data-testid="app-main-scroll"]')!.getBoundingClientRect()
    return elements.filter(element => {
      const rect = element.getBoundingClientRect()
      return rect.top >= viewport.top && rect.bottom <= viewport.bottom && rect.left >= 0 && rect.right <= innerWidth
    }).map(element => element.textContent)
  })
  expect(visibleRightValues.length).toBeGreaterThan(0)
  await writeFile(`.cache/uiux-implementation-20261030/round9-horizontal-${market}.json`, JSON.stringify({ horizontalBefore, horizontalAfter, maxHorizontal, headerRect, scrollRect, visibleRightValues }, null, 2))
  await capture(page, `narrow-dark-${market}-right-columns`)
  await expect(scroller).toBeFocused()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
  expect(await page.getByTestId('app-main-scroll').evaluate(element => element.scrollWidth <= element.clientWidth + 1)).toBe(true)
  await data(page).getByRole('heading').scrollIntoViewIfNeeded()
  await capture(page, `narrow-dark-${market}-summary-table`)
  await safe(s)
})
