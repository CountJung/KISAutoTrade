import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Grid from '@mui/material/Grid'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { STRATEGY_PARAM_META, type StrategyParamMeta } from './strategyMetadata'
import { parseDecimalDraft } from '../../../shared/lib'

const fields = STRATEGY_PARAM_META.consecutive_move.slice(2)
export function consecutiveEntryFiltersDisabled(params: Record<string, unknown>) { return fields.every(({ key }) => params[key] === undefined || params[key] === 0) }
export function isConsecutiveEntryParam(key: string) { return fields.some((field) => field.key === key) }

function fieldError(key: string, value: unknown) {
  const field = fields.find((item) => item.key === key)
  if (!field || value === undefined) return null
  if (typeof value !== 'number' || !Number.isFinite(value) || value < field.min || value > field.max || (key === 'long_trend_period' && !Number.isInteger(value))) {
    return `${field.label}: ${field.min}~${field.max}${key === 'long_trend_period' ? ' 정수' : ' 유한한 숫자'}를 입력하세요. 0은 비활성입니다.`
  }
  return null
}
export function consecutiveEntryParamError(params: Record<string, unknown>) {
  return fields.map((field) => fieldError(field.key, params[field.key])).find(Boolean) ?? null
}
function parsedDraft(key: string, raw: string): number | string {
  const numeric = parseDecimalDraft(raw)
  return typeof numeric === 'number' && !fieldError(key, numeric) ? numeric : raw
}
function ConsecutiveEntryField({ field, value, disabled, onChange }: {
  field: StrategyParamMeta; value: unknown; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  const [raw, setRaw] = useState(() => value === undefined ? '0' : String(value))
  // Keep decimal typing such as "1." while synchronizing actual external value changes.
  useEffect(() => {
    setRaw((previous) => Object.is(parsedDraft(field.key, previous), value) ? previous : value === undefined ? '0' : String(value))
  }, [field.key, value])
  const error = fieldError(field.key, value)
  return <TextField fullWidth size="small" label={field.label} value={raw}
    disabled={disabled} error={!!error} helperText={error ?? field.description}
    inputProps={{ inputMode: field.key === 'long_trend_period' ? 'numeric' : 'decimal' }}
    onChange={(event) => { setRaw(event.target.value); onChange(field.key, parsedDraft(field.key, event.target.value)) }} />
}
export function ConsecutiveEntryControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (key: string, value: number | string) => void
}) {
  return <Stack spacing={1} sx={{ minWidth: 0, maxWidth: '100%' }} data-testid="consecutive-entry-controls">
    <Alert severity="info" sx={{ py: 0.5, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }}>
      신규 진입 확인 · 세 조건은 0이면 비활성이고, 활성 조건을 모두 충족해야 매수합니다. 준비 미확인은 신규 진입만 보류하며 기존 연속 하락 청산을 막지 않습니다.
      실시간 연속 횟수는 약10초 가격관측, replay는 선택 봉 종가입니다. ATR·장기 MA는 실시간의 별도 완료 일봉, replay의 이전 완료 일봉을 사용합니다.
      ATR은 실제 완료 OHLC 15봉의 TR14가 필요하며 가격 tick만으로 준비되지 않습니다. ATR·MA 사용 시 일봉 replay만 지원합니다.
    </Alert>
    <Grid container spacing={2} sx={{ width: '100%', m: 0 }}>
      {fields.map((field) => <Grid item xs={12} sm={6} md={4} key={field.key}>
        <ConsecutiveEntryField field={field} value={params[field.key]} disabled={disabled} onChange={onChange} />
      </Grid>)}
    </Grid>
  </Stack>
}

export function ConsecutiveEntrySummary({ params }: { params: Record<string, unknown> }) {
  return <Typography variant="caption" display="block" sx={{ overflowWrap: 'anywhere' }}>
    신규 진입 필터 · 누적 상승률 {String(params.min_cumulative_move_pct ?? 0)}% · ATR {String(params.min_move_atr_multiplier ?? 0)}배 · 완료 종가 MA {String(params.long_trend_period ?? 0)}봉 · 0 비활성
  </Typography>
}
