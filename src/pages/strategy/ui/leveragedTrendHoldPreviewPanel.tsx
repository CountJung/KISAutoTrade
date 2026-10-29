import RefreshIcon from '@mui/icons-material/Refresh'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import type { CmdError } from '../../../api/types'
import { LeveragedTrendHoldPreviewChart } from './leveragedTrendHoldPreviewChart'
import { SimulationAssumptionsEditor, StrategyResearchResults } from './strategyResearchPanel'
import type { LeveragedTrendHoldEditorModel, PreviewCount, PreviewInterval } from './useLeveragedTrendHoldEditorModel'

// Presentation only: the parent retains query/state ownership and callback closures.
export function LeveragedTrendHoldPreviewPanel({ model }: { model: LeveragedTrendHoldEditorModel }) {
  const {
    entries, accountingError, appConfig, assumptions,
    bollingerEnabled, currentPreview, handlePreview, isTossActive,
    params, previewCount, previewEntry, previewGeneration,
    previewInterval, previewMutation, previewOptions, previewSymbol,
    setAccountingDraftRevision, setAccountingError, setAssumptions, setPreview,
    setPreviewCount, setPreviewInterval, setPreviewSymbol,
  } = model
  return (
    <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 1.25 }}>
      <Stack spacing={1}>
        <SimulationAssumptionsEditor
          key={`${appConfig?.active_profile_id}:${appConfig?.active_broker_account_id}:${previewEntry?.leveraged_symbol}`}
          isOverseas={previewEntry?.is_overseas ?? false} onValidationChange={setAccountingError}
          onDraftChange={() => { previewGeneration.current += 1; setPreview(null); setAccountingDraftRevision((revision) => revision + 1) }}
          value={assumptions}
          onChange={setAssumptions}
          disabled={previewMutation.isPending}
        />
        <Stack direction={{ xs: 'column', md: 'row' }} alignItems={{ xs: 'stretch', md: 'center' }} justifyContent="space-between" gap={1}>
          <Stack spacing={0.25}>
            <Stack direction="row" alignItems="center" gap={0.75} flexWrap="wrap">
              <Typography variant="caption" color="text.secondary" fontWeight={700}>
                전략 미리보기
              </Typography>
              <Chip
                size="small"
                variant="outlined"
                label={`Toss ${previewInterval === '1m' ? '1분봉' : '일봉'} · 최근 ${previewCount}봉`}
                sx={{ height: 22, fontSize: '0.65rem' }}
              />
            </Stack>
            <Typography variant="caption" color="text.secondary">
              현재 편집 중인 파라미터를 기준으로 매수/청산 신호를 읽기 전용으로 재계산합니다.
            </Typography>
          </Stack>
          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} alignItems={{ xs: 'stretch', sm: 'center' }}>
            <TextField
              select
              label="시뮬레이션 티커"
              size="small"
              value={previewSymbol}
              disabled={previewOptions.length === 0 || previewMutation.isPending}
              onChange={(e) => {
                setPreviewSymbol(e.target.value)
                previewMutation.reset()
              }}
              sx={{ minWidth: { xs: '100%', sm: 180 } }}
            >
              {previewOptions.map((entry) => (
                <MenuItem key={entry.leveraged_symbol} value={entry.leveraged_symbol}>
                  {entry.leveraged_symbol} · {entry.leveraged_symbol_name || (entry.is_overseas ? '미국 ETF' : '국내 ETF')}
                </MenuItem>
              ))}
            </TextField>
            <TextField
              select
              label="봉 단위"
              size="small"
              value={previewInterval}
              disabled={previewMutation.isPending}
              onChange={(event) => setPreviewInterval(event.target.value as PreviewInterval)}
              sx={{ minWidth: { xs: '100%', sm: 110 } }}
            >
              <MenuItem value="1m">1분봉</MenuItem>
              <MenuItem value="1d" disabled={bollingerEnabled}>일봉</MenuItem>
            </TextField>
            <TextField
              select
              label="분석 구간"
              size="small"
              value={previewCount}
              disabled={previewMutation.isPending}
              onChange={(event) => setPreviewCount(Number(event.target.value) as PreviewCount)}
              sx={{ minWidth: { xs: '100%', sm: 120 } }}
            >
              <MenuItem value={50}>최근 50봉</MenuItem>
              <MenuItem value={100}>최근 100봉</MenuItem>
              <MenuItem value={200}>최근 200봉</MenuItem>
            </TextField>
            <Button
              size="small"
              variant="outlined"
              onClick={handlePreview}
              disabled={!isTossActive || !previewEntry || previewMutation.isPending || !!accountingError}
              startIcon={previewMutation.isPending ? <CircularProgress size={14} /> : <RefreshIcon fontSize="small" />}
              sx={{ alignSelf: { xs: 'stretch', sm: 'center' }, whiteSpace: 'nowrap' }}
            >
              {previewMutation.isPending ? '계산 중...' : '미리보기 계산'}
            </Button>
          </Stack>
        </Stack>

        {!isTossActive && (
          <Alert severity="info" sx={{ py: 0.5 }}>
            <Typography variant="caption">
              Toss 활성 프로파일에서 Toss 1분봉 기반 미리보기를 사용할 수 있습니다.
            </Typography>
          </Alert>
        )}

        {previewMutation.isError && (
          <Alert severity="warning" sx={{ py: 0.5 }}>
            <Typography variant="caption">
              {(previewMutation.error as CmdError | null)?.message ?? '전략 미리보기 계산에 실패했습니다.'}
            </Typography>
          </Alert>
        )}

        {currentPreview ? (
          <Stack spacing={1}>
            <Alert severity={currentPreview.signals.length > 0 ? 'success' : 'info'} sx={{ py: 0.5 }}>
              <Typography variant="caption">{currentPreview.message}</Typography>
            </Alert>
            <LeveragedTrendHoldPreviewChart
              candles={currentPreview.candles}
              signals={currentPreview.signals}
              sourceLabel={`Toss ${currentPreview.interval === '1m' ? '1분봉' : '일봉'} · ${currentPreview.candleCount}봉`}
            />
            {currentPreview.backtest && currentPreview.replay && previewEntry && (
              <StrategyResearchResults
                report={currentPreview.backtest}
                replay={currentPreview.replay}
                strategyId="leveraged_trend_hold"
                brokerId="toss"
                brokerAccountId={appConfig?.active_broker_account_id ?? null}
                symbol={previewEntry.leveraged_symbol}
                params={{ ...params, entries }}
                orderQuantity={previewEntry.quantity}
                generatedAt={currentPreview.generatedAt}
              />
            )}
          </Stack>
        ) : (
          <Box sx={{ minHeight: 160, display: 'grid', placeItems: 'center', border: 1, borderColor: 'divider', borderRadius: 1, bgcolor: 'action.hover' }}>
            <Typography variant="caption" color="text.secondary">
              봉 단위와 분석 구간을 선택한 뒤 계산하면 매수/매도 신호가 표시됩니다.
            </Typography>
          </Box>
        )}
      </Stack>
    </Box>
  )
}
