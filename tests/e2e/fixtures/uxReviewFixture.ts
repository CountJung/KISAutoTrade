import type { Page, Route } from '@playwright/test'
import { mockApi } from './strategyResearchFixture'

export type UxResponse = { status?: number; json?: unknown }
export type UxHandler = (route: Route) => Promise<void>

// Browser-only synthetic data. No native shell, broker or account service runs.
export async function mockUx(page: Page, options: Parameters<typeof mockApi>[1] = {}) {
  const writes: string[] = []
  const external: string[] = []
  const errors: string[] = []
  const unknown: string[] = []
  const counts = new Map<string, number>()
  const responses = new Map<string, UxResponse | UxHandler>()
  page.on('pageerror', error => errors.push(error.message))
  await page.route('**/*', route => {
    const url = new URL(route.request().url())
    if (['127.0.0.1', 'localhost'].includes(url.hostname)) return route.fallback()
    external.push(url.origin)
    return route.abort()
  })
  await page.routeWebSocket('**/*', socket => socket.close())
  await mockApi(page, options)
  await page.route('**/api/**', async route => {
    const request = route.request()
    const url = new URL(request.url())
    if (!url.pathname.startsWith('/api/')) return route.fallback()
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) {
      external.push(url.origin)
      return route.abort()
    }
    const key = url.pathname
    counts.set(key, (counts.get(key) ?? 0) + 1)
    if (request.method() !== 'GET') {
      writes.push(key)
      return route.fulfill({ status: 403, json: { code: 'READ_ONLY_FIXTURE', message: 'Synthetic fixture denies writes' } })
    }
    const response = responses.get(key)
    if (typeof response === 'function') return response(route)
    if (response) return route.fulfill({ status: response.status ?? 200, json: response.json ?? [] })
    if (key === '/api/profiles' || key === '/api/stats') return route.fulfill({ json: [] })
    const known = new Set(['/api/check-update','/api/app-config','/api/check-config','/api/trading/status','/api/recent-logs','/api/broker-rate-limit-status','/api/balance','/api/overseas-balance','/api/broker-holdings','/api/today-trades','/api/positions','/api/pending-orders','/api/trades','/api/executed','/api/kis-executed','/api/today-stats','/api/exchange-rate','/api/exchange-rate/status','/api/refresh-interval','/api/log-config','/api/archive-config','/api/archive-stats','/api/web-config','/api/stock-list-stats','/api/risk-config','/api/strategies','/api/toss-market-calendar'])
    if (known.has(key)) return route.fallback()
    unknown.push(key)
    return route.fulfill({ status: 503, json: { code: 'UNKNOWN_FIXTURE', message: 'Unknown synthetic route denied' } })
  })
  return { unknown, writes, external, errors, counts, responses }
}
