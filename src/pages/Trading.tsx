/**
 * Trading 페이지 — KR / US 주식, 모바일 반응형
 */
import { useEffect, useRef, useState } from 'react'
import Typography from '@mui/material/Typography'
import Paper from '@mui/material/Paper'
import Box from '@mui/material/Box'
import Grid from '@mui/material/Grid'
import TextField from '@mui/material/TextField'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import InputAdornment from '@mui/material/InputAdornment'
import Alert from '@mui/material/Alert'
import CircularProgress from '@mui/material/CircularProgress'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Chip from '@mui/material/Chip'
import Divider from '@mui/material/Divider'
import Tooltip from '@mui/material/Tooltip'
import TrendingUpIcon from '@mui/icons-material/TrendingUp'
import TrendingDownIcon from '@mui/icons-material/TrendingDown'
import RefreshIcon from '@mui/icons-material/Refresh'
import SearchIcon from '@mui/icons-material/Search'
import PublicIcon from '@mui/icons-material/Public'
import FlagIcon from '@mui/icons-material/Flag'
import IconButton from '@mui/material/IconButton'

import {
  useBalance,
  useOverseasBalance,
  usePlaceOrder,
  usePrice,
  useStockSearch,
  useOverseasPrice,
  usePlaceOverseasOrder,
  useRefreshStockList,
  useTradingStatus,
  useClearBuySuspension,
} from '../api/hooks'
import * as cmd from '../api/commands'
import type {
  BalanceItem,
  OverseasBalanceItem,
  CmdError,
  OverseasExchange,
  OverseasOrderExchange,
  OrderSide,
  OrderType,
} from '../api/types'
import { StockChart } from '../components/chart/StockChart'
import { OverseasStockChart } from '../components/chart/OverseasStockChart'

function fmt(n: number) {
  return n.toLocaleString('ko-KR')
}

type Market = 'KR' | 'US'

const EXCHANGE_ORDER_MAP: Record<OverseasExchange, OverseasOrderExchange> = {
  NAS: 'NASD',
  NYS: 'NYSE',
  AMS: 'AMEX',
}

/**
 * KIS 잔고 API의 ovrs_excg_cd (NAS/NYS/AMS 또는 NASD/NYSE/AMEX 모두 처리)
 * → 프론트엔드 OverseasExchange ('NAS' | 'NYS' | 'AMS') 로 정규화
 */
function normalizeExchange(code: string): OverseasExchange {
  const map: Record<string, OverseasExchange> = {
    NAS: 'NAS', NASD: 'NAS', NASDAQ: 'NAS',
    NYS: 'NYS', NYSE: 'NYS',
    AMS: 'AMS', AMEX: 'AMS',
  }
  return map[code.toUpperCase()] ?? 'NAS'
}

// --- 보유 종목 테이블 (국내)
interface HoldingsTableProps {
  items: BalanceItem[]
  onSelect: (item: BalanceItem) => void
}

