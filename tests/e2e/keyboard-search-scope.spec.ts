import { expect, test, type Page } from '@playwright/test'
import { keyboardSearchRaceFixture, installKeyboardScopeHarness, openSearchSurface, waitSearchReply, settleSearchReply,
  expectSearchSelection, expectNoSearchSelection, expectNoSearchError, invalidSearchReply, failedSearchReply,
  captureSearchRace, safeSearchRace } from './fixtures/keyboardSearchRaceFixture'
import type { KeyboardRaceFixture, SearchScope, SearchSurface } from './fixtures/keyboardSearchRaceFixture'

const states = new WeakMap<Page, KeyboardRaceFixture>()
test.afterEach(async ({ page }, info) => { const state = states.get(page); if (state) try { await captureSearchRace(page, state, info.title, info.status) } finally { safeSearchRace(state) } })

const changes: Array<{ name: string; surface: SearchSurface; next: Partial<SearchScope>; oldReply: 'success' | 'invalid' | 'error' }> = [
  { name: 'account-only', surface: 'Trading', next: { account: 'SYNTHETIC-ACCOUNT-B' }, oldReply: 'success' },
  { name: 'profile-only', surface: 'Strategy', next: { profile: 'synthetic-profile-b' }, oldReply: 'invalid' },
  { name: 'paper-only', surface: 'LTH', next: { paper: false }, oldReply: 'error' },
  // KIS's stored paper metadata stays unchanged; no imaginary Toss paper mode is introduced.
  { name: 'broker-only', surface: 'Strategy', next: { broker: 'toss' }, oldReply: 'success' },
]

for (const change of changes) test(`P2-KBD-02 source KIS ${change.surface} ${change.surface === 'Strategy' ? 'scope reset and explicit A resubmission' : 'same raw A'} across ${change.name} fences old work and preserves new busy`, async ({ page }) => {
  const state = await keyboardSearchRaceFixture(page, 'kis'); states.set(page, state)
  const harness = await installKeyboardScopeHarness(page, state)
  const view = await openSearchSurface(page, change.surface)
  expect(harness.intercepted()).toBe(true)
  const identity = await harness.identity(), previous = { ...state.scope }
  state.hold('A', 'kis'); state.hold('A', change.next.broker ?? 'kis')
  await view.input.fill('A'); await view.input.press('Enter')
  const old = await waitSearchReply(state, 'A', 1, 'kis')
  await harness.change(change.next)
  const currentScope = { ...previous, ...change.next }
  expect(state.scope).toEqual(currentScope)
  await expect.poll(harness.current).toMatchObject({ active_broker_id: currentScope.broker,
    active_profile_id: currentScope.profile, active_broker_account_id: currentScope.account, kis_is_paper_trading: currentScope.paper })
  const indicator = page.getByLabel('현재 broker scope').first()
  await expect(indicator).toContainText(currentScope.broker === 'kis' ? 'KIS' : 'Toss')
  await expect(indicator).toContainText(`Synthetic ${currentScope.profile}`)
  await expect(indicator).toContainText(currentScope.account)
  if (currentScope.broker === 'kis') await expect(indicator).toContainText(currentScope.paper ? '모의투자' : '실전투자')
  expect(await harness.identity()).toBe(identity)
  if (change.surface === 'Strategy') {
    // Existing activeScopeKey policy intentionally clears Strategy input and selection.
    await expect(view.input).toHaveValue('')
    await expectNoSearchSelection(page, 'Strategy', 'A')
    await expectNoSearchError(page, 'Strategy', 'A')
    await expect(view.button).toHaveAttribute('aria-busy', 'false')
    await expect(view.button).toBeDisabled()
    await view.input.fill('A') // A new explicit request after the normal scope reset.
  } else await expect(view.input).toHaveValue('A')
  const ordinal = state.reads('A', currentScope.broker).filter(ticket => ticket.held).length + 1
  await view.input.press('Enter')
  const current = await waitSearchReply(state, 'A', ordinal, currentScope.broker)
  expect(old.scope).toEqual(previous)
  expect(current.scope).toEqual(currentScope)
  const fallbackBefore = state.tickets.filter(ticket => /\/overseas-price\/(NYS|AMS)\/A$/.test(ticket.path)).length
  await settleSearchReply(page, old, change.oldReply === 'error' ? failedSearchReply() : change.oldReply === 'invalid' ? invalidSearchReply('kis', 'A') : undefined)
  await expect(view.input).toHaveValue('A')
  await expectNoSearchSelection(page, change.surface, 'A'); await expectNoSearchError(page, change.surface, 'A')
  expect(old.browserFinished).toBe(true)
  expect(current.settled).toBe(false)
  expect(state.tickets.filter(ticket => /\/overseas-price\/(NYS|AMS)\/A$/.test(ticket.path)).length).toBe(fallbackBefore)
  await expect(view.button).toHaveAttribute('aria-busy', 'true')
  state.notes.push({ stage: 'isolated-scope-change-old-body-finished', boundary: change.name, identity, oldId: old.id,
    currentId: current.id, oldScope: old.scope, currentScope: current.scope, oldReply: change.oldReply, strategyInputReset: change.surface === 'Strategy' })
  state.auto('A', 'kis'); state.auto('A', currentScope.broker)
  await settleSearchReply(page, current)
  await expectSearchSelection(page, change.surface, 'A')
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
})

