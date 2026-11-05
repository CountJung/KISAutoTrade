/** Focus only within an opt-in result boundary; callers own selection and key events. */
export function focusResultButton(boundary: HTMLElement, target: EventTarget | null, key: string): boolean {
  if (!['ArrowUp', 'ArrowDown', 'Home', 'End'].includes(key) || !(target instanceof Element)) return false
  const origin = target.closest<HTMLButtonElement>('button')
  if (!origin || !boundary.contains(origin)) return false
  const buttons = Array.from(boundary.querySelectorAll<HTMLButtonElement>('button:not(:disabled):not([aria-disabled="true"])'))
  const index = buttons.indexOf(origin)
  if (index < 0) return false
  const next = key === 'Home' ? 0 : key === 'End' ? buttons.length - 1
    : key === 'ArrowUp' ? Math.max(0, index - 1) : Math.min(buttons.length - 1, index + 1)
  buttons[next].focus()
  return true
}
