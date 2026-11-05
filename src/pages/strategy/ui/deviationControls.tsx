import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'

export function isDeviationPolicyParam(key: string) {
  return key === 'entry_mode' || key === 'long_trend_period' || key === 'max_hold_observations' || key === 'stop_loss_pct'
}
const fields = STRATEGY_PARAM_META.deviation.filter((field) => isDeviationPolicyParam(field.key))
const modeOf = (params: Record<string, unknown>) => params.entry_mode === undefined ? 'level' : params.entry_mode
function fieldError(field: StrategyParamMeta, value: unknown): string | null {
  if (value === undefined) return null
  const integer = field.key !== 'stop_loss_pct'
  return !isFiniteNumberInRange(value, { min: field.min, max: field.max, integer })
    ? `${field.label}: ${field.min}~${field.max}${integer ? ' 정수' : ' 유한한 숫자'}를 입력하세요. 0은 비활성입니다.` : null
}
export function deviationPolicyError(params: Record<string, unknown>): string | null {
  const mode = modeOf(params)
  if (mode !== 'level' && mode !== 'crossing') return 'Deviation 진입 모드는 level 또는 crossing만 허용합니다. 저장된 잘못된 값을 수정하세요.'
  return fields.map((field) => fieldError(field, params[field.key])).find(Boolean) ?? null
}
function DeviationNumericField({ field, value, disabled, onChange }: {
  field: StrategyParamMeta; value: unknown; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const [raw, setRaw] = useState(() => value === undefined ? '0' : String(value))
  useEffect(() => { setRaw((previous) => Object.is(parseDecimalDraft(previous), value) ? previous : value === undefined ? '0' : String(value)) }, [value])
  const error = fieldError(field, value)
  return <TextField fullWidth size="small" label={field.label} value={raw} disabled={disabled} error={!!error}
    helperText={error ?? field.description} slotProps={{ htmlInput: { inputMode: field.key === 'stop_loss_pct' ? 'decimal' : 'numeric' } }}
    onChange={(event) => { setRaw(event.target.value); onChange(field.key, parseDecimalDraft(event.target.value)) }} />
}
export function DeviationControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const mode = modeOf(params), validMode = mode === 'level' || mode === 'crossing'
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="deviation-policy-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기본 level · MA20/매수 -5%/매도 +5%를 유지하며 추가 숫자 옵션은 모두 0이면 비활성입니다. 미보유 매수·실제 보유 매도만 평가합니다.
      crossing은 이전 이격도 &gt; 매수 기준, 현재 이격도 ≤ 매수 기준의 하향 교차입니다. 미체결 후보는 ≤ 조건 중 재시도하고 기준을 초과하면 취소합니다. 차단·pending은 체결과 구분합니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      <Grid size={{ xs: 12, sm: 6, md: 4 }}>
        <TextField fullWidth select size="small" label="Deviation 진입 모드" value={validMode ? mode : 'invalid'} disabled={disabled}
          error={!validMode} helperText={!validMode ? deviationPolicyError(params) : 'level은 기본 상태 조건, crossing은 하향 교차 후보를 사용합니다.'}
          onChange={(event) => onChange('entry_mode', event.target.value)}>
          {!validMode && <MenuItem value="invalid" disabled>저장된 잘못된 모드: {String(mode)}</MenuItem>}
          <MenuItem value="level">level · 이격 상태 (기본)</MenuItem><MenuItem value="crossing">crossing · 하향 교차</MenuItem>
        </TextField>
      </Grid>
      {fields.map((field) => <Grid size={{ xs: 12, sm: 6, md: 4 }} key={field.key}>
        <DeviationNumericField field={field} value={params[field.key]} disabled={disabled} onChange={onChange} />
      </Grid>)}
    </Grid>
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      추세 진입은 현재 가격이 같은 주기의 직전 N가격 MA보다 높을 때만 허용하며 현재 관측은 제외합니다. 실시간은 시작 후 실제 가격 N관측을 모아 다음 관측부터 확인하고, 연구는 엄격한 이전 봉으로 준비합니다. 일봉/1분봉 warmup을 실시간 관측과 섞지 않습니다.
      실시간 약10초 관측과 선택한 replay 봉 종가는 기간이 다릅니다. 최대 보유 63은 일봉 연구에서 63봉이고 실시간 63일을 뜻하지 않습니다.
      보유 관측은 옵션이 꺼져 있어도 실제 양수 보유 이후 누적하며, 같은 수량·부분 매도·warmup·재설정·중간 활성화로 지우지 않습니다. 복원 보유의 age 정보가 없고 최대 보유가 켜져 있으면 보수적 매도 후보를 냅니다.
      손절은 실제 native 평균가와 현재 관측가로 판단합니다. 다음 시가 체결·갭에 따라 모의 청산 손실이 입력률을 넘을 수 있습니다.
      매도 우선순위는 손절 → 최대 보유 → 고이격입니다. 손절/최대 보유는 실제 소유 잔량 전부를 후보로 내며 공통 guard/risk를 적용하고 부분 체결 뒤 잔량은 다시 평가합니다.
    </Alert>
  </Stack>
}
export function DeviationPolicySummary({ params }: { params: Record<string, unknown> }) {
  const mode = modeOf(params)
  return <Typography variant="caption" sx={{ display: "block", overflowWrap: 'anywhere'  }}>
    Deviation 진입 · {mode === 'level' || mode === 'crossing' ? mode : '모드 확인 필요'}{params.entry_mode === undefined ? ' (기본 level)' : ''}
    {' · '}추세 MA {String(params.long_trend_period === undefined ? 0 : params.long_trend_period)}관측
    {' · '}최대 보유 {String(params.max_hold_observations === undefined ? 0 : params.max_hold_observations)}관측
    {' · '}손절 {String(params.stop_loss_pct === undefined ? 0 : params.stop_loss_pct)}% · 숫자 0 비활성
  </Typography>
}
