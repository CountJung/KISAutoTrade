export function clampNumber(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}

export function readStoredNumber(
  key: string,
  defaultValue: number,
  min: number,
  max: number,
): number {
  if (typeof window === 'undefined') return defaultValue

  const raw = window.localStorage.getItem(key)
  if (!raw) return defaultValue

  const value = Number(raw)
  return Number.isFinite(value) ? clampNumber(value, min, max) : defaultValue
}

export function writeStoredNumber(
  key: string,
  value: number,
  min: number,
  max: number,
): void {
  if (typeof window === 'undefined') return
  if (!Number.isFinite(value)) return

  window.localStorage.setItem(key, String(clampNumber(value, min, max)))
}
