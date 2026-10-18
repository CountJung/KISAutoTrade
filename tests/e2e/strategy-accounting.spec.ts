import { expect, test, type Page } from '@playwright/test'
import type { BacktestReport, ReplayAccounting, ReplayComparison, SimulationAssumptions } from '../../src/shared/api/strategyResearchTypes'
import { mockApi } from './fixtures/strategyResearchFixture'

const accounting: ReplayAccounting = {
  realizedPnlKrw: 5_000, residualUnrealizedPnlKrw: -160_000, remainingQuantity: 2,
  remainingCostBasisKrw: 280_000, remainingBuyFeeKrw: 1_000,
  realizedPricePnlKrw: 6_000, realizedFxPnlKrw: -500,
  residualPricePnlKrw: -100_000, residualFxPnlKrw: -59_000,
  totalFeeKrw: 1_500, totalTaxKrw: 0, totalSlippageKrw: 1_000,
  dividendGrossKrw: 6_000, dividendTaxKrw: 1_000, dividendNetKrw: 5_000,
  terminalLiquidationEquityKrw: 9_848_000, terminalLiquidationReturnPct: -1.52,
  terminalExitFeeKrw: 1_000, terminalExitTaxKrw: 500, terminalExitSlippageKrw: 500,
  averageInvestedNotionalKrw: 250_000, averageInvestedCapitalPct: 2.5,
  averageInvestedEquityPct: 2.6, peakInvestedCapitalPct: 3, peakInvestedEquityPct: 3.1,
  realizedEquitySignMismatch: true, warnings: ['fixture: 1 completed trade is a small sample'],
}
const comparison: ReplayComparison = {
  sameInputQuantityBuyHold: {
    requestedQuantity: 3, actualEntryQuantity: 2, remainingQuantity: 2,
    finalEquityKrw: 10_020_000, cumulativeReturnPct: 0.2, terminalLiquidationReturnPct: 0.18,
    realizedPnlKrw: 0, residualUnrealizedPnlKrw: 15_000, dividendNetKrw: 5_000,
    mddPct: 0.4, averageInvestedCapitalPct: 3, averageInvestedEquityPct: 2.9,
    peakInvestedCapitalPct: 4, peakInvestedEquityPct: 3.9,
    holdingBarPct: 80, filledOrderCount: 1, blockedOrderCount: 0, unfilledOrderCount: 0,
  },
  sameExecutionPathCostReference: { restoredCostKrw: 2_500, finalEquityKrw: 9_852_500, cumulativeReturnPct: -1.475 },
  capitalExposureDifferencePct: -0.5, holdingBarDifferencePct: -30,
  limitations: ['fixture: fixed executed path only'],
}

async function mockAccounting(page: Page, { legacy = false, domestic = false, requests = [] }: { legacy?: boolean; domestic?: boolean; requests?: unknown[] } = {}) {
  await mockApi(page, domestic ? {} : { activeBroker: 'toss', genericSymbol: 'QQQ' })
  await page.route('**/api/strategy/preview', async (route) => {
    const body = route.request().postDataJSON()
    requests.push(body)
    const assumptions = { ...body.assumptions } as SimulationAssumptions
    if (legacy) {
      for (const key of ['costSource', 'costNote', 'buyFeeBps', 'sellFeeBps', 'minimumFeeNative', 'fxScenario', 'dividendScenario', 'dividendPriceBasis'] as const) delete assumptions[key]
    }
    const backtest: BacktestReport = {
      assumptions,
      summary: {
        initialCapitalKrw: 10_000_000, finalEquityKrw: 9_850_000, cumulativeReturnPct: -1.5,
        mddPct: 2, completedTrades: 1, winningTrades: 1, losingTrades: 0,
        winRatePct: 100, profitFactor: null, turnoverPct: 5, exposurePct: 50,
        signalCount: 3, orderEligibleCount: 3, filledOrderCount: 3, blockedOrderCount: 0, unfilledOrderCount: 0,
      },
      // Accounting aggregates must be displayed from the backend, never inferred
      // from the (possibly truncated) saved trade list.
      trades: [], phases: [], equityCurve: [], overfitWarning: null, executionWarnings: [],
      ...(!legacy ? { accounting, comparison } : {}),
    }
    await route.fulfill({ json: {
      strategyId: body.strategyId, symbol: body.symbol, candles: body.candles, signals: [],
      generatedAt: '2026-07-04T15:30:00+09:00', message: 'fixture accounting result',
      replay: {
        engineVersion: legacy ? 'strategy-replay-v10' : 'strategy-replay-v11', strategyVersion: 'fixture',
        sourceInterval: '1d', replayCadence: 'dailyClose', liveCadenceSeconds: 10, warmupCount: 1,
        dataStart: '20260701', dataEnd: '20260704', dataSource: 'fixture:daily',
        deterministic: true, lookAheadSafe: false, inputHash: 'fixture-accounting',
      }, backtest,
    } })
  })
}

