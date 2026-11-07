import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import type { LthExecutionClock, LthMinuteTiming } from '../../../api/types'

function supportedClock(value: unknown): value is LthExecutionClock {
  if (!value || typeof value !== 'object') return false
  const clock = value as LthExecutionClock
  const count = (n: unknown) => typeof n === 'number' && Number.isSafeInteger(n) && n >= 0
  return clock.version === 'us-regular-close-v1'
    && ['timezone', 'tzdbVersion', 'labelPolicy', 'blackoutTimezone', 'calendarSource', 'calendarHorizon', 'asOf', 'admissionIdentity']
      .every((key) => typeof clock[key as keyof LthExecutionClock] === 'string' && String(clock[key as keyof LthExecutionClock]).length > 0)
    && ['sourceRows', 'adoptedRows', 'warmupRows', 'unknownRows', 'omittedExamples', 'dayCount', 'omittedDays'].every((key) => count(clock[key as keyof LthExecutionClock]))
    && Array.isArray(clock.blackoutWindows) && clock.blackoutWindows.every((window) => typeof window === 'string')
    && Array.isArray(clock.days) && clock.days.length <= 8 && clock.days.every((day) => day && typeof day.date === 'string' && typeof day.open === 'string' && typeof day.close === 'string' && typeof day.earlyClose === 'boolean' && count(day.adoptedRows))
    && Array.isArray(clock.excluded) && clock.excluded.length <= 10 && clock.excluded.every((row) => row && typeof row.reason === 'string' && count(row.count))
    && Array.isArray(clock.examples) && clock.examples.length <= 6 && clock.examples.every((row) => row && typeof row.label === 'string' && typeof row.labelTruncated === 'boolean' && typeof row.reason === 'string' && count(row.inputIndex))
}

export function hasLthExecutionClock(value?: LthMinuteTiming): boolean {
  return !!value && value.diagnosticVersion === 'lth-minute-timing-v1' && hasSupportedCollections(value)
    && supportedClock(value.executionClock)
}

export function hasLthExecutionEvidence(value?: LthMinuteTiming): boolean {
  return hasLthExecutionClock(value) && !!value?.executionClock && value.executionClock.unknownRows === 0 && value.executionClock.adoptedRows > 0
}

export function lthTimingComparisonKey(value?: LthMinuteTiming): string {
  if (!value) return 'unavailable'
  return JSON.stringify([
    value.diagnosticVersion, value.exchangeTimezone, value.labelConvention,
    value.legacySessionPolicy, value.calendarVerified, value.providerCoverageVerified,
    value.requestedPeriodCoverageVerified, value.actualPublicationVerified,
    supportedClock(value.executionClock) ? [value.executionClock.version, value.executionClock.timezone,
      value.executionClock.tzdbVersion, value.executionClock.labelPolicy, value.executionClock.blackoutTimezone, value.executionClock.blackoutWindows,
      value.executionClock.calendarSource, value.executionClock.calendarHorizon, value.executionClock.admissionIdentity,
      value.executionClock.days, value.executionClock.dayCount, value.executionClock.unknownRows] : 'legacy/unknown',
  ])
}

const labelNames = { barClose: '봉 종료 표기', barStart: '봉 시작 표기', unknown: '봉 표기 미확인' }
const timeText = (value: string | null) => value ?? '미확인'

function hasSupportedCollections(value: LthMinuteTiming): boolean {
  return Array.isArray(value.days) && value.days.length <= 8
    && value.days.every((day) => day && typeof day === 'object'
      && typeof day.marketDate === 'string'
      && ['utcOffsetMinutes', 'regularUniqueBars', 'ordinaryWindowExpectedSlots',
        'interiorUnobservedSlots', 'maxInteriorGapSlots', 'leadingUnobservedWindowSlots',
        'trailingUnobservedWindowSlots'].every((key) => Number.isFinite(day[key as keyof typeof day]))
      && typeof day.firstCloseMarketLocal === 'string' && typeof day.lastCloseMarketLocal === 'string')
    && Array.isArray(value.examples) && value.examples.length <= 6
    && value.examples.every((example) => example && typeof example.kind === 'string' && typeof example.label === 'string')
    && Array.isArray(value.marketUtcOffsetsMinutes) && value.marketUtcOffsetsMinutes.every(Number.isFinite)
    && Array.isArray(value.limitations) && value.limitations.every((text) => typeof text === 'string')
}

