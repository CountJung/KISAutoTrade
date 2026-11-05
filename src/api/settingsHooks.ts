import { useMutation, useQuery, useQueryClient, type UseQueryOptions } from '@tanstack/react-query'

import * as cmd from './commands'
import { KEYS } from './queryKeys'
import type {
  BrokerRateLimitScopeView,
  UpdateInfo,
  WebConfig,
  SaveWebConfigInput,
  RefreshConfig,
} from './types'

// ─── Discord 테스트 ────────────────────────────────────────────────
export function useSendTestDiscord() {
  return useMutation({
    mutationFn: cmd.sendTestDiscord,
  })
}

// ─── 업데이트 확인 ────────────────────────────────────────────────
export function useUpdateCheck() {
  return useQuery<UpdateInfo>({
    queryKey: KEYS.updateCheck,
    queryFn: cmd.checkForUpdate,
    // 앱 시작 시 한 번만 확인. 성공하면 24시간 동안 재요청 안 함
    staleTime: 1000 * 60 * 60 * 24,
    retry: 1,
    // 실패해도 UI 먹스에 영향 없음
    throwOnError: false,
  })
}

// ─── 웹 접속 설정 ─────────────────────────────────────────────────
export function useWebConfig() {
  return useQuery<WebConfig>({
    queryKey: KEYS.webConfig,
    queryFn: cmd.getWebConfig,
    staleTime: Infinity,
  })
}

export function useSaveWebConfig() {
  const qc = useQueryClient()
  return useMutation<string, Error, SaveWebConfigInput>({
    mutationFn: cmd.saveWebConfig,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: KEYS.webConfig })
    },
  })
}

// ─── 환율 / 공통 갱신 주기 ──────────────────────────────────────
/**
 * REFRESH_INTERVAL_SEC를 Rust에서 읽어 반환합니다.
 * setRefreshConfig 호출 시 캐시가 즉시 갱신됩니다.
 */
export function useRefreshInterval() {
  return useQuery<number>({
    queryKey: KEYS.refreshInterval,
    queryFn: cmd.getRefreshInterval,
    staleTime: 30_000,  // 30초 후 stale (setRefreshConfig 시 queryData로 즉시 동기화)
    placeholderData: 30,
  })
}

/**
 * 데이터 갱신 주기 설정 조회
 */
export function useRefreshConfig() {
  return useQuery<RefreshConfig>({
    queryKey: KEYS.refreshConfig,
    queryFn: cmd.getRefreshConfig,
    staleTime: Infinity,
    placeholderData: { interval_sec: 30 },
  })
}

/**
 * 데이터 갱신 주기 변경 — JSON 저장 + 백그라운드 데몬 즉시 적용
 */
export function useSetRefreshConfig() {
  const qc = useQueryClient()
  return useMutation<RefreshConfig, Error, number>({
    mutationFn: (interval_sec) => cmd.setRefreshConfig(interval_sec),
    onSuccess: (data) => {
      qc.setQueryData(KEYS.refreshConfig, data)
      qc.setQueryData(KEYS.refreshInterval, data.interval_sec)
    },
  })
}

/** broker rate limit 운영 상태 (429 pause, 마지막 성공/실패, 연속 실패) */
export function useBrokerRateLimitStatus(
  options?: Partial<UseQueryOptions<BrokerRateLimitScopeView[]>>
) {
  return useQuery<BrokerRateLimitScopeView[]>({
    queryKey: KEYS.brokerRateLimitStatus,
    queryFn: cmd.getBrokerRateLimitStatus,
    refetchInterval: 10_000,
    staleTime: 5_000,
    ...options,
  })
}
