import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useScopedAccountQuery } from './accountHooks'
import { getAutoTradingBudget, updateAutoTradingBudget } from './commands'
import { KEYS } from './queryKeys'
import { POLL_INTERVALS } from '../scheduler'
import type { AppConfigView, AutoTradingBudgetInput, AutoTradingBudgetView, CmdError, UpdateAutoTradingBudgetInput } from './types'

export function useAutoTradingBudget(input: AutoTradingBudgetInput | undefined) {
  return useScopedAccountQuery(KEYS.autoTradingBudget(input?.brokerId, input?.brokerAccountId), () => getAutoTradingBudget(input!), {
    enabled: Boolean(input?.brokerAccountId),
    staleTime: 5_000,
    refetchInterval: POLL_INTERVALS.FAST,
  })
}

export function useUpdateAutoTradingBudget() {
  const qc = useQueryClient()
  const ownsPublication = (input: UpdateAutoTradingBudgetInput, publication: number | undefined) => {
    const current = qc.getQueryState<AppConfigView>(KEYS.appConfig)
    return qc.isMutating({ mutationKey: ['account-profile-publication'] }) === 0 && current?.status === 'success' && current.fetchStatus === 'idle' &&
      !current.isInvalidated && current.dataUpdateCount === publication &&
      current.data?.active_profile_id === input.expectedProfileId &&
      current.data?.kis_is_paper_trading === input.expectedIsPaper &&
      current.data?.active_broker_id === input.brokerId &&
      current.data?.active_broker_account_id === input.brokerAccountId
  }
  return useMutation<AutoTradingBudgetView, CmdError, UpdateAutoTradingBudgetInput, { publication: number | undefined }>({
    onMutate: () => ({ publication: qc.getQueryState(KEYS.appConfig)?.dataUpdateCount }),
    mutationFn: async (input) => {
      if (!ownsPublication(input, qc.getQueryState(KEYS.appConfig)?.dataUpdateCount)) throw { code: 'BUDGET_SCOPE_CHANGED', message: '계좌 선택을 다시 확인하세요.' }
      return updateAutoTradingBudget(input)
    },
    onSuccess: async (_data, input, context) => {
      if (!ownsPublication(input, context?.publication)) throw { code: 'BUDGET_SCOPE_CHANGED', message: '이전 선택의 저장 응답입니다. 현재 예산을 다시 확인하세요.' }
      try {
        // invalidateQueries resolves immediately for paused refetches. Await an
        // actual read of this publication's exact key before reporting success.
        const query = qc.getQueryCache().findAll({
          queryKey: KEYS.autoTradingBudget(input.brokerId, input.brokerAccountId),
          type: 'active',
        }).find(candidate => candidate.queryKey[candidate.queryKey.length - 5] === context?.publication)
        if (!query) throw new Error('현재 선택의 예산 조회를 찾을 수 없습니다.')
        await qc.cancelQueries({ queryKey: query.queryKey, exact: true })
        await qc.fetchQuery<AutoTradingBudgetView>({
          queryKey: query.queryKey,
          staleTime: 0,
          retry: false,
          queryFn: () => {
            if (!ownsPublication(input, context?.publication)) throw new Error('예산 선택이 변경되었습니다.')
            return getAutoTradingBudget({ brokerId: input.brokerId, brokerAccountId: input.brokerAccountId })
          },
        })
      } catch {
        throw { code: 'BUDGET_REVALIDATION_FAILED', message: '예산은 저장됐지만 재조회에 실패했습니다. 입력을 유지합니다. 다시 조회해 저장 결과를 확인하세요.' }
      }
      if (!ownsPublication(input, context?.publication)) throw { code: 'BUDGET_SCOPE_CHANGED', message: '계좌 선택이 변경되었습니다. 현재 예산을 다시 확인하세요.' }
    },
  })
}
