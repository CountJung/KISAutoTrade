import { expect, test } from '@playwright/test'
import type { ChartCandle } from '../../src/api/types'
import { historyFixture, mockApi, readyPreparation } from './fixtures/strategyResearchFixture'

test('Strategy initial viewport keeps visible main scrollbar gutter', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  const main = page.getByTestId('app-main-scroll')
  await expect(main).toBeVisible()
  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  await expect(page.getByTestId('app-main-scroll-rail')).toBeVisible()
  await expect(page.getByTestId('app-main-scroll-thumb')).toBeVisible()

  const metrics = await main.evaluate((el) => {
    const style = window.getComputedStyle(el)
    return {
      overflowY: style.overflowY,
      scrollbarGutter: style.scrollbarGutter,
      scrollHeight: el.scrollHeight,
      clientHeight: el.clientHeight,
    }
  })
  const thumbHeight = await page.getByTestId('app-main-scroll-thumb').evaluate((el) => el.getBoundingClientRect().height)

  expect(metrics.overflowY).toBe('scroll')
  expect(metrics.scrollbarGutter).toContain('stable')
  expect(metrics.scrollHeight).toBeGreaterThan(metrics.clientHeight)
  expect(thumbHeight).toBeGreaterThan(0)
})

test('Strategy scrollbar appears after delayed strategy content loads', async ({ page }) => {
  await mockApi(page, { strategyDelayMs: 250 })
  await page.goto('/strategy')

  const main = page.getByTestId('app-main-scroll')
  await expect(main).toBeVisible()
  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  await expect(page.getByTestId('app-main-scroll-rail')).toBeVisible()
  await expect(page.getByTestId('app-main-scroll-thumb')).toBeVisible()

  await expect
    .poll(() => main.evaluate((el) => el.scrollHeight > el.clientHeight))
    .toBe(true)
})

test('All strategy cards use the full content width', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 1000 })
  await mockApi(page)
  await page.goto('/strategy')

  const cards = page.getByTestId('strategy-card-grid')
  await expect(cards).toHaveCount(13)
  const containerBox = await cards.first().locator('..').boundingBox()
  const cardBoxes = await cards.evaluateAll((elements) => elements.map((element) => {
    const rect = element.getBoundingClientRect()
    return { x: rect.x, width: rect.width }
  }))

  expect(containerBox).not.toBeNull()
  expect(cardBoxes).toHaveLength(13)
  for (const box of cardBoxes) {
    expect(box.x).toBeCloseTo(cardBoxes[0].x, 0)
    expect(box.width).toBeCloseTo(cardBoxes[0].width, 0)
    expect(box.width).toBeGreaterThan((containerBox?.width ?? 0) * 0.95)
  }
})

test('Strategy saves with the profile and broker scope that produced the edit', async ({ page }) => {
  const updateRequests: unknown[] = []
  await mockApi(page, { updateRequests })
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await card.getByLabel('1회 수량').fill('2')
  await card.getByRole('button', { name: '변경사항 저장' }).click()

  await expect.poll(() => updateRequests.length).toBe(1)
  expect(updateRequests[0]).toMatchObject({
    id: 'ma_cross_default',
    expectedProfileId: 'paper',
    expectedBrokerId: 'kis',
    expectedBrokerAccountId: '12345678-01',
    orderQuantity: 2,
  })
})

test('Strategy discards unsaved edits when the active account scope changes', async ({ page }) => {
  const scopeController: { current: 'A' | 'B' } = { current: 'A' }
  const updateRequests: unknown[] = []
  await mockApi(page, { scopeController, updateRequests })
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await card.getByLabel('1회 수량').fill('2')
  await page.getByRole('button', { name: 'Settings' }).click()
  const profileB = page.getByText('계좌 B', { exact: true })
    .locator('xpath=ancestor::*[contains(@class,"MuiPaper-root")][1]')
  await profileB.getByRole('button').first().click()
  await expect(page.getByText('활성: 계좌 B')).toBeVisible()
  await page.getByRole('button', { name: 'Strategy' }).click()

  const refreshedCard = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await expect(refreshedCard.getByLabel('1회 수량')).toHaveValue('9')
  expect(updateRequests).toHaveLength(0)
})