function HoldingsTable({ items, onSelect }: HoldingsTableProps) {
  if (items.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
        보유 종목 없음
      </Typography>
    )
  }
  return (
    <TableContainer sx={{ maxHeight: { xs: 200, sm: 260 }, overflowX: 'auto' }}>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell sx={{ minWidth: 120 }}>종목</TableCell>
            <TableCell align="right" sx={{ minWidth: 70 }}>수량</TableCell>
            <TableCell align="right" sx={{ minWidth: 90 }}>현재가</TableCell>
            <TableCell align="right" sx={{ minWidth: 90 }}>손익률</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {items.map((item) => {
            const pfls = parseFloat(item.evlu_pfls_rt)
            const positive = pfls >= 0
            return (
              <TableRow
                key={item.pdno}
                hover
                onClick={() => onSelect(item)}
                sx={{ cursor: 'pointer' }}
              >
                <TableCell>
                  <Typography variant="body2" noWrap sx={{ maxWidth: { xs: 130, sm: 'none' } }}>
                    {item.prdt_name}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">{item.pdno}</Typography>
                </TableCell>
                <TableCell align="right">{fmt(parseInt(item.hldg_qty))}주</TableCell>
                <TableCell align="right" sx={{ whiteSpace: 'nowrap' }}>
                  {fmt(parseInt(item.prpr))}원
                </TableCell>
                <TableCell align="right">
                  <Typography
                    variant="body2"
                    color={positive ? 'success.main' : 'error.main'}
                    fontWeight={600}
                    noWrap
                  >
                    {positive ? '+' : ''}{pfls.toFixed(2)}%
                  </Typography>
                  <Typography
                    variant="caption"
                    color={positive ? 'success.main' : 'error.main'}
                    noWrap
                  >
                    {positive ? '+' : ''}{fmt(parseInt(item.evlu_pfls_amt))}원
                  </Typography>
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </TableContainer>
  )
}

// --- 해외 보유 종목 테이블
interface OverseasHoldingsTableProps {
  items: OverseasBalanceItem[]
  onSelect: (item: OverseasBalanceItem) => void
}
function OverseasHoldingsTable({ items, onSelect }: OverseasHoldingsTableProps) {
  if (items.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
        해외 보유 종목 없음
      </Typography>
    )
  }
  return (
    <TableContainer sx={{ maxHeight: { xs: 200, sm: 260 }, overflowX: 'auto' }}>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell sx={{ minWidth: 120 }}>종목</TableCell>
            <TableCell align="right" sx={{ minWidth: 60 }}>수량</TableCell>
            <TableCell align="right" sx={{ minWidth: 90 }}>현재가(USD)</TableCell>
            <TableCell align="right" sx={{ minWidth: 90 }}>손익률</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {items.map((item) => {
            const pfls = parseFloat(item.evlu_pfls_rt)
            const positive = pfls >= 0
            return (
              <TableRow
                key={item.ovrs_pdno}
                hover
                onClick={() => onSelect(item)}
                sx={{ cursor: 'pointer' }}
              >
                <TableCell>
                  <Typography variant="body2" noWrap sx={{ maxWidth: { xs: 130, sm: 'none' } }}>
                    {item.ovrs_item_name}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {item.ovrs_pdno} · {item.ovrs_excg_cd}
                  </Typography>
                </TableCell>
                <TableCell align="right">{item.ovrs_cblc_qty}</TableCell>
                <TableCell align="right" sx={{ whiteSpace: 'nowrap' }}>
                  ${parseFloat(item.now_pric2).toFixed(2)}
                </TableCell>
                <TableCell align="right">
                  <Typography
                    variant="body2"
                    color={positive ? 'success.main' : 'error.main'}
                    fontWeight={600}
                    noWrap
                  >
                    {positive ? '+' : ''}{pfls.toFixed(2)}%
                  </Typography>
                  <Typography
                    variant="caption"
                    color={positive ? 'success.main' : 'error.main'}
                    noWrap
                  >
                    {positive ? '+' : ''}${parseFloat(item.frcr_evlu_pfls_amt).toFixed(2)}
                  </Typography>
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </TableContainer>
  )
}

// --- 국내 종목 정보 카드
function KrStockInfoCard({ symbol }: { symbol: string }) {
  const { data: p, isLoading } = usePrice(symbol.length === 6 ? symbol : '')
  if (!p && !isLoading) return null
  if (isLoading) return (
    <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
      <CircularProgress size={16} />
      <Typography variant="body2" color="text.secondary">조회 중</Typography>
    </Box>
  )
  if (!p) return null

  const price  = parseInt(p.stck_prpr)
  const change = parseInt(p.prdy_vrss)
  const rt     = parseFloat(p.prdy_ctrt)
  const pos    = rt >= 0

  return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, py: 1.5 }}>
      <Stack direction="row" spacing={1.5} alignItems="baseline" flexWrap="wrap" mb={1}>
        <Box>
          <Typography variant="subtitle2" fontWeight={700}>{p.hts_kor_isnm}</Typography>
          <Typography variant="caption" color="text.secondary">{symbol}</Typography>
        </Box>
        <Typography variant="h5" fontWeight={700}>{fmt(price)}원</Typography>
        <Stack direction="row" alignItems="center" spacing={0.5}>
          {pos
            ? <TrendingUpIcon fontSize="small" color="success" />
            : <TrendingDownIcon fontSize="small" color="error" />}
          <Typography variant="body2" color={pos ? 'success.main' : 'error.main'} fontWeight={600}>
            {pos ? '+' : ''}{fmt(change)} ({pos ? '+' : ''}{rt.toFixed(2)}%)
          </Typography>
        </Stack>
      </Stack>
      <Stack direction="row" spacing={2} flexWrap="wrap">
        {[
          { label: '시가', value: p.stck_oprc },
          { label: '고가', value: p.stck_hgpr, color: 'error.main' },
          { label: '저가', value: p.stck_lwpr, color: 'primary.main' },
          { label: '거래량', text: fmt(parseInt(p.acml_vol)) },
        ].filter(i => i.value || i.text).map(({ label, value, color, text }) => (
          <Box key={label}>
            <Typography variant="caption" color="text.secondary" display="block">{label}</Typography>
            <Typography variant="caption" fontWeight={600} color={color ?? 'text.primary'}>
              {text ?? (value ? fmt(parseInt(value)) + '원' : '-')}
            </Typography>
          </Box>
        ))}
      </Stack>
    </Box>
  )
}

// --- 해외 종목 정보 카드
function UsStockInfoCard({ symbol, exchange }: { symbol: string; exchange: OverseasExchange }) {
  const { data: p, isLoading, isError } = useOverseasPrice(symbol, exchange)
  if (!symbol) return null
  if (isLoading) return (
    <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
      <CircularProgress size={16} />
      <Typography variant="body2" color="text.secondary">조회 중</Typography>
    </Box>
  )
  if (isError || !p) return (
    <Box sx={{ px: 2, py: 1 }}>
      <Typography variant="caption" color="error">종목 정보를 불러올 수 없습니다</Typography>
    </Box>
  )

  const price  = parseFloat(p.last)
  const change = parseFloat(p.diff)
  const rt     = parseFloat(p.rate)
  const pos    = rt >= 0

  return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, py: 1.5 }}>
      <Stack direction="row" spacing={1.5} alignItems="baseline" flexWrap="wrap" mb={1}>
        <Box>
          <Typography variant="subtitle2" fontWeight={700}>{p.name || symbol}</Typography>
          <Typography variant="caption" color="text.secondary">{symbol} · {exchange}</Typography>
        </Box>
        <Typography variant="h5" fontWeight={700}>${price.toFixed(2)}</Typography>
        <Stack direction="row" alignItems="center" spacing={0.5}>
          {pos
            ? <TrendingUpIcon fontSize="small" color="success" />
            : <TrendingDownIcon fontSize="small" color="error" />}
          <Typography variant="body2" color={pos ? 'success.main' : 'error.main'} fontWeight={600}>
            {pos ? '+' : ''}{change.toFixed(2)} ({pos ? '+' : ''}{rt.toFixed(2)}%)
          </Typography>
        </Stack>
      </Stack>
      <Stack direction="row" spacing={2} flexWrap="wrap">
        {[
          { label: '시가',  text: '$' + parseFloat(p.open).toFixed(2) },
          { label: '고가',  text: '$' + parseFloat(p.high).toFixed(2), color: 'error.main' },
          { label: '저가',  text: '$' + parseFloat(p.low).toFixed(2),  color: 'primary.main' },
          { label: '거래량', text: fmt(parseInt(p.tvol)) },
        ]
          .filter(i => i.text && i.text !== '$0.00' && i.text !== '$NaN')
          .map(({ label, text, color }) => (
            <Box key={label}>
              <Typography variant="caption" color="text.secondary" display="block">{label}</Typography>
              <Typography variant="caption" fontWeight={600} color={color ?? 'text.primary'}>{text}</Typography>
            </Box>
          ))}
      </Stack>
    </Box>
  )
}

