import { expect, test, type Page } from '@playwright/test'
import type { SectorPortfolioInput, SectorPortfolioReport, SectorStrategy } from '../../src/shared/api/sectorPortfolioTypes'
import { syntheticSectorPortfolioInput } from '../../src/pages/strategy/model/sectorPortfolioInput'
import { mockApi } from './fixtures/strategyResearchFixture'

const strategies: SectorStrategy[] = ['base', 'defensive', 'aggressive', 'spy', 'cash']
function fixtureReport(input: SectorPortfolioInput): SectorPortfolioReport {
  const daily = input.evaluation.map(day => ({
    date: day.date,
    plan: { strategy: 'base' as SectorStrategy, signalDate: input.history[input.history.length - 1].date, defensiveState: 'normal' as const, selectedSymbols: [], targetWeight: 0, referenceEquityUsd: 10_000, rebalance: false, reasons: ['UI 응답 fixture'], targets: [] },
    cashUsd: 10_000, equityUsd: 10_000, dailyReturnPct: 0, drawdownPct: 0, exposurePct: 0, holdings: [],
  }))
  return {
    engineVersion: 'sector-portfolio-ui-fixture', inputHash: 'a'.repeat(64), source: input.source,
    historyBars: input.history.length, evaluationBars: input.evaluation.length,
    startDate: daily[0].date, endDate: daily[daily.length - 1].date, executionModel: 'nextOpen',
    warnings: ['합성 응답은 Rust 전략 성과 검증을 대신하지 않습니다.'], rankings: [],
    portfolios: strategies.map(strategy => ({
      strategy, daily: daily.map(day => ({ ...day, plan: { ...day.plan, strategy } })), trades: [], endingHoldings: [],
      summary: { initialCashUsd: 10_000, endingEquityUsd: 10_000, netReturnPct: 0, samePathReturnBeforeCostsPct: 0, maxDrawdownPct: 0, maxDailyLossPct: 0, dailyVolatilityPct: 0, totalCostUsd: 0, realizedPnlUsd: 0, unrealizedPnlUsd: 0, averageExposurePct: 0, turnoverPct: 0, filledTradeCount: 0, cashLimitedCount: 0, endingCashUsd: 10_000 },
    })),
  }
}

async function setup(page: Page, options: { fail?: boolean; delayed?: Promise<void>; started?: () => void; accountError?: boolean } = {}) {
  const requests: SectorPortfolioInput[] = []
  const violations: string[] = []
  const errors: string[] = []
  page.on('pageerror', error => errors.push(error.message))
  await page.routeWebSocket('**/*', socket => socket.close())
  await mockApi(page)
  await page.route('**/api/strategy/sector-portfolio/preview', async route => {
    const input = route.request().postDataJSON() as SectorPortfolioInput
    requests.push(input)
    options.started?.()
    if (options.delayed) await options.delayed
    if (options.fail) await route.fulfill({ status: 400, json: { code: 'SECTOR_INVALID', error: '거래일 누락으로 평가 불가' } })
    else await route.fulfill({ json: fixtureReport(input) })
  })
  if (options.accountError) await page.route('**/api/strategies', route => route.fulfill({ status: 500, json: { code: 'FIXTURE', error: '합성 계좌 목록 오류' } }))
  await page.route('**/*', async route => {
    const url = new URL(route.request().url())
    if (url.protocol === 'data:' || url.protocol === 'blob:') return route.fallback()
    const purePreview = url.pathname === '/api/strategy/sector-portfolio/preview' && route.request().method() === 'POST'
    if (!['127.0.0.1', 'localhost'].includes(url.hostname) || (route.request().method() !== 'GET' && !purePreview)) {
      violations.push(`${route.request().method()} ${url.pathname}`)
      return route.abort()
    }
    await route.fallback()
  })
  await page.goto('/strategy')
  const panel = page.getByTestId('sector-portfolio-panel')
  await panel.getByRole('button', { name: '섹터 ETF 3전략 비교', exact: true }).click()
  await expect(panel.locator('.MuiCollapse-root').first()).toHaveClass(/MuiCollapse-entered/)
  return { panel, requests, violations, errors }
}

async function captureResults(page: Page, path: string) {
  await page.addStyleTag({ content: '* { scroll-behavior: auto !important; }' })
  await page.getByTestId('sector-portfolio-results').evaluate(element => element.scrollIntoView({ block: 'start', behavior: 'instant' }))
  await page.screenshot({ path, fullPage: true, animations: 'disabled' })
}

test('JSON research works independently of account errors, preserves metadata and renders five portfolios without financial mutations', async ({ page }) => {
  const { panel, requests, violations, errors } = await setup(page, { accountError: true })
  await expect(panel.getByRole('button', { name: '세 전략 비교 계산', exact: true })).toBeDisabled()
  await panel.getByRole('button', { name: '합성 예제 입력', exact: true }).click()
  await panel.getByRole('button', { name: '세 전략 비교 계산', exact: true }).click()
  await expect(panel.getByTestId('sector-portfolio-results')).toBeVisible()
  expect(requests).toHaveLength(1)
  expect(requests[0]).toEqual(syntheticSectorPortfolioInput())
  await expect(panel).toContainText('합성 자료 결과')
  await expect(panel).toContainText('09:50 조회 표시가격과 다릅니다')
  await expect(panel.getByLabel('세 전략과 비교군 성과표').locator('tbody tr')).toHaveCount(5)
  await page.setViewportSize({ width: 1440, height: 1000 })
  await captureResults(page, '.cache/sector-frontend/desktop-results.png')
  await expect(panel.getByRole('button', { name: '입력과 결과 JSON 다운로드', exact: true })).toBeVisible()
  const downloadPromise = page.waitForEvent('download')
  await panel.getByRole('button', { name: '입력과 결과 JSON 다운로드', exact: true }).click()
  const download = await downloadPromise
  expect(download.suggestedFilename()).toMatch(/^sector-portfolio-/)
  await panel.getByLabel('섹터 비교 자료 JSON', { exact: true }).fill('{')
  await expect(panel.getByTestId('sector-portfolio-results')).toHaveCount(0)
  await expect(panel.getByRole('button', { name: '세 전략 비교 계산', exact: true })).toBeDisabled()
  expect(violations).toEqual([])
  expect(errors).toEqual([])
})

