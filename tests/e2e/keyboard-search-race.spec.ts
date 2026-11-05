import { expect, test, type Page } from '@playwright/test'
import { keyboardSearchRaceFixture, openSearchSurface, switchSearchMarket, waitSearchReply, settleSearchReply,
  expectSearchSelection, expectNoSearchSelection, expectNoSearchError, invalidSearchReply, failedSearchReply,
  captureSearchRace, safeSearchRace } from './fixtures/keyboardSearchRaceFixture'
import type { KeyboardRaceFixture, KeyboardSearchView } from './fixtures/keyboardSearchRaceFixture'

const states = new WeakMap<Page, KeyboardRaceFixture>()
async function setup(page: Page, broker: 'kis' | 'toss') { const state = await keyboardSearchRaceFixture(page, broker); states.set(page, state); return state }
test.afterEach(async ({ page }, info) => { const state = states.get(page); if (state) try { await captureSearchRace(page, state, info.title, info.status) } finally { safeSearchRace(state) } })

const asynchronous = [
  { broker: 'kis', surface: 'Trading' }, { broker: 'kis', surface: 'Strategy' }, { broker: 'kis', surface: 'LTH' },
  { broker: 'toss', surface: 'Strategy' }, { broker: 'toss', surface: 'LTH' },
] as const
async function begin(state: KeyboardRaceFixture, view: KeyboardSearchView, symbol: string) {
  const broker = state.scope.broker
  const ordinal = state.reads(symbol, broker).filter(ticket => ticket.held).length + 1
  await view.input.fill(symbol); await view.input.press('Enter')
  return waitSearchReply(state, symbol, ordinal, broker)
}
function fallbackCount(state: KeyboardRaceFixture, symbol: string) {
  return state.tickets.filter(ticket => ticket.path === `/api/overseas-price/NYS/${symbol}` || ticket.path === `/api/overseas-price/AMS/${symbol}`).length
}