// --- Trading 메인
export default function Trading() {
  const [market, setMarket]           = useState<Market>('KR')
  const [symbol, setSymbol]           = useState('')
  const [inputValue, setInputValue]   = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [showResults, setShowResults] = useState(false)
  const [usExchange, setUsExchange]   = useState<OverseasExchange>('NAS')
  const [usSearching, setUsSearching] = useState(false)
  const [side, setSide]               = useState<OrderSide>('Buy')
  const [orderType, setOrderType]     = useState<OrderType>('Limit')
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
  }, [market])

  useEffect(() => {
    if (market !== 'KR' || !inputValue || !showResults) {
      if (!inputValue || !showResults) setSearchQuery('')
      return
    }
    const t = setTimeout(() => setSearchQuery(inputValue), 400)
    return () => clearTimeout(t)
  }, [inputValue, market, showResults])

  const { data: balance }                                           = useBalance()
  const { data: overseasBalance }                                   = useOverseasBalance()
  const { data: krPrice }                                           = usePrice(market === 'KR' && symbol.length === 6 ? symbol : '')
  const { data: usPrice }                                           = useOverseasPrice(market === 'US' ? symbol : '', market === 'US' ? usExchange : '')
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
  const krCurrentPrice = krPrice ? parseInt(krPrice.stck_prpr) : null
  const usCurrentPrice = usPrice ? parseFloat(usPrice.last) : null
  const stockName      = market === 'KR' ? krPrice?.hts_kor_isnm : (usPrice?.name ?? symbol)

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

  /** 미국 주식 거래소 자동 감지: NAS → NYS → AMS 순서로 조회 */
  const handleUsSearch = async () => {
    const ticker = inputValue.trim().toUpperCase()
    if (!ticker) return
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
    if (market === 'KR' && krCurrentPrice) setPrice(String(krCurrentPrice))
    if (market === 'US' && usCurrentPrice)  setPrice(usCurrentPrice.toFixed(2))
  }

  const handleSubmit = () => {
    setResult(null)
    setErrorMsg(null)
    const qty = parseInt(quantity)
    if (!symbol)          { setErrorMsg('종목을 선택하세요.'); return }
    if (!qty || qty <= 0) { setErrorMsg('수량을 입력하세요.'); return }

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
      placeOverseasOrder(
        { symbol, exchange: EXCHANGE_ORDER_MAP[usExchange], side, quantity: qty, price: prc },
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
            if (rawMsg.includes('해당업무가 제공되지 않습니다')) {
              setErrorMsg(
                `모의투자 미지원: ${rawMsg}\n` +
                `이 종목(${symbol}) 또는 거래소(${EXCHANGE_ORDER_MAP[usExchange]})는 모의투자에서 매도 주문이 지원되지 않습니다. ` +
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

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={2}>Trading</Typography>

      {/* 1. 보유 종목 */}
      <Paper sx={{ p: { xs: 1.5, sm: 2.5 }, mb: 2 }}>
        <Stack
          direction="row" alignItems="center" justifyContent="space-between"
          mb={1.5} flexWrap="wrap" gap={1}
        >
          <Typography variant="subtitle1" fontWeight={600}>보유 종목</Typography>
          {balance?.summary && (
            <Typography variant="caption" color="text.secondary" noWrap>
              예수금 {fmt(parseInt(balance.summary.dnca_tot_amt))}원
            </Typography>
          )}
        </Stack>
        <Divider sx={{ mb: 1.5 }} />

        {/* 국내 보유 */}
        <Typography variant="caption" color="text.secondary" fontWeight={600} sx={{ mb: 0.5, display: 'block' }}>
          🇰🇷 국내
        </Typography>
        <HoldingsTable items={balance?.items ?? []} onSelect={handleSelectHolding} />

        {/* 해외 보유 */}
        <Box sx={{ mt: 2 }}>
          <Typography variant="caption" color="text.secondary" fontWeight={600} sx={{ mb: 0.5, display: 'block' }}>
            🇺🇸 해외 — 클릭하면 US 매도 폼 자동 완성
          </Typography>
          <OverseasHoldingsTable
            items={overseasBalance?.items ?? []}
            onSelect={handleSelectOverseasHolding}
          />
        </Box>
      </Paper>

      {/* 2. 종목 검색 패널 */}
      <Paper sx={{ p: { xs: 1.5, sm: 2.5 }, mb: 2 }}>
        <Stack direction="row" alignItems="center" spacing={1} mb={1.5} flexWrap="wrap" gap={1}>
          <Typography variant="subtitle1" fontWeight={600}>종목 검색</Typography>
          <ToggleButtonGroup
            value={market} exclusive onChange={(_, v) => v && setMarket(v)} size="small"
          >
            <ToggleButton value="KR" sx={{ px: 1.5, gap: 0.5 }}>
              <FlagIcon sx={{ fontSize: 14 }} /> 국내
            </ToggleButton>
            <ToggleButton value="US" sx={{ px: 1.5, gap: 0.5 }}>
              <PublicIcon sx={{ fontSize: 14 }} /> 미국
            </ToggleButton>
          </ToggleButtonGroup>
        </Stack>
        <Divider sx={{ mb: 1.5 }} />

        {market === 'KR' ? (
          /* ── KR 종목 검색: TextField + 검색결과 드롭다운 테이블 ── */
          <Box>
            <TextField
              label="6자리 종목코드 (예: 005930, 0005A0)"
              value={inputValue}
              onChange={(e) => {
                const v = e.target.value
                setInputValue(v)
                if (/^[A-Z0-9]{6}$/i.test(v)) {
                  // 6자리 영숫자 코드 직접 입력 (0005A0 등 ETF 포함)
                  setSymbol(v.toUpperCase())
                  setShowResults(false)
                  setSearchQuery('')
                  setResult(null)
                  setErrorMsg(null)
                } else if (!v) {
                  setSymbol('')
                  setShowResults(false)
                  setSearchQuery('')
                } else if (v.length < 6) {
                  // 6자리 미만이면 대기
                  setShowResults(false)
                } else {
                  // 6자리 초과 입력 시 무시
                  setShowResults(false)
                }
              }}
              onBlur={() => {
                // 결과 테이블 클릭 허용을 위해 약간 지연 후 닫기
                closeTimerRef.current = setTimeout(() => setShowResults(false), 180)
              }}
              onFocus={() => {
                if (closeTimerRef.current) clearTimeout(closeTimerRef.current)
                if (inputValue.length >= 2 && !symbol) setShowResults(true)
              }}
              onKeyDown={(e) => {
                if (e.key === 'Escape') { setShowResults(false); setSearchQuery('') }
              }}
              size="small"
              fullWidth
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    {isFetchingSearch && (
                      <CircularProgress size={16} color="inherit" sx={{ mr: 0.5 }} />
                    )}
                    <Tooltip title="검색">
                      <span>
                        <IconButton
                          size="small"
                          onClick={() => {
                            if (inputValue.length >= 2) {
                              setSearchQuery(inputValue)
                              setShowResults(true)
                            }
                          }}
                          disabled={!inputValue || inputValue.length < 2}
                        >
                          <RefreshIcon fontSize="small" />
                        </IconButton>
                      </span>
                    </Tooltip>
                  </InputAdornment>
                ),
              }}
              helperText={
                symbol
                  ? `선택됨: ${symbol}`
                  : '국내 주식은 6자리 종목코드로만 검색 가능합니다 (예: 005930, 0005A0)'
              }
            />

            {/* 검색 결과 드롭다운 테이블 */}
            {showResults && (searchResults.length > 0 || isFetchingSearch) && (
              <Paper
                elevation={8}
                onMouseDown={(e) => {
                  // blur 이전에 클릭 이벤트가 발생하도록 preventDefault
                  e.preventDefault()
                  if (closeTimerRef.current) clearTimeout(closeTimerRef.current)
                }}
                sx={{
                  mt: 0.5,
                  maxHeight: 260,
                  overflow: 'auto',
                  border: 1,
                  borderColor: 'divider',
                  zIndex: 1400,
                  position: 'relative',
                }}
              >
                {isFetchingSearch && searchResults.length === 0 ? (
                  <Box sx={{ p: 1.5, display: 'flex', alignItems: 'center', gap: 1 }}>
                    <CircularProgress size={14} />
                    <Typography variant="caption" color="text.secondary">검색 중...</Typography>
                  </Box>
                ) : (
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell sx={{ py: 0.75, fontWeight: 700, fontSize: '0.7rem' }}>종목명</TableCell>
                        <TableCell sx={{ py: 0.75, fontWeight: 700, fontSize: '0.7rem', width: 80 }}>코드</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {searchResults.map((r) => (
                        <TableRow
                          key={r.pdno}
                          hover
                          sx={{ cursor: 'pointer' }}
                          onClick={() => {
                            setSymbol(r.pdno)
                            setInputValue(r.prdt_name)
                            setShowResults(false)
                            setSearchQuery('')
                            setResult(null)
                            setErrorMsg(null)
                          }}
                        >
                          <TableCell sx={{ py: 0.75 }}>
                            <Typography variant="body2" noWrap>{r.prdt_name}</Typography>
                          </TableCell>
                          <TableCell sx={{ py: 0.75 }}>
                            <Typography variant="caption" color="text.secondary">{r.pdno}</Typography>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                )}
              </Paper>
            )}

            {/* 검색어 입력됐으나 결과 없음 또는 종목 목록 비어있음 */}
            {showResults && !isFetchingSearch && searchQuery.length >= 2 && (searchResults.length === 0 || isStockListEmpty) && (
              <Box sx={{ mt: 0.5, px: 1.5, py: 1 }}>
                {isStockListEmpty ? (
                  <Stack direction="row" alignItems="center" spacing={1} flexWrap="wrap">
                    <Alert
                      severity="warning"
                      sx={{ flex: 1, py: 0.5, '& .MuiAlert-message': { display: 'flex', alignItems: 'center', gap: 1 } }}
                      action={
                        <Button
                          size="small"
                          color="warning"
                          variant="outlined"
                          onClick={() => doRefreshList()}
                          disabled={isRefreshing}
                          startIcon={isRefreshing ? <CircularProgress size={12} color="inherit" /> : undefined}
                          sx={{ whiteSpace: 'nowrap' }}
                        >
                          {isRefreshing ? '다운로드 중...' : '종목 목록 새로고침'}
                        </Button>
                      }
                    >
                      <Typography variant="caption">
                        종목 목록이 로드되지 않았습니다. KRX 서버 연결을 확인 후 새로고침을 눌러주세요.
                      </Typography>
                    </Alert>
                  </Stack>
                ) : (
                  <Typography variant="caption" color="text.secondary">
                    "{searchQuery}"에 대한 검색 결과가 없습니다
                  </Typography>
                )}
              </Box>
            )}
          </Box>
        ) : (
          <Stack direction="row" spacing={1} alignItems="flex-start">
            <TextField
              label="티커 (AAPL, TSLA …)"
              value={inputValue}
              onChange={(e) => {
                const v = e.target.value.toUpperCase()
                setInputValue(v)
                if (!v) setSymbol('')
              }}
              onKeyDown={(e) => { if (e.key === 'Enter' && inputValue) void handleUsSearch() }}
              size="small"
              fullWidth
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    {usSearching
                      ? <CircularProgress size={16} color="inherit" sx={{ mr: 0.5 }} />
                      : (
                        <IconButton
                          size="small"
                          onClick={() => void handleUsSearch()}
                          disabled={!inputValue}
                        >
                          <SearchIcon fontSize="small" />
                        </IconButton>
                      )
                    }
                  </InputAdornment>
                ),
              }}
              helperText="Enter 또는 검색 버튼 — 나스닥·뉴욕·AMEX 자동 감지"
            />
            {symbol && (
              <Chip
                label={usExchange}
                size="small"
                color="info"
                variant="outlined"
                sx={{ mt: 1, flexShrink: 0 }}
              />
            )}
          </Stack>
        )}

        {symbol && market === 'KR' && symbol.length === 6 && (
          <Box sx={{ mt: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
            <KrStockInfoCard symbol={symbol} />
          </Box>
        )}
        {symbol && market === 'US' && (
          <Box sx={{ mt: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
            <UsStockInfoCard symbol={symbol} exchange={usExchange} />
          </Box>
        )}
      </Paper>

      {/* 3. 주문 + 차트 */}
      <Grid container spacing={2}>
        <Grid item xs={12} md={4}>
          <Paper sx={{ p: { xs: 2, sm: 3 }, height: '100%' }}>
            <Stack direction="row" alignItems="center" spacing={1} mb={2}>
              <Typography variant="subtitle1" fontWeight={600}>수동 주문</Typography>
              {symbol && (
                <Chip
                  label={market === 'KR' ? symbol : symbol + ' (' + usExchange + ')'}
                  size="small"
                  color={market === 'US' ? 'info' : 'default'}
                  variant="outlined"
                />
              )}
            </Stack>

            <ToggleButtonGroup
              value={side} exclusive onChange={(_, v) => v && setSide(v)}
              fullWidth size="small" sx={{ mb: 2 }}
            >
              <ToggleButton value="Buy"  color="primary">매수</ToggleButton>
              <ToggleButton value="Sell" color="error">매도</ToggleButton>
            </ToggleButtonGroup>

            {market === 'KR' && (
              <ToggleButtonGroup
                value={orderType} exclusive onChange={(_, v) => v && setOrderType(v)}
                fullWidth size="small" sx={{ mb: 2 }}
              >
                <ToggleButton value="Limit">지정가</ToggleButton>
                <ToggleButton value="Market">시장가</ToggleButton>
              </ToggleButtonGroup>
            )}

            {(market === 'US' || orderType === 'Limit') && (
              <TextField
                label={market === 'US' ? '주문가격 (USD)' : '주문가격'}
                value={price}
                onChange={(e) => setPrice(e.target.value.replace(/[^0-9.]/g, ''))}
                fullWidth size="small" sx={{ mb: 1.5 }}
                InputProps={{
                  endAdornment: (
                    <InputAdornment position="end">
                      <Tooltip title="현재가 자동 입력">
                        <span>
                          <Button
                            size="small"
                            onClick={handleFillMarketPrice}
                            disabled={!(market === 'KR' ? krCurrentPrice : usCurrentPrice)}
                          >
                            현재가
                          </Button>
                        </span>
                      </Tooltip>
                    </InputAdornment>
                  ),
                }}
              />
            )}

            <TextField
              label="주문수량"
              value={quantity}
              onChange={(e) => setQuantity(e.target.value.replace(/\D/g, ''))}
              fullWidth size="small" sx={{ mb: 2 }}
            />

            {quantity && price && (
              <Box sx={{ mb: 2, p: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
                <Typography variant="body2">
                  예상 금액:{' '}
                  <strong>
                    {market === 'US'
                      ? '$' + (parseFloat(quantity || '0') * parseFloat(price || '0')).toFixed(2)
                      : fmt(parseInt(quantity || '0') * parseInt(price || '0')) + '원'}
                  </strong>
                </Typography>
                {market === 'KR' && (
                  <Typography variant="body2" color="text.secondary">
                    예수금: {fmt(availableCash)}원
                  </Typography>
                )}
              </Box>
            )}

            {result   && <Alert severity="success" sx={{ mb: 1.5 }}>{result}</Alert>}
            {errorMsg && <Alert severity="error"   sx={{ mb: 1.5 }}>{errorMsg}</Alert>}

            {/* 잔고 부족 매수 정지 경고 */}
            {tradingStatus?.buySuspended && side === 'Buy' && (
              <Alert
                severity="warning"
                sx={{ mb: 1.5 }}
                action={
                  <Button
                    size="small"
                    color="inherit"
                    onClick={() => clearBuySuspension()}
                    disabled={clearingBuySusp}
                    startIcon={clearingBuySusp ? <CircularProgress size={12} color="inherit" /> : undefined}
                  >
                    해제
                  </Button>
                }
              >
                잔고 부족으로 매수 주문이 정지되었습니다.
                매도 체결로 자본 확보 시 자동 재개됩니다.
              </Alert>
            )}

            <Button
              variant="contained"
              color={side === 'Buy' ? 'primary' : 'error'}
              fullWidth
              onClick={handleSubmit}
              disabled={isPending || !symbol}
              startIcon={isPending ? <CircularProgress size={16} color="inherit" /> : undefined}
              sx={{ py: 1.2 }}
            >
              {side === 'Buy' ? '매수 주문' : '매도 주문'}
              {market === 'US' && ' (USD 지정가)'}
            </Button>
          </Paper>
        </Grid>

        <Grid item xs={12} md={8}>
          <Paper sx={{ overflow: 'hidden' }}>
            {symbol && (market === 'KR' ? symbol.length === 6 : true) ? (
              <>
                {market === 'KR'
                  ? <KrStockInfoCard symbol={symbol} />
                  : <UsStockInfoCard symbol={symbol} exchange={usExchange} />}
                <Divider />
              </>
            ) : (
              <Box sx={{ px: 2.5, py: 2 }}>
                <Typography variant="body2" color="text.secondary">
                  위 검색창에서 종목을 선택하면 차트가 표시됩니다
                </Typography>
              </Box>
            )}
            {market === 'US' && symbol ? (
              <Box sx={{ p: { xs: 1, sm: 2 } }}>
                <OverseasStockChart symbol={symbol} exchange={usExchange} stockName={stockName} />
              </Box>
            ) : (
              <Box sx={{ p: { xs: 1, sm: 2 } }}>
                <StockChart symbol={market === 'KR' ? symbol : ''} stockName={stockName} />
              </Box>
            )}
          </Paper>
        </Grid>
      </Grid>
    </Box>
  )
}
