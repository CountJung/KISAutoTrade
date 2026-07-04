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
  useBrokerHoldings,
  useOverseasBalance,
  usePlaceOrder,
  usePrice,
  useStockSearch,
  useOverseasPrice,
  useTossMarketSnapshot,
  useTossOrderPreflight,
  useTossStockSafety,
  useTossMarketCalendar,
  usePlaceOverseasOrder,
  useProfiles,
  useRefreshStockList,
  useTradingStatus,
  useClearBuySuspension,
  useAppConfig,
  useUpdateProfile,
} from '../../../api/hooks'
import * as cmd from '../../../api/commands'
import type {
  AccountProfileView,
  AppConfigView,
  BalanceItem,
  BrokerHoldingView,
  OverseasBalanceItem,
  CmdError,
  OverseasExchange,
  OverseasOrderExchange,
  OrderSide,
  OrderType,
  TossMarketCalendarView,
  TossMarketSnapshotView,
  TossOrderPreflightView,
  TossStockSafetyView,
} from '../../../api/types'
import { StockChart, OverseasStockChart } from '../../../widgets/stock-chart'
import { BrokerScopeIndicator } from '../../../shared/ui'

function fmt(n: number) {
  return n.toLocaleString('ko-KR')
}

function fmtDecimal(value: string, fractionDigits = 4) {
  const parsed = Number(value.replace(/,/g, ''))
  if (!Number.isFinite(parsed)) return value
  return parsed.toLocaleString('ko-KR', { maximumFractionDigits: fractionDigits })
}

function fmtTossMoney(amount: string, currency: string) {
  return currency === 'KRW'
    ? `${fmtDecimal(amount, 0)}원`
    : `$${fmtDecimal(amount, 4)}`
}

function fmtBrokerMoney(money?: { amount: string; currency: string } | null) {
  return money ? fmtTossMoney(money.amount, money.currency) : '-'
}

function tossMarketLabel(market: TossMarketSnapshotView['market']) {
  return market === 'kr' ? '국내' : '미국'
}

function fmtTossSession(session: TossMarketCalendarView['kr']['regularSession']) {
  if (!session) return '휴장'
  const format = (value: string) => new Date(value).toLocaleTimeString('ko-KR', {
    hour: '2-digit',
    minute: '2-digit',
  })
  return `${format(session.startTime)}~${format(session.endTime)}`
}

type Market = 'KR' | 'US'

const EXCHANGE_ORDER_MAP: Record<OverseasExchange, OverseasOrderExchange> = {
  NAS: 'NASD',
  NYS: 'NYSE',
  AMS: 'AMEX',
}

const PAPER_UNSUPPORTED_US_SELL_SYMBOLS = ['VOO', 'SPY', 'QQQM']

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