function card(page: Page) { return page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first() }
async function choose(page: Page, label: string, option: string) {
  await card(page).getByRole('combobox', { name: label }).click()
  await page.getByRole('option', { name: option, exact: true }).click()
}

const fx = '[{"barTime":"20260702","rateKrw":1400}]'
const dividend = '[{"barTime":"20260703","amountPerShare":1,"withholdingTaxBps":1500}]'

test('cost defaults and explicitly verified manual costs/scenarios reach replay without other writes', async ({ page }) => {
  const requests: unknown[] = []
  const otherWrites: string[] = []
  await mockAccounting(page, { requests })
  page.on('request', (request) => { if (request.method() === 'POST' && !request.url().endsWith('/api/strategy/preview')) otherWrites.push(request.url()) })
  await page.goto('/strategy')
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('backtest-accounting-results')).toBeVisible()
  expect(requests[0]).toMatchObject({ assumptions: { costSource: 'estimated', buyFeeBps: null, sellFeeBps: null, minimumFeeNative: 0, fxScenario: [], dividendScenario: [], dividendPriceBasis: 'unverified' } })
  await choose(page, '비용 출처', '브로커 조건 사용자 수동 확인')
  await expect(panel.getByRole('button', { name: '미리보기 계산' })).toBeDisabled()
  await panel.getByRole('textbox', { name: '비용 출처 메모' }).fill('브로커 공지 수동 확인 2026-07-01')
  await panel.getByRole('spinbutton', { name: '매수 수수료(bps)' }).fill('12')
  await panel.getByRole('spinbutton', { name: '매도 수수료(bps)' }).fill('15')
  await panel.getByRole('spinbutton', { name: '체결당 최소 수수료(USD)' }).fill('0.5')
  await choose(page, '가격 자료의 배당 반영', '배당 미조정 가격 · 사용자 선언')
  await panel.getByRole('textbox', { name: '환율 시나리오 JSON' }).fill(fx)
  await panel.getByRole('textbox', { name: '배당 시나리오 JSON' }).fill(dividend)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect.poll(() => requests.length).toBe(2)
  expect(requests[1]).toMatchObject({ assumptions: {
    costSource: 'brokerVerifiedManual', costNote: '브로커 공지 수동 확인 2026-07-01', buyFeeBps: 12, sellFeeBps: 15, minimumFeeNative: 0.5,
    fxScenario: JSON.parse(fx), dividendScenario: JSON.parse(dividend), dividendPriceBasis: 'unadjusted',
  } })
  expect(otherWrites).toEqual([])
})

