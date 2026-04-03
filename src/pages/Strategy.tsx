import Typography from '@mui/material/Typography'
import Paper from '@mui/material/Paper'
import Box from '@mui/material/Box'
import Grid from '@mui/material/Grid'
import Switch from '@mui/material/Switch'
import FormControlLabel from '@mui/material/FormControlLabel'
import Chip from '@mui/material/Chip'
import Divider from '@mui/material/Divider'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import CircularProgress from '@mui/material/CircularProgress'
import Button from '@mui/material/Button'
import SaveIcon from '@mui/icons-material/Save'
import { useState } from 'react'
import { useStrategies, useUpdateStrategy, useTradingStatus } from '../api/hooks'
import type { UpdateStrategyInput } from '../api/types'

export default function Strategy() {
  const { data: strategies, isLoading } = useStrategies()
  const { data: tradingStatus } = useTradingStatus()
  const { mutate: updateStrategy, isPending: saving } = useUpdateStrategy()

  // 편집 중인 파라미터를 로컬 상태로 관리
  const [editMap, setEditMap] = useState<Record<string, { symbols: string; quantity: number; shortPeriod: number; longPeriod: number }>>({})

  const getEdit = (id: string, strategy: { targetSymbols: string[]; orderQuantity: number; params: Record<string, unknown> }) => {
    if (editMap[id]) return editMap[id]
    return {
      symbols: strategy.targetSymbols.join(','),
      quantity: strategy.orderQuantity,
      shortPeriod: (strategy.params.short_period as number) ?? 5,
      longPeriod: (strategy.params.long_period as number) ?? 20,
    }
  }

  const setEdit = (id: string, patch: Partial<{ symbols: string; quantity: number; shortPeriod: number; longPeriod: number }>) => {
    setEditMap((prev) => ({ ...prev, [id]: { ...getEdit(id, strategies!.find(s => s.id === id)!), ...patch } }))
  }

  const handleToggle = (id: string, enabled: boolean) => {
    const input: UpdateStrategyInput = { id, enabled }
    updateStrategy(input)
  }

  const handleSave = (id: string) => {
    const edit = editMap[id]
    if (!edit) return
    const input: UpdateStrategyInput = {
      id,
      targetSymbols: edit.symbols.split(',').map(s => s.trim()).filter(Boolean),
      orderQuantity: edit.quantity,
      params: { short_period: edit.shortPeriod, long_period: edit.longPeriod },
    }
    updateStrategy(input, { onSuccess: () => setEditMap((prev) => { const n = { ...prev }; delete n[id]; return n }) })
  }

  const activeCount = strategies?.filter(s => s.enabled).length ?? 0
  const isRunning = tradingStatus?.isRunning ?? false

  if (isLoading) {
    return <Box sx={{ display: 'flex', justifyContent: 'center', pt: 8 }}><CircularProgress /></Box>
  }

  return (
    <Box>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 3 }}>
        <Typography variant="h5" fontWeight={700}>
          Strategy
        </Typography>
        <Chip
          label={`${activeCount}개 활성`}
          color={activeCount > 0 ? 'success' : 'default'}
          size="small"
        />
        {isRunning && (
          <Chip label="자동매매 실행 중" color="success" size="small" variant="outlined" />
        )}
      </Box>

      <Grid container spacing={2}>
        {(strategies ?? []).map((s) => {
          const edit = getEdit(s.id, s)
          const isDirty = !!editMap[s.id]
          return (
            <Grid item xs={12} md={6} key={s.id}>
              <Paper sx={{ p: 3 }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}>
                  <Typography variant="subtitle1" fontWeight={600}>
                    {s.name}
                  </Typography>
                  <FormControlLabel
                    control={
                      <Switch
                        checked={s.enabled}
                        onChange={(e) => handleToggle(s.id, e.target.checked)}
                        color="success"
                        disabled={saving}
                      />
                    }
                    label={s.enabled ? '실행 중' : '정지'}
                    labelPlacement="start"
                  />
                </Box>
                <Divider sx={{ mb: 2 }} />

                <Stack spacing={2}>
                  <TextField
                    label="대상 종목코드 (쉼표 구분)"
                    value={edit.symbols}
                    onChange={(e) => setEdit(s.id, { symbols: e.target.value })}
                    size="small"
                    disabled={s.enabled}
                    helperText="예: 005930,035720"
                  />
                  <Grid container spacing={2}>
                    <Grid item xs={4}>
                      <TextField
                        label="1회 수량"
                        type="number"
                        value={edit.quantity}
                        onChange={(e) => setEdit(s.id, { quantity: Number(e.target.value) })}
                        size="small"
                        disabled={s.enabled}
                        inputProps={{ min: 1 }}
                      />
                    </Grid>
                    <Grid item xs={4}>
                      <TextField
                        label="단기 MA"
                        type="number"
                        value={edit.shortPeriod}
                        onChange={(e) => setEdit(s.id, { shortPeriod: Number(e.target.value) })}
                        size="small"
                        disabled={s.enabled}
                        inputProps={{ min: 2, max: 50 }}
                      />
                    </Grid>
                    <Grid item xs={4}>
                      <TextField
                        label="장기 MA"
                        type="number"
                        value={edit.longPeriod}
                        onChange={(e) => setEdit(s.id, { longPeriod: Number(e.target.value) })}
                        size="small"
                        disabled={s.enabled}
                        inputProps={{ min: 5, max: 200 }}
                      />
                    </Grid>
                  </Grid>
                </Stack>

                <Box sx={{ mt: 2, p: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
                  <Typography variant="caption" color="text.secondary">
                    단기 {edit.shortPeriod}MA가 장기 {edit.longPeriod}MA를 상향 돌파 시 매수 (골든크로스),
                    하향 돌파 시 매도 (데드크로스)
                  </Typography>
                </Box>

                {isDirty && !s.enabled && (
                  <Box sx={{ mt: 1.5 }}>
                    <Button
                      size="small"
                      variant="outlined"
                      startIcon={saving ? <CircularProgress size={14} /> : <SaveIcon />}
                      onClick={() => handleSave(s.id)}
                      disabled={saving}
                    >
                      변경사항 저장
                    </Button>
                  </Box>
                )}
              </Paper>
            </Grid>
          )
        })}
      </Grid>
    </Box>
  )
}

