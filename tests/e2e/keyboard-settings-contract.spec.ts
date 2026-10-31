import { expect, test, type Page } from '@playwright/test'
import { keyboardSettingsFixture, settingsPairs, settingsControl, openSettingsContract, controlNamingObservation,
  nativeBlurCount, safeSettings, captureSettings, namedSettingsControl, settingsTick, riskSave, baseRiskPayload, expectedSettingsArgs, waitSettingsWrite,
  type SettingsPair, type SettingsFixtureOptions, type SettingsFixture } from './fixtures/keyboardSettingsFixture'

const states = new WeakMap<Page, SettingsFixture>()
test.afterEach(async ({ page }, info) => { const state = states.get(page); if (state) try { await captureSettings(page, state, info.title, info.status) } finally { await safeSettings(state) } })
async function setup(page: Page, options: SettingsFixtureOptions & { narrow?: boolean; open?: boolean } = {}) {
  const { narrow = false, open = true, ...fixtureOptions } = options
  await page.setViewportSize(narrow ? { width: 375, height: 812 } : { width: 1280, height: 900 }); await page.emulateMedia({ colorScheme: narrow ? 'dark' : 'light' })
  const state = await keyboardSettingsFixture(page, fixtureOptions); states.set(page, state)
  if (open) await openSettingsContract(page)
  else { await page.goto('/settings'); await expect(page.getByRole('heading', { name: 'Settings', exact: true })).toBeVisible() }
  return state
}

test('P2-KBD-04 minimum all 11 settings pairs expose 22 static names and linked units', async ({ page }) => {
  const state = await setup(page)
  const observations = []
  // Collect every target before any name expectation, so the first missing name cannot hide the other 21.
  for (const pair of settingsPairs) {
    const control = settingsControl(page, pair)
    observations.push({ ...pair, slider: await controlNamingObservation(control.slider), textbox: await controlNamingObservation(control.input) })
  }
  state.notes.push({ stage: 'all-eleven-pairs-twenty-two-control-names-before-assertions', observations })
  expect(observations).toHaveLength(11)
  expect(observations.flatMap(item => [item.slider.count, item.textbox.count])).toEqual(Array(22).fill(1))
  for (const [index, pair] of settingsPairs.entries()) {
    const control = settingsControl(page, pair)
    await expect(control.slider).toHaveAccessibleName(pair.label)
    await expect(control.input).toHaveAccessibleName(pair.label)
    for (const observed of [observations[index].slider, observations[index].textbox]) {
      const links = observed.links!
      const linkedUnitText = [links.ariaLabel, ...links.labels, ...links.labelledBy.map(link => link.text), ...links.describedBy.map(link => link.text), links.valueText].filter(Boolean).join(' ')
      expect(linkedUnitText).toContain(pair.unit)
    }
  }
  expect((await state.snapshot()).mockWrites).toEqual([])
})

for (const composing of [true, false]) test(`P2-KBD-04 minimum log retention ordinary Enter positive then ${composing ? 'native isComposing' : 'standalone229'} Enter neither blurs nor saves`, async ({ page }) => {
  const state = await setup(page), control = settingsControl(page, settingsPairs[1]), blurs = await nativeBlurCount(control.input)
  await control.input.fill('7'); await control.input.press('Enter')
  await expect.poll(async () => (await state.snapshot()).mockWrites.length).toBe(1)
  await expect(control.input).toBeEnabled(); await expect(control.input).not.toBeFocused()
  expect(await blurs()).toBe(1)
  expect((await state.snapshot()).mockWrites.map(({ command, args }) => ({ command, args }))).toEqual([
    { command: 'set_log_config', args: { input: { retention_days: 7, max_size_mb: 100, api_debug: false } } },
  ])
  state.notes.push({ stage: 'ordinary-Enter-existing-blur-path-positive', mockSettingsWrites: 1, nativeBlur: await blurs(), actualSettingsWrites: 0 })
  await control.input.fill('8'); await expect(control.input).toBeFocused()
  await control.input.evaluate(element => {
    Reflect.set(element, '__kbd04CompositionEvent', null)
    element.addEventListener('keydown', event => Reflect.set(element, '__kbd04CompositionEvent', event), { once: true, passive: true, capture: true })
  })
  await control.input.dispatchEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: composing ? 13 : 229, isComposing: composing, bubbles: true, cancelable: true })
  await page.clock.runFor(1); await page.evaluate(() => Promise.resolve())
  const observed = await control.input.evaluate(element => {
    const field = element as HTMLInputElement, event = Reflect.get(field, '__kbd04CompositionEvent') as KeyboardEvent | null
    return { focused: document.activeElement === field, draft: field.value,
      event: event ? { key: event.key, keyCode: event.keyCode, isComposing: event.isComposing, isTrusted: event.isTrusted, defaultPrevented: event.defaultPrevented } : null }
  })
  const after = await state.snapshot()
  state.notes.push({ stage: 'synthetic-DOM-composition-Enter-before-assertions', composing, ...observed,
    additionalNativeBlurs: (await blurs()) - 1, additionalMockSettingsWrites: after.mockWrites.length - 1,
    limitation: 'DOM KeyboardEvent/nativeEvent only, isTrusted=false; not actual OS IME or native Tauri execution' })
  expect(observed.event).toMatchObject({ key: 'Enter', keyCode: composing ? 13 : 229, isComposing: composing, isTrusted: false })
  expect(observed.focused).toBe(true); expect(observed.draft).toBe('8')
  expect(await blurs()).toBe(1); expect(after.mockWrites).toHaveLength(1)
  // A subsequent non-composing Enter still reaches the original blur/commit once.
  await control.input.press('Enter')
  await expect.poll(async () => (await state.snapshot()).mockWrites.length).toBe(2)
  expect(await blurs()).toBe(2)
  expect((await state.snapshot()).mockWrites[1]).toMatchObject({ command: 'set_log_config', args: { input: { retention_days: 8, max_size_mb: 100, api_debug: false } } })
})