test('Strategy simulation controls stack without overflow on a narrow viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await mockApi(page)
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  const quantity = card.getByLabel('1회 수량')
  const tickerSelect = card.getByRole('combobox', { name: '시뮬레이션 티커' })
  const previewButton = card.getByRole('button', { name: '미리보기 계산' })
  const intervalSelect = card.getByRole('combobox', { name: '봉 단위' })
  const rangeSelect = card.getByRole('combobox', { name: '분석 구간' })
  await card.scrollIntoViewIfNeeded()
  await expect(quantity).toBeVisible()
  await expect(tickerSelect).toBeVisible()
  await expect(previewButton).toBeVisible()
  await expect(intervalSelect).toBeVisible()
  await expect(rangeSelect).toBeVisible()

  const cardBox = await card.boundingBox()
  const quantityBox = await quantity.boundingBox()
  const selectBox = await tickerSelect.boundingBox()
  const buttonBox = await previewButton.boundingBox()
  const intervalBox = await intervalSelect.boundingBox()
  const rangeBox = await rangeSelect.boundingBox()
  expect(cardBox).not.toBeNull()
  expect(quantityBox?.width ?? 0).toBeGreaterThan((cardBox?.width ?? 0) * 0.75)
  expect(buttonBox?.y ?? 0).toBeGreaterThan(selectBox?.y ?? 0)
  expect(intervalBox?.y ?? 0).toBeGreaterThan(selectBox?.y ?? 0)
  expect(rangeBox?.y ?? 0).toBeGreaterThan(intervalBox?.y ?? 0)
  expect((buttonBox?.x ?? 0) + (buttonBox?.width ?? 0)).toBeLessThanOrEqual((cardBox?.x ?? 0) + (cardBox?.width ?? 0))
})

test('main scrollbar thumb can be dragged with a pointer', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  const main = page.getByTestId('app-main-scroll')
  const thumb = page.getByTestId('app-main-scroll-thumb')
  await expect(main).toBeVisible()
  await expect(thumb).toBeVisible()

  const before = await main.evaluate((el) => el.scrollTop)
  const box = await thumb.boundingBox()
  expect(box).not.toBeNull()
  if (!box) return

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 160, { steps: 8 })
  await page.mouse.up()

  await expect
    .poll(() => main.evaluate((el) => el.scrollTop))
    .toBeGreaterThan(before + 20)
})

test('main scroll position is restored after visiting a short page', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  const main = page.getByTestId('app-main-scroll')
  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  const savedTop = await main.evaluate((el) => {
    el.scrollTop = 420
    el.dispatchEvent(new Event('scroll'))
    return el.scrollTop
  })
  expect(savedTop).toBeGreaterThan(100)

  await page.getByRole('button', { name: 'Log' }).click()
  await expect(page.getByRole('heading', { name: 'Log' })).toBeVisible()
  await expect.poll(() => main.evaluate((el) => el.scrollTop)).toBe(0)

  await page.getByRole('button', { name: 'Strategy' }).click()
  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  await expect
    .poll(() => main.evaluate((el) => el.scrollTop))
    .toBeGreaterThan(savedTop - 24)
})

test('Leveraged strategy editor uses single target ticker model', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  await expect(page.getByText('SOXL', { exact: true })).toBeVisible()
  await expect(page.getByLabel('초기 손절(%)')).toBeVisible()
  await expect(page.getByLabel('실패 판정 관측치')).toBeVisible()
  await expect(page.getByLabel('추적손절(%)')).toBeVisible()
  await expect(page.getByLabel('추적 활성 수익(%)')).toBeVisible()
  await expect(page.getByLabel('본전 보호 버퍼(%)')).toBeVisible()
  await expect(page.getByLabel('최소 보유 관측치')).toBeVisible()
  await expect(page.getByText('급반등 단독 진입 사용')).toBeVisible()
  await expect(page.getByLabel('최근 관측치')).toBeVisible()
  await expect(page.getByLabel('선행 급락(%)')).toBeVisible()
  await expect(page.getByLabel('저점 회복(%)')).toBeVisible()
  await expect(page.getByLabel('저점 후 허용 관측치')).toBeVisible()
  await expect(page.getByRole('button', { name: '대상 추가' }).first()).toBeVisible()
  await expect(page.getByText('운용 모드')).toHaveCount(0)
  await expect(page.getByText('기초지수')).toHaveCount(0)
  await expect(page.getByText('숏 실험')).toHaveCount(0)
})

