import { useEffect, useMemo, useRef, useState } from 'react'

import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Checkbox from '@mui/material/Checkbox'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import FormControlLabel from '@mui/material/FormControlLabel'
import IconButton from '@mui/material/IconButton'
import InputAdornment from '@mui/material/InputAdornment'
import MenuItem from '@mui/material/MenuItem'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import FlagIcon from '@mui/icons-material/Flag'
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined'
import PublicIcon from '@mui/icons-material/Public'
import RefreshIcon from '@mui/icons-material/Refresh'
import SearchIcon from '@mui/icons-material/Search'

import {
  useAppConfig,
  usePreviewLeveragedTrendHold,
  useRefreshStockList,
  useStockSearch,
} from '../../../api/hooks'
import * as cmd from '../../../api/commands'
import type {
  CmdError,
  LeveragedTrendHoldEntry,
  OverseasExchange,
  StockSearchItem,
} from '../../../api/types'
import { LeveragedTrendHoldPreviewChart } from './leveragedTrendHoldPreviewChart'

type Market = 'KR' | 'US'
type TargetDraftSelection = {
  stock: StockSearchItem
  market: Market
}

type LeveragedTrendHoldEditorPanelProps = {
  stratEnabled: boolean
  initialEntries: LeveragedTrendHoldEntry[]
  editedEntries: LeveragedTrendHoldEntry[] | undefined
  params: Record<string, unknown>
  onUpdate: (entries: LeveragedTrendHoldEntry[]) => void
  onParamsUpdate: (params: Record<string, unknown>) => void
}

const EXCHANGE_SEARCH_ORDER: OverseasExchange[] = ['NAS', 'NYS', 'AMS']
const US_TICKER_PATTERN = /^[A-Z][A-Z0-9.-]{0,9}$/

function normalizeUsTicker(value: string) {
  return value.trim().toUpperCase()
}

function numericParam(params: Record<string, unknown>, key: string, fallback: number) {
  const value = params[key]
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback
}

function boolParam(params: Record<string, unknown>, key: string, fallback: boolean) {
  const value = params[key]
  return typeof value === 'boolean' ? value : fallback
}

export function hasInvalidLthEntries(entries: LeveragedTrendHoldEntry[]): boolean {
  return entries.some((entry) => !entry.leveraged_symbol)
}

function newTargetEntry(selection: TargetDraftSelection, quantity: number): LeveragedTrendHoldEntry {
  return {
    leveraged_symbol: selection.stock.pdno,
    leveraged_symbol_name: selection.stock.prdt_name,
    inverse_leveraged_symbol: '',
    inverse_leveraged_symbol_name: '',
    base_symbols: [],
    base_symbol_names: {},
    base_symbol_roles: {},
    quantity: Math.max(1, quantity),
    inverse_quantity: 1,
    is_overseas: selection.market === 'US',
  }
}

