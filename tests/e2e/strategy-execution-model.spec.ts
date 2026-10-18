import { expect, test, type Locator, type Page } from '@playwright/test'
import type { BacktestReport, SimulationAssumptions } from '../../src/shared/api/strategyResearchTypes'
import { mockApi } from './fixtures/strategyResearchFixture'

type Options = { legacy?: boolean; requests?: unknown[] }

async function mockExecution(page: Page, { legacy = false, requests = [] }: Options = {}) {
  await mockApi(page)
  await page.route('**/api/strategy/preview', async (route) => {
    const body = route.request().postDataJSON()
    requests.push(body)
    const assumptions = { ...body.assumptions } as SimulationAssumptions
    if (legacy) {
      delete assumptions.executionModel
      delete assumptions.intrabarExitPolicy
      delete assumptions.stopLossPct
      delete assumptions.takeProfitPct
    }
    const nextOpen = assumptions.executionModel === 'nextOpen'
    const backtest: BacktestReport = {
      assumptions,
      summary: {
        initialCapitalKrw: 10_000_000, finalEquityKrw: 9_999_990, cumulativeReturnPct: -0.0001,
        mddPct: 0.0001, completedTrades: 1, winningTrades: 0, losingTrades: 1,
        winRatePct: 0, profitFactor: 0, turnoverPct: 0.01, exposurePct: 50,
        signalCount: 3, orderEligibleCount: 3, filledOrderCount: 2, blockedOrderCount: 0,
        ...(!legacy ? { unfilledOrderCount: nextOpen ? 1 : 0, ambiguousBarCount: 1 } : {}),
      },
      phases: [], equityCurve: [], overfitWarning: null,
      trades: [
        {
          time: nextOpen ? '20260702' : '20260701', side: 'buy', signalPrice: 100, fillPrice: nextOpen ? 110 : 100,
          quantity: 1, grossAmountKrw: 110, costKrw: 0, realizedPnlKrw: null,
          status: 'filled', reason: 'fixture buy', phase: 'inSample',
          ...(!legacy ? { signalAvailableAt: { barTime: '20260701', phase: 'close' }, fillAt: { barTime: nextOpen ? '20260702' : '20260701', phase: nextOpen ? 'open' : 'close' } } as const : {}),
        },
        {
          time: '20260702', side: 'sell', signalPrice: 105, fillPrice: 105,
          quantity: 1, grossAmountKrw: 105, costKrw: 0, realizedPnlKrw: -5,
          status: 'filled', reason: '손절·익절 동시 도달 · 손절 우선', phase: 'outOfSample',
          ...(!legacy ? { signalAvailableAt: { barTime: '20260702', phase: 'intrabar' }, fillAt: { barTime: '20260702', phase: 'intrabar' }, intrabarPathStatus: 'ambiguousStopFirst' } as const : {}),
        },
        {
          time: '20260704', side: 'buy', signalPrice: 100, fillPrice: null,
          quantity: 1, grossAmountKrw: 0, costKrw: 0, realizedPnlKrw: null,
          status: nextOpen ? 'unfilled' : 'blocked', reason: '다음 관측 봉 없음', phase: 'outOfSample',
          ...(!legacy ? { signalAvailableAt: { barTime: '20260704', phase: 'close' } } as const : {}),
        },
      ],
      ...(!legacy ? { executionWarnings: ['모의 체결 시각과 봉 내부 가격 경로는 실제 체결을 보장하지 않습니다.'] } : {}),
    }
    await route.fulfill({ json: {
      strategyId: body.strategyId, symbol: body.symbol, candles: body.candles, signals: [],
      generatedAt: '2026-07-04T15:30:00+09:00', message: 'fixture execution result',
      replay: {
        engineVersion: legacy ? 'strategy-replay-v8' : 'strategy-replay-v10', strategyVersion: 'mock-v1',
        sourceInterval: '1d', replayCadence: 'dailyClose', liveCadenceSeconds: 10, warmupCount: 1,
        dataStart: '20260701', dataEnd: '20260704', dataSource: 'mock:daily',
        deterministic: true, lookAheadSafe: true, inputHash: 'mock-execution-hash',
      }, backtest,
    } })
  })
}

function maCard(page: Page) {
  return page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
}

async function selectOption(page: Page, card: Locator, label: string, option: string) {
  await card.getByRole('combobox', { name: label }).click()
  await page.getByRole('option', { name: option, exact: true }).click()
}

test('execution controls send explicit defaults and edited research assumptions without live updates', async ({ page }) => {
  const requests: unknown[] = []
  const updates: string[] = []
  await mockExecution(page, { requests })
  page.on('request', (request) => {
    if (request.method() === 'POST' && !request.url().endsWith('/api/strategy/preview')) updates.push(request.url())
  })
  await page.goto('/strategy')
  const card = maCard(page)
  await expect(card.getByRole('combobox', { name: '체결 모델' })).toContainText('신호 봉 종가 체결 가정')
  await expect(card.getByRole('combobox', { name: '봉 내부 청산 정책' })).toContainText('봉 내부 경로 미검증')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-backtest-results')).toBeVisible()
  expect(requests[0]).toMatchObject({ assumptions: { executionModel: 'sameClose', intrabarExitPolicy: 'unverified', stopLossPct: 0, takeProfitPct: 0 } })
  await selectOption(page, card, '체결 모델', '다음 관측 봉 시가 체결 가정')
  await expect(card.getByTestId('strategy-backtest-results')).toHaveCount(0)
  await selectOption(page, card, '봉 내부 청산 정책', '손절 우선 · 보수적 모의 청산')
  await card.getByRole('spinbutton', { name: '연구 손절(%)' }).fill('3')
  await card.getByRole('spinbutton', { name: '연구 익절(%)' }).fill('5')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect.poll(() => requests.length).toBe(2)
  expect(requests[1]).toMatchObject({ assumptions: { executionModel: 'nextOpen', intrabarExitPolicy: 'stopFirst', stopLossPct: 3, takeProfitPct: 5 } })
  expect(updates).toEqual([])
})

