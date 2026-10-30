// State, query gates and command closures remain in the route/editor React lifetime.
import { useEffect, useRef, useState } from 'react'
import * as cmd from '../../../api/commands'
import {
  useAppConfig,
  useBalance,
  useBrokerHoldings,
  useClearBuySuspension,
  useOverseasBalance,
  useOverseasPrice,
  usePlaceOrder,
  usePlaceOverseasOrder,
  usePrice,
  useRefreshStockList,
  useStockSearch,
  useTossMarketCalendar,
  useTossMarketSnapshot,
  useTossOrderPreflight,
  useTossStockSafety,
  useTradingStatus,
} from '../../../api/hooks'
import type {
  BalanceItem,
  BrokerHoldingView,
  CmdError,
  OrderSide,
  OrderType,
  OverseasBalanceItem,
  OverseasExchange,
  OverseasOrderExchange,
  TossManualSession,
  TossMarketCalendarView,
} from '../../../api/types'
import { fmtNumber } from '../../../shared/lib'

function fmt(n: number) {
  return fmtNumber(n)
}

function tossMarketLabel(market: 'kr' | 'us') {
  return market === 'kr' ? '국내' : '미국'
}

type Market = 'KR' | 'US'

const EXCHANGE_ORDER_MAP: Record<OverseasExchange, OverseasOrderExchange> = {
  NAS: 'NASD',
  NYS: 'NYSE',
  AMS: 'AMEX',
}

const PAPER_UNSUPPORTED_US_SELL_SYMBOLS = ['VOO', 'SPY', 'QQQM']

const TOSS_US_SESSION_OPTIONS: Array<{ value: TossManualSession; label: string }> = [
  { value: 'auto', label: '자동' },
  { value: 'day', label: '데이' },
  { value: 'pre', label: '프리' },
  { value: 'regular', label: '정규' },
  { value: 'after', label: '애프터' },
]

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
  return true
}

function normalizeExchange(code: string): OverseasExchange {
  const map: Record<string, OverseasExchange> = {
    NAS: 'NAS', NASD: 'NAS', NASDAQ: 'NAS',
    NYS: 'NYS', NYSE: 'NYS',
    AMS: 'AMS', AMEX: 'AMS',
  }
  return map[code.toUpperCase()] ?? 'NAS'
}

