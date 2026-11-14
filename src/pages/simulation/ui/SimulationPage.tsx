import { useCallback, useSyncExternalStore } from 'react'
import { useIsMutating, useQueryClient } from '@tanstack/react-query'
import type { QueryClient } from '@tanstack/react-query'
import Box from '@mui/material/Box'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import { useAppConfig } from '../../../api/accountHooks'
import { KEYS } from '../../../api/queryKeys'
import { SectorPortfolioPanel } from '../../../widgets/sector-portfolio'
import type { AppConfigView } from '../../../shared/api/types'

function currentQuoteKey(client: QueryClient) {
  const state = client.getQueryState<AppConfigView>(KEYS.appConfig)
  const publishing = client.isMutating({ mutationKey: ['account-profile-publication'] })
  const ready = state?.status === 'success' && state.fetchStatus === 'idle' && !state.isInvalidated && publishing === 0
  return JSON.stringify([state?.data?.active_profile_id ?? null, state?.data?.active_broker_id ?? null,
    state?.data?.kis_is_paper_trading ?? null, state?.dataUpdateCount ?? 0, publishing, ready])
}

export default function SimulationPage() {
  const config = useAppConfig()
  const client = useQueryClient()
  const publishing = useIsMutating({ mutationKey: ['account-profile-publication'] })
  // Count publications rather than timestamps so fixed clocks and A→B→A cannot reuse authorization.
  const subscribe = useCallback((notify: () => void) => client.getQueryCache().subscribe(notify), [client])
  const snapshot = useCallback(() => client.getQueryState(KEYS.appConfig)?.dataUpdateCount ?? 0, [client])
  const publication = useSyncExternalStore(subscribe, snapshot, snapshot)
  const ready = config.isSuccess && config.fetchStatus === 'idle' && !config.isStale && publishing === 0
  const quoteScopeKey = JSON.stringify([
    config.data?.active_profile_id ?? null, config.data?.active_broker_id ?? null,
    config.data?.kis_is_paper_trading ?? null, publication, publishing, ready,
  ])
  const quoteScopeCurrent = useCallback((key: string) => key === currentQuoteKey(client), [client])
  const profile = ready ? config.data : undefined
  return <Box sx={{ minWidth: 0 }}>
    <Typography component="h1" variant="h5" sx={{ fontWeight: 700, mb: 2 }}>시뮬레이션</Typography>
    <Paper variant="outlined" sx={{ p: 2, mb: 3 }}>
      <Stack spacing={1}>
        <Typography variant="subtitle2">시세 조회 프로필과 가상 USD 장부</Typography>
        <Typography variant="body2" sx={{ overflowWrap: 'anywhere' }}>
          시세 조회 프로필: {profile ? `${profile.active_profile_name ?? '이름 없음'} · ${profile.active_broker_id === 'kis' ? `KIS · ${profile.kis_is_paper_trading ? '모의' : '실전'}` : 'Toss · 검증형 KIS 수집 미지원'}` : config.isError ? '확인 실패 · 인증 수집 보류' : '확인 중 · 인증 수집 보류'}
        </Typography>
        <Typography variant="body2">조회 프로필은 시세 자료 수집에 사용합니다. 가상 USD 장부의 초기자금은 입력 자료에 따르며 실제 예수금이나 잔고와 연결하지 않습니다. 프로필이 바뀌면 수집 범위와 동의를 다시 확인하세요. 이미 계산한 가상 장부는 유지됩니다.</Typography>
      </Stack>
    </Paper>
    <SectorPortfolioPanel mode="workspace" quoteScopeKey={quoteScopeKey} quoteScopeReady={ready && profile?.active_broker_id === 'kis'} brokerQuoteScopeReady={ready && (profile?.active_broker_id === 'kis' || profile?.active_broker_id === 'toss')} quoteScopeCurrent={quoteScopeCurrent} />
  </Box>
}
