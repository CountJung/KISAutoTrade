import Alert from '@mui/material/Alert'
import Typography from '@mui/material/Typography'
import type { EntryFilterPreparation } from '../../../api/types'
import { consecutiveEntryFiltersDisabled } from './consecutiveEntryControls'

const statusLabels: Record<EntryFilterPreparation['status'], string> = {
  ready: '준비 완료',
  insufficientHistory: '완료 자료 부족',
  invalidParams: '잘못된 설정',
  unsupportedObservation: '관측 형식 미지원',
  zeroAtr: 'ATR 0 · 진입 보류',
}
export function ConsecutiveEntryPreparation({ value, params, compact = false }: { value?: EntryFilterPreparation; params: Record<string, unknown>; compact?: boolean }) {
  if (!value && consecutiveEntryFiltersDisabled(params)) return <Alert severity="info" sx={{ py: 0.5 }} data-testid="entry-filter-preparation">
    신규 진입 필터 비활성 · 세 조건이 모두 0이므로 추가 진입 준비 진단을 사용하지 않습니다. 기존 연속 상승/하락 관측 규칙을 유지합니다.
  </Alert>
  if (!value) return <Alert severity="warning" sx={{ py: 0.5, overflowWrap: 'anywhere' }} data-testid="entry-filter-preparation">
    신규 진입 필터 준비 정보 없음 · 활성 필터의 이전/생략 응답입니다. 실제 완료 OHLC 준비를 확인할 수 없습니다. 수량 산정 ATR 정보로 대신 확인하지 않습니다.
  </Alert>
  return <Alert severity={value.ready ? 'info' : 'warning'} sx={{ py: 0.5, minWidth: 0, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }} data-testid="entry-filter-preparation">
    <Typography variant="body2" fontWeight={700}>신규 진입 필터 준비 · {value.enabled ? statusLabels[value.status] : '비활성'}</Typography>
    <Typography variant="caption" display="block">
      {value.cadence === 'priorCompletedDailyOhlc' ? '직전 완료 일봉 OHLC 확인' : '가격관측 확인 · 완료 OHLC 필터 미사용'}
      {' · '}필요 완료봉 {value.requiredCompletedBars} · 확보 {value.availableCompletedBars} · 실제 TR {value.trueRangeSamples}개
    </Typography>
    <Typography variant="caption" display="block">
      연속 상승·미보유 후보 {value.candidates} · 준비 미확인 보류 {value.unreadyCandidates} · 누적 상승률 거부 {value.minMoveRejections} · ATR 거부 {value.atrRejections} · 장기 MA 거부 {value.trendRejections} · 후보 통과 {value.passedCandidates}
    </Typography>
    {!compact && <Typography variant="caption" display="block">
      신규 진입만 확인하며 기존 연속 하락 청산은 유지합니다. 거부 사유는 중복 집계될 수 있고 후보 통과는 실제 체결이 아닙니다.
      실행 수량 산정용 ATR 준비와 별도이며 현재 봉의 OHLC·가격 tick으로 이전 완료 일봉을 합성하지 않습니다.
    </Typography>}
  </Alert>
}
