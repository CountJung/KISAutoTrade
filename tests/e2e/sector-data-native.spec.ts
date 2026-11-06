import { expect, test } from '@playwright/test'
import { syntheticSectorPortfolioInput } from '../../src/pages/strategy/model/sectorPortfolioInput'
import { blockUnsafe, openPreparation, requestFixture } from './fixtures/sectorDataFixture'

test('normal native source harness requires explicit scope and consent and resets stale verification on recollection', async ({ page }) => {
  const state = await blockUnsafe(page)
  const request = requestFixture()
  request.verification.corporateActions = [{ symbol: 'SPY', date: syntheticSectorPortfolioInput().evaluation[0].date, kind: 'dividend', dividendPerShare: 1 }]
  await page.addInitScript(({ bundle }) => {
    const calls: { command: string; args: Record<string, unknown> }[] = []
    Object.assign(window, { __sectorCalls: calls, __TAURI_INTERNALS__: { invoke: async (command: string, args: Record<string, unknown> = {}) => {
      calls.push({ command, args })
      if (command === 'get_sector_quote_scope') return { profileId: 'fixture-scope', provider: 'kis', paper: true }
      if (command === 'collect_sector_data') return bundle
      throw new Error(`금지한 네이티브 요청: ${command}`)
    } } })
  }, { bundle: { ...request.bundle, manifest: { ...request.bundle.manifest, synthetic: false } } })
  await page.goto('/tests/e2e/fixtures/sectorDataHarness.html')
  const panel = page.getByTestId('sector-portfolio-panel')
  await panel.getByRole('button', { name: '섹터 ETF 3전략 비교', exact: true }).click()
  const prep = await openPreparation(page)
  const calls = () => page.evaluate(() => (window as unknown as { __sectorCalls: { command: string; args: { input?: unknown } }[] }).__sectorCalls)
  expect(await calls()).toEqual([])
  await prep.getByLabel('자료 준비 JSON', { exact: true }).fill(JSON.stringify({ ...request, bundle: { ...request.bundle, manifest: { ...request.bundle.manifest, synthetic: false } } }))
  const collect = prep.getByRole('button', { name: 'KIS 12종목 자료 수집', exact: true })
  await expect(collect).toBeDisabled()
  await prep.getByRole('button', { name: 'KIS 조회 범위 확인', exact: true }).click()
  await expect(prep).toContainText('fixture-scope')
  await expect(collect).toBeDisabled()
  await prep.getByRole('checkbox', { name: '선택 프로필의 인증된 KIS 시세 조회 요청을 승인합니다. 주문은 하지 않습니다.', exact: true }).check()
  await collect.click()
  await expect.poll(async () => (await calls()).length).toBe(2)
  const collected = JSON.parse(await prep.getByLabel('자료 준비 JSON', { exact: true }).inputValue())
  expect(collected.verification).toMatchObject({ coverageVerified: false, adjustmentVerified: false, corporateActionsVerified: false, adjustmentEvidence: '', corporateActionsEvidence: '' })
  expect(collected.verification.corporateActions).toEqual(request.verification.corporateActions)
  expect((await calls())[1]).toEqual({ command: 'collect_sector_data', args: { input: { expectedProfileId: 'fixture-scope', authenticatedQuotesAcknowledged: true, manifest: { ...request.bundle.manifest, synthetic: false } } } })
  await expect(collect).toBeDisabled()
  await expect(prep.getByRole('checkbox')).not.toBeChecked()
  await page.screenshot({ path: '.cache/sector-frontend/collector-desktop.png', fullPage: true, animations: 'disabled' })
  expect(state.apiCalls).toEqual([])
  expect(state.errors).toEqual([])
  expect(state.violations).toEqual([])
})