test('late calculations are discarded after edits and duplicate activation sends one pure request', async ({ page }) => {
  let release: () => void = () => {}
  const delayed = new Promise<void>(resolve => { release = resolve })
  const { panel, requests, violations } = await setup(page, { delayed })
  await panel.getByRole('button', { name: '합성 예제 입력', exact: true }).click()
  const run = panel.getByRole('button', { name: '세 전략 비교 계산', exact: true })
  await run.dblclick()
  await expect.poll(() => requests.length).toBe(1)
  await panel.getByLabel('섹터 비교 자료 JSON', { exact: true }).fill('{}')
  release()
  await expect(panel.getByRole('button', { name: '세 전략 비교 계산', exact: true })).toBeDisabled()
  await expect(panel.getByTestId('sector-portfolio-results')).toHaveCount(0)
  expect(requests).toHaveLength(1)
  expect(violations).toEqual([])
})

test('backend validation failure is visible as unevaluable and a fresh retry is possible', async ({ page }) => {
  const { panel, requests, violations } = await setup(page, { fail: true })
  await panel.getByRole('button', { name: '합성 예제 입력', exact: true }).click()
  await panel.getByRole('button', { name: '세 전략 비교 계산', exact: true }).click()
  await expect(panel).toContainText('거래일 누락으로 평가 불가')
  await expect(panel.getByTestId('sector-portfolio-results')).toHaveCount(0)
  await panel.getByRole('button', { name: '세 전략 비교 계산', exact: true }).click()
  await expect.poll(() => requests.length).toBe(2)
  expect(violations).toEqual([])
})

test('file import, equal-window paper comparison, mobile keyboard controls and stale-result removal', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  const { panel, violations, errors } = await setup(page)
  const input = syntheticSectorPortfolioInput()
  await panel.getByLabel('섹터 자료 JSON 파일').setInputFiles({ name: 'synthetic.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(input)) })
  await panel.getByRole('button', { name: '세 전략 비교 계산', exact: true }).click()
  await expect(panel.getByTestId('sector-portfolio-results')).toBeVisible()
  await panel.getByRole('button', { name: '현재 모의 원장과 대조 (선택)', exact: true }).click()
  const paper = panel.getByLabel('모의 원장 평가 JSON', { exact: true })
  const mark = { date: input.evaluation[0].date, strategy: 'base', equityUsd: 10001, priceSource: '합성 UI 입력 · 09:50 표시가' }
  await paper.fill(JSON.stringify({ initialCashUsd: 10_000, costBps: 10, startDate: '2020-01-01', marks: [mark] }))
  await expect(panel).toContainText('직접 비교하려면 모의 원장의 시작일과 초기자금')
  await expect(panel.getByLabel('모의 원장과 코드 재생 차이')).toHaveCount(0)
  await paper.fill(JSON.stringify({ initialCashUsd: 11_000, costBps: 10, startDate: input.evaluation[0].date, marks: [mark] }))
  await expect(panel).toContainText('직접 비교하려면 모의 원장의 시작일과 초기자금')
  await expect(panel.getByLabel('모의 원장과 코드 재생 차이')).toHaveCount(0)
  await paper.fill(JSON.stringify({ initialCashUsd: 10_000, costBps: 10, startDate: input.evaluation[0].date, marks: [{ ...mark, date: '2020-01-01' }] }))
  await expect(panel.getByLabel('모의 원장과 코드 재생 차이')).toContainText('같은 평가일 없음')
  await expect(panel.getByLabel('모의 원장과 코드 재생 차이')).toContainText('비교 불가')
  await paper.fill(JSON.stringify({ initialCashUsd: 10_000, costBps: 20, startDate: input.evaluation[0].date, marks: [mark] }))
  await expect(panel).toContainText('비용 가정이 달라 직접 비교할 수 없습니다')
  await expect(panel.getByLabel('모의 원장과 코드 재생 차이')).toHaveCount(0)
  await paper.fill(JSON.stringify({ initialCashUsd: 10_000, costBps: 10, startDate: input.evaluation[0].date, marks: [mark] }))
  await expect(panel.getByLabel('모의 원장과 코드 재생 차이')).toContainText('1.00')
  await paper.fill(JSON.stringify({ initialCashUsd: 10_000, costBps: 10, startDate: input.evaluation[0].date, marks: [mark, mark] }))
  await expect(panel).toContainText('같은 전략·날짜의 모의 평가가 중복')
  await expect(panel.getByLabel('모의 원장과 코드 재생 차이')).toHaveCount(0)
  await panel.getByLabel('상세 비교 전략', { exact: true }).focus()
  await page.keyboard.press('ArrowDown')
  await page.getByRole('option', { name: '방어형', exact: true }).click()
  await expect(panel.getByLabel('상세 비교 전략', { exact: true })).toContainText('방어형')
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
  await captureResults(page, '.cache/sector-frontend/mobile-results.png')
  await panel.getByRole('button', { name: '합성 예제 입력', exact: true }).click()
  await expect(panel.getByTestId('sector-portfolio-results')).toHaveCount(0)
  expect(violations).toEqual([])
  expect(errors).toEqual([])
})
