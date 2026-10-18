import { useIsMutating, useMutation, useQuery, useQueryClient, type UseQueryOptions, type QueryClient } from '@tanstack/react-query'

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
const holdingsPrefixes = [KEYS.balance, KEYS.overseasBalance, KEYS.brokerHoldings] as const
const profileMutationKey = ['account-profile-publication'] as const
const profileMutationScope = { id: 'account-profile-publication' }

function trustedAccountConfig(qc: QueryClient) {
  const state = qc.getQueryState<AppConfigView>(KEYS.appConfig)
  return state?.status === 'success' && state.fetchStatus === 'idle' && !state.isInvalidated
    ? state.data : undefined
}

function accountScope(config: AppConfigView | undefined) {
  return [config?.active_profile_id ?? null, config?.active_broker_id ?? null,
    config?.active_broker_account_id ?? null, config?.kis_is_paper_trading ?? null] as const
}

/** Publish the new config before calling this fence. Old query closures reject
 * its scope; new observers cancelled during the transition are refetched below. */
async function resetAccountHoldings(qc: QueryClient) {
  await Promise.all(holdingsPrefixes.map((queryKey) => qc.cancelQueries({ queryKey })))
  const scope = accountScope(trustedAccountConfig(qc))
  for (const prefix of holdingsPrefixes) {
    // Also evict a completed response from the server-switch/UI-response gap, so
    // returning to an earlier account cannot reuse another account's cached data.
    qc.removeQueries({ queryKey: prefix, predicate: (query) =>
      query.queryKey.length !== prefix.length + scope.length ||
      scope.some((value, index) => query.queryKey[prefix.length + index] !== value),
    })
  }
  await Promise.all(holdingsPrefixes.map((prefix) => qc.invalidateQueries({ queryKey: [...prefix, ...scope] })))
}

class AccountConfigRefreshError extends Error {
  constructor(error: unknown) {
    super(error instanceof Error ? error.message : String(error))
    this.name = 'AccountConfigRefreshError'
  }
}

/** Server mutations without an AppConfig response leave the old config untrusted
 * immediately, and propagate a refresh failure instead of accepting stale data. */
async function refreshAccountConfig(qc: QueryClient) {
  const cancelled = qc.cancelQueries({ queryKey: KEYS.appConfig })
  void qc.invalidateQueries({ queryKey: KEYS.appConfig, refetchType: 'none' })
  const holdings = resetAccountHoldings(qc)
  await cancelled
  await holdings
  try {
    await qc.fetchQuery({ queryKey: KEYS.appConfig, queryFn: cmd.getAppConfig, staleTime: Infinity, retry: false })
  } catch (error) {
    // onError must not issue a second refresh after onSuccess already failed here.
    throw new AccountConfigRefreshError(error)
  }
  await resetAccountHoldings(qc)
}

async function reconcileRejectedProfileMutation(qc: QueryClient, error: unknown) {
  if (error instanceof AccountConfigRefreshError) return
  // A rejected publication can have changed the server before persistence failed.
  // refreshAccountConfig invalidates trust and cancels holdings before its first await.
  try {
    await refreshAccountConfig(qc)
  } catch {
    // Keep the original mutation error. The config query retains its own refresh
    // error, which the holdings hooks expose while leaving all holdings hidden.
  }
}

function useScopedAccountQuery<T>(
  prefix: readonly string[],
  fetch: () => Promise<T>,
  options?: Partial<UseQueryOptions<T>>,
) {
  const configQuery = useAppConfig()
  const qc = useQueryClient()
  const mutations = useIsMutating({ mutationKey: profileMutationKey })
  const ready = configQuery.isSuccess && !configQuery.isFetching && !configQuery.isStale && mutations === 0
  const config = ready ? configQuery.data : undefined
  const scope = accountScope(config)
  const scopeMatches = () => {
    const current = trustedAccountConfig(qc)
    return !!config && !!current && qc.isMutating({ mutationKey: profileMutationKey }) === 0 &&
      scope.every((value, index) => value === accountScope(current)[index])
  }
  const result = useQuery({
    staleTime: 30_000,
    refetchInterval: POLL_INTERVALS.SLOW,
    ...options,
    queryKey: [...prefix, ...scope],
    queryFn: async () => {
      if (!scopeMatches()) throw new Error('계좌가 변경되었습니다. 현재 계좌의 잔고를 다시 조회합니다.')
      const result = await fetch()
      if (!scopeMatches()) throw new Error('계좌가 변경되었습니다. 이전 계좌의 잔고 응답을 폐기합니다.')
      return result
    },
    enabled: (query) => ready && (typeof options?.enabled === 'function'
      ? options.enabled(query) : options?.enabled !== false),
    // Neither keepPreviousData nor caller-provided bootstrap data may bridge scopes.
    placeholderData: undefined,
    initialData: undefined,
  })
  if (!ready) {
    return { ...result, data: undefined, error: configQuery.error,
      isError: configQuery.isError, isSuccess: false,
      isPending: !configQuery.isError, isLoading: !configQuery.isError,
      status: configQuery.isError ? 'error' as const : 'pending' as const }
  }
  return result
}

