/**
 * Trading 페이지 — KR / US 주식, 모바일 반응형
 */
import { useEffect, useMemo, useState } from 'react'
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
import Autocomplete from '@mui/material/Autocomplete'
import MenuItem from '@mui/material/MenuItem'
import Select from '@mui/material/Select'
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import Tooltip from '@mui/material/Tooltip'
import TrendingUpIcon from '@mui/icons-material/TrendingUp'
import TrendingDownIcon from '@mui/icons-material/TrendingDown'
import SearchIcon from '@mui/icons-material/Search'
import PublicIcon from '@mui/icons-material/Public'
import FlagIcon from '@mui/icons-material/Flag'
import IconButton from '@mui/material/IconButton'

import {
  useBalance,
  usePlaceOrder,
  usePrice,
  useStockSearch,
  useTodayExecuted,
  useOverseasPrice,
  usePlaceOverseasOrder,
} from '../api/hooks'
import type {
  BalanceItem,
  OverseasExchange,
  OverseasOrderExchange,
  OrderSide,
  OrderType,
} from '../api/types'
import { StockChart } from '../components/chart/StockChart'

function fmt(n: number) {
  return n.toLocaleString('ko-KR')
}

type Market = 'KR' | 'US'

const EXCHANGE_ORDER_MAP: Record<OverseasExchange, OverseasOrderExchange> = {
  NAS: 'NASD',
  NYS: 'NYSE',
  AMS: 'AMEX',
}

// --- 보유 종목 테이블
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

// --- 검색 옵션 타입
interface StockOption {
  code: string
  name: string
  source: 'holding' | 'search'
  qty?: string
}