test('Bollinger enhancement saves on the existing leveraged strategy and reaches preview', async ({ page }) => {
  const updateRequests: unknown[] = []
  const previewRequests: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', updateRequests, previewRequests })
  await page.goto('/strategy')
  await page.getByLabel('볼린저 스퀴즈·추세 돌파 보강').check()
  await page.getByLabel('볼린저 기간(봉)').fill('25')
  await page.getByLabel('장기 추세 기간(봉)').fill('80')
  const card = page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
  await card.getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await expect.poll(() => previewRequests.length).toBe(1)
  expect(previewRequests[0]).toMatchObject({ interval: '1m', params: { bollinger: { enabled: true, period: 25, trend_period: 80 } } })
  await card.getByRole('button', { name: '변경사항 저장', exact: true }).click()
  await expect.poll(() => updateRequests.length).toBe(1)
  expect(updateRequests[0]).toMatchObject({ id: 'leveraged_trend_hold_default', params: { bollinger: { enabled: true, period: 25, trend_period: 80 } } })
})

test('Leveraged strategy preview can switch between configured tickers', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
  await expect(card.getByText('전략 미리보기')).toBeVisible()
  const tickerSelect = card.getByRole('combobox', { name: '시뮬레이션 티커' })
  await expect(tickerSelect).toContainText('SOXL')

  await tickerSelect.click()
  await page.getByRole('option', { name: /KORU/ }).click()

  await expect(tickerSelect).toContainText('KORU')
})

test('Leveraged strategy preview runs selected ticker and renders signal chart', async ({ page }) => {
  const previewRequests: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', previewRequests })
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
  await card.getByRole('combobox', { name: '봉 단위' }).click()
  await page.getByRole('option', { name: '일봉', exact: true }).click()
  await card.getByRole('combobox', { name: '분석 구간' }).click()
  await page.getByRole('option', { name: '최근 50봉', exact: true }).click()
  await card.getByRole('button', { name: '미리보기 계산' }).click()

  await expect(card.getByText('mock preview signals')).toBeVisible()
  await expect(card.getByTestId('lth-preview-chart')).toBeVisible()
  await expect(card.getByTestId('lth-preview-chart').locator('canvas').first()).toBeVisible()
  await expect(card.getByRole('checkbox', { name: '종가 선 그래프 표시' })).toBeChecked()
  await expect(card.getByText('종가 선 그래프')).toBeVisible()
  await expect(card.getByText('한 손가락 좌우 이동 · 두 손가락 확대/축소')).toBeVisible()
  await expect(card.getByRole('button', { name: '차트 확대' })).toBeVisible()
  await expect(card.getByRole('button', { name: '차트 축소' })).toBeVisible()
  await expect(card.getByTestId('strategy-replay-diagnostic')).toBeVisible()
  await expect(card.getByRole('button', { name: '현재 결과를 A로 저장' })).toHaveCount(0)
  expect(previewRequests).toHaveLength(1)
  expect(previewRequests[0]).toMatchObject({ symbol: 'SOXL', interval: '1d', count: 50 })
})

