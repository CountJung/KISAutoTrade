import { useState } from 'react'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import type { AppConfigView, LiveDailyHistoryStatus, TradingStatus } from '../api/types'

const PAGE_SIZE = 3
const MAX_STATUS_ROWS = 512
const STATE_LABELS: Record<string, string> = {
  pending: '준비 중', ready: '자료 준비 완료', insufficient: '일봉 부족',
  failed: '조회 실패', capacityExceeded: '대상 한도 초과',
}
const STATE_PRIORITY: Record<string, number> = { capacityExceeded: 0, failed: 1, insufficient: 2, pending: 3, ready: 4 }

type Props = {
  status?: TradingStatus
  appConfig?: AppConfigView
}

function retryTime(value: string | null) {
  if (!value) return '예정 없음'
  const parsed = new Date(value)
  return Number.isFinite(parsed.getTime())
    ? `${parsed.toLocaleString('ko-KR', { timeZone: 'Asia/Seoul', hour12: false })} KST`
    : value
}

function HistoryRow({ value }: { value: LiveDailyHistoryStatus }) {
  return (
    <Box data-testid="live-daily-history-row" sx={{ minWidth: 0, borderTop: 1, borderColor: 'divider', pt: 1 }}>
      <Stack direction="row" alignItems="center" gap={0.75} flexWrap="wrap">
        <Typography variant="body2" fontWeight={700} sx={{ overflowWrap: 'anywhere' }}>{value.symbol}</Typography>
        <Chip size="small" label={STATE_LABELS[value.state] ?? '상태 확인 필요'}
          color={value.state === 'ready' ? 'success' : value.state === 'failed' ? 'error' : 'warning'} />
        <Typography variant="body2">확보 {value.availableBars}봉 / 필요 {value.requiredBars}봉</Typography>
      </Stack>
      <Typography variant="caption" display="block">전략 {value.strategyId}</Typography>
      <Typography variant="caption" display="block">
        시장 기준일 {value.marketDate} · 마지막 완료 일봉 {value.lastCompletedDate ?? '미확인'}
      </Typography>
      <Typography variant="caption" display="block">출처 {value.source ?? '미확인'} · 재시도 {retryTime(value.nextRetryAt)}</Typography>
      {value.lastError && <Typography variant="caption" display="block" color="warning.main">사유: {value.lastError}</Typography>}
    </Box>
  )
}

