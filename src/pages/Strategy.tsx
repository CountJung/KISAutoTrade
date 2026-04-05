import Typography from '@mui/material/Typography'
import Paper from '@mui/material/Paper'
import Box from '@mui/material/Box'
import Grid from '@mui/material/Grid'
import Switch from '@mui/material/Switch'
import FormControlLabel from '@mui/material/FormControlLabel'
import Chip from '@mui/material/Chip'
import Divider from '@mui/material/Divider'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import CircularProgress from '@mui/material/CircularProgress'
import Button from '@mui/material/Button'
import Alert from '@mui/material/Alert'
import LinearProgress from '@mui/material/LinearProgress'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Tooltip from '@mui/material/Tooltip'
import SaveIcon from '@mui/icons-material/Save'
import WarningAmberIcon from '@mui/icons-material/WarningAmber'
import LockOpenIcon from '@mui/icons-material/LockOpen'
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined'
import { useState } from 'react'
import {
  useStrategies,
  useUpdateStrategy,
  useTradingStatus,
  useRiskConfig,
  useUpdateRiskConfig,
  useClearEmergencyStop,
  usePendingOrders,
} from '../api/hooks'
import type { UpdateStrategyInput } from '../api/types'

function fmt(n: number) {
  return n.toLocaleString('ko-KR')
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

// ─── Strategy 메인 ────────────────────────────────────────────────
export default function Strategy() {
  const { data: strategies, isLoading } = useStrategies()
  const { data: tradingStatus } = useTradingStatus()
  const { mutate: updateStrategy, isPending: saving } = useUpdateStrategy()

  const [editMap, setEditMap] = useState<Record<string, { symbols: string; quantity: number; shortPeriod: number; longPeriod: number }>>({})

  const getEdit = (id: string, strategy: { targetSymbols: string[]; orderQuantity: number; params: Record<string, unknown> }) => {
    if (editMap[id]) return editMap[id]
    return {
      symbols: strategy.targetSymbols.join(','),
      quantity: strategy.orderQuantity,
      shortPeriod: (strategy.params.short_period as number) ?? 5,
      longPeriod: (strategy.params.long_period as number) ?? 20,
    }
  }

  const setEdit = (id: string, patch: Partial<{ symbols: string; quantity: number; shortPeriod: number; longPeriod: number }>) => {
    setEditMap((prev) => ({ ...prev, [id]: { ...getEdit(id, strategies!.find(s => s.id === id)!), ...patch } }))
  }

  const handleToggle = (id: string, enabled: boolean) => {
    updateStrategy({ id, enabled } satisfies UpdateStrategyInput)
  }

  const handleSave = (id: string) => {
    const edit = editMap[id]
    if (!edit) return
    const input: UpdateStrategyInput = {
      id,
      targetSymbols: edit.symbols.split(',').map(s => s.trim()).filter(Boolean),
      orderQuantity: edit.quantity,
      params: { short_period: edit.shortPeriod, long_period: edit.longPeriod },
    }
    updateStrategy(input, { onSuccess: () => setEditMap((prev) => { const n = { ...prev }; delete n[id]; return n }) })
  }

  const activeCount = strategies?.filter(s => s.enabled).length ?? 0
  const isRunning = tradingStatus?.isRunning ?? false

  if (isLoading) {
    return <Box sx={{ display: 'flex', justifyContent: 'center', pt: 8 }}><CircularProgress /></Box>
  }

  return (
    <Box>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 3 }}>
        <Typography variant="h5" fontWeight={700}>Strategy</Typography>
        <Chip
          label={`${activeCount}개 활성`}
          color={activeCount > 0 ? 'success' : 'default'}
          size="small"
        />
        {isRunning && (
          <Chip label="자동매매 실행 중" color="success" size="small" variant="outlined" />
        )}
      </Box>

      {/* ── 1. 전략 카드 ──────────────────────────────────────────── */}
      <Grid container spacing={2} sx={{ mb: 3 }}>
        {(strategies ?? []).map((s) => {
          const edit = getEdit(s.id, s)
          const isDirty = !!editMap[s.id]
          return (
            <Grid item xs={12} md={6} key={s.id}>
              <Paper sx={{ p: 3 }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}>
                  <Typography variant="subtitle1" fontWeight={600}>{s.name}</Typography>
                  <FormControlLabel
                    control={
                      <Switch
                        checked={s.enabled}
                        onChange={(e) => handleToggle(s.id, e.target.checked)}
                        color="success"
                        disabled={saving}
                      />
                    }
                    label={s.enabled ? '실행 중' : '정지'}
                    labelPlacement="start"
                  />
                </Box>
                <Divider sx={{ mb: 2 }} />

                <Stack spacing={2}>
                  <TextField
                    label="대상 종목코드 (쉼표 구분)"
                    value={edit.symbols}
                    onChange={(e) => setEdit(s.id, { symbols: e.target.value })}
                    size="small"
                    disabled={s.enabled}
                    helperText="예: 005930,035720"
                  />
                  <Grid container spacing={2}>
                    <Grid item xs={4}>
                      <TextField
                        label="1회 수량"
                        type="number"
                        value={edit.quantity}
                        onChange={(e) => setEdit(s.id, { quantity: Number(e.target.value) })}
                        size="small"
                        disabled={s.enabled}
                        inputProps={{ min: 1 }}
                      />
                    </Grid>
                    <Grid item xs={4}>
                      <TextField
                        label="단기 MA"
                        type="number"
                        value={edit.shortPeriod}
                        onChange={(e) => setEdit(s.id, { shortPeriod: Number(e.target.value) })}
                        size="small"
                        disabled={s.enabled}
                        inputProps={{ min: 2, max: 50 }}
                      />
                    </Grid>
                    <Grid item xs={4}>
                      <TextField
                        label="장기 MA"
                        type="number"
                        value={edit.longPeriod}
                        onChange={(e) => setEdit(s.id, { longPeriod: Number(e.target.value) })}
                        size="small"
                        disabled={s.enabled}
                        inputProps={{ min: 5, max: 200 }}
                      />
                    </Grid>
                  </Grid>
                </Stack>

                <Box sx={{ mt: 2, p: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
                  <Typography variant="caption" color="text.secondary">
                    단기 {edit.shortPeriod}MA가 장기 {edit.longPeriod}MA를 상향 돌파 시 매수 (골든크로스),
                    하향 돌파 시 매도 (데드크로스)
                  </Typography>
                </Box>

                {isDirty && !s.enabled && (
                  <Box sx={{ mt: 1.5 }}>
                    <Button
                      size="small"
                      variant="outlined"
                      startIcon={saving ? <CircularProgress size={14} /> : <SaveIcon />}
                      onClick={() => handleSave(s.id)}
                      disabled={saving}
                    >
                      변경사항 저장
                    </Button>
                  </Box>
                )}
              </Paper>
            </Grid>
          )
        })}
      </Grid>

      {/* ── 2. OrderManager: 리스크 관리 ─────────────────────────── */}
      <Paper sx={{ p: { xs: 2, sm: 3 }, mb: 2 }}>
        <Stack direction="row" alignItems="center" spacing={1} mb={1.5}>
          <Typography variant="subtitle1" fontWeight={600}>리스크 관리</Typography>
          <Tooltip
            title="일일 손실이 한도를 초과하거나, 종목 비중이 초과되면 주문이 자동으로 차단됩니다."
            arrow
          >
            <InfoOutlinedIcon fontSize="small" sx={{ color: 'text.disabled', cursor: 'pointer' }} />
          </Tooltip>
        </Stack>
        <Divider sx={{ mb: 2 }} />
        <RiskPanel />
      </Paper>

      {/* ── 3. OrderManager: 미체결 주문 ─────────────────────────── */}
      <Paper sx={{ p: { xs: 2, sm: 3 } }}>
        <Stack direction="row" alignItems="center" spacing={1} mb={1.5}>
          <Typography variant="subtitle1" fontWeight={600}>미체결 주문</Typography>
          <Tooltip
            title="자동 매매 엔진이 KIS API에 접수했으나 아직 체결되지 않은 주문 목록입니다. 5초마다 갱신됩니다."
            arrow
          >
            <InfoOutlinedIcon fontSize="small" sx={{ color: 'text.disabled', cursor: 'pointer' }} />
          </Tooltip>
        </Stack>
        <Divider sx={{ mb: 2 }} />
        <PendingOrdersPanel />
      </Paper>
    </Box>
  )
}
