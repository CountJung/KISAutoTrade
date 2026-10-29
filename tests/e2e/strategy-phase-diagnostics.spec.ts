import { expect, test, type Page } from '@playwright/test'
import { mockApi, mockResearchResult } from './fixtures/strategyResearchFixture'

const diagnostic = {
  version: 1, policy: 'sameLedgerObservationSplit', observedBars: 700, heldBars: 210,
  exposurePct: 30, startQuantity: 0, endQuantity: 3, pendingAtStart: false, pendingAtEnd: true,
  realizedPnlKrw: 1200, startResidualUnrealizedPnlKrw: 0, endResidualUnrealizedPnlKrw: -200,
  dividendNetKrw: 25, equityChangeKrw: 1025,
}
async function setup(page: Page, mode: 'valid' | 'legacy' | 'invalid' | 'empty' = 'valid') {
  await mockApi(page)
  await page.route('**/api/strategy/preview', async route => {
    const body = route.request().postDataJSON()
    const result = mockResearchResult(body, 'D')
    const phases = result.backtest.phases.map((phase, index) => ({ ...phase,
      diagnostics: mode === 'legacy' ? undefined : mode === 'invalid' ? { ...diagnostic, heldBars: '210' } : {
        ...diagnostic, observedBars: index ? 300 : 700, heldBars: index ? 90 : 210,
      },
    }))
    await route.fulfill({ json: { strategyId: body.strategyId, symbol: body.symbol, candles: body.candles,
      signals: [], generatedAt: '2026-10-29T00:00:00Z', message: 'synthetic phase fixture', ...result,
      backtest: { ...result.backtest, phases: mode === 'empty' ? [] : phases,
        overfitWarning: mode === 'valid' ? '후반 보합 관측 fixture' : 'LEGACY 과최적화 확정 문구',
        // Store diagnostics must survive curve/trade compaction without being recomputed.
        equityCurve: Array.from({ length: 450 }, (_, i) => ({ ...result.backtest.equityCurve[0], time: String(i) })),
        trades: Array.from({ length: 120 }, () => result.backtest.trades[0]),
      },
    } })
  })
}
function card(page: Page) { return page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first() }
async function calculate(page: Page) {
  await page.goto('/strategy')
  await card(page).getByRole('button', { name: '미리보기 계산' }).click()
  return card(page).getByTestId('strategy-phase-results').first()
}

test('flat phases display observation, carry and attribution diagnostics without independent OOS labels', async ({ page }) => {
  await setup(page)
  const panel = await calculate(page)
  await expect(card(page).getByRole('spinbutton', { name: '앞 구간 비율(%)' })).toHaveValue('70')
  await expect(panel.getByTestId('phase-policy')).toContainText('독립 검증 성과로 해석하지 않습니다')
  await expect(panel).toContainText('관측 700봉 · 종가 보유 210봉')
  await expect(panel).toContainText('관측 300봉 · 종가 보유 90봉')
  await expect(panel).toContainText('경계 보유 0 → 3주 · 대기 주문 없음 → 있음')
  await expect(panel).toContainText('완료 매도 0회')
  await expect(panel).toContainText('작은 표본')
  await expect(panel).toContainText('후반 보합 관측 fixture')
  await expect(panel).toContainText('매도 수량에 배분된 원래 진입원가')
})

for (const mode of ['legacy', 'invalid'] as const) {
  test(`${mode} diagnostics remain unknown and suppress obsolete warning`, async ({ page }) => {
    await setup(page, mode)
    const panel = await calculate(page)
    await expect(panel.getByText(/구간 진단 정보 없음/)).toHaveCount(2)
    await expect(panel).not.toContainText('LEGACY 과최적화')
    await expect(panel).not.toContainText('관측 700봉')
    await expect(panel).not.toContainText('NaN')
  })
}

test('A/B saves diagnostics unchanged through compaction and reload, including mixed legacy slot', async ({ page }) => {
  await setup(page)
  await calculate(page)
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장' }).click()
  const snapshot = await page.evaluate(() => {
    const key = Object.keys(localStorage).find(k => k.startsWith('act:strategy-experiments:v1:') && !k.endsWith(':index'))!
    const slots = JSON.parse(localStorage.getItem(key)!)
    const saved = slots.A.backtest
    slots.B = structuredClone(slots.A)
    slots.B.slot = 'B'
    slots.B.backtest.phases.forEach((p: Record<string, unknown>) => delete p.diagnostics)
    slots.B.backtest.overfitWarning = 'LEGACY 과최적화 확정 문구'
    localStorage.setItem(key, JSON.stringify(slots))
    return { trades: saved.trades.length, points: saved.equityCurve.length, diagnostic: saved.phases[0].diagnostics }
  })
  expect(snapshot.trades).toBe(100)
  expect(snapshot.points).toBeLessThanOrEqual(201)
  expect(snapshot.diagnostic).toEqual(diagnostic)
  await page.reload()
  await card(page).getByRole('button', { name: '미리보기 계산' }).click()
  const ab = card(page).getByTestId('strategy-ab-comparison')
  await expect(ab).toContainText('관측 700봉 · 종가 보유 210봉')
  await expect(ab.getByText(/구간 진단 정보 없음/)).toHaveCount(2)
  await expect(ab).not.toContainText('LEGACY 과최적화')
})

test('empty phases remain neutral', async ({ page }) => {
  await setup(page, 'empty')
  const panel = await calculate(page)
  await expect(panel).toContainText('표시할 구간 관측이 없습니다')
  await expect(panel.getByRole('table')).toHaveCount(0)
  await expect(panel).not.toContainText('LEGACY 과최적화')
})

test('375px phases scroll their table without overflowing the document', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  await setup(page)
  const panel = await calculate(page)
  await expect(panel).toBeVisible()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBeTruthy()
})