// --- Toss read-only 시세 카드
function TossMarketSnapshotCard({ symbol }: { symbol: string }) {
  const { data, isLoading, isError, error } = useTossMarketSnapshot(symbol)
  if (!symbol) return null
  if (isLoading) return (
    <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
      <CircularProgress size={16} />
      <Typography variant="body2" color="text.secondary">Toss 시세 조회 중</Typography>
    </Box>
  )
  if (isError || !data) return (
    <Box sx={{ px: 2, py: 1 }}>
      <Typography variant="caption" color="error">
        Toss 시세를 불러올 수 없습니다: {(error as { message?: string } | null)?.message ?? '연결 진단을 확인하세요.'}
      </Typography>
    </Box>
  )

  const bestAsk = data.orderbook.asks[0]
  const bestBid = data.orderbook.bids[0]
  const latestTrade = data.trades[0]

  return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, py: 1.5 }}>
      <Stack direction="row" spacing={1.5} alignItems="baseline" flexWrap="wrap" mb={1}>
        <Box>
          <Stack direction="row" alignItems="center" spacing={0.75}>
            <Typography variant="subtitle2" fontWeight={700}>{symbol}</Typography>
            <Chip size="small" label={`Toss · ${tossMarketLabel(data.market)}`} sx={{ height: 18, fontSize: '0.65rem' }} />
          </Stack>
          {data.timestamp && (
            <Typography variant="caption" color="text.secondary">{new Date(data.timestamp).toLocaleString('ko-KR')}</Typography>
          )}
        </Box>
        <Typography variant="h5" fontWeight={700}>
          {fmtTossMoney(data.price.amount, data.price.currency)}
        </Typography>
      </Stack>

      <Grid container spacing={1.5}>
        <Grid item xs={6} sm={3}>
          <Typography variant="caption" color="text.secondary" display="block">매도 1호가</Typography>
          <Typography variant="caption" fontWeight={600} color="error.main">
            {bestAsk ? fmtTossMoney(bestAsk.price, data.orderbook.currency) : '-'}
          </Typography>
          {bestAsk && <Typography variant="caption" color="text.secondary" display="block">잔량 {fmtDecimal(bestAsk.volume, 4)}</Typography>}
        </Grid>
        <Grid item xs={6} sm={3}>
          <Typography variant="caption" color="text.secondary" display="block">매수 1호가</Typography>
          <Typography variant="caption" fontWeight={600} color="primary.main">
            {bestBid ? fmtTossMoney(bestBid.price, data.orderbook.currency) : '-'}
          </Typography>
          {bestBid && <Typography variant="caption" color="text.secondary" display="block">잔량 {fmtDecimal(bestBid.volume, 4)}</Typography>}
        </Grid>
        <Grid item xs={6} sm={3}>
          <Typography variant="caption" color="text.secondary" display="block">상한가</Typography>
          <Typography variant="caption" fontWeight={600} color="error.main">
            {data.priceLimits.upperLimitPrice ? fmtTossMoney(data.priceLimits.upperLimitPrice, data.priceLimits.currency) : '-'}
          </Typography>
        </Grid>
        <Grid item xs={6} sm={3}>
          <Typography variant="caption" color="text.secondary" display="block">하한가</Typography>
          <Typography variant="caption" fontWeight={600} color="primary.main">
            {data.priceLimits.lowerLimitPrice ? fmtTossMoney(data.priceLimits.lowerLimitPrice, data.priceLimits.currency) : '-'}
          </Typography>
        </Grid>
      </Grid>

      {latestTrade && (
        <Box sx={{ mt: 1.25 }}>
          <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 0.5 }}>최근 체결</Typography>
          <Stack direction="row" spacing={1.5} flexWrap="wrap" useFlexGap>
            {data.trades.slice(0, 3).map((trade) => (
              <Typography key={`${trade.timestamp}:${trade.price}:${trade.volume}`} variant="caption">
                {fmtTossMoney(trade.price, trade.currency)} · {fmtDecimal(trade.volume, 4)}
              </Typography>
            ))}
          </Stack>
        </Box>
      )}
    </Box>
  )
}

