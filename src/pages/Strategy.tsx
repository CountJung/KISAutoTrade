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
import InputAdornment from '@mui/material/InputAdornment'
import IconButton from '@mui/material/IconButton'
import SaveIcon from '@mui/icons-material/Save'
import WarningAmberIcon from '@mui/icons-material/WarningAmber'
import LockOpenIcon from '@mui/icons-material/LockOpen'
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined'
import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import SearchIcon from '@mui/icons-material/Search'
import RefreshIcon from '@mui/icons-material/Refresh'
import PublicIcon from '@mui/icons-material/Public'
import FlagIcon from '@mui/icons-material/Flag'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import { useState, useRef, useEffect } from 'react'
import {
  useStrategies,
  useUpdateStrategy,
  useTradingStatus,
  useRiskConfig,
  useUpdateRiskConfig,
  useClearEmergencyStop,
  usePendingOrders,
  useStockSearch,
  useRefreshStockList,
} from '../api/hooks'
import * as cmd from '../api/commands'
import type { CmdError, OverseasExchange, StockSearchItem, UpdateStrategyInput } from '../api/types'

type Market = 'KR' | 'US'

const EXCHANGE_SEARCH_ORDER: OverseasExchange[] = ['NAS', 'NYS', 'AMS']

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

// ─── 전략 타입별 파라미터 메타 ────────────────────────────────────
interface ParamMeta {
  key: string
  label: string
  min: number
  max: number
  step?: number
  description: string
}

