import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'

export function isMeanReversionPolicyParam(key: string) {
  return key === 'exit_mode' || key === 'long_trend_period' || key === 'max_hold_observations'
}
const fields = STRATEGY_PARAM_META.mean_reversion.filter((field) => isMeanReversionPolicyParam(field.key))
const modeOf = (params: Record<string, unknown>) => params.exit_mode === undefined ? 'upperBand' : params.exit_mode
function fieldError(field: StrategyParamMeta, value: unknown): string | null {
  if (value === undefined) return null
  return !isFiniteNumberInRange(value, { min: field.min, max: field.max, integer: true })
    ? `${field.label}: ${field.min}~${field.max} 정수를 입력하세요. 0은 비활성입니다.` : null
}
export function meanReversionPolicyError(params: Record<string, unknown>): string | null {
  const mode = modeOf(params)
  if (mode !== 'upperBand' && mode !== 'middleBand') return 'MeanReversion 청산 모드는 upperBand 또는 middleBand만 허용합니다. 저장된 잘못된 값을 수정하세요.'
  return fields.map((field) => fieldError(field, params[field.key])).find(Boolean) ?? null
}
function MeanReversionNumericField({ field, value, disabled, onChange }: {
  field: StrategyParamMeta; value: unknown; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const [raw, setRaw] = useState(() => value === undefined ? '0' : String(value))
  useEffect(() => { setRaw((previous) => Object.is(parseDecimalDraft(previous), value) ? previous : value === undefined ? '0' : String(value)) }, [value])
  const error = fieldError(field, value)
  return <TextField fullWidth size="small" label={field.label} value={raw} disabled={disabled} error={!!error}
    helperText={error ?? field.description} slotProps={{ htmlInput: { inputMode: 'numeric' } }}
    onChange={(event) => { setRaw(event.target.value); onChange(field.key, parseDecimalDraft(event.target.value)) }} />
}
export function MeanReversionControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const mode = modeOf(params), validMode = mode === 'upperBand' || mode === 'middleBand'
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="mean-reversion-policy-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기본 기간20·표준편차2·손절5를 유지합니다. 현재 관측을 포함한 볼린저 평균·모집단 분산으로 밴드를 계산하고 가격이 하단 미만일 때만 매수합니다.
      upperBand는 현재 가격 &gt; 같은 현재 포함 상단, middleBand는 현재 가격 &gt; 같은 현재 포함 평균에서 보유 청산합니다. 경계와 같으면 밴드 매매 조건이 아닙니다.
      보유는 실제 양수 수량·native 평균가로 확인합니다. 신호만으로 보유를 바꾸지 않으며 차단·pending·부분 체결 뒤에도 실제 잔량의 청산을 계속 평가합니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      <Grid size={{ xs: 12, sm: 6, md: 4 }}>
        <TextField fullWidth select size="small" label="MeanReversion 청산 모드" value={validMode ? mode : 'invalid'} disabled={disabled}
          error={!validMode} helperText={!validMode ? meanReversionPolicyError(params) : '기본 상단 초과 또는 선택 중심선 초과에서 청산합니다. 같은 값은 제외합니다.'}
          onChange={(event) => onChange('exit_mode', event.target.value)}>
          {!validMode && <MenuItem value="invalid" disabled>저장된 잘못된 모드: {String(mode)}</MenuItem>}
          <MenuItem value="upperBand">upperBand · 상단 초과 (기본)</MenuItem><MenuItem value="middleBand">middleBand · 중심선 초과</MenuItem>
        </TextField>
      </Grid>
      {fields.map((field) => <Grid size={{ xs: 12, sm: 6, md: 4 }} key={field.key}>
        <MeanReversionNumericField field={field} value={params[field.key]} disabled={disabled} onChange={onChange} />
      </Grid>)}
    </Grid>
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      새 추세·최대 보유 옵션만 0이면 비활성입니다. 추세는 신규 진입에만 적용하며 현재 가격 &gt; 같은 주기의 직전 N가격 MA를 확인하고 현재 관측은 제외합니다.
      실시간은 시작 후 전용 추세 버퍼를 비우고 실제 가격 N관측을 모아 다음 관측부터 확인합니다. 일봉/1분봉 warmup과 섞지 않습니다. 일봉 replay는 엄격한 이전 봉으로 준비하며, 기존 1분 미리보기는 사전 분봉으로 밴드·추세를 초기화하지 않고 평가 관측부터 준비합니다.
      실시간 약10초 관측과 선택한 replay 봉 종가는 기간이 다릅니다. 최대 보유 63은 일봉 연구의 63봉이며 실시간 63일이 아닙니다.
      보유 관측은 옵션이 꺼져 있어도 실제 양수 체결 후 기록하며 같은 수량·부분 매도·warmup·재설정으로 지우지 않습니다. 복원 보유의 나이가 미확인이면 활성 보유 상한은 다음 관측에서 보수적 매도 후보를 냅니다.
      기존 손절 0은 비활성이 아니라 원가 이하 손절입니다. 손절 → 최대 보유 → 선택 밴드 순으로 평가하며 손절·밴드 매도는 설정 수량, 최대 보유만 실제 잔량 전량을 후보로 냅니다.
      공통 guard/risk를 유지하며 다음 시가 체결·갭으로 실제 모의 손실이 입력률을 넘을 수 있습니다. 관측가 손절은 장중 손실 상한을 보장하지 않습니다.
    </Alert>
  </Stack>
}
export function MeanReversionPolicySummary({ params }: { params: Record<string, unknown> }) {
  const mode = modeOf(params)
  return <Typography variant="caption" sx={{ display: "block", overflowWrap: 'anywhere'  }}>
    MeanReversion 청산 · {mode === 'upperBand' || mode === 'middleBand' ? mode : '모드 확인 필요'}{params.exit_mode === undefined ? ' (기본 upperBand)' : ''}
    {' · '}추세 MA {String(params.long_trend_period === undefined ? 0 : params.long_trend_period)}관측
    {' · '}최대 보유 {String(params.max_hold_observations === undefined ? 0 : params.max_hold_observations)}관측 · 새 숫자 옵션 0 비활성
    {' · '}기존 손절 {String(params.stop_loss_pct === undefined ? 5 : params.stop_loss_pct)}% (0도 원가 이하 손절)
  </Typography>
}
