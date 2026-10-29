import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'

const fields = STRATEGY_PARAM_META.ma_cross.slice(2)
export function isMaCrossEntryParam(key: string) { return fields.some((field) => field.key === key) }
export function maCrossEntryFiltersDisabled(params: Record<string, unknown>) { return fields.every(({ key }) => params[key] === undefined || params[key] === 0) }
function fieldError(field: StrategyParamMeta, value: unknown) {
  if (value === undefined) return null
  return !isFiniteNumberInRange(value, { min: field.min, max: field.max, integer: field.key === 'long_trend_period' })
    ? `${field.label}: ${field.min}~${field.max}${field.key === 'long_trend_period' ? ' 정수' : ' 유한한 숫자'}를 입력하세요. 0은 비활성입니다.` : null
}
export function maCrossEntryParamError(params: Record<string, unknown>): string | null {
  const error = fields.map((field) => fieldError(field, params[field.key])).find(Boolean)
  if (error) return error
  if (typeof params.min_long_ma_slope_pct === 'number' && params.min_long_ma_slope_pct > 0) {
    if (!isFiniteNumberInRange(params.long_period === undefined ? 20 : params.long_period, { min: 1, max: 511, integer: true })) return '장기 일봉 MA 기울기 활성 시 기존 장기 MA(long_period)는 1~511 정수여야 합니다.'
    if (params.short_period !== undefined && !isFiniteNumberInRange(params.short_period, { min: 0, max: Number.MAX_SAFE_INTEGER, integer: true })) return '장기 일봉 MA 기울기 활성 시 기존 단기 MA(short_period)는 정확히 표현 가능한 0 이상의 정수여야 합니다.'
    if ((params.short_period === undefined && params.long_period !== undefined && params.long_period !== 20) || (params.long_period === undefined && params.short_period !== undefined && params.short_period !== 5)) return '장기 일봉 MA 기울기 활성 시 기존 MA 한 필드가 누락되면 실제 기본 기간 5/20과 일치해야 합니다. 두 기간을 함께 입력하세요.'
  }
  return null
}
function MaCrossEntryField({ field, value, disabled, onChange }: {
  field: StrategyParamMeta; value: unknown; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const [raw, setRaw] = useState(() => value === undefined ? '0' : String(value))
  useEffect(() => { setRaw((previous) => Object.is(parseDecimalDraft(previous), value) ? previous : value === undefined ? '0' : String(value)) }, [value])
  const error = fieldError(field, value)
  return <TextField fullWidth size="small" label={field.label} value={raw} disabled={disabled} error={!!error}
    helperText={error ?? field.description} slotProps={{ htmlInput: { inputMode: field.key === 'long_trend_period' ? 'numeric' : 'decimal' } }}
    onChange={(event) => { setRaw(event.target.value); onChange(field.key, parseDecimalDraft(event.target.value)) }} />
}
export function MaCrossEntryControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="ma-cross-entry-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기존 골든크로스 매수 후보에만 선택 조건을 모두 적용합니다. 네 조건은 누락/0이면 비활성입니다. 기존 데드크로스 매도와 매 관측의 이전 MA 갱신을 유지하며, 거절한 교차를 나중에 다시 살리지 않습니다.
      기존 단기5·장기20 MA는 입력 관측 수입니다. 실시간은 약10초 가격관측, replay는 선택 봉 종가이므로 5/20일을 뜻하지 않습니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      {fields.map((field) => <Grid size={{ xs: 12, sm: 6 }} key={field.key}>
        <MaCrossEntryField field={field} value={params[field.key]} disabled={disabled} onChange={onChange} />
      </Grid>)}
    </Grid>
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      새 필터는 모두 엄격한 이전 완료 일봉 OHLC를 사용합니다. 현재 봉은 판단 후 추가하며 가격 tick이나 분봉으로 일봉을 합성하지 않습니다. 실시간은 별도로 공급한 완료 일봉 맥락을 사용하고, 활성 필터의 replay는 일봉만 지원합니다.
      추세는 후보 현재가 &gt; 이전 N일봉 평균, 기울기는 (이전 long_period 일봉 평균 / 그 한 일봉 전 평균 − 1) ×100의 계산값이 입력 임계값 이상인지 확인합니다. 기울기 활성 시 long_period 1~511과 이전 long_period+1봉이 필요합니다.
      ATR 비율은 이전 15일봉의 TR14 산술평균 / 이전 종가 ×100, 밴드폭은 이전 20종가의 4×모집단 표준편차 / 평균 ×100입니다.
      ATR·밴드폭 0도 자료가 충분하면 준비 완료지만 양수 임계값에서는 거부됩니다. 준비 부족은 신규 매수만 보류합니다. 공통 guard·실제 소유 수량 제한을 유지하며 필터 통과는 체결이 아닙니다.
    </Alert>
  </Stack>
}
export function MaCrossEntrySummary({ params }: { params: Record<string, unknown> }) {
  return <Typography variant="caption" sx={{ display: "block", overflowWrap: 'anywhere'  }}>
    MA 교차 진입 확인 · 기존 MA {String(params.short_period === undefined ? 5 : params.short_period)}/{String(params.long_period === undefined ? 20 : params.long_period)} 입력 관측
    {' · '}이전 일봉 추세 {String(params.long_trend_period === undefined ? 0 : params.long_trend_period)}봉
    {' · '}장기 일봉 MA 기울기 {String(params.min_long_ma_slope_pct === undefined ? 0 : params.min_long_ma_slope_pct)}%
    {' · '}ATR 비율 {String(params.min_atr_pct === undefined ? 0 : params.min_atr_pct)}%
    {' · '}밴드폭 {String(params.min_bandwidth_pct === undefined ? 0 : params.min_bandwidth_pct)}% · 새 옵션 0 비활성
  </Typography>
}
