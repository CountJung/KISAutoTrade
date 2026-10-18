import { expect, test, type Page } from '@playwright/test'
import type { BacktestAtrPreparation, BacktestReport, SimulationAssumptions } from '../../src/shared/api/strategyResearchTypes'
import { mockApi } from './fixtures/strategyResearchFixture'

const readyAtr: BacktestAtrPreparation = {
  period: 14,
  historyBars: 15,
  readyAtStart: true,
  readyAtEnd: true,
  firstReadyTime: '20260701',
  unreadyEvaluationBars: 0,
  rangePolicy: 'observedBarsWithPreviousCloseNoInterpolation',
}

async function mockAtrResult(page: Page, preparation?: BacktestAtrPreparation, requests: unknown[] = []) {
  await mockApi(page)
  // All broker traffic stays in the existing API fixture; this route replaces
  // only the production replay response to exercise the actual results DOM.
  await page.route('**/api/strategy/preview', async (route) => {
    const body = route.request().postDataJSON()
    requests.push(body)
    const blocked = preparation != null && preparation.unreadyEvaluationBars > 0
    const report: BacktestReport = {
      assumptions: body.assumptions as SimulationAssumptions,
      summary: {
        initialCapitalKrw: 10_000_000, finalEquityKrw: 10_000_000, cumulativeReturnPct: 0,
        mddPct: 0, completedTrades: 0, winningTrades: 0, losingTrades: 0,
        winRatePct: 0, profitFactor: null, turnoverPct: 0, exposurePct: 0,
        signalCount: 1, orderEligibleCount: blocked ? 0 : 1,
        filledOrderCount: blocked ? 0 : 1, blockedOrderCount: blocked ? 1 : 0,
      },
      phases: [],
      trades: [{
        time: '20260701', side: 'buy', signalPrice: 104, fillPrice: blocked ? null : 104,
        quantity: 1, grossAmountKrw: blocked ? 0 : 104, costKrw: 0, realizedPnlKrw: null,
        status: blocked ? 'blocked' : 'filled', reason: 'mock strategy signal',
        ...(blocked ? { blockedReason: 'ATR 준비 부족: 14개 True Range가 필요합니다' } : {}),
        phase: 'inSample',
      }],
      equityCurve: [], overfitWarning: null,
      ...(preparation ? { atrPreparation: preparation } : {}),
    }
    await route.fulfill({ json: {
      strategyId: body.strategyId, symbol: body.symbol, candles: body.candles, signals: [],
      generatedAt: '2026-07-04T15:30:00+09:00', message: 'mock ATR research result',
      replay: {
        engineVersion: preparation ? 'strategy-replay-v8' : 'strategy-replay-v7',
        strategyVersion: 'mock-v1', sourceInterval: '1d', replayCadence: 'dailyClose',
        liveCadenceSeconds: 10, warmupCount: 1, dataStart: '20260701', dataEnd: '20260704',
        dataSource: 'mock:daily', deterministic: true, lookAheadSafe: true, inputHash: 'mock-atr-hash',
      },
      backtest: report,
    } })
  })
}

function maCard(page: Page) {
  return page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
}

async function calculateAtr(page: Page) {
  await page.goto('/strategy')
  const card = maCard(page)
  await card.getByRole('checkbox', { name: '백테스트 ATR 기반 수량 산정' }).check()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  return card
}

