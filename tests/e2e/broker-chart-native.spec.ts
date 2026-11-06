import { readFile } from 'node:fs/promises'
import { expect, test } from '@playwright/test'
import { syntheticBrokerChartInput } from '../../src/widgets/sector-portfolio/model/brokerChartInput'
import { syntheticSectorPortfolioInput } from '../../src/widgets/sector-portfolio/model/sectorPortfolioInput'
import { brokerChartFixtureReport } from './fixtures/brokerChartFixture'
import { blockUnsafe } from './fixtures/sectorDataFixture'

test('synthetic native source needs separate explicit consent, discards ABA late data and preserves independent price results', async ({ page }) => {
  const safe = await blockUnsafe(page)
  const input = { ...syntheticBrokerChartInput(), acknowledgedPriceOnly: true }
  const report = brokerChartFixtureReport(input)
  await page.addInitScript(({ input, report }) => {
    const calls: { command: string; args: unknown }[] = []
    const collection = { input: { ...input, acknowledgedPriceOnly: false }, diagnostics: ['합성 수집 fixture · 실제 API 호출 없음'],
      receipts: [{ symbol: 'SPY', exchange: null, page: 1, rowCount: 42, oldestDate: input.history[0].date,
        newestDate: input.evaluation[19].date, rowsHash: 'e'.repeat(64), requestAdjustment: 'unadjusted' }], collectionHash: 'd'.repeat(64),
      exchangeMappings: [{ symbol: 'SPY', exchange: 'AMS', evidence: '합성 매핑 fixture · 실제 마스터 확인 아님' }] }
    Object.assign(window, { __brokerCalls: calls, __holdCollect: false, __releaseCollect: () => {}, __TAURI_INTERNALS__: {
      invoke: async (command: string, args: unknown) => {
        calls.push({ command, args })
        if (command === 'get_broker_chart_scope') return { profileId: 'synthetic-toss', provider: 'toss', paper: false }
        if (command === 'preview_broker_chart_portfolio') return report
        if (command === 'collect_broker_chart_data') {
          if ((window as unknown as { __holdCollect: boolean }).__holdCollect) await new Promise<void>(resolve => Object.assign(window, { __releaseCollect: resolve }))
          return collection
        }
        throw new Error(`차단한 네이티브 요청: ${command}`)
      },
    } })
  }, { input, report })
  await page.goto('/tests/e2e/fixtures/brokerChartHarness.html')
  await page.getByRole('tab', { name: '세 전략 비교', exact: true }).click()
  await page.getByRole('combobox', { name: '비교 방식', exact: true }).click()
  await page.getByRole('option', { name: '차트 가격수익률 근사', exact: true }).click()
  const chart = page.getByTestId('broker-chart-panel')
  const calls = () => page.evaluate(() => (window as unknown as { __brokerCalls: { command: string; args: unknown }[] }).__brokerCalls)
  expect(await calls()).toEqual([])
  await chart.getByLabel('차트 가격 자료 JSON', { exact: true }).fill(JSON.stringify(input))
  await chart.getByRole('button', { name: '가격 근사 비교 계산', exact: true }).click()
  await expect(chart.getByTestId('broker-chart-results')).toBeVisible()
  const lookup = chart.getByRole('button', { name: '차트 조회 프로필 확인', exact: true })
  const windowSelect = chart.getByRole('combobox', { name: '차트 수집 평가 거래일', exact: true })
  await expect(windowSelect).toContainText('30거래일')
  await windowSelect.focus(); await page.keyboard.press('ArrowDown'); await page.keyboard.press('Home'); await page.keyboard.press('Enter')
  const collect = chart.getByRole('button', { name: '차트 자료 20봉 수집', exact: true })
  const ack = chart.getByRole('checkbox', { name: '확인한 프로필의 인증 시세 조회에 동의합니다 (주문·잔고 동의 아님)', exact: true })
  await expect(collect).toBeDisabled()
  await lookup.click(); await expect(chart).toContainText('synthetic-toss')
  await expect(collect).toBeDisabled()
  await ack.check()
  await page.evaluate(() => Object.assign(window, { __holdCollect: true }))
  await collect.click()
  await expect.poll(async () => (await calls()).filter(call => call.command === 'collect_broker_chart_data').length).toBe(1)
  await page.getByRole('button', { name: '합성 프로필 B', exact: true }).click()
  await page.getByRole('button', { name: '합성 프로필 A 재게시', exact: true }).click()
  await expect(lookup).toBeDisabled()
  await expect(chart.getByTestId('broker-chart-results')).toBeVisible()
  await page.evaluate(() => (window as unknown as { __releaseCollect: () => void }).__releaseCollect())
  await expect(lookup).toBeEnabled()
  await expect(ack).not.toBeChecked()
  await expect(collect).toBeDisabled()
  await expect(chart.getByLabel('차트 가격 자료 JSON', { exact: true })).toHaveValue(JSON.stringify(input))
  await expect(chart.getByTestId('broker-chart-results')).toBeVisible()
  // Recollection is a new source: reset price acknowledgement and retain declared known events.
  const withSplit = { ...input, knownCorporateActions: [{ symbol: 'SPY', date: input.evaluation[0].date, kind: 'split', splitRatio: 2 }] }
  await chart.getByLabel('차트 가격 자료 JSON', { exact: true }).fill(JSON.stringify(withSplit))
  await page.evaluate(() => Object.assign(window, { __holdCollect: false }))
  await lookup.click(); await ack.check(); await collect.click()
  await expect(chart).toContainText('합성 수집 fixture')
  const collected = JSON.parse(await chart.getByLabel('차트 가격 자료 JSON', { exact: true }).inputValue())
  expect(collected.acknowledgedPriceOnly).toBe(false)
  expect(collected.knownCorporateActions).toEqual(withSplit.knownCorporateActions)
  const downloadPromise = page.waitForEvent('download')
  await chart.getByRole('button', { name: '차트 수집 근거 JSON 다운로드', exact: true }).click()
  const downloaded = await downloadPromise
  const evidence = JSON.parse(await readFile((await downloaded.path())!, 'utf8'))
  expect(evidence.mode).toBe('brokerChartPriceApproximation')
  expect(evidence.collection.receipts).toEqual([expect.objectContaining({ symbol: 'SPY', requestAdjustment: 'unadjusted', rowsHash: 'e'.repeat(64) })])
  expect(evidence.collection.collectionHash).toBe('d'.repeat(64))
  expect(evidence.collection.exchangeMappings).toEqual([{ symbol: 'SPY', exchange: 'AMS', evidence: '합성 매핑 fixture · 실제 마스터 확인 아님' }])
  expect(evidence.collection.input.knownCorporateActions).toEqual([])
  expect(evidence.input.knownCorporateActions).toEqual(withSplit.knownCorporateActions)
  await expect(chart.getByRole('checkbox', { name: '배당 포함 총수익·권리 정산 미검증인 가격 근사 한계를 확인했습니다', exact: true })).not.toBeChecked()
  await expect(chart.getByTestId('broker-chart-results')).toHaveCount(0)
  const requests = (await calls()).filter(call => call.command === 'collect_broker_chart_data')
  expect(requests).toHaveLength(2)
  expect(requests[1].args).toEqual({ input: { expectedProfileId: 'synthetic-toss', authenticatedQuotesAcknowledged: true, evaluationBars: 20 } })
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.screenshot({ path: '.cache/sector-frontend/broker-chart-native-desktop.png', fullPage: true, animations: 'disabled' })
  expect(safe.apiCalls).toEqual([]); expect(safe.violations).toEqual([]); expect(safe.errors).toEqual([])
})

