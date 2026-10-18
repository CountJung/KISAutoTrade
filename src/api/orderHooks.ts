import { useMutation, useQuery, useQueryClient, type UseQueryOptions } from '@tanstack/react-query'

import { POLL_INTERVALS, ORDER_REFETCH_DELAY_MS } from '../scheduler'
import * as cmd from './commands'
import { KEYS } from './queryKeys'
import type {
  PlaceOrderInput,
  PlaceOverseasOrderInput,
  PendingOrderView,
  CmdError,
  TossModifyOrderInput,
  TossOpenOrderView,
  TossOrderPreflightInput,
  TossOrderPreflightView,
  TossSmallBuyVerificationInput,
  TossSmallBuyVerificationView,
} from './types'

// ─── 주문 ──────────────────────────────────────────────────────────
export function usePlaceOrder() {
  const qc = useQueryClient()

  return useMutation({
    mutationFn: (input: PlaceOrderInput) => cmd.placeOrder(input),
    onSuccess: () => {
      // 잔고·로컬 체결 기록은 즉시 무효화
      void qc.invalidateQueries({ queryKey: KEYS.balance })
      void qc.invalidateQueries({ queryKey: KEYS.todayTrades })
      void qc.invalidateQueries({ queryKey: ['tossOpenOrders'] })
      void qc.invalidateQueries({ queryKey: KEYS.pendingOrders })
      // KIS 서버 처리 딜레이 후 체결 내역 갱신 (즉시 재조회 시 미반영 가능)
      setTimeout(
        () => void qc.invalidateQueries({ queryKey: KEYS.todayExecuted }),
        ORDER_REFETCH_DELAY_MS.REAL,
      )
    },
  })
}

export function useTossOrderPreflight(
  input: TossOrderPreflightInput,
  options?: Partial<UseQueryOptions<TossOrderPreflightView>>
) {
  return useQuery({
    queryKey: KEYS.tossOrderPreflight(input),
    queryFn: () => cmd.checkTossOrderPreflight(input),
    enabled: !!input.symbol && !!input.quantity,
    staleTime: 5_000,
    gcTime: 60_000,
    retry: false,
    ...options,
  })
}

export function useTossOpenOrders(
  symbol?: string | null,
  options?: Partial<UseQueryOptions<TossOpenOrderView[]>>
) {
  return useQuery({
    queryKey: KEYS.tossOpenOrders(symbol),
    queryFn: () => cmd.listTossOpenOrders({ symbol: symbol || null }),
    staleTime: 5_000,
    refetchInterval: POLL_INTERVALS.NORMAL,
    retry: false,
    ...options,
  })
}

export function useModifyTossOrder() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: TossModifyOrderInput) => cmd.modifyTossOrder(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['tossOpenOrders'] })
      void qc.invalidateQueries({ queryKey: KEYS.pendingOrders })
      void qc.invalidateQueries({ queryKey: KEYS.todayTrades })
    },
  })
}

export function useSubmitTossSmallBuyVerification() {
  const qc = useQueryClient()
  return useMutation<TossSmallBuyVerificationView, CmdError, TossSmallBuyVerificationInput>({
    mutationFn: (input) => cmd.submitTossSmallBuyVerification(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.brokerHoldings })
      void qc.invalidateQueries({ queryKey: KEYS.pendingOrders })
      void qc.invalidateQueries({ queryKey: KEYS.todayTrades })
    },
  })
}

// ─── 해외 주문 (지정가 한정) ────────────────────────────────────────
/** 해외 주문 — 지정가만 지원, USD 단위 가격 입력 */
export function usePlaceOverseasOrder() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: PlaceOverseasOrderInput) => cmd.placeOverseasOrder(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.balance })
      // 해외 주문 체결 후 해외 잔고도 갱신
      void qc.invalidateQueries({ queryKey: KEYS.overseasBalance })
      // KIS 서버 처리 딜레이 후 체결 내역 갱신
      setTimeout(
        () => {
          void qc.invalidateQueries({ queryKey: KEYS.todayExecuted })
          void qc.invalidateQueries({ queryKey: KEYS.todayOverseasExecuted })
        },
        ORDER_REFETCH_DELAY_MS.REAL,
      )
    },
  })
}

export function usePendingOrders(options?: Partial<UseQueryOptions<PendingOrderView[]>>) {
  return useQuery<PendingOrderView[]>({
    queryKey: KEYS.pendingOrders,
    queryFn: cmd.getPendingOrders,
    staleTime: 5_000,
    refetchInterval: POLL_INTERVALS.PENDING, // 5s — 스케쥴러 기준
    placeholderData: [],
    ...options,
  })
}
