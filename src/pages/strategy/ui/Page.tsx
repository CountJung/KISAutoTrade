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
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import InputAdornment from '@mui/material/InputAdornment'
import IconButton from '@mui/material/IconButton'
import SaveIcon from '@mui/icons-material/Save'
import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import SearchIcon from '@mui/icons-material/Search'
import RefreshIcon from '@mui/icons-material/Refresh'
import PublicIcon from '@mui/icons-material/Public'
import FlagIcon from '@mui/icons-material/Flag'
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Tooltip from '@mui/material/Tooltip'
import { useState, useRef, useEffect } from 'react'
import {
  useStrategies,
  useUpdateStrategy,
  useTradingStatus,
  useStockSearch,
  useRefreshStockList,
} from '../../../api/hooks'
import * as cmd from '../../../api/commands'
import type {
  CmdError,
  LeveragedTrendHoldBaseRole,
  LeveragedTrendHoldEntry,
  OverseasExchange,
  PriceConditionSymbolConfig,
  StockSearchItem,
  UpdateStrategyInput,
} from '../../../api/types'

type Market = 'KR' | 'US'
type LeveragedSetDraftSlot = 'base' | 'long' | 'inverse'
type LeveragedSetDraftSelection = {
  stock: StockSearchItem
  market: Market
}

const EXCHANGE_SEARCH_ORDER: OverseasExchange[] = ['NAS', 'NYS', 'AMS']

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
  trend_filter: [
    { key: 'long_period',  label: '장기 MA 기간',  min: 50,  max: 500, step: 1, description: '장기 추세 판단 기준 이동평균 기간 (기본 200일)' },
    { key: 'short_period', label: '단기 MA 기간',  min: 2,   max: 30,  step: 1, description: '단기 모멘텀 판단 이동평균 기간 (기본 5일)' },
    { key: 'mid_period',   label: '중기 MA 기간',  min: 5,   max: 60,  step: 1, description: '중기 추세 비교 이동평균 기간 (기본 20일)' },
  ],
  // price_condition은 커스텀 편집 UI로 처리하므로 STRATEGY_PARAM_META 제외
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
  trend_filter:          '장기 MA(기본 200일) 위에서 단기 MA(5일)가 중기 MA(20일)를 상회할 때만 매수(이중 추세 확인). 현재가가 장기 MA 아래로 하락하면 추세 반전으로 판단하여 청산. 자동매매 시작 시 과거 종가로 버퍼 적재.',
  leveraged_trend_hold:  'SOXX/SMH 같은 기초 ETF의 상승 추세에서는 정방향 레버리지를, 하락 추세에서는 선택한 역방향 레버리지를 보유한다. 고점 대비 하락, 기초 추세 이탈, RSI 반전, 장마감 전에는 청산한다.',
  price_condition:        '지정가 이하에서 자동 매수. 매수 후 지정가 또는 설정 % 이상 상승 시 익절 매도, 손절 % 이하 하락 시 손절. 가격/비율 조건을 각각 설정하거나 조합해서 사용 가능. 0은 해당 조건 비활성.',
}

const LTH_BASE_ROLE_LABEL: Record<LeveragedTrendHoldBaseRole, string> = {
  underlying: '기초',
  proxy: '유사',
}

const LTH_DRAFT_SLOT_LABEL: Record<LeveragedSetDraftSlot, string> = {
  base: '기초지수',
  long: '롱 ETF',
  inverse: '숏 ETF',
}

function getLthBaseRole(entry: LeveragedTrendHoldEntry, base: string): LeveragedTrendHoldBaseRole {
  return entry.base_symbol_roles?.[base] ?? 'underlying'
}

function hasInvalidLthEntries(entries: LeveragedTrendHoldEntry[]): boolean {
  return entries.some((entry) => !entry.leveraged_symbol || entry.base_symbols.length === 0)
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
  if (id.startsWith('trend_filter'))              return 'trend_filter'
  if (id.startsWith('leveraged_trend_hold'))       return 'leveraged_trend_hold'
  if (id.startsWith('price_condition'))            return 'price_condition'
  return 'unknown'
}

