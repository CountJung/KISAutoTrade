import Typography from '@mui/material/Typography'
import Grid from '@mui/material/Grid'
import Paper from '@mui/material/Paper'
import Box from '@mui/material/Box'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Alert from '@mui/material/Alert'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Divider from '@mui/material/Divider'
import Stack from '@mui/material/Stack'
import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
import LinearProgress from '@mui/material/LinearProgress'
import Collapse from '@mui/material/Collapse'
import TrendingUpIcon from '@mui/icons-material/TrendingUp'
import TrendingDownIcon from '@mui/icons-material/TrendingDown'
import RefreshIcon from '@mui/icons-material/Refresh'
import PlayArrowIcon from '@mui/icons-material/PlayArrow'
import StopIcon from '@mui/icons-material/Stop'
import WarningAmberIcon from '@mui/icons-material/WarningAmber'
import LockOpenIcon from '@mui/icons-material/LockOpen'
import SaveIcon from '@mui/icons-material/Save'
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'
import ExpandLessIcon from '@mui/icons-material/ExpandLess'
import IconButton from '@mui/material/IconButton'
import Tooltip from '@mui/material/Tooltip'
import { useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'

import {
  useBalance,
  useTodayStats,
  useCheckConfig,
  useTradingStatus,
  useStartTrading,
  useStopTrading,
  usePositions,
  usePendingOrders,
  useTradesByRange,
  useRiskConfig,
  useUpdateRiskConfig,
  useClearEmergencyStop,
  KEYS,
} from '../api/hooks'

function fmt(n: number) {
  return n.toLocaleString('ko-KR')
}

function todayStr() {
  return new Date().toISOString().slice(0, 10)
}

// ─── 리스크 관리 패널 ─────────────────────────────────────────────
function RiskPanel() {
  const { data: risk, isLoading } = useRiskConfig()
  const { mutate: update, isPending: saving } = useUpdateRiskConfig()
  const { mutate: clearStop, isPending: clearing } = useClearEmergencyStop()

  const [limitInput, setLimitInput]   = useState('')
  const [ratioInput, setRatioInput]   = useState('')
  const [dirty, setDirty]             = useState(false)

  const handleSave = () => {
    const input: { dailyLossLimit?: number; maxPositionRatio?: number } = {}
    const parsed = parseInt(limitInput.replace(/,/g, ''))
    const parsedRatio = parseFloat(ratioInput)
    if (!isNaN(parsed) && parsed >= 0)            input.dailyLossLimit = parsed
    if (!isNaN(parsedRatio) && parsedRatio > 0)   input.maxPositionRatio = parsedRatio / 100
    update(input, {
      onSuccess: () => { setLimitInput(''); setRatioInput(''); setDirty(false) },
    })
  }

  if (isLoading || !risk) {
    return <Box sx={{ py: 2, display: 'flex', justifyContent: 'center' }}><CircularProgress size={20} /></Box>
  }

  const lossRatioPct = Math.min(risk.lossRatio * 100, 100)
  const barColor = lossRatioPct < 50 ? 'success' : lossRatioPct < 80 ? 'warning' : 'error'

  return (
    <Box>
      {/* 비상 정지 배너 */}
      {risk.isEmergencyStop && (
        <Alert
          severity="error"
          icon={<WarningAmberIcon />}
          sx={{ mb: 2 }}
          action={
            <Button
              size="small"
              color="inherit"
              startIcon={clearing ? <CircularProgress size={14} color="inherit" /> : <LockOpenIcon />}
              onClick={() => clearStop()}
              disabled={clearing}
            >
              비상정지 해제
            </Button>
          }
        >
          <strong>비상 정지 활성</strong> — 일일 손실 한도를 초과하여 자동 매매가 중단되었습니다.
          시장 상황을 확인 후 수동으로 해제하세요.
        </Alert>
      )}

      {/* 손실 한도 진행바 */}
      <Box sx={{ mb: 2 }}>
        <Stack direction="row" justifyContent="space-between" mb={0.5}>
          <Typography variant="caption" color="text.secondary">
            손실 소진율
          </Typography>
          <Typography
            variant="caption"
            fontWeight={700}
            color={`${barColor}.main`}
          >
            {fmt(Math.abs(risk.currentLoss))}원 / {fmt(risk.dailyLossLimit)}원
            &nbsp;({lossRatioPct.toFixed(1)}%)
          </Typography>
        </Stack>
        <LinearProgress
          variant="determinate"
          value={lossRatioPct}
          color={barColor}
          sx={{ borderRadius: 1, height: 8 }}
        />
      </Box>

      {/* 현재 설정값 표시 */}
      <Grid container spacing={1.5} sx={{ mb: 2 }}>
        <Grid item xs={6}>
          <Box sx={{ p: 1.5, bgcolor: 'action.hover', borderRadius: 1, textAlign: 'center' }}>
            <Typography variant="caption" color="text.secondary" display="block">일일 손실 한도</Typography>
            <Typography variant="body1" fontWeight={700}>{fmt(risk.dailyLossLimit)}원</Typography>
          </Box>
        </Grid>
        <Grid item xs={6}>
          <Box sx={{ p: 1.5, bgcolor: 'action.hover', borderRadius: 1, textAlign: 'center' }}>
            <Typography variant="caption" color="text.secondary" display="block">종목당 최대 비중</Typography>
            <Typography variant="body1" fontWeight={700}>{(risk.maxPositionRatio * 100).toFixed(0)}%</Typography>
          </Box>
        </Grid>
      </Grid>

      {/* 설정 변경 입력 */}
      <Grid container spacing={1.5} alignItems="flex-end">
        <Grid item xs={12} sm={5}>
          <Tooltip
            title="하루 최대 허용 손실 금액(원). 이 금액을 초과하면 비상 정지됩니다."
            arrow placement="top"
          >
            <TextField
              label="일일 손실 한도 (원)"
              value={limitInput}
              placeholder={fmt(risk.dailyLossLimit)}
              onChange={(e) => { setLimitInput(e.target.value.replace(/[^\d,]/g, '')); setDirty(true) }}
              size="small"
              fullWidth
              InputProps={{ endAdornment: <InfoOutlinedIcon fontSize="small" sx={{ color: 'text.disabled' }} /> }}
            />
          </Tooltip>
        </Grid>
        <Grid item xs={12} sm={5}>
          <Tooltip
            title="단일 종목에 투자할 수 있는 최대 비중(%). 예: 20 → 총 잔고의 20%까지."
            arrow placement="top"
          >
            <TextField
              label="종목당 최대 비중 (%)"
              value={ratioInput}
              placeholder={(risk.maxPositionRatio * 100).toFixed(0)}
              onChange={(e) => { setRatioInput(e.target.value.replace(/[^\d.]/g, '')); setDirty(true) }}
              size="small"
              fullWidth
              InputProps={{ endAdornment: <InfoOutlinedIcon fontSize="small" sx={{ color: 'text.disabled' }} /> }}
            />
          </Tooltip>
        </Grid>
        <Grid item xs={12} sm={2}>
          <Button
            variant="contained"
            size="small"
            startIcon={saving ? <CircularProgress size={14} color="inherit" /> : <SaveIcon />}
            onClick={handleSave}
            disabled={!dirty || saving}
            fullWidth
          >
            저장
          </Button>
        </Grid>
      </Grid>

      {!risk.isEmergencyStop && (
        <Typography
          variant="caption"
          color={risk.canTrade ? 'success.main' : 'warning.main'}
          sx={{ mt: 1, display: 'block' }}
        >
          {risk.canTrade ? '✅ 거래 가능 상태' : '⚠️ 거래 불가 상태'}
        </Typography>
      )}
    </Box>
  )
}

// ─── 체결 내역 패널 (자동매매 기록) ────────────────────
function FilledOrdersPanel() {
  const [from, setFrom] = useState(todayStr)
  const [to, setTo] = useState(todayStr)
  const [queryFrom, setQueryFrom] = useState(todayStr)
  const [queryTo, setQueryTo] = useState(todayStr)

  const { data: trades = [], isLoading } = useTradesByRange(queryFrom, queryTo)

  const handleQuery = () => {
    setQueryFrom(from)
    setQueryTo(to)
  }

  return (
    <Box>
      {/* 기간 선택 */}
      <Stack direction="row" spacing={1} alignItems="center" mb={1.5} flexWrap="wrap">
        <TextField
          type="date"
          label="시작일"
          value={from}
          onChange={(e) => setFrom(e.target.value)}
          size="small"
          slotProps={{ inputLabel: { shrink: true } }}
          sx={{ width: 150 }}
        />
        <TextField
          type="date"
          label="종료일"
          value={to}
          onChange={(e) => setTo(e.target.value)}
          size="small"
          slotProps={{ inputLabel: { shrink: true } }}
          sx={{ width: 150 }}
        />
        <Button variant="outlined" size="small" onClick={handleQuery}>
          조회
        </Button>
      </Stack>

      {isLoading ? (
        <Box sx={{ py: 2, display: 'flex', justifyContent: 'center' }}>
          <CircularProgress size={20} />
        </Box>
      ) : trades.length === 0 ? (
        <Typography variant="body2" color="text.secondary">
          해당 기간에 체결 내역이 없습니다.
        </Typography>
      ) : (
        <TableContainer sx={{ maxHeight: 320 }}>
          <Table size="small" stickyHeader>
            <TableHead>
              <TableRow>
                <TableCell>종목</TableCell>
                <TableCell>구분</TableCell>
                <TableCell align="right">수량</TableCell>
                <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>단가</TableCell>
                <TableCell align="right" sx={{ display: { xs: 'none', md: 'table-cell' } }}>금액</TableCell>
                <TableCell sx={{ display: { xs: 'none', sm: 'table-cell' } }}>일시</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {trades.map((t) => (
                <TableRow key={t.id} hover>
                  <TableCell>
                    <Typography variant="body2" component="span" fontWeight={500}>
                      {t.symbol_name}
                    </Typography>
                    <Typography
                      variant="caption"
                      color="text.secondary"
                      component="span"
                      sx={{ ml: 0.5 }}
                    >
                      {t.symbol}
                    </Typography>
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={t.side === 'buy' ? '매수' : '매도'}
                      color={t.side === 'buy' ? 'primary' : 'error'}
                      size="small"
                    />
                  </TableCell>
                  <TableCell align="right">{fmt(t.quantity)}</TableCell>
                  <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                    {fmt(t.price)}원
                  </TableCell>
                  <TableCell align="right" sx={{ display: { xs: 'none', md: 'table-cell' } }}>
                    {fmt(t.total_amount)}원
                  </TableCell>
                  <TableCell sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                    {t.timestamp.slice(0, 16).replace('T', ' ')}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Box>
  )
}

// ─── 미체결 주문 패널 ─────────────────────────────────────────────
function PendingOrdersPanel() {
  const { data: orders = [], isLoading } = usePendingOrders()

  if (isLoading) {
    return <Box sx={{ py: 2, display: 'flex', justifyContent: 'center' }}><CircularProgress size={20} /></Box>
  }

  if (orders.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
        미체결 주문이 없습니다.
      </Typography>
    )
  }

  return (
    <TableContainer sx={{ maxHeight: 260 }}>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>종목</TableCell>
            <TableCell>구분</TableCell>
            <TableCell align="right">수량</TableCell>
            <TableCell sx={{ display: { xs: 'none', sm: 'table-cell' } }}>주문번호</TableCell>
            <TableCell sx={{ display: { xs: 'none', md: 'table-cell' } }}>신호 이유</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {orders.map((o) => (
            <TableRow key={o.odno || o.symbol + o.timestamp}>
              <TableCell>
                <Typography variant="body2" noWrap>{o.symbolName}</Typography>
                <Typography variant="caption" color="text.secondary">{o.symbol}</Typography>
              </TableCell>
              <TableCell>
                <Chip
                  label={o.side === 'buy' ? '매수' : '매도'}
                  color={o.side === 'buy' ? 'primary' : 'error'}
                  size="small"
                />
              </TableCell>
              <TableCell align="right">{o.quantity.toLocaleString()}</TableCell>
              <TableCell sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                <Typography variant="caption" color="text.secondary" noWrap>
                  {o.odno || '—'}
                </Typography>
              </TableCell>
              <TableCell sx={{ display: { xs: 'none', md: 'table-cell' } }}>
                <Typography variant="caption" color="text.secondary" noWrap>
                  {o.signalReason || '—'}
                </Typography>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  )
}

function StatCard({
  label,
  value,
  sub,
  positive,
  loading,
}: {
  label: string
  value: string
  sub?: string
  positive?: boolean
  loading?: boolean
}) {
  return (
    <Paper sx={{ p: 2.5, height: '100%' }}>
      <Typography variant="caption" color="text.secondary" gutterBottom display="block">
        {label}
      </Typography>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        {loading ? (
          <CircularProgress size={20} />
        ) : (
          <Typography variant="h5" fontWeight={700}>
            {value}
          </Typography>
        )}
        {!loading && positive !== undefined &&
          (positive ? (
            <TrendingUpIcon color="success" fontSize="small" />
          ) : (
            <TrendingDownIcon color="error" fontSize="small" />
          ))}
      </Box>
      {sub && (
        <Typography variant="body2" color="text.secondary" mt={0.5}>
          {sub}
        </Typography>
      )}
    </Paper>
  )
}

export default function Dashboard() {
  const qc = useQueryClient()
  const navigate = useNavigate()

  const [riskExpanded, setRiskExpanded] = useState(false)

  const { data: balance, isLoading: balanceLoading, isError: balanceError, error: balanceErrorDetail } = useBalance()
  const { data: stats, isLoading: statsLoading } = useTodayStats()
  const { data: diag } = useCheckConfig()
  const { data: tradingStatus } = useTradingStatus()
  const { data: positions } = usePositions()
  const { mutate: startTrading, isPending: startPending } = useStartTrading()
  const { mutate: stopTrading, isPending: stopPending } = useStopTrading()

  const totalBalance = parseInt(balance?.summary?.tot_evlu_amt ?? '0') || 0
  const availableCash = parseInt(balance?.summary?.dnca_tot_amt ?? '0') || 0
  const netProfit = stats?.net_profit ?? 0
  const profitPositive = netProfit >= 0
  const isRunning = tradingStatus?.isRunning ?? false
  const configReady = diag?.is_ready ?? true  // 데이터 없으면 배너 숨김

  const handleRefresh = () => {
    void qc.invalidateQueries({ queryKey: KEYS.balance })
    void qc.invalidateQueries({ queryKey: KEYS.todayStats })
    void qc.invalidateQueries({ queryKey: KEYS.tradingStatus })
    void qc.invalidateQueries({ queryKey: KEYS.positions })
  }

  return (
    <Box>
      {/* 잔고 조회 실패 배너 */}
      {balanceError && (
        <Alert severity="error" sx={{ mb: 2 }}>
          <strong>잔고 조회 실패</strong> —{' '}
          {(balanceErrorDetail as { message?: string } | null)?.message ?? '알 수 없는 오류. Log 페이지에서 상세 내용을 확인하세요.'}
        </Alert>
      )}

      {/* 설정 미비 경고 배너 */}
      {diag && !configReady && (
        <Alert
          severity="warning"
          icon={<WarningAmberIcon />}
          action={
            <Button
              color="inherit"
              size="small"
              onClick={() => void navigate({ to: '/settings' })}
            >
              설정으로 이동
            </Button>
          }
          sx={{ mb: 2 }}
        >
          <strong>API 설정 미완료</strong> — {diag.issues[0]}
        </Alert>
      )}

      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 3 }}>
        <Typography variant="h5" fontWeight={700}>
          Dashboard
        </Typography>
        <Chip
          label={isRunning ? '자동매매 실행 중' : '대기'}
          color={isRunning ? 'success' : 'default'}
          size="small"
        />

        <Box sx={{ ml: 'auto', display: 'flex', gap: 1 }}>
          {isRunning ? (
            <Button
              variant="outlined"
              color="error"
              size="small"
              startIcon={stopPending ? <CircularProgress size={16} /> : <StopIcon />}
              onClick={() => stopTrading()}
              disabled={stopPending}
            >
              자동매매 정지
            </Button>
          ) : (
            <Button
              variant="contained"
              color="primary"
              size="small"
              startIcon={startPending ? <CircularProgress size={16} /> : <PlayArrowIcon />}
              onClick={() => startTrading()}
              disabled={startPending || !configReady}
            >
              자동매매 시작
            </Button>
          )}
          <Tooltip title="새로고침">
            <IconButton size="small" onClick={handleRefresh}>
              <RefreshIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      <Grid container spacing={2} mb={3}>
        <Grid item xs={12} sm={6} md={3}>
          <StatCard
            label="총 평가금액"
            value={fmt(totalBalance) + '원'}
            sub="예수금 + 주식평가"
            loading={balanceLoading}
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <StatCard
            label="예수금"
            value={fmt(availableCash) + '원'}
            sub="매매 가능 금액"
            loading={balanceLoading}
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <StatCard
            label="당일 손익"
            value={(profitPositive ? '+' : '') + fmt(netProfit) + '원'}
            sub={`승률 ${stats ? (stats.win_rate * 100).toFixed(1) : '-'}%`}
            positive={profitPositive}
            loading={statsLoading}
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <StatCard
            label="보유 포지션"
            value={`${tradingStatus?.positionCount ?? 0}종목`}
            sub={`미실현 손익 ${fmt(tradingStatus?.totalUnrealizedPnl ?? 0)}원`}
            positive={(tradingStatus?.totalUnrealizedPnl ?? 0) >= 0}
          />
        </Grid>
      </Grid>

      {/* 포지션 현황 */}
      {positions && positions.length > 0 && (
        <Paper sx={{ p: 2.5, mb: 2 }}>
          <Typography variant="subtitle1" fontWeight={600} mb={2}>
            보유 포지션
          </Typography>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>종목명</TableCell>
                <TableCell align="right">수량</TableCell>
                <TableCell align="right">평균단가</TableCell>
                <TableCell align="right">현재가</TableCell>
                <TableCell align="right">미실현손익</TableCell>
                <TableCell align="right">수익률</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {positions.map((p) => (
                <TableRow key={p.symbol}>
                  <TableCell>{p.symbolName} ({p.symbol})</TableCell>
                  <TableCell align="right">{fmt(p.quantity)}</TableCell>
                  <TableCell align="right">{fmt(Math.round(p.avgPrice))}원</TableCell>
                  <TableCell align="right">{fmt(p.currentPrice)}원</TableCell>
                  <TableCell
                    align="right"
                    sx={{ color: p.unrealizedPnl >= 0 ? 'success.main' : 'error.main' }}
                  >
                    {p.unrealizedPnl >= 0 ? '+' : ''}{fmt(p.unrealizedPnl)}원
                  </TableCell>
                  <TableCell
                    align="right"
                    sx={{ color: p.unrealizedPnlRate >= 0 ? 'success.main' : 'error.main' }}
                  >
                    {p.unrealizedPnlRate >= 0 ? '+' : ''}{p.unrealizedPnlRate.toFixed(2)}%
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Paper>
      )}

      {/* 미체결 주문 (자동매매) */}
      <Paper sx={{ p: 2.5, mb: 2 }}>
        <Stack direction="row" alignItems="center" spacing={1} mb={1.5}>
          <Typography variant="subtitle1" fontWeight={600}>미체결 주문</Typography>
          <Tooltip
            title="자동 매매 엔진이 KIS API에 접수했으나 아직 체결되지 않은 주문 목록입니다. 5초마다 갱신됩니다."
            arrow
          >
            <CircularProgress size={14} sx={{ display: 'none' }} />
          </Tooltip>
        </Stack>
        <Divider sx={{ mb: 2 }} />
        <PendingOrdersPanel />
      </Paper>

      {/* 자동매매 체결 내역 */}
      <Paper sx={{ p: 2.5, mb: 2 }}>
        <Typography variant="subtitle1" fontWeight={600} mb={1.5}>
          체결 내역 (자동매매)
        </Typography>
        <Divider sx={{ mb: 2 }} />
        <FilledOrdersPanel />
      </Paper>

      {/* ── 리스크 관리 (펼치기/접기) ────────────────────────────── */}
      <Paper sx={{ p: 2.5 }}>
        <Stack
          direction="row"
          alignItems="center"
          spacing={1}
          onClick={() => setRiskExpanded(v => !v)}
          sx={{ cursor: 'pointer', userSelect: 'none' }}
        >
          <Typography variant="subtitle1" fontWeight={600}>리스크 관리</Typography>
          <Tooltip
            title="일일 손실이 한도를 초과하거나, 종목 비중이 초과되면 주문이 자동으로 차단됩니다."
            arrow
          >
            <InfoOutlinedIcon fontSize="small" sx={{ color: 'text.disabled' }} />
          </Tooltip>
          <Box sx={{ ml: 'auto' }}>
            <IconButton size="small" tabIndex={-1}>
              {riskExpanded ? <ExpandLessIcon fontSize="small" /> : <ExpandMoreIcon fontSize="small" />}
            </IconButton>
          </Box>
        </Stack>
        <Collapse in={riskExpanded}>
          <Divider sx={{ my: 1.5 }} />
          <RiskPanel />
        </Collapse>
      </Paper>
    </Box>
  )
}


