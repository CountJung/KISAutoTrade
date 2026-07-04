import { useEffect, useRef, useState } from 'react'

import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Grid from '@mui/material/Grid'
import IconButton from '@mui/material/IconButton'
import InputAdornment from '@mui/material/InputAdornment'
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

import { useRefreshStockList, useStockSearch } from '../../../api/hooks'
import * as cmd from '../../../api/commands'
import type {
  CmdError,
  LeveragedTrendHoldBaseRole,
  LeveragedTrendHoldEntry,
  OverseasExchange,
  StockSearchItem,
} from '../../../api/types'

type Market = 'KR' | 'US'
type LeveragedSetDraftSlot = 'base' | 'long' | 'inverse'
type LeveragedSetDraftSelection = {
  stock: StockSearchItem
  market: Market
}

const EXCHANGE_SEARCH_ORDER: OverseasExchange[] = ['NAS', 'NYS', 'AMS']

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

export function hasInvalidLthEntries(entries: LeveragedTrendHoldEntry[]): boolean {
  return entries.some((entry) => !entry.leveraged_symbol || entry.base_symbols.length === 0)
}

export function LeveragedTrendHoldEditorPanel({
  stratEnabled,
  initialEntries,
  editedEntries,
  params,
  onUpdate,
  onParamsUpdate,
}: {
  stratEnabled: boolean
  initialEntries: LeveragedTrendHoldEntry[]
  editedEntries: LeveragedTrendHoldEntry[] | undefined
  params: Record<string, unknown>
  onUpdate: (entries: LeveragedTrendHoldEntry[]) => void
  onParamsUpdate: (params: Record<string, unknown>) => void
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
  const upwardSensitivity = typeof params.upward_sensitivity === 'number' ? params.upward_sensitivity : 1
  const downwardSensitivity = typeof params.downward_sensitivity === 'number' ? params.downward_sensitivity : 1
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

  const handleSensitivityChange = (key: 'upward_sensitivity' | 'downward_sensitivity', value: number) => {
    const nextValue = Number.isFinite(value) ? Math.max(1, Math.min(5, value)) : 1
    onParamsUpdate({ ...params, [key]: nextValue })
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

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={1}>
            <TextField
              label="상승 민감도"
              type="number"
              value={upwardSensitivity}
              disabled={stratEnabled}
              size="small"
              onChange={(e) => handleSensitivityChange('upward_sensitivity', Number(e.target.value))}
              inputProps={{ min: 1, max: 5, step: 0.5 }}
              sx={{ width: { xs: '100%', md: 140 } }}
            />
            <TextField
              label="하락 민감도"
              type="number"
              value={downwardSensitivity}
              disabled={stratEnabled}
              size="small"
              onChange={(e) => handleSensitivityChange('downward_sensitivity', Number(e.target.value))}
              inputProps={{ min: 1, max: 5, step: 0.5 }}
              sx={{ width: { xs: '100%', md: 140 } }}
            />
            <Typography variant="caption" color="text.secondary" sx={{ alignSelf: { xs: 'flex-start', md: 'center' } }}>
              1은 기본값, 값이 높을수록 롱/숏 진입 신호를 더 민감하게 판단합니다.
            </Typography>
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
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 300 }}>기초/유사 지수 ETF</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 160 }} align="center">기초 추가</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 150 }}>롱 레버리지 ETF</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 100 }} align="center">롱 수량</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 180 }}>숏 레버리지 ETF</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 100 }} align="center">숏 수량</TableCell>
                <TableCell sx={{ width: 36, py: 0.75 }} />
              </TableRow>
            </TableHead>
            <TableBody>
              {entries.map((entry) => (
                <TableRow key={entry.leveraged_symbol}>
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
                  <TableCell sx={{ py: 0.75 }}>
                    <Stack direction="row" alignItems="center" gap={0.5} flexWrap="wrap">
                      {entry.is_overseas && (
                        <Typography variant="caption" color="primary.main" fontWeight={700} sx={{ fontSize: '0.6rem' }}>$</Typography>
                      )}
                      <Chip size="small" label="롱" color="primary" sx={{ height: 20, fontSize: '0.62rem' }} />
                      <Box>
                        <Typography variant="caption" color="primary.main" fontWeight={700}>{entry.leveraged_symbol}</Typography>
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
                      <Stack direction="row" alignItems="center" gap={0.5} flexWrap="wrap">
                        <Chip size="small" label="숏" color="secondary" sx={{ height: 20, fontSize: '0.62rem' }} />
                        <Chip
                          size="small"
                          color="secondary"
                          variant="outlined"
                          label={`${entry.inverse_leveraged_symbol_name || entry.inverse_leveraged_symbol} (${entry.inverse_leveraged_symbol})`}
                          onDelete={stratEnabled ? undefined : () => handleClearInverse(entry.leveraged_symbol)}
                          sx={{ height: 22, fontSize: '0.65rem' }}
                        />
                      </Stack>
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
