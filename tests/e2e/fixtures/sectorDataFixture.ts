import { expect, test, type Page } from '@playwright/test'
import { syntheticSectorPortfolioInput } from '../../../src/pages/strategy/model/sectorPortfolioInput'
import type { SectorDataValidation, ValidateSectorDataInput } from '../../../src/shared/api/sectorDataTypes'

export function requestFixture(): ValidateSectorDataInput {
  const input = syntheticSectorPortfolioInput()
  const all = [...input.history, ...input.evaluation]
  return {
    bundle: {
      schemaVersion: 1, retrievedAt: '2026-01-01T00:00:00Z', pages: [],
      manifest: { synthetic: true, sessionDates: all.map(day => day.date), endDate: all[all.length - 1].date,
        calendarSource: '합성 UI fixture 달력', calendarEvidence: '합성 fixture 내부 완전성, 실제 거래일 검증 아님',
        exchangeMappings: Object.keys(all[0].prices).map(symbol => ({ symbol, exchange: 'AMS', evidence: '합성 UI 매핑, 실제 조회 금지' })) },
      series: Object.keys(all[0].prices).map(symbol => ({ symbol, exchange: 'AMS',
        raw: all.map(day => ({ date: day.date, open: day.prices[symbol].open, close: day.prices[symbol].close })),
        adjusted: all.map(day => ({ date: day.date, close: day.prices[symbol].adjustedClose })) })),
    },
    verification: { coverageVerified: true, adjustmentVerified: true, corporateActionsVerified: true,
      adjustmentEvidence: '합성 UI fixture 조정 없음', corporateActionsEvidence: '합성 UI fixture 이벤트 없음', corporateActions: [] },
    initialCashUsd: 10_000, costBps: 10,
  }
}

export function validationFixture(ready: boolean): SectorDataValidation {
  return { ready, input: ready ? syntheticSectorPortfolioInput() : null,
    bundleHash: 'a'.repeat(64), verificationHash: 'b'.repeat(64), warnings: ['합성 UI 검증 응답, 실제 자료 검증 아님'],
    diagnostics: ready ? [] : [{ code: 'EVIDENCE_REQUIRED', message: '달력·조정·기업행위 근거 확인 전 평가 보류' }] }
}

export async function blockUnsafe(page: Page) {
  const violations: string[] = []
  const errors: string[] = []
  const apiCalls: string[] = []
  const devSockets: string[] = []
  page.on('pageerror', error => errors.push(error.message))
  await page.routeWebSocket('**/*', socket => {
    const url = new URL(socket.url())
    // Vite 8 client attempts vite-hmr even with hmr:false/ws:false; close it too.
    // A production preview has no Vite client and must have zero such attempts.
    if ((test.info().project.name.includes('source-harness') || (!process.env.CI && !test.info().project.name.includes('production'))) && ['127.0.0.1', 'localhost'].includes(url.hostname) && url.pathname === '/' && url.searchParams.has('token')) devSockets.push(`Vite dev ${url.host}`)
    else violations.push(`WebSocket ${url.host}${url.pathname}`)
    socket.close()
  })
  await page.route('**/*', async route => {
    const url = new URL(route.request().url())
    const method = route.request().method()
    const pure = method === 'POST' && ['/api/strategy/sector-data/validate', '/api/strategy/sector-portfolio/preview'].includes(url.pathname)
    if (!['localhost', '127.0.0.1'].includes(url.hostname) || (url.pathname.startsWith('/api/') && !pure) || (!pure && method !== 'GET')) {
      violations.push(`${method} ${url.pathname}`)
      return route.abort()
    }
    if (url.pathname.startsWith('/api/')) apiCalls.push(`${method} ${url.pathname}`)
    return route.fallback()
  })
  return { violations, errors, apiCalls, devSockets }
}

export async function openPreparation(page: Page) {
  const prep = page.getByTestId('sector-data-preparation')
  await prep.getByRole('button', { name: '자료 수집·검증 준비', exact: true }).click()
  await expect(prep.locator('.MuiCollapse-root').first()).toHaveClass(/MuiCollapse-entered/)
  return prep
}

