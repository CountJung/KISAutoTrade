import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import type { LthCollectionSummary, LthCollectionStream } from '../../../shared/api/strategyResearchTypes'

const object = (v: unknown): v is Record<string, unknown> => !!v && typeof v === 'object' && !Array.isArray(v)
const count = (v: unknown): v is number => typeof v === 'number' && Number.isSafeInteger(v) && v >= 0
const hash = (v: unknown): v is string => typeof v === 'string' && /^[a-f0-9]{64}$/.test(v)
const date = (v: unknown): v is string => typeof v === 'string' && /^\d{8}$/.test(v) && Number.isFinite(Date.parse(`${v.slice(0, 4)}-${v.slice(4, 6)}-${v.slice(6, 8)}T00:00:00Z`)) && new Date(`${v.slice(0, 4)}-${v.slice(4, 6)}-${v.slice(6, 8)}T00:00:00Z`).toISOString().slice(0, 10).replace(/-/g, '') === v
const time = (v: unknown): v is string => typeof v === 'string' && v.length <= 64
  && /^\d{4}-\d\d-\d\dT(?:[01]\d|2[0-3]):[0-5]\d:[0-5]\d(?:\.\d{1,9})?(?:Z|[+-](?:[01]\d|2[0-3]):[0-5]\d)$/.test(v)
  && date(v.slice(0, 10).replace(/-/g, '')) && Number.isFinite(Date.parse(v))
function range(start: unknown, end: unknown, cutoff: string) {
  return time(start) && time(end) && Date.parse(start) <= Date.parse(end) && Date.parse(end) <= Date.parse(cutoff)
}
function stream(v: unknown, name: 'D' | '1m', cutoff: string, maxPages: number, maxRows: number): v is LthCollectionStream {
  if (!object(v) || !['dispatchedPages', 'returnedPages', 'validatedPages', 'returnedTypedRows', 'validatedTypedRows', 'duplicateRows', 'uniqueRows'].every((k) => count(v[k]))) return false
  const n = v as unknown as LthCollectionStream
  if (n.validatedPages > n.returnedPages || n.returnedPages > n.dispatchedPages || n.dispatchedPages > maxPages
    || n.validatedTypedRows > n.returnedTypedRows || n.returnedTypedRows > maxRows
    || n.uniqueRows + n.duplicateRows !== n.validatedTypedRows
    || !range(n.observedStart, n.observedEnd, cutoff) || !Array.isArray(n.sources) || n.sources.length > n.returnedPages) return false
  const indices = new Set<number>()
  return n.sources.every((s: unknown) => {
    if (!object(s) || s.stream !== name || !count(s.responseIndex) || s.responseIndex < 1 || s.responseIndex > n.returnedPages || indices.has(s.responseIndex)
      || !hash(s.sha256) || !count(s.byteLength) || !time(s.collectedAt)) return false
    indices.add(s.responseIndex); return true
  })
}
// Bounded display integrity checks only. Selection/calendar/admission remain backend-owned.
function valid(v: unknown): v is LthCollectionSummary {
  if (!object(v) || !time(v.asOf) || !(v.requestedStart === null || time(v.requestedStart) && Date.parse(v.requestedStart) <= Date.parse(v.asOf))
    || !['requestedCount', 'maxPages', 'maxTypedRows', 'pageSize', 'pageTimeoutSeconds', 'remainingBudgetMsAtStart'].every((k) => count(v[k]))
    || v.coverageVerified !== false || v.publicationVerification !== 'unknown' || v.dailyOhlcVerification !== 'unverified' || v.adjustmentVerification !== 'unverified'
    || !hash(v.normalizedInputSha256) || v.admissionStatus !== 'lastCompleted') return false
  const n = v as unknown as LthCollectionSummary
  if (n.requestedCount < 1 || n.maxPages < 1 || n.maxPages > 16 || n.maxTypedRows < 1 || n.maxTypedRows > 3200 || n.pageSize < 1 || n.pageSize > 200
    || n.pageTimeoutSeconds < 1 || n.pageTimeoutSeconds > 15 || n.remainingBudgetMsAtStart > 30000
    || !stream(n.daily, 'D', n.asOf, n.maxPages, n.maxTypedRows) || !stream(n.minute, '1m', n.asOf, n.maxPages, n.maxTypedRows)
    || n.daily.dispatchedPages + n.minute.dispatchedPages > n.maxPages || n.daily.returnedTypedRows + n.minute.returnedTypedRows > n.maxTypedRows) return false
  const a: unknown = n.admission
  if (!object(a) || !count(a.adoptedRows) || !count(a.usedDailyRows) || a.adoptedRows !== n.requestedCount || a.adoptedRows > n.minute.uniqueRows || a.usedDailyRows < 1 || a.usedDailyRows > n.daily.uniqueRows
    || !range(a.evaluationStart, a.evaluationEnd, n.asOf) || Date.parse(a.evaluationStart as string) < Date.parse(n.minute.observedStart!) || Date.parse(a.evaluationEnd as string) > Date.parse(n.minute.observedEnd!)
    || !date(a.dailyContextStart) || !date(a.dailyContextEnd) || a.dailyContextStart > a.dailyContextEnd || a.dailyContextEnd > n.asOf.slice(0, 10).replace(/-/g, '')
    || !object(a.excluded) || Object.keys(a.excluded).length > 32 || !Object.entries(a.excluded).every(([k, c]) => k.length <= 64 && count(c))) return false
  const sources = n.daily.sources.length + n.minute.sources.length
  const responses = n.daily.returnedPages + n.minute.returnedPages
  return n.sourceAvailability === (sources === 0 ? 'unavailable' : sources === responses ? 'complete' : 'partial')
}