function TossStockSafetyCard({
  data,
  isLoading,
  isError,
  error,
}: {
  data: TossStockSafetyView | undefined
  isLoading: boolean
  isError: boolean
  error: unknown
}) {
  if (isLoading) return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, pb: 1.5, display: 'flex', alignItems: 'center', gap: 1 }}>
      <CircularProgress size={14} />
      <Typography variant="caption" color="text.secondary">Toss 종목 유의사항 조회 중</Typography>
    </Box>
  )
  if (isError) return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, pb: 1.5 }}>
      <Typography variant="caption" color="error">
        Toss 종목 유의사항을 불러올 수 없습니다: {(error as { message?: string } | null)?.message ?? '연결 진단을 확인하세요.'}
      </Typography>
    </Box>
  )
  if (!data) return null

  const info = data.stockInfo

  return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, pb: 1.5 }}>
      <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap sx={{ mb: data.warnings.length ? 1 : 0 }}>
        {info && (
          <>
            <Chip size="small" label={info.name || info.symbol} variant="outlined" sx={{ height: 22, fontSize: '0.7rem' }} />
            <Chip size="small" label={info.market} variant="outlined" sx={{ height: 22, fontSize: '0.7rem' }} />
            <Chip
              size="small"
              label={info.status}
              color={info.status === 'ACTIVE' ? 'success' : 'warning'}
              variant="outlined"
              sx={{ height: 22, fontSize: '0.7rem' }}
            />
            <Chip size="small" label={info.securityType} variant="outlined" sx={{ height: 22, fontSize: '0.7rem' }} />
          </>
        )}
        {!info && (
          <Chip size="small" label="종목 기본 정보 없음" color="warning" variant="outlined" sx={{ height: 22, fontSize: '0.7rem' }} />
        )}
      </Stack>

      {data.warnings.length > 0 ? (
        <Alert severity={data.buyBlocked ? 'warning' : 'info'} sx={{ py: 0.5 }}>
          <Stack spacing={0.5}>
            <Typography variant="caption" fontWeight={600}>
              Toss 매수 유의사항 {data.warnings.length}건
            </Typography>
            <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
              {data.warnings.map((warning) => (
                <Chip
                  key={`${warning.warningType}:${warning.exchange ?? ''}:${warning.startDate ?? ''}`}
                  size="small"
                  label={`${warning.label}${warning.exchange ? ` · ${warning.exchange}` : ''}`}
                  color={warning.blockingForBuy ? 'warning' : 'default'}
                  variant={warning.blockingForBuy ? 'filled' : 'outlined'}
                  sx={{ height: 22, fontSize: '0.7rem' }}
                />
              ))}
            </Stack>
          </Stack>
        </Alert>
      ) : (
        <Typography variant="caption" color="text.secondary">
          Toss 매수 유의사항 없음
        </Typography>
      )}
    </Box>
  )
}

function TossMarketCalendarStrip({
  data,
  isLoading,
  isError,
}: {
  data: TossMarketCalendarView | undefined
  isLoading: boolean
  isError: boolean
}) {
  if (isLoading) return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, pt: 1.5, display: 'flex', alignItems: 'center', gap: 1 }}>
      <CircularProgress size={14} />
      <Typography variant="caption" color="text.secondary">Toss 장 운영 정보 조회 중</Typography>
    </Box>
  )
  if (isError || !data) return null

  return (
    <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap sx={{ px: { xs: 1.5, sm: 2.5 }, pt: 1.5 }}>
      <Chip
        size="small"
        label={`KRX ${data.kr.isRegularOpen ? '정규장 개장' : '정규장 폐장'} · ${fmtTossSession(data.kr.regularSession)}`}
        color={data.kr.isRegularOpen ? 'success' : 'default'}
        variant="outlined"
        sx={{ height: 22, fontSize: '0.7rem' }}
      />
      <Chip
        size="small"
        label={`US ${data.us.isRegularOpen ? '정규장 개장' : '정규장 폐장'} · ${fmtTossSession(data.us.regularSession)}`}
        color={data.us.isRegularOpen ? 'success' : 'default'}
        variant="outlined"
        sx={{ height: 22, fontSize: '0.7rem' }}
      />
    </Stack>
  )
}

