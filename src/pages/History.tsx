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
import Tab from '@mui/material/Tab'
import Tabs from '@mui/material/Tabs'

import { useKisExecutedByRange, useStatsByRange, useTradesByRange } from '../api/hooks'

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
  const [tab, setTab]     = useState(0) // 0=KIS 체결, 1=로컬 기록

  // ─── KIS 체결 ───────────────────────────────────────────────────
  const {
    data: kisOrders,
    isLoading: kisLoading,
    error: kisError,
  } = useKisExecutedByRange(query?.from ?? '', query?.to ?? '', { enabled: !!query })

  // ─── 로컬 기록 ──────────────────────────────────────────────────
  const {
    data: trades,
    isLoading: tradesLoading,
    error: tradesError,
  } = useTradesByRange(query?.from ?? '', query?.to ?? '', { enabled: !!query && tab === 1 })

  const {
    data: stats,
    isLoading: statsLoading,
  } = useStatsByRange(query?.from ?? '', query?.to ?? '', { enabled: !!query && tab === 1 })

  const handleSearch = () => setQuery({ from, to })

  // KIS 요약
  const kisTotal  = kisOrders?.length ?? 0
  const kisBuys   = kisOrders?.filter(o => o.sll_buy_dvsn_cd === '02').length ?? 0
  const kisSells  = kisOrders?.filter(o => o.sll_buy_dvsn_cd === '01').length ?? 0
  const kisAmount = kisOrders?.reduce((s, o) => s + parseInt(o.tot_ccld_amt || '0'), 0) ?? 0

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

      {/* 탭 */}
      <Paper sx={{ mb: 2 }}>
        <Tabs value={tab} onChange={(_, v: number) => setTab(v)}>
          <Tab label="KIS 체결 내역" />
          <Tab label="로컬 자동매매 기록" />
        </Tabs>
      </Paper>

      {/* ── KIS 체결 탭 ─────────────────────────────────────────── */}
      {tab === 0 && (
        <>
          {query && (
            <Grid container spacing={2} mb={2}>
              {[
                { label: '총 체결 건수', value: `${fmt(kisTotal)}건` },
                { label: '매수', value: `${fmt(kisBuys)}건` },
                { label: '매도', value: `${fmt(kisSells)}건` },
                { label: '총 체결 금액', value: `${fmt(kisAmount)}원` },
              ].map(({ label, value }) => (
                <Grid item xs={6} sm={3} key={label}>
                  <Paper sx={{ p: 2 }}>
                    <Typography variant="caption" color="text.secondary">{label}</Typography>
                    <Typography variant="h6" fontWeight={700}>
                      {kisLoading ? <CircularProgress size={16} /> : value}
                    </Typography>
                  </Paper>
                </Grid>
              ))}
            </Grid>
          )}

          <Paper sx={{ p: 2.5 }}>
            <Typography variant="subtitle1" fontWeight={600} mb={2}>KIS 체결 내역</Typography>
            <Divider sx={{ mb: 2 }} />
            <Alert severity="info" sx={{ mb: 2 }}>
              국내 주식(KRX) 체결 내역만 조회됩니다. 해외 주식(NASDAQ/NYSE/AMEX)은 KIS 해외 체결 조회 API를 통해 별도 확인하세요.
            </Alert>
            {!query && (
              <Typography variant="body2" color="text.secondary">
                기간을 선택하고 조회 버튼을 누르세요.
              </Typography>
            )}
            {kisLoading && <CircularProgress />}
            {kisError && <Alert severity="error">{(kisError as { message?: string })?.message ?? String(kisError)}</Alert>}
            {kisOrders && kisOrders.length === 0 && !kisLoading && (
              <Typography variant="body2" color="text.secondary">
                해당 기간에 체결 내역이 없습니다.
              </Typography>
            )}
            {kisOrders && kisOrders.length > 0 && (
              <TableContainer sx={{ maxHeight: 480 }}>
                <Table size="small" stickyHeader>
                  <TableHead>
                    <TableRow>
                      <TableCell>일자</TableCell>
                      <TableCell>시각</TableCell>
                      <TableCell>종목</TableCell>
                      <TableCell>구분</TableCell>
                      <TableCell align="right">체결수량</TableCell>
                      <TableCell align="right">단가</TableCell>
                      <TableCell align="right">금액</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {kisOrders.map((o, i) => (
                      <TableRow key={`${o.odno}-${o.ord_tmd}-${i}`}>
                        <TableCell>{o.ord_dt}</TableCell>
                        <TableCell>{o.ord_tmd}</TableCell>
                        <TableCell>
                          {o.prdt_name}
                          <br />
                          <Typography variant="caption" color="text.secondary">{o.pdno}</Typography>
                        </TableCell>
                        <TableCell>
                          <Chip
                            label={o.sll_buy_dvsn_cd === '01' ? '매도' : '매수'}
                            color={o.sll_buy_dvsn_cd === '01' ? 'error' : 'primary'}
                            size="small"
                          />
                        </TableCell>
                        <TableCell align="right">{fmt(parseInt(o.tot_ccld_qty))}</TableCell>
                        <TableCell align="right">{fmt(parseInt(o.ord_unpr))}원</TableCell>
                        <TableCell align="right">{fmt(parseInt(o.tot_ccld_amt))}원</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            )}
          </Paper>
        </>
      )}

      {/* ── 로컬 기록 탭 ─────────────────────────────────────────── */}
      {tab === 1 && (
        <>
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
            {trades && trades.length === 0 && !tradesLoading && (
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
        </>
      )}
    </Box>
  )
}