test('invalid JSON invalidates the old preview and never silently falls back to an empty scenario', async ({ page }) => {
  const requests: unknown[] = []
  await mockAccounting(page, { requests })
  await page.goto('/strategy')
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('strategy-backtest-results')).toBeVisible()
  await panel.getByRole('textbox', { name: '환율 시나리오 JSON' }).fill('[{')
  await expect(panel.getByTestId('strategy-backtest-results')).toHaveCount(0)
  await expect(panel.getByTestId('simulation-accounting-error')).toContainText('유효한 JSON 배열')
  await expect(panel.getByRole('button', { name: '미리보기 계산' })).toBeDisabled()
  expect(requests).toHaveLength(1)
  await panel.getByRole('textbox', { name: '환율 시나리오 JSON' }).fill(fx)
  await expect(panel.getByRole('button', { name: '미리보기 계산' })).toBeEnabled()
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect.poll(() => requests.length).toBe(2)
  expect(requests[1]).toMatchObject({ assumptions: { fxScenario: JSON.parse(fx) } })
})

test('scenario bounds, duplicate labels and domestic FX are explicit errors', async ({ page }) => {
  await mockAccounting(page, { domestic: true })
  await page.goto('/strategy')
  const panel = card(page)
  await panel.getByRole('textbox', { name: '환율 시나리오 JSON' }).fill(fx)
  await expect(panel.getByTestId('simulation-accounting-error')).toContainText('국내 종목')
  await panel.getByRole('textbox', { name: '환율 시나리오 JSON' }).fill('[]')
  await panel.getByRole('textbox', { name: '배당 시나리오 JSON' }).fill('[{"barTime":"20260703","amountPerShare":1,"withholdingTaxBps":0},{"barTime":"2026-07-03","amountPerShare":1,"withholdingTaxBps":0}]')
  await expect(panel.getByTestId('simulation-accounting-error')).toContainText('같은 봉 라벨')
  await panel.getByRole('textbox', { name: '배당 시나리오 JSON' }).fill(JSON.stringify(Array.from({ length: 65 }, (_, index) => ({ barTime: String(index), amountPerShare: 1, withholdingTaxBps: 0 }))))
  await expect(panel.getByTestId('simulation-accounting-error')).toContainText('최대 64개')
  await expect(panel.getByRole('button', { name: '미리보기 계산' })).toBeDisabled()
})

test('known dividend-adjusted prices cannot be combined with cash dividends', async ({ page }) => {
  await mockAccounting(page)
  await page.goto('/strategy')
  await choose(page, '가격 자료의 배당 반영', '배당 조정 가격 · 사용자 선언')
  const panel = card(page)
  await panel.getByRole('textbox', { name: '배당 시나리오 JSON' }).fill(dividend)
  await expect(panel.getByTestId('simulation-accounting-error')).toContainText('이중 계산')
  await expect(panel.getByRole('button', { name: '미리보기 계산' })).toBeDisabled()
  await choose(page, '가격 자료의 배당 반영', '배당 조정 여부 미확인')
  await expect(panel.getByRole('button', { name: '미리보기 계산' })).toBeEnabled()
  await expect(panel.getByText('가격 자료의 배당 조정 여부가 확인되지 않았습니다.', { exact: false })).toBeVisible()
})

test('realized profit and residual loss stay separate with sample, FX, fees and hypothetical liquidation', async ({ page }) => {
  await mockAccounting(page)
  await page.goto('/strategy')
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  const results = panel.getByTestId('backtest-accounting-results')
  await expect(results).toContainText('실현손익 · 매도 확정')
  await expect(results).toContainText('5,000원')
  await expect(results).toContainText('-160,000원')
  await expect(results.getByTestId('backtest-accounting-sample')).toContainText('완료 거래 표본 1회')
  await expect(results.getByTestId('backtest-accounting-sample')).toContainText('실현손익과 전체 자산 수익률의 부호가 다릅니다')
  await expect(results).toContainText('실현 환율 효과')
  await expect(results).toContainText('배당 원천징수')
  await expect(results.getByTestId('backtest-capital-observations')).toContainText('봉말 평균 보유 평가액 250,000원')
  await expect(results.getByTestId('backtest-capital-observations')).toContainText('단순 평균')
  await expect(results.getByTestId('backtest-terminal-liquidation')).toContainText('실제 거래에 미포함')
  await expect(results.getByTestId('backtest-terminal-liquidation')).toContainText('9,848,000원')
  const comparisons = panel.getByTestId('backtest-accounting-comparison')
  await expect(comparisons).toContainText('요청 3주 · 실제 진입 2주')
  await expect(comparisons).toContainText('동일 체결 경로의 비용 제거 참고값')
  await expect(comparisons).toContainText('비용이 없는 전략을 다시 실행한 성과나 실제 거래 결과가 아닙니다')
})

