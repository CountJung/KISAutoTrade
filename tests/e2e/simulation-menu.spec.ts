import { expect, test, type Page } from '@playwright/test'
import { mockUx } from './fixtures/uxReviewFixture'
import { fixtureReport } from './fixtures/sectorPortfolioFixture'
import { requestFixture, validationFixture } from './fixtures/sectorDataFixture'
import type { SectorPortfolioInput } from '../../src/shared/api/sectorPortfolioTypes'

async function setup(page: Page, options: { running?: boolean; configError?: boolean } = {}) {
  const audit = await mockUx(page, { tradingStatusOverride: () => ({ isRunning: options.running ?? false }),
    appConfigOverride: { active_profile_name: '합성 시세 프로필', kis_account_no: 'FAKE-ACCOUNT-NOT-CAPITAL' } })
  const calculations: SectorPortfolioInput[] = []
  if (options.configError) audit.responses.set('/api/app-config', { status: 503, json: { error: 'Synthetic config unavailable' } })
  // These two explicit pure calls are the only POST requests allowed by this fixture.
  await page.route('**/api/strategy/sector-portfolio/preview', route => {
    const input = route.request().postDataJSON() as SectorPortfolioInput
    calculations.push(input)
    return route.fulfill({ json: fixtureReport(input) })
  })
  await page.route('**/api/strategy/sector-data/validate', route => route.fulfill({ json: validationFixture(true) }))
  function safe() {
    expect(audit.writes).toEqual([])
    expect(audit.external).toEqual([])
    expect(audit.errors).toEqual([])
    expect(audit.unknown).toEqual([])
    for (const path of ['/api/balance', '/api/overseas-balance', '/api/broker-holdings']) expect(audit.counts.get(path) ?? 0).toBe(0)
  }
  return { audit, calculations, safe }
}

test('desktop Simulation menu and Strategy shortcut open an independent USD flow without starting or stopping operations', async ({ page }) => {
  const state = await setup(page)
  await page.goto('/strategy')
  await page.getByRole('button', { name: 'Simulation으로 이동', exact: true }).click()
  await expect(page).toHaveURL(/\/simulation$/)
  await expect(page.getByRole('heading', { name: '시뮬레이션', exact: true })).toBeVisible()
  await expect(page.getByRole('tab', { name: '자료 준비', exact: true })).toHaveAttribute('aria-selected', 'true')
  await expect(page.getByTestId('simulation-operating-status')).toContainText('운영 자동매매 정지 상태')
  await expect(page.getByTestId('simulation-operating-status')).toContainText('주문 권한이 차단되지는 않습니다')
  await page.getByRole('navigation').getByRole('button', { name: 'Strategy', exact: true }).click()
  await page.getByRole('navigation').getByRole('button', { name: 'Simulation', exact: true }).click()
  await expect(page).toHaveURL(/\/simulation$/)
  expect(state.calculations).toEqual([])
  await page.screenshot({ path: '.cache/simulation-menu/desktop-workspace.png', fullPage: true, animations: 'disabled' })
  state.safe()
})

test('preparation applies data to comparison and tab changes preserve USD and manually supplied paper drafts', async ({ page }) => {
  const state = await setup(page, { running: true })
  await page.goto('/simulation')
  await expect(page.getByTestId('simulation-operating-status')).toContainText('운영 자동매매 실행 중')
  const prep = page.getByRole('tabpanel', { name: '자료 준비', exact: true })
  await prep.getByLabel('자료 준비 JSON', { exact: true }).fill(JSON.stringify(requestFixture()))
  await prep.getByRole('button', { name: '파일 자료 검증', exact: true }).click()
  await prep.getByRole('button', { name: '검증된 자료를 비교 입력으로 적용', exact: true }).click()
  const compareTab = page.getByRole('tab', { name: '세 전략 비교', exact: true })
  await expect(compareTab).toHaveAttribute('aria-selected', 'true')
  await expect(compareTab).toBeFocused()
  const compare = page.getByRole('tabpanel', { name: '세 전략 비교', exact: true })
  const before = await compare.getByLabel('섹터 비교 자료 JSON', { exact: true }).inputValue()
  await compare.getByRole('button', { name: '세 전략 비교 계산', exact: true }).click()
  await expect(compare.getByTestId('sector-portfolio-results')).toBeVisible()
  expect(state.calculations).toHaveLength(1)
  expect(state.calculations[0].initialCashUsd).toBe(10_000)
  expect(JSON.stringify(state.calculations[0])).not.toContain('FAKE-ACCOUNT-NOT-CAPITAL')
  await page.getByRole('tab', { name: '모의 결과 대조', exact: true }).click()
  const paper = page.getByLabel('모의 원장 평가 JSON', { exact: true })
  await paper.fill('{"draft":"only manually entered"}')
  await page.getByRole('tab', { name: '자료 준비', exact: true }).click()
  await expect(page.getByLabel('자료 준비 JSON', { exact: true })).toHaveValue(JSON.stringify(requestFixture()))
  await compareTab.click()
  await expect(compare.getByLabel('섹터 비교 자료 JSON', { exact: true })).toHaveValue(before)
  await expect(compare.getByTestId('sector-portfolio-results')).toBeVisible()
  await page.getByRole('tab', { name: '모의 결과 대조', exact: true }).click()
  await expect(paper).toHaveValue('{"draft":"only manually entered"}')
  await expect(page.getByTestId('simulation-operating-status')).toContainText('운영 자동매매 실행 중')
  state.safe()
})