export function LthMinuteTimingPanel({ value, compact = false }: { value?: LthMinuteTiming; compact?: boolean }) {
  if (!value) return <Alert severity="warning" data-testid="lth-minute-timing-missing" sx={{ py: 0.5, overflowWrap: 'anywhere' }}>
    분봉 시각 진단 정보 없음 · 이전 응답이나 저장 결과입니다. 시간대·봉 표기·내부 미관측 구간을 확인할 수 없습니다.
  </Alert>
  if (value.diagnosticVersion !== 'lth-minute-timing-v1' || !hasSupportedCollections(value)) return <Alert severity="warning" data-testid="lth-minute-timing-unsupported" sx={{ py: 0.5, overflowWrap: 'anywhere' }}>
    분봉 시각 진단 형식 확인 필요 · 이 화면에서 지원하지 않는 진단입니다. 다시 계산하고 원본 자료를 확인하세요.
  </Alert>
  const clock = supportedClock(value.executionClock) ? value.executionClock : undefined
  return <Box data-testid="lth-minute-timing" sx={{ minWidth: 0, overflowWrap: 'anywhere' }}>
    <Stack spacing={0.75}>
      {clock ? <Box data-testid="lth-execution-clock" sx={{ minWidth: 0 }}>
        <Typography variant="body2">실제 연구 실행 clock · {clock.version} · {clock.timezone} · IANA {clock.tzdbVersion}</Typography>
        <Typography variant="caption" sx={{ display: 'block' }}>종료 라벨 정책 {clock.labelPolicy} · 단일 cutoff/asOf {clock.asOf} · blackout 레거시 {clock.blackoutTimezone} · 창 {clock.blackoutWindows.join(', ') || '없음'}</Typography>
        <Typography variant="caption" sx={{ display: 'block' }}>계획 달력 {clock.calendarSource} · horizon {clock.calendarHorizon} · 전체 특별휴장 coverage 미검증</Typography>
        <Typography variant="caption" sx={{ display: 'block' }}>원본 공급/준비 {clock.sourceRows} · 실행 채택 {clock.adoptedRows} · 사전 일봉 {clock.warmupRows} · 달력 unknown {clock.unknownRows}</Typography>
        <Typography variant="caption" sx={{ display: 'block' }}>실제 제외 {clock.excluded.map((row) => `${row.reason} ${row.count}`).join(' · ') || '없음'}</Typography>
        <Typography variant="caption" sx={{ display: 'block' }}>실행 거래일 {clock.dayCount}개 · 표시 {Math.min(clock.days.length, compact ? 2 : 8)}개 · 생략 {clock.omittedDays + (compact ? Math.max(0, clock.days.length - 2) : 0)}개</Typography>
        {clock.days.slice(0, compact ? 2 : 8).map((day) => <Typography key={day.date} variant="caption" sx={{ display: 'block' }}>{day.date} · {day.open} → {day.close} · {day.earlyClose ? '단축장' : '정규장'} · 채택 {day.adoptedRows}</Typography>)}
        {!compact && clock.examples.map((row, index) => <Typography key={index} variant="caption" sx={{ display: 'block' }}>제외 원본 index {row.inputIndex} · {row.reason} · {row.label}{row.labelTruncated ? '… (원본 라벨 잘림)' : ''}</Typography>)}
        <Typography variant="caption" sx={{ display: 'block' }}>제외 예시 {compact ? 0 : clock.examples.length}개 · 생략 {clock.omittedExamples + (compact ? clock.examples.length : 0)}개 · inputHash는 params·비용·scope 포함</Typography>
      </Box> : <Alert severity="warning" data-testid="lth-execution-clock-unknown">실행 clock legacy/unknown · {value.executionClock ? '미지원 또는 손상된 실행 근거' : '과거 결과에 실행 근거 없음'} · 비교 가능 인증이 아닙니다.</Alert>}
      <Typography variant="body2" sx={{ fontWeight: 700 }}>분봉 시각·세션 진단 · {value.exchangeTimezone}</Typography>
      <Typography variant="caption" sx={{ display: "block" }}>
        {labelNames[value.labelConvention] ?? '봉 표기 미확인'} · UTC 오프셋(분) {value.marketUtcOffsetsMinutes.slice(0, 8).join(', ') || '미확인'}
        {' · '}{clock ? '별도 일반 평일 시간창 진단 · 실행 달력은 위 근거 참조' : '일반 평일 정규 시간창 · 실제 휴장/조기마감 달력 미확인'}
      </Typography>
      <Typography variant="caption" sx={{ display: "block" }}>
        공급 {value.sourceRows}행 · 선택 {value.selectedRows}행 · 실행 관측 {value.usedRows}행 · 가격 제외 {value.priceExcludedRows}행
        {' · '}진단 처리 {value.processedRows}/{value.inputLimit}행{value.inputTruncated ? ' · 진단 입력 잘림' : ''}
      </Typography>
      <Typography variant="caption" sx={{ display: "block" }}>
        시각 해석 {value.parsedRows} · 무효 {value.invalidRows} · KST 가정 {value.assumedKstRows}
        {' · '}중복 순간 {value.duplicateInstants} · 실제 순간 역순 {value.outOfOrderRows} · 분 경계 미정렬 {value.nonMinuteAlignedRows}
      </Typography>
      <Typography variant="caption" sx={{ display: "block" }}>
        정규 시간창 {value.regularRows} · 창 밖 {value.offSessionRows} · 주말 {value.weekendRows} · 기준 시각 뒤 종가 {value.unfinishedRows}
        {' · '}기준 시각 {value.referenceAsOf}
      </Typography>
      <Typography variant="caption" sx={{ display: "block" }}>
        내부 미관측 1분 슬롯 {value.interiorUnobservedSlots} · 최대 연속 공백 {value.maxInteriorGapSlots}슬롯
      </Typography>
      {!compact && <>
        <Typography variant="caption" sx={{ display: "block" }}>원본 라벨 현지 시각 {timeText(value.firstMarketLocal)} → {timeText(value.lastMarketLocal)}</Typography>
        <Typography variant="caption" sx={{ display: "block" }}>봉 시작 UTC {timeText(value.firstBarStartUtc)} → {timeText(value.lastBarStartUtc)}</Typography>
        <Typography variant="caption" sx={{ display: "block" }}>종가 공개 가능 UTC {timeText(value.firstCloseAvailableAtUtc)} → {timeText(value.lastCloseAvailableAtUtc)}</Typography>
      </>}
      <Typography variant="caption" sx={{ display: "block" }}>
        날짜 진단 {Math.min(value.days.length, compact ? 2 : 8)}/{value.observedDaysCount}개{value.omittedDayRows > 0 ? ` · 생략 ${value.omittedDayRows}개` : ''}
      </Typography>
      {value.days.slice(0, compact ? 2 : 8).map((day) => <Box key={day.marketDate} sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 0.75, minWidth: 0 }}>
        <Typography variant="caption" sx={{ fontWeight: 700, display: "block" }}>{day.marketDate} · UTC 오프셋 {day.utcOffsetMinutes}분</Typography>
        <Typography variant="caption" sx={{ display: "block" }}>정규창 고유 관측 {day.regularUniqueBars} / 일반 시간창 {day.ordinaryWindowExpectedSlots}슬롯</Typography>
        <Typography variant="caption" sx={{ display: "block" }}>종료 라벨 {timeText(day.firstCloseMarketLocal)} → {timeText(day.lastCloseMarketLocal)}</Typography>
        <Typography variant="caption" sx={{ display: "block" }}>내부 미관측 {day.interiorUnobservedSlots} · 최대 공백 {day.maxInteriorGapSlots} · 표본 앞 시간창 {day.leadingUnobservedWindowSlots} / 뒤 {day.trailingUnobservedWindowSlots}슬롯</Typography>
      </Box>)}
      {compact && value.days.length > 2 && <Typography variant="caption">저장된 날짜 중 처음 2개만 표시합니다.</Typography>}
      {!compact && value.examples.slice(0, 6).map((example, index) => <Typography key={`${example.kind}:${index}`} variant="caption" sx={{ display: "block" }}>진단 예 · {example.kind}: {example.label}</Typography>)}
      <Typography variant="caption" color="warning.main" sx={{ display: "block" }}>
        시간대 변환은 실제 거래일·자료 완전성·발행/수신 시각의 검증이 아닙니다. 휴장·조기마감·거래정지·무거래봉 생략과 공급원 누락을 구분할 수 없습니다.
        표본 앞뒤 시간창과 관측 없는 날짜를 실제 누락으로 확정하지 않으며 요청 기간 전체 coverage는 미확인입니다.
        {clock ? '연구 실행은 위 NY 정규장 clock을 적용합니다. 실제 공급원 raw/offset·발행 지연·일봉 regular-only OHLC·coverage·native/실운용은 미검증입니다.' : value.executionClock ? '실행 clock 정책은 미확인입니다. 과거 정책으로 추정하지 않습니다.' : '기존 고정 KST 전략 시계·신호·체결 계산은 유지합니다.'} 이 진단만으로 성과 평가나 실운용 검증을 완료하지 않습니다.
      </Typography>
      {!compact && value.limitations?.slice(0, 8).map((text, index) => <Typography key={index} variant="caption" sx={{ display: "block" }}>{text}</Typography>)}
    </Stack>
  </Box>
}
