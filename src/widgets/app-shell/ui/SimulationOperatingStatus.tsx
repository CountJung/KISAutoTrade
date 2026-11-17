import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Typography from '@mui/material/Typography'
import { useNavigate } from '@tanstack/react-router'
import { useTradingStatus } from '../../../api/tradingHooks'
import { QueryStatusSummary } from '../../../shared/ui/QueryStatusSummary'

/** Observes the Sidebar's existing status cache; never starts/stops trading. */
export function SimulationOperatingStatus() {
  const query = useTradingStatus({ refetchInterval: false })
  const navigate = useNavigate()
  const uncertain = query.isError || query.data === undefined
  const running = query.data?.isRunning === true
  return <Alert severity={uncertain ? 'warning' : 'info'} sx={{ mb: 2 }} data-testid="simulation-operating-status">
    <Typography component="h2" variant="subtitle2" sx={{ mb: 1 }}>운영 자동매매 상태</Typography>
    <QueryStatusSummary
      label="운영 자동매매 상태"
      statusText={query.data ? running ? '운영 자동매매 실행 중' : '운영 자동매매 정지 상태' : undefined}
      statusColor={running ? 'success' : 'default'}
      isError={query.isError}
      isFetching={query.isFetching}
      dataUpdatedAt={query.dataUpdatedAt}
      onRetry={() => { void query.refetch() }}
      testId="simulation-operating-summary"
    />
    <Typography variant="body2" sx={{ mt: 1 }}>
      {uncertain ? '현재 운영 상태가 확인되지 않았습니다. 이전 상태를 현재 상태로 판단하지 마세요.'
        : running ? '운영 자동매매는 이 비교 계산과 별도로 계속 실행 중입니다.'
          : '마지막 정상 조회에서 운영 자동매매는 정지 상태였습니다.'}
      {' '}메뉴 이동과 비교 계산은 운영 자동매매를 시작하거나 정지하지 않습니다.
    </Typography>
    <Typography variant="body2" sx={{ mt: 0.5 }}>이 메뉴에서도 앱의 기존 계좌·운영 상태 조회는 계속됩니다. 메뉴를 여는 것으로 앱의 주문 권한이 차단되지는 않습니다.</Typography>
    <Box sx={{ mt: 1 }}><Button size="small" color="inherit" onClick={() => { void navigate({ to: '/trading' }) }}>운영 화면으로 이동</Button></Box>
  </Alert>
}
