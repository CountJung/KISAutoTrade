import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined'
import RefreshIcon from '@mui/icons-material/Refresh'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Checkbox from '@mui/material/Checkbox'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import FormControlLabel from '@mui/material/FormControlLabel'
import IconButton from '@mui/material/IconButton'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import type { LeveragedTrendHoldEntry } from '../../../api/types'
import { BollingerControls } from './bollingerControls'
import { LeveragedTrendHoldPreviewPanel } from './leveragedTrendHoldPreviewPanel'
import { LeveragedTrendHoldTickerPicker } from './leveragedTrendHoldTickerPicker'
import { useLeveragedTrendHoldEditorModel, type LeveragedTrendHoldEditorPanelProps } from './useLeveragedTrendHoldEditorModel'

export function hasInvalidLthEntries(entries: LeveragedTrendHoldEntry[]): boolean {
  return entries.some((entry) => !entry.leveraged_symbol)
}

export function LeveragedTrendHoldEditorPanel(props: LeveragedTrendHoldEditorPanelProps) {
  const model = useLeveragedTrendHoldEditorModel(props)
  const {
    breakevenBuffer, canAddTarget, doPickerRefreshList, entries,
    entryFailureObservations, entrySensitivity, handleAddTarget, handleQuantity,
    handleRemoveTarget, handleSensitivityChange, initialStopLoss, minHoldObservations,
    params, pickerError, pickerRefreshing, pickerStockListEmpty,
    rapidReboundDrop, rapidReboundEnabled, rapidReboundLookbackTicks, rapidReboundMaxLowAgeTicks,
    rapidReboundRecovery, reboundBaselineTicks, reboundBuyPressure, reboundConfirmTicks,
    reboundEnabled, reboundPullback, reboundRsiMin, stratEnabled,
    trailingActivationProfit, trailingStopPct, updateIntegerParam, updateNumericParam,
  } = model
  return (
    <Stack spacing={1.5}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" flexWrap="wrap" gap={1}>
        <Stack direction="row" alignItems="center" gap={0.5}>
          <Typography variant="caption" color="text.secondary" fontWeight={600}>
            레버리지 대상 ETF ({entries.length}개)
          </Typography>
          <Tooltip
            title="롱/숏 레버리지 구분 없이 선택한 ETF 자체가 상승 추세이면 매수하고, 추세가 훼손되면 청산합니다."
            arrow
          >
            <InfoOutlinedIcon sx={{ fontSize: 13, color: 'text.disabled', cursor: 'help' }} />
          </Tooltip>
        </Stack>
        <Button
          size="small"
          variant="outlined"
          startIcon={<AddIcon />}
          disabled={!canAddTarget || stratEnabled}
          onClick={handleAddTarget}
          sx={{ fontSize: '0.7rem', py: 0.3 }}
        >
          대상 추가
        </Button>
      </Stack>

      <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 1.25, bgcolor: 'action.hover' }}>
        <Stack spacing={1}>
          <LeveragedTrendHoldTickerPicker model={model} />

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} alignItems={{ xs: 'stretch', md: 'center' }}>
            <TextField
              label="진입 민감도"
              type="number"
              value={entrySensitivity}
              disabled={stratEnabled}
              size="small"
              onChange={(e) => handleSensitivityChange(Number(e.target.value))}
              inputProps={{ min: 1, max: 5, step: 0.1 }}
              sx={{ width: { xs: '100%', md: 140 } }}
            />
            <Typography variant="caption" color="text.secondary">
              1은 기본값, 값이 높을수록 상승 진입 RSI 기준을 완화합니다.
            </Typography>
          </Stack>

          <BollingerControls params={params} disabled={stratEnabled} onChange={props.onParamsUpdate} />
          <Box sx={{ borderTop: 1, borderColor: 'divider', pt: 1 }}>
            <Stack spacing={1}>
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} sx={{ flexWrap: 'wrap' }}>
                <TextField
                  label="초기 손절(%)"
                  type="number"
                  value={initialStopLoss}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('initial_stop_loss_pct', Number(e.target.value), 0.1, 20)}
                  inputProps={{ min: 0.1, max: 20, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 140 } }}
                />
                <TextField
                  label="실패 판정 관측치"
                  type="number"
                  value={entryFailureObservations}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('entry_failure_observations', Number(e.target.value), 1, 60)}
                  inputProps={{ min: 1, max: 60, step: 1 }}
                  sx={{ width: { xs: '100%', md: 160 } }}
                />
                <TextField
                  label="추적손절(%)"
                  type="number"
                  value={trailingStopPct}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('trailing_stop_pct', Number(e.target.value), 0.5, 20)}
                  inputProps={{ min: 0.5, max: 20, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 140 } }}
                />
                <TextField
                  label="추적 활성 수익(%)"
                  type="number"
                  value={trailingActivationProfit}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('trailing_activation_profit_pct', Number(e.target.value), 0.1, 20)}
                  inputProps={{ min: 0.1, max: 20, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 160 } }}
                />
                <TextField
                  label="본전 보호 버퍼(%)"
                  type="number"
                  value={breakevenBuffer}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('breakeven_buffer_pct', Number(e.target.value), 0, 10)}
                  inputProps={{ min: 0, max: 10, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 160 } }}
                />
                <TextField
                  label="최소 보유 관측치"
                  type="number"
                  value={minHoldObservations}
                  disabled={stratEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('min_hold_observations', Number(e.target.value), 0, 60)}
                  inputProps={{ min: 0, max: 60, step: 1 }}
                  sx={{ width: { xs: '100%', md: 150 } }}
                />
              </Stack>
              <Typography variant="caption" color="text.secondary">
                반등이 틀리면 초기 손절/실패 판정으로 먼저 빠지고, 고점 수익률이 활성 기준을 넘긴 뒤에는 본전 보호와 추적손절로 수익을 지킵니다.
              </Typography>
            </Stack>
          </Box>

          <Box sx={{ borderTop: 1, borderColor: 'divider', pt: 1 }}>
            <Stack spacing={1}>
              <FormControlLabel
                control={
                  <Checkbox
                    checked={rapidReboundEnabled}
                    disabled={stratEnabled}
                    onChange={(e) => props.onParamsUpdate({
                      ...params,
                      rapid_rebound_enabled: e.target.checked,
                    })}
                    size="small"
                  />
                }
                label={
                  <Typography variant="caption" fontWeight={600}>
                    급반등 단독 진입 사용
                  </Typography>
                }
              />
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={1}>
                <TextField
                  label="최근 관측치"
                  type="number"
                  value={rapidReboundLookbackTicks}
                  disabled={stratEnabled || !rapidReboundEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('rapid_rebound_lookback_ticks', Number(e.target.value), 3, 120)}
                  inputProps={{ min: 3, max: 120, step: 1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="선행 급락(%)"
                  type="number"
                  value={rapidReboundDrop}
                  disabled={stratEnabled || !rapidReboundEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('rapid_rebound_drop_pct', Number(e.target.value), 0.5, 30)}
                  inputProps={{ min: 0.5, max: 30, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="저점 회복(%)"
                  type="number"
                  value={rapidReboundRecovery}
                  disabled={stratEnabled || !rapidReboundEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('rapid_rebound_recovery_pct', Number(e.target.value), 0.5, 30)}
                  inputProps={{ min: 0.5, max: 30, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="저점 후 허용 관측치"
                  type="number"
                  value={rapidReboundMaxLowAgeTicks}
                  disabled={stratEnabled || !rapidReboundEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('rapid_rebound_max_low_age_ticks', Number(e.target.value), 1, 30)}
                  inputProps={{ min: 1, max: 30, step: 1 }}
                  sx={{ width: { xs: '100%', md: 170 } }}
                />
              </Stack>
              <Typography variant="caption" color="text.secondary">
                최근 관측 구간 안에서 먼저 급락이 발생하고, 저점 이후 지정 관측치 안에 강하게 되돌리면 추세/ADX 조건 없이 급반등만으로 진입합니다.
              </Typography>
            </Stack>
          </Box>

          <Box sx={{ borderTop: 1, borderColor: 'divider', pt: 1 }}>
            <Stack spacing={1}>
              <FormControlLabel
                control={
                  <Checkbox
                    checked={reboundEnabled}
                    disabled={stratEnabled}
                    onChange={(e) => props.onParamsUpdate({
                      ...params,
                      intraday_rebound_enabled: e.target.checked,
                    })}
                    size="small"
                  />
                }
                label={
                  <Typography variant="caption" fontWeight={600}>
                    장중 반동 진입 사용
                  </Typography>
                }
              />
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={1}>
                <TextField
                  label="기준 관측치"
                  type="number"
                  value={reboundBaselineTicks}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('rebound_baseline_ticks', Number(e.target.value), 2, 120)}
                  inputProps={{ min: 2, max: 120, step: 1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="확인 관측치"
                  type="number"
                  value={reboundConfirmTicks}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateIntegerParam('rebound_confirm_ticks', Number(e.target.value), 2, 60)}
                  inputProps={{ min: 2, max: 60, step: 1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="선행 하락(%)"
                  type="number"
                  value={reboundPullback}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('rebound_pullback_pct', Number(e.target.value), 0.5, 30)}
                  inputProps={{ min: 0.5, max: 30, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="매수세 상승(%)"
                  type="number"
                  value={reboundBuyPressure}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('rebound_buy_pressure_pct', Number(e.target.value), 0.5, 30)}
                  inputProps={{ min: 0.5, max: 30, step: 0.1 }}
                  sx={{ width: { xs: '100%', md: 130 } }}
                />
                <TextField
                  label="반동 RSI 하한"
                  type="number"
                  value={reboundRsiMin}
                  disabled={stratEnabled || !reboundEnabled}
                  size="small"
                  onChange={(e) => updateNumericParam('rebound_rsi_min', Number(e.target.value), 0, 70)}
                  inputProps={{ min: 0, max: 70, step: 1 }}
                  sx={{ width: { xs: '100%', md: 140 } }}
                />
              </Stack>
              <Typography variant="caption" color="text.secondary">
                특정 시각이 아니라 기준 관측 구간에서 충분히 밀린 뒤, 바로 다음 확인 구간에서 강한 가격 회복이 나올 때 매수세 반동으로 판단합니다.
              </Typography>
            </Stack>
          </Box>

          {pickerError && (
            <Alert severity="warning" sx={{ py: 0.5 }}>
              <Typography variant="caption">{pickerError}</Typography>
            </Alert>
          )}

          {pickerStockListEmpty && (
            <Alert
              severity="warning"
              sx={{ py: 0.5 }}
              action={
                <Button
                  size="small"
                  color="warning"
                  variant="outlined"
                  onClick={() => doPickerRefreshList()}
                  disabled={pickerRefreshing}
                  startIcon={pickerRefreshing ? <CircularProgress size={12} /> : <RefreshIcon fontSize="small" />}
                >
                  {pickerRefreshing ? '다운로드 중...' : '목록 새로고침'}
                </Button>
              }
            >
              <Typography variant="caption">종목 목록이 로드되지 않았습니다.</Typography>
            </Alert>
          )}
        </Stack>
      </Box>

      {entries.length > 0 && (
        <LeveragedTrendHoldPreviewPanel model={model} />
      )}

      {entries.length === 0 ? (
        <Typography variant="caption" color="text.disabled" sx={{ pl: 0.5 }}>
          추가된 레버리지 대상 ETF가 없습니다
        </Typography>
      ) : (
        <TableContainer sx={{ border: 1, borderColor: 'divider', borderRadius: 1, overflowX: 'auto' }}>
          <Table size="small" sx={{ minWidth: 620 }}>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 90 }}>시장</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 120 }}>티커</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75 }}>종목명</TableCell>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, width: 100 }} align="center">수량</TableCell>
                <TableCell sx={{ width: 36, py: 0.75 }} />
              </TableRow>
            </TableHead>
            <TableBody>
              {entries.map((entry) => (
                <TableRow key={entry.leveraged_symbol}>
                  <TableCell sx={{ py: 0.75 }}>
                    <Chip
                      size="small"
                      label={entry.is_overseas ? '미국' : '국내'}
                      color={entry.is_overseas ? 'primary' : 'default'}
                      variant="outlined"
                      sx={{ height: 22, fontSize: '0.65rem' }}
                    />
                  </TableCell>
                  <TableCell sx={{ py: 0.75 }}>
                    <Typography variant="caption" color="primary.main" fontWeight={700}>
                      {entry.leveraged_symbol}
                    </Typography>
                  </TableCell>
                  <TableCell sx={{ py: 0.75 }}>
                    <Typography variant="caption" color="text.secondary" noWrap>
                      {entry.leveraged_symbol_name || entry.leveraged_symbol}
                    </Typography>
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }} align="center">
                    <TextField
                      type="number"
                      value={entry.quantity}
                      disabled={stratEnabled}
                      size="small"
                      onChange={(e) => handleQuantity(entry.leveraged_symbol, Number(e.target.value))}
                      inputProps={{
                        'aria-label': `${entry.leveraged_symbol} 수량`,
                        min: 1,
                        step: 1,
                        style: { padding: '4px 4px', fontSize: '0.75rem', textAlign: 'right' },
                      }}
                      sx={{ width: 80, '& .MuiInputBase-root': { fontSize: '0.75rem' } }}
                    />
                  </TableCell>
                  <TableCell sx={{ py: 0.5 }}>
                    <IconButton
                      size="small"
                      disabled={stratEnabled}
                      aria-label={`${entry.leveraged_symbol} 삭제`}
                      onClick={() => handleRemoveTarget(entry.leveraged_symbol)}
                    >
                      <DeleteIcon sx={{ fontSize: 14 }} />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Stack>
  )
}