for (const { broker, surface } of asynchronous) {
  test(`P2-KBD-02 ${broker.toUpperCase()} ${surface} duplicate button Enter3 repeat has one pending logical search and stable focus`, async ({ page }) => {
    const state = await setup(page, broker)
    const view = await openSearchSurface(page, surface)
    state.hold('Z')
    await view.input.fill('Z'); await view.input.focus(); await view.input.press('Tab')
    await expect(view.button).toBeFocused()
    const initial = await view.button.elementHandle()
    if (!initial) throw new Error('No initial search button')
    await page.keyboard.press('Space')
    const ticket = await waitSearchReply(state, 'Z')
    const pending = { count: await view.button.count(), sameDom: await view.button.count() === 1 ? await view.button.evaluate((element, old) => element === old, initial) : false,
      focused: await initial.evaluate(element => element === document.activeElement), busy: await view.button.count() === 1 ? await view.button.getAttribute('aria-busy') : null,
      ariaDisabled: await view.button.count() === 1 ? await view.button.getAttribute('aria-disabled') : null }
    expect(pending).toEqual({ count: 1, sameDom: true, focused: true, busy: 'true', ariaDisabled: 'true' })
    // Exercise the still-focused pending button, not only the protected input handler.
    await page.keyboard.press('Enter'); await page.keyboard.press('Space')
    const bounds = await view.button.boundingBox()
    if (!bounds) throw new Error('Pending search button has no pointer target')
    // Real pointer activation: locator.click intentionally waits for aria-disabled to clear.
    await page.mouse.click(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2)
    await expect(view.button).toBeFocused()
    await view.input.focus()
    for (let repeat = 0; repeat < 3; repeat++) await view.input.press('Enter')
    await view.input.dispatchEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, repeat: true, bubbles: true })
    await view.input.evaluate(element => {
      for (let burst = 0; burst < 3; burst++) element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true }))
    })
    await page.clock.runFor(600)
    state.notes.push({ stage: 'held-before-first-response', pending, attempts: 'pending button Enter/Space/click + input Enter3/repeat + untrusted same-turn burst', requestIds: state.reads('Z').map(read => read.id) })
    expect(state.reads('Z').filter(read => read.held).length).toBe(1)
    state.auto('Z'); await settleSearchReply(page, ticket)
    await expectSearchSelection(page, surface, 'Z')
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
  })

  test(`P2-KBD-02 ${broker.toUpperCase()} ${surface} stale A success cannot select A or finish the pending B search`, async ({ page }) => {
    const state = await setup(page, broker), view = await openSearchSurface(page, surface)
    state.hold('A'); state.hold('B')
    const old = await begin(state, view, 'A'), current = await begin(state, view, 'B')
    state.auto('A'); await settleSearchReply(page, old)
    state.notes.push({ stage: 'A-body-finished-while-B-pending', oldId: old.id, currentId: current.id, oldBodyFinished: old.browserFinished, currentStillPending: !current.settled })
    await expect(view.input).toHaveValue('B')
    await expectNoSearchSelection(page, surface, 'A'); await expectNoSearchError(page, surface, 'A')
    expect(current.settled).toBe(false)
    await expect(view.button).toHaveAttribute('aria-busy', 'true')
    state.auto('B'); await settleSearchReply(page, current)
    await expectSearchSelection(page, surface, 'B')
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
  })

  test(`P2-KBD-02 ${broker.toUpperCase()} ${surface} input-only invalidation and raw ABA keep old epochs dead`, async ({ page }) => {
    const state = await setup(page, broker), view = await openSearchSurface(page, surface)
    state.hold('A')
    const edited = await begin(state, view, 'A')
    await view.input.fill('B') // No second submission: input editing itself is the boundary.
    await page.clock.runFor(1)
    const editedFallbacks = fallbackCount(state, 'A')
    state.auto('A'); await settleSearchReply(page, edited, failedSearchReply())
    await expect(view.input).toHaveValue('B')
    await expectNoSearchSelection(page, surface, 'A'); await expectNoSearchError(page, surface, 'A')
    expect(state.reads('B')).toHaveLength(0)
    expect(fallbackCount(state, 'A')).toBe(editedFallbacks)
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
    state.notes.push({ stage: 'input-only-B-without-request', oldId: edited.id, bodyFinished: edited.browserFinished })

    state.hold('A')
    const cleared = await begin(state, view, 'A')
    await view.input.fill('')
    await expect(view.button).toBeDisabled()
    state.auto('A'); await settleSearchReply(page, cleared)
    await expect(view.input).toHaveValue('')
    await expectNoSearchSelection(page, surface, 'A'); await expectNoSearchError(page, surface, 'A')
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
    state.notes.push({ stage: 'input-cleared-without-request', oldId: cleared.id, bodyFinished: cleared.browserFinished })

    state.hold('A'); state.hold('B')
    const firstA = await begin(state, view, 'A'), middleB = await begin(state, view, 'B'), newA = await begin(state, view, 'A')
    expect(new Set([firstA.id, middleB.id, newA.id]).size).toBe(3)
    await settleSearchReply(page, firstA)
    await expectNoSearchSelection(page, surface, 'A')
    await expect(view.input).toHaveValue('A'); await expect(view.button).toHaveAttribute('aria-busy', 'true')
    const before = fallbackCount(state, 'B')
    await settleSearchReply(page, middleB, failedSearchReply())
    await expectNoSearchSelection(page, surface, 'B'); await expectNoSearchError(page, surface, 'B')
    expect(fallbackCount(state, 'B')).toBe(before)
    expect(newA.settled).toBe(false)
    await expect(view.button).toHaveAttribute('aria-busy', 'true')
    state.auto('A'); state.auto('B'); await settleSearchReply(page, newA)
    await expectSearchSelection(page, surface, 'A')
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
    state.notes.push({ stage: 'raw-ABA-distinct-response-tickets', firstA: firstA.id, middleB: middleB.id, newA: newA.id })
  })

  test(`P2-KBD-02 ${broker.toUpperCase()} ${surface} stale error and invalid replies cannot fall back or leak selection or error`, async ({ page }) => {
    const state = await setup(page, broker), view = await openSearchSurface(page, surface)
    for (const kind of ['error', 'invalid'] as const) {
      state.hold('A'); state.hold('B')
      const old = await begin(state, view, 'A'), current = await begin(state, view, 'B')
      const before = fallbackCount(state, 'A')
      await settleSearchReply(page, old, kind === 'error' ? failedSearchReply() : invalidSearchReply(broker, 'A'))
      await expect(view.input).toHaveValue('B')
      await expectNoSearchSelection(page, surface, 'A'); await expectNoSearchError(page, surface, 'A')
      expect(fallbackCount(state, 'A')).toBe(before)
      expect(current.settled).toBe(false)
      await expect(view.button).toHaveAttribute('aria-busy', 'true')
      state.notes.push({ stage: `stale-${kind}-body-finished`, oldId: old.id, currentId: current.id, bodyFinished: old.browserFinished })
      state.auto('A'); state.auto('B'); await settleSearchReply(page, current)
      await expectSearchSelection(page, surface, 'B')
      await expect(view.button).toHaveAttribute('aria-busy', 'false')
    }
  })

  test(`P2-KBD-02 ${broker.toUpperCase()} ${surface} current failure preserves its domain fallback and permits one explicit retry`, async ({ page }) => {
    const state = await setup(page, broker)
    const paths = broker === 'kis' ? ['NAS', 'NYS', 'AMS'].map(exchange => state.lookupPath('Z', broker, exchange)) : [state.lookupPath('Z', broker)]
    const originals = paths.map(path => [path, state.defaults.get(path)!] as const)
    for (const path of paths) state.defaults.set(path, failedSearchReply())
    const view = await openSearchSurface(page, surface)
    await view.input.fill('Z'); await view.input.press('Enter')
    if (surface === 'LTH') {
      await expect(view.card.getByText('Z (Z)', { exact: true })).toBeVisible()
      await expect(view.card).toContainText('검증은 실패했지만')
    } else await expect(view.card).toContainText('"Z"')
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
    if (broker === 'kis') expect(state.tickets.filter(ticket => paths.includes(ticket.path)).map(ticket => ticket.path)).toEqual(paths)
    for (const [path, reply] of originals) state.defaults.set(path, reply)
    const before = state.reads('Z').length
    state.hold('Z'); await view.button.focus(); await page.keyboard.press('Enter')
    const retry = await waitSearchReply(state, 'Z')
    expect(state.reads('Z').length).toBe(before + 1)
    await expect(view.button).toBeFocused()
    state.auto('Z'); await settleSearchReply(page, retry)
    await expectSearchSelection(page, surface, 'Z'); await expectNoSearchError(page, surface, 'Z')
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
    state.notes.push({ stage: 'current-failure-and-explicit-retry', failurePaths: paths, retryId: retry.id, lthDirectFallbackPreserved: surface === 'LTH' })
  })

  test(`P2-KBD-02 ${broker.toUpperCase()} ${surface} market reset and SPA unmount fence old work without blocking a new caller`, async ({ page }) => {
    const state = await setup(page, broker)
    let view = await openSearchSurface(page, surface)
    state.hold('A')
    const oldMarket = await begin(state, view, 'A')
    await switchSearchMarket(page, surface, 'KR')
    const krInput = surface === 'LTH' ? view.card.getByRole('textbox', { name: 'ETF 코드 또는 이름', exact: true }) : page.getByRole('textbox', { name: '6자리 종목코드 (예: 005930, 0005A0)', exact: true })
    await expect(krInput).toHaveValue('')
    state.auto('A'); await settleSearchReply(page, oldMarket)
    await expect(krInput).toHaveValue('')
    await expectNoSearchSelection(page, surface, 'A'); await expectNoSearchError(page, surface, 'A')
    await switchSearchMarket(page, surface, 'US')
    state.hold('B'); const b = await begin(state, view, 'B')
    state.auto('B'); await settleSearchReply(page, b); await expectSearchSelection(page, surface, 'B')

    state.hold('A'); const oldUnmount = await begin(state, view, 'A')
    const runtime = await page.evaluate(() => { const value = Math.random().toString(36); Reflect.set(window, 'kbd02RuntimeIdentity', value); return value })
    await page.getByRole('button', { name: 'History', exact: true }).click()
    await expect(page).toHaveURL(/\/history$/)
    view = await openSearchSurface(page, surface, true)
    expect(await page.evaluate(() => Reflect.get(window, 'kbd02RuntimeIdentity'))).toBe(runtime)
    state.hold('Z'); const newCaller = await begin(state, view, 'Z')
    const before = fallbackCount(state, 'A')
    await settleSearchReply(page, oldUnmount, failedSearchReply())
    expect(fallbackCount(state, 'A')).toBe(before)
    await expectNoSearchSelection(page, surface, 'A'); await expectNoSearchError(page, surface, 'A')
    await expect(view.input).toHaveValue('Z'); await expect(view.button).toHaveAttribute('aria-busy', 'true')
    expect(newCaller.settled).toBe(false)
    state.auto('A'); state.auto('Z'); await settleSearchReply(page, newCaller)
    await expectSearchSelection(page, surface, 'Z')
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
    state.notes.push({ stage: 'market-and-real-SPA-unmount', marketOld: oldMarket.id, unmountedOld: oldUnmount.id, newCaller: newCaller.id, runtime })
  })
}