test('Generic strategy card preview runs with edited card settings', async ({ page }) => {
  const genericPreviewRequests: unknown[] = []
  const chartRequests: string[] = []
  await mockApi(page, { genericPreviewRequests, chartRequests })
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await expect(card.getByText('전략 미리보기')).toBeVisible()
  await card.getByRole('combobox', { name: '봉 단위' }).click()
  await page.getByRole('option', { name: '주봉', exact: true }).click()
  await card.getByRole('combobox', { name: '분석 구간' }).click()
  await page.getByRole('option', { name: '최근 100봉', exact: true }).click()
  await card.getByRole('button', { name: '미리보기 계산' }).click()

  await expect(card.getByText('mock generic preview signals')).toBeVisible()
  await expect(card.getByText(/KIS 주봉 · 실제 4봉 \/ 요청 100봉/)).toBeVisible()
  await expect(card.getByText(/매수 07\/02 00:00/)).toBeVisible()
  await expect(card.getByText(/매도 07\/04 00:00/)).toBeVisible()
  await expect(card.getByTestId('lth-preview-chart').locator('canvas').first()).toBeVisible()
  await expect(card.getByText('한 손가락 좌우 이동 · 두 손가락 확대/축소')).toBeVisible()
  expect(await card.getByTestId('lth-preview-chart').evaluate((element) => getComputedStyle(element).touchAction)).toBe('pan-y')
  expect(chartRequests).toHaveLength(1)
  expect(chartRequests[0]).toContain('period=W')
  expect(chartRequests[0]).toContain('count=352')
  expect(genericPreviewRequests).toHaveLength(1)
  expect(genericPreviewRequests[0]).toMatchObject({
    strategyId: 'ma_cross_default',
    symbol: '000002',
    orderQuantity: 1,
    interval: 'W',
    brokerId: 'kis',
    brokerAccountId: '12345678-01',
    historyCandles: [],
  })
  await expect(card.getByTestId('strategy-backtest-results')).toBeVisible()
  await expect(card.getByText('원시 신호 3개 · 주문 가능 2개 · 체결 가정 2개 · 차단 1개')).toBeVisible()
  await expect(card.getByRole('img', { name: /자산 곡선/ })).toBeVisible()
})

test('Strategy research assumptions and A/B experiments are reproducible in the same data range', async ({ page }) => {
  const genericPreviewRequests: Array<{ assumptions?: { initialCapitalKrw?: number; feeBps?: number } }> = []
  await mockApi(page, { genericPreviewRequests })
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await card.getByRole('spinbutton', { name: '초기자본(원)' }).fill('20000000')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-backtest-results')).toBeVisible()
  expect(genericPreviewRequests[0]?.assumptions).toMatchObject({ initialCapitalKrw: 20_000_000 })

  await card.getByRole('button', { name: '현재 결과를 A로 저장' }).click()
  await expect(card.getByTestId('strategy-ab-comparison').getByText('실험 결과 A')).toBeVisible()
  await card.getByRole('spinbutton', { name: '수수료(bps)' }).fill('7.5')
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect.poll(() => genericPreviewRequests.length).toBe(2)
  expect(genericPreviewRequests[1]?.assumptions).toMatchObject({ feeBps: 7.5 })
  await card.getByRole('button', { name: '현재 결과를 B로 저장' }).click()
  await expect(card.getByTestId('strategy-ab-comparison').getByText('실험 결과 B')).toBeVisible()

  await page.reload()
  const reloadedCard = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await reloadedCard.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(reloadedCard.getByTestId('strategy-ab-comparison')).toBeVisible()
})

test('Generic strategy preview discards an in-flight result after parameters change', async ({ page }) => {
  const genericPreviewRequests: Array<{ params?: Record<string, unknown> }> = []
  await mockApi(page, { genericPreviewRequests, genericPreviewDelayMs: 300 })
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  const previewButton = card.getByRole('button', { name: '미리보기 계산' })
  await previewButton.click()
  await expect.poll(() => genericPreviewRequests.length).toBe(1)

  await card.getByRole('spinbutton', { name: '단기 MA', exact: true }).fill('7')
  await page.waitForTimeout(400)
  await expect(card.getByText('mock generic preview signals')).toHaveCount(0)

  await previewButton.click()
  await expect(card.getByText('mock generic preview signals')).toBeVisible()
  expect(genericPreviewRequests).toHaveLength(2)
  expect(genericPreviewRequests[1]?.params).toMatchObject({ short_period: 7 })
})

