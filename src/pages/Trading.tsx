/**
 * Trading 페이지
 *
 * 레이아웃:
 *   [좌: 주문 패널] [우: 종목 정보 + 차트]
 *   [보유 종목] [당일 체결 내역]
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
import TrendingUpIcon from '@mui/icons-material/TrendingUp'
import TrendingDownIcon from '@mui/icons-material/TrendingDown'

import { useBalance, usePlaceOrder, usePrice, useStockSearch, useTodayExecuted } from '../api/hooks'
import type { BalanceItem, OrderSide, OrderType } from '../api/types'
import { StockChart } from '../components/chart/StockChart'

function fmt(n: number) {
  return n.toLocaleString('ko-KR')
}

// --- 종목 정보 헤더 카드 ---

interface StockInfoCardProps {
  symbol: string
}

function StockInfoCard({ symbol }: StockInfoCardProps) {
  const { data: priceData, isLoading } = usePrice(symbol.length === 6 ? symbol : '')
  if (!priceData && !isLoading) return null
  if (isLoading) return (
    <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
      <CircularProgress size={16} />
      <Typography variant="body2" color="text.secondary">종목 정보 조회 중</Typography>
    </Box>
  )
  if (!priceData) return null

  const price    = parseInt(priceData.stck_prpr)
  const change   = parseInt(priceData.prdy_vrss)
  const changeRt = parseFloat(priceData.prdy_ctrt)
  const positive = changeRt >= 0

  const ohlcItems = [
    { label: '시가',  value: priceData.stck_oprc },
    { label: '고가',  value: priceData.stck_hgpr, color: 'error.main'   },
    { label: '저가',  value: priceData.stck_lwpr, color: 'primary.main' },
    { label: '거래량', value: null, formatted: fmt(parseInt(priceData.acml_vol)) },
    { label: '상한가', value: priceData.stck_mxpr, color: 'error.main'   },
    { label: '하한가', value: priceData.stck_llam, color: 'primary.main' },
    { label: '52주고', value: priceData.w52_hgpr },
    { label: '52주저', value: priceData.w52_lwpr },
  ].filter(i => i.value || i.formatted)

  return (
    <Box sx={{ px: 2.5, py: 1.5 }}>
      {/* 상단: 종목명 + 현재가 + 등락 */}
      <Stack direction="row" spacing={2} alignItems="baseline" flexWrap="wrap" mb={1}>
        <Box>
          <Typography variant="subtitle2" fontWeight={700}>{priceData.hts_kor_isnm}</Typography>
          <Typography variant="caption" color="text.secondary">{symbol}</Typography>
        </Box>
        <Typography variant="h5" fontWeight={700}>{fmt(price)}원</Typography>
        <Stack direction="row" alignItems="center" spacing={0.5}>
          {positive
            ? <TrendingUpIcon fontSize="small" color="success" />
            : <TrendingDownIcon fontSize="small" color="error" />
          }
          <Typography
            variant="body2"
            color={positive ? 'success.main' : 'error.main'}
            fontWeight={600}
          >
            {positive ? '+' : ''}{fmt(change)} ({positive ? '+' : ''}{changeRt.toFixed(2)}%)
          </Typography>
        </Stack>
      </Stack>
      {/* 하단: OHLC 상세 */}
      {ohlcItems.length > 0 && (
        <Stack direction="row" spacing={2} flexWrap="wrap">
          {ohlcItems.map(({ label, value, formatted, color }) => (
            <Box key={label}>
              <Typography variant="caption" color="text.secondary" display="block">{label}</Typography>
              <Typography
                variant="caption"
                fontWeight={600}
                color={color ?? 'text.primary'}
              >
                {formatted ?? (value ? fmt(parseInt(value)) + '원' : '-')}
              </Typography>
            </Box>
          ))}
        </Stack>
      )}
    </Box>
  )
}

// --- 보유 종목 테이블 ---

interface HoldingsTableProps {
  items: BalanceItem[]
  onSelect: (item: BalanceItem) => void
}