function TossOrderPreflightPanel({
  data,
  isLoading,
  isError,
  error,
}: {
  data: TossOrderPreflightView | undefined
  isLoading: boolean
  isError: boolean
  error: unknown
}) {
  if (isLoading) {
    return (
      <Box sx={{ mb: 1.5, p: 1.25, bgcolor: 'action.hover', borderRadius: 1 }}>
        <Stack direction="row" spacing={1} alignItems="center">
          <CircularProgress size={14} />
          <Typography variant="caption" color="text.secondary">Toss 주문 전 검증 중</Typography>
        </Stack>
      </Box>
    )
  }
  if (isError) {
    return (
      <Alert severity="warning" sx={{ mb: 1.5 }}>
        Toss 주문 전 검증 실패: {(error as { message?: string } | null)?.message ?? '연결 진단을 확인하세요.'}
      </Alert>
    )
  }
  if (!data) return null

  return (
    <Box sx={{ mb: 1.5, p: 1.25, bgcolor: 'action.hover', borderRadius: 1 }}>
      <Stack direction="row" spacing={0.75} alignItems="center" flexWrap="wrap" mb={1}>
        <Typography variant="subtitle2" fontWeight={700}>Toss 주문 전 검증</Typography>
        <Chip
          size="small"
          label={data.liquidityOk ? '가능금액 확인' : '가능금액 부족'}
          color={data.liquidityOk ? 'success' : 'warning'}
          variant="outlined"
          sx={{ height: 20, fontSize: '0.68rem' }}
        />
        <Chip
          size="small"
          label={data.orderAdapterSupported ? '주문 연결됨' : '주문 차단'}
          color={data.orderAdapterSupported ? 'success' : 'default'}
          variant="outlined"
          sx={{ height: 20, fontSize: '0.68rem' }}
        />
      </Stack>
      <Stack spacing={0.25}>
        <Typography variant="caption" color="text.secondary">
          주문금액 {fmtTossMoney(data.grossAmount.amount, data.grossAmount.currency)}
          {data.requiredCash ? ` · 필요 ${fmtTossMoney(data.requiredCash.amount, data.requiredCash.currency)}` : ''}
        </Typography>
        {data.buyingPower && (
          <Typography variant="caption" color="text.secondary">
            매수가능금액 {fmtTossMoney(data.buyingPower.amount, data.buyingPower.currency)}
          </Typography>
        )}
        {data.sellableQuantity && (
          <Typography variant="caption" color="text.secondary">
            매도가능수량 {fmtDecimal(data.sellableQuantity, 6)}
          </Typography>
        )}
        {data.commissionRate && (
          <Typography variant="caption" color="text.secondary">
            수수료율 {data.commissionRate}% · 추정 수수료 {data.estimatedCommission ? fmtTossMoney(data.estimatedCommission.amount, data.estimatedCommission.currency) : '-'}
          </Typography>
        )}
      </Stack>
      {(data.blockedReasons.length > 0 || data.warnings.length > 0) && (
        <Stack spacing={0.25} mt={0.75}>
          {data.blockedReasons.map((reason) => (
            <Typography key={reason} variant="caption" color="warning.main" display="block">
              {reason}
            </Typography>
          ))}
          {data.warnings.map((warning) => (
            <Typography key={warning} variant="caption" color="text.secondary" display="block">
              {warning}
            </Typography>
          ))}
        </Stack>
      )}
    </Box>
  )
}

