import { useState } from 'react'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import { useUpdateProfile } from '../../../api/hooks'
import type {
  AccountProfileView,
  AppConfigView,
  OrderSide,
  OrderType,
  TossOrderPreflightView,
} from '../../../api/types'
import { fmtBrokerMoney, fmtNumber } from '../../../shared/lib'

type TradingMarket = 'KR' | 'US'

export function TossManualTradeVerificationPanel({
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
  adapterBlockedMessage = '현재 주문 생성 adapter가 gate 뒤에 있어 실제 주문 제출은 계속 차단됩니다.',
}: {
  appConfig: AppConfigView | undefined
  activeProfile: AccountProfileView | null
  symbol: string
  market: TradingMarket
  side: OrderSide
  orderType: OrderType
  quantity: string
  price: string
  preflight: TossOrderPreflightView | undefined
  preflightLoading: boolean
  preflightError: boolean
  adapterBlockedMessage?: string
}) {
  const { mutate: updateProfile, isPending } = useUpdateProfile()
  const [consentError, setConsentError] = useState<string | null>(null)

  if (preflight?.canSubmit) return null

  const cleanQuantity = quantity.replace(/,/g, '')
  const cleanPrice = price.replace(/,/g, '')
  const quantityValue = Number(cleanQuantity)
  const priceValue = Number(cleanPrice)
  const preflightPriceValue = preflight
    ? Number(preflight.price.amount.replace(/,/g, ''))
    : NaN
  const effectivePriceValue =
    Number.isFinite(priceValue) && priceValue > 0 ? priceValue : preflightPriceValue
  const symbolReady = symbol.trim().length > 0
  const quantityReady = Number.isFinite(quantityValue) && quantityValue > 0
  const priceReady = orderType === 'Market'
    ? Number.isFinite(preflightPriceValue) && preflightPriceValue > 0
    : Number.isFinite(priceValue) && priceValue > 0
  const orderShapeReady = orderType === 'Market' || market === 'US' || orderType === 'Limit'
  const consentReady = activeProfile?.live_trading_consent ?? false
  const readOnlyReady = !!preflight && preflight.liquidityOk && preflight.safetyOk && !preflightError
  const estimatedAmount =
    quantityReady && Number.isFinite(effectivePriceValue) && effectivePriceValue > 0
      ? quantityValue * effectivePriceValue
      : null
  const orderTypeLabel = orderType === 'Market' ? '시장가' : '지정가'

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
          {statusChip(orderTypeLabel, orderShapeReady)}
          {statusChip('수량', quantityReady)}
          {statusChip('가격', priceReady, orderType === 'Market' && preflightLoading)}
          {statusChip('동의', consentReady)}
          {statusChip('사전검증', readOnlyReady, preflightLoading)}
          {statusChip('주문 adapter', preflight?.orderAdapterSupported ?? false)}
        </Stack>

        <Box>
          <Typography variant="caption" color="text.secondary" display="block">
            검증 주문: {symbolReady ? symbol : '종목 미선택'} · {side === 'Buy' ? '매수' : '매도'} · {orderTypeLabel} · {market === 'US' ? '미국' : '국내'}
            {preflight ? ` · 예상 ${fmtBrokerMoney(preflight.grossAmount)}` : estimatedAmount !== null ? ` · 예상 ${market === 'US' ? '$' + estimatedAmount.toFixed(2) : fmtNumber(Math.round(estimatedAmount)) + '원'}` : ''}
          </Typography>
          <Typography variant="caption" color="text.secondary" display="block">
            {orderType === 'Market'
              ? '시장가 1주 조건은 현재 read-only 사전검증만 수행합니다. 실제 제출은 별도 승인, 주문 ID 저장, 체결/취소 확인 gate가 끝난 뒤 열립니다.'
              : '소액 검증은 지정가 주문, 즉시 사전검증, 주문 ID 저장, 미체결 시 취소 확인까지 끝난 뒤 수동 주문 제한을 해제합니다.'}
          </Typography>
          {orderType === 'Market' && (
            <Typography variant="caption" color="warning.main" display="block">
              시장가 실주문은 체결 가격이 확정되지 않으므로 현재 안전 정책상 제출 버튼은 계속 차단됩니다.
            </Typography>
          )}
        </Box>

        {preflight && (
          <Stack spacing={0.25}>
            {preflight.blockedReasons.map((reason) => (
              <Typography key={reason} variant="caption" color="warning.main">{reason}</Typography>
            ))}
            {!preflight.orderAdapterSupported && (
              <Typography variant="caption" color="text.secondary">
                {adapterBlockedMessage}
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