test('P2-KBD-02 TOSS Trading preserves immediate symbol selection while ordinary safety query remains slow', async ({ page }) => {
  const state = await setup(page, 'toss'), view = await openSearchSurface(page, 'Trading')
  state.hold('Z'); await view.input.fill('Z'); await view.input.press('Enter')
  const safety = await waitSearchReply(state, 'Z')
  await expectSearchSelection(page, 'Trading', 'Z'); await expect(view.button).toBeVisible()
  expect(await view.button.getAttribute('aria-busy')).not.toBe('true')
  expect(safety.settled).toBe(false)
  state.notes.push({ stage: 'immediate-selection-before-safety-response', safetyId: safety.id, selected: 'Z' })
  state.auto('Z'); await settleSearchReply(page, safety)
})

for (const surface of ['Trading', 'Strategy', 'LTH'] as const) test(`P2-KBD-02 KIS ${surface} fallback keeps NAS NYS AMS order and a valid one-letter ticker`, async ({ page }) => {
  const state = await setup(page, 'kis')
  state.defaults.set('/api/overseas-price/NAS/Z', failedSearchReply())
  state.defaults.set('/api/overseas-price/NYS/Z', invalidSearchReply('kis', 'Z'))
  const view = await openSearchSurface(page, surface)
  await view.input.fill('Z'); await view.input.press('Enter')
  await expectSearchSelection(page, surface, 'Z')
  const discovery = state.tickets.filter(read => /\/overseas-price\//.test(read.path)).slice(0, 3).map(read => read.path)
  expect(discovery).toEqual(['/api/overseas-price/NAS/Z', '/api/overseas-price/NYS/Z', '/api/overseas-price/AMS/Z'])
  state.notes.push({ stage: 'existing-exchange-discovery', surface, discovery })
})
