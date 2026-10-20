/** Parse decimal keyboard input without coercing incomplete or non-finite drafts. */
export function parseDecimalDraft(raw: string): number | string {
  if (raw.trim() === '' || !/^[+-]?(?:\d+\.?\d*|\.\d+)$/.test(raw)) return raw
  const value = Number(raw)
  return Number.isFinite(value) ? value : raw
}
