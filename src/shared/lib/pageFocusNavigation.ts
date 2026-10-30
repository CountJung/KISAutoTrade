// Explicit, user-triggered in-page focus navigation supersedes only its own route restore.
export const PAGE_FOCUS_NAVIGATION_EVENT = 'act:page-focus-navigation'

export function notifyPageFocusNavigation(routeKey: string) {
  window.dispatchEvent(new CustomEvent(PAGE_FOCUS_NAVIGATION_EVENT, { detail: { routeKey } }))
}
