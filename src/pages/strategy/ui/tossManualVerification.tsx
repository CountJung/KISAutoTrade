import { useMemo } from 'react'

import { useProfiles, useTossOrderPreflight } from '../../../api/hooks'
import type { AppConfigView, PriceConditionSymbolConfig } from '../../../api/types'
import { TossManualTradeVerificationPanel } from '../../../features/manual-order'

function toOrderNumber(value: number | undefined) {
  return Number.isFinite(value) && (value ?? 0) > 0 ? String(value) : ''
}

export function TossPriceConditionVerificationGate({
  appConfig,
  symbols,
  scopeMatchesActive,
}: {
  appConfig: AppConfigView | undefined
  symbols: PriceConditionSymbolConfig[]
  scopeMatchesActive: boolean
}) {
  const { data: profiles = [] } = useProfiles()
  const activeProfile = profiles.find((profile) => profile.id === appConfig?.active_profile_id) ?? null
  const candidate = useMemo(
    () => symbols.find((item) => item.symbol.trim().length > 0 && item.quantity > 0) ?? null,
    [symbols],
  )

  const symbol = candidate?.symbol ?? ''
  const quantity = toOrderNumber(candidate?.quantity)
  const price = toOrderNumber(candidate?.buy_trigger_price)
  const market = candidate?.is_overseas ? 'US' : 'KR'
  const shouldCheckPreflight = scopeMatchesActive && !!symbol && !!quantity && !!price
  const {
    data: preflight,
    isLoading: preflightLoading,
    isError: preflightError,
  } = useTossOrderPreflight(
    {
      symbol,
      side: 'Buy',
      quantity,
      price: price || null,
    },
    { enabled: shouldCheckPreflight },
  )

  if (!scopeMatchesActive) return null

  return (
    <TossManualTradeVerificationPanel
      appConfig={appConfig}
      activeProfile={activeProfile}
      symbol={symbol}
      market={market}
      side="Buy"
      orderType="Limit"
      quantity={quantity}
      price={price}
      preflight={preflight}
      preflightLoading={preflightLoading}
      preflightError={preflightError}
    />
  )
}
