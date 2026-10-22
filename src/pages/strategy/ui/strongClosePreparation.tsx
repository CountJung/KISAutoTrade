import Alert from '@mui/material/Alert'
import Typography from '@mui/material/Typography'
import type { EntryFilterPreparation } from '../../../api/types'
import { strongCloseTrendDisabled } from './strongCloseControls'

const statusLabels: Partial<Record<EntryFilterPreparation['status'], string>> = {
  ready: '준비 완료', insufficientHistory: '이전 완료 자료 부족', invalidParams: '잘못된 설정', unsupportedObservation: '관측 형식 미지원',
}
export function StrongClosePreparation({ value, params, compact = false }: {
  value?: EntryFilterPreparation; params: Record<string, unknown>; compact?: boolean
}) {
  if (!value && strongCloseTrendDisabled(params)) return <Alert severity="info" sx={{ py: 0.5 }} data-testid="strong-close-preparation">
    강한 종가 진입 추세 비활성 · 이전 완료 일봉 추세가 누락/0이며 기존 전일 고가·종가 조건을 유지합니다. 수익 보호·보유 한도는 별도입니다.
  </Alert>
  if (!value) return <Alert severity="warning" sx={{ py: 0.5, overflowWrap: 'anywhere' }} data-testid="strong-close-preparation">
    강한 종가 진입 준비 정보 없음 · 활성 추세 또는 잘못된 선택 설정의 이전/생략 응답입니다. 이전 완료 일봉 준비를 확인할 수 없습니다. 수량 산정 ATR로 대신 확인하지 않습니다.
  </Alert>
  if (value.cadence !== 'priorCompletedDailyOhlc' || !statusLabels[value.status]) return <Alert severity="warning" sx={{ py: 0.5, overflowWrap: 'anywhere' }} data-testid="strong-close-preparation">
    강한 종가 진입 준비 형식 미지원 · 이전/알 수 없는 진단 응답의 준비 상태를 확정하지 않습니다.
  </Alert>
  return <Alert severity={value.ready ? 'info' : 'warning'} sx={{ py: 0.5, minWidth: 0, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }} data-testid="strong-close-preparation">
    <Typography variant="body2" fontWeight={700}>강한 종가 진입 준비 · {value.enabled ? statusLabels[value.status] : '비활성'}</Typography>
    <Typography variant="caption" display="block">이전 완료 일봉 OHLC 확인 · 필요 완료봉 {value.requiredCompletedBars} · 확보 {value.availableCompletedBars}</Typography>
    <Typography variant="caption" display="block">전일 조건 후보 {value.candidates} · 준비 미확인 보류 {value.unreadyCandidates} · 추세 거부 {value.trendRejections} · 후보 통과 {value.passedCandidates}</Typography>
    {!compact && <Typography variant="caption" display="block">
      현재 후보 가격이 이전 N완료 일봉 종가 평균보다 엄격히 높아야 합니다. 현재 봉 OHLC는 이 평균에 미리 들어가지 않습니다.
      후보 통과는 공통 주문 제한·실제 소유·수량 검사 전 확인이며 실제 체결이 아닙니다. 수량 산정용 ATR 준비와 별도입니다.
    </Typography>}
  </Alert>
}
