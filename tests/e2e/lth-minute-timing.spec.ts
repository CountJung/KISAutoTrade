import { expect, test, type Page } from '@playwright/test'
import type { LthMinuteTiming } from '../../src/api/types'
import { mockApi } from './fixtures/strategyResearchFixture'

function timing(patch: Partial<LthMinuteTiming> = {}): LthMinuteTiming {
  return {
    diagnosticVersion: 'lth-minute-timing-v1', labelConvention: 'barClose', market: 'us', exchangeTimezone: 'America/New_York',
    referenceAsOf: '2026-03-09T13:33:30Z', legacySessionPolicy: 'legacyFixedKstUnverified',
    sourceRows: 5, selectedRows: 4, usedRows: 3, priceExcludedRows: 1, inputLimit: 512, processedRows: 3, inputTruncated: false,
    parsedRows: 3, invalidRows: 0, assumedKstRows: 0, duplicateInstants: 0, outOfOrderRows: 0, nonMinuteAlignedRows: 0,
    regularRows: 3, offSessionRows: 0, weekendRows: 0, unfinishedRows: 1, marketUtcOffsetsMinutes: [-240],
    firstBarStartUtc: '2026-03-09T13:30:00Z', lastBarStartUtc: '2026-03-09T13:33:00Z',
    firstCloseAvailableAtUtc: '2026-03-09T13:31:00Z', lastCloseAvailableAtUtc: '2026-03-09T13:34:00Z',
    firstMarketLocal: '2026-03-09T09:31:00-04:00', lastMarketLocal: '2026-03-09T09:34:00-04:00',
    observedDaysCount: 1, dayRowLimit: 8, omittedDayRows: 0, interiorUnobservedSlots: 1, maxInteriorGapSlots: 1,
    days: [{ marketDate: '2026-03-09', utcOffsetMinutes: -240, regularUniqueBars: 3,
      firstCloseMarketLocal: '2026-03-09T09:31:00-04:00', lastCloseMarketLocal: '2026-03-09T09:34:00-04:00',
      ordinaryWindowExpectedSlots: 390, interiorUnobservedSlots: 1, maxInteriorGapSlots: 1,
      leadingUnobservedWindowSlots: 0, trailingUnobservedWindowSlots: 386 }],
    examples: [{ kind: 'interiorGap', label: '2026-03-09T09:32:00-04:00' }],
    calendarVerified: false, providerCoverageVerified: false, requestedPeriodCoverageVerified: false, actualPublicationVerified: false,
    limitations: ['synthetic fixture · 실제 공급원 자료 완전성을 증명하지 않습니다.'], ...patch,
  }
}
const card = (page: Page) => page.locator('.MuiPaper-root').filter({ hasText: 'LeveragedTrendHoldStrategy' }).first()
async function preview(page: Page) {
  await card(page).getByRole('button', { name: '미리보기 계산', exact: true }).click()
  await expect(card(page).getByTestId('replay-assessment-notice')).toBeVisible()
}

test('minute diagnostics distinguish close labels, DST offset, source exclusions and unobserved slots', async ({ page }) => {
  const writes: unknown[] = []
  await mockApi(page, { activeBroker: 'toss', leveragedTiming: timing(), updateRequests: writes })
  const forbidden: string[] = []
  page.on('request', (request) => { if (/\/api\/(?:orders|trading\/(?:start|stop))/.test(request.url())) forbidden.push(request.url()) })
  await page.goto('/strategy'); await preview(page)
  const panel = card(page).getByTestId('lth-minute-timing')
  await expect(panel).toContainText('America/New_York')
  await expect(panel).toContainText('봉 종료 표기 · UTC 오프셋(분) -240')
  await expect(panel).toContainText('공급 5행 · 선택 4행 · 실행 관측 3행 · 가격 제외 1행')
  await expect(panel).toContainText('내부 미관측 1분 슬롯 1')
  await expect(panel).toContainText('기준 시각 뒤 종가 1')
  await expect(panel).toContainText('봉 시작 UTC 2026-03-09T13:30:00Z')
  await expect(panel).toContainText('종가 공개 가능 UTC 2026-03-09T13:31:00Z')
  await expect(panel).toContainText('일반 시간창 390슬롯')
  await expect(panel).toContainText('표본 앞 시간창 0 / 뒤 386슬롯')
  await expect(panel).toContainText('실제 휴장/조기마감 달력 미확인')
  await expect(panel).toContainText('기존 고정 KST 전략 시계·신호·체결 계산은 유지')
  await expect(card(page).getByTestId('strategy-backtest-results')).toBeVisible()
  expect(writes).toEqual([]); expect(forbidden).toEqual([])
})

test('A/B and reload preserve the diagnostic object without upgrading eligibility', async ({ page }) => {
  const value = timing()
  await mockApi(page, { activeBroker: 'toss', leveragedTiming: value })
  await page.goto('/strategy'); await preview(page)
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  const stored = await page.evaluate(() => {
    const key = Object.keys(localStorage).find((key) => key.startsWith('act:strategy-experiments:v1:') && !key.endsWith(':index'))
    return key ? JSON.parse(localStorage.getItem(key)!).A.replay : null
  })
  expect(stored.assessment.minuteTiming).toEqual(value)
  expect(stored.assessment.performanceStatus).toBe('sampleOnly')
  expect(stored.lookAheadSafe).toBe(false)
  await page.reload(); await preview(page)
  const saved = card(page).getByTestId('strategy-ab-comparison')
  await expect(saved.getByTestId('lth-minute-timing')).toContainText('내부 미관측 1분 슬롯 1')
  await expect(saved).toContainText('분봉 표본 ·')
})

