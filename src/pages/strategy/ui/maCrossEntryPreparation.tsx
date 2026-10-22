import Alert from '@mui/material/Alert'
import Typography from '@mui/material/Typography'
import type { EntryFilterPreparation } from '../../../api/types'
import { maCrossEntryFiltersDisabled } from './maCrossEntryControls'

const statusLabels: Record<EntryFilterPreparation['status'], string> = {
  unverifiedVolume: '완료 거래량 미확인', zeroReferenceVolume: '기준 거래량 평균 0', ready: '준비 완료', insufficientHistory: '완료 자료 부족', invalidParams: '잘못된 설정', unsupportedObservation: '관측 형식 미지원', zeroAtr: 'ATR 0 상태 · 응답 확인 필요',
}
export function MaCrossEntryPreparation({ value, params, compact = false }: { value?: EntryFilterPreparation; params: Record<string, unknown>; compact?: boolean }) {
  if (!value && maCrossEntryFiltersDisabled(params)) return <Alert severity="info" sx={{ py: 0.5 }} data-testid="ma-cross-entry-preparation">
    MA 교차 진입 필터 비활성 · 네 조건이 모두 누락/0이므로 기존 관측 교차 규칙을 유지합니다.
  </Alert>
  if (!value) return <Alert severity="warning" sx={{ py: 0.5, overflowWrap: 'anywhere' }} data-testid="ma-cross-entry-preparation">
    MA 교차 진입 필터 준비 정보 없음 · 활성 필터의 이전/생략 응답입니다. 이전 완료 일봉 OHLC 준비를 확인할 수 없습니다. 수량 산정 ATR 정보로 대신 확인하지 않습니다.
  </Alert>
  return <Alert severity={value.ready ? 'info' : 'warning'} sx={{ py: 0.5, minWidth: 0, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }} data-testid="ma-cross-entry-preparation">
    <Typography variant="body2" fontWeight={700}>MA 교차 진입 필터 준비 · {value.enabled ? statusLabels[value.status] : '비활성'}</Typography>
    <Typography variant="caption" display="block">
      {value.cadence === 'priorCompletedDailyOhlc' ? '직전 완료 일봉 OHLC 확인' : '가격관측 확인 · 완료 일봉 필터 미사용'}
      {' · '}필요 완료봉 {value.requiredCompletedBars} · 확보 {value.availableCompletedBars} · 실제 TR {value.trueRangeSamples}개
    </Typography>
    <Typography variant="caption" display="block">
      골든크로스 후보 {value.candidates} · 준비 미확인 보류 {value.unreadyCandidates} · 추세 거부 {value.trendRejections}
      {' · '}기울기 거부 {value.slopeRejections ?? '정보 없음'} · ATR 거부 {value.atrRejections} · 밴드폭 거부 {value.bandwidthRejections ?? '정보 없음'} · 후보 통과 {value.passedCandidates}
    </Typography>
    {!compact && <Typography variant="caption" display="block">
      후보는 공통 guard의 실제 보유 검사 전 골든크로스이며 미보유/체결을 뜻하지 않습니다. 거부 사유는 중복 집계될 수 있고 통과는 실제 체결이 아닙니다.
      기존 데드크로스 매도는 유지합니다. 실행 수량 산정용 ATR 준비와 별도이며 현재 봉 OHLC를 판단에 미리 사용하지 않습니다.
    </Typography>}
  </Alert>
}