export function useTradingOrderModel() {
  const [market, setMarket]           = useState<Market>('KR')
  const [symbol, setSymbol]           = useState('')
  const [inputValue, setInputValue]   = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [showResults, setShowResults] = useState(false)
  const [usExchange, setUsExchange]   = useState<OverseasExchange>('NAS')
  const [usSearching, setUsSearching] = useState(false)
  const [side, setSide]               = useState<OrderSide>('Buy')
  const [orderType, setOrderType]     = useState<OrderType>('Limit')
  const [tossSession, setTossSession] = useState<TossManualSession>('auto')
  const [quantity, setQuantity]       = useState('')
  const [price, setPrice]             = useState('')
  const [result, setResult]           = useState<string | null>(null)
  const [errorMsg, setErrorMsg]       = useState<string | null>(null)
  // 검색 결과 테이블 마우스 다운 시 blur 이벤트 지연 취소용
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    setSymbol('')
    setInputValue('')
    setSearchQuery('')
    setShowResults(false)
    setPrice('')
    setQuantity('')
    setResult(null)
    setErrorMsg(null)
    setUsSearching(false)
    setTossSession('auto')
  }, [market])

  useEffect(() => {
    if (market !== 'KR' || !inputValue || !showResults) {
      if (!inputValue || !showResults) setSearchQuery('')
      return
    }
    const t = setTimeout(() => setSearchQuery(inputValue), 400)
    return () => clearTimeout(t)
  }, [inputValue, market, showResults])

  const { data: appConfig }                                         = useAppConfig()
  const isTossActive = appConfig?.active_broker_id === 'toss'
  const isKisActive = appConfig?.active_broker_id === 'kis'
  const balanceQuery = useBalance({ enabled: isKisActive })
  const overseasBalanceQuery = useOverseasBalance({ enabled: isKisActive })
  const { data: balance } = balanceQuery
  const { data: overseasBalance } = overseasBalanceQuery
  const { data: brokerHoldings = [],
          isLoading: brokerHoldingsLoading,
          isError: brokerHoldingsError }                            = useBrokerHoldings({ enabled: isTossActive })
  const { data: krPrice }                                           = usePrice(!isTossActive && market === 'KR' && symbol.length === 6 ? symbol : '')
  const { data: usPrice }                                           = useOverseasPrice(!isTossActive && market === 'US' ? symbol : '', !isTossActive && market === 'US' ? usExchange : '')
  const { data: tossSnapshot }                                      = useTossMarketSnapshot(isTossActive && symbol ? symbol : '')
  const { data: tossSafety, isLoading: isTossSafetyLoading,
          isError: isTossSafetyError, error: tossSafetyError }      = useTossStockSafety(isTossActive && symbol ? symbol : '')
  const { data: tossPreflight, isLoading: isTossPreflightLoading,
          isError: isTossPreflightError, error: tossPreflightError } = useTossOrderPreflight(
    {
      symbol,
      side,
      quantity,
      price: price || null,
    },
    { enabled: isTossActive && !!symbol && !!quantity },
  )
  const { data: tossCalendar, isLoading: isTossCalendarLoading,
          isError: isTossCalendarError }                            = useTossMarketCalendar({ enabled: isTossActive })
  const { data: searchResults = [], isFetching: isFetchingSearch,
          isError: isSearchError, error: searchError }              = useStockSearch(searchQuery)
  const { mutate: placeOrder,         isPending: isPendingKr }     = usePlaceOrder()
  const { mutate: placeOverseasOrder, isPending: isPendingUs }     = usePlaceOverseasOrder()
  const { mutate: doRefreshList,      isPending: isRefreshing }    = useRefreshStockList()
  const { data: tradingStatus }                                     = useTradingStatus()
  const { mutate: clearBuySuspension, isPending: clearingBuySusp } = useClearBuySuspension()
  const isPending = isPendingKr || isPendingUs

  // STOCK_LIST_EMPTY 에러 감지: KRX 다운로드 미완료 or 실패
  const isStockListEmpty = isSearchError && (searchError as CmdError | null)?.code === 'STOCK_LIST_EMPTY'

  const availableCash  = parseInt(balance?.summary?.dnca_tot_amt ?? '0') || 0
  const tossCurrentPrice = tossSnapshot ? Number(tossSnapshot.price.amount.replace(/,/g, '')) : null
  const krCurrentPrice = krPrice ? parseInt(krPrice.stck_prpr) : null
  const usCurrentPrice = usPrice ? parseFloat(usPrice.last) : null
  const stockName      = isTossActive
    ? (inputValue || symbol)
    : market === 'KR' ? krPrice?.hts_kor_isnm : (usPrice?.name ?? symbol)
  const isPaperTrading = appConfig?.kis_is_paper_trading ?? false
  const overseasOrderExchange = EXCHANGE_ORDER_MAP[usExchange]
  const normalizedUsSymbol = symbol.trim().toUpperCase()
  const isPaperUnsupportedUsSell =
    market === 'US' &&
    isPaperTrading &&
    side === 'Sell' &&
    (overseasOrderExchange === 'AMEX' || PAPER_UNSUPPORTED_US_SELL_SYMBOLS.includes(normalizedUsSymbol))
  const tossManualOrderReady = tossPreflight?.canSubmit ?? false
  const showTossUsSessionSelector = isTossActive && market === 'US'
  const selectedTossSessionWindow = tossUsSessionWindow(tossCalendar, tossSession)
  const selectedTossSessionOpen = isTossUsSessionOpen(tossCalendar, tossSession)

  const handleSelectHolding = (item: BalanceItem) => {
    setSymbol(item.pdno)
    setInputValue(item.prdt_name)
    setResult(null)
    setErrorMsg(null)
  }

  /** 해외 보유 종목 클릭 → US 모드로 전환 후 매도 폼 자동 완성 */
  const handleSelectOverseasHolding = (item: OverseasBalanceItem) => {
    setMarket('US')
    setSymbol(item.ovrs_pdno)
    setInputValue(item.ovrs_item_name)
    // ovrs_excg_cd: KIS는 잔고 API에서 'NAS'/'NYS'/'AMS' 반환
    // 방어적으로 'NASD'/'NYSE'/'AMEX' 장포맷도 OverseasExchange로 정규화
    setUsExchange(normalizeExchange(item.ovrs_excg_cd))
    setResult(null)
    setErrorMsg(null)
  }

  const handleSelectBrokerHolding = (item: BrokerHoldingView) => {
    setMarket(item.market === 'kr' ? 'KR' : 'US')
    setSymbol(item.symbol)
    setInputValue(item.symbolName || item.symbol)
    setResult(null)
    setErrorMsg(null)
  }

  /** 미국 주식 거래소 자동 감지: NAS → NYS → AMS 순서로 조회 */
  const handleUsSearch = async () => {
    const ticker = inputValue.trim().toUpperCase()
    if (!ticker) return
    if (isTossActive) {
      setSymbol(ticker)
      setResult(null)
      setErrorMsg(null)
      return
    }
    setUsSearching(true)
    setErrorMsg(null)
    const exchanges: OverseasExchange[] = ['NAS', 'NYS', 'AMS']
    for (const exc of exchanges) {
      try {
        const res = await cmd.getOverseasPrice(ticker, exc)
        // 가격 > 0 이거나 종목명이 있으면 유효 (NYSE Arca ETF 등 특수 케이스 포함)
        const validPrice = parseFloat(res.last) > 0
        const hasName = res.name && res.name.trim().length > 0
        if (res && (validPrice || hasName)) {
          setUsExchange(exc)
          setSymbol(ticker)
          setResult(null)
          setUsSearching(false)
          return
        }
      } catch { /* 다음 거래소 시도 */ }
    }
    setErrorMsg(`"${ticker}"을 NAS·NYS·AMEX에서 찾을 수 없습니다.`)
    setUsSearching(false)
  }

  const handleFillMarketPrice = () => {
    if (isTossActive && tossCurrentPrice && Number.isFinite(tossCurrentPrice)) {
      setPrice(tossSnapshot?.price.currency === 'KRW' ? String(Math.round(tossCurrentPrice)) : tossCurrentPrice.toFixed(4))
      return
    }
    if (market === 'KR' && krCurrentPrice) setPrice(String(krCurrentPrice))
    if (market === 'US' && usCurrentPrice)  setPrice(usCurrentPrice.toFixed(2))
  }

  const handleSubmit = () => {
    setResult(null)
    setErrorMsg(null)
    const qty = parseInt(quantity)
    if (!symbol)          { setErrorMsg('종목을 선택하세요.'); return }
    if (!qty || qty <= 0) { setErrorMsg('수량을 입력하세요.'); return }
    if (isTossActive && !tossManualOrderReady) {
      setErrorMsg(tossPreflight?.blockedReasons?.[0] ?? 'Toss 주문 전 사전검증을 통과해야 수동 주문을 제출할 수 있습니다.')
      return
    }
    if (isTossActive) {
      const prc = parseFloat(price || '0')
      if (orderType === 'Limit' && (!prc || prc <= 0)) {
        setErrorMsg('Toss 지정가 주문은 가격을 입력해야 합니다.')
        return
      }
      if (market === 'US' && tossSession !== 'auto') {
        if (!tossCalendar) {
          setErrorMsg('Toss 미국 세션 정보를 확인한 뒤 주문을 제출할 수 있습니다.')
          return
        }
        if (!selectedTossSessionOpen) {
          setErrorMsg(
            `현재 시간은 선택한 Toss 미국 ${tossSessionLabel(tossSession)} 세션이 아닙니다. ` +
            `세션 시간: ${fmtTossSessionWindow(selectedTossSessionWindow)}`
          )
          return
        }
      }
      placeOrder(
        {
          symbol,
          side,
          order_type: orderType,
          quantity: qty,
          price: orderType === 'Market' ? 0 : prc,
          toss_session: market === 'US' ? tossSession : null,
        },
        {
          onSuccess: (d) => {
            const odno = d.odno || '(접수됨)'
            setResult('Toss 주문 완료 — 주문번호: ' + odno)
            setQuantity('')
            setPrice('')
          },
          onError: (e) => {
            const err = e as { message?: string } | Error | null
            setErrorMsg(err instanceof Error ? err.message : (err as { message?: string })?.message ?? String(e))
          },
        },
      )
      return
    }

    if (market === 'KR') {
      if (!/^[A-Z0-9]{6}$/i.test(symbol)) { setErrorMsg('국내 종목코드는 6자리 영숫자입니다 (예: 005930, 0005A0).'); return }
      const prc = parseInt(price)
      if (orderType === 'Limit' && (!prc || prc <= 0)) {
        setErrorMsg('지정가 주문은 가격을 입력해야 합니다.')
        return
      }
      placeOrder(
        { symbol, side, order_type: orderType, quantity: qty, price: orderType === 'Market' ? 0 : prc },
        {
          onSuccess: (d) => {
            const odno = d.odno || '(접수됨)'
            setResult('주문 완료 — 주문번호: ' + odno)
            setQuantity('')
            setPrice('')
          },
          onError:   (e) => {
            const err = e as { message?: string } | Error | null
            setErrorMsg(err instanceof Error ? err.message : (err as { message?: string })?.message ?? String(e))
          },
        },
      )
    } else {
      const prc = parseFloat(price)
      if (!prc || prc <= 0) { setErrorMsg('해외 주문은 USD 가격을 입력해야 합니다.'); return }
      if (isPaperUnsupportedUsSell) {
        setErrorMsg(
          `모의투자 미지원 사전 검증: ${normalizedUsSymbol} (${overseasOrderExchange}) 매도 주문은 ` +
          'KIS 모의투자 해외 주문 universe에서 제한될 수 있습니다. 실전투자로 전환하거나 NASD/NYSE 지원 종목으로 검증하세요.'
        )
        return
      }
      placeOverseasOrder(
        { symbol, exchange: overseasOrderExchange, side, quantity: qty, price: prc },
        {
          onSuccess: (d) => {
            const odno = d.odno || '(접수됨)'
            setResult('주문 완료 — 주문번호: ' + odno)
            setQuantity('')
            setPrice('')
          },
          onError: (e) => {
            const err = e as { message?: string } | Error | null
            const rawMsg = err instanceof Error ? err.message : (err as { message?: string })?.message ?? String(e)
            if (
              rawMsg.includes('해당업무가 제공되지 않습니다') ||
              rawMsg.includes('모의투자 미지원') ||
              rawMsg.includes('PAPER_OVERSEAS_UNSUPPORTED')
            ) {
              setErrorMsg(
                `모의투자 미지원: ${rawMsg}\n` +
                `이 종목(${symbol}) 또는 거래소(${overseasOrderExchange})는 모의투자에서 매도 주문이 지원되지 않습니다. ` +
                `실전투자로 전환하거나 NASD/NYSE 종목을 이용하세요.`
              )
            } else {
              setErrorMsg(rawMsg)
            }
          },
        },
      )
    }
  }

  return {
    TOSS_US_SESSION_OPTIONS, appConfig, availableCash, balance, balanceQuery,
    brokerHoldings, brokerHoldingsError, brokerHoldingsLoading, clearBuySuspension,
    clearingBuySusp, closeTimerRef, doRefreshList, errorMsg,
    fmt, fmtTossSessionWindow, handleFillMarketPrice, handleSelectBrokerHolding,
    handleSelectHolding, handleSelectOverseasHolding, handleSubmit, handleUsSearch,
    inputValue, isFetchingSearch, isPaperTrading, isPaperUnsupportedUsSell,
    isPending, isRefreshing, isStockListEmpty, isTossActive,
    isTossCalendarError, isTossCalendarLoading, isTossPreflightError, isTossPreflightLoading,
    isTossSafetyError, isTossSafetyLoading, krCurrentPrice, market,
    normalizedUsSymbol, orderType, overseasBalance, overseasBalanceQuery, overseasOrderExchange,
    price, quantity, result, searchQuery,
    searchResults, selectedTossSessionOpen, selectedTossSessionWindow, setErrorMsg,
    setInputValue, setMarket, setOrderType, setPrice,
    setQuantity, setResult, setSearchQuery, setShowResults,
    setSide, setSymbol, setTossSession, showResults,
    showTossUsSessionSelector, side, stockName, symbol,
    tossCalendar, tossCurrentPrice, tossManualOrderReady, tossMarketLabel,
    tossPreflight, tossPreflightError, tossSafety, tossSafetyError,
    tossSession, tossSessionLabel, tradingStatus, usCurrentPrice,
    usExchange, usSearching,
  }
}

export type TradingOrderModel = ReturnType<typeof useTradingOrderModel>
