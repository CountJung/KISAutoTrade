import { useState } from 'react'
import Typography from '@mui/material/Typography'
import Paper from '@mui/material/Paper'
import Box from '@mui/material/Box'
import Grid from '@mui/material/Grid'
import TextField from '@mui/material/TextField'
import Button from '@mui/material/Button'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Alert from '@mui/material/Alert'
import Divider from '@mui/material/Divider'
import { useStatsByRange, useTradesByRange } from '../api/hooks'

function fmt(n: number) {
  return n.toLocaleString('ko-KR')
}

function today() {
  return new Date().toISOString().slice(0, 10)
}

function weekAgo() {
  const d = new Date()
  d.setDate(d.getDate() - 7)
  return d.toISOString().slice(0, 10)
}

export default function History() {
  const [from, setFrom]   = useState(weekAgo())
  const [to, setTo]       = useState(today())
  const [query, setQuery] = useState<{ from: string; to: string } | null>(null)

  // ─── 로컬 기록 ──────────────────────────────────────────────────
  const {
    data: trades,
    isLoading: tradesLoading,
    error: tradesError,
  } = useTradesByRange(query?.from ?? '', query?.to ?? '', { enabled: !!query })

  const {
    data: stats,
    isLoading: statsLoading,
  } = useStatsByRange(query?.from ?? '', query?.to ?? '', { enabled: !!query })

  const handleSearch = () => setQuery({ from, to })

  // 로컬 요약
  const totalNetProfit = stats?.reduce((acc, s) => acc + s.net_profit, 0) ?? 0
  const totalTrades    = stats?.reduce((acc, s) => acc + s.total_trades, 0) ?? 0
  const avgWinRate     = stats && stats.length > 0
    ? stats.reduce((acc, s) => acc + s.win_rate, 0) / stats.length
    : 0

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        History
      </Typography>

      {/* 기간 선택 */}
      <Paper sx={{ p: 2.5, mb: 2 }}>
        <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', flexWrap: 'wrap' }}>
          <TextField
            label="시작일"
            type="date"
            value={from}
            onChange={(e) => setFrom(e.target.value)}
            size="small"
            InputLabelProps={{ shrink: true }}
          />
          <TextField
            label="종료일"
            type="date"
            value={to}
            onChange={(e) => setTo(e.target.value)}
            size="small"
            InputLabelProps={{ shrink: true }}
          />
          <Button variant="contained" onClick={handleSearch}>
            조회
          </Button>
        </Box>
      </Paper>

      {/* ── 자동매매 체결 기록 ──────────────────────────────────── */}
      {query && (
        <Grid container spacing={2} mb={2}>
          {[
            { label: '기간 순손익', value: (totalNetProfit >= 0 ? '+' : '') + fmt(totalNetProfit) + '원' },
            { label: '총 거래 수', value: `${fmt(totalTrades)}건` },
            { label: '평균 승률', value: `${(avgWinRate * 100).toFixed(1)}%` },
          ].map(({ label, value }) => (
            <Grid item xs={12} sm={4} key={label}>
              <Paper sx={{ p: 2 }}>
                <Typography variant="caption" color="text.secondary">{label}</Typography>
                <Typography variant="h6" fontWeight={700}>
                  {statsLoading ? <CircularProgress size={16} /> : value}
                </Typography>
              </Paper>
            </Grid>
          ))}
        </Grid>
      )}

      <Paper sx={{ p: 2.5 }}>
        <Typography variant="subtitle1" fontWeight={600} mb={2}>자동매매 체결 기록</Typography>
        <Divider sx={{ mb: 2 }} />
        {!query && (
          <Typography variant="body2" color="text.secondary">
            기간을 선택하고 조회 버튼을 누르세요.
          </Typography>
        )}
        {tradesLoading && <CircularProgress />}
        {tradesError  && <Alert severity="error">{(tradesError as { message?: string })?.message ?? String(tradesError)}</Alert>}
        {trades && trades.length === 0 && !tradesLoading && query && (
          <Typography variant="body2" color="text.secondary">
            해당 기간에 체결 기록이 없습니다. (자동매매 실행 시 기록됩니다)
          </Typography>
        )}
        {trades && trades.length > 0 && (
          <TableContainer sx={{ maxHeight: 480 }}>
            <Table size="small" stickyHeader>
              <TableHead>
                <TableRow>
                  <TableCell>일시</TableCell>
                  <TableCell>종목</TableCell>
                  <TableCell>구분</TableCell>
                  <TableCell align="right">수량</TableCell>
                  <TableCell align="right">가격</TableCell>
                  <TableCell align="right">금액</TableCell>
                  <TableCell align="right">수수료</TableCell>
                  <TableCell>전략</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {trades.map((t) => (
                  <TableRow key={t.id}>
                    <TableCell>{t.timestamp.slice(0, 19).replace('T', ' ')}</TableCell>
                    <TableCell>
                      {t.symbol_name}
                      <br />
                      <Typography variant="caption" color="text.secondary">{t.symbol}</Typography>
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={t.side === 'buy' ? '매수' : '매도'}
                        color={t.side === 'buy' ? 'primary' : 'error'}
                        size="small"
                      />
                    </TableCell>
                    <TableCell align="right">{fmt(t.quantity)}</TableCell>
                    <TableCell align="right">{fmt(t.price)}원</TableCell>
                    <TableCell align="right">{fmt(t.total_amount)}원</TableCell>
                    <TableCell align="right">{fmt(t.fee)}원</TableCell>
                    <TableCell>{t.strategy_id ?? '-'}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        )}
      </Paper>
    </Box>
  )
}
