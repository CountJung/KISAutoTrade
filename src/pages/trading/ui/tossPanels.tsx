import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import { useTossMarketSnapshot } from '../../../api/hooks'
import type {
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