test('A/B persists backend accounting aggregates and exposes differing cost and cash-flow assumptions', async ({ page }) => {
  await mockAccounting(page)
  await page.goto('/strategy')
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await panel.getByRole('button', { name: '현재 결과를 A로 저장' }).click()
  await panel.getByRole('spinbutton', { name: '매수 수수료(bps)' }).fill('12')
  await panel.getByRole('textbox', { name: '환율 시나리오 JSON' }).fill(fx)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await panel.getByRole('button', { name: '현재 결과를 B로 저장' }).click()
  const ab = panel.getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('엔진 또는 체결 가정이 다릅니다')
  await expect(ab).toContainText('매수 12bps')
  await expect(ab.getByText('실현 5,000원 · 잔여 평가 -160,000원 · 순배당 5,000원', { exact: true })).toHaveCount(2)
  await ab.getByText('저장된 환율·배당 가정 보기', { exact: true }).click()
  await expect(ab).toContainText('1400')
  await page.reload()
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('strategy-ab-comparison')).toContainText('매수 12bps')
})

test('legacy accounting and comparator data remain unknown after saving and reloading', async ({ page }) => {
  await mockAccounting(page, { legacy: true })
  await page.goto('/strategy')
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('backtest-accounting-unknown')).toContainText('0으로 추정하지 않습니다')
  await expect(panel.getByTestId('backtest-comparison-unknown')).toBeVisible()
  await panel.getByRole('button', { name: '현재 결과를 A로 저장' }).click()
  await page.reload()
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('strategy-ab-comparison').getByTestId('backtest-accounting-unknown')).toBeVisible()
})

test('mobile cost/scenario controls and accounting results stay within the card', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await mockAccounting(page)
  await page.goto('/strategy')
  const panel = card(page)
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('backtest-accounting-results')).toBeVisible()
  const metrics = await panel.evaluate((element) => ({ width: element.clientWidth, scroll: element.scrollWidth }))
  expect(metrics.scroll).toBeLessThanOrEqual(metrics.width + 1)
  const panelBox = await panel.boundingBox()
  for (const label of ['환율 시나리오 JSON', '배당 시나리오 JSON']) {
    const box = await panel.getByRole('textbox', { name: label }).boundingBox()
    expect(box!.x + box!.width).toBeLessThanOrEqual(panelBox!.x + panelBox!.width)
  }
})

test('LTH invalid drafts disable calculation and daily diagnostics hide accounting performance', async ({ page }) => {
  await mockApi(page, { activeBroker: 'toss' })
  await page.goto('/strategy')
  const panel = page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
  await panel.getByRole('textbox', { name: '배당 시나리오 JSON' }).fill('invalid')
  await expect(panel.getByRole('button', { name: '미리보기 계산' })).toBeDisabled()
  await panel.getByRole('textbox', { name: '배당 시나리오 JSON' }).fill('[]')
  await panel.getByRole('combobox', { name: '봉 단위' }).click()
  await page.getByRole('option', { name: '일봉', exact: true }).click()
  await panel.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(panel.getByTestId('strategy-replay-diagnostic')).toBeVisible()
  await expect(panel.getByTestId('backtest-accounting-results')).toHaveCount(0)
  await expect(panel.getByTestId('backtest-accounting-comparison')).toHaveCount(0)
})
