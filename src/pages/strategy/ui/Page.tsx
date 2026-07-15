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
  LeveragedTrendHoldEditorPanel,
  hasInvalidLthEntries,
} from './leveragedTrendHoldEditorPanel'
import { StrategyPreviewPanel } from './strategyPreviewPanel'
import { PriceConditionEditorPanel } from './priceConditionEditorPanel'
import {
  STRATEGY_DESCRIPTION,
  STRATEGY_PARAM_META,
  getStrategyType,
} from './strategyMetadata'
import {
  useAppConfig,
  useStrategies,
  useUpdateStrategy,
  useTradingStatus,
  useStockSearch,
  useRefreshStockList,
  useTossMarketCalendar,
} from '../../../api/hooks'
import * as cmd from '../../../api/commands'
import type {
  AppConfigView,
  BrokerId,
  CmdError,
  LeveragedTrendHoldEntry,
  OverseasExchange,
  PriceConditionSymbolConfig,
  StockSearchItem,
  TossManualSession,
  TossMarketCalendarView,
  UpdateStrategyInput,
} from '../../../api/types'
import { BrokerScopeIndicator, tradingHealthProblem } from '../../../shared/ui'

type Market = 'KR' | 'US'

const EXCHANGE_SEARCH_ORDER: OverseasExchange[] = ['NAS', 'NYS', 'AMS']
const TOSS_US_SESSION_OPTIONS: Array<{ value: TossManualSession; label: string }> = [
  { value: 'auto', label: '자동' },
  { value: 'day', label: '데이' },
  { value: 'pre', label: '프리' },
  { value: 'regular', label: '정규' },
  { value: 'after', label: '애프터' },
]

function isDomesticSymbol(symbol: string) {
  return symbol.length === 6 && /^[0-9]/.test(symbol)
}

function isTossManualSession(value: unknown): value is TossManualSession {
  return value === 'auto' || value === 'day' || value === 'pre' || value === 'regular' || value === 'after'
}

function tossSessionLabel(session: TossManualSession) {
  return TOSS_US_SESSION_OPTIONS.find((option) => option.value === session)?.label ?? session
}

function fmtTossSessionWindow(session: TossMarketCalendarView['us']['regularSession']) {
  if (!session) return '오늘 세션 없음'
  const format = (value: string) => new Date(value).toLocaleTimeString('ko-KR', {
    hour: '2-digit',
    minute: '2-digit',
  })
  return `${format(session.startTime)}~${format(session.endTime)}`
}

function tossUsSessionWindow(
  calendar: TossMarketCalendarView | undefined,
  session: TossManualSession,
) {
  if (!calendar) return null
  if (session === 'day') return calendar.us.daySession
  if (session === 'pre') return calendar.us.preSession
  if (session === 'regular') return calendar.us.regularSession
  if (session === 'after') return calendar.us.afterSession
  return null
}

function isTossUsSessionOpen(
  calendar: TossMarketCalendarView | undefined,
  session: TossManualSession,
) {
  if (!calendar) return false
  if (session === 'day') return calendar.us.isDayOpen
  if (session === 'pre') return calendar.us.isPreOpen
  if (session === 'regular') return calendar.us.isRegularOpen
  if (session === 'after') return calendar.us.isAfterOpen
  return calendar.us.isDayOpen || calendar.us.isPreOpen || calendar.us.isRegularOpen || calendar.us.isAfterOpen
}

function withTossUsSessionParam(params: Record<string, unknown>, session: TossManualSession) {
  return { ...params, toss_us_session: session }
}

function brokerLabel(brokerId: BrokerId) {
  return brokerId === 'toss' ? 'Toss' : 'KIS'
}

function isActiveStrategyScope(
  strategy: { brokerId: BrokerId; brokerAccountId: string | null },
  appConfig?: AppConfigView | null,
) {
  return (
    strategy.brokerId === appConfig?.active_broker_id &&
    strategy.brokerAccountId === appConfig?.active_broker_account_id
  )
}