test('next-open results distinguish signal availability, fill phase, ambiguous bar and final unfilled', async ({ page }) => {
  await mockExecution(page)
  await page.goto('/strategy')
  const card = maCard(page)
  await selectOption(page, card, '체결 모델', '다음 관측 봉 시가 체결 가정')
  await selectOption(page, card, '봉 내부 청산 정책', '손절 우선 · 보수적 모의 청산')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  const results = card.getByTestId('strategy-backtest-results')
  await expect(results.getByText('엔진 strategy-replay-v10', { exact: true })).toBeVisible()
  const table = results.getByRole('table', { name: '백테스트 거래 목록' })
  const buy = table.getByRole('row').filter({ hasText: 'fixture buy' })
  await expect(buy.getByRole('cell', { name: '20260701 · 종가', exact: true })).toBeVisible()
  await expect(buy.getByRole('cell', { name: '20260702 · 시가', exact: true })).toBeVisible()
  await expect(table.getByRole('row').filter({ hasText: '다음 관측 봉 없음' }).getByText('미체결', { exact: true })).toBeVisible()
  await expect(results.getByText(/차단 0개 · 미체결 1개/)).toBeVisible()
  await expect(results.getByText(/손절·익절 동시 도달 1봉/)).toBeVisible()
  await expect(results.getByTestId('backtest-execution-assumptions')).toContainText('정확한 거래소 체결 시각을')
})

test('A/B retains and shows differing execution assumptions with engine versions', async ({ page }) => {
  await mockExecution(page)
  await page.goto('/strategy')
  const card = maCard(page)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await card.getByRole('button', { name: '현재 결과를 A로 저장' }).click()
  await selectOption(page, card, '체결 모델', '다음 관측 봉 시가 체결 가정')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await card.getByRole('button', { name: '현재 결과를 B로 저장' }).click()
  const comparison = card.getByTestId('strategy-ab-comparison')
  await expect(comparison).toContainText('A와 B의 엔진 또는 체결 가정이 다릅니다')
  await expect(comparison).toContainText('신호 봉 종가')
  await expect(comparison).toContainText('다음 관측 봉 시가')
  await expect(comparison.getByText(/엔진 strategy-replay-v10/)).toHaveCount(2)
  await page.reload()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-ab-comparison')).toContainText('다음 관측 봉 시가')
})

test('legacy reports and saved snapshots keep execution model and event clocks unknown', async ({ page }) => {
  await mockExecution(page, { legacy: true })
  await page.goto('/strategy')
  const card = maCard(page)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('backtest-execution-assumptions')).toContainText('체결 모델 정보 없음 · 이전 결과')
  await expect(card.getByTestId('backtest-execution-assumptions')).toContainText('이전 응답에는 체결 진단 정보가 없습니다')
  await card.getByRole('button', { name: '현재 결과를 A로 저장' }).click()
  await page.reload()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-ab-comparison')).toContainText('체결 모델 정보 없음 · 이전 결과')
  const row = card.getByRole('table', { name: '백테스트 거래 목록' }).getByRole('row').filter({ hasText: 'fixture buy' })
  await expect(row.getByRole('cell', { name: '정보 없음', exact: true })).toHaveCount(2)
})

test('mobile research results keep controls and timing table inside the strategy card', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await mockExecution(page)
  await page.goto('/strategy')
  const card = maCard(page)
  await selectOption(page, card, '체결 모델', '다음 관측 봉 시가 체결 가정')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-backtest-results')).toBeVisible()
  const cardBox = await card.boundingBox()
  for (const label of ['체결 모델', '봉 내부 청산 정책']) {
    const box = await card.getByRole('combobox', { name: label }).boundingBox()
    expect(box).not.toBeNull()
    expect(box!.x + box!.width).toBeLessThanOrEqual(cardBox!.x + cardBox!.width)
  }
  const metrics = await card.evaluate((element) => ({ width: element.clientWidth, scroll: element.scrollWidth }))
  expect(metrics.scroll).toBeLessThanOrEqual(metrics.width + 1)
})


test('LTH minute preview carries the shared execution assumptions while daily remains diagnostic', async ({ page }) => {
  const requests: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', previewRequests: requests })
  await page.goto('/strategy')
  const card = page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
  await selectOption(page, card, '체결 모델', '다음 관측 봉 시가 체결 가정')
  await selectOption(page, card, '봉 내부 청산 정책', '손절 우선 · 보수적 모의 청산')
  await card.getByRole('spinbutton', { name: '연구 손절(%)' }).fill('2.5')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-backtest-results')).toBeVisible()
  expect(requests[0]).toMatchObject({ interval: '1m', assumptions: { executionModel: 'nextOpen', intrabarExitPolicy: 'stopFirst', stopLossPct: 2.5 } })
  await selectOption(page, card, '봉 단위', '일봉')
  await expect(card.getByTestId('strategy-backtest-results')).toHaveCount(0)
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-replay-diagnostic')).toBeVisible()
  await expect(card.getByTestId('strategy-backtest-results')).toHaveCount(0)
})
