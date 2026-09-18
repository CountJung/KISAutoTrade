import Box from '@mui/material/Box'
import Checkbox from '@mui/material/Checkbox'
import FormControlLabel from '@mui/material/FormControlLabel'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'

export function BollingerControls({ params, disabled, onChange }: {
  params: Record<string, unknown>; disabled: boolean; onChange: (params: Record<string, unknown>) => void
}) {
  const value = (params.bollinger ?? {}) as Record<string, unknown>
  const enabled = value.enabled === true
  const update = (key: string, next: number | boolean) => onChange({ ...params, bollinger: { ...value, [key]: next } })
  const fields = [
    ['period', '볼린저 기간(봉)', 20, 5, 100, 1],
    ['stddev_multiplier', '표준편차 배수', 2, 1, 4, 0.1],
    ['trend_period', '장기 추세 기간(봉)', 60, 5, 250, 1],
    ['squeeze_lookback', '밴드폭 비교 기간(봉)', 20, 5, 100, 1],
    ['squeeze_ratio', '스퀴즈 폭 비율', 0.5, 0.1, 1, 0.05],
    ['atr_stop_multiplier', '볼린저 ATR 손절 배수', 2, 0.5, 10, 0.1],
  ] as const
  return (
    <Box sx={{ borderTop: 1, borderColor: 'divider', pt: 1 }}>
      <Stack spacing={1}>
        <FormControlLabel label="볼린저 스퀴즈·추세 돌파 보강" control={<Checkbox checked={enabled} disabled={disabled} onChange={e => update('enabled', e.target.checked)} />} />
        <Typography variant="caption" color="text.secondary">
          기존 추세·반동·급반등 진입에 추가 조건을 적용합니다. 최근 5개 확정봉 안의 밴드폭 축소 후,
          밴드폭 확장·중심선 상승·장기 평균 위 상단 돌파를 모두 확인합니다. 상단 접촉만으로 매도하지 않습니다.
          중심선 종가 이탈·ATR 손절을 추가하며 기존 손절은 유지합니다.
        </Typography>
        <Typography variant="caption" color="text.secondary">
          실시간은 1분봉 기준이며 다음 관측에서 직전 봉을 확정합니다. 일봉과 분봉을 섞지 않고,
          준비된 봉이 부족하면 매수를 보류합니다. 보강 옵션은 1분봉 미리보기로 점검하세요.
        </Typography>
        <Stack direction="row" useFlexGap flexWrap="wrap" gap={1}>
          {fields.map(([key, label, fallback, min, max, step]) => (
            <TextField key={key} label={label} type="number" size="small" value={typeof value[key] === 'number' ? value[key] : fallback}
              disabled={disabled || !enabled} inputProps={{ min, max, step }} sx={{ width: { xs: '100%', sm: 175 } }}
              onChange={e => { const number = Number(e.target.value); if (Number.isFinite(number)) update(key, Math.max(min, Math.min(max, step === 1 ? Math.round(number) : number))) }} />
          ))}
        </Stack>
      </Stack>
    </Box>
  )
}