test('30-session collection preserves 22 warmup bars and option ABA requires fresh consent and discards late data', async ({ page }) => {
  const safe = await blockUnsafe(page)
  const input = syntheticBrokerChartInput()
  const strict = syntheticSectorPortfolioInput()
  input.evaluation.push(...strict.evaluation.slice(20).map(day => ({ date: day.date,
    prices: Object.fromEntries(Object.entries(day.prices).map(([symbol, price]) => [symbol, {
      open: price.open, high: price.close + 1, low: price.open - 1, close: price.close, volume: 100_000, currency: 'USD' as const,
    }])),
  })))
  input.expectedSessionDates = [...input.history, ...input.evaluation].map(day => day.date)
  expect(input.history).toHaveLength(22); expect(input.evaluation).toHaveLength(30)
  await page.addInitScript(input => {
    const calls: { command: string; args: unknown }[] = []
    Object.assign(window, { __windowCalls: calls, __holdWindow: true, __releaseWindow: () => {}, __TAURI_INTERNALS__: {
      invoke: async (command: string, args: unknown) => {
        calls.push({ command, args })
        if (command === 'get_broker_chart_scope') return { profileId: 'synthetic-kis', provider: 'kis', paper: true }
        if (command === 'collect_broker_chart_data') {
          if ((window as unknown as { __holdWindow: boolean }).__holdWindow) await new Promise<void>(resolve => Object.assign(window, { __releaseWindow: resolve }))
          return { input, diagnostics: ['합성30봉 수집 fixture · 실제 API 호출 없음'], receipts: [], collectionHash: 'f'.repeat(64) }
        }
        throw new Error(`차단한 네이티브 요청: ${command}`)
      },
    } })
  }, input)
  await page.setViewportSize({ width: 375, height: 900 })
  await page.goto('/tests/e2e/fixtures/brokerChartHarness.html')
  await page.getByRole('tab', { name: '세 전략 비교', exact: true }).click()
  await page.getByRole('combobox', { name: '비교 방식', exact: true }).click()
  await page.getByRole('option', { name: '차트 가격수익률 근사', exact: true }).click()
  const chart = page.getByTestId('broker-chart-panel')
  const windowSelect = chart.getByRole('combobox', { name: '차트 수집 평가 거래일', exact: true })
  const collect = chart.getByRole('button', { name: '차트 자료 30봉 수집', exact: true })
  const ack = chart.getByRole('checkbox', { name: '확인한 프로필의 인증 시세 조회에 동의합니다 (주문·잔고 동의 아님)', exact: true })
  await expect(windowSelect).toContainText('30거래일')
  await expect(chart).toContainText('사전22봉 + 평가30봉')
  await chart.getByRole('button', { name: '차트 조회 프로필 확인', exact: true }).click()
  await ack.check(); await collect.click()
  await expect(chart.getByRole('status')).toBeVisible()
  // A→B→A options must not make the original request current again.
  await windowSelect.focus(); await page.keyboard.press('ArrowDown'); await page.keyboard.press('Home'); await page.keyboard.press('Enter')
  await expect(ack).not.toBeChecked()
  await windowSelect.focus(); await page.keyboard.press('ArrowDown'); await page.keyboard.press('End'); await page.keyboard.press('Enter')
  await page.evaluate(() => (window as unknown as { __releaseWindow: () => void }).__releaseWindow())
  await expect(chart.getByRole('status')).toHaveCount(0)
  await expect(chart.getByLabel('차트 가격 자료 JSON', { exact: true })).toHaveValue('')
  await expect(collect).toBeDisabled()
  await page.evaluate(() => Object.assign(window, { __holdWindow: false }))
  await ack.check(); await collect.click()
  await expect(chart.getByLabel('차트 가격 자료 JSON', { exact: true })).toHaveValue(JSON.stringify(input, null, 2))
  await expect(chart).toContainText('사전 22봉 / 평가 30봉')
  await expect(chart.getByRole('checkbox', { name: '배당 포함 총수익·권리 정산 미검증인 가격 근사 한계를 확인했습니다', exact: true })).not.toBeChecked()
  const requests = await page.evaluate(() => (window as unknown as { __windowCalls: { command: string; args: unknown }[] }).__windowCalls)
  expect(requests.filter(call => call.command === 'collect_broker_chart_data')).toEqual(Array.from({ length: 2 }, () => ({
    command: 'collect_broker_chart_data', args: { input: { expectedProfileId: 'synthetic-kis', authenticatedQuotesAcknowledged: true, evaluationBars: 30 } },
  })))
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
  await page.screenshot({ path: '.cache/sector-frontend/broker-chart-30-mobile.png', fullPage: true, animations: 'disabled' })
  const downloadPromise = page.waitForEvent('download')
  await chart.getByRole('button', { name: '차트 수집 근거 JSON 다운로드', exact: true }).click()
  const download = await downloadPromise
  await download.saveAs('.cache/sector-frontend/broker-chart-30-collection.json')
  const evidence = JSON.parse(await readFile((await download.path())!, 'utf8'))
  expect(evidence.collection.input).toEqual(input)
  expect(evidence.input).toEqual(input)
  expect(safe.apiCalls).toEqual([]); expect(safe.violations).toEqual([]); expect(safe.errors).toEqual([])
})