test('Generic Toss strategy preview uses the selected one-minute interval and range', async ({ page }) => {
  const chartRequests: string[] = []
  const genericPreviewRequests: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', chartRequests, genericPreviewRequests })
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  const intervalSelect = card.getByRole('combobox', { name: '봉 단위' })
  await intervalSelect.click()
  await expect(page.getByRole('option', { name: '1분봉', exact: true })).toBeVisible()
  await expect(page.getByRole('option', { name: '일봉', exact: true })).toBeVisible()
  await expect(page.getByRole('option', { name: '주봉', exact: true })).toHaveCount(0)
  await page.getByRole('option', { name: '1분봉', exact: true }).click()

  await card.getByRole('combobox', { name: '분석 구간' }).click()
  await page.getByRole('option', { name: '최근 200봉', exact: true }).click()
  await card.getByRole('button', { name: '미리보기 계산' }).click()

  await expect(card.getByText(/Toss 1분봉 · 실제 4봉 \/ 요청 200봉/)).toBeVisible()
  expect(chartRequests).toHaveLength(1)
  expect(chartRequests[0]).toContain('interval=1m')
  expect(chartRequests[0]).toContain('count=200')
  expect(genericPreviewRequests).toHaveLength(1)
  expect(genericPreviewRequests[0]).toMatchObject({ historyCandles: [] })
})

test('Leveraged strategy preview discards an in-flight result after parameters change', async ({ page }) => {
  const previewRequests: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', previewRequests, leveragedPreviewDelayMs: 300 })
  await page.goto('/strategy')

  const card = page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
  const previewButton = card.getByRole('button', { name: '미리보기 계산' })
  await previewButton.click()
  await expect.poll(() => previewRequests.length).toBe(1)

  await card.getByRole('spinbutton', { name: '진입 민감도' }).fill('2')
  await page.waitForTimeout(400)
  await expect(card.getByText('mock preview signals')).toHaveCount(0)
})

test('Generic preview separates sorted history from the complete requested evaluation window', async ({ page }) => {
  const genericPreviewRequests: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[]; warmupCount?: number }> = []
  const chartRequests: string[] = []
  const ordered = historyFixture(302)
  await mockApi(page, { genericPreviewRequests, chartRequests, chartCandles: [...ordered].reverse().concat(ordered[0]), genericPreparation: readyPreparation })
  await page.goto('/strategy')
  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-evaluation-range')).toHaveText('요청 분석 50봉 · 실제 평가 50봉 · 사전 자료 252봉')
  await expect(card.getByTestId('strategy-preparation-status')).toContainText('평가 시작 사전 자료 충분 · 지표 준비 완료 · 매매 조건 불충족')
  await expect(card.getByTestId('strategy-preparation-status')).toContainText('실제 조건 평가 50봉')
  await expect(card.getByTestId('strategy-preparation-status')).toContainText('지표 버퍼 252봉')
  await expect(card.getByTestId('strategy-preparation-status')).toContainText('봉 처리 후 첫 준비 완료 시점')
  expect(chartRequests).toHaveLength(1)
  expect(chartRequests[0]).toContain('count=302')
  expect(genericPreviewRequests[0].candles).toEqual(ordered.slice(-50))
  expect(genericPreviewRequests[0].historyCandles).toEqual(ordered.slice(0, 252))
  expect(genericPreviewRequests[0].warmupCount).toBeUndefined()
})

test('Generic preview retains evaluation bars when the provider returns insufficient history', async ({ page }) => {
  const genericPreviewRequests: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[] }> = []
  const chartRequests: string[] = []
  const ordered = historyFixture(120)
  await mockApi(page, { genericPreviewRequests, chartRequests, chartCandles: ordered, genericPreparation: {
    ...readyPreparation, providedHistoryBars: 20, availableHistoryBars: 20, readyAtStart: false, readyAtEnd: false,
    firstReadyTime: null, unreadyEvaluationBars: 100, evaluatedBars: 0, historyStatus: 'insufficient', indicatorStatus: 'warmingUp', outcome: 'notEvaluable',
  } })
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto('/strategy')
  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await card.getByRole('combobox', { name: '분석 구간' }).click()
  await page.getByRole('option', { name: '최근 100봉', exact: true }).click()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-evaluation-range')).toHaveText('요청 분석 100봉 · 실제 평가 100봉 · 사전 자료 20봉')
  const status = card.getByTestId('strategy-preparation-status')
  await expect(status).toContainText('평가 시작 사전 자료 부족 · 지표 준비 중 · 평가 불가')
  await expect(status).toContainText('필요 252봉')
  await expect(status).toContainText('봉 처리 후 지표 준비 미확인 100봉')
  await expect(status).toContainText('실제 조건 평가 0봉')
  await expect(status).toContainText('지표 버퍼 20봉')
  expect(genericPreviewRequests[0].candles).toEqual(ordered.slice(-100))
  expect(genericPreviewRequests[0].historyCandles).toEqual(ordered.slice(0, 20))
  expect(chartRequests).toHaveLength(1)
  expect(await status.evaluate((element) => element.scrollWidth <= element.clientWidth)).toBe(true)
  const statusBox = await status.boundingBox()
  const cardBox = await card.boundingBox()
  expect(statusBox!.x + statusBox!.width).toBeLessThanOrEqual(cardBox!.x + cardBox!.width + 1)
})