export function LthCollectionSummaryPanel({ summary }: { summary: unknown }) {
  if (!valid(summary)) return <Alert severity="info">수집 상세 기록 확인 불가</Alert>
  const a = summary.admission!
  return <Box component="section" aria-label="수집 상세 기록" sx={{ minWidth: 0, overflowWrap: 'anywhere', border: 1, borderColor: 'divider', borderRadius: 1, p: 1 }}>
    <Stack spacing={0.75}>
      <Typography component="h3" variant="subtitle2">수집 상세 기록</Typography>
      <Typography variant="caption">요청 {summary.requestedCount}봉 · 요청 시작 {summary.requestedStart ?? '지정 안 됨'} · 예약 고정 asOf {summary.asOf}</Typography>
      <Typography variant="caption">실제 평가 UTC {a.evaluationStart} ~ {a.evaluationEnd} · 채택 {a.adoptedRows}봉</Typography>
      <Typography variant="caption">일봉 context 원래 join label {a.dailyContextStart} ~ {a.dailyContextEnd} · 사용 {a.usedDailyRows}봉</Typography>
      {(['daily', 'minute'] as const).map((key) => {
        const s = summary[key]
        return <Box key={key} sx={{ minWidth: 0 }}>
          <Typography component="h4" variant="caption">{key === 'daily' ? '일봉' : '분봉'} 관측 UTC {s.observedStart} ~ {s.observedEnd}</Typography>
          <Typography variant="caption" component="p">페이지 dispatch {s.dispatchedPages} / returned {s.returnedPages} / validated {s.validatedPages} · typed rows returned {s.returnedTypedRows} / validated {s.validatedTypedRows} / unique {s.uniqueRows} / dedup {s.duplicateRows}</Typography>
          {s.sources.map((source) => <Typography key={source.responseIndex} variant="caption" component="p" sx={{ overflowWrap: 'anywhere' }}>
            raw SHA {source.sha256} · byteLength {source.byteLength} · 수신시각 collectedAt {source.collectedAt} · stream {source.stream} / responseIndex {source.responseIndex}
          </Typography>)}
        </Box>
      })}
      <Typography variant="caption">제외사유 {Object.entries(a.excluded).map(([reason, n]) => `${reason}: ${n}`).join(', ') || '없음'} · collector dedup은 Prepared DuplicateInstant 제외와 별개입니다. typed rows는 HTTP 원본 행수가 아닙니다.</Typography>
      <Typography variant="caption">정규화 수집입력 SHA {summary.normalizedInputSha256} · raw 응답 지문 및 평가 cohort·성과·replay inputHash와 다른 지문입니다.</Typography>
      <Typography variant="caption">maxPages {summary.maxPages} · maxTypedRows {summary.maxTypedRows} · pageSize {summary.pageSize} · pageTimeoutSeconds {summary.pageTimeoutSeconds} · 수집 진입 시 남은 예산 {summary.remainingBudgetMsAtStart}ms</Typography>
      <Alert severity="info">source {summary.sourceAvailability}: 응답 metadata 기록 유무입니다. 출처 진위·PIT·기간 확보 인증이 아닙니다. coverageVerified=false · publication unknown · dailyOHLC/adjustment unverified. 요청 기간 전체 확보를 보장하지 않습니다.</Alert>
    </Stack>
  </Box>
}
