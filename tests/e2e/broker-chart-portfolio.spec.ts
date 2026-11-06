import { readFile } from 'node:fs/promises'
import { expect, test, type Page } from '@playwright/test'
import type { BrokerChartInput } from '../../src/shared/api/brokerChartTypes'
import { syntheticBrokerChartInput } from '../../src/widgets/sector-portfolio/model/brokerChartInput'
import { brokerChartFixtureReport } from './fixtures/brokerChartFixture'
import { fixtureReport } from './fixtures/sectorPortfolioFixture'
import { mockApi } from './fixtures/strategyResearchFixture'

const limitAck = '배당 포함 총수익·권리 정산 미검증인 가격 근사 한계를 확인했습니다'
async function setup(page: Page, options: { research?: boolean; wait?: Promise<void>; validateFixture?: boolean } = {}) {
  const calls: BrokerChartInput[] = []; const violations: string[] = []; const errors: string[] = []
  page.on('pageerror', error => errors.push(error.message))
  await page.routeWebSocket('**/*', socket => {
    const url = new URL(socket.url())
    if (process.env.CI || !['localhost', '127.0.0.1'].includes(url.hostname) || !url.searchParams.has('token')) violations.push(`WS ${url.pathname}`)
    socket.close()
  })
  if (!options.research) await mockApi(page)
  await page.route('**/api/strategy/sector-portfolio/preview', async route => route.fulfill({ json: fixtureReport(route.request().postDataJSON()) }))
  await page.route('**/api/strategy/broker-chart/preview', async route => {
    const input = route.request().postDataJSON() as BrokerChartInput; calls.push(input)
    if (options.wait) await options.wait
    // Deliberate error fixtures exercise the presentation, not financial validation itself.
    const code = input.knownCorporateActions.length ? 'UNSUPPORTED_BROKER_CHART_CORPORATE_ACTION'
      : input.evaluation[0].prices.SPY.high < input.evaluation[0].prices.SPY.close ? 'INVALID_BROKER_CHART_OHLCV'
        : input.evaluation[0].date !== input.expectedSessionDates[input.history.length] ? 'BROKER_CHART_SESSION_MISMATCH' : null
    if (options.validateFixture && code) return route.fulfill({ status: 400, json: { code, error: code } })
    return route.fulfill({ json: brokerChartFixtureReport(input) })
  })
  await page.route('**/*', async route => {
    const url = new URL(route.request().url()); const method = route.request().method()
    const pure = method === 'POST' && ['/api/strategy/broker-chart/preview', '/api/strategy/sector-portfolio/preview'].includes(url.pathname)
    if (!['127.0.0.1', 'localhost'].includes(url.hostname) || (method !== 'GET' && !pure)
      || (options.research && url.pathname.startsWith('/api/') && !pure)) {
      violations.push(`${method} ${url.pathname}`); return route.abort()
    }
    return route.fallback()
  })
  await page.goto(options.research ? '/?researchOnly=1' : '/simulation')
  const panel = page.getByTestId('sector-portfolio-panel')
  if (!options.research) await panel.getByRole('tab', { name: '세 전략 비교', exact: true }).click()
  const mode = panel.getByRole('combobox', { name: '비교 방식', exact: true })
  await expect(mode).toContainText('검증형 비교')
  await mode.click(); await page.getByRole('option', { name: '차트 가격수익률 근사', exact: true }).click()
  return { panel, chart: panel.getByTestId('broker-chart-panel'), calls, violations, errors, mode }
}

