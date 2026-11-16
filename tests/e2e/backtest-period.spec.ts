import { expect, test } from '@playwright/test'
import { backtestPeriodError, backtestPreset, validCalendarDate } from '../../src/widgets/sector-portfolio/model/backtestPeriod'

test('period presets use New York calendar dates through DST, year and short-month boundaries', () => {
  expect(backtestPreset('lastWeek', new Date('2026-11-16T01:00:00Z'))).toEqual({ startDate: '2026-11-02', endDate: '2026-11-08' })
  expect(backtestPreset('lastWeek', new Date('2026-11-16T15:00:00Z'))).toEqual({ startDate: '2026-11-09', endDate: '2026-11-15' })
  expect(backtestPreset('lastWeek', new Date('2026-01-05T15:00:00Z'))).toEqual({ startDate: '2025-12-29', endDate: '2026-01-04' })
  expect(backtestPreset('month', new Date('2026-04-01T15:00:00Z'))).toEqual({ startDate: '2026-03-01', endDate: '2026-03-31' })
  expect(backtestPreset('month', new Date('2024-03-31T15:00:00Z'))).toEqual({ startDate: '2024-03-01', endDate: '2024-03-30' })
})
test('period validation rejects nonexistent, reversed and oversized dates', () => {
  expect(validCalendarDate('2026-02-30')).toBe(false)
  expect(validCalendarDate('2024-02-29')).toBe(true)
  expect(backtestPeriodError('sessions', '', '')).toBeNull()
  expect(backtestPeriodError('custom', '', '')).not.toBeNull()
  expect(backtestPeriodError('custom', '2026-11-10', '2026-11-09')).toContain('늦을')
  expect(backtestPeriodError('custom', '2026-01-01', '2026-11-09')).toContain('180')
  expect(backtestPeriodError('custom', '2026-11-09', '2026-11-09')).toBeNull()
})
