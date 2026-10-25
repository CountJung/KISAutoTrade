import { ConsecutiveEntryControls, consecutiveEntryParamError, isConsecutiveEntryParam } from './consecutiveEntryControls'
import { MomentumEntryControls, isMomentumEntryParam, momentumEntryParamError } from './momentumEntryControls'
import { FailedBreakoutControls, isFailedBreakoutPolicyParam, failedBreakoutPolicyError } from './failedBreakoutControls'
import { RsiControls, isRsiPolicyParam, rsiPolicyError } from './rsiControls'
import { StrongCloseControls, isStrongClosePolicyParam, strongClosePolicyError } from './strongCloseControls'
import { VolatilityExpansionControls, isVolatilityExpansionParam, volatilityExpansionParamError } from './volatilityExpansionControls'
import { MaCrossEntryControls, isMaCrossEntryParam, maCrossEntryParamError } from './maCrossEntryControls'
import { MeanReversionControls, isMeanReversionPolicyParam, meanReversionPolicyError } from './meanReversionControls'
import { DeviationControls, isDeviationPolicyParam, deviationPolicyError } from './deviationControls'
import { TrendFilterControls, isTrendFilterParam, trendFilterParamError } from './trendFilterControls'
import { getStrategyType } from './strategyMetadata'

type Params = Record<string, unknown>
type Props = { type: string; params: Params; disabled: boolean; onChange: (key: string, value: number | string) => void }
// 전략별 계약과 raw 해석은 각 controls에 남기고 페이지 조립만 담당한다.
const policies = {
  failed_breakout: { Controls: FailedBreakoutControls, isParam: isFailedBreakoutPolicyParam, error: failedBreakoutPolicyError },
  volatility_expansion: { Controls: VolatilityExpansionControls, isParam: isVolatilityExpansionParam, error: volatilityExpansionParamError },
  strong_close: { Controls: StrongCloseControls, isParam: isStrongClosePolicyParam, error: strongClosePolicyError },
  rsi: { Controls: RsiControls, isParam: isRsiPolicyParam, error: rsiPolicyError },
  ma_cross: { Controls: MaCrossEntryControls, isParam: isMaCrossEntryParam, error: maCrossEntryParamError },
  mean_reversion: { Controls: MeanReversionControls, isParam: isMeanReversionPolicyParam, error: meanReversionPolicyError },
  deviation: { Controls: DeviationControls, isParam: isDeviationPolicyParam, error: deviationPolicyError },
  momentum: { Controls: MomentumEntryControls, isParam: isMomentumEntryParam, error: momentumEntryParamError },
  consecutive_move: { Controls: ConsecutiveEntryControls, isParam: isConsecutiveEntryParam, error: consecutiveEntryParamError },
  trend_filter: { Controls: TrendFilterControls, isParam: isTrendFilterParam, error: trendFilterParamError },
}
function policy(type: string) { return policies[type as keyof typeof policies] }
export function isStrategyPolicyParam(type: string, key: string) { return policy(type)?.isParam(key) ?? false }
export function preservesStrategyRawParam(id: string, key: string) {
  const type = getStrategyType(id)
  return isStrategyPolicyParam(type, key) || (type === 'ma_cross' && (key === 'short_period' || key === 'long_period'))
}
export function strategyPolicyError(type: string, params: Params) { return policy(type)?.error(params) ?? null }
export function StrategyPolicyControls({ type, ...props }: Props) {
  const Controls = policy(type)?.Controls
  return Controls ? <Controls {...props} /> : null
}