test('research price mode needs its own acknowledgement, preserves 20bp raw metadata and exports the full wrapper at 375px', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 })
  const state = await setup(page, { research: true })
  const { chart } = state
  expect(state.calls).toEqual([])
  await expect(chart.getByRole('button', { name: '차트 조회 프로필 확인', exact: true })).toBeDisabled()
  await chart.getByRole('button', { name: '합성 차트 예제 입력', exact: true }).click()
  const run = chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true })
  await expect(run).toBeDisabled()
  await chart.getByRole('combobox', { name: '가격 근사 편도 총비용', exact: true }).click()
  await page.getByRole('option', { name: '20bp · 0.2% 스트레스', exact: true }).click()
  const ack = chart.getByRole('checkbox', { name: limitAck, exact: true })
  await ack.focus(); await page.keyboard.press('Space'); await expect(ack).toBeChecked()
  await run.click()
  await expect(chart.getByTestId('broker-chart-results')).toBeVisible()
  await expect(chart.getByLabel('세 전략과 비교군 성과표').locator('tbody tr')).toHaveCount(5)
  expect(state.calls).toHaveLength(1)
  expect(state.calls[0]).toMatchObject({ initialCashUsd: 10_000, costBps: 20, acknowledgedPriceOnly: true, source: { adjustment: 'unadjusted', synthetic: true } })
  expect(state.calls[0].history).toHaveLength(22); expect(state.calls[0].evaluation).toHaveLength(20)
  await expect(chart).toContainText('배당재투자 총수익이 아닙니다')
  await expect(chart).toContainText('실제 09:50')
  const promise = page.waitForEvent('download')
  await chart.getByRole('button', { name: '가격 근사 입력과 결과 JSON 다운로드', exact: true }).click()
  const downloaded = await promise
  const exported = JSON.parse(await readFile((await downloaded.path())!, 'utf8'))
  expect(exported.report).toMatchObject({ mode: 'brokerChartPriceApproximation', dividendReturnIncluded: false, economicReturnVerified: false })
  expect(exported.input.costBps).toBe(20)
  await expect.poll(() => page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
  await chart.getByTestId('broker-chart-results').scrollIntoViewIfNeeded()
  await page.screenshot({ path: '.cache/sector-frontend/broker-chart-mobile.png', fullPage: true, animations: 'disabled' })
  expect(state.violations).toEqual([]); expect(state.errors).toEqual([])
})

test('strict and price experiments retain separate state across mode and workspace tabs without creating a paper ledger', async ({ page }) => {
  const state = await setup(page)
  await state.chart.getByRole('button', { name: '합성 차트 예제 입력', exact: true }).click()
  await state.chart.getByRole('checkbox', { name: limitAck, exact: true }).check()
  await state.chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true }).click()
  await expect(state.chart.getByTestId('broker-chart-results')).toBeVisible()
  const rawText = await state.chart.getByLabel('차트 가격 자료 JSON', { exact: true }).inputValue()
  await state.panel.getByRole('tab', { name: '모의 결과 대조', exact: true }).click()
  await expect(state.panel.getByLabel('모의 원장 평가 JSON', { exact: true })).toHaveCount(0)
  await expect(state.panel).toContainText('차트 가격 근사 원장을 자동 합산하거나 동기화하지 않습니다')
  await state.panel.getByRole('tab', { name: '세 전략 비교', exact: true }).click()
  await state.mode.click(); await page.getByRole('option', { name: '검증형 비교', exact: true }).click()
  await expect(state.panel.getByLabel('섹터 비교 자료 JSON', { exact: true })).toHaveValue('')
  await state.panel.getByRole('button', { name: '합성 예제 입력', exact: true }).click()
  await state.panel.getByRole('button', { name: '세 전략 비교 계산', exact: true }).click()
  await expect(state.panel.getByTestId('sector-portfolio-results').filter({ visible: true })).toBeVisible()
  await state.mode.focus(); await page.keyboard.press('ArrowDown'); await page.keyboard.press('End'); await page.keyboard.press('Enter')
  await expect(state.chart).toBeVisible()
  await expect(state.chart.getByLabel('차트 가격 자료 JSON', { exact: true })).toHaveValue(rawText)
  await expect(state.chart.getByTestId('broker-chart-results')).toBeVisible()
  await state.chart.getByLabel('차트 가격 자료 JSON', { exact: true }).fill('{')
  await expect(state.chart.getByTestId('broker-chart-results')).toHaveCount(0)
  expect(state.violations).toEqual([]); expect(state.errors).toEqual([])
})

