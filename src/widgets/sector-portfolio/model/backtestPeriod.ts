/** Calendar presets use New York dates, not the device's zone or a guessed trading calendar. */
export type BacktestPeriod = 'sessions' | 'lastWeek' | 'month' | 'custom'
export function validCalendarDate(value: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(value) && Number.isFinite(Date.parse(`${value}T00:00:00Z`))
    && new Date(`${value}T00:00:00Z`).toISOString().slice(0, 10) === value
}
function date(value: Date) { return value.toISOString().slice(0, 10) }
export function backtestPreset(period: 'lastWeek' | 'month', now = new Date()): { startDate: string; endDate: string } {
  const parts = new Intl.DateTimeFormat('en-US', { timeZone: 'America/New_York', year: 'numeric', month: '2-digit', day: '2-digit' }).formatToParts(now)
  const part = (name: string) => parts.find(item => item.type === name)!.value
  const today = new Date(`${part('year')}-${part('month')}-${part('day')}T00:00:00Z`)
  if (period === 'lastWeek') {
    const monday = new Date(today)
    monday.setUTCDate(today.getUTCDate() - ((today.getUTCDay() + 6) % 7))
    const end = new Date(monday); end.setUTCDate(end.getUTCDate() - 1)
    const start = new Date(monday); start.setUTCDate(start.getUTCDate() - 7)
    return { startDate: date(start), endDate: date(end) }
  }
  const end = new Date(today); end.setUTCDate(end.getUTCDate() - 1)
  const previousMonthEnd = new Date(Date.UTC(end.getUTCFullYear(), end.getUTCMonth(), 0))
  const start = new Date(Date.UTC(previousMonthEnd.getUTCFullYear(), previousMonthEnd.getUTCMonth(), Math.min(end.getUTCDate(), previousMonthEnd.getUTCDate()) + 1))
  return { startDate: date(start), endDate: date(end) }
}
export function backtestPeriodError(period: BacktestPeriod, start: string, end: string): string | null {
  if (end && !validCalendarDate(end)) return '종료일을 실제 YYYY-MM-DD 날짜로 입력하세요.'
  if (period !== 'sessions') {
    if (!validCalendarDate(start) || !validCalendarDate(end)) return '시작일과 종료일을 실제 YYYY-MM-DD 날짜로 입력하세요.'
    if (start > end) return '시작일은 종료일보다 늦을 수 없습니다.'
    if ((Date.parse(end) - Date.parse(start)) / 86_400_000 + 1 > 180) return '직접 지정 기간은 최대 180일입니다.'
  }
  return null
}