test('different time-label conventions warn even when source and dates match', async ({ page }) => {
  const options = { activeBroker: 'toss' as const, leveragedTiming: timing() }
  await mockApi(page, options); await page.goto('/strategy'); await preview(page)
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  options.leveragedTiming = timing({ labelConvention: 'barStart' })
  await preview(page)
  await expect(card(page).getByTestId('replay-assessment-notice').getByTestId('lth-minute-timing')).toContainText('봉 시작 표기')
  await card(page).getByRole('button', { name: '현재 결과를 B로 저장', exact: true }).click()
  await expect(card(page).getByTestId('strategy-ab-comparison')).toContainText('시각 진단 버전·봉 표기·시간대 또는 검증 가정이 다릅니다')
})

test('older minute responses and saved slots explicitly lack time diagnostics', async ({ page }) => {
  await mockApi(page, { activeBroker: 'toss' }); await page.goto('/strategy'); await preview(page)
  await expect(card(page).getByTestId('lth-minute-timing-missing')).toBeVisible()
  await expect(card(page).getByTestId('lth-minute-timing')).toHaveCount(0)
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await expect(card(page).getByTestId('strategy-ab-comparison').getByTestId('lth-minute-timing-missing')).toBeVisible()
})

test('an unknown future diagnostic version stays unknown rather than crashing or certifying coverage', async ({ page }) => {
  const errors: string[] = []; page.on('pageerror', (error) => errors.push(error.message))
  await mockApi(page, { activeBroker: 'toss', leveragedTiming: timing({ diagnosticVersion: 'future-v2' }) })
  await page.goto('/strategy'); await preview(page)
  await expect(card(page).getByTestId('lth-minute-timing-unsupported')).toBeVisible()
  await expect(card(page).getByTestId('lth-minute-timing')).toHaveCount(0)
  expect(errors).toEqual([])
})

test('empty minute data keeps notEvaluable while showing time diagnostic counters', async ({ page }) => {
  await mockApi(page, { activeBroker: 'toss', leveragedZero: true, leveragedTiming: timing({
    sourceRows: 0, selectedRows: 0, usedRows: 0, processedRows: 0, parsedRows: 0, priceExcludedRows: 0,
    regularRows: 0, unfinishedRows: 0, observedDaysCount: 0, days: [], examples: [],
    firstMarketLocal: null, lastMarketLocal: null, firstBarStartUtc: null, lastBarStartUtc: null,
    firstCloseAvailableAtUtc: null, lastCloseAvailableAtUtc: null, interiorUnobservedSlots: 0, maxInteriorGapSlots: 0,
  }) })
  await page.goto('/strategy'); await preview(page)
  await expect(card(page).getByTestId('lth-minute-timing')).toContainText('진단 처리 0/512행')
  await expect(card(page).getByTestId('strategy-replay-diagnostic')).toContainText('성과 평가 불가')
  await expect(card(page).getByTestId('strategy-backtest-results')).toHaveCount(0)
  await expect(card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true })).toHaveCount(0)
})

for (const [field, malformed] of [['days', [null]], ['examples', [null]], ['limitations', 'invalid']] as const) {
  test(`malformed ${field} remains an unsupported diagnostic after A/B saving`, async ({ page }) => {
    const errors: string[] = []; page.on('pageerror', (error) => errors.push(error.message))
    await mockApi(page, { activeBroker: 'toss', leveragedTiming: Object.assign(timing(), { [field]: malformed }) })
    await page.goto('/strategy'); await preview(page)
    await expect(card(page).getByTestId('lth-minute-timing-unsupported')).toBeVisible()
    await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
    await page.reload(); await preview(page)
    await expect(card(page).getByTestId('strategy-ab-comparison').getByTestId('lth-minute-timing-unsupported')).toBeVisible()
    expect(errors).toEqual([])
  })
}

test('daily diagnostic never displays minute-time results or performance', async ({ page }) => {
  await mockApi(page, { activeBroker: 'toss', leveragedTiming: timing() }); await page.goto('/strategy')
  await card(page).getByRole('combobox', { name: '봉 단위' }).click()
  await page.getByRole('option', { name: '일봉', exact: true }).click(); await preview(page)
  await expect(card(page).getByTestId('lth-minute-timing')).toHaveCount(0)
  await expect(card(page).getByTestId('lth-minute-timing-missing')).toHaveCount(0)
  await expect(card(page).getByTestId('strategy-backtest-results')).toHaveCount(0)
})

test('mobile diagnostics wrap timestamps and show truncation without horizontal overflow', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  const value = timing({ inputTruncated: true, usedRows: 600, processedRows: 512, observedDaysCount: 10, omittedDayRows: 2,
    examples: [{ kind: 'invalidTimestamp', label: 'x'.repeat(64) }] })
  value.days = Array.from({ length: 8 }, (_, index) => ({ ...value.days[0], marketDate: `2026-03-${String(9 + index).padStart(2, '0')}` }))
  await mockApi(page, { activeBroker: 'toss', leveragedTiming: value }); await page.goto('/strategy'); await preview(page)
  const panel = card(page).getByTestId('lth-minute-timing')
  await panel.scrollIntoViewIfNeeded()
  await expect(panel).toContainText('진단 입력 잘림')
  await expect(panel).toContainText('날짜 진단 8/10개 · 생략 2개')
  expect(await panel.evaluate((element) => element.scrollWidth <= element.clientWidth + 1)).toBe(true)
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
  await card(page).getByRole('button', { name: '현재 결과를 A로 저장', exact: true }).click()
  await expect(card(page).getByTestId('strategy-ab-comparison').getByTestId('lth-minute-timing')).toContainText('날짜 진단 2/10개')
})