// ─── 가격 조건 매매 커스텀 편집 UI ─────────────────────────────
function PriceConditionEditorPanel({
  stratEnabled,
  initialSymbols,
  editedSymbols,
  selectedStock,
  market,
  onUpdate,
}: {
  stratEnabled: boolean
  initialSymbols: PriceConditionSymbolConfig[]
  editedSymbols: PriceConditionSymbolConfig[] | undefined
  selectedStock: StockSearchItem | null
  market: Market
  onUpdate: (symbols: PriceConditionSymbolConfig[]) => void
}) {
  const symbols = editedSymbols ?? initialSymbols

  const handleAdd = () => {
    if (!selectedStock || symbols.some((s) => s.symbol === selectedStock.pdno)) return
    onUpdate([
      ...symbols,
      {
        symbol: selectedStock.pdno,
        symbol_name: selectedStock.prdt_name,
        quantity: 1,
        buy_trigger_price: 0,
        sell_trigger_price: 0,
        take_profit_pct: 5,
        stop_loss_pct: 3,
        // 현재 시장 선택 기준으로 is_overseas 자동 설정
        is_overseas: market === 'US',
      },
    ])
  }

  const handleRemove = (sym: string) => {
    onUpdate(symbols.filter((s) => s.symbol !== sym))
  }

  const handleFieldChange = (
    sym: string,
    field: keyof PriceConditionSymbolConfig,
    value: number | boolean,
  ) => {
    onUpdate(symbols.map((s) => (s.symbol === sym ? { ...s, [field]: value } : s)))
  }

  // 고정 컬럼 정의 (가격 컬럼은 is_overseas에 따라 step/단위가 달라짐)
  const numFields: { key: keyof PriceConditionSymbolConfig; label: string; isPrice: boolean }[] = [
    { key: 'quantity',           label: '수량',   isPrice: false },
    { key: 'buy_trigger_price',  label: '매수가', isPrice: true  },
    { key: 'sell_trigger_price', label: '익절가', isPrice: true  },
    { key: 'take_profit_pct',    label: '익절%',  isPrice: false },
    { key: 'stop_loss_pct',      label: '손절%',  isPrice: false },
  ]

  return (
    <Stack spacing={1.5}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" flexWrap="wrap" gap={1}>
        <Stack direction="row" alignItems="center" gap={0.5}>
          <Typography variant="caption" color="text.secondary" fontWeight={600}>
            대상 종목 ({symbols.length}개)
          </Typography>
          <Tooltip
            title="종목별로 매수가·익절가·손절%·익절%를 독립 설정. 매수가 ≤ 현재가이면 매수, 매수 후 지정가/비율 조건 달성 시 매도. 0은 해당 조건 비활성."
            arrow
          >
            <InfoOutlinedIcon sx={{ fontSize: 13, color: 'text.disabled', cursor: 'help' }} />
          </Tooltip>
        </Stack>
        <Button
          size="small"
          variant="outlined"
          startIcon={<AddIcon />}
          disabled={!selectedStock || stratEnabled || symbols.some((s) => s.symbol === (selectedStock?.pdno ?? ''))}
          onClick={handleAdd}
          sx={{ fontSize: '0.7rem', py: 0.3 }}
        >
          {selectedStock ? `${selectedStock.prdt_name} 추가` : '위에서 종목 선택'}
        </Button>
      </Stack>

      {symbols.length === 0 ? (
        <Typography variant="caption" color="text.disabled" sx={{ pl: 0.5 }}>
          추가된 종목이 없습니다
        </Typography>
      ) : (
        <TableContainer sx={{ border: 1, borderColor: 'divider', borderRadius: 1, overflowX: 'auto' }}>
          <Table size="small" sx={{ minWidth: 750 }}>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 110 }}>종목</TableCell>
                {numFields.map((f) => (
                  <TableCell
                    key={f.key}
                    sx={{ fontSize: '0.7rem', py: 0.75, minWidth: f.isPrice ? 130 : 90 }}
                    align="center"
                  >
                    {f.isPrice
                      ? `${f.label}(원/$)`
                      : f.key === 'quantity' ? f.label : `${f.label}`}
                  </TableCell>
                ))}
                <TableCell sx={{ width: 36, py: 0.75 }} />
              </TableRow>
            </TableHead>
            <TableBody>
              {symbols.map((sym) => (
                <TableRow key={sym.symbol}>
                  <TableCell sx={{ py: 0.5 }}>
                    <Stack direction="row" alignItems="center" gap={0.5}>
                      {sym.is_overseas && (
                        <Typography variant="caption" color="primary.main" fontWeight={700} sx={{ fontSize: '0.6rem' }}>$</Typography>
                      )}
                      <Box>
                        <Typography variant="caption" fontWeight={600}>{sym.symbol}</Typography>
                        <Typography variant="caption" color="text.secondary" display="block" noWrap sx={{ maxWidth: 80 }}>
                          {sym.symbol_name}
                        </Typography>
                      </Box>
                    </Stack>
                  </TableCell>
                  {numFields.map((f) => {
                    const step = f.isPrice ? (sym.is_overseas ? 0.01 : 100) : 0.5
                    const fieldStep = f.key === 'quantity' ? 1 : step
                    return (
                      <TableCell key={f.key} sx={{ py: 0.25 }} align="center">
                        <TextField
                          type="number"
                          value={sym[f.key] as number}
                          disabled={stratEnabled}
                          size="small"
                          onChange={(e) => handleFieldChange(sym.symbol, f.key, Number(e.target.value))}
                          inputProps={{
                            min: f.key === 'quantity' ? 1 : 0,
                            step: fieldStep,
                            style: { padding: '4px 4px', fontSize: '0.75rem', textAlign: 'right' },
                          }}
                          InputProps={f.isPrice ? {
                            endAdornment: (
                              <InputAdornment position="end">
                                <Typography variant="caption" color="text.secondary" sx={{ fontSize: '0.65rem', lineHeight: 1 }}>
                                  {sym.is_overseas ? '$' : '원'}
                                </Typography>
                              </InputAdornment>
                            ),
                          } : undefined}
                          sx={{ width: '100%', '& .MuiInputBase-root': { fontSize: '0.75rem' } }}
                        />
                      </TableCell>
                    )
                  })}
                  <TableCell sx={{ py: 0.5 }}>
                    <IconButton size="small" disabled={stratEnabled} onClick={() => handleRemove(sym.symbol)}>
                      <DeleteIcon sx={{ fontSize: 14 }} />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Stack>
  )
}

// ─── 레버리지 추세 보유 커스텀 편집 UI ───────────────────────────
function LeveragedTrendHoldEditorPanel({
  stratEnabled,
  initialEntries,
  editedEntries,
  onUpdate,
}: {
  stratEnabled: boolean
  initialEntries: LeveragedTrendHoldEntry[]
  editedEntries: LeveragedTrendHoldEntry[] | undefined
  onUpdate: (entries: LeveragedTrendHoldEntry[]) => void
}) {
  const entries = editedEntries ?? initialEntries
  const [pickerMarket, setPickerMarket] = useState<Market>('US')
  const [pickerInput, setPickerInput] = useState('')
  const [pickerQuery, setPickerQuery] = useState('')
  const [pickerOpen, setPickerOpen] = useState(false)
  const [pickerStock, setPickerStock] = useState<StockSearchItem | null>(null)
  const pickerCloseTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const [pickerSearching, setPickerSearching] = useState(false)
  const [pickerError, setPickerError] = useState<string | null>(null)
  const [draftSlot, setDraftSlot] = useState<LeveragedSetDraftSlot>('base')
  const [draftBase, setDraftBase] = useState<LeveragedSetDraftSelection | null>(null)
  const [draftLong, setDraftLong] = useState<LeveragedSetDraftSelection | null>(null)
  const [draftInverse, setDraftInverse] = useState<LeveragedSetDraftSelection | null>(null)
  const [draftBaseRole, setDraftBaseRole] = useState<LeveragedTrendHoldBaseRole>('underlying')
  const [draftQuantity, setDraftQuantity] = useState(1)
  const [draftInverseQuantity, setDraftInverseQuantity] = useState(1)
  const { mutate: doPickerRefreshList, isPending: pickerRefreshing } = useRefreshStockList()
  const selectedIsOverseas = pickerMarket === 'US'
  const draftMarket = draftLong?.market ?? draftBase?.market ?? draftInverse?.market ?? null
  const invalidEntries = entries.filter((entry) => entry.base_symbols.length === 0)
  const entriesWithoutInverse = entries.filter((entry) => !entry.inverse_leveraged_symbol)
  const {
    data: pickerResults = [],
    isFetching: pickerFetching,
    isError: pickerIsError,
    error: pickerSearchError,
  } = useStockSearch(pickerMarket === 'KR' ? pickerQuery : '')
  const pickerStockListEmpty = pickerIsError && (pickerSearchError as CmdError | null)?.code === 'STOCK_LIST_EMPTY'

  const isSelectionCompatible = (entry: LeveragedTrendHoldEntry) =>
    !!pickerStock && entry.is_overseas === selectedIsOverseas
  const canAddDraftSet =
    !!draftBase &&
    !!draftLong &&
    draftBase.market === draftLong.market &&
    (!draftInverse || draftInverse.market === draftLong.market) &&
    draftBase.stock.pdno !== draftLong.stock.pdno &&
    (!draftInverse || (
      draftInverse.stock.pdno !== draftBase.stock.pdno &&
      draftInverse.stock.pdno !== draftLong.stock.pdno
    )) &&
    !entries.some((entry) => entry.leveraged_symbol === draftLong.stock.pdno)

  useEffect(() => {
    if (pickerMarket !== 'KR' || !pickerInput || !pickerOpen) {
      setPickerQuery('')
      return
    }
    const timer = setTimeout(() => setPickerQuery(pickerInput), 250)
    return () => clearTimeout(timer)
  }, [pickerInput, pickerMarket, pickerOpen])

  useEffect(() => {
    setPickerInput('')
    setPickerQuery('')
    setPickerOpen(false)
    setPickerStock(null)
    setPickerError(null)
  }, [pickerMarket])

  const handlePickerSelect = (stock: StockSearchItem) => {
    if (draftMarket && draftMarket !== pickerMarket) {
      setPickerError('한 세트에는 같은 시장의 ETF만 넣을 수 있습니다. 시장을 바꾸려면 현재 세트 초안을 먼저 비우세요.')
      return
    }
    setPickerStock(stock)
    const selection = { stock, market: pickerMarket }
    if (draftSlot === 'base') {
      setDraftBase(selection)
    } else if (draftSlot === 'long') {
      setDraftLong(selection)
    } else {
      setDraftInverse(selection)
    }
    setPickerInput(pickerMarket === 'US' ? stock.pdno : stock.prdt_name)
    setPickerOpen(false)
    setPickerQuery('')
    setPickerError(null)
  }

  const clearDraft = () => {
    setDraftBase(null)
    setDraftLong(null)
    setDraftInverse(null)
    setDraftBaseRole('underlying')
    setDraftQuantity(1)
    setDraftInverseQuantity(1)
    setPickerStock(null)
    setPickerInput('')
    setPickerError(null)
  }

  const handlePickerUsSearch = async () => {
    const ticker = pickerInput.trim().toUpperCase()
    if (!ticker) return
    setPickerSearching(true)
    setPickerError(null)
    for (const exc of EXCHANGE_SEARCH_ORDER) {
      try {
        const res = await cmd.getOverseasPrice(ticker, exc)
        const validPrice = parseFloat(res.last) > 0
        const hasName = res.name && res.name.trim().length > 0
        if (res && (validPrice || hasName)) {
          handlePickerSelect({ pdno: ticker, prdt_name: res.name || ticker })
          setPickerSearching(false)
          return
        }
      } catch {
        // 다음 거래소 시도
      }
    }
    setPickerError(`"${ticker}"을 NAS·NYS·AMEX에서 찾을 수 없습니다.`)
    setPickerStock(null)
    setPickerSearching(false)
  }

  const handleAddLeveraged = () => {
    if (!canAddDraftSet || !draftBase || !draftLong) return
    const inverse = draftInverse?.market === draftLong.market ? draftInverse.stock : null
    onUpdate([
      ...entries,
      {
        leveraged_symbol: draftLong.stock.pdno,
        leveraged_symbol_name: draftLong.stock.prdt_name,
        inverse_leveraged_symbol: inverse?.pdno ?? '',
        inverse_leveraged_symbol_name: inverse?.prdt_name ?? '',
        base_symbols: [draftBase.stock.pdno],
        base_symbol_names: { [draftBase.stock.pdno]: draftBase.stock.prdt_name },
        base_symbol_roles: { [draftBase.stock.pdno]: draftBaseRole },
        quantity: Math.max(1, draftQuantity),
        inverse_quantity: Math.max(1, draftInverseQuantity),
        is_overseas: draftLong.market === 'US',
      },
    ])
    clearDraft()
  }

  const handleSetInverse = (leveraged: string) => {
    if (!pickerStock) return
    onUpdate(entries.map((entry) => {
      if (entry.leveraged_symbol !== leveraged) return entry
      if (!isSelectionCompatible(entry)) return entry
      if (entry.leveraged_symbol === pickerStock.pdno || entry.base_symbols.includes(pickerStock.pdno)) return entry
      return {
        ...entry,
        inverse_leveraged_symbol: pickerStock.pdno,
        inverse_leveraged_symbol_name: pickerStock.prdt_name,
        inverse_quantity: entry.inverse_quantity || 1,
      }
    }))
  }

  const handleClearInverse = (leveraged: string) => {
    onUpdate(entries.map((entry) => (
      entry.leveraged_symbol === leveraged
        ? { ...entry, inverse_leveraged_symbol: '', inverse_leveraged_symbol_name: '', inverse_quantity: 1 }
        : entry
    )))
  }

  const handleAddBase = (leveraged: string, role: LeveragedTrendHoldBaseRole) => {
    if (!pickerStock) return
    onUpdate(entries.map((entry) => {
      if (entry.leveraged_symbol !== leveraged) return entry
      if (!isSelectionCompatible(entry)) return entry
      if (
        entry.leveraged_symbol === pickerStock.pdno ||
        entry.inverse_leveraged_symbol === pickerStock.pdno ||
        entry.base_symbols.includes(pickerStock.pdno)
      ) return entry
      return {
        ...entry,
        base_symbols: [...entry.base_symbols, pickerStock.pdno],
        base_symbol_names: { ...entry.base_symbol_names, [pickerStock.pdno]: pickerStock.prdt_name },
        base_symbol_roles: { ...(entry.base_symbol_roles ?? {}), [pickerStock.pdno]: role },
      }
    }))
  }

  const handleRemoveBase = (leveraged: string, base: string) => {
    onUpdate(entries.map((entry) => {
      if (entry.leveraged_symbol !== leveraged) return entry
      const nextNames = { ...entry.base_symbol_names }
      const nextRoles = { ...(entry.base_symbol_roles ?? {}) }
      delete nextNames[base]
      delete nextRoles[base]
      return {
        ...entry,
        base_symbols: entry.base_symbols.filter((s) => s !== base),
        base_symbol_names: nextNames,
        base_symbol_roles: nextRoles,
      }
    }))
  }

  const handleRemoveLeveraged = (leveraged: string) => {
    onUpdate(entries.filter((entry) => entry.leveraged_symbol !== leveraged))
  }

  const handleQuantity = (leveraged: string, quantity: number, field: 'quantity' | 'inverse_quantity') => {
    onUpdate(entries.map((entry) => (
      entry.leveraged_symbol === leveraged
        ? { ...entry, [field]: Math.max(1, quantity) }
        : entry
    )))
  }

  return (
    <Stack spacing={1.5}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" flexWrap="wrap" gap={1}>
        <Stack direction="row" alignItems="center" gap={0.5}>
          <Typography variant="caption" color="text.secondary" fontWeight={600}>
            레버리지 ETF 세트 ({entries.length}개)
          </Typography>
          <Tooltip
            title="한 세트에 롱 레버리지 ETF, 선택 숏 ETF, 추세 판단용 기초 ETF를 묶는다. TECL처럼 직접 기초지수가 애매하면 VGT 같은 유사 기초 ETF로 추가한다."
            arrow
          >
            <InfoOutlinedIcon sx={{ fontSize: 13, color: 'text.disabled', cursor: 'help' }} />
          </Tooltip>
        </Stack>
        <Button
          size="small"
          variant="outlined"
          startIcon={<AddIcon />}
          disabled={!canAddDraftSet || stratEnabled}
          onClick={handleAddLeveraged}
          sx={{ fontSize: '0.7rem', py: 0.3 }}
        >
          세트 추가
        </Button>
      </Stack>

      <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 1.25, bgcolor: 'action.hover' }}>
        <Stack spacing={1}>
          <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
            <Typography variant="caption" color="text.secondary" fontWeight={600}>
              새 세트 구성
            </Typography>
            <ToggleButtonGroup
              value={draftSlot}
              exclusive
              onChange={(_, v) => { if (v) setDraftSlot(v as LeveragedSetDraftSlot) }}
              size="small"
              disabled={stratEnabled}
            >
              <ToggleButton value="base">기초지수</ToggleButton>
              <ToggleButton value="long">롱</ToggleButton>
              <ToggleButton value="inverse">숏(옵션)</ToggleButton>
            </ToggleButtonGroup>
            {draftSlot === 'base' && (
              <ToggleButtonGroup
                value={draftBaseRole}
                exclusive
                onChange={(_, v) => { if (v) setDraftBaseRole(v as LeveragedTrendHoldBaseRole) }}
                size="small"
                disabled={stratEnabled}
              >
                <ToggleButton value="underlying">기초</ToggleButton>
                <ToggleButton value="proxy">유사기초</ToggleButton>
              </ToggleButtonGroup>
            )}
          </Stack>

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} alignItems={{ xs: 'stretch', md: 'center' }}>
            <ToggleButtonGroup
              value={pickerMarket}
              exclusive
              onChange={(_, v) => { if (v) setPickerMarket(v as Market) }}
              size="small"
              disabled={stratEnabled}
              sx={{ flexShrink: 0 }}
            >
              <ToggleButton value="KR">
                <FlagIcon fontSize="small" sx={{ mr: 0.5 }} />국내
              </ToggleButton>
              <ToggleButton value="US">
                <PublicIcon fontSize="small" sx={{ mr: 0.5 }} />미국
              </ToggleButton>
            </ToggleButtonGroup>

            <Box sx={{ position: 'relative', flex: 1 }}>
              <TextField
                label={`${LTH_DRAFT_SLOT_LABEL[draftSlot]} ${pickerMarket === 'US' ? 'ETF 티커' : 'ETF 코드 또는 이름'}`}
                value={pickerInput}
                onChange={(e) => {
                  const next = pickerMarket === 'US' ? e.target.value.toUpperCase() : e.target.value
                  setPickerInput(next)
                  setPickerStock(null)
                  setPickerError(null)
                  if (pickerMarket === 'KR') setPickerOpen(next.length >= 2)
                }}
                onFocus={() => {
                  if (pickerCloseTimer.current) clearTimeout(pickerCloseTimer.current)
                  if (pickerMarket === 'KR' && pickerInput.length >= 2) setPickerOpen(true)
                }}
                onBlur={() => {
                  pickerCloseTimer.current = setTimeout(() => setPickerOpen(false), 180)
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && pickerMarket === 'US') void handlePickerUsSearch()
                }}
                size="small"
                fullWidth
                disabled={stratEnabled}
                inputProps={{ style: { textTransform: pickerMarket === 'US' ? 'uppercase' : 'none' } }}
                InputProps={{
                  endAdornment: (
                    <InputAdornment position="end">
                      {(pickerFetching || pickerSearching) && <CircularProgress size={14} color="inherit" sx={{ mr: 0.5 }} />}
                      <IconButton
                        size="small"
                        disabled={stratEnabled || !pickerInput.trim() || (pickerMarket === 'KR' && pickerInput.trim().length < 2)}
                        onClick={() => {
                          if (pickerMarket === 'US') {
                            void handlePickerUsSearch()
                          } else {
                            setPickerQuery(pickerInput)
                            setPickerOpen(true)
                          }
                        }}
                      >
                        <SearchIcon fontSize="small" />
                      </IconButton>
                    </InputAdornment>
                  ),
                }}
              />

              {pickerMarket === 'KR' && pickerOpen && (pickerResults.length > 0 || pickerFetching) && (
                <Paper
                  elevation={8}
                  onMouseDown={(e) => {
                    e.preventDefault()
                    if (pickerCloseTimer.current) clearTimeout(pickerCloseTimer.current)
                  }}
                  sx={{ mt: 0.5, maxHeight: 220, overflow: 'auto', border: 1, borderColor: 'divider', zIndex: 1400, position: 'absolute', width: '100%' }}
                >
                  {pickerFetching && pickerResults.length === 0 ? (
                    <Box sx={{ p: 1.5, display: 'flex', alignItems: 'center', gap: 1 }}>
                      <CircularProgress size={14} />
                      <Typography variant="caption" color="text.secondary">검색 중...</Typography>
                    </Box>
                  ) : (
                    <Table size="small">
                      <TableBody>
                        {pickerResults.map((r) => (
                          <TableRow
                            key={r.pdno}
                            hover
                            sx={{ cursor: 'pointer' }}
                            onClick={() => handlePickerSelect(r)}
                          >
                            <TableCell sx={{ py: 0.75 }}>
                              <Typography variant="body2" noWrap>{r.prdt_name}</Typography>
                            </TableCell>
                            <TableCell sx={{ py: 0.75, width: 90 }}>
                              <Typography variant="caption" color="text.secondary">{r.pdno}</Typography>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  )}
                </Paper>
              )}
            </Box>
          </Stack>

          <Grid container spacing={1}>
            {([
              ['base', '기초지수', draftBase],
              ['long', '롱 ETF', draftLong],
              ['inverse', '숏 ETF(옵션)', draftInverse],
            ] as const).map(([slot, label, selection]) => (
              <Grid item xs={12} md={4} key={slot}>
                <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 1, bgcolor: 'background.paper', minHeight: 64 }}>
                  <Typography variant="caption" color="text.secondary" fontWeight={600} display="block" mb={0.5}>
                    {label}
                  </Typography>
                  {selection ? (
                    <Chip
                      size="small"
                      color={slot === 'long' ? 'success' : slot === 'inverse' ? 'warning' : 'primary'}
                      variant={slot === 'inverse' ? 'outlined' : 'filled'}
                      label={`${selection.stock.prdt_name} (${selection.stock.pdno})`}
                      onDelete={stratEnabled ? undefined : () => {
                        if (slot === 'base') setDraftBase(null)
                        if (slot === 'long') setDraftLong(null)
                        if (slot === 'inverse') setDraftInverse(null)
                      }}
                      sx={{ maxWidth: '100%', '& .MuiChip-label': { overflow: 'hidden', textOverflow: 'ellipsis' } }}
                    />
                  ) : (
                    <Typography variant="caption" color="text.disabled">
                      {slot === 'inverse' ? '미설정 시 롱 전용' : `${label}을 선택하세요`}
                    </Typography>
                  )}
                </Box>
              </Grid>
            ))}
          </Grid>

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} alignItems={{ xs: 'stretch', md: 'center' }}>
            <TextField
              label="롱 수량"
              type="number"
              value={draftQuantity}
              disabled={stratEnabled}
              size="small"
              onChange={(e) => setDraftQuantity(Math.max(1, Number(e.target.value)))}
              inputProps={{ min: 1, step: 1 }}
              sx={{ width: { xs: '100%', md: 120 } }}
            />
            <TextField
              label="숏 수량"
              type="number"
              value={draftInverseQuantity}
              disabled={stratEnabled || !draftInverse}
              size="small"
              onChange={(e) => setDraftInverseQuantity(Math.max(1, Number(e.target.value)))}
              inputProps={{ min: 1, step: 1 }}
              sx={{ width: { xs: '100%', md: 120 } }}
            />
            <Button
              variant="contained"
              size="small"
              startIcon={<AddIcon />}
              disabled={!canAddDraftSet || stratEnabled}
              onClick={handleAddLeveraged}
            >
              세트 추가
            </Button>
            <Button
              variant="outlined"
              size="small"
              disabled={stratEnabled || (!draftBase && !draftLong && !draftInverse)}
              onClick={clearDraft}
            >
              초안 비우기
            </Button>
          </Stack>

          {pickerError && (
            <Alert severity="warning" sx={{ py: 0.5 }}>
              <Typography variant="caption">{pickerError}</Typography>
            </Alert>
          )}

          {pickerStockListEmpty && (
            <Alert
              severity="warning"
              sx={{ py: 0.5 }}
              action={
                <Button
                  size="small"
                  color="warning"
                  variant="outlined"
                  onClick={() => doPickerRefreshList()}
                  disabled={pickerRefreshing}
                  startIcon={pickerRefreshing ? <CircularProgress size={12} /> : <RefreshIcon fontSize="small" />}
                >
                  {pickerRefreshing ? '다운로드 중...' : '목록 새로고침'}
                </Button>
              }
            >
              <Typography variant="caption">종목 목록이 로드되지 않았습니다.</Typography>
            </Alert>
          )}
        </Stack>
      </Box>

      {(invalidEntries.length > 0 || entriesWithoutInverse.length > 0) && (
        <Alert severity={invalidEntries.length > 0 ? 'warning' : 'info'} sx={{ py: 0.5 }}>
          <Typography variant="caption">
            {invalidEntries.length > 0
              ? '기초/유사 기초 ETF가 없는 세트는 저장할 수 없습니다.'
              : '숏 ETF가 비어 있는 세트는 하락 추세 진입 없이 롱 진입과 청산만 동작합니다.'}
          </Typography>
        </Alert>
      )}

      {entries.length === 0 ? (
        <Typography variant="caption" color="text.disabled" sx={{ pl: 0.5 }}>
          추가된 레버리지 ETF 세트가 없습니다
        </Typography>
      ) : (
        <TableContainer sx={{ border: 1, borderColor: 'divider', borderRadius: 1, overflowX: 'auto' }}>
          <Table size="small" sx={{ minWidth: 1120 }}>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 150 }}>롱 레버리지 ETF</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 100 }} align="center">수량</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 180 }}>숏 레버리지 ETF</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 100 }} align="center">숏 수량</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 300 }}>기초/유사 지수 ETF</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 160 }} align="center">기초 추가</TableCell>
                <TableCell sx={{ width: 36, py: 0.75 }} />
              </TableRow>
            </TableHead>
            <TableBody>
              {entries.map((entry) => (
                <TableRow key={entry.leveraged_symbol}>
                  <TableCell sx={{ py: 0.75 }}>
                    <Stack direction="row" alignItems="center" gap={0.5}>
                      {entry.is_overseas && (
                        <Typography variant="caption" color="primary.main" fontWeight={700} sx={{ fontSize: '0.6rem' }}>$</Typography>
                      )}
                      <Box>
                        <Typography variant="caption" fontWeight={600}>{entry.leveraged_symbol}</Typography>
                        <Typography variant="caption" color="text.secondary" display="block" noWrap sx={{ maxWidth: 120 }}>
                          {entry.leveraged_symbol_name}
                        </Typography>
                      </Box>
                    </Stack>
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }} align="center">
                    <TextField
                      type="number"
                      value={entry.quantity}
                      disabled={stratEnabled}
                      size="small"
                      onChange={(e) => handleQuantity(entry.leveraged_symbol, Number(e.target.value), 'quantity')}
                      inputProps={{ min: 1, step: 1, style: { padding: '4px 4px', fontSize: '0.75rem', textAlign: 'right' } }}
                      sx={{ width: 80, '& .MuiInputBase-root': { fontSize: '0.75rem' } }}
                    />
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }}>
                    {entry.inverse_leveraged_symbol ? (
                      <Chip
                        size="small"
                        label={`${entry.inverse_leveraged_symbol_name || entry.inverse_leveraged_symbol} (${entry.inverse_leveraged_symbol})`}
                        onDelete={stratEnabled ? undefined : () => handleClearInverse(entry.leveraged_symbol)}
                        sx={{ height: 22, fontSize: '0.65rem' }}
                      />
                    ) : (
                      <Button
                        size="small"
                        variant="outlined"
                        startIcon={<AddIcon />}
                        disabled={
                          !pickerStock ||
                          stratEnabled ||
                          !isSelectionCompatible(entry) ||
                          pickerStock.pdno === entry.leveraged_symbol ||
                          entry.base_symbols.includes(pickerStock?.pdno ?? '')
                        }
                        onClick={() => handleSetInverse(entry.leveraged_symbol)}
                        sx={{ fontSize: '0.7rem', py: 0.25 }}
                      >
                        {pickerStock ? '숏으로 설정' : 'ETF 선택'}
                      </Button>
                    )}
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }} align="center">
                    <TextField
                      type="number"
                      value={entry.inverse_quantity || 1}
                      disabled={stratEnabled || !entry.inverse_leveraged_symbol}
                      size="small"
                      onChange={(e) => handleQuantity(entry.leveraged_symbol, Number(e.target.value), 'inverse_quantity')}
                      inputProps={{ min: 1, step: 1, style: { padding: '4px 4px', fontSize: '0.75rem', textAlign: 'right' } }}
                      sx={{ width: 80, '& .MuiInputBase-root': { fontSize: '0.75rem' } }}
                    />
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }}>
                    <Stack direction="row" gap={0.5} flexWrap="wrap">
                      {entry.base_symbols.length === 0 ? (
                        <Typography variant="caption" color="text.disabled">기초 항목 없음</Typography>
                      ) : entry.base_symbols.map((base) => (
                        <Chip
                          key={base}
                          size="small"
                          label={`${LTH_BASE_ROLE_LABEL[getLthBaseRole(entry, base)]}: ${entry.base_symbol_names[base] ?? base} (${base})`}
                          color={getLthBaseRole(entry, base) === 'proxy' ? 'warning' : 'default'}
                          variant={getLthBaseRole(entry, base) === 'proxy' ? 'outlined' : 'filled'}
                          onDelete={stratEnabled ? undefined : () => handleRemoveBase(entry.leveraged_symbol, base)}
                          sx={{ height: 22, fontSize: '0.65rem' }}
                        />
                      ))}
                    </Stack>
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }} align="center">
                    <Stack direction="row" spacing={0.5} justifyContent="center">
                      {(['underlying', 'proxy'] as const).map((role) => (
                        <Button
                          key={role}
                          size="small"
                          variant="outlined"
                          startIcon={<AddIcon />}
                          disabled={
                            !pickerStock ||
                            stratEnabled ||
                            !isSelectionCompatible(entry) ||
                            pickerStock.pdno === entry.leveraged_symbol ||
                            pickerStock.pdno === entry.inverse_leveraged_symbol ||
                            entry.base_symbols.includes(pickerStock?.pdno ?? '')
                          }
                          onClick={() => handleAddBase(entry.leveraged_symbol, role)}
                          sx={{ fontSize: '0.68rem', py: 0.25, px: 0.75, minWidth: 64 }}
                        >
                          {pickerStock ? `${LTH_BASE_ROLE_LABEL[role]} 추가` : 'ETF 선택'}
                        </Button>
                      ))}
                    </Stack>
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }}>
                    <IconButton size="small" disabled={stratEnabled} onClick={() => handleRemoveLeveraged(entry.leveraged_symbol)}>
                      <DeleteIcon sx={{ fontSize: 14 }} />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Stack>
  )
}