async function pairIdentity(page: Page, pair: SettingsPair) {
  const control = namedSettingsControl(page, pair)
  return { slider: await control.slider.getAttribute('id'), input: await control.input.getAttribute('id'),
    label: await control.slider.getAttribute('aria-labelledby'), description: await control.input.getAttribute('aria-describedby') }
}
async function assertStablePair(page: Page, pair: SettingsPair, identity: Awaited<ReturnType<typeof pairIdentity>>) {
  const control = namedSettingsControl(page, pair)
  await expect(control.slider).toHaveAccessibleName(pair.label); await expect(control.input).toHaveAccessibleName(pair.label)
  expect(await pairIdentity(page, pair)).toEqual(identity)
}
async function assertWrite(page: Page, state: SettingsFixture, pair: SettingsPair, value: number, ordinal: number) {
  const call = await waitSettingsWrite(state, pair.command, ordinal, 'success')
  expect(call.args).toEqual(expectedSettingsArgs(pair, value))
  await settingsTick(page)
  await expect(namedSettingsControl(page, pair).input).toBeEnabled()
  return call
}

// The log retention pair is covered by the two retained minimum IME cases above.
for (const pair of settingsPairs.filter(pair => pair !== settingsPairs[1])) test(`P2-KBD-04 ${pair.section} ${pair.label} composing guards then one existing Enter commit`, async ({ page }) => {
  const state = await setup(page), control = namedSettingsControl(page, pair), identity = await pairIdentity(page, pair)
  const blurs = await nativeBlurCount(control.input)
  await control.input.fill(pair.draft)
  for (const composing of [true, false]) {
    await control.input.dispatchEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: composing ? 13 : 229, isComposing: composing, bubbles: true, cancelable: true })
    await settingsTick(page)
    await expect(control.input).toBeFocused(); await expect(control.input).toHaveValue(pair.draft)
    expect(await blurs()).toBe(0); expect((await state.snapshot()).mockWrites).toHaveLength(0)
  }
  await control.input.press('Enter')
  await expect(control.input).not.toBeFocused(); await expect(control.input).toHaveValue(String(pair.rounded))
  expect(await blurs()).toBe(1)
  if (pair.family === 'risk') {
    expect((await state.snapshot()).mockWrites).toHaveLength(0)
    await expect(riskSave(page)).toBeEnabled(); await riskSave(page).click()
    await assertWrite(page, state, pair, pair.rounded, 1)
    await expect(riskSave(page)).toBeDisabled()
  } else await assertWrite(page, state, pair, pair.rounded, 1)
  await page.keyboard.press('Enter'); await settingsTick(page)
  expect((await state.snapshot()).mockWrites).toHaveLength(1)
  await assertStablePair(page, pair, identity)
  state.notes.push({ stage: 'pair-IME-Enter-contract', label: pair.label, family: pair.family, nativeBlurs: await blurs(),
    mockSettingsWrites: 1, riskRequiresExplicitSave: pair.family === 'risk', draft: pair.draft, rounded: pair.rounded })
})

