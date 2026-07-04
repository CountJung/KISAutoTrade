import type { BrokerId } from '../../../api/types'

export function cmdErrMsg(e: unknown): string {
  if (e && typeof e === 'object' && 'message' in e) {
    return String((e as { message: unknown }).message)
  }
  return String(e)
}

export function brokerLabel(brokerId: BrokerId | null | undefined): string {
  switch (brokerId) {
    case 'toss':
      return '토스증권'
    case 'kis':
    default:
      return '한국투자증권'
  }
}
