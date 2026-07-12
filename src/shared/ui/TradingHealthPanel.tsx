import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Chip from '@mui/material/Chip'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import type { TradingHealthStatus } from '../api/types'

function timestamp(value: string | null) {
  return value ? new Date(value).toLocaleString('ko-KR') : '아직 확인되지 않음'
}

export function tradingHealthProblem(health?: TradingHealthStatus | null) {
  if (!health) return null
  return health.persistenceError
    ?? (health.persistenceBlocked ? '데이터 영속화 장애로 신규 주문이 차단되었습니다.' : null)
    ?? health.lastHoldingsSyncError
    ?? health.lastReconciliationError
    ?? health.daemonLastError
}

export function TradingHealthPanel({ health }: { health: TradingHealthStatus }) {
  const problem = tradingHealthProblem(health)
  const unknown = !health.lastHoldingsSyncAt || !health.lastReconciliationAt
  return (
    <Paper variant="outlined" sx={{ p: 2 }}>
      <Stack direction="row" justifyContent="space-between" alignItems="center" mb={1.5}>
        <Typography variant="subtitle2" fontWeight={700}>자동매매 건강 상태</Typography>
        <Chip
          size="small"
          color={problem ? 'error' : unknown ? 'warning' : 'success'}
          label={problem ? '확인 필요' : unknown ? '확인 대기' : '정상'}
        />
      </Stack>
      {problem && <Alert severity="error" sx={{ mb: 1.5 }}>{problem}</Alert>}
      <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, 1fr)' }, gap: 1 }}>
        <Typography variant="body2">마지막 정상 잔고 동기화: {timestamp(health.lastHoldingsSyncAt)}</Typography>
        <Typography variant="body2">마지막 정상 체결 대조: {timestamp(health.lastReconciliationAt)}</Typography>
        <Typography variant="body2">미체결: {health.pendingOrderCount}건</Typography>
        <Typography variant="body2">최장 미체결: {timestamp(health.oldestPendingAt)}</Typography>
        <Typography variant="body2">영속화 차단: {health.persistenceBlocked ? '차단됨' : '정상'}</Typography>
        <Typography variant="body2">데몬 연속 실패: {health.daemonConsecutiveFailures}회</Typography>
      </Box>
      {health.persistenceBlocked && (
        <Typography variant="caption" color="text.secondary" display="block" mt={1.5}>
          저장소 연결과 쓰기 권한을 복구한 뒤 앱을 재시작해야 신규 주문 차단이 해제됩니다.
        </Typography>
      )}
      {!health.persistenceBlocked && (health.lastHoldingsSyncError || health.lastReconciliationError) && (
        <Typography variant="caption" color="text.secondary" display="block" mt={1.5}>
          broker 연결·계좌 인증을 확인하세요. 다음 polling 성공 시 해당 동기화 오류는 자동으로 해제됩니다.
        </Typography>
      )}
    </Paper>
  )
}
