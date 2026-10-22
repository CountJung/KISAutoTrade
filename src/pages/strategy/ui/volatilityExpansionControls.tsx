import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'

const fields = STRATEGY_PARAM_META.volatility_expansion
const defaults: Record<string, number> = { lookback_days: 10, expansion_factor: 2, stop_loss_pct: 3 }
export function isVolatilityExpansionParam(key: string) { return fields.some((field) => field.key === key) }
function fieldError(field: StrategyParamMeta, value: unknown): string | null {
  if (value === undefined) return null
  const integer = field.key === 'lookback_days' || field.key === 'max_hold_observations'
  return !isFiniteNumberInRange(value, { min: field.min, max: field.max, integer })
    ? `${field.label}: ${field.min}~${field.max}${integer ? ' 정수' : ' 유한한 숫자'}를 입력하세요.` : null
}
function profitPairError(params: Record<string, unknown>) {
  const activation = params.profit_activation_pct === undefined ? 0 : params.profit_activation_pct
  const trailing = params.profit_trailing_pct === undefined ? 0 : params.profit_trailing_pct
  return (activation === 0 && trailing === 0) || (typeof activation === 'number' && activation > 0 && typeof trailing === 'number' && trailing > 0)
    ? null : '수익 보호 시작·되밀림은 둘 다 0(비활성)이거나 둘 다 양수여야 합니다.'
}
export function volatilityExpansionParamError(params: Record<string, unknown>): string | null {
  return fields.map((field) => fieldError(field, params[field.key])).find(Boolean) ?? profitPairError(params)
}
function VolatilityNumericField({ field, params, disabled, onChange }: {
  field: StrategyParamMeta; params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const value = params[field.key]
  const fallback = String(defaults[field.key] ?? 0)
  const [raw, setRaw] = useState(() => value === undefined ? fallback : String(value))
  useEffect(() => { setRaw((previous) => Object.is(parseDecimalDraft(previous), value) ? previous : value === undefined ? fallback : String(value)) }, [value, fallback])
  const error = fieldError(field, value) ?? (field.key.startsWith('profit_') ? profitPairError(params) : null)
  return <TextField fullWidth size="small" label={field.label} value={raw} disabled={disabled} error={!!error}
    helperText={error ?? field.description} inputProps={{ inputMode: field.key === 'lookback_days' || field.key === 'max_hold_observations' ? 'numeric' : 'decimal' }}
    onChange={(event) => { setRaw(event.target.value); onChange(field.key, parseDecimalDraft(event.target.value)) }} />
}
export function VolatilityExpansionControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="volatility-expansion-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기본 10완료 일봉·2배·손절3%를 유지합니다. 당일 고가−저가 &gt; 이전 N완료 일봉 범위 산술평균 × 배수이며 현재가 &gt; 당일 시가를 모두 충족해야 합니다. 두 조건 모두 엄격한 초과이며 0범위도 평균 일수에 포함합니다. 배수0도 양의 당일 범위가 필요합니다.
      일봉 replay는 D 종가에 완성 OHLC로 신호를 판단합니다. nextOpen 선택 시 다음 관측 시가에 체결을 가정하며 sameClose 선택 시 신호 봉 종가에 체결을 가정합니다. 장중 최초 발동·고저 순서·실제 체결을 확인하지 않습니다. 이 전략의 generic 미리보기는 일봉만 지원합니다.
      실시간은 검증된 시장일의 첫 관측 가격을 시가 대용으로 쓰고 관측한 고저만 추적하므로 거래소 시가·당일 전체 고저와 다릅니다. 같은 날 자료 갱신으로 관측 범위를 초기화하지 않습니다. 이전 완료 자료가 부족·무효·오래됨이면 신규 매수만 보류하고 보유 보호는 계속합니다.
      실제 보유 수량·종목 통화 기준 평균 매수가를 확인하며 미보유에서만 매수합니다. 매도는 실제 잔량 전량 후보이고 신호는 체결이 아닙니다. 원래 손절0도 평균가 이하 청산이며 비활성이 아닙니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      {fields.map((field) => <Grid item xs={12} sm={6} key={field.key}><VolatilityNumericField field={field} params={params} disabled={disabled} onChange={onChange} /></Grid>)}
    </Grid>
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      새 세 옵션만 누락/0 비활성입니다. 수익 보호는 실제 평균 매수가 대비 시작 이익률에서 한 번 무장한 뒤 실제 보유 중 관측 최고가의 되밀림에서 잔량 전량 매도 후보를 냅니다. 진입 전 고가·현재 봉 전체 고가를 최고가로 쓰지 않습니다.
      기존 손절 → 수익 보호 → 보유 한도 순입니다. 주문 거절·부분 체결·설정 변경으로 알려진 최고가·무장·보유 관측 수를 초기화하지 않으며 조건이 지속되면 실제 잔량으로 다시 후보를 냅니다.
      보유 관측 수는 실제 첫 양수 체결 시 0, 이후 첫 보유 관측을 1로 셉니다. 복원 보유의 나이가 미확인이면 활성 보유 한도에서 다음 보유 관측에 보수적 매도 후보를 냅니다. 관측 수는 일수나 실제 보유시간 보장이 아닙니다.
      공통 주문 제한·다음 관측 시가 체결·가격 갭으로 손실률이나 실제 보유시간을 보장하지 않습니다. 연구 후보를 운영에 자동 적용하지 않습니다.
    </Alert>
  </Stack>
}
function numericLabel(params: Record<string, unknown>, field: StrategyParamMeta, unit: string) {
  const value = params[field.key]
  return value === undefined ? `${defaults[field.key] ?? 0}${unit} (누락 · 기본${defaults[field.key] === undefined ? ' 비활성' : ''})`
    : fieldError(field, value) ? `확인 필요 (${String(value)})` : `${value}${unit}`
}
export function VolatilityExpansionSummary({ params }: { params: Record<string, unknown> }) {
  return <Typography variant="caption" display="block" sx={{ overflowWrap: 'anywhere' }} data-testid="volatility-expansion-summary">
    변동성 확장 이전 {numericLabel(params, fields[0], '완료봉')} 범위 평균 × {numericLabel(params, fields[1], '배')} 초과 · 현재가 &gt; 당일 시가
    {' · '}손절 {numericLabel(params, fields[2], '%')} (0도 평균가 이하 청산)
    {' · '}수익 보호 시작 {numericLabel(params, fields[3], '%')} / 되밀림 {numericLabel(params, fields[4], '%')}{profitPairError(params) ? ' (조합 확인 필요)' : ''}
    {' · '}최대 보유 {numericLabel(params, fields[5], '관측')} · 새 세 옵션 0 비활성
    {' · '}D 종가 완성 OHLC 신호. 체결 가정은 sameClose 선택 시 신호 봉 종가, nextOpen 선택 시 다음 관측 시가입니다. 설정 요약이며 이전 엔진의 새 보호 적용·장중 최초 발동·실제 체결을 증명하지 않습니다.
  </Typography>
}
