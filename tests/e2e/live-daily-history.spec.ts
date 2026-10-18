import { expect, test } from '@playwright/test'
import type { LiveDailyHistoryStatus, TradingStatus } from '../../src/api/types'
import { mockApi } from './fixtures/strategyResearchFixture'

function historyRow(overrides: Partial<LiveDailyHistoryStatus> = {}): LiveDailyHistoryStatus {
  return {
    strategyId: 'fifty_two_week_high_default', symbol: 'SOXQ', brokerId: 'kis', profileId: 'paper',
    marketDate: '2026-10-19', requiredBars: 252, availableBars: 0, state: 'pending',
    source: null, lastCompletedDate: null, lastError: null, nextRetryAt: null,
    ...overrides,
  }
}

function running(dailyHistory?: LiveDailyHistoryStatus[], overrides: Partial<TradingStatus> = {}): Partial<TradingStatus> {
  return { isRunning: true, tradingProfileId: 'paper', tradingBrokerId: 'kis', tradingAccountId: '12345678-01', dailyHistory, ...overrides }
}

for (const route of ['/', '/strategy']) {
  test(`${route} uses the existing ten-second status poll to change pending to ready`, async ({ page }) => {
    await page.clock.install({ time: new Date('2026-10-19T00:00:00Z') })
    let row = historyRow()
    let requests = 0
    const mutations: string[] = []
    page.on('request', (request) => {
      if (/^\/api\/(?:order|trading\/(?:start|stop))(?:\/|$)/.test(new URL(request.url()).pathname)) mutations.push(request.url())
    })
    await mockApi(page, { tradingStatusOverride: () => { requests += 1; return running([row]) } })
    await page.goto(route)
    const panel = page.getByTestId('live-daily-history-panel')
    await expect(panel.getByText('준비 중', { exact: true })).toBeVisible()
    await expect(panel).toContainText('확보 0봉 / 필요 252봉')
    expect(requests).toBe(1)
    row = historyRow({ state: 'ready', availableBars: 252, source: 'KIS completed daily page', lastCompletedDate: '2026-10-16' })
    await page.clock.fastForward(10_100)
    await expect(panel.getByText('자료 준비 완료', { exact: true })).toBeVisible()
    await expect(panel).toContainText('확보 252봉 / 필요 252봉')
    await expect(panel).toContainText('마지막 완료 일봉 2026-10-16')
    expect(requests).toBe(2)
    expect(mutations).toHaveLength(0)
  })
}

test('insufficient and failed daily context explain buy deferral and continuing protective exits', async ({ page }) => {
  await mockApi(page, { tradingStatusOverride: () => running([
    historyRow({ state: 'insufficient', availableBars: 120, source: 'KIS NAS daily', lastCompletedDate: '2026-10-16',
      lastError: '제공 자료가 끝나 252봉 기준에 미달합니다.', nextRetryAt: '2026-10-19T00:15:00Z' }),
    historyRow({ symbol: 'TQQQ', state: 'failed', lastError: '조회 제한: 다음 재시도까지 기다립니다.', nextRetryAt: '2026-10-19T00:15:00Z' }),
  ]) })
  await page.goto('/strategy')
  const panel = page.getByTestId('live-daily-history-panel')
  await expect(panel).toContainText('일봉 부족')
  await expect(panel).toContainText('조회 실패')
  await expect(panel).toContainText('확보 120봉 / 필요 252봉')
  await expect(panel).toContainText('시장 기준일 2026-10-19 · 마지막 완료 일봉 2026-10-16')
  await expect(panel).toContainText('출처 KIS NAS daily')
  await expect(panel).toContainText('9시 15분 0초 KST')
  await expect(panel).toContainText('252봉 기준에 미달')
  await expect(panel).toContainText('신규 매수를 보류합니다. 보유 포지션의 손절·청산 평가는 계속합니다.')
  await expect(page.getByRole('navigation').getByRole('button', { name: '자동매매 정지' })).toBeVisible()
  await expect(panel.getByText('자료 준비 완료', { exact: true })).toHaveCount(0)
  await expect(panel).not.toContainText('0%')
})

for (const scenario of ['stopped', 'legacy', 'empty'] as const) {
  test(`${scenario} daily status stays neutral without inventing readiness`, async ({ page }) => {
    const status = scenario === 'stopped' ? running([historyRow({ state: 'ready', availableBars: 252 })], { isRunning: false })
      : scenario === 'legacy' ? running() : running([])
    await mockApi(page, { tradingStatusOverride: () => status })
    await page.goto('/strategy')
    const panel = page.getByTestId('live-daily-history-panel')
    const text = { stopped: '미실행', legacy: '일봉 준비 상태 미지원', empty: '표시할 일봉 준비 대상이 없습니다.' }
    await expect(panel).toContainText(text[scenario])
    await expect(panel.getByTestId('live-daily-history-row')).toHaveCount(0)
    await expect(panel.getByText('자료 준비 완료', { exact: true })).toHaveCount(0)
  })
}

test('web status shows matching broker and profile identity and excludes other scope rows', async ({ page }) => {
  await mockApi(page, { activeBroker: 'toss', tradingStatusOverride: () => running([
    historyRow({ brokerId: 'toss', profileId: 'toss-live', symbol: 'VISIBLE' }),
    historyRow({ brokerId: 'toss', profileId: 'other-profile', symbol: 'OTHER_PROFILE' }),
    historyRow({ brokerId: 'kis', profileId: 'toss-live', symbol: 'OTHER_BROKER' }),
  ], { tradingProfileId: 'toss-live', tradingBrokerId: 'toss', tradingAccountId: '1' }) })
  await page.goto('/strategy')
  const panel = page.getByTestId('live-daily-history-panel')
  await expect(panel.getByTestId('live-daily-history-scope')).toHaveText('실행 프로파일 toss-live · toss · 계좌 1')
  await expect(panel.getByTestId('live-daily-history-row')).toHaveCount(1)
  await expect(panel).toContainText('VISIBLE')
  await expect(panel).not.toContainText('OTHER_PROFILE')
  await expect(panel).not.toContainText('OTHER_BROKER')
})