test('ATR settings remain off by default and edited research assumptions reach replay', async ({ page }) => {
  const requests: unknown[] = []
  await mockAtrResult(page, readyAtr, requests)
  await page.goto('/strategy')
  const card = maCard(page)
  await expect(card.getByRole('checkbox', { name: '백테스트 ATR 기반 수량 산정' })).not.toBeChecked()
  await expect(card.getByRole('spinbutton', { name: '거래당 위험(bps)' })).toHaveCount(0)
  await card.getByRole('checkbox', { name: '백테스트 ATR 기반 수량 산정' }).check()
  await card.getByRole('spinbutton', { name: '거래당 위험(bps)' }).fill('75')
  await card.getByRole('spinbutton', { name: 'ATR 손절 배수' }).fill('3.5')
  await expect(card.getByText(/이 백테스트에만 적용/)).toBeVisible()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect.poll(() => requests.length).toBe(1)
  expect(requests[0]).toMatchObject({ assumptions: { volatilitySizingEnabled: true, riskPerTradeBps: 75, atrStopMultiplier: 3.5 } })
  const notice = card.getByTestId('backtest-atr-preparation')
  await expect(notice).toHaveClass(/MuiAlert-standardInfo/)
  await expect(notice.getByText(/ATR 기간 14 · 사전 관측 15봉 · 시작 준비 · 종료 준비/)).toBeVisible()
  await expect(notice.getByText(/전략 지표 준비 상태와 별도/)).toBeVisible()
})

test('ATR shortage warns separately and the blocked buy reason remains visible', async ({ page }) => {
  await mockAtrResult(page, { ...readyAtr, historyBars: 10, readyAtStart: false, readyAtEnd: false, firstReadyTime: null, unreadyEvaluationBars: 4 })
  const card = await calculateAtr(page)
  const notice = card.getByTestId('backtest-atr-preparation')
  await expect(notice).toHaveClass(/MuiAlert-standardWarning/)
  await expect(notice.getByText(/사전 관측 10봉 · 시작 미준비 · 종료 미준비/)).toBeVisible()
  await expect(notice.getByText(/ATR 미준비 평가 4봉 · 최초 준비 확인되지 않음/)).toBeVisible()
  await expect(card.getByRole('table', { name: '백테스트 거래 목록' }).getByText('ATR 준비 부족: 14개 True Range가 필요합니다')).toBeVisible()
})

test('ATR that becomes ready during evaluation keeps the initial shortage warning', async ({ page }) => {
  await mockAtrResult(page, { ...readyAtr, historyBars: 13, readyAtStart: false, firstReadyTime: '20260702', unreadyEvaluationBars: 1 })
  const card = await calculateAtr(page)
  const notice = card.getByTestId('backtest-atr-preparation')
  await expect(notice).toHaveClass(/MuiAlert-standardWarning/)
  await expect(notice.getByText(/시작 미준비 · 종료 준비/)).toBeVisible()
  await expect(notice.getByText(/ATR 미준비 평가 1봉 · 최초 준비 20260702/)).toBeVisible()
})

test('legacy ATR-enabled reports show unknown readiness instead of assuming prepared', async ({ page }) => {
  await mockAtrResult(page)
  const card = await calculateAtr(page)
  await expect(card.getByTestId('backtest-atr-preparation').getByText(/이전 응답에는 ATR 준비 정보가 없습니다/)).toBeVisible()
})

test('disabled ATR sizing does not show an unrelated readiness warning', async ({ page }) => {
  await mockAtrResult(page, { ...readyAtr, readyAtStart: false, readyAtEnd: false, unreadyEvaluationBars: 4 })
  await page.goto('/strategy')
  const card = maCard(page)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-backtest-results')).toBeVisible()
  await expect(card.getByTestId('backtest-atr-preparation')).toHaveCount(0)
})

test('LTH daily evaluation unavailable remains prior to ATR result rendering', async ({ page }) => {
  await mockApi(page, { activeBroker: 'toss' })
  await page.goto('/strategy')
  const card = page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
  await card.getByRole('checkbox', { name: '백테스트 ATR 기반 수량 산정' }).check()
  await card.getByRole('combobox', { name: '봉 단위' }).click()
  await page.getByRole('option', { name: '일봉', exact: true }).click()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-replay-diagnostic')).toBeVisible()
  await expect(card.getByTestId('backtest-atr-preparation')).toHaveCount(0)
  await expect(card.getByTestId('strategy-backtest-results')).toHaveCount(0)
})