export function LeveragedTrendHoldEditorPanel(props: LeveragedTrendHoldEditorPanelProps) {
  const { stratEnabled, initialEntries, editedEntries, params, onUpdate } = props
  const entries = editedEntries ?? initialEntries
  const [pickerMarket, setPickerMarket] = useState<Market>('US')
  const [pickerInput, setPickerInput] = useState('')
  const [pickerQuery, setPickerQuery] = useState('')
  const [pickerOpen, setPickerOpen] = useState(false)
  const [pickerSelection, setPickerSelection] = useState<TargetDraftSelection | null>(null)
  const pickerCloseTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const [pickerSearching, setPickerSearching] = useState(false)
  const [pickerError, setPickerError] = useState<string | null>(null)
  const [draftQuantity, setDraftQuantity] = useState(1)
  const [previewSymbol, setPreviewSymbol] = useState('')
  const entrySensitivity = numericParam(params, 'upward_sensitivity', 1)
  const reboundEnabled = boolParam(params, 'intraday_rebound_enabled', false)
  const reboundBaselineTicks = numericParam(params, 'rebound_baseline_ticks', 8)
  const reboundConfirmTicks = numericParam(params, 'rebound_confirm_ticks', 3)
  const reboundPullback = numericParam(params, 'rebound_pullback_pct', 4)
  const reboundBuyPressure = numericParam(params, 'rebound_buy_pressure_pct', 1.5)
  const reboundRsiMin = numericParam(params, 'rebound_rsi_min', 30)
  const trailingStopPct = numericParam(params, 'trailing_stop_pct', 1.5)
  const trailingActivationProfit = numericParam(params, 'trailing_activation_profit_pct', 1)
  const breakevenBuffer = numericParam(params, 'breakeven_buffer_pct', 0.2)
  const minHoldObservations = numericParam(params, 'min_hold_observations', 2)
  const initialStopLoss = numericParam(params, 'initial_stop_loss_pct', 1)
  const entryFailureObservations = numericParam(params, 'entry_failure_observations', 3)
  const { data: appConfig } = useAppConfig()
  const isTossActive = appConfig?.active_broker_id === 'toss'
  const previewMutation = usePreviewLeveragedTrendHold()
  const previewOptions = useMemo(
    () => entries.filter((entry) => !!entry.leveraged_symbol),
    [entries],
  )
  const previewSymbolsKey = useMemo(
    () => previewOptions.map((entry) => entry.leveraged_symbol).join('|'),
    [previewOptions],
  )
  const defaultPreviewSymbol = useMemo(
    () => (
      previewOptions.find((entry) => entry.is_overseas)?.leveraged_symbol
      ?? previewOptions[0]?.leveraged_symbol
      ?? ''
    ),
    [previewOptions],
  )
  const previewEntry = useMemo(
    () => previewOptions.find((entry) => entry.leveraged_symbol === previewSymbol) ?? null,
    [previewOptions, previewSymbol],
  )
  const currentPreview = previewMutation.data?.symbol === previewEntry?.leveraged_symbol
    ? previewMutation.data
    : null
  const { mutate: doPickerRefreshList, isPending: pickerRefreshing } = useRefreshStockList()
  const {
    data: pickerResults = [],
    isFetching: pickerFetching,
    isError: pickerIsError,
    error: pickerSearchError,
  } = useStockSearch(pickerMarket === 'KR' ? pickerQuery : '')
  const pickerStockListEmpty = pickerIsError && (pickerSearchError as CmdError | null)?.code === 'STOCK_LIST_EMPTY'
  const canAddTarget = !!pickerSelection && !entries.some((entry) => entry.leveraged_symbol === pickerSelection.stock.pdno)

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
    setPickerSelection(null)
    setPickerError(null)
  }, [pickerMarket])

  useEffect(() => {
    if (!defaultPreviewSymbol) {
      setPreviewSymbol('')
      return
    }
    const previewSymbols = previewSymbolsKey ? previewSymbolsKey.split('|') : []
    if (!previewSymbols.includes(previewSymbol)) {
      setPreviewSymbol(defaultPreviewSymbol)
    }
  }, [defaultPreviewSymbol, previewSymbol, previewSymbolsKey])

  const handlePickerSelect = (stock: StockSearchItem) => {
    setPickerSelection({ stock, market: pickerMarket })
    setPickerInput(pickerMarket === 'US' ? stock.pdno : stock.prdt_name)
    setPickerOpen(false)
    setPickerQuery('')
    setPickerError(null)
  }

  const handlePickerUsSearch = async () => {
    const ticker = normalizeUsTicker(pickerInput)
    if (!ticker) return
    if (!US_TICKER_PATTERN.test(ticker)) {
      setPickerError('미국 ETF 티커는 영문으로 시작하고 영문/숫자/./-만 입력할 수 있습니다.')
      setPickerSelection(null)
      return
    }
    setPickerSearching(true)
    setPickerError(null)

    if (isTossActive) {
      try {
        const safety = await cmd.getTossStockSafety(ticker)
        if (safety.stockInfo) {
          handlePickerSelect({ pdno: ticker, prdt_name: safety.stockInfo.name || ticker })
          setPickerSearching(false)
          return
        }
      } catch {
        // 아래 fallback으로 직접 선택 처리
      }
      setPickerSelection({ stock: { pdno: ticker, prdt_name: ticker }, market: 'US' })
      setPickerInput(ticker)
      setPickerError(`Toss 종목 정보로 "${ticker}" 검증은 실패했지만 티커 형식이 유효해 직접 선택했습니다. 저장 후 시세/주문 연결 상태를 확인하세요.`)
      setPickerSearching(false)
      return
    }

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
    setPickerSelection({ stock: { pdno: ticker, prdt_name: ticker }, market: 'US' })
    setPickerInput(ticker)
    setPickerError(`KIS 해외 현재가로 "${ticker}" 검증은 실패했지만 티커 형식이 유효해 직접 선택했습니다. 저장 후 시세/주문 연결 상태를 확인하세요.`)
    setPickerSearching(false)
  }

  const handleAddTarget = () => {
    if (!canAddTarget || !pickerSelection) return
    onUpdate([...entries, newTargetEntry(pickerSelection, draftQuantity)])
    setPickerSelection(null)
    setPickerInput('')
    setPickerError(null)
    setDraftQuantity(1)
  }

  const handleRemoveTarget = (symbol: string) => {
    onUpdate(entries.filter((entry) => entry.leveraged_symbol !== symbol))
  }

  const handleQuantity = (symbol: string, quantity: number) => {
    onUpdate(entries.map((entry) => (
      entry.leveraged_symbol === symbol
        ? { ...entry, quantity: Math.max(1, quantity) }
        : entry
    )))
  }

  const handleSensitivityChange = (value: number) => {
    const nextValue = Number.isFinite(value) ? Math.max(1, Math.min(5, value)) : 1
    props.onParamsUpdate({ ...params, upward_sensitivity: nextValue })
  }

  const updateNumericParam = (key: string, value: number, min: number, max: number) => {
    const nextValue = Number.isFinite(value) ? Math.max(min, Math.min(max, value)) : min
    props.onParamsUpdate({ ...params, [key]: nextValue })
  }

  const updateIntegerParam = (key: string, value: number, min: number, max: number) => {
    const nextValue = Number.isFinite(value) ? Math.max(min, Math.min(max, Math.round(value))) : min
    props.onParamsUpdate({ ...params, [key]: nextValue })
  }

  const handlePreview = () => {
    if (!previewEntry) return
    previewMutation.mutate({
      symbol: previewEntry.leveraged_symbol,
      params: { ...params, entries },
      count: 200,
    })
  }

  return (
    <Stack spacing={1.5}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" flexWrap="wrap" gap={1}>
        <Stack direction="row" alignItems="center" gap={0.5}>
          <Typography variant="caption" color="text.secondary" fontWeight={600}>
            레버리지 대상 ETF ({entries.length}개)
          </Typography>
          <Tooltip
            title="롱/숏 레버리지 구분 없이 선택한 ETF 자체가 상승 추세이면 매수하고, 추세가 훼손되면 청산합니다."
            arrow
          >
            <InfoOutlinedIcon sx={{ fontSize: 13, color: 'text.disabled', cursor: 'help' }} />
          </Tooltip>
        </Stack>
        <Button
          size="small"
          variant="outlined"
          startIcon={<AddIcon />}
          disabled={!canAddTarget || stratEnabled}
          onClick={handleAddTarget}
          sx={{ fontSize: '0.7rem', py: 0.3 }}
        >
          대상 추가
        </Button>
      </Stack>

      <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 1.25, bgcolor: 'action.hover' }}>
        <Stack spacing={1}>
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
                label={pickerMarket === 'US' ? 'ETF 티커' : 'ETF 코드 또는 이름'}
                value={pickerInput}
                onChange={(e) => {
                  const next = pickerMarket === 'US' ? e.target.value.toUpperCase() : e.target.value
                  setPickerInput(next)
                  setPickerSelection(null)
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

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} alignItems={{ xs: 'stretch', md: 'center' }}>
            <TextField
              label="1회 수량"
              type="number"
              value={draftQuantity}
              disabled={stratEnabled}
              size="small"
              onChange={(e) => setDraftQuantity(Math.max(1, Number(e.target.value)))}
              inputProps={{ min: 1, step: 1 }}
              sx={{ width: { xs: '100%', md: 120 } }}
            />
            <Button
              variant="contained"
              size="small"
              startIcon={<AddIcon />}
              disabled={!canAddTarget || stratEnabled}
              onClick={handleAddTarget}
            >
              대상 추가
            </Button>
            {pickerSelection && (
              <Chip
                size="small"
                color="primary"
                label={`${pickerSelection.stock.prdt_name} (${pickerSelection.stock.pdno})`}
                onDelete={stratEnabled ? undefined : () => {
                  setPickerSelection(null)
                  setPickerInput('')
                  setPickerError(null)
                }}
                sx={{ maxWidth: '100%', '& .MuiChip-label': { overflow: 'hidden', textOverflow: 'ellipsis' } }}
              />
            )}
          </Stack>

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} alignItems={{ xs: 'stretch', md: 'center' }}>
            <TextField
              label="진입 민감도"
              type="number"
              value={entrySensitivity}
              disabled={stratEnabled}
              size="small"
              onChange={(e) => handleSensitivityChange(Number(e.target.value))}
              inputProps={{ min: 1, max: 5, step: 0.5 }}
              sx={{ width: { xs: '100%', md: 140 } }}
            />
            <Typography variant="caption" color="text.secondary">
              1은 기본값, 값이 높을수록 상승 진입 RSI 기준을 완화합니다.
            </Typography>
          </Stack>

          <Box sx={{ borderTop: 1, borderColor: 'divider', pt: 1 }}>
            <Stack spacing={1}>
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} sx={{ flexWrap: 'wrap' }}>
                <TextField
                  label="초기 손절(%)"
                  type="number"
                  value={initialStopLoss}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('initial_stop_loss_pct', Number(e.target.value), 0.1, 20)}
                  inputProps={{ min: 0.1, max: 20, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 140 } }}
                />
                <TextField
                  label="실패 판정 관측치"
                  type="number"
                  value={entryFailureObservations}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('entry_failure_observations', Number(e.target.value), 1, 60)}
                  inputProps={{ min: 1, max: 60, step: 1 }}
                  sx={{ width: { xs: '100%', md: 160 } }}
                />
                <TextField
                  label="추적손절(%)"
                  type="number"
                  value={trailingStopPct}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('trailing_stop_pct', Number(e.target.value), 0.5, 20)}
                  inputProps={{ min: 0.5, max: 20, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 140 } }}
                />
                <TextField
                  label="추적 활성 수익(%)"
                  type="number"
                  value={trailingActivationProfit}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('trailing_activation_profit_pct', Number(e.target.value), 0.1, 20)}
                  inputProps={{ min: 0.1, max: 20, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 160 } }}
                />
                <TextField
                  label="본전 보호 버퍼(%)"
                  type="number"
                  value={breakevenBuffer}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('breakeven_buffer_pct', Number(e.target.value), 0, 10)}
                  inputProps={{ min: 0, max: 10, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 160 } }}
                />
                <TextField
                  label="최소 보유 관측치"
                  type="number"
                  value={minHoldObservations}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('min_hold_observations', Number(e.target.value), 0, 60)}
                  inputProps={{ min: 0, max: 60, step: 1 }}
                  sx={{ width: { xs: '100%', md: 150 } }}
                />
              </Stack>
              <Typography variant="caption" color="text.secondary">
                반등이 틀리면 초기 손절/실패 판정으로 먼저 빠지고, 고점 수익률이 활성 기준을 넘긴 뒤에는 본전 보호와 추적손절로 수익을 지킵니다.
              </Typography>
            </Stack>
          </Box>

          <Box sx={{ borderTop: 1, borderColor: 'divider', pt: 1 }}>
            <Stack spacing={1}>
              <FormControlLabel
                control={
                  <Checkbox
                    checked={reboundEnabled}
                    disabled={stratEnabled}
                    onChange={(e) => props.onParamsUpdate({
                      ...params,
                      intraday_rebound_enabled: e.target.checked,
                    })}
                    size="small"
                  />
                }
                label={
                  <Typography variant="caption" fontWeight={600}>
                    장중 반동 진입 사용
                  </Typography>
                }
              />
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={1}>
                <TextField
                  label="기준 관측치"
                  type="number"
                  value={reboundBaselineTicks}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('rebound_baseline_ticks', Number(e.target.value), 2, 120)}
                  inputProps={{ min: 2, max: 120, step: 1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="확인 관측치"
                  type="number"
                  value={reboundConfirmTicks}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('rebound_confirm_ticks', Number(e.target.value), 2, 60)}
                  inputProps={{ min: 2, max: 60, step: 1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="선행 하락(%)"
                  type="number"
                  value={reboundPullback}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('rebound_pullback_pct', Number(e.target.value), 0.5, 30)}
                  inputProps={{ min: 0.5, max: 30, step: 0.5 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="매수세 상승(%)"
                  type="number"
                  value={reboundBuyPressure}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('rebound_buy_pressure_pct', Number(e.target.value), 0.5, 30)}
                  inputProps={{ min: 0.5, max: 30, step: 0.5 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="반동 RSI 하한"
                  type="number"
                  value={reboundRsiMin}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('rebound_rsi_min', Number(e.target.value), 0, 70)}
                  inputProps={{ min: 0, max: 70, step: 1 }}
                  sx={{ width: { xs: '100%', md: 140 } }}
                />
              </Stack>
              <Typography variant="caption" color="text.secondary">
                특정 시각이 아니라 기준 관측 구간에서 충분히 밀린 뒤, 바로 다음 확인 구간에서 강한 가격 회복이 나올 때 매수세 반동으로 판단합니다.
              </Typography>
            </Stack>
          </Box>

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

      {entries.length > 0 && (
        <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 1.25 }}>
          <Stack spacing={1}>
            <Stack direction={{ xs: 'column', md: 'row' }} alignItems={{ xs: 'stretch', md: 'center' }} justifyContent="space-between" gap={1}>
              <Stack spacing={0.25}>
                <Stack direction="row" alignItems="center" gap={0.75} flexWrap="wrap">
                  <Typography variant="caption" color="text.secondary" fontWeight={700}>
                    전략 미리보기
                  </Typography>
                  <Chip
                    size="small"
                    variant="outlined"
                    label="Toss 1분봉"
                    sx={{ height: 22, fontSize: '0.65rem' }}
                  />
                </Stack>
                <Typography variant="caption" color="text.secondary">
                  현재 편집 중인 파라미터를 기준으로 매수/청산 신호를 읽기 전용으로 재계산합니다.
                </Typography>
              </Stack>
              <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} alignItems={{ xs: 'stretch', sm: 'center' }}>
                <TextField
                  select
                  label="시뮬레이션 티커"
                  size="small"
                  value={previewSymbol}
                  disabled={previewOptions.length === 0 || previewMutation.isPending}
                  onChange={(e) => {
                    setPreviewSymbol(e.target.value)
                    previewMutation.reset()
                  }}
                  sx={{ minWidth: { xs: '100%', sm: 180 } }}
                >
                  {previewOptions.map((entry) => (
                    <MenuItem key={entry.leveraged_symbol} value={entry.leveraged_symbol}>
                      {entry.leveraged_symbol} · {entry.leveraged_symbol_name || (entry.is_overseas ? '미국 ETF' : '국내 ETF')}
                    </MenuItem>
                  ))}
                </TextField>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={handlePreview}
                  disabled={!isTossActive || !previewEntry || previewMutation.isPending}
                  startIcon={previewMutation.isPending ? <CircularProgress size={14} /> : <RefreshIcon fontSize="small" />}
                  sx={{ alignSelf: { xs: 'stretch', sm: 'center' }, whiteSpace: 'nowrap' }}
                >
                  {previewMutation.isPending ? '계산 중...' : '미리보기 계산'}
                </Button>
              </Stack>
            </Stack>

            {!isTossActive && (
              <Alert severity="info" sx={{ py: 0.5 }}>
                <Typography variant="caption">
                  Toss 활성 프로파일에서 Toss 1분봉 기반 미리보기를 사용할 수 있습니다.
                </Typography>
              </Alert>
            )}

            {previewMutation.isError && (
              <Alert severity="warning" sx={{ py: 0.5 }}>
                <Typography variant="caption">
                  {(previewMutation.error as CmdError | null)?.message ?? '전략 미리보기 계산에 실패했습니다.'}
                </Typography>
              </Alert>
            )}

            {currentPreview ? (
              <Stack spacing={1}>
                <Alert severity={currentPreview.signals.length > 0 ? 'success' : 'info'} sx={{ py: 0.5 }}>
                  <Typography variant="caption">{currentPreview.message}</Typography>
                </Alert>
                <LeveragedTrendHoldPreviewChart
                  candles={currentPreview.candles}
                  signals={currentPreview.signals}
                />
              </Stack>
            ) : (
              <Box sx={{ minHeight: 160, display: 'grid', placeItems: 'center', border: 1, borderColor: 'divider', borderRadius: 1, bgcolor: 'action.hover' }}>
                <Typography variant="caption" color="text.secondary">
                  미리보기 계산을 실행하면 이곳에 1분봉 차트와 매수/매도 신호가 표시됩니다.
                </Typography>
              </Box>
            )}
          </Stack>
        </Box>
      )}

      {entries.length === 0 ? (
        <Typography variant="caption" color="text.disabled" sx={{ pl: 0.5 }}>
          추가된 레버리지 대상 ETF가 없습니다
        </Typography>
      ) : (
        <TableContainer sx={{ border: 1, borderColor: 'divider', borderRadius: 1, overflowX: 'auto' }}>
          <Table size="small" sx={{ minWidth: 620 }}>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 90 }}>시장</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 120 }}>티커</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75 }}>종목명</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 100 }} align="center">수량</TableCell>
                <TableCell sx={{ width: 36, py: 0.75 }} />
              </TableRow>
            </TableHead>
            <TableBody>
              {entries.map((entry) => (
                <TableRow key={entry.leveraged_symbol}>
                  <TableCell sx={{ py: 0.75 }}>
                    <Chip
                      size="small"
                      label={entry.is_overseas ? '미국' : '국내'}
                      color={entry.is_overseas ? 'primary' : 'default'}
                      variant="outlined"
                      sx={{ height: 22, fontSize: '0.65rem' }}
                    />
                  </TableCell>
                  <TableCell sx={{ py: 0.75 }}>
                    <Typography variant="caption" color="primary.main" fontWeight={700}>
                      {entry.leveraged_symbol}
                    </Typography>
                  </TableCell>
                  <TableCell sx={{ py: 0.75 }}>
                    <Typography variant="caption" color="text.secondary" noWrap>
                      {entry.leveraged_symbol_name || entry.leveraged_symbol}
                    </Typography>
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }} align="center">
                    <TextField
                      type="number"
                      value={entry.quantity}
                      disabled={stratEnabled}
                      size="small"
                      onChange={(e) => handleQuantity(entry.leveraged_symbol, Number(e.target.value))}
                      inputProps={{ min: 1, step: 1, style: { padding: '4px 4px', fontSize: '0.75rem', textAlign: 'right' } }}
                      sx={{ width: 80, '& .MuiInputBase-root': { fontSize: '0.75rem' } }}
                    />
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }}>
                    <IconButton size="small" disabled={stratEnabled} onClick={() => handleRemoveTarget(entry.leveraged_symbol)}>
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
