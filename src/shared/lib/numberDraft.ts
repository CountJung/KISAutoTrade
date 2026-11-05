/** Parse decimal keyboard input without coercing incomplete or non-finite drafts. */
export function parseDecimalDraft(raw: string): number | string {
  if (raw.trim() === '' || !/^[+-]?(?:\d+\.?\d*|\.\d+)$/.test(raw)) return raw
  const value = Number(raw)
  return Number.isFinite(value) ? value : raw
}

/** Strict numeric bounds without coercion; callers decide optional/default policy. */
export function isFiniteNumberInRange(value: unknown, bounds: { min: number; max: number; integer?: boolean }): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= bounds.min && value <= bounds.max && (!bounds.integer || Number.isInteger(value))
}
