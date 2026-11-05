import Alert from '@mui/material/Alert'
import Typography from '@mui/material/Typography'
import type { VolatilityExpansionPreparation } from '../../../api/types'

const statuses: Record<VolatilityExpansionPreparation['status'], string> = {
  ready: '준비 완료', insufficientHistory: '이전 완료 자료 부족', invalidParams: '잘못된 설정', unsupportedObservation: '관측 형식 미지원',
}
const cadences: Record<VolatilityExpansionPreparation['cadence'], string> = {
  dailyCloseRange: '일봉 종가 완성 범위', observedLiveQuotes: '실시간 관측 가격 범위', unverified: '관측 주기 미확인',
}
const counts = ['requiredCompletedBars', 'availableCompletedBars', 'evaluatedObservations', 'ownedObservations', 'candidateObservations', 'unreadyCandidates', 'rangeRejections', 'directionRejections', 'passedCandidates'] as const
function validPreparation(value: VolatilityExpansionPreparation): boolean {
  return typeof value === 'object' && value !== null && typeof value.ready === 'boolean'
    && Object.prototype.hasOwnProperty.call(statuses, value.status) && Object.prototype.hasOwnProperty.call(cadences, value.cadence)
    && counts.every((key) => typeof value[key] === 'number' && Number.isSafeInteger(value[key]) && value[key] >= 0)
    && value.requiredCompletedBars >= 1 && value.requiredCompletedBars <= 512 && value.availableCompletedBars <= 512
    && value.ready === (value.status === 'ready')
    && (!value.ready || (value.cadence !== 'unverified' && value.availableCompletedBars >= value.requiredCompletedBars))
    && value.evaluatedObservations === value.ownedObservations + value.candidateObservations
    && value.candidateObservations === value.unreadyCandidates + value.rangeRejections + value.directionRejections + value.passedCandidates
}
export function VolatilityPreparation({ value, compact = false }: { value?: VolatilityExpansionPreparation; compact?: boolean }) {
  if (value === undefined || value === null) return <Alert severity="warning" sx={{ py: 0.5, overflowWrap: 'anywhere' }} data-testid="volatility-expansion-preparation">
    변동성 확장 준비 정보 없음 · 이전/생략 응답에서 완료 일봉 범위의 준비·후보·거부를 확인할 수 없습니다. 0이나 준비 완료로 대신 표시하지 않습니다. 수량 산정 ATR와 별도입니다.
  </Alert>
  if (!validPreparation(value)) return <Alert severity="warning" sx={{ py: 0.5, overflowWrap: 'anywhere' }} data-testid="volatility-expansion-preparation">
    변동성 확장 준비 정보 확인 필요 · 알 수 없는 형식·손상된 수치나 분류입니다. 준비 상태·후보 수를 확정하지 않습니다.
  </Alert>
  return <Alert severity={value.ready ? 'info' : 'warning'} sx={{ py: 0.5, minWidth: 0, overflowWrap: 'anywhere', '& .MuiAlert-message': { minWidth: 0 } }} data-testid="volatility-expansion-preparation">
    <Typography variant="body2" sx={{ fontWeight: 700 }}>변동성 확장 준비 · {statuses[value.status]}</Typography>
    <Typography variant="caption" sx={{ display: "block" }}>주기: {cadences[value.cadence]} · 필요 완료봉 {value.requiredCompletedBars} · 확보 {value.availableCompletedBars}</Typography>
    <Typography variant="caption" sx={{ display: "block" }}>평가 관측 {value.evaluatedObservations} · 보유 관측 {value.ownedObservations} · 미보유 후보 {value.candidateObservations}</Typography>
    <Typography variant="caption" sx={{ display: "block" }}>준비 미확인 보류 {value.unreadyCandidates} · 범위 거부 {value.rangeRejections} · 방향 거부 {value.directionRejections} · 후보 통과 {value.passedCandidates}</Typography>
    {!compact && <Typography variant="caption" sx={{ display: "block" }}>
      미보유 후보는 준비 확인 → 이전 평균 대비 범위 초과 → 시가 대비 상승 순서로 분류하므로 거부는 중복되지 않습니다. 보유 관측은 진입 후보에 포함하지 않습니다.
      후보 통과는 공통 주문 제한·수량 검사 전 확인이며 주문·체결이 아닙니다. 범위의 평균0은 자료 부족과 다르고 충분한 0범위도 평균에 포함합니다. 수량 산정용 ATR·다른 전략의 진입 필터와 별도입니다.
    </Typography>}
  </Alert>
}