test('known split, OHLC bounds and session errors are displayed without fabricated results and valid input can retry', async ({ page }) => {
  const state = await setup(page, { research: true, validateFixture: true })
  const valid = { ...syntheticBrokerChartInput(), acknowledgedPriceOnly: true }
  const invalids = [
    { input: { ...valid, knownCorporateActions: [{ symbol: 'SPY', date: valid.evaluation[0].date, kind: 'split', splitRatio: 2 }] }, code: 'UNSUPPORTED_BROKER_CHART_CORPORATE_ACTION' },
    { input: { ...valid, evaluation: valid.evaluation.map((day, index) => index ? day : { ...day, prices: { ...day.prices, SPY: { ...day.prices.SPY, high: 1 } } }) }, code: 'INVALID_BROKER_CHART_OHLCV' },
    { input: { ...valid, expectedSessionDates: valid.expectedSessionDates.map((date, index) => index === 22 ? '2025-01-01' : date) }, code: 'BROKER_CHART_SESSION_MISMATCH' },
  ]
  for (const fixture of invalids) {
    await state.chart.getByLabel('차트 가격 자료 JSON', { exact: true }).fill(JSON.stringify(fixture.input))
    await state.chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true }).click()
    await expect(state.chart.getByRole('alert').filter({ hasText: fixture.code })).toBeVisible()
    await expect(state.chart.getByTestId('broker-chart-results')).toHaveCount(0)
  }
  await state.chart.getByLabel('차트 가격 자료 JSON 파일', { exact: true }).setInputFiles({ name: 'raw.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(valid)) })
  await state.chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true }).click()
  await expect(state.chart.getByTestId('broker-chart-results')).toBeVisible()
  await state.chart.getByLabel('차트 가격 자료 JSON', { exact: true }).fill(JSON.stringify({ ...valid, source: { ...valid.source, calendarEvidence: {} } }))
  await expect(state.chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true })).toBeDisabled()
  expect(state.violations).toEqual([]); expect(state.errors).toEqual([])
})

test('late price preview is ignored after data edits and duplicate activation sends only one pure request', async ({ page }) => {
  let release: () => void = () => {}
  const wait = new Promise<void>(resolve => { release = resolve })
  const state = await setup(page, { research: true, wait })
  await state.chart.getByRole('button', { name: '합성 차트 예제 입력', exact: true }).click()
  await state.chart.getByRole('checkbox', { name: limitAck, exact: true }).check()
  await state.chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true }).dblclick()
  await expect.poll(() => state.calls.length).toBe(1)
  await state.chart.getByLabel('차트 가격 자료 JSON', { exact: true }).fill('{}')
  release()
  await expect(state.chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true })).toBeDisabled()
  await expect(state.chart.getByTestId('broker-chart-results')).toHaveCount(0)
  expect(state.violations).toEqual([]); expect(state.errors).toEqual([])
})

test('a non-synthetic source declaration remains non-synthetic without claiming authenticated provenance', async ({ page }) => {
  const state = await setup(page, { research: true })
  const fixture = syntheticBrokerChartInput()
  // Only a display fixture for a caller-declared source. No market data is fetched.
  const input = { ...fixture, acknowledgedPriceOnly: true, source: { ...fixture.source,
    name: '비합성 출처 선언 UI fixture · 실제 시세 아님', synthetic: false } }
  await state.chart.getByLabel('차트 가격 자료 JSON', { exact: true }).fill(JSON.stringify(input))
  await state.chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true }).click()
  await expect(state.chart.getByTestId('broker-chart-results')).toBeVisible()
  expect(state.calls[0].source.synthetic).toBe(false)
  await expect(state.chart).toContainText('브로커 자료라는 입력 선언')
  await expect(state.chart).toContainText('공급자 조정 범위·과거 기수정 가능성은 검증하지 않았습니다')
  await expect(state.chart.getByTestId('sector-portfolio-results')).toContainText('입력한 과거 자료의 재생 결과')
  await expect(state.chart.getByText('합성 자료 결과 · 실제 투자 성과가 아닙니다.', { exact: true })).toHaveCount(0)
  expect(state.violations).toEqual([]); expect(state.errors).toEqual([])
})
