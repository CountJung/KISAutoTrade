import Alert from '@mui/material/Alert'
import Chip from '@mui/material/Chip'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Typography from '@mui/material/Typography'

import { useBrokerRateLimitStatus } from '../../../api/hooks'
import { Section } from './section'

function fmtEpoch(epochMs: number | null): string {
  if (!epochMs) return '-'
  return new Date(epochMs).toLocaleTimeString('ko-KR', { hour12: false })
}

function fmtMs(ms: number): string {
  if (ms <= 0) return '-'
  return ms >= 1_000 ? `${(ms / 1_000).toFixed(1)}s` : `${ms}ms`
}

export function BrokerRateLimitSection() {
  const { data: scopes = [], isError } = useBrokerRateLimitStatus()

  return (
    <Section title="Broker 요청 상태 (rate limit)">
      {isError && <Alert severity="error">rate limit 상태를 불러오지 못했습니다.</Alert>}
      {!isError && scopes.length === 0 && (
        <Alert severity="info">아직 broker API 요청이 없습니다. 시세·잔고 조회나 주문 후 표시됩니다.</Alert>
      )}
      <Stack spacing={2}>
        {scopes.map((scope) => {
          const paused = scope.groups.some((group) => group.pausedRemainingMs > 0)
          const failing = scope.groups.some((group) => group.consecutiveFailures >= 3)
          return (
            <Paper key={scope.scope} variant="outlined" sx={{ p: 1.5 }}>
              <Stack direction="row" spacing={1} alignItems="center" mb={1} flexWrap="wrap" useFlexGap>
                <Typography variant="body2" fontWeight={600} sx={{ wordBreak: 'break-all' }}>
                  {scope.scope}
                </Typography>
                {paused && <Chip size="small" color="warning" label="rate limit pause 중" />}
                {failing && <Chip size="small" color="error" label="연속 실패" />}
              </Stack>
              <Table size="small" sx={{ '& td, & th': { py: 0.5, fontSize: '0.75rem' } }}>
                <TableHead>
                  <TableRow>
                    <TableCell>그룹</TableCell>
                    <TableCell align="right">최소 간격</TableCell>
                    <TableCell align="right">pause 잔여</TableCell>
                    <TableCell align="right">rate limit 누적</TableCell>
                    <TableCell align="right">연속 실패</TableCell>
                    <TableCell align="right">마지막 성공</TableCell>
                    <TableCell align="right">마지막 실패</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {scope.groups.map((group) => (
                    <TableRow key={group.group}>
                      <TableCell><code>{group.group}</code></TableCell>
                      <TableCell align="right">{fmtMs(group.minIntervalMs)}</TableCell>
                      <TableCell align="right" sx={group.pausedRemainingMs > 0 ? { color: 'warning.main', fontWeight: 700 } : undefined}>
                        {fmtMs(group.pausedRemainingMs)}
                      </TableCell>
                      <TableCell align="right">{group.rateLimitedCount}</TableCell>
                      <TableCell align="right" sx={group.consecutiveFailures > 0 ? { color: 'error.main', fontWeight: 700 } : undefined}>
                        {group.consecutiveFailures}
                      </TableCell>
                      <TableCell align="right">{fmtEpoch(group.lastSuccessEpochMs)}</TableCell>
                      <TableCell align="right">{fmtEpoch(group.lastFailureEpochMs)}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </Paper>
          )
        })}
      </Stack>
    </Section>
  )
}