test('P2-KBD-04 narrow dark all 11 pairs retain IDs names units ranges and native Tab across rerender', async ({ page }) => {
  const state = await setup(page, { narrow: true })
  const identities = []
  for (const pair of settingsPairs) {
    const control = namedSettingsControl(page, pair)
    await expect(control.input).toHaveCount(1); await expect(control.slider).toHaveCount(1)
    await expect(control.slider).toHaveAttribute('min', String(pair.min)); await expect(control.slider).toHaveAttribute('max', String(pair.max))
    await expect(control.slider).toHaveAttribute('step', String(pair.step))
    const identity = await pairIdentity(page, pair); identities.push(identity)
    expect(identity.slider).toBeTruthy(); expect(identity.input).toBeTruthy(); expect(identity.label).toBeTruthy(); expect(identity.description).toBeTruthy()
    await expect(control.slider).toHaveAttribute('aria-labelledby', identity.label!)
    await expect(control.input).toHaveAttribute('aria-labelledby', identity.label!)
    const description = await control.input.getAttribute('aria-describedby')
    await expect(control.slider).toHaveAttribute('aria-describedby', description!)
    const link = await controlNamingObservation(control.input)
    const descriptionText = link.links!.describedBy.map(item => item.text).join(' ')
    for (const required of [pair.unit, String(pair.min), String(pair.max), String(pair.step)]) expect(descriptionText).toContain(required)
  }
  expect(new Set(identities.flatMap(item => [item.slider, item.input])).size).toBe(22)
  await page.setViewportSize({ width: 1280, height: 900 }); await page.emulateMedia({ colorScheme: 'light' })
  for (const [index, pair] of settingsPairs.entries()) await assertStablePair(page, pair, identities[index])
  await page.setViewportSize({ width: 375, height: 812 }); await page.emulateMedia({ colorScheme: 'dark' })
  const final = namedSettingsControl(page, settingsPairs[10])
  await final.slider.focus(); await page.keyboard.press('Tab'); await expect(final.input).toBeFocused()
  await page.keyboard.press('Shift+Tab'); await expect(final.slider).toBeFocused()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
  const geometry = await final.slider.evaluate(element => { const rect = element.getBoundingClientRect(); return { left: rect.left, right: rect.right, viewport: innerWidth } })
  expect(geometry.left).toBeGreaterThanOrEqual(-1); expect(geometry.right).toBeLessThanOrEqual(geometry.viewport + 1)
  expect((await state.snapshot()).mockWrites).toHaveLength(0)
  state.notes.push({ stage: 'all-pair-static-IDs-and-descriptions', identities, geometry, riskTabBlurIsLocalOnly: true })
})

for (const pair of [settingsPairs[0], settingsPairs[2], settingsPairs[4]]) test(`P2-KBD-04 ${pair.family} textbox Tab ShiftTab keep parser clamp rounding and empty restore`, async ({ page }) => {
  const state = await setup(page), control = namedSettingsControl(page, pair)
  let writes = 0
  for (const [draft, value, key] of [[pair.draft, pair.rounded, 'Tab'], ['', pair.rounded, 'Tab'],
    [String(pair.min / 2), pair.min, 'Shift+Tab'], [String(pair.max * 10), pair.max, 'Tab']] as const) {
    await control.input.fill(draft); await control.input.press(key)
    await expect(control.input).toHaveValue(String(value))
    if (draft) { writes++; await assertWrite(page, state, pair, value, writes) }
    else { await settingsTick(page); expect((await state.snapshot()).mockWrites).toHaveLength(writes) }
  }
  state.notes.push({ stage: 'existing-parser-blur', family: pair.family, saves: writes, emptyCommit: false, rounding: pair.rounded })
})

test('P2-KBD-04 risk parser rounds local drafts and explicit save preserves all original conversions', async ({ page }) => {
  const state = await setup(page)
  for (const pair of [settingsPairs[5], settingsPairs[6], settingsPairs[9], settingsPairs[10]]) {
    const control = namedSettingsControl(page, pair)
    await control.input.fill(pair.draft); await control.input.press('Tab')
    await expect(control.input).toHaveValue(String(pair.rounded))
    expect((await state.snapshot()).mockWrites).toHaveLength(0)
  }
  const ratio = namedSettingsControl(page, settingsPairs[6])
  await ratio.input.fill(''); await ratio.input.press('Shift+Tab'); await expect(ratio.input).toHaveValue('23')
  await expect(riskSave(page)).toBeEnabled(); await riskSave(page).click()
  const write = await waitSettingsWrite(state, 'update_risk_config', 1, 'success')
  expect(write.args).toEqual({ input: { ...baseRiskPayload, dailyLossLimit: 200000, maxPositionRatio: .23, riskPerTradeBps: 130, atrStopMultiplier: 2.5 } })
  await settingsTick(page); await expect(riskSave(page)).toBeDisabled()
})