test('P2-KBD-02 source KIS Trading account ABA cannot revive an earlier same-key request or finish the new one', async ({ page }) => {
  const state = await keyboardSearchRaceFixture(page, 'kis'); states.set(page, state)
  const harness = await installKeyboardScopeHarness(page, state)
  const view = await openSearchSurface(page, 'Trading')
  expect(harness.intercepted()).toBe(true)
  const identity = await harness.identity(), original = { ...state.scope }
  state.hold('A')
  await view.input.fill('A'); await view.input.press('Enter'); const first = await waitSearchReply(state, 'A')
  await harness.change({ account: 'SYNTHETIC-ACCOUNT-B' })
  await expect(view.input).toHaveValue('A')
  await view.input.press('Enter'); const middle = await waitSearchReply(state, 'A', 2)
  await harness.change({ account: original.account })
  await expect.poll(harness.current).toMatchObject({ active_broker_account_id: original.account })
  await expect(view.input).toHaveValue('A')
  await view.input.press('Enter'); const newest = await waitSearchReply(state, 'A', 3)
  expect(first.scope).toEqual(newest.scope)
  expect(middle.scope.account).not.toBe(first.scope.account)
  expect(new Set([first.id, middle.id, newest.id]).size).toBe(3)
  expect(await harness.identity()).toBe(identity)
  await settleSearchReply(page, first)
  await expectNoSearchSelection(page, 'Trading', 'A')
  await expect(view.button).toHaveAttribute('aria-busy', 'true')
  await settleSearchReply(page, middle, failedSearchReply())
  await expectNoSearchSelection(page, 'Trading', 'A'); await expectNoSearchError(page, 'Trading', 'A')
  expect(state.tickets.filter(ticket => /\/overseas-price\/(NYS|AMS)\/A$/.test(ticket.path))).toHaveLength(0)
  expect(newest.settled).toBe(false)
  await expect(view.button).toHaveAttribute('aria-busy', 'true')
  state.auto('A'); await settleSearchReply(page, newest)
  await expectSearchSelection(page, 'Trading', 'A')
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
  state.notes.push({ stage: 'same-raw-scope-ABA', identity, first: first.id, middle: middle.id, newest: newest.id,
    scopes: [first.scope, middle.scope, newest.scope] })
})


