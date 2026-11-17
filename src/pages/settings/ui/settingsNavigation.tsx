import { useEffect } from 'react'
import { notifyPageFocusNavigation, PAGE_FOCUS_NAVIGATION_EVENT } from '../../../shared/lib'

import Link from '@mui/material/Link'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

export const settingsSections = [
  ['settings-theme', '테마'],
  ['settings-refresh', '데이터 갱신 주기'],
  ['settings-logs', '로그 설정'],
  ['settings-archive', '체결 기록 보관'],
  ['settings-database', '데이터베이스 및 데이터 이관'],
  ['settings-rate-limit', 'Broker 요청 상태 (rate limit)'],
  ['settings-active-profile', '활성 증권사 프로파일'],
  ['settings-kis-profiles', '한국투자증권 계좌 프로파일'],
  ['settings-toss-profiles', '토스증권 계좌 프로파일'],
  ['settings-web', '웹 접속 설정'],
  ['settings-stocks', '종목 목록 관리'],
  ['settings-budget', '자동매매 전용 예산'],
  ['settings-recovery', '응답 불명 주문 대조·복구'],
  ['settings-risk', '리스크 관리'],
  ['settings-discord', 'Discord 알림'],
] as const

export type SettingsSectionId = typeof settingsSections[number][0]
export const settingsTocId = 'settings-contents'
export const settingsHeadingFocus = {
  scrollMarginBlock: 16,
  '&:focus-visible': { outline: '2px solid', outlineColor: 'primary.main', outlineOffset: 4 },
}

let cancelSettingsVisibility: (() => void) | undefined

// Explicit focus/scroll without a URL hash or history entry. AppShell owns route restoration.
export function focusSettingsTarget(id: string) {
  cancelSettingsVisibility?.()
  const target = document.getElementById(id)
  if (!target) return
  notifyPageFocusNavigation(window.location.pathname)
  target.focus({ preventScroll: true })
  const main = target.closest<HTMLElement>('[data-testid="app-main-scroll"]')
  if (!main) {
    target.scrollIntoView({ block: 'start', inline: 'nearest' })
    return
  }
  // scrollIntoView also moves overflow:hidden shell ancestors when main is clamped.
  const margin = Number.parseFloat(getComputedStyle(target).scrollMarginTop) || 0
  const offset = target.getBoundingClientRect().top - main.getBoundingClientRect().top
  main.scrollTop += offset - main.clientTop - margin
  // Keep only this explicit destination visible across late query/layout changes.
  // No polling: one frame per observed change, and never refocus the heading.
  const route = window.location.pathname
  let active = true
  let frame: number | undefined
  const cancel = () => {
    if (!active) return
    active = false
    if (frame !== undefined) cancelAnimationFrame(frame)
    resize.disconnect()
    mutations.disconnect()
    for (const type of ['wheel', 'touchstart', 'pointerdown', 'keydown', 'focusin', 'popstate', PAGE_FOCUS_NAVIGATION_EVENT]) {
      window.removeEventListener(type, intent, true)
    }
    if (cancelSettingsVisibility === cancel) cancelSettingsVisibility = undefined
  }
  const intent = (event: Event) => {
    if (event.type === 'focusin' && event.target === target) return
    if (event instanceof KeyboardEvent && !['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'PageUp', 'PageDown', 'Home', 'End', ' ', 'Tab', 'Enter'].includes(event.key)) return
    cancel()
  }
  const schedule = () => {
    if (!active || frame !== undefined) return
    frame = requestAnimationFrame(() => {
      frame = undefined
      if (!target.isConnected || !main.isConnected || document.activeElement !== target || window.location.pathname !== route) {
        cancel()
        return
      }
      const rect = target.getBoundingClientRect()
      const top = main.getBoundingClientRect().top + main.clientTop
      if (rect.top < top || rect.bottom > top + main.clientHeight) {
        const currentMargin = Number.parseFloat(getComputedStyle(target).scrollMarginTop) || 0
        main.scrollTop += rect.top - top - currentMargin
      }
    })
  }
  const resize = new ResizeObserver(schedule)
  const observeContent = () => {
    if (!target.isConnected || !main.isConnected) { cancel(); return }
    resize.disconnect()
    resize.observe(main)
    Array.from(main.children).forEach(child => resize.observe(child))
    schedule()
  }
  const mutations = new MutationObserver(observeContent)
  cancelSettingsVisibility = cancel
  // Capture has already passed for the Enter/click that initiated this request.
  for (const type of ['wheel', 'touchstart', 'pointerdown', 'keydown', 'focusin', 'popstate', PAGE_FOCUS_NAVIGATION_EVENT]) {
    window.addEventListener(type, intent, { capture: true, passive: true })
  }
  mutations.observe(main, { childList: true, subtree: true, characterData: true })
  observeContent()
}

export function SettingsContentsLink({ title }: { title: string }) {
  return (
    <Link href={`#${settingsTocId}`} aria-label={`${title}에서 설정 목차로`}
      onClick={event => { event.preventDefault(); focusSettingsTarget(settingsTocId) }}
      sx={{ ...settingsHeadingFocus, fontSize: '0.75rem', py: 0.5 }}>
      목차로
    </Link>
  )
}

export function SettingsContents() {
  useEffect(() => () => cancelSettingsVisibility?.(), [])
  return (
    <Paper component="nav" aria-labelledby={settingsTocId} sx={{ p: 3, mb: 2 }}>
      <Typography id={settingsTocId} component="h2" variant="subtitle1" tabIndex={-1}
        sx={{ ...settingsHeadingFocus, fontWeight: 600, mb: 1 }}>
        설정 목차
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
        테마·Discord 활성화는 변경 즉시 반영됩니다. 갱신 주기·로그·체결 기록 보관은 조작 완료 시 저장됩니다.
        리스크 사용 여부·종목 자동 갱신 간격은 변경 즉시 저장됩니다.
        리스크 수치·예산·프로파일·웹 설정은 각 영역의 저장·확인 버튼과 안내를 따르세요.
      </Typography>
      <Stack component="ul" direction="row" useFlexGap sx={{ m: 0, pl: 2, gap: 1, columnGap: 3, flexWrap: 'wrap' }}>
        {settingsSections.map(([id, title]) => (
          <li key={id}>
            <Link href={`#${id}`} onClick={event => { event.preventDefault(); focusSettingsTarget(id) }}
              sx={{ ...settingsHeadingFocus, display: 'inline-block', py: 0.5 }}>
              {title}
            </Link>
          </li>
        ))}
      </Stack>
    </Paper>
  )
}
