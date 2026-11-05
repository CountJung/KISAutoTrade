// Document-start fixture transport. No native bridge or UI automation is installed.
(() => {
  'use strict'
  const fixture = __SMOKE_FIXTURE_JSON__
  const limit = 256
  const record = { version: 1, syntheticOnly: true, browserTransport: !('__TAURI_INTERNALS__' in window), requests: [], denied: [], errors: [], warnings: [], policies: [], overflow: false }
  const push = (key, item) => {
    if (record[key].length < limit) record[key].push(item)
    else record.overflow = true
  }
  const message = value => String(value?.message ?? value).slice(0, 512)
  for (const [method, key] of [['error', 'errors'], ['warn', 'warnings']]) {
    const original = console[method].bind(console)
    console[method] = (...items) => { push(key, items.map(message).join(' ').slice(0, 1024)); original(...items) }
  }
  addEventListener('error', event => push('errors', message(event.message ?? 'resource load error')))
  addEventListener('unhandledrejection', event => push('errors', message(event.reason)))
  addEventListener('securitypolicyviolation', event => push('policies', { directive: event.effectiveDirective, blocked: event.blockedURI.slice(0, 256) }))
  const deny = (channel, target = '') => {
    push('denied', { channel, target: String(target).slice(0, 256) })
    throw new TypeError(`Synthetic smoke denied ${channel}`)
  }
  const json = value => new Response(JSON.stringify(value), { status: 200, headers: { 'content-type': 'application/json' } })
  const fetchFixture = async (input, options = {}) => {
    const url = new URL(input instanceof Request ? input.url : String(input), location.href)
    const method = String(options.method ?? (input instanceof Request ? input.method : 'GET')).toUpperCase()
    if (url.protocol !== 'kis-smoke:' || url.hostname !== 'localhost' || !url.pathname.startsWith('/api/')) return deny('fetch-origin', url.href)
    if (url.search.length > 2048) return deny('fetch-query-size', url.pathname)
    const allowedKeys = new Set(['count', 'from', 'to', 'interval', 'period', 'brokerId', 'brokerAccountId', 'symbol', 'isOverseas', 'cursor', 'endTime', 'startTime', 'feed'])
    for (const key of url.searchParams.keys()) if (!allowedKeys.has(key)) return deny('fetch-query-key', key)
    push('requests', { method, path: url.pathname, query: url.search.slice(0, 256) })
    if (method === 'GET' && Object.hasOwn(fixture.get, url.pathname)) return json(fixture.get[url.pathname])
    if (method === 'GET' && /^\/api\/(toss-chart|chart)\/(SOXL|SOXS|SOXX|000002)$/.test(url.pathname)) return json(fixture.candles)
    if (method === 'GET' && url.pathname === '/api/candles/page') return json({ candles: fixture.candles, nextCursor: null, hasMore: false, source: 'synthetic-only' })
    if (method === 'POST' && ['/api/strategy/preview', '/api/strategy/leveraged-trend-hold/preview'].includes(url.pathname)) {
      const bodyText = String(options.body ?? '')
      if (bodyText.length > 65536) return deny('preview-body-size')
      const body = JSON.parse(bodyText)
      return json({ symbol: body.symbol, strategyId: body.strategyId, interval: body.interval ?? '1m', candleCount: 3, candles: fixture.candles, signals: fixture.signals, generatedAt: '2026-07-07T17:03:00+09:00', message: 'Synthetic smoke preview — no orders or provider calls' })
    }
    return deny('fetch-method-or-path', `${method} ${url.pathname}`)
  }
  Object.defineProperty(window, 'fetch', { value: fetchFixture, configurable: false, writable: false })
  for (const name of ['XMLHttpRequest', 'WebSocket', 'EventSource', 'Worker', 'SharedWorker', 'RTCPeerConnection', 'webkitRTCPeerConnection']) {
    Object.defineProperty(window, name, { value: function () { return deny(name) }, configurable: false, writable: false })
  }
  Object.defineProperty(navigator, 'sendBeacon', { value: () => { push('denied', { channel: 'beacon' }); return false } })
  if (navigator.serviceWorker) {
    try { Object.defineProperty(navigator.serviceWorker, 'register', { value: () => Promise.reject(new TypeError('Synthetic smoke denies service workers')) }) }
    catch (error) { push('errors', `service-worker guard: ${message(error)}`) }
  }
  const canvasState = canvas => {
    const sample = document.createElement('canvas')
    sample.width = Math.min(canvas.width, 160)
    sample.height = Math.min(canvas.height, 90)
    const context = sample.getContext('2d', { willReadFrequently: true })
    const colors = new Set()
    if (context && sample.width && sample.height) {
      context.drawImage(canvas, 0, 0, sample.width, sample.height)
      const pixels = context.getImageData(0, 0, sample.width, sample.height).data
      for (let index = 0; index < pixels.length && colors.size < 8; index += 16) {
        if (pixels[index + 3]) colors.add(`${pixels[index]},${pixels[index + 1]},${pixels[index + 2]}`)
      }
    }
    return { width: canvas.width, height: canvas.height, cssWidth: canvas.getBoundingClientRect().width, cssHeight: canvas.getBoundingClientRect().height, sampledColors: colors.size }
  }
  const ephemeralMarker = { initiallyAbsent: false, readbackMatches: false }
  try {
    const key = 'kis-smoke-own-ephemeral-marker-v1'
    ephemeralMarker.initiallyAbsent = localStorage.getItem(key) === null
    localStorage.setItem(key, 'synthetic-only')
    ephemeralMarker.readbackMatches = localStorage.getItem(key) === 'synthetic-only'
  } catch (error) { push('errors', `ephemeral storage: ${message(error)}`) }
  // Only read by a fixed host evaluation callback; never invokes native commands.
  Object.defineProperty(window, '__SMOKE_READ_REPORT__', { value: () => ({
    ...record,
    ephemeralMarker,
    location: location.pathname,
    title: document.title,
    rootChildren: document.getElementById('root')?.childElementCount ?? 0,
    theme: document.documentElement.dataset.theme ?? null,
    bodyBg: document.body ? getComputedStyle(document.body).backgroundColor : null,
    viewport: { width: innerWidth, height: innerHeight, dpr: devicePixelRatio },
    dialogCount: document.querySelectorAll('[role="dialog"]').length,
    focusedRole: document.activeElement?.getAttribute('role') ?? document.activeElement?.tagName,
    canvas: [...document.querySelectorAll('canvas')].slice(0, 12).map(canvasState),
  }), configurable: false, writable: false })
})()
