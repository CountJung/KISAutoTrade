import { expect, test, type Locator, type Page } from '@playwright/test'
import { mkdir, writeFile } from 'node:fs/promises'
import { keyboardSearchFixture, safeKeyboardSearch, keyboardObservationWindow, keyboardStock, enterTerm, buttonTerm, oneLetterTicker } from './fixtures/keyboardSearchFixture'
import type { KeyboardSearchFixture } from './fixtures/keyboardSearchFixture'

const states = new WeakMap<Page, KeyboardSearchFixture>()
const directory = '.cache/kbd01-implementation-20261031/evidence'
type Surface = 'Trading' | 'Strategy' | 'LTH'
async function setup(page: Page, broker: 'kis' | 'toss') {
  const state = await keyboardSearchFixture(page, broker); states.set(page, state); return state
}
async function usInput(page: Page, surface: Surface): Promise<Locator> {
  await page.goto(surface === 'Trading' ? '/trading' : '/strategy')
  if (surface === 'LTH') {
    const card = page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' })
    const input = card.getByRole('textbox', { name: 'ETF 티커', exact: true })
    await expect(input).toBeVisible() // Existing picker default is US.
    return input
  }
  await page.getByRole('button', { name: surface === 'Trading' ? '미국' : '해외 (미국)', exact: true }).click()
  return page.getByRole('textbox', { name: surface === 'Trading' ? '티커 (AAPL, TSLA …)' : '티커 (예: AAPL, SPYM, QQQ)', exact: true })
}
async function selected(page: Page, surface: Surface) {
  if (surface === 'Trading') await expect(page.getByTestId('candle-data-summary')).toHaveAccessibleName(`${oneLetterTicker} 차트 자료`)
  else if (surface === 'Strategy') await expect(page.getByText(`선택됨: Synthetic Z (${oneLetterTicker})`, { exact: true })).toBeVisible()
  else {
    const card = page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' })
    const quantity = card.getByRole('spinbutton', { name: '1회 수량', exact: true })
    await expect(quantity).toHaveCount(1)
    // Closest ancestor with a direct add-button is this picker action Stack, not another card control.
    const pickerActions = quantity.locator('xpath=ancestor::*[child::button[normalize-space(.)="대상 추가"]][1]')
    await expect(pickerActions).toHaveCount(1)
    await expect(pickerActions.getByText(`Synthetic Z (${oneLetterTicker})`, { exact: true })).toBeVisible()
    const add = pickerActions.getByRole('button', { name: '대상 추가', exact: true })
    await expect(add).toHaveCount(1)
    await expect(add).toBeEnabled()
  }
}
test.afterEach(async ({ page }, info) => {
  const state = states.get(page)
  if (!state) return
  try {
    await state.release()
    await mkdir(directory, { recursive: true })
    const name = info.title.replace(/[^a-zA-Z0-9-]/g, '_')
    await writeFile(`${directory}/${name}.json`, JSON.stringify({ status: info.status, broker: state.broker, notes: state.notes,
      reads: state.reads, readPosts: state.readPosts, counts: Object.fromEntries(state.audit.counts), writes: state.audit.writes,
      external: state.audit.external, unknown: state.audit.unknown, pageErrors: state.audit.errors }, null, 2))
    await page.screenshot({ path: `${directory}/${name}.png`, animations: 'disabled' })
  } finally { safeKeyboardSearch(state) }
})

