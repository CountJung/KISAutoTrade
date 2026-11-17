import { expect, test } from '@playwright/test'
import { syntheticSectorPortfolioInput } from '../../src/pages/strategy/model/sectorPortfolioInput'
import type { ValidateSectorDataInput } from '../../src/shared/api/sectorDataTypes'
import { fixtureReport } from './fixtures/sectorPortfolioFixture'
import { blockUnsafe, openPreparation, requestFixture, validationFixture } from './fixtures/sectorDataFixture'

test('query research entry sends no automatic account, settings, quote or application WebSocket requests; malformed imports are safe', async ({ page }) => {
  const state = await blockUnsafe(page)
  await page.addInitScript(() => {
    const calls: string[] = []
    const original = Storage.prototype.getItem
    Storage.prototype.getItem = function (key: string) { if (this === window.localStorage) calls.push(key); return original.call(this, key) }
    Object.assign(window, { __researchStorageReads: calls })
  })
  await page.goto('/?researchOnly=1')
  await expect(page.getByRole('heading', { name: '섹터 ETF 독립 연구', exact: true })).toBeVisible()
  expect(state.apiCalls).toEqual([])
  expect(await page.evaluate(() => (window as unknown as { __researchStorageReads: string[] }).__researchStorageReads)).toEqual([])
  const prep = await openPreparation(page)
  await expect(prep.getByRole('button', { name: 'KIS 조회 범위 확인', exact: true })).toHaveCount(0)
  await prep.getByRole('button', { name: '템플릿 편집 시작', exact: true }).click()
  const template = JSON.parse(await prep.getByLabel('자료 준비 JSON', { exact: true }).inputValue())
  expect(template.verification.coverageVerified).toBe(false)
  expect(template.verification.adjustmentVerified).toBe(false)
  expect(template.verification.corporateActionsVerified).toBe(false)
  expect(template.bundle.manifest.exchangeMappings.every((item: { evidence: string; exchange: string }) => !item.evidence && !item.exchange)).toBe(true)
  for (const key of ['endDate', 'calendarSource', 'calendarEvidence']) {
    const malformed = structuredClone(template)
    malformed.bundle.manifest[key] = {}
    await prep.getByLabel('자료 준비 JSON', { exact: true }).fill(JSON.stringify(malformed))
    await expect(prep.getByRole('button', { name: '파일 자료 검증', exact: true })).toBeDisabled()
  }
  await prep.getByLabel('자료 준비 JSON', { exact: true }).fill(JSON.stringify({ ...template, bundle: { ...template.bundle, retrievedAt: {}, pages: {} } }))
  await expect(prep.getByRole('button', { name: '파일 자료 검증', exact: true })).toBeDisabled()
  expect(state.apiCalls).toEqual([])
  expect(state.errors).toEqual([])
  expect(state.violations).toEqual([])
  test.info().annotations.push({ type: 'blocked-vite-dev-websockets', description: String(state.devSockets.length) })
})

