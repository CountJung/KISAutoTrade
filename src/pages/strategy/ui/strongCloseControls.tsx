import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'

const fields = STRATEGY_PARAM_META.strong_close.slice(2)
export function isStrongClosePolicyParam(key: string) { return fields.some((field) => field.key === key) }
function fieldError(field: StrategyParamMeta, value: unknown): string | null {
  if (value === undefined) return null
  const integer = field.key === 'long_trend_period' || field.key === 'max_hold_observations'
  return !isFiniteNumberInRange(value, { min: field.min, max: field.max, integer })
    ? `${field.label}: ${field.min}~${field.max}${integer ? ' 정수' : ' 유한한 숫자'}를 입력하세요. 0은 비활성입니다.` : null
}
function profitPairError(params: Record<string, unknown>) {
  const activation = params.profit_activation_pct === undefined ? 0 : params.profit_activation_pct
  const trailing = params.profit_trailing_pct === undefined ? 0 : params.profit_trailing_pct
  return (activation === 0 && trailing === 0) || (typeof activation === 'number' && activation > 0 && typeof trailing === 'number' && trailing > 0)
    ? null : '수익 보호 시작·되밀림은 둘 다 0(비활성)이거나 둘 다 양수여야 합니다.'
}
export function strongClosePolicyError(params: Record<string, unknown>): string | null {
  return fields.map((field) => fieldError(field, params[field.key])).find(Boolean) ?? profitPairError(params)
}
export function strongCloseTrendDisabled(params: Record<string, unknown>) {
  return !strongClosePolicyError(params) && (params.long_trend_period === undefined || params.long_trend_period === 0)
}
function StrongCloseNumericField({ field, params, disabled, onChange }: {
  field: StrategyParamMeta; params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const value = params[field.key]
  const [raw, setRaw] = useState(() => value === undefined ? '0' : String(value))
  useEffect(() => { setRaw((previous) => Object.is(parseDecimalDraft(previous), value) ? previous : value === undefined ? '0' : String(value)) }, [value])
  const error = fieldError(field, value) ?? (field.key.startsWith('profit_') ? profitPairError(params) : null)
  return <TextField fullWidth size="small" label={field.label} value={raw} disabled={disabled} error={!!error}
    helperText={error ?? field.description} inputProps={{ inputMode: field.key.startsWith('profit_') ? 'decimal' : 'numeric' }}
    onChange={(event) => { setRaw(event.target.value); onChange(field.key, parseDecimalDraft(event.target.value)) }} />
}
export function StrongCloseControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="strong-close-policy-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기존 기준3%·손절3%를 유지합니다. 이전 완료 일봉 (고가−종가)/고가×100 ≤ 기준이며 양봉·상승률 조건이 아닙니다. 고가=종가인 유효한 봉도 통과합니다.
      실시간은 검증된 전일 조건 뒤 첫 적격 가격관측에서 후보를 냅니다. 일봉 replay는 D-1 완료 고가·종가 조건 → D 종가 관측 신호 → D+1 다음 관측 시가 체결(nextOpen)이며 실시간과 주기가 다릅니다.
      실제 보유 수량·종목 통화 기준 평균 매수가를 확인하며 미보유에서만 매수합니다. 매도는 실제 잔량 전량 후보이고 신호는 체결이 아닙니다.
      원래 손절은 실제 평균가 대비 관측 손실률 ≥ 기준이며 손절0도 평균가 이하 청산입니다. 비활성으로 해석하지 않습니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      {fields.map((field) => <Grid item xs={12} sm={6} key={field.key}><StrongCloseNumericField field={field} params={params} disabled={disabled} onChange={onChange} /></Grid>)}
    </Grid>
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      새 네 옵션만 누락/0 비활성입니다. 추세는 현재 후보 가격 &gt; 이전 N완료 일봉 종가의 산술평균이며 현재 관측을 제외합니다. 원래 전일 조건도 함께 필요하고 자료 부족은 신규 매수만 보류합니다.
      기존 일봉 자료 상태에서 갱신·실패를 확인하세요. 실시간 전일 자료가 무효·오래됨·미검증이면 매수를 보류하며 보유 보호는 계속합니다. 같은 날 자료 갱신이 이미 소비한 조건을 다시 만들지는 않습니다.
      수익 보호는 실제 평균 매수가 대비 시작 이익률에서 한 번 무장한 뒤 실제 보유 중 관측 최고가의 되밀림에서 잔량 전량 매도 후보를 냅니다. 진입 전 고가·현재 봉 전체 고가를 쓰지 않습니다.
      기존 손절 → 수익 보호 → 보유 한도 순입니다. 주문 거절·부분 체결·설정 변경으로 알려진 최고가·무장·보유 관측 수를 초기화하지 않으며 조건이 지속되면 실제 잔량으로 다시 후보를 냅니다.
      보유 관측 수는 실제 첫 양수 체결 시 0, 이후 첫 보유 관측을 1로 셉니다. 복원 보유의 나이가 미확인이면 활성 보유 한도에서 다음 보유 관측에 보수적 매도 후보를 냅니다.
      관측 상한은 일수나 실제 보유시간 보장이 아닙니다. 공통 주문 제한·다음 관측 시가 체결·가격 갭으로 손실률이나 실제 보유시간을 보장하지 않습니다. 이 전략의 generic 미리보기는 일봉만 지원합니다.
      연구 후보를 운영에 자동 적용하지 않습니다. 설정 변경·자료 갱신이 실제 소유·체결을 대신하지 않습니다.
    </Alert>
  </Stack>
}
function numericLabel(params: Record<string, unknown>, field: StrategyParamMeta, unit: string) {
  const value = params[field.key]
  return value === undefined ? `0${unit} (누락 · 기본 비활성)` : fieldError(field, value) ? `확인 필요 (${String(value)})` : `${value}${unit}`
}
export function StrongClosePolicySummary({ params }: { params: Record<string, unknown> }) {
  const core = (key: string) => params[key] === undefined ? '3 (기본)' : typeof params[key] === 'number' && Number.isFinite(params[key]) ? String(params[key]) : '확인 필요'
  return <Typography variant="caption" display="block" sx={{ overflowWrap: 'anywhere' }} data-testid="strong-close-policy-summary">
    강한 종가 (이전 고가−종가)/고가×100 ≤ {core('threshold_pct')}% · 손절 {core('stop_loss_pct')}% (0도 평균가 이하 청산)
    {' · '}이전 완료 일봉 추세 {numericLabel(params, fields[0], '봉')}
    {' · '}수익 보호 시작 {numericLabel(params, fields[1], '%')} / 되밀림 {numericLabel(params, fields[2], '%')}{profitPairError(params) ? ' (조합 확인 필요)' : ''}
    {' · '}최대 보유 {numericLabel(params, fields[3], '관측')} · 새 네 옵션 0 비활성
    {' · '}D-1 조건 → D 종가 신호 → D+1 시가 체결(nextOpen). 설정 요약이며 이전 엔진의 새 보호 적용·실제 보유·체결을 증명하지 않습니다.
  </Typography>
}