function TossManualTradeVerificationPanel({
  appConfig,
  activeProfile,
  symbol,
  market,
  side,
  orderType,
  quantity,
  price,
  preflight,
  preflightLoading,
  preflightError,
}: {
  appConfig: AppConfigView | undefined
  activeProfile: AccountProfileView | null
  symbol: string
  market: Market
  side: OrderSide
  orderType: OrderType
  quantity: string
  price: string
  preflight: TossOrderPreflightView | undefined
  preflightLoading: boolean
  preflightError: boolean
}) {
  const { mutate: updateProfile, isPending } = useUpdateProfile()
  const [consentError, setConsentError] = useState<string | null>(null)

  if (preflight?.canSubmit) return null

  const cleanQuantity = quantity.replace(/,/g, '')
  const cleanPrice = price.replace(/,/g, '')
  const quantityValue = Number(cleanQuantity)
  const priceValue = Number(cleanPrice)
  const symbolReady = symbol.trim().length > 0
  const quantityReady = Number.isFinite(quantityValue) && quantityValue > 0
  const priceReady = Number.isFinite(priceValue) && priceValue > 0
  const limitOrderReady = market === 'US' || orderType === 'Limit'
  const consentReady = activeProfile?.live_trading_consent ?? false
  const readOnlyReady = !!preflight && preflight.liquidityOk && preflight.safetyOk && !preflightError
  const estimatedAmount = quantityReady && priceReady ? quantityValue * priceValue : null

  const handleConsent = () => {
    if (!activeProfile) return
    setConsentError(null)
    updateProfile(
      { id: activeProfile.id, live_trading_consent: true },
      { onError: (e) => setConsentError((e as { message?: string } | null)?.message ?? String(e)) },
    )
  }

  const statusChip = (label: string, ok: boolean, pending = false) => (
    <Chip
      size="small"
      label={pending ? `${label} 확인 중` : label}
      color={ok ? 'success' : pending ? 'default' : 'warning'}
      variant="outlined"
      sx={{ height: 22, fontSize: '0.7rem' }}
    />
  )

  return (
    <Box sx={{ mb: 1.5, p: 1.5, border: 1, borderColor: 'divider', borderRadius: 1 }}>
      <Stack spacing={1.25}>
        <Stack direction="row" spacing={0.75} alignItems="center" flexWrap="wrap" useFlexGap>
          <Typography variant="subtitle2" fontWeight={700}>Toss 소액 수동매매 검증</Typography>
          <Chip size="small" label="실거래 gate" color="warning" variant="outlined" sx={{ height: 22, fontSize: '0.7rem' }} />
          {appConfig?.active_broker_account_id && (
            <Chip
              size="small"
              label={`accountSeq ${appConfig.active_broker_account_id}`}
              variant="outlined"
              sx={{ height: 22, fontSize: '0.7rem' }}
            />
          )}
        </Stack>

        <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
          {statusChip('종목', symbolReady)}
          {statusChip('지정가', limitOrderReady)}
          {statusChip('수량', quantityReady)}
          {statusChip('가격', priceReady)}
          {statusChip('동의', consentReady)}
          {statusChip('사전검증', readOnlyReady, preflightLoading)}
          {statusChip('주문 adapter', preflight?.orderAdapterSupported ?? false)}
        </Stack>

        <Box>
          <Typography variant="caption" color="text.secondary" display="block">
            검증 주문: {symbolReady ? symbol : '종목 미선택'} · {side === 'Buy' ? '매수' : '매도'} · {market === 'US' ? '미국' : '국내'}
            {estimatedAmount !== null ? ` · 예상 ${market === 'US' ? '$' + estimatedAmount.toFixed(2) : fmt(Math.round(estimatedAmount)) + '원'}` : ''}
          </Typography>
          <Typography variant="caption" color="text.secondary" display="block">
            소액 검증은 지정가 주문, 즉시 사전검증, 주문 ID 저장, 미체결 시 취소 확인까지 끝난 뒤 수동 주문 제한을 해제합니다.
          </Typography>
        </Box>

        {preflight && (
          <Stack spacing={0.25}>
            {preflight.blockedReasons.map((reason) => (
              <Typography key={reason} variant="caption" color="warning.main">{reason}</Typography>
            ))}
            {!preflight.orderAdapterSupported && (
              <Typography variant="caption" color="text.secondary">
                현재 주문 생성 adapter가 gate 뒤에 있어 실제 주문 제출은 계속 차단됩니다.
              </Typography>
            )}
          </Stack>
        )}

        {consentError && <Alert severity="error" sx={{ py: 0.5 }}>{consentError}</Alert>}

        {!consentReady && (
          <Box>
            <Button
              size="small"
              variant="outlined"
              color="warning"
              onClick={handleConsent}
              disabled={!activeProfile || isPending}
              startIcon={isPending ? <CircularProgress size={14} color="inherit" /> : undefined}
            >
              소액 실거래 검증 동의 저장
            </Button>
          </Box>
        )}
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

  const { data: appConfig }                                         = useAppConfig()
  const { data: profiles = [] }                                     = useProfiles()
  const isTossActive = appConfig?.active_broker_id === 'toss'
  const isKisActive = appConfig?.active_broker_id === 'kis'
  const { data: balance }                                           = useBalance({ enabled: isKisActive })
  const { data: overseasBalance }                                   = useOverseasBalance({ enabled: isKisActive })
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
  const activeProfile = profiles.find((profile) => profile.id === appConfig?.active_profile_id) ?? null

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
      setErrorMsg('Toss 수동 주문은 소액 검증 gate 통과 전까지 차단됩니다.')
      return
    }
    if (isTossActive) {
      setErrorMsg('Toss 주문 제출 IPC는 아직 수동 주문 버튼에 연결되지 않았습니다. 소액 검증 주문 command 연결 후 제출됩니다.')
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

  return (
    <Box>
      <Stack direction="row" alignItems="center" gap={1.5} mb={2} flexWrap="wrap">
        <Typography variant="h5" fontWeight={700}>Trading</Typography>
        <BrokerScopeIndicator appConfig={appConfig} compact />
      </Stack>

      {/* 1. 보유 종목 */}
      <Paper sx={{ p: { xs: 1.5, sm: 2.5 }, mb: 2 }}>
        <Stack
          direction="row" alignItems="center" justifyContent="space-between"
          mb={1.5} flexWrap="wrap" gap={1}
        >
          <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
            <Typography variant="subtitle1" fontWeight={600}>보유 종목</Typography>
            {isTossActive && (
              <Chip
                size="small"
                label={appConfig?.active_broker_account_id
                  ? `Toss · accountSeq ${appConfig.active_broker_account_id}`
                  : 'Toss'}
                color="secondary"
                variant="outlined"
              />
            )}
          </Stack>
          {!isTossActive && balance?.summary && (
            <Typography variant="caption" color="text.secondary" noWrap>
              예수금 {fmt(parseInt(balance.summary.dnca_tot_amt))}원
            </Typography>
          )}
        </Stack>
        <Divider sx={{ mb: 1.5 }} />

        {isTossActive ? (
          brokerHoldingsLoading ? (
            <Box sx={{ py: 2, display: 'flex', justifyContent: 'center' }}>
              <CircularProgress size={20} />
            </Box>
          ) : brokerHoldingsError ? (
            <Alert severity="warning" sx={{ py: 0.5 }}>
              Toss 보유종목 조회 실패 — Settings의 Toss Client ID, Client Secret, accountSeq를 확인하세요.
            </Alert>
          ) : brokerHoldings.length === 0 ? (
            <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
              보유한 Toss 종목이 없습니다.
            </Typography>
          ) : (
            <TableContainer>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>종목명</TableCell>
                    <TableCell align="right">수량</TableCell>
                    <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>평균단가</TableCell>
                    <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>현재가</TableCell>
                    <TableCell align="right">평가손익</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {brokerHoldings.map((item) => {
                    const pnl = Number(item.unrealizedPnl?.amount.replace(/,/g, '') ?? 0)
                    const pnlPositive = Number.isFinite(pnl) ? pnl >= 0 : true
                    return (
                      <TableRow
                        key={`${item.brokerId}:${item.market}:${item.symbol}`}
                        hover
                        sx={{ cursor: 'pointer' }}
                        onClick={() => handleSelectBrokerHolding(item)}
                      >
                        <TableCell>
                          <Typography variant="body2" fontWeight={500}>{item.symbolName || item.symbol}</Typography>
                          <Typography variant="caption" color="text.secondary">
                            {item.symbol} · {tossMarketLabel(item.market)}
                          </Typography>
                        </TableCell>
                        <TableCell align="right">{fmtDecimal(item.quantity, 6)}</TableCell>
                        <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                          {fmtBrokerMoney(item.averagePrice)}
                        </TableCell>
                        <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                          {fmtBrokerMoney(item.currentPrice)}
                        </TableCell>
                        <TableCell
                          align="right"
                          sx={{ color: pnlPositive ? 'success.main' : 'error.main', fontWeight: 600 }}
                        >
                          {item.unrealizedPnl ? fmtBrokerMoney(item.unrealizedPnl) : '-'}
                        </TableCell>
                      </TableRow>
                    )
                  })}
                </TableBody>
              </Table>
            </TableContainer>
          )
        ) : (
          <>
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
          </>
        )}
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
              helperText={isTossActive ? 'Enter 또는 검색 버튼 — Toss read-only 시세 조회' : 'Enter 또는 검색 버튼 — 나스닥·뉴욕·AMEX 자동 감지'}
            />
            {symbol && (
              <Chip
                label={isTossActive ? 'Toss' : usExchange}
                size="small"
                color={isTossActive ? 'secondary' : 'info'}
                variant="outlined"
                sx={{ mt: 1, flexShrink: 0 }}
              />
            )}
          </Stack>
        )}

        {symbol && isTossActive && (
          <Box sx={{ mt: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
            <TossMarketCalendarStrip
              data={tossCalendar}
              isLoading={isTossCalendarLoading}
              isError={isTossCalendarError}
            />
            <TossMarketSnapshotCard symbol={symbol} />
            <TossStockSafetyCard
              data={tossSafety}
              isLoading={isTossSafetyLoading}
              isError={isTossSafetyError}
              error={tossSafetyError}
            />
          </Box>
        )}
        {symbol && !isTossActive && market === 'KR' && symbol.length === 6 && (
          <Box sx={{ mt: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
            <KrStockInfoCard symbol={symbol} />
          </Box>
        )}
        {symbol && !isTossActive && market === 'US' && (
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
                  label={isTossActive ? `${symbol} (Toss read-only)` : market === 'KR' ? symbol : symbol + ' (' + usExchange + ')'}
                  size="small"
                  color={isTossActive ? 'secondary' : market === 'US' ? 'info' : 'default'}
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

            {isTossActive && !tossManualOrderReady && (
              <Alert severity="info" sx={{ mb: 1.5 }}>
                Toss 프로파일은 현재 read-only 시세와 잔고 조회만 지원합니다. 주문 생성은 소액 검증 gate 이후 연결됩니다.
              </Alert>
            )}

            {isTossActive && tossSafety?.buyBlockReason && (
              <Alert severity="warning" sx={{ mb: 1.5 }}>
                {tossSafety.buyBlockReason}
              </Alert>
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
                            disabled={isTossActive ? !(tossCurrentPrice && Number.isFinite(tossCurrentPrice)) : !(market === 'KR' ? krCurrentPrice : usCurrentPrice)}
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

            {isTossActive && symbol && quantity && (
              <TossOrderPreflightPanel
                data={tossPreflight}
                isLoading={isTossPreflightLoading}
                isError={isTossPreflightError}
                error={tossPreflightError}
              />
            )}

            {isTossActive && (
              <TossManualTradeVerificationPanel
                appConfig={appConfig}
                activeProfile={activeProfile}
                symbol={symbol}
                market={market}
                side={side}
                orderType={orderType}
                quantity={quantity}
                price={price}
                preflight={tossPreflight}
                preflightLoading={isTossPreflightLoading}
                preflightError={isTossPreflightError}
              />
            )}

            {result   && <Alert severity="success" sx={{ mb: 1.5 }}>{result}</Alert>}
            {errorMsg && <Alert severity="error"   sx={{ mb: 1.5 }}>{errorMsg}</Alert>}

            {market === 'US' && isPaperTrading && (
              <Alert severity={isPaperUnsupportedUsSell ? 'warning' : 'info'} sx={{ mb: 1.5 }}>
                {isPaperUnsupportedUsSell
                  ? `${normalizedUsSymbol || '선택 종목'} (${overseasOrderExchange}) 매도는 KIS 모의투자에서 제한될 수 있어 주문 전 차단됩니다.`
                  : '모의 해외 주문은 USD 지정가로만 전송합니다. 일부 AMEX/ETF 종목은 모의투자 주문 universe에서 제한될 수 있습니다.'}
              </Alert>
            )}

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
              disabled={isPending || !symbol || isPaperUnsupportedUsSell || (isTossActive && !tossManualOrderReady)}
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
                {isTossActive
                  ? <TossMarketSnapshotCard symbol={symbol} />
                  : market === 'KR'
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
            {isTossActive && symbol ? (
              <Box sx={{ p: { xs: 1, sm: 2 } }}>
                <StockChart symbol={symbol} stockName={stockName} source="toss" />
              </Box>
            ) : market === 'US' && symbol ? (
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