for (const broker of ['kis', 'toss'] as const) {
  const label = broker.toUpperCase()
  test(`P2-KBD-01 ${label} contract Trading KR Enter uses the search path and preserves direct code and button keys`, async ({ page }) => {
    const state = await setup(page, broker)
    await page.goto('/trading')
    const input = page.getByRole('textbox', { name: '6자리 종목코드 (예: 005930, 0005A0)', exact: true })
    await expect(page.getByLabel('현재 broker scope').first().getByText(broker === 'kis' ? 'KIS' : 'Toss', { exact: true })).toBeVisible()
    await page.clock.pauseAt(await page.evaluate(() => Date.now() + 100))
    await input.fill('가'); await input.press('Enter'); await page.clock.runFor(1)
    expect(state.reads.filter(read => read.path.startsWith('/api/search/')).length).toBe(0)
    await input.fill(enterTerm); await page.clock.runFor(1)
    expect(state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0).toBe(0)
    for (const variant of [{ isComposing: true, keyCode: 13 }, { isComposing: false, keyCode: 229 }, { isComposing: true, keyCode: 229 }]) {
      await input.dispatchEvent(variant.isComposing ? 'compositionstart' : 'compositionend', { data: enterTerm })
      await input.dispatchEvent('keydown', { key: 'Enter', code: 'Enter', ...variant, which: variant.keyCode, bubbles: true })
      await page.clock.runFor(1)
      state.notes.push({ stage: 'Trading-KR-IME-before-auto', variant, reads: state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0 })
      expect(state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0).toBe(0)
      await input.dispatchEvent('compositionend', { data: enterTerm })
    }
    const frozenAt = await page.evaluate(() => Date.now())
    await input.press('Enter')
    await expect.poll(() => state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0).toBe(1)
    expect(await page.evaluate(() => Date.now())).toBe(frozenAt)
    await page.clock.runFor(600)
    state.notes.push({ stage: 'Trading-KR-immediate-Enter-and-auto400-boundary', frozenAt, searchReads: state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0 })
    expect(state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0).toBe(1)
    const result = page.getByRole('button', { name: `${keyboardStock.prdt_name} (${keyboardStock.pdno}) 선택`, exact: true })
    await expect(result).toBeVisible()
    await input.press('Escape'); await expect(input).toBeFocused(); await expect(result).toHaveCount(0)
    await input.fill(buttonTerm); await input.press('Tab')
    await expect(page.getByRole('button', { name: '국내 종목 검색', exact: true })).toBeFocused()
    const buttonResponse = page.waitForResponse(response => new URL(response.url()).pathname === `/api/search/${encodeURIComponent(buttonTerm)}`)
    await page.keyboard.press('Space')
    await (await buttonResponse).finished()
    await page.evaluate(() => Promise.resolve()); await page.clock.runFor(1)
    await expect(result).toBeVisible()
    expect(state.audit.counts.get(`/api/search/${encodeURIComponent(buttonTerm)}`)).toBe(1)
    await input.fill(keyboardStock.pdno)
    await expect(page.getByText(`선택됨: ${keyboardStock.pdno}`, { exact: true })).toBeVisible()
    await input.press('Enter'); await keyboardObservationWindow(page)
    expect(state.audit.counts.get(`/api/search/${keyboardStock.pdno}`) ?? 0).toBe(0)
  })

  for (const surface of ['Trading', 'Strategy', 'LTH'] as const) test(`P2-KBD-01 ${label} contract ${surface} US ignores composing Enter and 229 but permits a one-letter ticker after composition`, async ({ page }) => {
    const state = await setup(page, broker)
    const input = await usInput(page, surface)
    await input.fill(' '); await input.press('Enter'); await keyboardObservationWindow(page)
    expect(state.logicalReads()).toBe(0)
    if (surface === 'LTH') {
      await input.fill('1'); await input.press('Enter'); await keyboardObservationWindow(page)
      expect(state.logicalReads()).toBe(0) // Preserve the existing LTH US ticker regex only here.
    }
    await input.fill(oneLetterTicker)
    for (const variant of [{ isComposing: true, keyCode: 13 }, { isComposing: false, keyCode: 229 }, { isComposing: true, keyCode: 229 }]) {
      await input.evaluate(element => element.addEventListener('keydown', event => {
        const key = event as KeyboardEvent
        Reflect.set(window, 'kbd01Event', { isComposing: key.isComposing, keyCode: key.keyCode, isTrusted: key.isTrusted })
      }, { capture: true, once: true }))
      await input.dispatchEvent(variant.isComposing ? 'compositionstart' : 'compositionend', { data: oneLetterTicker })
      await input.dispatchEvent('keydown', { key: 'Enter', code: 'Enter', ...variant, which: variant.keyCode, bubbles: true })
      await keyboardObservationWindow(page)
      const actual = await page.evaluate(() => Reflect.get(window, 'kbd01Event'))
      state.notes.push({ surface, stage: 'IME-guard', variant, actual, logicalReads: state.logicalReads() })
      expect(actual).toMatchObject({ ...variant, isTrusted: false })
      expect(state.logicalReads()).toBe(0)
      await input.dispatchEvent('compositionend', { data: oneLetterTicker })
    }
    await input.press('Enter')
    await expect.poll(state.logicalReads).toBe(1) // Held response excludes normal post-selection hooks.
    await state.release(); await selected(page, surface)
  })

  test(`P2-KBD-01 ${label} contract Trading US search button has a name and native Tab Enter performs exactly one logical read`, async ({ page }) => {
    const state = await setup(page, broker)
    const input = await usInput(page, 'Trading')
    await input.fill(oneLetterTicker); await input.focus(); await input.press('Tab')
    const button = input.locator('..').getByRole('button')
    await expect(button).toBeFocused()
    await page.keyboard.press('Enter'); await keyboardObservationWindow(page)
    state.notes.push({ stage: 'adornment-native-Enter-before-release', logicalReads: state.logicalReads() })
    expect(state.logicalReads()).toBe(1)
    await state.release(); await selected(page, 'Trading')
    await expect(input.locator('..').getByRole('button')).toHaveAccessibleName('미국 종목 검색')
  })
}