function HoldingsTable({ items, onSelect }: HoldingsTableProps) {
  if (items.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary">
        보유 종목 없음
      </Typography>
    )
  }

  return (
    <TableContainer sx={{ maxHeight: 240 }}>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>종목</TableCell>
            <TableCell align="right">보유수량</TableCell>
            <TableCell align="right">현재가</TableCell>
            <TableCell align="right">손익률</TableCell>
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
                  {item.prdt_name}
                  <br />
                  <Typography variant="caption" color="text.secondary">
                    {item.pdno}
                  </Typography>
                </TableCell>
                <TableCell align="right">
                  {fmt(parseInt(item.hldg_qty))}주
                </TableCell>
                <TableCell align="right">
                  {fmt(parseInt(item.prpr))}원
                </TableCell>
                <TableCell align="right">
                  <Typography
                    variant="body2"
                    color={positive ? 'success.main' : 'error.main'}
                    fontWeight={600}
                  >
                    {positive ? '+' : ''}{pfls.toFixed(2)}%
                  </Typography>
                  <Typography
                    variant="caption"
                    color={positive ? 'success.main' : 'error.main'}
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

// --- 종목 검색 옵션 타입 ---

interface StockOption {
  code: string
  name: string
  source: 'holding' | 'search'
  qty?: string
}

// --- Trading 페이지 ---

export default function Trading() {
  const [symbol, setSymbol]           = useState('')
  const [inputValue, setInputValue]   = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [side, setSide]               = useState<OrderSide>('Buy')
  const [orderType, setOrderType]     = useState<OrderType>('Limit')
  const [quantity, setQuantity]       = useState('')
  const [price, setPrice]             = useState('')
  const [result, setResult]           = useState<string | null>(null)
  const [errorMsg, setErrorMsg]       = useState<string | null>(null)

  // 입력값 변경 시 400ms 디바운스로 검색 쿼리 설정 (숫자 입력 시 비활성)
  useEffect(() => {
    if (!inputValue || /^\d+$/.test(inputValue)) {
      setSearchQuery('')
      return
    }
    const t = setTimeout(() => setSearchQuery(inputValue), 400)
    return () => clearTimeout(t)
  }, [inputValue])

  const { data: balance }                 = useBalance()
  const { data: priceData }               = usePrice(symbol.length === 6 ? symbol : '')
  const { data: executed }                = useTodayExecuted()
  const { data: searchResults = [] }      = useStockSearch(searchQuery)
  const { mutate: placeOrder, isPending } = usePlaceOrder()

  // Autocomplete 옵션: 보유 종목 (즉시) + 검색 결과 (API, 디바운스)
  const symbolOptions = useMemo<StockOption[]>(() => {
    const q = inputValue.toLowerCase()
    const holdingOpts: StockOption[] = (balance?.items ?? [])
      .filter(
        (i) =>
          q.length < 1 ||
          i.pdno.includes(q) ||
          i.prdt_name.toLowerCase().includes(q)
      )
      .map((i) => ({
        code: i.pdno,
        name: i.prdt_name,
        source: 'holding' as const,
        qty: i.hldg_qty,
      }))

    const holdingCodes = new Set(holdingOpts.map((h) => h.code))
    const searchOpts: StockOption[] = searchResults
      .filter((r) => !holdingCodes.has(r.pdno))
      .map((r) => ({ code: r.pdno, name: r.prdt_name, source: 'search' as const }))

    return [...holdingOpts, ...searchOpts]
  }, [balance?.items, inputValue, searchResults])

  const availableCash = parseInt(balance?.summary?.dnca_tot_amt ?? '0') || 0
  const currentPrice  = priceData ? parseInt(priceData.stck_prpr) : null
  const stockName     = priceData?.hts_kor_isnm

  const handleSelectHolding = (item: BalanceItem) => {
    setSymbol(item.pdno)
    setInputValue(item.prdt_name)
    setResult(null)
    setErrorMsg(null)
  }

  const handleFillMarketPrice = () => {
    if (currentPrice) setPrice(String(currentPrice))
  }

  const handleSubmit = () => {
    setResult(null)
    setErrorMsg(null)

    const qty = parseInt(quantity)
    const prc = parseInt(price)

    if (!symbol || symbol.length !== 6) {
      setErrorMsg('종목코드는 6자리 숫자입니다.')
      return
    }
    if (!qty || qty <= 0) {
      setErrorMsg('수량을 입력하세요.')
      return
    }
    if (orderType === 'Limit' && (!prc || prc <= 0)) {
      setErrorMsg('지정가 주문은 가격을 입력해야 합니다.')
      return
    }

    placeOrder(
      {
        symbol,
        side,
        order_type: orderType,
        quantity: qty,
        price: orderType === 'Market' ? 0 : prc,
      },
      {
        onSuccess: (data) => {
          setResult(`주문 완료 -- 주문번호: ${data.odno} (${data.msg1})`)
          setQuantity('')
          setPrice('')
        },
        onError: (err) => {
          setErrorMsg(String(err))
        },
      }
    )
  }

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={2}>
        Trading
      </Typography>

      {/* ─── 보유 종목 (최상단) ─── */}
      <Paper sx={{ p: 2.5, mb: 2 }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" mb={1.5}>
          <Typography variant="subtitle1" fontWeight={600}>
            보유 종목
          </Typography>
          {balance?.summary && (
            <Typography variant="caption" color="text.secondary">
              총평가 {fmt(parseInt(balance.summary.tot_evlu_amt))}원 ·
              예수금 {fmt(parseInt(balance.summary.dnca_tot_amt))}원
            </Typography>
          )}
        </Stack>
        <Divider sx={{ mb: 1.5 }} />
        <HoldingsTable items={balance?.items ?? []} onSelect={handleSelectHolding} />
      </Paper>

      {/* ─── 주문 + 종목차트 ─── */}
      <Grid container spacing={2}>
        {/* 좌: 주문 패널 */}
        <Grid item xs={12} md={4}>
          <Paper sx={{ p: 3, height: '100%' }}>
            <Typography variant="subtitle1" fontWeight={600} mb={2}>
              수동 주문
            </Typography>

            <ToggleButtonGroup
              value={side}
              exclusive
              onChange={(_, v) => v && setSide(v)}
              fullWidth
              size="small"
              sx={{ mb: 2 }}
            >
              <ToggleButton value="Buy"  color="primary">매수</ToggleButton>
              <ToggleButton value="Sell" color="error">매도</ToggleButton>
            </ToggleButtonGroup>

            <ToggleButtonGroup
              value={orderType}
              exclusive
              onChange={(_, v) => v && setOrderType(v)}
              fullWidth
              size="small"
              sx={{ mb: 2 }}
            >
              <ToggleButton value="Limit">지정가</ToggleButton>
              <ToggleButton value="Market">시장가</ToggleButton>
            </ToggleButtonGroup>

            <Autocomplete<StockOption, false, false, true>
              freeSolo
              options={symbolOptions}
              groupBy={(opt) =>
                typeof opt !== 'string'
                  ? opt.source === 'holding'
                    ? '보유 중'
                    : '검색 결과'
                  : ''
              }
              getOptionLabel={(opt) =>
                typeof opt === 'string' ? opt : opt.name
              }
              renderOption={(props, opt) => {
                const { key: _k, ...rest } = props as React.HTMLAttributes<HTMLLIElement> & { key?: React.Key }
                return (
                  <Box component="li" key={opt.code} {...rest}>
                    <Typography variant="body2" sx={{ flexGrow: 1 }}>
                      {opt.name}
                    </Typography>
                    <Typography variant="caption" color="text.secondary" sx={{ ml: 1 }}>
                      {opt.code}
                      {opt.qty ? ` · ${fmt(parseInt(opt.qty))}주` : ''}
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
                  if (/^\d{6}$/.test(v)) {
                    setSymbol(v)
                    setResult(null)
                    setErrorMsg(null)
                  } else if (!v) {
                    setSymbol('')
                  }
                }
              }}
              onChange={(_, value) => {
                if (!value) {
                  setSymbol('')
                  setInputValue('')
                } else if (typeof value === 'string') {
                  if (/^\d{6}$/.test(value)) {
                    setSymbol(value)
                    setInputValue(value)
                  }
                } else {
                  setSymbol(value.code)
                  setInputValue(value.name)
                  setResult(null)
                  setErrorMsg(null)
                }
              }}
              renderInput={(params) => (
                <TextField
                  {...params}
                  label="종목 검색"
                  size="small"
                  sx={{ mb: 1.5 }}
                  helperText={
                    priceData
                      ? `${priceData.hts_kor_isnm} / 현재가 ${fmt(parseInt(priceData.stck_prpr))}원`
                      : '종목명 또는 6자리 코드 입력'
                  }
                />
              )}
              sx={{ mb: 0 }}
            />

            {orderType === 'Limit' && (
              <TextField
                label="주문가격"
                value={price}
                onChange={(e) => setPrice(e.target.value.replace(/\D/g, ''))}
                fullWidth
                size="small"
                sx={{ mb: 1.5 }}
                InputProps={{
                  endAdornment: (
                    <InputAdornment position="end">
                      <Button size="small" onClick={handleFillMarketPrice} disabled={!currentPrice}>
                        현재가
                      </Button>
                    </InputAdornment>
                  ),
                }}
              />
            )}

            <TextField
              label="주문수량"
              value={quantity}
              onChange={(e) => setQuantity(e.target.value.replace(/\D/g, ''))}
              fullWidth
              size="small"
              sx={{ mb: 2 }}
            />

            {quantity && price && (
              <Box sx={{ mb: 2, p: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
                <Typography variant="body2">
                  예상 금액:{' '}
                  <strong>
                    {fmt(parseInt(quantity || '0') * parseInt(price || '0'))}원
                  </strong>
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  예수금: {fmt(availableCash)}원
                </Typography>
              </Box>
            )}

            {result   && <Alert severity="success" sx={{ mb: 1.5 }}>{result}</Alert>}
            {errorMsg && <Alert severity="error"   sx={{ mb: 1.5 }}>{errorMsg}</Alert>}

            <Button
              variant="contained"
              color={side === 'Buy' ? 'primary' : 'error'}
              fullWidth
              onClick={handleSubmit}
              disabled={isPending}
              startIcon={isPending ? <CircularProgress size={16} color="inherit" /> : undefined}
            >
              {side === 'Buy' ? '매수 주문' : '매도 주문'}
            </Button>
          </Paper>
        </Grid>

        {/* 우: 종목 정보 + 차트 */}
        <Grid item xs={12} md={8}>
          <Paper sx={{ overflow: 'hidden' }}>
            {symbol.length === 6 ? (
              <>
                <StockInfoCard symbol={symbol} />
                <Divider />
              </>
            ) : (
              <Box sx={{ px: 2.5, py: 2 }}>
                <Typography variant="body2" color="text.secondary">
                  좌측에서 종목코드를 입력하면 차트가 표시됩니다
                </Typography>
              </Box>
            )}
            <Box sx={{ p: 2 }}>
              <StockChart symbol={symbol} stockName={stockName} />
            </Box>
          </Paper>
        </Grid>
      </Grid>

      {/* ─── 당일 체결 내역 ─── */}
      <Paper sx={{ p: 2.5, mt: 2 }}>
        <Typography variant="subtitle1" fontWeight={600} mb={2}>
          당일 체결 내역 (KIS)
        </Typography>
        <Divider sx={{ mb: 2 }} />
        {!executed || executed.length === 0 ? (
          <Typography variant="body2" color="text.secondary">
            당일 체결 내역이 없습니다.
          </Typography>
        ) : (
          <TableContainer sx={{ maxHeight: 280 }}>
            <Table size="small" stickyHeader>
              <TableHead>
                <TableRow>
                  <TableCell>종목</TableCell>
                  <TableCell>구분</TableCell>
                  <TableCell align="right">체결수량</TableCell>
                  <TableCell align="right">단가</TableCell>
                  <TableCell align="right">금액</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {executed.map((o) => (
                  <TableRow key={o.odno + o.ord_tmd}>
                    <TableCell>
                      {o.prdt_name}
                      <br />
                      <Typography variant="caption" color="text.secondary">
                        {o.pdno}
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={o.sll_buy_dvsn_cd === '01' ? '매도' : '매수'}
                        color={o.sll_buy_dvsn_cd === '01' ? 'error' : 'primary'}
                        size="small"
                      />
                    </TableCell>
                    <TableCell align="right">{fmt(parseInt(o.tot_ccld_qty))}</TableCell>
                    <TableCell align="right">{fmt(parseInt(o.ord_unpr))}원</TableCell>
                    <TableCell align="right">{fmt(parseInt(o.tot_ccld_amt))}원</TableCell>
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