test('standalone bundle validation gates replay, exports full evidence, and invalidates results after edits on mobile', async ({ page }) => {
  const calls: ValidateSectorDataInput[] = []
  await page.route('**/api/strategy/sector-data/validate', async route => {
    const input = route.request().postDataJSON() as ValidateSectorDataInput
    calls.push(input)
    await route.fulfill({ json: validationFixture(input.verification.coverageVerified) })
  })
  await page.route('**/api/strategy/sector-portfolio/preview', route => route.fulfill({ json: fixtureReport(syntheticSectorPortfolioInput()) }))
  const state = await blockUnsafe(page)
  await page.setViewportSize({ width: 375, height: 812 })
  await page.goto('/?researchOnly=1')
  const prep = await openPreparation(page)
  const request = requestFixture()
  const blocked = { ...request, verification: { ...request.verification, coverageVerified: false } }
  await prep.getByLabel('자료 준비 JSON 파일').setInputFiles({ name: 'synthetic-bundle.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify(blocked)) })
  await prep.getByRole('button', { name: '파일 자료 검증', exact: true }).click()
  await expect(prep.getByTestId('sector-data-validation')).toContainText('평가 보류')
  await expect(prep.getByRole('button', { name: '검증된 자료를 비교 입력으로 적용', exact: true })).toBeDisabled()
  await expect(page.getByRole('button', { name: '세 전략 비교 계산', exact: true })).toBeDisabled()
  await prep.getByLabel('자료 준비 JSON', { exact: true }).fill(JSON.stringify(request))
  await expect(prep.getByTestId('sector-data-validation')).toHaveCount(0)
  await prep.getByRole('button', { name: '파일 자료 검증', exact: true }).click()
  await expect(prep).toContainText('자료 검증 통과')
  const downloadPromise = page.waitForEvent('download')
  await prep.getByRole('button', { name: '자료와 검증 보고서 다운로드', exact: true }).click()
  const download = await downloadPromise
  const stream = await download.createReadStream()
  const buffers: Buffer[] = []
  for await (const chunk of stream) buffers.push(Buffer.from(chunk))
  const evidence = JSON.parse(Buffer.concat(buffers).toString())
  expect(evidence.request).toEqual(request)
  expect(evidence.report.verificationHash).toBe('b'.repeat(64))
  await prep.getByRole('button', { name: '검증된 자료를 비교 입력으로 적용', exact: true }).click()
  await page.getByRole('button', { name: '세 전략 비교 계산', exact: true }).click()
  await expect(page.getByTestId('sector-portfolio-results')).toBeVisible()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true)
  await prep.scrollIntoViewIfNeeded()
  await page.screenshot({ path: '.cache/sector-frontend/research-mobile.png', fullPage: true, animations: 'disabled' })
  await prep.getByLabel('자료 준비 JSON', { exact: true }).fill(JSON.stringify({ ...evidence, report: { ...evidence.report, ready: true } }))
  await expect(prep.getByTestId('sector-data-validation')).toHaveCount(0)
  await expect(page.getByTestId('sector-portfolio-results')).toHaveCount(0)
  await expect(page.getByRole('button', { name: '세 전략 비교 계산', exact: true })).toBeDisabled()
  await prep.getByRole('button', { name: '파일 자료 검증', exact: true }).click()
  await expect(prep).toContainText('자료 검증 통과')
  expect(calls).toHaveLength(3)
  expect(calls[2]).toEqual(request)
  expect(state.apiCalls.filter(call => call.includes('/preview'))).toHaveLength(1)
  expect(state.errors).toEqual([])
  expect(state.violations).toEqual([])
})

test('native research marker exposes only pure operations and never calls the authenticated scope or collector', async ({ page }) => {
  const state = await blockUnsafe(page)
  await page.addInitScript(() => {
    const calls: string[] = []
    Object.assign(window, { __KIS_RESEARCH_ONLY__: true, __sectorCalls: calls, __TAURI_INTERNALS__: { invoke: async (command: string) => { calls.push(command); throw new Error('정상 UI 동작에 네이티브 요청이 없어야 합니다.') } } })
  })
  await page.goto('/')
  await expect(page.getByRole('heading', { name: '섹터 ETF 독립 연구', exact: true })).toBeVisible()
  const prep = await openPreparation(page)
  await expect(prep.getByRole('button', { name: 'KIS 조회 범위 확인', exact: true })).toHaveCount(0)
  expect(await page.evaluate(() => (window as unknown as { __sectorCalls: string[] }).__sectorCalls)).toEqual([])
  expect(state.apiCalls).toEqual([])
  expect(state.errors).toEqual([])
  expect(state.violations).toEqual([])
})

test('editing a pending validation discards its late report and prevents duplicate validation requests', async ({ page }) => {
  let release: () => void = () => {}
  const delayed = new Promise<void>(resolve => { release = resolve })
  let count = 0
  await page.route('**/api/strategy/sector-data/validate', async route => {
    count += 1
    await delayed
    await route.fulfill({ json: validationFixture(true) })
  })
  const state = await blockUnsafe(page)
  await page.goto('/?researchOnly=1')
  const prep = await openPreparation(page)
  await prep.getByLabel('자료 준비 JSON', { exact: true }).fill(JSON.stringify(requestFixture()))
  await prep.getByRole('button', { name: '파일 자료 검증', exact: true }).evaluate((button: HTMLButtonElement) => { button.click(); button.click() })
  await expect.poll(() => count).toBe(1)
  await prep.getByLabel('자료 준비 JSON', { exact: true }).fill('{}')
  const response = page.waitForResponse('**/api/strategy/sector-data/validate')
  release()
  await response
  await expect(prep.getByRole('button', { name: '파일 자료 검증', exact: true })).toBeDisabled()
  await expect(prep.getByTestId('sector-data-validation')).toHaveCount(0)
  expect(count).toBe(1)
  expect(state.errors).toEqual([])
  expect(state.violations).toEqual([])
})