test('P2-KBD-02 source KIS Trading same raw A across profile-only change rejects old invalid body and preserves new busy', async ({ page }) => {
  const state = await keyboardSearchRaceFixture(page, 'kis'); states.set(page, state)
  const harness = await installKeyboardScopeHarness(page, state)
  const view = await openSearchSurface(page, 'Trading')
  expect(harness.intercepted()).toBe(true)
  const identity = await harness.identity(), previous = { ...state.scope }
  state.hold('A')
  await view.input.fill('A'); await view.input.press('Enter')
  const old = await waitSearchReply(state, 'A')
  await harness.change({ profile: 'synthetic-profile-b' })
  const next = { ...previous, profile: 'synthetic-profile-b' }
  expect(state.scope).toEqual(next)
  await expect.poll(harness.current).toMatchObject({ active_broker_id: next.broker, active_profile_id: next.profile,
    active_broker_account_id: next.account, kis_is_paper_trading: next.paper })
  await expect(page.getByLabel('현재 broker scope').first()).toContainText(`Synthetic ${next.profile}`)
  expect(await harness.identity()).toBe(identity)
  await expect(view.input).toHaveValue('A') // No edit: profile is the only invalidation boundary.
  await view.input.press('Enter')
  const current = await waitSearchReply(state, 'A', 2)
  expect(old.path).toBe(current.path); expect(old.id).not.toBe(current.id)
  expect(old.scope).toEqual(previous); expect(current.scope).toEqual(next)
  await settleSearchReply(page, old, invalidSearchReply('kis', 'A'))
  await expect(view.input).toHaveValue('A')
  await expectNoSearchSelection(page, 'Trading', 'A'); await expectNoSearchError(page, 'Trading', 'A')
  expect(state.tickets.filter(ticket => /\/overseas-price\/(NYS|AMS)\/A$/.test(ticket.path))).toHaveLength(0)
  expect(current.settled).toBe(false)
  await expect(view.button).toHaveAttribute('aria-busy', 'true')
  state.notes.push({ stage: 'same-raw-profile-only-old-invalid-body-finished', identity, oldId: old.id,
    currentId: current.id, oldScope: old.scope, currentScope: current.scope, oldBodyFinished: old.browserFinished })
  state.auto('A'); await settleSearchReply(page, current)
  await expectSearchSelection(page, 'Trading', 'A')
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
})

test('P2-KBD-02 source Trading same raw A across broker-only change preserves Toss immediate selection against old KIS invalid body', async ({ page }) => {
  const state = await keyboardSearchRaceFixture(page, 'kis'); states.set(page, state)
  const harness = await installKeyboardScopeHarness(page, state)
  const view = await openSearchSurface(page, 'Trading')
  expect(harness.intercepted()).toBe(true)
  const identity = await harness.identity(), previous = { ...state.scope }
  state.hold('A', 'kis'); state.hold('A', 'toss')
  await view.input.fill('A'); await view.input.press('Enter')
  const old = await waitSearchReply(state, 'A', 1, 'kis')
  await harness.change({ broker: 'toss' })
  const next: SearchScope = { ...previous, broker: 'toss' }
  expect(state.scope).toEqual(next)
  await expect.poll(harness.current).toMatchObject({ active_broker_id: next.broker, active_profile_id: next.profile,
    active_broker_account_id: next.account, kis_is_paper_trading: next.paper })
  await expect(page.getByLabel('현재 broker scope').first()).toContainText('Toss')
  expect(await harness.identity()).toBe(identity)
  await expect(view.input).toHaveValue('A') // Broker alone changes; do not conflate it with an input edit.
  await view.input.press('Enter')
  const current = await waitSearchReply(state, 'A', 1, 'toss')
  expect(old.scope).toEqual(previous); expect(current.scope).toEqual(next)
  expect(old.id).not.toBe(current.id)
  expect(old.path).toBe('/api/overseas-price/NAS/A'); expect(current.path).toBe('/api/toss-stock-safety/A')
  await expectSearchSelection(page, 'Trading', 'A')
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
  expect(current.settled).toBe(false)
  await settleSearchReply(page, old, invalidSearchReply('kis', 'A'))
  await expect(view.input).toHaveValue('A')
  await expectSearchSelection(page, 'Trading', 'A'); await expectNoSearchError(page, 'Trading', 'A')
  expect(state.tickets.filter(ticket => /\/overseas-price\/(NYS|AMS)\/A$/.test(ticket.path))).toHaveLength(0)
  expect(state.reads('A', 'toss')).toHaveLength(1)
  expect(current.settled).toBe(false)
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
  state.notes.push({ stage: 'same-raw-broker-only-old-KIS-invalid-body-finished', identity, oldId: old.id,
    currentId: current.id, oldScope: old.scope, currentScope: current.scope, oldBodyFinished: old.browserFinished,
    tossImmediateSelectionBeforeSafety: true, tossSafetyStillHeld: !current.settled })
  state.auto('A', 'kis'); state.auto('A', 'toss'); await settleSearchReply(page, current)
  await expectSearchSelection(page, 'Trading', 'A')
  await expectNoSearchError(page, 'Trading', 'A')
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
})