test('an active-account mismatch hides stale running history', async ({ page }) => {
  await mockApi(page, { tradingStatusOverride: () => running([historyRow()], { tradingAccountId: 'other-account' }) })
  await page.goto('/strategy')
  const panel = page.getByTestId('live-daily-history-panel')
  await expect(panel).toContainText('활성 계좌와 실행 계좌가 달라')
  await expect(panel.getByTestId('live-daily-history-row')).toHaveCount(0)
})

test('legacy KIS status without profile or dailyHistory remains explicitly unsupported', async ({ page }) => {
  await mockApi(page, { appConfigOverride: { active_profile_id: null },
    tradingStatusOverride: () => running(undefined, { tradingProfileId: null }) })
  await page.goto('/')
  const panel = page.getByTestId('live-daily-history-panel')
  await expect(panel).toContainText('일봉 준비 상태 미지원')
  await expect(panel.getByTestId('live-daily-history-row')).toHaveCount(0)
})

for (const activeProfileId of [null, 'paper']) {
test(`legacy KIS execution without a profile displays only its matching failure with active profile ${activeProfileId}`, async ({ page }) => {
  const reason = '실행 프로파일·계좌를 찾지 못해 완료 일봉 조회를 중단했습니다.'
  await mockApi(page, { appConfigOverride: { active_profile_id: activeProfileId }, tradingStatusOverride: () => running([
    historyRow({ profileId: '', state: 'failed', lastError: reason }),
    historyRow({ profileId: 'paper', symbol: 'OTHER_PROFILE', state: 'failed', lastError: '다른 프로파일' }),
    historyRow({ profileId: '', brokerId: 'toss', symbol: 'OTHER_BROKER', state: 'failed' }),
    historyRow({ profileId: '', symbol: 'UNVERIFIED', state: 'ready', availableBars: 252 }),
  ], { tradingProfileId: null }) })
  await page.goto('/')
  const panel = page.getByTestId('live-daily-history-panel')
  await expect(panel).toContainText('미지정(기존 KIS 설정)')
  await expect(panel).toContainText(reason)
  await expect(panel).toContainText('조회 실패')
  await expect(panel.getByTestId('live-daily-history-row')).toHaveCount(1)
  await expect(panel).not.toContainText('OTHER_PROFILE')
  await expect(panel).not.toContainText('OTHER_BROKER')
  await expect(panel).not.toContainText('UNVERIFIED')
  if (activeProfileId) await expect(panel).toContainText('활성 프로파일 paper와 달리 실행 프로파일이 지정되지 않았습니다.')
  await expect(panel).toContainText('10일을 초과해 오래되면 신규 매수를 보류')
  await expect(panel).toContainText('거래 달력을 확정하는 검사는 아닙니다.')
})
}

test('512 status rows show summary and only three mounted rows per page', async ({ page }) => {
  const rows = Array.from({ length: 511 }, (_, index) => historyRow({ symbol: `T${String(index).padStart(3, '0')}` }))
  rows.push(historyRow({ symbol: 'LIMITED', state: 'capacityExceeded', lastError: '511대상 상한을 초과했습니다.' }))
  await mockApi(page, { tradingStatusOverride: () => running(rows) })
  await page.goto('/strategy')
  const panel = page.getByTestId('live-daily-history-panel')
  await expect(panel).toContainText('상태 항목 512개')
  await expect(panel).toContainText('대상 한도 초과 1개')
  await expect(panel).toContainText('511대상 상한을 초과했습니다.')
  await expect(panel.getByTestId('live-daily-history-row')).toHaveCount(3)
  await expect(panel).toContainText('1~3 / 전체 512개 상태 항목')
  await panel.getByRole('button', { name: '다음 일봉 상태' }).click()
  await expect(panel).toContainText('4~6 / 전체 512개 상태 항목')
  await expect(panel.getByTestId('live-daily-history-row')).toHaveCount(3)
  await expect(panel).not.toContainText('511대상 상한을 초과했습니다.')
})

test('mobile daily-history text and paging controls stay within the panel', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await mockApi(page, { tradingStatusOverride: () => running(Array.from({ length: 4 }, (_, index) => historyRow({
    symbol: `SOXQ-${index}`, state: 'failed', source: `read-only-source-${'long'.repeat(20)}`,
    lastError: `공급자 조회 실패: ${'재시도 후 자료를 확인합니다.'.repeat(8)}`, nextRetryAt: '2026-10-19T00:15:00Z',
  }))) })
  await page.goto('/strategy')
  const panel = page.getByTestId('live-daily-history-panel')
  await expect(panel.getByTestId('live-daily-history-row')).toHaveCount(3)
  await panel.scrollIntoViewIfNeeded()
  expect(await panel.evaluate((element) => element.scrollWidth <= element.clientWidth)).toBe(true)
  for (const row of await panel.getByTestId('live-daily-history-row').all()) {
    expect(await row.evaluate((element) => element.scrollWidth <= element.clientWidth)).toBe(true)
  }
  await panel.getByRole('button', { name: '다음 일봉 상태' }).click()
  await expect(panel).toContainText('4~4 / 전체 4개 상태 항목')
})
