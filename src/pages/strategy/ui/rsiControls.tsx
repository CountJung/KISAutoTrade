import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'

export function isRsiPolicyParam(key: string) {
  return key === 'long_trend_period' || key === 'stop_loss_pct' || key === 'max_hold_observations'
}
const fields = STRATEGY_PARAM_META.rsi.filter((field) => isRsiPolicyParam(field.key))
function fieldError(field: StrategyParamMeta, value: unknown): string | null {
  if (value === undefined) return null
  const integer = field.key !== 'stop_loss_pct'
  return !isFiniteNumberInRange(value, { min: field.min, max: field.max, integer })
    ? `${field.label}: ${field.min}~${field.max}${integer ? ' 정수' : ' 유한한 숫자'}를 입력하세요. 0은 비활성입니다.` : null
}
export function rsiPolicyError(params: Record<string, unknown>): string | null {
  return fields.map((field) => fieldError(field, params[field.key])).find(Boolean) ?? null
}
function RsiNumericField({ field, value, disabled, onChange }: {
  field: StrategyParamMeta; value: unknown; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const [raw, setRaw] = useState(() => value === undefined ? '0' : String(value))
  useEffect(() => { setRaw((previous) => Object.is(parseDecimalDraft(previous), value) ? previous : value === undefined ? '0' : String(value)) }, [value])
  const error = fieldError(field, value)
  return <TextField fullWidth size="small" label={field.label} value={raw} disabled={disabled} error={!!error}
    helperText={error ?? field.description} slotProps={{ htmlInput: { inputMode: field.key === 'stop_loss_pct' ? 'decimal' : 'numeric' } }}
    onChange={(event) => { setRaw(event.target.value); onChange(field.key, parseDecimalDraft(event.target.value)) }} />
}
export function RsiControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="rsi-policy-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기존 기간14·과매도30·과매수70은 유지합니다. 가격 변화의 단순평균 RSI이며 Wilder 방식이 아닙니다. 하락 평균이 0이면 횡보를 포함해 RSI 100으로 계산합니다.
      미보유에서 이전 RSI ≤ 과매도 → 현재 RSI &gt; 과매도 반등 교차만 매수 후보, 실제 보유에서 이전 RSI ≥ 과매수 → 현재 RSI &lt; 과매수 하락 교차만 잔량 전량 매도 후보입니다.
      신호만으로 보유를 바꾸지 않습니다. 실제 보유 수량·종목 통화 기준 평균 매수가로 보유를 확인하며 공통 주문 제한·소유 수량 제한을 유지합니다. 후보 통과·신호는 체결이 아닙니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      {fields.map((field) => <Grid size={{ xs: 12, sm: 6, md: 4 }} key={field.key}>
        <RsiNumericField field={field} value={params[field.key]} disabled={disabled} onChange={onChange} />
      </Grid>)}
    </Grid>
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      새 옵션은 모두 누락/0이면 비활성입니다. 추세는 신규 미보유 반등 교차에만 적용하며 현재 가격 &gt; 같은 주기의 직전 N가격 평균을 확인하고 현재 관측은 제외합니다.
      실시간 첫 관측은 추세용 사전 자료를 비우고 실제 N관측을 모아 다음 관측부터 확인합니다. 실시간 약10초 관측과 선택한 미리보기 봉 종가는 기간이 다르며 N관측은 N일이 아닙니다.
      보유 손절은 종목 통화 기준 실제 평균 매수가 대비 관측가 손실률 ≥ 입력률에서, 최대 보유는 실제 양수 체결 후 첫 다음 관측을 1로 세어 한도 이상에서 잔량 전량 매도 후보를 냅니다.
      손절 → 최대 보유 → 기존 RSI 하락 교차 순서이며 추세·RSI 준비 부족은 보유 보호를 막지 않습니다. 부분 매도·설정 변경으로 알려진 보유 관측 수를 초기화하지 않습니다.
      복원 보유의 나이가 미확인이면 활성 보유 상한은 다음 보유 관측에서 보수적 매도 후보를 냅니다. 나이를 0으로 가정하지 않습니다.
      손절·보유 한도는 공통 주문 제한·부분 체결 후 실제 잔량이 있고 조건이 지속되면 다시 후보를 냅니다. 원래 RSI 교차는 매 관측 기준을 갱신하며 거절한 교차를 조건 지속만으로 재시도하지 않습니다.
      손절은 종가 등 관측 후 다음 관측 시가 체결·가격 갭에 따라 입력 손실률을 넘을 수 있고 손실 상한을 보장하지 않습니다. 보유 관측 상한도 일수나 실제 보유시간 상한이 아닙니다.
      연구 후보 200관측·3%·63관측은 운영에 자동 적용하지 않습니다. 성과는 실제 모의 체결·완료 청산 표본과 함께 확인하세요.
    </Alert>
  </Stack>
}
function policyLabel(params: Record<string, unknown>, field: StrategyParamMeta, unit: string) {
  const value = params[field.key]
  if (value === undefined) return `0${unit} (누락 · 기본 비활성)`
  return fieldError(field, value) ? `확인 필요 (${String(value)})` : `${value}${unit}`
}
export function RsiPolicySummary({ params }: { params: Record<string, unknown> }) {
  const core = (key: string, fallback: number) => params[key] === undefined ? `${fallback} (기본)` : typeof params[key] === 'number' && Number.isFinite(params[key]) ? String(params[key]) : '확인 필요'
  return <Typography variant="caption" sx={{ display: "block", overflowWrap: 'anywhere'  }} data-testid="rsi-policy-summary">
    RSI 단순평균 · 기간 {core('period', 14)}관측 · 과매도 {core('oversold', 30)} / 과매수 {core('overbought', 70)}
    {' · '}진입 추세 {policyLabel(params, fields[0], '관측')}
    {' · '}보유 손절 {policyLabel(params, fields[1], '%')}
    {' · '}최대 보유 {policyLabel(params, fields[2], '관측')} · 새 옵션 0 비활성
    {' · '}설정 요약이며 이전 엔진의 보호 적용·실제 보유·체결을 증명하지 않습니다.
  </Typography>
}