// --- Trading 메인
export default function Trading() {
  const [market, setMarket]           = useState<Market>('KR')
  const [symbol, setSymbol]           = useState('')
  const [inputValue, setInputValue]   = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [usExchange, setUsExchange]   = useState<OverseasExchange>('NAS')
  const [side, setSide]               = useState<OrderSide>('Buy')
  const [orderType, setOrderType]     = useState<OrderType>('Limit')
  const [quantity, setQuantity]       = useState('')
  const [price, setPrice]             = useState('')
  const [result, setResult]           = useState<string | null>(null)
  const [errorMsg, setErrorMsg]       = useState<string | null>(null)

  useEffect(() => {
    setSymbol('')
    setInputValue('')
    setSearchQuery('')
    setPrice('')
    setQuantity('')
    setResult(null)
    setErrorMsg(null)
  }, [market])

  useEffect(() => {
    if (market !== 'KR' || !inputValue || /^\d+$/.test(inputValue)) {
      setSearchQuery('')
      return
    }
    const t = setTimeout(() => setSearchQuery(inputValue), 400)
    return () => clearTimeout(t)
  }, [inputValue, market])

  const { data: balance }                                           = useBalance()
  const { data: krPrice }                                           = usePrice(market === 'KR' && symbol.length === 6 ? symbol : '')
  const { data: usPrice }                                           = useOverseasPrice(market === 'US' ? symbol : '', market === 'US' ? usExchange : '')
  const { data: executed }                                          = useTodayExecuted()
  const { data: searchResults = [], isFetching: isFetchingSearch } = useStockSearch(searchQuery)
  const { mutate: placeOrder,         isPending: isPendingKr }     = usePlaceOrder()
  const { mutate: placeOverseasOrder, isPending: isPendingUs }     = usePlaceOverseasOrder()
  const isPending = isPendingKr || isPendingUs

  const symbolOptions = useMemo<StockOption[]>(() => {
    if (market !== 'KR') return []
    const q = inputValue.toLowerCase()
    const holdingOpts: StockOption[] = (balance?.items ?? [])
      .filter(i => q.length < 1 || i.pdno.includes(q) || i.prdt_name.toLowerCase().includes(q))
      .map(i => ({ code: i.pdno, name: i.prdt_name, source: 'holding' as const, qty: i.hldg_qty }))
    const holdingCodes = new Set(holdingOpts.map(h => h.code))
    const searchOpts: StockOption[] = searchResults
      .filter(r => !holdingCodes.has(r.pdno))
      .map(r => ({ code: r.pdno, name: r.prdt_name, source: 'search' as const }))
    return [...holdingOpts, ...searchOpts]
  }, [balance?.items, inputValue, searchResults, market])

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
      if (symbol.length !== 6) { setErrorMsg('국내 종목코드는 6자리 숫자입니다.'); return }
      const prc = parseInt(price)
      if (orderType === 'Limit' && (!prc || prc <= 0)) {
        setErrorMsg('지정가 주문은 가격을 입력해야 합니다.')
        return
      }
      placeOrder(
        { symbol, side, order_type: orderType, quantity: qty, price: orderType === 'Market' ? 0 : prc },
        {
          onSuccess: (d) => { setResult('주문 완료 — 주문번호: ' + d.odno); setQuantity(''); setPrice('') },
          onError:   (e) => setErrorMsg(String(e)),
        },
      )
    } else {
      const prc = parseFloat(price)
      if (!prc || prc <= 0) { setErrorMsg('해외 주문은 USD 가격을 입력해야 합니다.'); return }
      placeOverseasOrder(
        { symbol, exchange: EXCHANGE_ORDER_MAP[usExchange], side, quantity: qty, price: prc },
        {
          onSuccess: (d) => { setResult('주문 완료 — 주문번호: ' + d.odno); setQuantity(''); setPrice('') },
          onError:   (e) => setErrorMsg(String(e)),
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
              총평가 {fmt(parseInt(balance.summary.tot_evlu_amt))}원 ·
              예수금 {fmt(parseInt(balance.summary.dnca_tot_amt))}원
            </Typography>
          )}
        </Stack>
        <Divider sx={{ mb: 1.5 }} />
        <HoldingsTable items={balance?.items ?? []} onSelect={handleSelectHolding} />
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
          <Autocomplete<StockOption, false, false, true>
            freeSolo
            options={symbolOptions}
            groupBy={(opt) =>
              typeof opt !== 'string'
                ? opt.source === 'holding' ? '보유 중' : '검색 결과'
                : ''
            }
            getOptionLabel={(opt) => typeof opt === 'string' ? opt : opt.name}
            renderOption={(props, opt) => {
              const { key: _k, ...rest } = props as React.HTMLAttributes<HTMLLIElement> & { key?: React.Key }
              return (
                <Box component="li" key={opt.code} {...rest}>
                  <Typography variant="body2" sx={{ flexGrow: 1 }}>{opt.name}</Typography>
                  <Typography variant="caption" color="text.secondary" sx={{ ml: 1 }}>
                    {opt.code}{opt.qty ? ' · ' + fmt(parseInt(opt.qty)) + '주' : ''}
                  </Typography>
                </Box>
              )
            }}
            isOptionEqualToValue={(opt, val) =>
              typeof opt !== 'string' && typeof val !== 'string' && opt.code === val.code
            }
            inputValue={inputValue}
            onInputChange={(_, v, reason) => {
              setInputValue(v)
              if (reason === 'input') {
                if (/^\d{6}$/.test(v)) { setSymbol(v); setResult(null); setErrorMsg(null) }
                else if (!v) setSymbol('')
              }
            }}
            onChange={(_, value) => {
              if (!value) {
                setSymbol(''); setInputValue('')
              } else if (typeof value === 'string') {
                if (/^\d{6}$/.test(value)) { setSymbol(value); setInputValue(value) }
              } else {
                setSymbol(value.code); setInputValue(value.name)
                setResult(null); setErrorMsg(null)
              }
            }}
            renderInput={(params) => (
              <TextField
                {...params}
                label="종목명 또는 6자리 코드"
                size="small"
                fullWidth
                InputProps={{
                  ...params.InputProps,
                  endAdornment: (
                    <>
                      {isFetchingSearch && (
                        <CircularProgress size={16} color="inherit" sx={{ mr: 0.5 }} />
                      )}
                      <IconButton
                        size="small"
                        onClick={() => setSearchQuery(inputValue)}
                        disabled={!inputValue}
                      >
                        <SearchIcon fontSize="small" />
                      </IconButton>
                      {params.InputProps.endAdornment}
                    </>
                  ),
                }}
              />
            )}
          />
        ) : (
          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1.5} alignItems="flex-start">
            <FormControl size="small" sx={{ minWidth: 120, flexShrink: 0 }}>
              <InputLabel>거래소</InputLabel>
              <Select
                value={usExchange}
                label="거래소"
                onChange={(e) => setUsExchange(e.target.value as OverseasExchange)}
              >
                <MenuItem value="NAS">NASDAQ</MenuItem>
                <MenuItem value="NYS">NYSE</MenuItem>
                <MenuItem value="AMS">AMEX</MenuItem>
              </Select>
            </FormControl>
            <TextField
              label="티커 (AAPL, TSLA …)"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value.toUpperCase())}
              onKeyDown={(e) => { if (e.key === 'Enter' && inputValue) setSymbol(inputValue.trim()) }}
              size="small"
              fullWidth
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    <IconButton
                      size="small"
                      onClick={() => inputValue && setSymbol(inputValue.trim())}
                      disabled={!inputValue}
                    >
                      <SearchIcon fontSize="small" />
                    </IconButton>
                  </InputAdornment>
                ),
              }}
              helperText="Enter 또는 검색 버튼 클릭 시 현재가를 조회합니다"
            />
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
              <Box sx={{ p: 3, textAlign: 'center' }}>
                <Typography variant="body2" color="text.secondary">
                  해외 주식 차트는 준비 중입니다.
                </Typography>
              </Box>
            ) : (
              <Box sx={{ p: { xs: 1, sm: 2 } }}>
                <StockChart symbol={market === 'KR' ? symbol : ''} stockName={stockName} />
              </Box>
            )}
          </Paper>
        </Grid>
      </Grid>

      {/* 4. 당일 체결 내역 */}
      <Paper sx={{ p: { xs: 1.5, sm: 2.5 }, mt: 2 }}>
        <Typography variant="subtitle1" fontWeight={600} mb={2}>
          당일 체결 내역 (KIS)
        </Typography>
        <Divider sx={{ mb: 2 }} />
        {!executed || executed.length === 0 ? (
          <Typography variant="body2" color="text.secondary">당일 체결 내역이 없습니다.</Typography>
        ) : (
          <TableContainer sx={{ maxHeight: 280, overflowX: 'auto' }}>
            <Table size="small" stickyHeader>
              <TableHead>
                <TableRow>
                  <TableCell sx={{ minWidth: 100 }}>종목</TableCell>
                  <TableCell>구분</TableCell>
                  <TableCell align="right">체결수량</TableCell>
                  <TableCell align="right">단가</TableCell>
                  <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>금액</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {executed.map((o) => (
                  <TableRow key={o.odno + o.ord_tmd}>
                    <TableCell>
                      <Typography variant="body2" noWrap sx={{ maxWidth: { xs: 100, sm: 'none' } }}>
                        {o.prdt_name}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">{o.pdno}</Typography>
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={o.sll_buy_dvsn_cd === '01' ? '매도' : '매수'}
                        color={o.sll_buy_dvsn_cd === '01' ? 'error' : 'primary'}
                        size="small"
                      />
                    </TableCell>
                    <TableCell align="right">{fmt(parseInt(o.tot_ccld_qty))}</TableCell>
                    <TableCell align="right" sx={{ whiteSpace: 'nowrap' }}>
                      {fmt(parseInt(o.ord_unpr))}원
                    </TableCell>
                    <TableCell align="right" sx={{ whiteSpace: 'nowrap', display: { xs: 'none', sm: 'table-cell' } }}>
                      {fmt(parseInt(o.tot_ccld_amt))}원
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        )}
      </Paper>
    </Box>
  )
}