// Separate Enter intent from the existing Strategy immediate-code / LTH 250ms auto query.
for (const broker of ['kis', 'toss'] as const) for (const surface of ['Strategy', 'LTH'] as const) {
  test(`P2-KBD-01 ${broker.toUpperCase()} contract ${surface} KR Enter starts a name query before auto debounce and ignores IME without duplicate reads`, async ({ page }) => {
    const state = await setup(page, broker)
    await page.goto('/strategy')
    const card = page.getByTestId('strategy-card-grid').filter({ hasText: 'LeveragedTrendHoldStrategy' })
    await expect(card.getByRole('textbox', { name: 'ETF 티커', exact: true })).toBeVisible()
    if (surface === 'LTH') await card.getByRole('button', { name: '국내', exact: true }).click()
    const input = surface === 'LTH' ? card.getByRole('textbox', { name: 'ETF 코드 또는 이름', exact: true })
      : page.getByRole('textbox', { name: '6자리 종목코드 (예: 005930, 0005A0)', exact: true })
    const searchButton = surface === 'LTH' ? card.getByRole('button', { name: '레버리지 대상 검색', exact: true })
      : page.getByRole('button', { name: '국내 종목 검색', exact: true })
    await expect(input).toBeVisible()
    // Initial query UI is ready; freezing now cannot block its first TanStack notification.
    await page.clock.pauseAt(await page.evaluate(() => Date.now() + 100))
    await input.fill('가'); await input.press('Enter'); await page.clock.runFor(1)
    await expect(searchButton).toBeDisabled()
    expect(state.reads.filter(read => read.path.startsWith('/api/search/')).length).toBe(0)

    // Existing button is the positive fixture/path control BEFORE the new Enter requirement.
    await input.fill(buttonTerm); await input.press('Tab'); await expect(searchButton).toBeFocused()
    await page.keyboard.press('Space')
    await expect.poll(() => state.audit.counts.get(`/api/search/${encodeURIComponent(buttonTerm)}`) ?? 0).toBe(1)
    state.notes.push({ surface, stage: 'existing-min-two-name-button-control', count: state.audit.counts.get(`/api/search/${encodeURIComponent(buttonTerm)}`) })

    await input.focus(); await input.fill(enterTerm); await page.clock.runFor(1)
    for (const variant of [{ isComposing: true, keyCode: 13 }, { isComposing: false, keyCode: 229 }, { isComposing: true, keyCode: 229 }]) {
      await input.dispatchEvent(variant.isComposing ? 'compositionstart' : 'compositionend', { data: enterTerm })
      await input.dispatchEvent('keydown', { key: 'Enter', code: 'Enter', ...variant, which: variant.keyCode, bubbles: true })
      await page.clock.runFor(1) // Below 250ms: never count LTH's later automatic query as an Enter query.
      state.notes.push({ surface, stage: 'KR-composition-before-auto', variant, nameReads: state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0 })
      expect(state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0).toBe(0)
      await input.dispatchEvent('compositionend', { data: enterTerm })
    }
    const frozenAt = await page.evaluate(() => Date.now())
    await input.press('Enter')
    state.notes.push({ surface, stage: 'normal-KR-Enter-clock-frozen-before-auto', frozenAt })
    // Playwright's real-time poll leaves browser time frozen: baseline's auto250 cannot satisfy this.
    await expect.poll(() => state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0).toBe(1)
    expect(await page.evaluate(() => Date.now())).toBe(frozenAt)
    await page.clock.runFor(300)
    expect(state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`)).toBe(1)
    state.notes.push({ surface, stage: 'after-original-auto-debounce', nameReads: state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) })

    if (surface === 'Strategy') {
      const result = page.getByRole('button', { name: `${keyboardStock.prdt_name} (${keyboardStock.pdno}) 선택`, exact: true })
      await expect(result).toBeVisible()
      const cachedReadsBefore = state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0
      await input.press('Escape'); await page.clock.runFor(1)
      await expect(result).toHaveCount(0)
      await expect(input).toBeFocused()
      await expect(input).toHaveValue(enterTerm)
      state.notes.push({ stage: 'Strategy-name-Escape-closed', cachedReadsBefore })
      await input.press('Enter'); await page.clock.runFor(1)
      await expect(result).toBeVisible()
      await expect(input).toBeFocused()
      await page.clock.runFor(600)
      await expect(result).toBeVisible()
      await expect(input).toBeFocused()
      state.notes.push({ stage: 'Strategy-unchanged-name-Enter-reopen-after-1-and-600ms', cachedReadsBefore,
        cachedReadsAfter: state.audit.counts.get(`/api/search/${encodeURIComponent(enterTerm)}`) ?? 0 })
      // A fresh cached result is valid: reopening does not require an extra GET.
    }

    await input.fill(keyboardStock.pdno)
    if (surface === 'LTH') {
      await page.clock.runFor(249)
      expect(state.audit.counts.get(`/api/search/${keyboardStock.pdno}`) ?? 0).toBe(0)
      await page.clock.runFor(1) // Preserve LTH's existing 250ms code lookup.
    } else await page.clock.runFor(1) // Strategy's existing six-character lookup is immediate.
    await expect.poll(() => state.audit.counts.get(`/api/search/${keyboardStock.pdno}`) ?? 0).toBe(1)
    await input.press('Enter'); await page.clock.runFor(300)
    expect(state.audit.counts.get(`/api/search/${keyboardStock.pdno}`)).toBe(1)
    state.notes.push({ surface, stage: 'existing-six-character-auto-plus-Enter', codeReads: state.audit.counts.get(`/api/search/${keyboardStock.pdno}`) })
  })
}