const STRATEGY_PARAM_META: Record<string, ParamMeta[]> = {
  ma_cross: [
    { key: 'short_period', label: '단기 MA', min: 2, max: 50, description: '단기 이동평균 기간' },
    { key: 'long_period',  label: '장기 MA', min: 5, max: 200, description: '장기 이동평균 기간' },
  ],
  rsi: [
    { key: 'period',     label: 'RSI 기간',    min: 5,  max: 50,  description: 'RSI 계산 기간 (기본 14)' },
    { key: 'oversold',   label: '과매도 기준', min: 10, max: 40,  step: 1, description: 'RSI가 이 이하 → 이 이상 시 매수 신호 (기본 30)' },
    { key: 'overbought', label: '과매수 기준', min: 60, max: 90,  step: 1, description: 'RSI가 이 이상 → 이 이하 시 매도 신호 (기본 70)' },
  ],
  momentum: [
    { key: 'lookback_period', label: '비교 기간',    min: 5,  max: 60, description: 'N기간 전 가격 대비 변화율 계산 기간 (기본 20)' },
    { key: 'threshold_pct',   label: '임계값 (%)', min: 1,  max: 20, step: 0.5, description: '매매 발동 최소 변화율 % (기본 5.0)' },
  ],
  deviation: [
    { key: 'ma_period',          label: 'MA 기간',       min: 5,   max: 120, description: '이격도 기준 이동평균 기간 (기본 20)' },
    { key: 'buy_threshold_pct',  label: '매수 이격 (%)', min: -20, max: -1,  step: 0.5, description: '현재가가 MA 대비 이 % 이하이면 매수 (기본 -5.0, 음수)' },
    { key: 'sell_threshold_pct', label: '매도 이격 (%)', min: 1,   max: 20,  step: 0.5, description: '현재가가 MA 대비 이 % 이상이면 매도 (기본 5.0)' },
  ],
  fifty_two_week_high: [
    { key: 'lookback_days', label: '조회 기간 (거래일)', min: 60, max: 504, step: 1, description: '52주 신고가 계산을 위한 과거 거래일 수 (기본 252 ≈ 1년)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)',      min: 1,  max: 15,  step: 0.5, description: '매수가 대비 이 % 이상 하락 시 손절 매도 (기본 3.0)' },
  ],
  consecutive_move: [
    { key: 'buy_days',  label: '연속 상승 횟수', min: 2, max: 10, step: 1, description: 'N일 연속 종가 상승 시 매수 (기본 3)' },
    { key: 'sell_days', label: '연속 하락 횟수', min: 2, max: 10, step: 1, description: 'M일 연속 종가 하락 시 매도 (기본 3)' },
  ],
  failed_breakout: [
    { key: 'lookback_days', label: '전고점 기간', min: 5, max: 60, step: 1, description: '전고점 계산을 위한 과거 기간 (기본 20)' },
    { key: 'buffer_pct',    label: '돌파 버퍼 (%)', min: 0.1, max: 5.0, step: 0.1, description: '전고점 대비 돌파로 인정하는 추가 % (기본 0.5)' },
  ],
  strong_close: [
    { key: 'threshold_pct', label: '강한 종가 기준 (%)', min: 0.5, max: 10.0, step: 0.5, description: '종가가 고가 대비 이 % 이내이면 실개로 강한 종가로 판단 (기본 3.0)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 1.0, max: 10.0, step: 0.5, description: '매수가 대비 이 % 이상 하락 시 손절 (기본 3.0)' },
  ],
  volatility_expansion: [
    { key: 'lookback_days',     label: '평균 기간 (거래일)', min: 3, max: 60,  step: 1,   description: '평균 변동폭 계산에 사용할 과거 거래일 수 (기본 10)' },
    { key: 'expansion_factor',  label: '확장 배율',          min: 1.1, max: 5.0, step: 0.1, description: '당일 변동폭이 평균의 이 배 이상이면 매수 (기본 2.0)' },
    { key: 'stop_loss_pct',     label: '손절 기준 (%)',       min: 1.0, max: 10.0, step: 0.5, description: '매수가 대비 이 % 이상 하락 시 손절 (기본 3.0)' },
  ],
  mean_reversion: [
    { key: 'period',        label: '볼린저 밴드 기간', min: 5,   max: 120, step: 1,   description: '이동평균과 표준편차 계산 기간 (기본 20)' },
    { key: 'std_dev',       label: '표준편차 배율',       min: 0.5, max: 4.0, step: 0.5, description: '상/하단 밴드 너비 조정 (기본 2.0 = ±2시그마)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)',             min: 1.0, max: 15.0, step: 0.5, description: '매수가 대비 이 % 이상 하락 시 손절 (기본 5.0)' },
  ],
}

const STRATEGY_DESCRIPTION: Record<string, string> = {
  ma_cross:              '단기 MA가 장기 MA를 상향 돌파(골든크로스) 시 매수, 하향 돌파(데드크로스) 시 매도.',
  rsi:                   'RSI가 과매도 기준 이하에서 반등하면 매수, 과매수 기준 이상에서 하락하면 매도.',
  momentum:              'N기간 전 가격 대비 현재가 변화율이 임계값 이상이면 매수, 이하이면 매도 (추세 추종).',
  deviation:             '현재가가 이동평균 대비 일정 % 이하이면 매수(저평가), 일정 % 이상이면 매도(고평가).',
  fifty_two_week_high:   '최근 252 거래일(1년) 최고가를 재돌파하면 매수. 매수 후 지정 % 하락 시 손절. 자동매매 시작 시 KIS 차트 API로 초기화됨.',
  consecutive_move:      'N일 연속 종가 상승 시 매수, M일 연속 하락 시 매도. 추세 과쟥에 상승/하락할 때 조기에 편승하는 추세추종 전략.',
  failed_breakout:       '최근 N일 전고점을 버퍼% 이상 돌파하면 매수. 이후 가격이 전고점 이하로 내려오면 돌파 실패로 판단하여 즉시 매도.',
  strong_close:          '자동매매 시작 시 전일 종가가 고가 대비 N% 이내여서 강하게 마감하면 당일 첫 틱에 매수. 매수 후 지정 % 하락 시 손절.',
  volatility_expansion:  '당일 변동폭(고-저)이 최근 N거래일 평균 변동폭의 K배 이상이며 현재가 > 시가인 경우 매수. 장중 변동성 폭발 구간에 상승 방향으로 편승. 매수 후 지정 % 하락 시 손절.',
  mean_reversion:        '현재가가 볼린저 밴드 하단(mean - Nσ) 아래로 바운스하면 매수(과매도). 현재가 상단 밴드 돌파 시 익절 매도, 손절 기준 % 이상 하락 시 손절. 자동매매 시작 시 과거 N일 종가로 버퍼 적재.',
}

function getStrategyType(id: string): string {
  if (id.startsWith('ma_cross'))             return 'ma_cross'
  if (id.startsWith('rsi'))                  return 'rsi'
  if (id.startsWith('momentum'))             return 'momentum'
  if (id.startsWith('deviation'))            return 'deviation'
  if (id.startsWith('fifty_two_week_high'))  return 'fifty_two_week_high'
  if (id.startsWith('consecutive_move'))     return 'consecutive_move'
  if (id.startsWith('failed_breakout'))      return 'failed_breakout'
  if (id.startsWith('strong_close'))             return 'strong_close'
  if (id.startsWith('volatility_expansion'))     return 'volatility_expansion'
  if (id.startsWith('mean_reversion'))            return 'mean_reversion'
  return 'unknown'
}

type EditState = { symbols: string[]; quantity: number; params: Record<string, number> }

// ─── Strategy 메인 ────────────────────────────────────────────────
export default function Strategy() {
  const { data: strategies, isLoading } = useStrategies()
  const { data: tradingStatus } = useTradingStatus()
  const { mutate: updateStrategy, isPending: saving } = useUpdateStrategy()

  const [editMap, setEditMap] = useState<Record<string, EditState>>({})

  // ── 상단 종목 검색 상태 ─────────────────────────────────────────
  const [market, setMarket]                   = useState<Market>('KR')
  const [selectedStock, setSelectedStock]     = useState<StockSearchItem | null>(null)
  const [searchInput, setSearchInput]         = useState('')
  const [searchQuery, setSearchQuery]         = useState('')
  const [showSearch, setShowSearch]           = useState(false)
  const searchCloseTimer                      = useRef<ReturnType<typeof setTimeout> | null>(null)
  const [usSearching, setUsSearching]         = useState(false)
  const [usSearchError, setUsSearchError]     = useState<string | null>(null)
  /** 종목코드 → 이름 캐시 (세션 중 검색으로 추가된 종목) */
  const [symbolNames, setSymbolNames]         = useState<Record<string, string>>({})

  const { data: searchResults = [], isFetching: isFetchingSearch,
          isError: isSearchError, error: searchError }  = useStockSearch(searchQuery)
  const { mutate: doRefreshList, isPending: isRefreshing } = useRefreshStockList()
  const isStockListEmpty = isSearchError && (searchError as CmdError | null)?.code === 'STOCK_LIST_EMPTY'

  // 검색어 디바운스 — 6자리 영숫자 코드만 검색 허용 (0005A0, 0089C0 등 ETF 포함)
  useEffect(() => {
    if (!searchInput || !showSearch) { setSearchQuery(''); return }
    if (/^[A-Z0-9]{6}$/i.test(searchInput)) {
      setSearchQuery(searchInput.toUpperCase())
      return
    }
    // 6자 미만이면 대기, 그 외(6자 초과 등)는 무시
    setSearchQuery('')
  }, [searchInput, showSearch])
  /** 해외(미국) 거래소 자동 감지: NAS → NYS → AMS 순서로 조회 */
  const handleUsSearch = async () => {
    const ticker = searchInput.trim().toUpperCase()
    if (!ticker) return
    setUsSearching(true)
    setUsSearchError(null)
    for (const exc of EXCHANGE_SEARCH_ORDER) {
      try {
        const res = await cmd.getOverseasPrice(ticker, exc)
        // 가격 또는 종목명이 있으면 유효한 종목으로 간주
        const validPrice = parseFloat(res.last) > 0
        const hasName = res.name && res.name.trim().length > 0
        if (res && (validPrice || hasName)) {
          const item: StockSearchItem = { pdno: ticker, prdt_name: res.name || ticker }
          setSelectedStock(item)
          setSearchInput(res.name || ticker)
          setSymbolNames(prev => ({ ...prev, [ticker]: res.name || ticker }))
          setUsSearching(false)
          return
        }
      } catch { /* 다음 거래소 시도 */ }
    }
    setUsSearchError(`"${ticker}"을 NAS·NYS·AMEX에서 찾을 수 없습니다.`)
    setUsSearching(false)
  }
  // 전략이 로드됐을 때 기존 symbolNames에 없는 종목 이름 등록 (strategies 데이터에서)
  useEffect(() => {
    if (!strategies) return
    // targetSymbolNames는 없으므로 code 그대로 사용 (이름은 검색 시 채워짐)
    // 단, strategies에 name 정보가 있으면 활용
  }, [strategies])

  // 시장 변경 시 검색 상태 초기화
  useEffect(() => {
    setSelectedStock(null)
    setSearchInput('')
    setSearchQuery('')
    setShowSearch(false)
    setUsSearchError(null)
  }, [market])

  const getEdit = (id: string, s: { targetSymbols: string[]; orderQuantity: number; params: Record<string, unknown> }): EditState => {
    if (editMap[id]) return editMap[id]
    const numericParams: Record<string, number> = {}
    for (const [k, v] of Object.entries(s.params)) {
      numericParams[k] = typeof v === 'number' ? v : Number(v)
    }
    return { symbols: s.targetSymbols, quantity: s.orderQuantity, params: numericParams }
  }

  const setEdit = (id: string, s: { targetSymbols: string[]; orderQuantity: number; params: Record<string, unknown> }, patch: Partial<EditState>) =>
    setEditMap(prev => ({ ...prev, [id]: { ...getEdit(id, s), ...patch } }))

  const setParam = (id: string, s: { targetSymbols: string[]; orderQuantity: number; params: Record<string, unknown> }, key: string, val: number) => {
    const cur = getEdit(id, s)
    setEditMap(prev => ({ ...prev, [id]: { ...cur, params: { ...cur.params, [key]: val } } }))
  }

  const addSymbol = (stratId: string, s: { targetSymbols: string[]; orderQuantity: number; params: Record<string, unknown> }, stock: StockSearchItem) => {
    const cur = getEdit(stratId, s)
    if (cur.symbols.includes(stock.pdno)) return
    setEdit(stratId, s, { symbols: [...cur.symbols, stock.pdno] })
    setSymbolNames(prev => ({ ...prev, [stock.pdno]: stock.prdt_name }))
  }

  const removeSymbol = (stratId: string, s: { targetSymbols: string[]; orderQuantity: number; params: Record<string, unknown> }, pdno: string) => {
    const cur = getEdit(stratId, s)
    setEdit(stratId, s, { symbols: cur.symbols.filter(c => c !== pdno) })
  }

  const handleToggle = (id: string, enabled: boolean) => {
    updateStrategy({ id, enabled } satisfies UpdateStrategyInput)
  }

  const handleSave = (id: string) => {
    const edit = editMap[id]
    if (!edit) return
    updateStrategy(
      {
        id,
        targetSymbols: edit.symbols,
        orderQuantity: edit.quantity,
        params: edit.params,
      } satisfies UpdateStrategyInput,
      { onSuccess: () => setEditMap(prev => { const n = { ...prev }; delete n[id]; return n }) },
    )
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

      {/* ── 0. 종목 선택 패널 ─────────────────────────────────────── */}
      <Paper sx={{ p: { xs: 1.5, sm: 2.5 }, mb: 2 }}>
        <Typography variant="subtitle1" fontWeight={600} mb={0.5}>종목 선택</Typography>
        <Typography variant="caption" color="text.secondary" display="block" mb={1}>
          종목을 검색하여 선택한 후, 각 전략 카드의 "추가" 버튼으로 전략에 등록하세요
        </Typography>

        {/* 시장 토글 */}
        <ToggleButtonGroup
          value={market}
          exclusive
          onChange={(_, v) => { if (v) setMarket(v as Market) }}
          size="small"
          sx={{ mb: 1.5 }}
        >
          <ToggleButton value="KR">
            <FlagIcon fontSize="small" sx={{ mr: 0.5 }} />국내
          </ToggleButton>
          <ToggleButton value="US">
            <PublicIcon fontSize="small" sx={{ mr: 0.5 }} />해외 (미국)
          </ToggleButton>
        </ToggleButtonGroup>

        <Box sx={{ position: 'relative' }}>
          {market === 'KR' ? (
            <>
              <TextField
                label="6자리 종목코드 (예: 005930, 0005A0)"
                value={searchInput}
                onChange={(e) => {
                  setSearchInput(e.target.value)
                  setShowSearch(true)
                  if (!e.target.value) { setSelectedStock(null); setShowSearch(false) }
                }}
                onBlur={() => { searchCloseTimer.current = setTimeout(() => setShowSearch(false), 180) }}
                onFocus={() => {
                  if (searchCloseTimer.current) clearTimeout(searchCloseTimer.current)
                  if (searchInput.length >= 2) setShowSearch(true)
                }}
                size="small"
                fullWidth
                InputProps={{
                  endAdornment: (
                    <InputAdornment position="end">
                      {isFetchingSearch && <CircularProgress size={14} color="inherit" sx={{ mr: 0.5 }} />}
                      <IconButton
                        size="small"
                        disabled={!searchInput || searchInput.length < 2}
                        onClick={() => { if (searchInput.length >= 2) { setSearchQuery(searchInput); setShowSearch(true) } }}
                      >
                        <SearchIcon fontSize="small" />
                      </IconButton>
                    </InputAdornment>
                  ),
                }}
                helperText={
                  selectedStock
                    ? `선택됨: ${selectedStock.prdt_name} (${selectedStock.pdno})`
                    : '국내 주식은 6자리 종목코드로만 검색 가능합니다 (예: 005930, 0005A0)'
                }
              />
              {/* 검색 결과 드롭다운 */}
              {showSearch && (searchResults.length > 0 || isFetchingSearch) && (
                <Paper
                  elevation={8}
                  onMouseDown={(e) => { e.preventDefault(); if (searchCloseTimer.current) clearTimeout(searchCloseTimer.current) }}
                  sx={{ mt: 0.5, maxHeight: 240, overflow: 'auto', border: 1, borderColor: 'divider', zIndex: 1400, position: 'absolute', width: '100%' }}
                >
                  {isFetchingSearch && searchResults.length === 0 ? (
                    <Box sx={{ p: 1.5, display: 'flex', alignItems: 'center', gap: 1 }}>
                      <CircularProgress size={14} />
                      <Typography variant="caption" color="text.secondary">검색 중...</Typography>
                    </Box>
                  ) : (
                    <Table size="small">
                      <TableBody>
                        {searchResults.map((r) => (
                          <TableRow
                            key={r.pdno}
                            hover
                            sx={{ cursor: 'pointer' }}
                            onClick={() => {
                              setSelectedStock(r)
                              setSearchInput(r.prdt_name)
                              setShowSearch(false)
                              setSearchQuery('')
                              setSymbolNames(prev => ({ ...prev, [r.pdno]: r.prdt_name }))
                            }}
                          >
                            <TableCell sx={{ py: 0.75 }}>
                              <Typography variant="body2" noWrap>{r.prdt_name}</Typography>
                            </TableCell>
                            <TableCell sx={{ py: 0.75, width: 80 }}>
                              <Typography variant="caption" color="text.secondary">{r.pdno}</Typography>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  )}
                </Paper>
              )}
              {/* STOCK_LIST_EMPTY 경고 */}
              {showSearch && isStockListEmpty && (
                <Alert
                  severity="warning"
                  sx={{ mt: 0.5 }}
                  action={
                    <Button
                      size="small"
                      color="warning"
                      variant="outlined"
                      onClick={() => doRefreshList()}
                      disabled={isRefreshing}
                      startIcon={isRefreshing ? <CircularProgress size={12} /> : <RefreshIcon fontSize="small" />}
                    >
                      {isRefreshing ? '다운로드 중...' : '종목 목록 새로고침'}
                    </Button>
                  }
                >
                  <Typography variant="caption">종목 목록이 로드되지 않았습니다. 새로고침을 눌러주세요.</Typography>
                </Alert>
              )}
            </>
          ) : (
            /* ── 해외(미국) 주식 검색 ─── */
            <Stack spacing={1}>
              <Stack direction="row" spacing={1}>
                <TextField
                  label="티커 (예: AAPL, SPYM, QQQ)"
                  value={searchInput}
                  onChange={(e) => {
                    setSearchInput(e.target.value.toUpperCase())
                    setUsSearchError(null)
                    setSelectedStock(null)
                  }}
                  onKeyDown={(e) => { if (e.key === 'Enter') handleUsSearch() }}
                  size="small"
                  fullWidth
                  inputProps={{ style: { textTransform: 'uppercase' } }}
                  helperText={
                    selectedStock
                      ? `선택됨: ${selectedStock.prdt_name} (${selectedStock.pdno})`
                      : 'NAS(NASDAQ)·NYS·AMEX 순서로 자동 감지합니다'
                  }
                />
                <Button
                  variant="contained"
                  size="small"
                  onClick={handleUsSearch}
                  disabled={!searchInput.trim() || usSearching}
                  startIcon={usSearching ? <CircularProgress size={14} color="inherit" /> : <SearchIcon />}
                  sx={{ minWidth: 80, alignSelf: 'flex-start', mt: 0.5 }}
                >
                  검색
                </Button>
              </Stack>
              {usSearchError && (
                <Alert severity="warning" sx={{ py: 0.5 }}>
                  <Typography variant="caption">{usSearchError}</Typography>
                </Alert>
              )}
            </Stack>
          )}

          {/* 선택된 종목 칩 */}
          {selectedStock && (
            <Box sx={{ mt: 1 }}>
              <Chip
                label={`${selectedStock.prdt_name} (${selectedStock.pdno})`}
                onDelete={() => { setSelectedStock(null); setSearchInput('') }}
                color="primary"
                size="small"
              />
            </Box>
          )}
        </Box>
      </Paper>

      {/* ── 1. 전략 카드 ──────────────────────────────────────────── */}
      <Grid container spacing={2} sx={{ mb: 3 }}>
        {(strategies ?? []).map((s) => {
          const edit = getEdit(s.id, s)
          const isDirty = !!editMap[s.id]
          const sType = getStrategyType(s.id)
          const paramMetas = STRATEGY_PARAM_META[sType] ?? []
          const stratDesc = STRATEGY_DESCRIPTION[sType]
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
                  {/* 종목 목록 테이블 */}
                  <Box>
                    <Stack direction="row" alignItems="center" justifyContent="space-between" mb={0.5}>
                      <Typography variant="caption" color="text.secondary" fontWeight={600}>
                        대상 종목 ({edit.symbols.length}개)
                      </Typography>
                      <Button
                        size="small"
                        variant="outlined"
                        startIcon={<AddIcon />}
                        disabled={!selectedStock || s.enabled || edit.symbols.includes(selectedStock?.pdno ?? '')}
                        onClick={() => selectedStock && addSymbol(s.id, s, selectedStock)}
                        sx={{ fontSize: '0.7rem', py: 0.3 }}
                      >
                        {selectedStock ? `${selectedStock.prdt_name} 추가` : '위에서 종목 선택'}
                      </Button>
                    </Stack>
                    {edit.symbols.length === 0 ? (
                      <Typography variant="caption" color="text.disabled" sx={{ pl: 0.5 }}>
                        추가된 종목이 없습니다
                      </Typography>
                    ) : (
                      <TableContainer sx={{ maxHeight: 160, border: 1, borderColor: 'divider', borderRadius: 1 }}>
                        <Table size="small">
                          <TableHead>
                            <TableRow>
                              <TableCell sx={{ py: 0.5, fontSize: '0.7rem' }}>코드</TableCell>
                              <TableCell sx={{ py: 0.5, fontSize: '0.7rem' }}>종목명</TableCell>
                              <TableCell sx={{ py: 0.5, width: 36 }} />
                            </TableRow>
                          </TableHead>
                          <TableBody>
                            {edit.symbols.map((code) => (
                              <TableRow key={code}>
                                <TableCell sx={{ py: 0.5 }}>
                                  <Typography variant="caption">{code}</Typography>
                                </TableCell>
                                <TableCell sx={{ py: 0.5 }}>
                                  <Typography variant="caption" noWrap>
                                    {symbolNames[code] ?? '—'}
                                  </Typography>
                                </TableCell>
                                <TableCell sx={{ py: 0.5 }}>
                                  <IconButton
                                    size="small"
                                    disabled={s.enabled}
                                    onClick={() => removeSymbol(s.id, s, code)}
                                  >
                                    <DeleteIcon sx={{ fontSize: 14 }} />
                                  </IconButton>
                                </TableCell>
                              </TableRow>
                            ))}
                          </TableBody>
                        </Table>
                      </TableContainer>
                    )}
                  </Box>
                  <Grid container spacing={2}>
                    <Grid item xs={4}>
                      <TextField
                        label="1회 수량"
                        type="number"
                        value={edit.quantity}
                        onChange={(e) => setEdit(s.id, s, { quantity: Number(e.target.value) })}
                        size="small"
                        disabled={s.enabled}
                        inputProps={{ min: 1 }}
                      />
                    </Grid>
                    {paramMetas.map((meta) => (
                      <Grid item xs={4} key={meta.key}>
                        <TextField
                          label={meta.label}
                          type="number"
                          title={meta.description}
                          value={edit.params[meta.key] ?? 0}
                          onChange={(e) => setParam(s.id, s, meta.key, Number(e.target.value))}
                          size="small"
                          disabled={s.enabled}
                          inputProps={{ min: meta.min, max: meta.max, step: meta.step ?? 1 }}
                        />
                      </Grid>
                    ))}
                  </Grid>
                </Stack>

                {stratDesc && (
                  <Box sx={{ mt: 2, p: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
                    <Typography variant="caption" color="text.secondary">
                      {stratDesc}
                    </Typography>
                  </Box>
                )}

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
