import { useState } from 'react'

import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Typography from '@mui/material/Typography'

import { useModifyTossOrder, useTossMarketSnapshot, useTossOpenOrders } from '../../../api/hooks'
import type {
  CmdError,
  TossOpenOrderView,
  TossMarketCalendarView,
  TossOrderPreflightView,
  TossStockSafetyView,
} from '../../../api/types'
import { fmtDecimalString } from '../../../shared/lib'

function fmtTossMoney(amount: string, currency: string) {
  return currency === 'KRW'
    ? `${fmtDecimalString(amount, 0)}원`
    : `$${fmtDecimalString(amount, 4)}`
}

function tossMarketLabel(market: 'kr' | 'us') {
  return market === 'kr' ? '국내' : '미국'
}

function tossSideLabel(side: string) {
  return side === 'BUY' ? '매수' : side === 'SELL' ? '매도' : side
}

function tossOrderTypeLabel(orderType: string) {
  return orderType === 'MARKET' ? '시장가' : orderType === 'LIMIT' ? '지정가' : orderType
}

function tossStatusColor(status: string): 'default' | 'success' | 'warning' | 'error' {
  if (status === 'FILLED') return 'success'
  if (status === 'PARTIAL_FILLED') return 'warning'
  if (status.includes('REJECTED') || status === 'CANCELED') return 'error'
  return 'default'
}

function fmtTossSession(session: TossMarketCalendarView['kr']['regularSession']) {
  if (!session) return '휴장'
  const format = (value: string) => new Date(value).toLocaleTimeString('ko-KR', {
    hour: '2-digit',
    minute: '2-digit',
  })
  return `${format(session.startTime)}~${format(session.endTime)}`
}

export function TossMarketSnapshotCard({ symbol }: { symbol: string }) {
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
          {bestAsk && <Typography variant="caption" color="text.secondary" display="block">잔량 {fmtDecimalString(bestAsk.volume, 4)}</Typography>}
        </Grid>
        <Grid item xs={6} sm={3}>
          <Typography variant="caption" color="text.secondary" display="block">매수 1호가</Typography>
          <Typography variant="caption" fontWeight={600} color="primary.main">
            {bestBid ? fmtTossMoney(bestBid.price, data.orderbook.currency) : '-'}
          </Typography>
          {bestBid && <Typography variant="caption" color="text.secondary" display="block">잔량 {fmtDecimalString(bestBid.volume, 4)}</Typography>}
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
                {fmtTossMoney(trade.price, trade.currency)} · {fmtDecimalString(trade.volume, 4)}
              </Typography>
            ))}
          </Stack>
        </Box>
      )}
    </Box>
  )
}