type EditState = { symbols: string[]; quantity: number; params: Record<string, number> }

// ─── Strategy 메인 ────────────────────────────────────────────────
export default function Strategy() {
  const { data: appConfig } = useAppConfig()
  const activeScopeKey = appConfig?.active_profile_id
    ? `${appConfig.active_profile_id}:${appConfig.active_broker_id}:${appConfig.active_broker_account_id ?? ''}`
    : null
  const {
    data: strategies,
    isLoading,
    isError: isStrategiesError,
    error: strategiesError,
    refetch: refetchStrategies,
  } = useStrategies(activeScopeKey)
  const { data: tradingStatus } = useTradingStatus()
  const {
    mutate: updateStrategy,
    isPending: saving,
    error: updateError,
    reset: resetUpdateError,
  } = useUpdateStrategy()

  const [editMap, setEditMap] = useState<Record<string, EditState>>({})
  // 가격 조건 매매 전략 전용: 종목별 설정 배열
  const [pcEditMap, setPcEditMap] = useState<Record<string, PriceConditionSymbolConfig[]>>({})
  // 레버리지 추세 보유 전략 전용: 레버리지/기초 매핑 배열
  const [lthEditMap, setLthEditMap] = useState<Record<string, LeveragedTrendHoldEntry[]>>({})
  const [lthParamEditMap, setLthParamEditMap] = useState<Record<string, Record<string, unknown>>>({})
  const [tossSessionEditMap, setTossSessionEditMap] = useState<Record<string, TossManualSession>>({})

  useEffect(() => {
    setEditMap({})
    setPcEditMap({})
    setLthEditMap({})
    setLthParamEditMap({})
    setTossSessionEditMap({})
    setSelectedStock(null)
    setSearchInput('')
    setSearchQuery('')
    setShowSearch(false)
    setSymbolNames({})
    resetUpdateError()
  }, [activeScopeKey, resetUpdateError])

  const expectedScope = appConfig?.active_profile_id
    ? {
        expectedProfileId: appConfig.active_profile_id,
        expectedBrokerId: appConfig.active_broker_id,
        expectedBrokerAccountId: appConfig.active_broker_account_id,
      }
    : null

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
  const { data: tossCalendar } = useTossMarketCalendar({ enabled: appConfig?.active_broker_id === 'toss' })
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
  /** 해외(미국) 거래소 자동 감지: NAS → NYS → AMS 순서로 조회 (활성 프로파일이 Toss면 Toss 종목 정보로 검증) */
  const handleUsSearch = async () => {
    const ticker = searchInput.trim().toUpperCase()
    if (!ticker) return
    setUsSearching(true)
    setUsSearchError(null)

    if (appConfig?.active_broker_id === 'toss') {
      try {
        const safety = await cmd.getTossStockSafety(ticker)
        if (safety.stockInfo) {
          const item: StockSearchItem = { pdno: ticker, prdt_name: safety.stockInfo.name || ticker }
          setSelectedStock(item)
          setSearchInput(safety.stockInfo.name || ticker)
          setSymbolNames(prev => ({ ...prev, [ticker]: safety.stockInfo!.name || ticker }))
          setUsSearching(false)
          return
        }
      } catch { /* fallback 메시지로 처리 */ }
      setUsSearchError(`Toss 종목 정보에서 "${ticker}"을 찾을 수 없습니다.`)
      setUsSearching(false)
      return
    }

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

  const getSavedTossSession = (params: Record<string, unknown>): TossManualSession => {
    return isTossManualSession(params.toss_us_session) ? params.toss_us_session : 'auto'
  }

  const getTossSession = (id: string, params: Record<string, unknown>): TossManualSession => {
    return tossSessionEditMap[id] ?? getSavedTossSession(params)
  }

  const setTossSession = (id: string, params: Record<string, unknown>, session: TossManualSession) => {
    const saved = getSavedTossSession(params)
    setTossSessionEditMap((prev) => {
      const next = { ...prev }
      if (session === saved) {
        delete next[id]
      } else {
        next[id] = session
      }
      return next
    })
  }

  const hasTossSessionEdit = (id: string, params: Record<string, unknown>) => {
    return tossSessionEditMap[id] !== undefined && tossSessionEditMap[id] !== getSavedTossSession(params)
  }

  const strategyHasUsTargets = (
    type: string,
    strategy: { targetSymbols: string[]; params: Record<string, unknown> },
    pcSymbols: PriceConditionSymbolConfig[],
    lthEntries: LeveragedTrendHoldEntry[],
  ) => {
    if (type === 'price_condition') return pcSymbols.some((symbol) => symbol.is_overseas)
    if (type === 'leveraged_trend_hold') return lthEntries.some((entry) => entry.is_overseas)
    return strategy.targetSymbols.some((symbol) => !isDomesticSymbol(symbol))
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
    if (!expectedScope) return
    updateStrategy({ id, enabled, ...expectedScope } satisfies UpdateStrategyInput)
  }

  const handleSave = (id: string, strategy: { params: Record<string, unknown> }) => {
    if (!expectedScope) return
    const edit = editMap[id]
    const session = getTossSession(id, strategy.params)
    const mergedParams = edit
      ? { ...strategy.params, ...edit.params }
      : { ...strategy.params }
    updateStrategy(
      {
        id,
        ...expectedScope,
        ...(edit ? { targetSymbols: edit.symbols, orderQuantity: edit.quantity } : {}),
        params: withTossUsSessionParam(mergedParams, session),
      } satisfies UpdateStrategyInput,
      {
        onSuccess: () => {
          setEditMap(prev => { const n = { ...prev }; delete n[id]; return n })
          setTossSessionEditMap(prev => { const n = { ...prev }; delete n[id]; return n })
        },
      },
    )
  }

  const handleSavePc = (
    id: string,
    params: Record<string, unknown>,
    initialSymbols: PriceConditionSymbolConfig[],
  ) => {
    if (!expectedScope) return
    const pcSymbols = pcEditMap[id] ?? initialSymbols
    const session = getTossSession(id, params)
    updateStrategy(
      {
        id,
        ...expectedScope,
        targetSymbols: pcSymbols.map((s) => s.symbol),
        params: withTossUsSessionParam({ ...params, symbols: pcSymbols }, session),
      } satisfies UpdateStrategyInput,
      {
        onSuccess: () => {
          setPcEditMap(prev => { const n = { ...prev }; delete n[id]; return n })
          setTossSessionEditMap(prev => { const n = { ...prev }; delete n[id]; return n })
        },
      },
    )
  }

  const handleSaveLth = (id: string, params: Record<string, unknown>, initialEntries: LeveragedTrendHoldEntry[]) => {
    if (!expectedScope) return
    const entries = lthEditMap[id] ?? initialEntries
    const nextParams = lthParamEditMap[id] ?? params
    if (hasInvalidLthEntries(entries)) return
    const targetSymbols = Array.from(new Set(entries.map((entry) => entry.leveraged_symbol).filter(Boolean)))
    updateStrategy(
      {
        id,
        ...expectedScope,
        targetSymbols,
        params: withTossUsSessionParam(
          { ...nextParams, entries },
          getTossSession(id, nextParams),
        ),
      } satisfies UpdateStrategyInput,
      {
        onSuccess: () => {
          setLthEditMap(prev => { const n = { ...prev }; delete n[id]; return n })
          setLthParamEditMap(prev => { const n = { ...prev }; delete n[id]; return n })
          setTossSessionEditMap(prev => { const n = { ...prev }; delete n[id]; return n })
        },
      },
    )
  }

  const activeCount = strategies?.filter(s => s.enabled).length ?? 0
  const isRunning = tradingStatus?.isRunning ?? false
  const activeBrokerIsToss = appConfig?.active_broker_id === 'toss'

  if (isLoading) {
    return <Box sx={{ display: 'flex', justifyContent: 'center', pt: 8 }}><CircularProgress /></Box>
  }

  if (isStrategiesError) {
    const message = (strategiesError as { message?: string } | null)?.message ?? '전략 목록을 불러오지 못했습니다.'
    return (
      <Alert
        severity="error"
        action={<Button color="inherit" size="small" onClick={() => void refetchStrategies()}>다시 시도</Button>}
      >
        {message}
      </Alert>
    )
  }

  return (
    <Box>
      {updateError && (
        <Alert severity="error" onClose={resetUpdateError} sx={{ mb: 2 }}>
          {(updateError as { message?: string }).message ?? '전략 변경사항을 저장하지 못했습니다.'}
        </Alert>
      )}
      {(strategies?.length ?? 0) === 0 && (
        <Alert severity="info" sx={{ mb: 2 }}>현재 계좌 범위에 등록된 전략이 없습니다.</Alert>
      )}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 3, flexWrap: 'wrap' }}>
        <Typography variant="h5" fontWeight={700}>Strategy</Typography>
        <Chip
          label={`${activeCount}개 활성`}
          color={activeCount > 0 ? 'success' : 'default'}
          size="small"
        />
        {isRunning && (
          <Chip label="자동매매 실행 중" color="success" size="small" variant="outlined" />
        )}
        <BrokerScopeIndicator appConfig={appConfig} compact />
      </Box>

      {tradingHealthProblem(tradingStatus?.health) && (
        <Alert severity="error" sx={{ mb: 2 }}>
          자동매매 상태 확인 필요: {tradingHealthProblem(tradingStatus?.health)}
        </Alert>
      )}

      {activeBrokerIsToss && !isRunning && (
        <Alert severity="info" sx={{ mb: 2 }}>
          Toss 프로파일은 실거래 동의가 저장된 경우 자동매매 주문/체결 확인 경로를 사용합니다. 시작 전 Dashboard 또는 Trading에서 소액 주문 검증을 먼저 확인하세요.
        </Alert>
      )}

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
          const sType = getStrategyType(s.id)
          const paramMetas = STRATEGY_PARAM_META[sType] ?? []
          const stratDesc = STRATEGY_DESCRIPTION[sType]
          const pcInitialSymbols = (s.params['symbols'] as PriceConditionSymbolConfig[] | undefined) ?? []
          const lthInitialEntries = (s.params['entries'] as LeveragedTrendHoldEntry[] | undefined) ?? []
          const lthParams = lthParamEditMap[s.id] ?? s.params
          const pcSessionSymbols = pcEditMap[s.id] ?? pcInitialSymbols
          const lthSessionEntries = lthEditMap[s.id] ?? lthInitialEntries
          const sessionParams = sType === 'leveraged_trend_hold' ? lthParams : s.params
          const previewSymbolNames = { ...s.targetSymbolNames, ...symbolNames }
          const tossSession = getTossSession(s.id, sessionParams)
          const showTossSession = activeBrokerIsToss && strategyHasUsTargets(sType, s, pcSessionSymbols, lthSessionEntries)
          const selectedTossSessionWindow = tossUsSessionWindow(tossCalendar, tossSession)
          const selectedTossSessionOpen = isTossUsSessionOpen(tossCalendar, tossSession)
          const sessionDirty = hasTossSessionEdit(s.id, sessionParams)
          const isDirty = !!editMap[s.id] || sessionDirty
          const scopeMatchesActive = isActiveStrategyScope(s, appConfig)
          const saveAction = sType === 'price_condition' ? (
            (pcEditMap[s.id] !== undefined || sessionDirty) && !s.enabled ? (
              <Button
                size="small"
                variant="outlined"
                startIcon={saving ? <CircularProgress size={14} /> : <SaveIcon />}
                onClick={() => handleSavePc(s.id, s.params, pcInitialSymbols)}
                disabled={saving || !scopeMatchesActive}
              >
                변경사항 저장
              </Button>
            ) : null
          ) : sType === 'leveraged_trend_hold' ? (
            (lthEditMap[s.id] !== undefined || lthParamEditMap[s.id] !== undefined || sessionDirty) && !s.enabled ? (
              <Button
                size="small"
                variant="outlined"
                startIcon={saving ? <CircularProgress size={14} /> : <SaveIcon />}
                onClick={() => handleSaveLth(s.id, s.params, lthInitialEntries)}
                disabled={saving || !scopeMatchesActive || hasInvalidLthEntries(lthEditMap[s.id] ?? lthInitialEntries)}
              >
                변경사항 저장
              </Button>
            ) : null
          ) : (
            isDirty && !s.enabled ? (
              <Button
                size="small"
                variant="outlined"
                startIcon={saving ? <CircularProgress size={14} /> : <SaveIcon />}
                onClick={() => handleSave(s.id, s)}
                disabled={saving || !scopeMatchesActive}
              >
                변경사항 저장
              </Button>
            ) : null
          )
          return (
            <Grid item xs={12} key={`${activeScopeKey}:${s.id}`} data-testid="strategy-card-grid">
              <Paper sx={{ p: { xs: 2, sm: 3 } }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 1, gap: 1.5 }}>
                  <Stack direction="row" alignItems="center" gap={0.75} flexWrap="wrap">
                    <Typography variant="subtitle1" fontWeight={600}>{s.name}</Typography>
                    <Chip
                      size="small"
                      label={`${brokerLabel(s.brokerId)}${s.brokerAccountId ? ` · ${s.brokerAccountId}` : ''}`}
                      color={scopeMatchesActive ? 'default' : 'warning'}
                      variant="outlined"
                      sx={{ maxWidth: '100%', '& .MuiChip-label': { overflow: 'hidden', textOverflow: 'ellipsis' } }}
                    />
                  </Stack>
                  <Stack
                    direction={{ xs: 'column', sm: 'row' }}
                    alignItems={{ xs: 'flex-end', sm: 'center' }}
                    spacing={1}
                    sx={{ flexShrink: 0 }}
                  >
                    {saveAction}
                    <FormControlLabel
                      control={
                        <Switch
                          checked={s.enabled}
                          onChange={(e) => handleToggle(s.id, e.target.checked)}
                          color="success"
                          disabled={saving || !scopeMatchesActive}
                        />
                      }
                      label={s.enabled ? '실행 중' : '정지'}
                      labelPlacement="start"
                      sx={{ m: 0 }}
                    />
                  </Stack>
                </Box>
                <Divider sx={{ mb: 2 }} />

              <Stack spacing={2}>
                  {showTossSession && (
                    <Box
                      sx={{
                        p: 1.25,
                        border: 1,
                        borderColor: 'divider',
                        borderRadius: 1,
                        bgcolor: 'action.hover',
                      }}
                    >
                      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1} sx={{ mb: 1 }}>
                        <Typography variant="caption" fontWeight={700}>
                          Toss 미국 자동매매 세션
                        </Typography>
                        <Chip
                          size="small"
                          label={selectedTossSessionOpen ? '현재 가능' : '현재 아님'}
                          color={selectedTossSessionOpen ? 'success' : 'default'}
                          variant="outlined"
                          sx={{ height: 20, fontSize: '0.68rem' }}
                        />
                      </Stack>
                      <ToggleButtonGroup
                        value={tossSession}
                        exclusive
                        onChange={(_, v: TossManualSession | null) => v && setTossSession(s.id, sessionParams, v)}
                        fullWidth
                        size="small"
                        disabled={s.enabled || saving}
                        sx={{ mb: 0.75 }}
                      >
                        {TOSS_US_SESSION_OPTIONS.map((option) => (
                          <ToggleButton key={option.value} value={option.value}>
                            {option.label}
                          </ToggleButton>
                        ))}
                      </ToggleButtonGroup>
                      <Typography variant="caption" color="text.secondary">
                        {tossSession === 'auto'
                          ? 'Toss US 데이/프리/정규/애프터 중 열려 있는 세션에서 전략 틱과 주문을 허용합니다.'
                          : `${tossSessionLabel(tossSession)} ${fmtTossSessionWindow(selectedTossSessionWindow)} · ` +
                            (selectedTossSessionOpen ? '선택 세션이 열려 있습니다.' : '선택 세션 시간이 아닙니다.')}
                      </Typography>
                    </Box>
                  )}

                  {/* price_condition: 커스텀 편집 UI */}
                  {sType === 'price_condition' ? (
                    <>
                      <PriceConditionEditorPanel
                        stratEnabled={s.enabled}
                        initialSymbols={pcInitialSymbols}
                        editedSymbols={pcEditMap[s.id]}
                        selectedStock={selectedStock}
                        market={market}
                        onUpdate={(syms) => setPcEditMap((prev) => ({ ...prev, [s.id]: syms }))}
                      />
                      <StrategyPreviewPanel
                        strategyId={s.id}
                        strategyName={s.name}
                        brokerId={s.brokerId}
                        brokerAccountId={s.brokerAccountId}
                        symbols={pcSessionSymbols.map((symbol) => symbol.symbol)}
                        symbolNames={{
                          ...previewSymbolNames,
                          ...Object.fromEntries(pcSessionSymbols.map((symbol) => [symbol.symbol, symbol.symbol_name])),
                        }}
                        orderQuantity={s.orderQuantity}
                        params={withTossUsSessionParam({ ...s.params, symbols: pcSessionSymbols }, tossSession)}
                      />
                    </>
                  ) : sType === 'leveraged_trend_hold' ? (
                    <LeveragedTrendHoldEditorPanel
                      stratEnabled={s.enabled}
                      initialEntries={lthInitialEntries}
                      editedEntries={lthEditMap[s.id]}
                      params={lthParams}
                      onUpdate={(entries) => setLthEditMap((prev) => ({ ...prev, [s.id]: entries }))}
                      onParamsUpdate={(params) => setLthParamEditMap((prev) => ({ ...prev, [s.id]: params }))}
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
                                    aria-label={`${code} 삭제`}
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
                    <Grid item xs={12} sm={6} md={4}>
                      <TextField
                        label="1회 수량"
                        type="number"
                        value={edit.quantity}
                        onChange={(e) => setEdit(s.id, s, { quantity: Number(e.target.value) })}
                        size="small"
                        fullWidth
                        disabled={s.enabled}
                        inputProps={{ min: 1 }}
                      />
                    </Grid>
                    {paramMetas.map((meta) => (
                      <Grid item xs={12} sm={6} md={4} key={meta.key}>
                        <TextField
                          label={meta.label}
                          type="number"
                          title={meta.description}
                          value={edit.params[meta.key] ?? 0}
                          onChange={(e) => setParam(s.id, s, meta.key, Number(e.target.value))}
                          size="small"
                          fullWidth
                          disabled={s.enabled}
                          inputProps={{ min: meta.min, max: meta.max, step: meta.step ?? 1 }}
                        />
                      </Grid>
                    ))}
                  </Grid>
                  <StrategyPreviewPanel
                    strategyId={s.id}
                    strategyName={s.name}
                    brokerId={s.brokerId}
                    brokerAccountId={s.brokerAccountId}
                    symbols={edit.symbols}
                    symbolNames={previewSymbolNames}
                    orderQuantity={edit.quantity}
                    params={withTossUsSessionParam({ ...s.params, ...edit.params }, tossSession)}
                  />
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

              </Paper>
            </Grid>
          )
        })}
      </Grid>
    </Box>
  )
}
