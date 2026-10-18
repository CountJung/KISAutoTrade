import { useMutation, useQuery, useQueryClient, type UseQueryOptions } from '@tanstack/react-query'

import { POLL_INTERVALS } from '../scheduler'
import * as cmd from './commands'
import { KEYS } from './queryKeys'
import type {
  FrontendLogInput,
  LogConfig,
  SetLogConfigInput,
  AppLogEntry,
} from './types'

// ─── 로그 설정 ─────────────────────────────────────────────────────
export function useLogConfig(
  options?: Partial<UseQueryOptions<LogConfig>>
) {
  return useQuery({
    queryKey: KEYS.logConfig,
    queryFn: cmd.getLogConfig,
    staleTime: Infinity,
    ...options,
  })
}

export function useSetLogConfig() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: SetLogConfigInput) => cmd.setLogConfig(input),
    onSuccess: (newCfg) => {
      qc.setQueryData(KEYS.logConfig, newCfg)
    },
  })
}

// ─── 프론트엔드 로그 기록 ─────────────────────────────────────────
export function useWriteFrontendLog() {
  return useMutation({
    mutationFn: (input: FrontendLogInput) => cmd.writeFrontendLog(input),
  })
}

// ─── 최근 앱 로그 (파일 기반) ─────────────────────────────────────
export function useRecentLogs(count = 200) {
  return useQuery<AppLogEntry[]>({
    queryKey: KEYS.recentLogs(count),
    queryFn: () => cmd.getRecentLogs(count),
    refetchInterval: POLL_INTERVALS.LOG,
    staleTime: 0,
    placeholderData: [],
  })
}

/**
 * 프론트엔드 로거 — 일반 함수 형태 (훅 외부에서 직접 호출 가능)
 * Tauri 환경에서만 동작하며, 실패해도 콘솔만 출력하고 에러 전파 안 함
 */
export const frontendLogger = {
  error: (message: string, context?: string) => {
    console.error(`[${context ?? 'frontend'}] ${message}`)
    cmd.writeFrontendLog({ level: 'error', message, context }).catch(() => {})
  },
  warn: (message: string, context?: string) => {
    console.warn(`[${context ?? 'frontend'}] ${message}`)
    cmd.writeFrontendLog({ level: 'warn', message, context }).catch(() => {})
  },
  info: (message: string, context?: string) => {
    console.info(`[${context ?? 'frontend'}] ${message}`)
    cmd.writeFrontendLog({ level: 'info', message, context }).catch(() => {})
  },
  debug: (message: string, context?: string) => {
    console.debug(`[${context ?? 'frontend'}] ${message}`)
    cmd.writeFrontendLog({ level: 'debug', message, context }).catch(() => {})
  },
}