test('Toss capped response preserves all 200 evaluation bars and reports unfilled signals separately', async ({ page }) => {
  const genericPreviewRequests: Array<{ candles: ChartCandle[]; historyCandles: ChartCandle[] }> = []
  const chartRequests: string[] = []
  const ordered = historyFixture(200)
  await mockApi(page, { activeBroker: 'toss', genericPreviewRequests, chartRequests, chartCandles: ordered, genericPreparation: {
    ...readyPreparation, requiredHistoryBars: 200, providedHistoryBars: 0, availableHistoryBars: 0, readyAtStart: false,
    unreadyEvaluationBars: 199, evaluatedBars: 1, historyStatus: 'insufficient', outcome: 'noTrades',
  } })
  await page.goto('/strategy')
  const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
  await card.getByRole('combobox', { name: '분석 구간' }).click()
  await page.getByRole('option', { name: '최근 200봉', exact: true }).click()
  await card.getByRole('button', { name: '미리보기 계산' }).click()
  await expect(card.getByTestId('strategy-evaluation-range')).toHaveText('요청 분석 200봉 · 실제 평가 200봉 · 사전 자료 0봉')
  await expect(card.getByTestId('strategy-preparation-status')).toContainText('지표 준비 완료 · 신호 발생 · 체결 없음')
  await expect(card.getByTestId('strategy-preparation-status')).toContainText('필요 200봉')
  await expect(card.getByText(/Toss는 한 페이지 최대 200봉/)).toBeVisible()
  expect(chartRequests).toHaveLength(1)
  expect(chartRequests[0]).toContain('count=200')
  expect(genericPreviewRequests[0].candles).toEqual(ordered)
  expect(genericPreviewRequests[0].historyCandles).toEqual([])
})

test('Daily boundary strategies restrict replay to daily candles and explain close execution', async ({ page }) => {
  await mockApi(page, { activeBroker: 'toss' })
  await page.goto('/strategy')
  for (const name of ['StrongCloseStrategy', 'VolatilityExpansionStrategy']) {
    const card = page.locator('.MuiPaper-root').filter({ hasText: name }).first()
    await expect(card.getByTestId('daily-event-replay-note')).toContainText('일봉만 지원하며 체결은 종가 가정')
    await card.getByRole('combobox', { name: '봉 단위' }).click()
    await expect(page.getByRole('option', { name: '일봉', exact: true })).toBeVisible()
    await expect(page.getByRole('option', { name: '1분봉', exact: true })).toHaveCount(0)
    await page.getByRole('option', { name: '일봉', exact: true }).click()
  }
})

test('Sidebar trading action toggles auto trading from strategy page', async ({ page }) => {
  await mockApi(page)
  await page.goto('/strategy')

  await expect(page.getByText('레버리지 대상 ETF')).toBeVisible()
  const sidebar = page.getByRole('navigation')
  const startButton = sidebar.getByRole('button', { name: '자동매매 시작' })
  await expect(startButton).toBeVisible()

  await startButton.click()
  await expect(sidebar.getByText('자동매매 실행 중')).toBeVisible()
  const stopButton = sidebar.getByRole('button', { name: '자동매매 정지' })
  await expect(stopButton).toBeVisible()

  await stopButton.click()
  await expect(sidebar.getByText('대기 중')).toBeVisible()
  await expect(sidebar.getByRole('button', { name: '자동매매 시작' })).toBeVisible()
})