export function useBalance(options?: Partial<UseQueryOptions<BalanceResult>>) {
  return useScopedAccountQuery(KEYS.balance, cmd.getBalance, options)
}

export function useOverseasBalance(options?: Partial<UseQueryOptions<OverseasBalanceResult>>) {
  return useScopedAccountQuery(KEYS.overseasBalance, cmd.getOverseasBalance, options)
}

export function useBrokerHoldings(options?: Partial<UseQueryOptions<BrokerHoldingView[]>>) {
  return useScopedAccountQuery(KEYS.brokerHoldings, cmd.getBrokerHoldings, options)
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
    mutationKey: profileMutationKey,
    scope: profileMutationScope,
    onError: (error) => reconcileRejectedProfileMutation(qc, error),
    mutationFn: (input: AddProfileInput) => cmd.addProfile(input),
    onSuccess: async () => {
      await refreshAccountConfig(qc)
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
      void qc.invalidateQueries({ queryKey: ['strategies'] })
    },
  })
}

export function useUpdateProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationKey: profileMutationKey,
    scope: profileMutationScope,
    onError: (error) => reconcileRejectedProfileMutation(qc, error),
    mutationFn: (input: UpdateProfileInput) => cmd.updateProfile(input),
    onSuccess: async (updatedProfile) => {
      qc.setQueryData<AccountProfileView[]>(KEYS.profiles, (old) =>
        old?.map((profile) =>
          profile.id === updatedProfile.id ? updatedProfile : profile
        ) ?? [updatedProfile],
      )
      const cancelled = updatedProfile.is_active
        ? qc.cancelQueries({ queryKey: KEYS.appConfig }) : Promise.resolve()
      if (updatedProfile.is_active) {
        qc.setQueryData<AppConfigView>(KEYS.appConfig, (old) =>
          old
            ? {
                ...old,
                active_profile_id: updatedProfile.id,
                active_profile_name: updatedProfile.name,
                active_broker_id: updatedProfile.broker_id,
                active_broker_account_id: updatedProfile.broker_account_id,
                kis_is_paper_trading: updatedProfile.is_paper_trading,
              }
            : old,
        )
      }
      await cancelled
      if (updatedProfile.is_active) {
        if (trustedAccountConfig(qc)) await resetAccountHoldings(qc)
        else await refreshAccountConfig(qc)
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
    mutationKey: profileMutationKey,
    scope: profileMutationScope,
    onError: (error) => reconcileRejectedProfileMutation(qc, error),
    mutationFn: (id: string) => cmd.deleteProfile(id),
    onSuccess: async () => {
      await refreshAccountConfig(qc)
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
      void qc.invalidateQueries({ queryKey: ['strategies'] })
    },
  })
}

export function useSetActiveProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationKey: profileMutationKey,
    scope: profileMutationScope,
    onError: (error) => reconcileRejectedProfileMutation(qc, error),
    mutationFn: (id: string) => cmd.setActiveProfile(id),
    onSuccess: async (newConfig) => {
      const cancelled = qc.cancelQueries({ queryKey: KEYS.appConfig })
      qc.setQueryData(KEYS.appConfig, newConfig)
      await cancelled
      await resetAccountHoldings(qc)
      void qc.invalidateQueries({ queryKey: KEYS.profiles })
      void qc.invalidateQueries({ queryKey: KEYS.checkConfig })
      void qc.invalidateQueries({ queryKey: ['strategies'] })
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
    mutationKey: profileMutationKey,
    scope: profileMutationScope,
    onError: (error) => reconcileRejectedProfileMutation(qc, error),
    mutationFn: (profileId) => cmd.detectProfileTradingType(profileId),
    onSuccess: async (updatedProfile) => {
      qc.setQueryData<AccountProfileView[]>(KEYS.profiles, (old) =>
        old?.map((profile) =>
          profile.id === updatedProfile.id ? updatedProfile : profile
        ) ?? [updatedProfile],
      )
      const cancelled = updatedProfile.is_active
        ? qc.cancelQueries({ queryKey: KEYS.appConfig }) : Promise.resolve()
      if (updatedProfile.is_active) {
        qc.setQueryData<AppConfigView>(KEYS.appConfig, (old) =>
          old
            ? {
                ...old,
                active_profile_id: updatedProfile.id,
                active_profile_name: updatedProfile.name,
                active_broker_id: updatedProfile.broker_id,
                active_broker_account_id: updatedProfile.broker_account_id,
                kis_is_paper_trading: updatedProfile.is_paper_trading,
              }
            : old,
        )
      }
      await cancelled
      if (updatedProfile.is_active) {
        if (trustedAccountConfig(qc)) await resetAccountHoldings(qc)
        else await refreshAccountConfig(qc)
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