for (const pair of [settingsPairs[0], settingsPairs[2], settingsPairs[4], settingsPairs[6]]) test(`P2-KBD-04 ${pair.family} native Slider arrows Home End retain original step commit and Tab order`, async ({ page }) => {
  const state = await setup(page), control = namedSettingsControl(page, pair)
  await control.slider.focus(); await page.keyboard.press('Tab'); await expect(control.input).toBeFocused()
  await page.keyboard.press('Shift+Tab'); await expect(control.slider).toBeFocused()
  let writes = pair.family === 'risk' ? 0 : 1
  if (writes) await assertWrite(page, state, pair, pair.initial, writes)
  for (const [key, value] of [['ArrowRight', pair.initial + pair.step], ['ArrowDown', pair.initial], ['ArrowUp', pair.initial + pair.step],
    ['ArrowLeft', pair.initial], ['Home', pair.min], ['End', pair.max]] as const) {
    await expect(control.slider).toBeEnabled(); await control.slider.focus(); await page.keyboard.press(key)
    await expect(control.slider).toHaveValue(String(value)); await expect(control.input).toHaveValue(String(value))
    if (pair.family === 'risk') expect((await state.snapshot()).mockWrites).toHaveLength(0)
    else { writes++; await assertWrite(page, state, pair, value, writes) }
  }
  if (pair.family === 'risk') { await riskSave(page).click(); await assertWrite(page, state, pair, pair.max, 1) }
  state.notes.push({ stage: 'native-MUI-slider-keys', family: pair.family, keys: ['ArrowRight', 'ArrowDown', 'ArrowUp', 'ArrowLeft', 'Home', 'End'],
    initialTabBlurSaved: pair.family !== 'risk', mockWrites: (await state.snapshot()).mockWrites.length })
})

test('P2-KBD-04 log Slider pointer release remains one exact mock settings commit', async ({ page }) => {
  const state = await setup(page), pair = settingsPairs[2], control = namedSettingsControl(page, pair)
  const root = control.slider.locator('xpath=ancestor::*[contains(@class,"MuiSlider-root")][1]')
  const bounds = await root.boundingBox(); if (!bounds) throw new Error('No slider pointer bounds')
  await root.click({ position: { x: bounds.width * .5, y: bounds.height * .5 } })
  const value = Number(await control.slider.inputValue())
  expect(value).toBeGreaterThan(pair.initial); expect(value % pair.step).toBe(0)
  await assertWrite(page, state, pair, value, 1)
  expect((await state.snapshot()).mockWrites).toHaveLength(1)
})

for (const pair of [settingsPairs[0], settingsPairs[1], settingsPairs[3], settingsPairs[6]]) test(`P2-KBD-04 ${pair.family} held save disables its family once failure preserves draft and explicit retry recovers`, async ({ page }) => {
  const state = await setup(page), control = namedSettingsControl(page, pair)
  await state.configure(pair.command, { hold: true, failures: 1 })
  await control.input.fill(pair.draft); await control.input.press('Enter')
  if (pair.family === 'risk') { expect((await state.snapshot()).mockWrites).toHaveLength(0); await riskSave(page).click() }
  const first = await waitSettingsWrite(state, pair.command, 1, 'pending')
  expect(first.args).toEqual(expectedSettingsArgs(pair, pair.rounded))
  for (const sibling of settingsPairs.filter(item => item.family === pair.family)) {
    const locked = namedSettingsControl(page, sibling)
    await expect(locked.input).toBeDisabled(); await expect(locked.slider).toBeDisabled()
  }
  if (pair.family === 'risk') await expect(riskSave(page)).toBeDisabled()
  await page.keyboard.press('Enter'); await page.keyboard.press('Enter'); await settingsTick(page)
  expect((await state.snapshot()).mockWrites).toHaveLength(1)
  await state.release(pair.command); await waitSettingsWrite(state, pair.command, 1, 'failure'); await settingsTick(page)
  await expect(control.input).toBeEnabled(); await expect(control.input).toHaveValue(String(pair.rounded))
  if (pair.family === 'risk') { await expect(riskSave(page)).toBeEnabled(); await riskSave(page).click() }
  else { await control.input.focus(); await control.input.press('Enter') }
  const retry = await assertWrite(page, state, pair, pair.rounded, 2)
  expect(retry.args).toEqual(first.args)
  expect((await state.snapshot()).intentionalFailures).toHaveLength(1)
  if (pair.family === 'risk') await expect(riskSave(page)).toBeDisabled()
  // SPA revisit exercises successful query-cache values, not persisted OS/file settings.
  await page.getByRole('button', { name: 'History', exact: true }).click()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(namedSettingsControl(page, pair).input).toHaveValue(String(pair.rounded))
  state.notes.push({ stage: 'intentional-native-mock-500-and-explicit-retry', family: pair.family, failed: first.id, success: retry.id,
    actualNativeWrites: 0, mockWrites: 2, noNewErrorUIRequirement: true })
})

