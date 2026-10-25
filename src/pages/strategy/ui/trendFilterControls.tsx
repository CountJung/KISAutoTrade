import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'

const fields = STRATEGY_PARAM_META.trend_filter
const defaults: Record<string, number> = { long_period: 200, short_period: 5, mid_period: 20, exit_confirm_observations: 1 }
export function isTrendFilterParam(key: string) { return fields.some((field) => field.key === key) }
function fieldError(field: StrategyParamMeta, value: unknown): string | null {
  if (value === undefined) return null
  const integer = !field.key.startsWith('profit_')
  return isFiniteNumberInRange(value, { min: field.min, max: field.max, integer }) ? null
    : `${field.label}: ${field.min}~${field.max}${integer ? ' 정수' : ' 유한한 숫자'}를 입력하세요.`
}
function pairError(params: Record<string, unknown>) {
  const a = params.profit_activation_pct === undefined ? 0 : params.profit_activation_pct
  const t = params.profit_trailing_pct === undefined ? 0 : params.profit_trailing_pct
  return (a === 0 && t === 0) || (typeof a === 'number' && a > 0 && typeof t === 'number' && t > 0)
    ? null : '수익 보호 시작·되밀림은 둘 다 0(비활성)이거나 둘 다 양수여야 합니다.'
}
export function trendFilterParamError(params: Record<string, unknown>): string | null {
  return fields.map((field) => fieldError(field, params[field.key])).find(Boolean) ?? pairError(params)
}
function NumericField({ field, params, disabled, onChange }: Props & { field: StrategyParamMeta }) {
  const value = params[field.key]
  const fallback = String(defaults[field.key] ?? 0)
  const [raw, setRaw] = useState(() => value === undefined ? fallback : String(value))
  useEffect(() => { setRaw((old) => Object.is(parseDecimalDraft(old), value) ? old : value === undefined ? fallback : String(value)) }, [value, fallback])
  const error = fieldError(field, value) ?? (field.key.startsWith('profit_') ? pairError(params) : null)
  return <TextField fullWidth size="small" label={field.label} value={raw} disabled={disabled} error={!!error}
    helperText={error ?? field.description} inputProps={{ inputMode: field.key.startsWith('profit_') ? 'decimal' : 'numeric' }}
    onChange={(e) => { setRaw(e.target.value); onChange(field.key, parseDecimalDraft(e.target.value)) }} />
}
type Props = { params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void }
export function TrendFilterControls(props: Props) {
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="trend-filter-policy-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기본 MA 200/5/20과 이탈 확인 1관측을 유지합니다. 산술평균에 현재 관측을 포함하며 현재가 &gt; 장기 MA, 단기 MA &gt; 중기 MA일 때 실제 미보유에서만 매수합니다. 등호는 진입하지 않습니다.
      원래 청산은 세 MA가 모두 준비된 뒤 현재가 &lt; 장기 MA인 연속 관측입니다. 현재가 ≥ 장기 MA 또는 준비 부족이면 확인값은 0입니다. 단기 MA 하락만으로 청산하지 않으며 별도 원가 손절은 없습니다.
      실시간 약10초 quote와 선택 일봉/분봉의 관측 주기가 다릅니다. MA 200관측은 완료 일봉 200일을 보장하지 않습니다. 기간 순서는 강제하지 않습니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>{fields.map((field) => <Grid item xs={12} sm={6} key={field.key}><NumericField field={field} {...props} /></Grid>)}</Grid>
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      이탈 확인은 최소 1이며 0으로 끌 수 없습니다. 수익 보호 쌍·보유 한도만 누락/0 비활성입니다. 실제 native 평균 매수가 대비 시작 %에서 한 번 무장하고 실제 보유 중 관측 최고가 대비 되밀림에서 청산 후보를 냅니다. 진입 전 가격·현재 봉 전체 고가는 사용하지 않습니다.
      실제 첫 체결의 보유 나이는 0, 다음 보유 관측은 1입니다. 옵션이 꺼져 있어도 세며 나이 불명 복원은 활성 한도에서 다음 보유 관측에 보수적으로 청산 후보를 냅니다. 선택 수익·보유 보호는 MA 준비 부족에도 평가합니다.
      장기 이탈 확인 → 수익 보호 → 보유 한도 순서입니다. 매도는 실제 잔량 전량 후보이며 거절·부분 체결·동량 갱신으로 보호를 지우지 않습니다. 재시작·과거 지표 재적재·핵심 MA 기간 변경은 이탈 연속값만 0부터 다시 관측합니다.
      관측 한도는 날짜나 실제 보유시간 보장이 아닙니다. 신호는 체결이 아니며 공통 주문 제한·다음 관측 시가 체결(nextOpen)·가격 갭으로 손실이나 체결 시점을 보장하지 않습니다. 연구 후보를 운영에 자동 적용하지 않습니다.
    </Alert>
  </Stack>
}
export function TrendFilterSummary({ params, engineVersion }: { params: Record<string, unknown>; engineVersion?: string }) {
  const label = (field: StrategyParamMeta) => {
    const v = params[field.key]
    return v === undefined ? `${defaults[field.key] ?? 0} (기본)` : fieldError(field, v) ? `확인 필요 (${String(v)})` : String(v)
  }
  return <Stack spacing={0.5} data-testid="trend-filter-policy-summary">
    <Typography variant="caption" display="block" sx={{ overflowWrap: 'anywhere' }}>
      추세 필터 현재 포함 MA 장기/단기/중기 {fields.slice(0, 3).map(label).join('/')}관측 · 이탈 확인 {label(fields[3])}연속 관측
      {' · '}수익 보호 시작 {label(fields[4])}% / 되밀림 {label(fields[5])}%{pairError(params) ? ' (조합 확인 필요)' : ''} · 최대 보유 {label(fields[6])}관측 (수익·보유만 0 비활성)
      {' · '}설정 요약이며 실제 소유·무장·체결을 증명하지 않습니다. 일봉 종가 신호 → 다음 관측 시가(nextOpen), 실시간 약10초 관측과 기간이 다릅니다.
    </Typography>
    {(!engineVersion || !/^strategy-replay-v(2[2-9]|[3-9]\d|[1-9]\d{2,})$/.test(engineVersion)) && <Alert severity="warning" sx={{ py: 0.25 }}>이전 또는 미확인 엔진 결과입니다. 실제 소유·선택 보호 적용 여부는 미확인이며 다시 계산해야 합니다.</Alert>}
  </Stack>
}
