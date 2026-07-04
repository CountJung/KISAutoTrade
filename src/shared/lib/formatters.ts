import type { BrokerMoneyView } from '../api/types'

export function fmtNumber(value: number): string {
  return value.toLocaleString('ko-KR')
}

export function fmtDecimalString(value: string, fractionDigits = 4): string {
  const parsed = Number(value.replace(/,/g, ''))
  if (!Number.isFinite(parsed)) return value
  return parsed.toLocaleString('ko-KR', { maximumFractionDigits: fractionDigits })
}

export function parseDecimalString(value: string | null | undefined): number {
  if (!value) return 0
  const parsed = Number(value.replace(/,/g, ''))
  return Number.isFinite(parsed) ? parsed : 0
}

export function fmtBrokerMoney(money?: BrokerMoneyView | null): string {
  if (!money) return '-'
  return money.currency === 'KRW'
    ? `${fmtDecimalString(money.amount, 0)}원`
    : `$${fmtDecimalString(money.amount, 4)}`
}
