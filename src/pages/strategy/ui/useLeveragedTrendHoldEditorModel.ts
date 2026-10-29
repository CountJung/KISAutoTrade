// State, query gates and command closures remain in the route/editor React lifetime.
import {
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react'
import * as cmd from '../../../api/commands'
import {
  useAppConfig,
  usePreviewLeveragedTrendHold,
  useRefreshStockList,
  useStockSearch,
} from '../../../api/hooks'
import type {
  CmdError,
  LeveragedTrendHoldEntry,
  LeveragedTrendHoldPreviewView,
  OverseasExchange,
  SimulationAssumptions,
  StockSearchItem,
} from '../../../api/types'
import { defaultSimulationAssumptions, rebaseDefaultMarketCosts } from './strategyResearchPanel'

export type Market = 'KR' | 'US'

export type PreviewInterval = '1m' | '1d'

export type PreviewCount = 50 | 100 | 200

type TargetDraftSelection = {
  stock: StockSearchItem
  market: Market
}

export type LeveragedTrendHoldEditorPanelProps = {
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

export function useLeveragedTrendHoldEditorModel(props: LeveragedTrendHoldEditorPanelProps) {
  const { stratEnabled, initialEntries, editedEntries, params, onUpdate } = props
  const entries = editedEntries ?? initialEntries
  const initialPreviewIsOverseas = (entries.find((entry) => entry.is_overseas) ?? entries[0])?.is_overseas ?? true
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
  const [previewInterval, setPreviewInterval] = useState<PreviewInterval>('1m')
  const [previewCount, setPreviewCount] = useState<PreviewCount>(200)
  const [preview, setPreview] = useState<LeveragedTrendHoldPreviewView | null>(null)
  const [assumptions, setAssumptions] = useState<SimulationAssumptions>(() => defaultSimulationAssumptions(initialPreviewIsOverseas))
  const assumptionsMarket = useRef(initialPreviewIsOverseas)
  const [accountingError, setAccountingError] = useState<string | null>(null)
  const [accountingDraftRevision, setAccountingDraftRevision] = useState(0)
  const previewGeneration = useRef(0)
  const bollingerEnabled = (params.bollinger as { enabled?: boolean } | undefined)?.enabled === true
  useEffect(() => { if (bollingerEnabled) setPreviewInterval('1m') }, [bollingerEnabled])
  const entrySensitivity = numericParam(params, 'upward_sensitivity', 1)
  const reboundEnabled = boolParam(params, 'intraday_rebound_enabled', false)
  const reboundBaselineTicks = numericParam(params, 'rebound_baseline_ticks', 8)
  const reboundConfirmTicks = numericParam(params, 'rebound_confirm_ticks', 3)
  const reboundPullback = numericParam(params, 'rebound_pullback_pct', 4)
  const reboundBuyPressure = numericParam(params, 'rebound_buy_pressure_pct', 1.5)
  const reboundRsiMin = numericParam(params, 'rebound_rsi_min', 30)
  const rapidReboundEnabled = boolParam(params, 'rapid_rebound_enabled', false)
  const rapidReboundLookbackTicks = numericParam(params, 'rapid_rebound_lookback_ticks', 8)
  const rapidReboundDrop = numericParam(params, 'rapid_rebound_drop_pct', 2)
  const rapidReboundRecovery = numericParam(params, 'rapid_rebound_recovery_pct', 1.2)
  const rapidReboundMaxLowAgeTicks = numericParam(params, 'rapid_rebound_max_low_age_ticks', 3)
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
  const currentPreview = preview
    && preview.symbol === previewEntry?.leveraged_symbol
    && preview.interval === previewInterval
    ? preview
    : null
  useEffect(() => {
    const nextIsOverseas = previewEntry?.is_overseas ?? assumptionsMarket.current
    setAssumptions((current) => rebaseDefaultMarketCosts(current, assumptionsMarket.current, nextIsOverseas))
    assumptionsMarket.current = nextIsOverseas
  }, [previewEntry?.is_overseas])
  const previewInputKey = JSON.stringify({
    symbol: previewEntry?.leveraged_symbol ?? '',
    interval: previewInterval,
    count: previewCount,
    params,
    entries,
    assumptions, accountingDraftRevision,
    activeProfileId: appConfig?.active_profile_id,
    activeBrokerAccountId: appConfig?.active_broker_account_id,
  })
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

  useEffect(() => {
    previewGeneration.current += 1
    setPreview(null)
    previewMutation.reset()
  }, [previewInputKey]) // eslint-disable-line react-hooks/exhaustive-deps -- serialized preview input is the invalidation boundary

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

  const handlePreview = async () => {
    if (!previewEntry || accountingError) return
    const requestGeneration = previewGeneration.current
    try {
      const result = await previewMutation.mutateAsync({
        symbol: previewEntry.leveraged_symbol,
        params: { ...params, entries },
        interval: previewInterval,
        count: previewCount,
        assumptions,
        expectedProfileId: appConfig?.active_profile_id,
        expectedBrokerAccountId: appConfig?.active_broker_account_id,
      })
      if (requestGeneration !== previewGeneration.current) return
      setPreview(result)
    } catch {
      if (requestGeneration !== previewGeneration.current) return
      setPreview(null)
    }
  }

  return {
    accountingError, appConfig, assumptions, bollingerEnabled,
    breakevenBuffer, canAddTarget, currentPreview, doPickerRefreshList,
    draftQuantity, entries, entryFailureObservations, entrySensitivity,
    handleAddTarget, handlePickerSelect, handlePickerUsSearch, handlePreview,
    handleQuantity, handleRemoveTarget, handleSensitivityChange, initialStopLoss,
    isTossActive, minHoldObservations, params, pickerCloseTimer,
    pickerError, pickerFetching, pickerInput, pickerMarket,
    pickerOpen, pickerRefreshing, pickerResults, pickerSearching,
    pickerSelection, pickerStockListEmpty, previewCount, previewEntry,
    previewGeneration, previewInterval, previewMutation, previewOptions,
    previewSymbol, props, rapidReboundDrop, rapidReboundEnabled,
    rapidReboundLookbackTicks, rapidReboundMaxLowAgeTicks, rapidReboundRecovery, reboundBaselineTicks,
    reboundBuyPressure, reboundConfirmTicks, reboundEnabled, reboundPullback,
    reboundRsiMin, setAccountingDraftRevision, setAccountingError, setAssumptions,
    setDraftQuantity, setPickerError, setPickerInput, setPickerMarket,
    setPickerOpen, setPickerQuery, setPickerSelection, setPreview,
    setPreviewCount, setPreviewInterval, setPreviewSymbol, stratEnabled,
    trailingActivationProfit, trailingStopPct, updateIntegerParam, updateNumericParam,
  }
}

export type LeveragedTrendHoldEditorModel = ReturnType<typeof useLeveragedTrendHoldEditorModel>