test('P2-KBD-04 risk disabled GET hides its six pairs without toggling or saving', async ({ page }) => {
  const state = await setup(page, { riskEnabled: false, open: false })
  await expect.poll(async () => (await state.snapshot()).calls.filter(call => call.command === 'get_risk_config' && call.status === 'success').length).toBe(1)
  const region = page.getByRole('region', { name: '리스크 관리', exact: true })
  await expect(region.getByRole('slider')).toHaveCount(0)
  for (const pair of settingsPairs.slice(5)) await expect(region.getByRole('textbox', { name: pair.label, exact: true })).toHaveCount(0)
  expect((await state.snapshot()).mockWrites).toHaveLength(0)
})

test('P2-KBD-04 volatility disabled GET locks only its two risk pairs', async ({ page }) => {
  const state = await setup(page, { volatility: false, narrow: true })
  for (const [index, pair] of settingsPairs.entries()) if (pair.family === 'risk') {
    const control = namedSettingsControl(page, pair)
    if (index >= 9) { await expect(control.input).toBeDisabled(); await expect(control.slider).toBeDisabled() }
    else { await expect(control.input).toBeEnabled(); await expect(control.slider).toBeEnabled() }
  }
  expect((await state.snapshot()).mockWrites).toHaveLength(0); await expect(riskSave(page)).toBeDisabled()
})

test('P2-KBD-04 held risk read and intentional read500 retain heading then existing poll recovers', async ({ page }) => {
  const state = await setup(page, { holdRisk: true, riskFailures: 3, open: false })
  const region = page.getByRole('region', { name: '리스크 관리', exact: true })
  await expect(region).toContainText('리스크 설정을 불러오는 중')
  await expect(region.getByRole('slider')).toHaveCount(0)
  await state.release('get_risk_config')
  const reads = async () => (await state.snapshot()).calls.filter(call => call.command === 'get_risk_config')
  await expect.poll(async () => (await reads())[0]?.status).toBe('failure')
  await settingsTick(page, 1000); await expect.poll(async () => (await reads())[1]?.status).toBe('failure')
  await settingsTick(page, 2000); await expect.poll(async () => (await reads())[2]?.status).toBe('failure')
  await expect(region).toContainText('리스크 설정을 불러오지 못했습니다')
  await expect(region.getByRole('slider')).toHaveCount(0)
  await state.configure('get_risk_config', { failures: 0 }); await settingsTick(page, 10000)
  await expect(region.getByRole('slider')).toHaveCount(6)
  expect((await state.snapshot()).intentionalFailures).toHaveLength(3)
  expect((await state.snapshot()).mockWrites).toHaveLength(0)
})

test('P2-KBD-04 risk dirty draft survives synthetic server poll and saves its existing conversion only on request', async ({ page }) => {
  const state = await setup(page), pair = settingsPairs[6], control = namedSettingsControl(page, pair)
  await page.clock.pauseAt(await page.evaluate(() => Date.now() + 1000))
  await control.input.fill(pair.draft)
  const before = (await state.snapshot()).calls.filter(call => call.command === 'get_risk_config').length
  await state.setRisk({ maxPositionRatio: .7 }); await settingsTick(page, 10000)
  await expect.poll(async () => (await state.snapshot()).calls.filter(call => call.command === 'get_risk_config').length).toBeGreaterThan(before)
  await settingsTick(page)
  await expect(control.input).toHaveValue(pair.draft)
  expect((await state.snapshot()).mockWrites).toHaveLength(0)
  await control.input.press('Enter'); await expect(control.input).toHaveValue(String(pair.rounded))
  await riskSave(page).click(); await assertWrite(page, state, pair, pair.rounded, 1)
  await expect(riskSave(page)).toBeDisabled()
})
