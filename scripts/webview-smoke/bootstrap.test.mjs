// Pure source-level fixture tests. No WebView, operating app or network is run.
import test from 'node:test'
import assert from 'node:assert/strict'
import vm from 'node:vm'
import { readFile } from 'node:fs/promises'
const source = await readFile(new URL('./bootstrap.js', import.meta.url), 'utf8')
const fixture = JSON.parse(await readFile(new URL('./fixtures.json', import.meta.url), 'utf8'))
function realm() {
  let nativeFetchCalls = 0
  const storage = new Map()
  const sandbox = {
    URL, Request, Response, console, location: { href: 'kis-smoke://localhost/', pathname: '/' },
    navigator: {}, addEventListener() {}, innerWidth: 1280, innerHeight: 900, devicePixelRatio: 2,
    bodyBackground: 'rgb(245, 245, 245)',
    getComputedStyle: () => ({ backgroundColor: sandbox.bodyBackground }),
    document: { body: {}, title: 'Synthetic', documentElement: { dataset: { theme: 'light' } }, activeElement: { tagName: 'BODY', getAttribute: () => null }, getElementById: () => ({ childElementCount: 1 }), querySelectorAll: () => [] },
    localStorage: { getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, value) },
    fetch: () => { nativeFetchCalls += 1; throw new Error('native fetch must never run') },
  }
  sandbox.window = sandbox
  vm.runInNewContext(source.replace('__SMOKE_FIXTURE_JSON__', JSON.stringify(fixture)), sandbox)
  return { window: sandbox, nativeFetchCalls: () => nativeFetchCalls }
}
test('known read-only and pure preview use only the fixed synthetic responses', async () => {
  const { window, nativeFetchCalls } = realm()
  assert.equal((await (await window.fetch('/api/app-config')).json()).active_profile_id, 'synthetic')
  const preview = await window.fetch('/api/strategy/leveraged-trend-hold/preview', { method: 'POST', body: JSON.stringify({ symbol: 'SOXL', interval: '1m' }) })
  assert.equal((await preview.json()).candles.length, 3)
  assert.equal(nativeFetchCalls(), 0)
  assert.equal(window.__SMOKE_READ_REPORT__().browserTransport, true)
  assert.equal('__TAURI_INTERNALS__' in window, false)
  assert.equal('ipc' in window, false)
})
test('external URLs and financial mutations are denied without a native fallback', async () => {
  const { window, nativeFetchCalls } = realm()
  for (const [url, method] of [['https://example.invalid/api/app-config','GET'], ['/api/order','POST'], ['/api/trading/start','POST'], ['/api/profiles/add','POST'], ['/api/unknown','GET']]) {
    await assert.rejects(window.fetch(url, { method }), /Synthetic smoke denied/)
  }
  assert.throws(() => new window.WebSocket('wss://example.invalid'), /Synthetic smoke denied/)
  assert.equal(nativeFetchCalls(), 0)
  assert.equal(window.__SMOKE_READ_REPORT__().denied.length, 6)
})
test('telemetry is bounded and markers use only the provided ephemeral test storage', async () => {
  const { window } = realm()
  for (let index = 0; index < 300; index += 1) await window.fetch('/api/unknown').catch(() => {})
  const report = window.__SMOKE_READ_REPORT__()
  assert.equal(report.denied.length, 256)
  assert.equal(report.overflow, true)
  assert.equal(report.ephemeralMarker.initiallyAbsent, true)
  assert.equal(report.ephemeralMarker.readbackMatches, true)
  assert.equal(realm().window.__SMOKE_READ_REPORT__().ephemeralMarker.initiallyAbsent, true)
})

test('read-only screen observations track real body style and viewport without UI actions', () => {
  const { window } = realm()
  const before = window.__SMOKE_READ_REPORT__()
  assert.equal(before.bodyBg, 'rgb(245, 245, 245)')
  assert.equal(before.viewport.width, 1280)
  window.bodyBackground = 'rgb(30, 30, 30)'
  window.innerWidth = 375
  const after = window.__SMOKE_READ_REPORT__()
  assert.equal(after.bodyBg, 'rgb(30, 30, 30)')
  assert.equal(after.viewport.width, 375)
  assert.equal(after.viewport.dpr, 2)
  assert.equal(after.requests.length, 0)
  assert.equal(after.denied.length, 0)
})
