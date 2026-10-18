import { useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'

import type { ExchangeRateView } from './types'
import { KEYS } from './queryKeys'

function canUseTauriEvents(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/**
 * 백그라운드 데몬 이벤트 수신
 *
 * 환율은 전역 캐시를 갱신하고, 계좌 잔고는 현재 계좌의 쿼리를 무효화합니다.
 * 지연된 이전 계좌 이벤트의 payload를 현재 계좌 캐시에 대입하지 않습니다.
 * App 컴포넌트 루트에서 1회 호출하세요:
 *   function App() { useBackendEvents(); ... }
 */
export function useBackendEvents() {
  const qc = useQueryClient()

  useEffect(() => {
    if (!canUseTauriEvents()) {
      return
    }

    let active = true
    const unlisteners: (() => void)[] = []
    const release = (unlisten: () => void) => {
      void Promise.resolve().then(unlisten).catch((error) => {
        console.warn('Backend event listener cleanup failed', error)
      })
    }
    const register = (registration: Promise<() => void>) => {
      void registration.then((unlisten) => {
        if (active) unlisteners.push(unlisten)
        else release(unlisten)
      }).catch((error) => {
        console.warn('Backend event listener registration failed', error)
      })
    }

    void import('@tauri-apps/api/event').then(({ listen }) => {
      if (!active) {
        return
      }

      // 환율 갱신 이벤트
      register(
        listen<number>('exchange-rate-updated', (event) => {
          if (!active) return
          qc.setQueryData(KEYS.exchangeRate, event.payload)
        })
      )
      register(
        listen<ExchangeRateView>('exchange-rate-status-updated', (event) => {
          if (!active) return
          qc.setQueryData(KEYS.exchangeRateStatus, event.payload)
          qc.setQueryData(KEYS.exchangeRate, event.payload.rate)
        })
      )

      // 국내 잔고 갱신 이벤트
      register(
        listen<unknown>('balance-updated', () => {
          if (!active) return
          void qc.invalidateQueries({ queryKey: KEYS.balance })
        })
      )

      // 해외 잔고 갱신 이벤트
      register(
        listen<unknown>('overseas-balance-updated', () => {
          if (!active) return
          void qc.invalidateQueries({ queryKey: KEYS.overseasBalance })
        })
      )
    }).catch((error) => {
      console.warn('Backend event module could not be loaded', error)
    })

    return () => {
      active = false
      unlisteners.forEach(release)
    }
  }, [qc])
}
