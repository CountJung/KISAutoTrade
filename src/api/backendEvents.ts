import { useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'

import type { BalanceResult, ExchangeRateView, OverseasBalanceResult } from './types'
import { KEYS } from './queryKeys'

function canUseTauriEvents(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/**
 * 백그라운드 데몬 이벤트 수신
 *
 * Rust 백그라운드에서 발행하는 이벤트를 리슨하여 TanStack Query 캐시를 직접 갱신합니다.
 * 프론트엔드 폴링 없이 데이터를 즉시 수신하여 UI를 업데이트합니다.
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
    const unlisteners: Promise<() => void>[] = []

    void import('@tauri-apps/api/event').then(({ listen }) => {
      if (!active) {
        return
      }

      // 환율 갱신 이벤트
      unlisteners.push(
        listen<number>('exchange-rate-updated', (event) => {
          qc.setQueryData(KEYS.exchangeRate, event.payload)
        })
      )
      unlisteners.push(
        listen<ExchangeRateView>('exchange-rate-status-updated', (event) => {
          qc.setQueryData(KEYS.exchangeRateStatus, event.payload)
          qc.setQueryData(KEYS.exchangeRate, event.payload.rate)
        })
      )

      // 국내 잔고 갱신 이벤트
      unlisteners.push(
        listen<BalanceResult>('balance-updated', (event) => {
          qc.setQueryData(KEYS.balance, event.payload)
        })
      )

      // 해외 잔고 갱신 이벤트
      unlisteners.push(
        listen<OverseasBalanceResult>('overseas-balance-updated', (event) => {
          qc.setQueryData(KEYS.overseasBalance, event.payload)
        })
      )
    })

    return () => {
      active = false
      unlisteners.forEach((p) => p.then((fn) => fn()))
    }
  }, [qc])
}