type EditState = { symbols: string[]; quantity: number; params: Record<string, number> }

// ─── Strategy 메인 ────────────────────────────────────────────────
export default function Strategy() {
  const { data: strategies, isLoading } = useStrategies()
  const { data: tradingStatus } = useTradingStatus()
  const { mutate: updateStrategy, isPending: saving } = useUpdateStrategy()

  const [editMap, setEditMap] = useState<Record<string, EditState>>({})
  // 가격 조건 매매 전략 전용: 종목별 설정 배열
  const [pcEditMap, setPcEditMap] = useState<Record<string, PriceConditionSymbolConfig[]>>({})
  // 레버리지 추세 보유 전략 전용: 레버리지/기초 매핑 배열
  const [lthEditMap, setLthEditMap] = useState<Record<string, LeveragedTrendHoldEntry[]>>({})

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
  // 전략이 로드됐을 때 targetSymbolNames를 symbolNames 캐시에 반영
  useEffect(() => {
    if (!strategies) return
    const names: Record<string, string> = {}
    for (const s of strategies) {
      for (const [code, name] of Object.entries(s.targetSymbolNames)) {
        names[code] = name
      }
    }
    if (Object.keys(names).length > 0) {
      setSymbolNames(prev => ({ ...names, ...prev }))
    }
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

  const handleSavePc = (id: string) => {
    const pcSymbols = pcEditMap[id] ?? []
    updateStrategy(
      {
        id,
        targetSymbols: pcSymbols.map((s) => s.symbol),
        params: { symbols: pcSymbols },
      } satisfies UpdateStrategyInput,
      { onSuccess: () => setPcEditMap(prev => { const n = { ...prev }; delete n[id]; return n }) },
    )
  }

  const handleSaveLth = (id: string, params: Record<string, unknown>) => {
    const entries = lthEditMap[id] ?? []
    if (hasInvalidLthEntries(entries)) return
    const targetSymbols = Array.from(new Set(entries.flatMap((entry) => [
      entry.leveraged_symbol,
      entry.inverse_leveraged_symbol,
      ...entry.base_symbols,
    ]).filter(Boolean)))
    updateStrategy(
      {
        id,
        targetSymbols,
        params: { ...params, entries },
      } satisfies UpdateStrategyInput,
      { onSuccess: () => setLthEditMap(prev => { const n = { ...prev }; delete n[id]; return n }) },
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
            <Grid item xs={12} md={sType === 'price_condition' || sType === 'leveraged_trend_hold' ? 12 : 6} key={s.id}>
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
                  {/* price_condition: 커스텀 편집 UI */}
                  {sType === 'price_condition' ? (
                    <PriceConditionEditorPanel
                      stratEnabled={s.enabled}
                      initialSymbols={(s.params['symbols'] as PriceConditionSymbolConfig[] | undefined) ?? []}
                      editedSymbols={pcEditMap[s.id]}
                      selectedStock={selectedStock}
                      market={market}
                      onUpdate={(syms) => setPcEditMap((prev) => ({ ...prev, [s.id]: syms }))}
                    />
                  ) : sType === 'leveraged_trend_hold' ? (
                    <LeveragedTrendHoldEditorPanel
                      stratEnabled={s.enabled}
                      initialEntries={(s.params['entries'] as LeveragedTrendHoldEntry[] | undefined) ?? []}
                      editedEntries={lthEditMap[s.id]}
                      onUpdate={(entries) => setLthEditMap((prev) => ({ ...prev, [s.id]: entries }))}
                    />
                  ) : (
                    <>
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
                    </>
                  )}
                </Stack>

                {stratDesc && sType !== 'price_condition' && sType !== 'leveraged_trend_hold' && (
                  <Tooltip title={stratDesc} arrow placement="bottom-start">
                    <Stack direction="row" alignItems="center" gap={0.5} sx={{ mt: 1.5, cursor: 'help', width: 'fit-content' }}>
                      <InfoOutlinedIcon sx={{ fontSize: 13, color: 'text.disabled' }} />
                      <Typography variant="caption" color="text.disabled" sx={{ fontSize: '0.65rem' }}>전략 설명 보기</Typography>
                    </Stack>
                  </Tooltip>
                )}

                {sType === 'price_condition' ? (
                  pcEditMap[s.id] !== undefined && !s.enabled && (
                    <Box sx={{ mt: 1.5 }}>
                      <Button
                        size="small"
                        variant="outlined"
                        startIcon={saving ? <CircularProgress size={14} /> : <SaveIcon />}
                        onClick={() => handleSavePc(s.id)}
                        disabled={saving}
                      >
                        변경사항 저장
                      </Button>
                    </Box>
                  )
                ) : sType === 'leveraged_trend_hold' ? (
                  lthEditMap[s.id] !== undefined && !s.enabled && (
                    <Box sx={{ mt: 1.5 }}>
                      <Button
                        size="small"
                        variant="outlined"
                        startIcon={saving ? <CircularProgress size={14} /> : <SaveIcon />}
                        onClick={() => handleSaveLth(s.id, s.params)}
                        disabled={saving || hasInvalidLthEntries(lthEditMap[s.id] ?? [])}
                      >
                        변경사항 저장
                      </Button>
                    </Box>
                  )
                ) : (
                  isDirty && !s.enabled && (
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
                  )
                )}
              </Paper>
            </Grid>
          )
        })}
      </Grid>
    </Box>
  )
}