test('375px shortcut, drawer navigation and keyboard tabs remain accessible without trading mutations', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 })
  const state = await setup(page)
  await page.goto('/strategy')
  const shortcut = page.getByRole('button', { name: '시뮬레이션 메뉴로 이동', exact: true })
  await expect(shortcut).toBeVisible()
  await shortcut.click()
  await expect(shortcut).toHaveAttribute('aria-current', 'page')
  await page.getByRole('button', { name: '메뉴 열기', exact: true }).click()
  await page.getByRole('button', { name: 'Simulation', exact: true }).click()
  const first = page.getByRole('tab', { name: '자료 준비', exact: true })
  await first.focus()
  await page.keyboard.press('End')
  await expect(page.getByRole('tab', { name: '모의 결과 대조', exact: true })).toBeFocused()
  await expect(page.getByRole('tabpanel', { name: '모의 결과 대조', exact: true })).toContainText('먼저 세 전략 비교를 계산하세요')
  await page.keyboard.press('Home')
  await expect(first).toBeFocused()
  await page.keyboard.press('ArrowRight')
  await expect(page.getByRole('tab', { name: '세 전략 비교', exact: true })).toBeFocused()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
  await page.screenshot({ path: '.cache/simulation-menu/mobile-workspace.png', fullPage: true, animations: 'disabled' })
  state.safe()
})

test('operation refresh failure displays the prior state and uncertainty without assuming stopped', async ({ page }) => {
  const state = await setup(page, { running: true })
  await page.goto('/simulation')
  const banner = page.getByTestId('simulation-operating-status')
  await expect(banner).toContainText('운영 자동매매 실행 중')
  expect(state.audit.counts.get('/api/trading/status')).toBe(1)
  state.audit.responses.set('/api/trading/status', { status: 503, json: { error: 'Synthetic operation status unavailable' } })
  await banner.getByRole('button', { name: '운영 자동매매 상태 다시 확인', exact: true }).click()
  await expect(banner).toContainText('이전 상태: 운영 자동매매 실행 중', { timeout: 12000 })
  await expect(banner).toContainText('현재 운영 상태가 확인되지 않았습니다')
  await expect(banner).not.toContainText('운영 자동매매 정지 상태')
  state.safe()
})

test('account configuration failure leaves file calculation available and blocks authenticated collection', async ({ page }) => {
  const state = await setup(page, { configError: true })
  await page.goto('/simulation')
  await expect(page.getByText('확인 실패 · 인증 수집 보류', { exact: false })).toBeVisible({ timeout: 12000 })
  await expect(page.getByRole('button', { name: 'KIS 조회 범위 확인', exact: true })).toBeDisabled()
  await page.getByRole('tab', { name: '세 전략 비교', exact: true }).click()
  await page.getByRole('button', { name: '합성 예제 입력', exact: true }).click()
  await page.getByRole('button', { name: '세 전략 비교 계산', exact: true }).click()
  await expect(page.getByTestId('sector-portfolio-results')).toBeVisible()
  expect(state.calculations[0].initialCashUsd).toBe(10_000)
  state.safe()
})
