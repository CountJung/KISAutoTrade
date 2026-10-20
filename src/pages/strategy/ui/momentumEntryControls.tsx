import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { isFiniteNumberInRange, parseDecimalDraft } from '../../../shared/lib'

export function isMomentumEntryParam(key: string) { return key === 'entry_mode' || key === 'entry_hysteresis_pct' }
const modeOf = (params: Record<string, unknown>) => params.entry_mode === undefined ? 'level' : params.entry_mode
const hysteresisOf = (params: Record<string, unknown>) => params.entry_hysteresis_pct === undefined ? 0 : params.entry_hysteresis_pct
const thresholdOf = (params: Record<string, unknown>) => params.threshold_pct === undefined ? 5 : params.threshold_pct
function hysteresisError(value: unknown) {
  return !isFiniteNumberInRange(value, { min: 0, max: 100 }) ? '재무장 간격은 0~100의 유한한 숫자(%p)로 입력하세요.' : null
}
export function momentumEntryParamError(params: Record<string, unknown>): string | null {
  const mode = modeOf(params), hysteresis = hysteresisOf(params), threshold = thresholdOf(params)
  if (mode !== 'level' && mode !== 'crossing') return '진입 모드는 level 또는 crossing만 허용합니다. 저장된 잘못된 값을 수정하세요.'
  const error = hysteresisError(hysteresis)
  if (error) return error
  if (mode === 'crossing' && (typeof threshold !== 'number' || !Number.isFinite(threshold) || (hysteresis as number) > threshold)) return 'crossing의 재무장 간격은 모멘텀 임계값(%) 이하여야 합니다.'
  return null
}
function parsedDraft(raw: string): number | string {
  const numeric = parseDecimalDraft(raw)
  return typeof numeric === 'number' && !hysteresisError(numeric) ? numeric : raw
}
export function MomentumEntryControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const value = params.entry_hysteresis_pct, mode = modeOf(params)
  const [raw, setRaw] = useState(() => value === undefined ? '0' : String(value))
  useEffect(() => { setRaw((previous) => Object.is(parsedDraft(previous), value) ? previous : value === undefined ? '0' : String(value)) }, [value])
  const error = momentumEntryParamError(params)
  const validMode = mode === 'level' || mode === 'crossing'
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="momentum-entry-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      기본 level은 모멘텀이 양의 임계값 이상인 상태에서 진입 후보를 냅니다. crossing은 이전 모멘텀 &lt; 임계값, 현재 모멘텀 ≥ 임계값인 상향 돌파를 확인합니다.
      모멘텀 ≤ 임계값−재무장 간격에서 다시 준비합니다. 예: 임계값 5%, 간격 1%p이면 4% 이하에서 재무장합니다.
      실제 보유의 음의 임계값 이하 청산은 두 모드 모두 유지합니다. 차단·대기 신호는 체결과 구분하고 실제 보유 소유 피드백을 따릅니다.
      미체결 후보는 조건 유지 중 재시도할 수 있고 하회하면 취소합니다. 실시간은 약10초 가격관측, replay는 선택 봉 종가입니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      <Grid item xs={12} sm={6} md={4}>
        <TextField fullWidth select size="small" label="Momentum 진입 모드" value={validMode ? mode : 'invalid'} disabled={disabled}
          error={!validMode} helperText={!validMode ? error : 'level은 기본 동작, crossing은 상향 돌파와 재무장 조건을 사용합니다.'}
          onChange={(event) => onChange('entry_mode', event.target.value)}>
          {!validMode && <MenuItem value="invalid" disabled>저장된 잘못된 모드: {String(mode)}</MenuItem>}
          <MenuItem value="level">level · 임계값 상태 (기본)</MenuItem>
          <MenuItem value="crossing">crossing · 상향 돌파</MenuItem>
        </TextField>
      </Grid>
      <Grid item xs={12} sm={6} md={4}>
        <TextField fullWidth size="small" label="재무장 간격 (%p)" value={raw} disabled={disabled} error={!!error}
          helperText={error ?? (mode === 'level' ? 'level에서는 미사용입니다. 입력값은 보존하며 0~100 범위를 검증합니다.' : 'crossing에서 모멘텀 ≤ 임계값−간격이면 재무장합니다. 간격은 임계값 이하여야 합니다.')}
          inputProps={{ inputMode: 'decimal' }} onChange={(event) => { setRaw(event.target.value); onChange('entry_hysteresis_pct', parsedDraft(event.target.value)) }} />
      </Grid>
    </Grid>
  </Stack>
}
export function MomentumEntrySummary({ params }: { params: Record<string, unknown> }) {
  const mode = modeOf(params), hysteresis = hysteresisOf(params), error = momentumEntryParamError(params)
  return <Typography variant="caption" display="block" sx={{ overflowWrap: 'anywhere' }}>
    Momentum 진입 모드 · {error ? '설정 확인 필요' : String(mode)}{params.entry_mode === undefined ? ' (기본 level)' : ''}
    {' · '}재무장 간격 {String(hysteresis)}%p{mode === 'level' ? ' · level에서 미사용' : ''}
  </Typography>
}
