import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'

const fields = STRATEGY_PARAM_META.failed_breakout.slice(2)
const basisOf = (params: Record<string, unknown>) => params.breakout_basis === undefined ? 'priceObservations' : params.breakout_basis
const validBasis = (value: unknown) => value === 'priceObservations' || value === 'completedHigh'
export function isFailedBreakoutPolicyParam(key: string) { return key === 'breakout_basis' || fields.some((field) => field.key === key) }
export function failedBreakoutDailyFiltersDisabled(params: Record<string, unknown>) {
  return basisOf(params) === 'priceObservations' && ['atr_buffer_multiplier', 'min_volume_ratio'].every((key) => params[key] === undefined || params[key] === 0)
}
function fieldError(field: StrategyParamMeta, value: unknown): string | null {
  if (value === undefined) return null
  const integer = field.key === 'max_hold_observations'
  return !isFiniteNumberInRange(value, { min: field.min, max: field.max, integer })
    ? `${field.label}: ${field.min}~${field.max}${integer ? ' 정수' : ' 유한한 숫자'}를 입력하세요. 0은 비활성입니다.` : null
}
function profitPairError(params: Record<string, unknown>) {
  const activation = params.profit_activation_pct === undefined ? 0 : params.profit_activation_pct
  const trailing = params.profit_trailing_pct === undefined ? 0 : params.profit_trailing_pct
  return (activation === 0 && trailing === 0) || (typeof activation === 'number' && activation > 0 && typeof trailing === 'number' && trailing > 0)
    ? null : '수익 보호 시작·되밀림은 둘 다 0(비활성)이거나 둘 다 양수여야 합니다.'
}
export function failedBreakoutPolicyError(params: Record<string, unknown>): string | null {
  if (!validBasis(basisOf(params))) return '돌파 기준은 priceObservations 또는 completedHigh만 허용합니다. 저장된 잘못된 값을 수정하세요.'
  return fields.map((field) => fieldError(field, params[field.key])).find(Boolean) ?? profitPairError(params)
}
function BreakoutNumericField({ field, params, disabled, onChange }: {
  field: StrategyParamMeta; params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const value = params[field.key]
  const [raw, setRaw] = useState(() => value === undefined ? '0' : String(value))
  useEffect(() => { setRaw((previous) => Object.is(parseDecimalDraft(previous), value) ? previous : value === undefined ? '0' : String(value)) }, [value])
  const error = fieldError(field, value) ?? (field.key.startsWith('profit_') ? profitPairError(params) : null)
  return <TextField fullWidth size="small" label={field.label} value={raw} disabled={disabled} error={!!error}
    helperText={error ?? field.description} inputProps={{ inputMode: field.key === 'max_hold_observations' ? 'numeric' : 'decimal' }}
    onChange={(event) => { setRaw(event.target.value); onChange(field.key, parseDecimalDraft(event.target.value)) }} />
}
export function FailedBreakoutControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const basis = basisOf(params), valid = validBasis(basis)
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="failed-breakout-policy-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기존 기간20·버퍼0.5%를 유지합니다. 기본은 이전 N가격관측의 최대값이며 실시간 가격과 미리보기 봉 종가는 주기가 다릅니다. N관측은 N일이 아닙니다.
      선택 완료 고가는 이전 N완료 일봉 고가의 최대값입니다. 현재 가격으로 돌파를 판단하며 현재 봉 고가를 돌파 신호로 쓰지 않습니다.
      실제 보유 수량·종목 통화 기준 평균 매수가를 확인하며 미보유에서만 매수합니다. 실패 기준은 버퍼 전 진입 기준 고점 미만입니다.
      실제 청산 전에는 주문 거절·부분 체결·설정 변경으로 이 기준을 지우지 않습니다. 신호는 체결이 아니며 공통 주문 제한·실제 잔량 제한을 유지합니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      <Grid item xs={12} sm={6}>
        <TextField fullWidth select size="small" label="돌파 기준" value={valid ? basis : 'invalid'} disabled={disabled} error={!valid}
          helperText={!valid ? failedBreakoutPolicyError(params) : '기본 이전 가격관측 최대값 또는 선택 이전 완료 일봉 고가'} onChange={(event) => onChange('breakout_basis', event.target.value)}>
          {!valid && <MenuItem value="invalid" disabled>저장된 잘못된 기준: {String(basis)}</MenuItem>}
          <MenuItem value="priceObservations">이전 가격관측 최대값 (기본)</MenuItem><MenuItem value="completedHigh">이전 완료 일봉 고가</MenuItem>
        </TextField>
      </Grid>
      {fields.map((field) => <Grid item xs={12} sm={6} key={field.key}><BreakoutNumericField field={field} params={params} disabled={disabled} onChange={onChange} /></Grid>)}
    </Grid>
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      새 숫자 옵션은 누락/0 비활성입니다. ATR 버퍼 문턱은 기존 비율 문턱(정수 내림)과 기준 고점 + 이전 완료 15일봉의 TR14 산술평균×배수 중 큰 가격입니다. 추가 ATR 문턱은 정수가격에서 올림 효과로 비교합니다. ATR 0도 자료가 충분하면 준비 완료입니다.
      거래량 비율은 직전 완료 일봉량 / 그 이전 20일 평균으로 총 21개 이전 봉이 필요합니다. 현재 봉 거래량·실시간 누적량을 사용하지 않으며 미확인 값, 검증된 0, 기준 평균 0을 구분합니다.
      완료 고가·ATR·거래량을 켠 미리보기는 일봉만 지원하며 실시간 저장은 지원합니다. 수익 보호·보유 한도만 켠 분봉 미리보기는 허용합니다.
      수익 보호는 실제 평균 매수가 대비 시작 이익률에서 한 번 무장한 뒤 실제 보유 중 관측 최고가의 되밀림에서 잔량 전량 매도 후보를 냅니다. 진입 전 고가·현재 봉 전체 고가를 쓰지 않습니다.
      실패 → 수익 보호 → 보유 한도 순이며 알려진 관측 최고가·무장·나이는 부분 체결·설정 변경으로 초기화하지 않습니다. 보유 관측 수는 실제 양수 체결 후 다음 관측을 1로 셉니다.
      이전 v1 복원 보유의 미확인 나이·최고가·무장 정보를 꾸미지 않습니다. 활성 보유 한도에서 나이가 미확인이면 다음 보유 관측에 보수적 매도 후보를 냅니다.
      주문 제한·다음 시가 체결·가격 갭으로 손익이나 실제 보유시간을 보장하지 않습니다. 연구 후보를 운영에 자동 적용하지 않습니다.
    </Alert>
  </Stack>
}
function numericLabel(params: Record<string, unknown>, field: StrategyParamMeta, unit: string) {
  const value = params[field.key]
  return value === undefined ? `0${unit} (누락 · 기본 비활성)` : fieldError(field, value) ? `확인 필요 (${String(value)})` : `${value}${unit}`
}
export function FailedBreakoutPolicySummary({ params }: { params: Record<string, unknown> }) {
  const basis = basisOf(params)
  const core = (key: string, fallback: number) => params[key] === undefined ? `${fallback} (기본)` : typeof params[key] === 'number' && Number.isFinite(params[key]) ? String(params[key]) : '확인 필요'
  return <Typography variant="caption" display="block" sx={{ overflowWrap: 'anywhere' }} data-testid="failed-breakout-policy-summary">
    돌파 기준 {basis === 'priceObservations' ? '이전 가격관측 최대값' : basis === 'completedHigh' ? '이전 완료 일봉 고가' : `확인 필요 (${String(basis)})`}{params.breakout_basis === undefined ? ' (누락 · 기본)' : ''}
    {' · '}기간 {core('lookback_days', 20)}관측/봉 · 버퍼 {core('buffer_pct', 0.5)}%
    {' · '}ATR 버퍼 {numericLabel(params, fields[0], '배')} · 거래량 {numericLabel(params, fields[1], '배')}
    {' · '}수익 보호 시작 {numericLabel(params, fields[2], '%')} / 되밀림 {numericLabel(params, fields[3], '%')}{profitPairError(params) ? ' (조합 확인 필요)' : ''}
    {' · '}최대 보유 {numericLabel(params, fields[4], '관측')} · 새 숫자 옵션 0 비활성
    {' · '}설정 요약이며 이전 엔진·v1 복원에서 새 보호 적용이나 실제 보유·체결을 확인한 것은 아닙니다.
  </Typography>
}