export function TossStockSafetyCard({
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

export function TossMarketCalendarStrip({
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

export function TossOrderPreflightPanel({
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
            매도가능수량 {fmtDecimalString(data.sellableQuantity, 6)}
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

export function TossOpenOrdersPanel({ symbol }: { symbol?: string }) {
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editOrderType, setEditOrderType] = useState<'MARKET' | 'LIMIT'>('LIMIT')
  const [editQuantity, setEditQuantity] = useState('')
  const [editPrice, setEditPrice] = useState('')
  const [message, setMessage] = useState<string | null>(null)
  const [messageSeverity, setMessageSeverity] = useState<'success' | 'error'>('success')

  const {
    data: orders = [],
    isLoading,
    isError,
    error,
  } = useTossOpenOrders(null)
  const { mutate: modifyOrder, isPending: modifying } = useModifyTossOrder()

  const startEdit = (order: TossOpenOrderView) => {
    setEditingId(order.orderId)
    setEditOrderType(order.orderType === 'MARKET' ? 'MARKET' : 'LIMIT')
    setEditQuantity(order.quantity)
    setEditPrice(order.price ?? '')
    setMessage(null)
    setMessageSeverity('success')
  }

  const submitEdit = () => {
    if (!editingId) return
    setMessage(null)
    setMessageSeverity('success')
    modifyOrder(
      {
        orderId: editingId,
        orderType: editOrderType,
        quantity: editQuantity.trim() || null,
        price: editOrderType === 'LIMIT' ? editPrice.trim() || null : null,
        confirmHighValueOrder: false,
      },
      {
        onSuccess: (result) => {
          setMessageSeverity('success')
          setMessage(result.message)
          setEditingId(null)
        },
        onError: (e) => {
          const err = e as CmdError | Error | null
          setMessageSeverity('error')
          setMessage(err instanceof Error ? err.message : err?.message ?? String(e))
        },
      },
    )
  }

  const currentSymbol = symbol?.trim().toUpperCase()
  const sortedOrders = [...orders].sort((a, b) => {
    if (!currentSymbol) return a.orderedAt.localeCompare(b.orderedAt) * -1
    const aMatch = a.symbol === currentSymbol ? 0 : 1
    const bMatch = b.symbol === currentSymbol ? 0 : 1
    return aMatch - bMatch || b.orderedAt.localeCompare(a.orderedAt)
  })

  return (
    <Box sx={{ mt: 2 }}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1} mb={1}>
        <Typography variant="subtitle2" fontWeight={700}>
          Toss 접수 주문
        </Typography>
        {isLoading && <CircularProgress size={16} />}
      </Stack>

      {isError && (
        <Alert severity="warning" sx={{ mb: 1.5 }}>
          Toss 접수 주문 조회 실패: {(error as { message?: string } | null)?.message ?? '연결 상태를 확인하세요.'}
        </Alert>
      )}

      {!isLoading && !isError && sortedOrders.length === 0 && (
        <Alert severity="info" sx={{ mb: 1.5 }}>
          현재 Toss 계좌에 접수된 미체결 주문이 없습니다.
        </Alert>
      )}

      {sortedOrders.length > 0 && (
        <Box sx={{ overflowX: 'auto', border: 1, borderColor: 'divider', borderRadius: 1 }}>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>종목</TableCell>
                <TableCell>구분</TableCell>
                <TableCell>유형</TableCell>
                <TableCell align="right">수량</TableCell>
                <TableCell align="right">가격</TableCell>
                <TableCell align="right">체결</TableCell>
                <TableCell>상태</TableCell>
                <TableCell align="right">작업</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {sortedOrders.map((order) => {
                const editing = editingId === order.orderId
                return (
                  <TableRow key={order.orderId} sx={order.symbol === currentSymbol ? { bgcolor: 'action.hover' } : undefined}>
                    <TableCell>
                      <Typography variant="body2" fontWeight={700}>{order.symbol}</Typography>
                      <Typography variant="caption" color="text.secondary">{order.orderId}</Typography>
                    </TableCell>
                    <TableCell>{tossSideLabel(order.side)}</TableCell>
                    <TableCell>
                      {editing ? (
                        <ToggleButtonGroup
                          value={editOrderType}
                          exclusive
                          size="small"
                          onChange={(_, value) => value && setEditOrderType(value)}
                        >
                          <ToggleButton value="LIMIT">지정가</ToggleButton>
                          <ToggleButton value="MARKET">시장가</ToggleButton>
                        </ToggleButtonGroup>
                      ) : tossOrderTypeLabel(order.orderType)}
                    </TableCell>
                    <TableCell align="right">
                      {editing ? (
                        <TextField
                          value={editQuantity}
                          onChange={(e) => setEditQuantity(e.target.value.replace(/[^0-9.]/g, ''))}
                          size="small"
                          sx={{ width: 88 }}
                        />
                      ) : fmtDecimalString(order.quantity, 6)}
                    </TableCell>
                    <TableCell align="right">
                      {editing ? (
                        <TextField
                          value={editPrice}
                          onChange={(e) => setEditPrice(e.target.value.replace(/[^0-9.]/g, ''))}
                          size="small"
                          disabled={editOrderType === 'MARKET'}
                          sx={{ width: 112 }}
                        />
                      ) : order.price ? fmtTossMoney(order.price, order.currency) : '-'}
                    </TableCell>
                    <TableCell align="right">{fmtDecimalString(order.filledQuantity, 6)}</TableCell>
                    <TableCell>
                      <Chip
                        size="small"
                        label={order.status}
                        color={tossStatusColor(order.status)}
                        variant="outlined"
                        sx={{ height: 20, fontSize: '0.68rem' }}
                      />
                    </TableCell>
                    <TableCell align="right">
                      {editing ? (
                        <Stack direction="row" spacing={0.5} justifyContent="flex-end">
                          <Button size="small" onClick={() => setEditingId(null)} disabled={modifying}>
                            취소
                          </Button>
                          <Button
                            size="small"
                            variant="contained"
                            onClick={submitEdit}
                            disabled={modifying || !editQuantity || (editOrderType === 'LIMIT' && !editPrice)}
                          >
                            정정
                          </Button>
                        </Stack>
                      ) : (
                        <Button size="small" variant="outlined" onClick={() => startEdit(order)}>
                          수정
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        </Box>
      )}

      {message && (
        <Alert severity={messageSeverity} sx={{ mt: 1.5 }}>
          {message}
        </Alert>
      )}
    </Box>
  )
}