/** Shares the existing trading status query; bounds the mounted rows independently of target count. */
export function LiveDailyHistoryPanel({ status, appConfig }: Props) {
  const [page, setPage] = useState(0)
  const legacyProfileFailure = !!status?.isRunning && !status.tradingProfileId &&
    status.tradingBrokerId === 'kis' && appConfig?.active_broker_id === 'kis' &&
    !!status.tradingAccountId && status.tradingAccountId === appConfig.active_broker_account_id &&
    !!status.dailyHistory?.slice(0, MAX_STATUS_ROWS).some((row) => row.profileId === '' && row.brokerId === 'kis' && row.state === 'failed')
  const scopeMatches = legacyProfileFailure || (!!appConfig?.active_profile_id &&
    status?.tradingProfileId === appConfig.active_profile_id &&
    status?.tradingBrokerId === appConfig.active_broker_id &&
    status?.tradingAccountId === appConfig.active_broker_account_id)
  const rows = scopeMatches ? (status?.dailyHistory ?? []).slice(0, MAX_STATUS_ROWS)
    .filter((row) => row.profileId === (status?.tradingProfileId ?? '') && row.brokerId === status?.tradingBrokerId &&
      (!legacyProfileFailure || row.state === 'failed'))
    .sort((left, right) => (STATE_PRIORITY[left.state] ?? 0) - (STATE_PRIORITY[right.state] ?? 0) ||
      left.strategyId.localeCompare(right.strategyId) || left.symbol.localeCompare(right.symbol)) : []
  const lastPage = Math.max(0, Math.ceil(rows.length / PAGE_SIZE) - 1)
  const visiblePage = Math.min(page, lastPage)
  const start = visiblePage * PAGE_SIZE
  const visible = rows.slice(start, start + PAGE_SIZE)
  const ready = rows.filter((row) => row.state === 'ready').length
  const neutral = !status ? '자동매매 상태 확인 중입니다.'
    : !status.isRunning ? '미실행 · 자동매매를 시작하면 일봉 준비 상태를 확인할 수 있습니다.'
      : !status.dailyHistory ? '일봉 준비 상태 미지원 · 현재 응답에는 준비 진단이 포함되어 있지 않습니다.'
      : !appConfig?.active_profile_id && !legacyProfileFailure ? '활성 프로파일 범위를 확인 중입니다.'
        : !scopeMatches ? '활성 계좌와 실행 계좌가 달라 현재 계좌의 일봉 준비 상태를 표시하지 않습니다.'
          : rows.length === 0 ? '현재 실행 범위에 표시할 일봉 준비 대상이 없습니다.' : null
  return (
    <Paper variant="outlined" data-testid="live-daily-history-panel" sx={{ p: 1.5, mb: 2, minWidth: 0, overflowWrap: 'anywhere' }}>
      <Typography variant="subtitle2" fontWeight={700}>실시간 전략 일봉 준비</Typography>
      {neutral ? <Typography variant="body2" color="text.secondary" sx={{ mt: 0.75 }}>{neutral}</Typography> : (
        <Stack spacing={1} sx={{ minWidth: 0 }}>
          <Typography variant="caption" data-testid="live-daily-history-scope">
            실행 프로파일 {status!.tradingProfileId || '미지정(기존 KIS 설정)'} · {status!.tradingBrokerId} · 계좌 {status!.tradingAccountId}
          </Typography>
          {legacyProfileFailure && appConfig?.active_profile_id && <Alert severity="warning" sx={{ py: 0.25 }}>
            활성 프로파일 {appConfig.active_profile_id}와 달리 실행 프로파일이 지정되지 않았습니다. 일치하는 실행 계좌의 실패 진단만 표시합니다.
          </Alert>}
          <Typography variant="body2">상태 항목 {rows.length}개 · 준비 완료 {ready}개 · 준비 미완료 {rows.length - ready}개</Typography>
          <Typography variant="caption">
            준비 중 {rows.filter((row) => row.state === 'pending').length}개 · 일봉 부족 {rows.filter((row) => row.state === 'insufficient').length}개
            {' · '}조회 실패 {rows.filter((row) => row.state === 'failed').length}개 · 대상 한도 초과 {rows.filter((row) => row.state === 'capacityExceeded').length}개
          </Typography>
          <Alert severity={ready === rows.length ? 'info' : 'warning'} sx={{ py: 0.25, '& .MuiAlert-message': { minWidth: 0 } }}>
            대기·자료 부족·조회 실패·한도 초과 대상은 신규 매수를 보류합니다. 보유 포지션의 손절·청산 평가는 계속합니다.
          </Alert>
          <Typography variant="caption" color="text.secondary">
            국내는 서울, 미국은 뉴욕 거래일 기준입니다. 당일 미완료 일봉은 제외하며, 자료 준비 완료만으로 매수 조건 충족을 뜻하지 않습니다.
            {' '}마지막 완료봉이 시장 기준일보다 10일을 초과해 오래되면 신규 매수를 보류합니다. 장기 휴장도 포함하는 보수적 기준이며 거래 달력을 확정하는 검사는 아닙니다.
          </Typography>
          {visible.map((row) => <HistoryRow key={`${row.strategyId}:${row.symbol}:${row.marketDate}`} value={row} />)}
          <Stack direction="row" alignItems="center" gap={0.75} flexWrap="wrap">
            <Typography variant="caption">{start + 1}~{Math.min(start + PAGE_SIZE, rows.length)} / 전체 {rows.length}개 상태 항목</Typography>
            {rows.length > PAGE_SIZE && <>
              <Button size="small" disabled={visiblePage === 0} onClick={() => setPage(visiblePage - 1)}>이전 일봉 상태</Button>
              <Button size="small" disabled={visiblePage === lastPage} onClick={() => setPage(visiblePage + 1)}>다음 일봉 상태</Button>
            </>}
          </Stack>
        </Stack>
      )}
    </Paper>
  )
}