test('P2-KBD-02 source LTH synthetic enabled GET locks a pending picker without reviving success or direct fallback and unlocks normally', async ({ page }) => {
  const state = await keyboardSearchRaceFixture(page, 'kis'); states.set(page, state)
  const harness = await installKeyboardScopeHarness(page, state)
  const view = await openSearchSurface(page, 'LTH')
  expect(harness.intercepted()).toBe(true)
  expect(state.strategyState.enabled).toBe(false)
  const identity = await harness.identity(), originalScope = { ...state.scope }
  const originalInput = await view.input.elementHandle()
  if (!originalInput) throw new Error('No original LTH picker input')
  const add = view.card.getByRole('button', { name: '대상 추가', exact: true })
  await expect(add).toHaveCount(2) // Header and picker action keep their existing add paths.
  const fallbackReads = () => state.tickets.filter(ticket => /\/overseas-price\/(NYS|AMS)\/A$/.test(ticket.path)).length
  const checkIdentity = async () => {
    expect(await harness.identity()).toBe(identity)
    expect(state.scope).toEqual(originalScope)
    expect(await view.input.evaluate((element, original) => element === original, originalInput)).toBe(true)
  }
  const setSyntheticLock = async (enabled: boolean) => {
    state.strategyState.enabled = enabled
    await harness.change({}) // Existing GET invalidation only: never click a strategy switch or save.
    await checkIdentity()
    if (enabled) {
      await expect(view.input).toBeDisabled(); await expect(view.button).toBeDisabled()
      await expect(add.nth(0)).toBeDisabled(); await expect(add.nth(1)).toBeDisabled()
    } else {
      await expect(view.input).toBeEnabled(); await expect(view.button).toBeEnabled()
    }
  }
  state.hold('A')
  await view.input.fill('A'); await view.input.press('Enter')
  const oldSuccess = await waitSearchReply(state, 'A')
  await expect(view.button).toHaveAttribute('aria-busy', 'true')
  await setSyntheticLock(true)
  await settleSearchReply(page, oldSuccess)
  await expectNoSearchSelection(page, 'LTH', 'A'); await expectNoSearchError(page, 'LTH', 'A')
  await expect(view.card).not.toContainText('검증은 실패했지만')
  expect(fallbackReads()).toBe(0)
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
  await expect(add.nth(0)).toBeDisabled(); await expect(add.nth(1)).toBeDisabled()
  state.notes.push({ stage: 'synthetic-enabled-lock-old-success-body-finished', oldId: oldSuccess.id,
    oldBodyFinished: oldSuccess.browserFinished, identity, originalScope })

  await setSyntheticLock(false)
  await expect(view.input).toHaveValue('A')
  await view.input.press('Enter')
  const oldInvalid = await waitSearchReply(state, 'A', 2)
  await expect(view.button).toHaveAttribute('aria-busy', 'true')
  await setSyntheticLock(true)
  await settleSearchReply(page, oldInvalid, invalidSearchReply('kis', 'A'))
  await expectNoSearchSelection(page, 'LTH', 'A'); await expectNoSearchError(page, 'LTH', 'A')
  await expect(view.card).not.toContainText('검증은 실패했지만')
  expect(fallbackReads()).toBe(0)
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
  await expect(add.nth(0)).toBeDisabled(); await expect(add.nth(1)).toBeDisabled()
  state.notes.push({ stage: 'synthetic-enabled-lock-old-invalid-body-finished', oldId: oldInvalid.id,
    oldBodyFinished: oldInvalid.browserFinished, noDirectTickerFallback: true, noFallbackReads: fallbackReads() === 0 })

  await setSyntheticLock(false)
  state.hold('B'); await view.input.fill('B'); await view.input.press('Enter')
  const current = await waitSearchReply(state, 'B')
  state.auto('A'); state.auto('B'); await settleSearchReply(page, current)
  await expectSearchSelection(page, 'LTH', 'B'); await expectNoSearchError(page, 'LTH', 'A')
  await expect(view.card).not.toContainText('검증은 실패했지만')
  await expect(view.button).toHaveAttribute('aria-busy', 'false')
  await expect(add.nth(0)).toBeEnabled(); await expect(add.nth(1)).toBeEnabled()
  await checkIdentity()
  expect(state.audit.writes).toEqual([])
  state.notes.push({ stage: 'synthetic-enabled-unlock-fresh-B-recovery', currentId: current.id,
    sameMainAndPickerInput: true, strategyWrites: state.audit.writes, strictModeClaim: 'source initial setup/cleanup and ordinary lookup only; no separate pending dispose-activate reproduction' })
})
