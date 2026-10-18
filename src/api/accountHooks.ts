import { useMutation, useQuery, useQueryClient, type UseQueryOptions } from '@tanstack/react-query'

import { POLL_INTERVALS } from '../scheduler'
import * as cmd from './commands'
import { KEYS } from './queryKeys'
import type {
  AccountProfileView,
  AddProfileInput,
  AppConfigView,
  BrokerHoldingView,
  BalanceResult,
  OverseasBalanceResult,
  ConfigDiagnostic,
  UpdateProfileInput,
  DetectTradingTypeResult,
  TossConnectionDiagnostic,
  TossAccountLookupInput,
  TossAccountOptionView,
} from './types'

// ─── 앱 설정 ───────────────────────────────────────────────────────
export function useAppConfig(
  options?: Partial<UseQueryOptions<AppConfigView>>
) {
  return useQuery({
    queryKey: KEYS.appConfig,
    queryFn: cmd.getAppConfig,
    staleTime: Infinity,
    ...options,
  })
}

// ─── 잔고 ──────────────────────────────────────────────────────────
export function useBalance(
  options?: Partial<UseQueryOptions<BalanceResult>>
) {
  return useQuery({
    queryKey: KEYS.balance,
    queryFn: cmd.getBalance,
    staleTime: 30_000,
    refetchInterval: POLL_INTERVALS.SLOW, // 60s — 스케쥴러 기준
    ...options,
  })
}

export function useOverseasBalance(
  options?: Partial<UseQueryOptions<OverseasBalanceResult>>
) {
  return useQuery({
    queryKey: KEYS.overseasBalance,
    queryFn: cmd.getOverseasBalance,
    staleTime: 30_000,
    refetchInterval: POLL_INTERVALS.SLOW, // 60s
    ...options,
  })
}

export function useBrokerHoldings(
  options?: Partial<UseQueryOptions<BrokerHoldingView[]>>
) {
  return useQuery({
    queryKey: KEYS.brokerHoldings,
    queryFn: cmd.getBrokerHoldings,
    staleTime: 30_000,
    refetchInterval: POLL_INTERVALS.SLOW,
    ...options,
  })
}

// ─── 진단 모드 ───────────────────────────────────────────────
export function useCheckConfig(
  options?: Partial<UseQueryOptions<ConfigDiagnostic>>
) {
  return useQuery({
    queryKey: KEYS.checkConfig,
    queryFn: cmd.checkConfig,
    staleTime: 30_000,
    ...options,
  })
}

// ─── 계좌 프로파일 ──────────────────────────────────────────────────
export function useProfiles(
  options?: Partial<UseQueryOptions<AccountProfileView[]>>
) {
  return useQuery({
    queryKey: KEYS.profiles,
    queryFn: cmd.listProfiles,
    staleTime: Infinity,
    ...options,
  })
}

export function useAddProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: AddProfileInput) => cmd.addProfile(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.appConfig })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
      void qc.invalidateQueries({ queryKey: ['strategies'] })
    },
  })
}

export function useUpdateProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: UpdateProfileInput) => cmd.updateProfile(input),
    onSuccess: (updatedProfile) => {
      qc.setQueryData<AccountProfileView[]>(KEYS.profiles, (old) =>
        old?.map((profile) =>
          profile.id === updatedProfile.id ? updatedProfile : profile
        ) ?? [updatedProfile],
      )
      if (updatedProfile.is_active) {
        qc.setQueryData<AppConfigView>(KEYS.appConfig, (old) =>
          old
            ? {
                ...old,
                active_broker_id: updatedProfile.broker_id,
                active_broker_account_id: updatedProfile.broker_account_id,
                kis_is_paper_trading: updatedProfile.is_paper_trading,
              }
            : old,
        )
      }
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.appConfig })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
      void qc.invalidateQueries({ queryKey: ['strategies'] })
    },
  })
}

export function useDeleteProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => cmd.deleteProfile(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.appConfig })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
      void qc.invalidateQueries({ queryKey: ['strategies'] })
    },
  })
}

export function useSetActiveProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => cmd.setActiveProfile(id),
    onSuccess: (newConfig) => {
      qc.setQueryData(KEYS.appConfig, newConfig)
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
      void qc.invalidateQueries({ queryKey: ['strategies'] })
      // 프로파일 전환 시 잔고 캠시 무효화
      void qc.invalidateQueries({ queryKey: KEYS.balance })
    },
  })
}

/** APP KEY + APP SECRET으로 실전/모의투자를 자동 감지합니다. */
export function useDetectTradingType() {
  return useMutation<DetectTradingTypeResult, Error, { appKey: string; appSecret: string }>({
    mutationFn: ({ appKey, appSecret }) => cmd.detectTradingType(appKey, appSecret),
  })
}

/** 저장된 프로파일 키로 실전/모의 감지 후 즉시 저장합니다. */
export function useDetectProfileTradingType() {
  const qc = useQueryClient()
  return useMutation<AccountProfileView, Error, string>({
    mutationFn: (profileId) => cmd.detectProfileTradingType(profileId),
    onSuccess: (updatedProfile) => {
      qc.setQueryData<AccountProfileView[]>(KEYS.profiles, (old) =>
        old?.map((profile) =>
          profile.id === updatedProfile.id ? updatedProfile : profile
        ) ?? [updatedProfile],
      )
      if (updatedProfile.is_active) {
        qc.setQueryData<AppConfigView>(KEYS.appConfig, (old) =>
          old
            ? {
                ...old,
                active_broker_id: updatedProfile.broker_id,
                active_broker_account_id: updatedProfile.broker_account_id,
                kis_is_paper_trading: updatedProfile.is_paper_trading,
              }
            : old,
        )
      }
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.appConfig })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
    },
  })
}

/** 입력한 토스증권 Client ID/Secret으로 accountSeq 목록을 조회합니다. */
export function useListTossAccounts() {
  return useMutation<TossAccountOptionView[], Error, TossAccountLookupInput>({
    mutationFn: (input) => cmd.listTossAccounts(input),
  })
}

/** 저장된 토스증권 프로파일로 accountSeq 목록을 조회합니다. */
export function useListTossProfileAccounts() {
  return useMutation<TossAccountOptionView[], Error, string>({
    mutationFn: (profileId) => cmd.listTossProfileAccounts(profileId),
  })
}

/** 저장된 토스 프로파일로 OpenAPI/token/accounts/holdings 연결을 진단합니다. */
export function useCheckTossProfileConnection() {
  return useMutation<TossConnectionDiagnostic, Error, string>({
    mutationFn: (profileId) => cmd.checkTossProfileConnection(profileId),
  })
}
