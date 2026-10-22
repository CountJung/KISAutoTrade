import Alert from '@mui/material/Alert'
import Typography from '@mui/material/Typography'
import type { EntryFilterPreparation } from '../../../api/types'
import { failedBreakoutDailyFiltersDisabled } from './failedBreakoutControls'

const statusLabels: Partial<Record<EntryFilterPreparation['status'], string>> = {
  ready: '준비 완료', insufficientHistory: '이전 완료 자료 부족', invalidParams: '잘못된 설정',
  unsupportedObservation: '관측 형식 미지원', unverifiedVolume: '완료 거래량 미확인', zeroReferenceVolume: '기준 거래량 평균 0',
}
export function FailedBreakoutPreparation({ value, params, compact = false }: {
  value?: EntryFilterPreparation; params: Record<string, unknown>; compact?: boolean
}) {
  if (!value && failedBreakoutDailyFiltersDisabled(params)) return <Alert severity="info" sx={{ py: 0.5 }} data-testid="failed-breakout-preparation">
    돌파 완료 일봉 필터 비활성 · 기본 이전 가격관측 기준을 유지합니다. 수익 보호·보유 한도 설정은 위 요약에서 별도로 확인하세요.
  </Alert>
  if (!value) return <Alert severity="warning" sx={{ py: 0.5, overflowWrap: 'anywhere' }} data-testid="failed-breakout-preparation">
    돌파 진입 준비 정보 없음 · 활성 완료 일봉 필터의 이전/생략 응답입니다. 이전 고가·ATR·거래량 준비를 확인할 수 없습니다. 수량 산정 ATR로 대신 확인하지 않습니다.
  </Alert>
  const supported = (value.breakoutBasis === 'priceObservations' || value.breakoutBasis === 'completedHigh') && !!statusLabels[value.status]
    && (value.cadence === 'priceObservations' || value.cadence === 'priorCompletedDailyOhlcv')
  if (!supported) return <Alert severity="warning" sx={{ py: 0.5, overflowWrap: 'anywhere' }} data-testid="failed-breakout-preparation">
    돌파 진입 준비 형식 미지원 · 이전/알 수 없는 진단 응답의 준비 상태를 확정하지 않습니다.
  </Alert>
  return <Alert severity={value.ready ? 'info' : 'warning'} sx={{ py: 0.5, minWidth: 0, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }} data-testid="failed-breakout-preparation">
    <Typography variant="body2" fontWeight={700}>돌파 진입 준비 · {value.enabled ? statusLabels[value.status] : '비활성'}</Typography>
    <Typography variant="caption" display="block">
      {value.breakoutBasis === 'completedHigh' ? '이전 완료 일봉 고가' : '이전 가격관측 최대값'} · {value.cadence === 'priorCompletedDailyOhlcv' ? '이전 완료 일봉 OHLC·거래량 확인' : '가격관측 확인'}
      {' · '}필요 완료봉 {value.requiredCompletedBars} · 확보 {value.availableCompletedBars} · 실제 TR {value.trueRangeSamples}개
    </Typography>
    <Typography variant="caption" display="block">
      이전 ATR {typeof value.priorAtr === 'number' && Number.isFinite(value.priorAtr) ? value.priorAtr : '미확인'}
      {' · '}직전 완료 거래량 비율 {typeof value.priorVolumeRatio === 'number' && Number.isFinite(value.priorVolumeRatio) ? `${value.priorVolumeRatio}배${value.priorVolumeRatio === 0 ? ' (검증된 0)' : ''}` : '미확인'}
    </Typography>
    <Typography variant="caption" display="block">
      돌파 후보 {value.candidates} · 준비 미확인 보류 {value.unreadyCandidates} · ATR 거부 {value.atrRejections} · 거래량 거부 {value.volumeRejections ?? '정보 없음'} · 후보 통과 {value.passedCandidates}
    </Typography>
    {!compact && <Typography variant="caption" display="block">
      ATR 0은 자료가 충분하면 준비 완료입니다. 검증된 거래량 0은 양수 기준 평균에서 유효한 비율 0이며, 기준 평균 0이나 거래량 미확인과 다릅니다.
      후보 통과는 주문 제한·수량 제한 전 확인이며 실제 체결이 아닙니다. 수량 산정용 ATR 준비와 별도이고 현재 봉·누적 거래량을 미리 사용하지 않습니다.
    </Typography>}
  </Alert>
}
