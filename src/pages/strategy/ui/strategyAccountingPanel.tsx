import { useEffect, useState } from 'react'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Grid from '@mui/material/Grid'
import MenuItem from '@mui/material/MenuItem'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import type { BacktestReport, DividendScenarioPoint, FxScenarioPoint, SimulationAssumptions } from '../../../api/types'
import { fmtRoundedKrw as money } from '../../../shared/lib/formatters'

const MAX_POINTS = 64
const MAX_DRAFT_LENGTH = 24_000

function scenarioRows(raw: string): Record<string, unknown>[] {
  if (raw.length > MAX_DRAFT_LENGTH) throw new Error('시나리오 입력은 24,000자 이하여야 합니다.')
  let parsed: unknown
  try { parsed = JSON.parse(raw) } catch { throw new Error('유효한 JSON 배열을 입력하세요. 비활성화하려면 []를 입력하세요.') }
  if (!Array.isArray(parsed) || parsed.length > MAX_POINTS) throw new Error('시나리오는 최대 64개 행의 JSON 배열이어야 합니다.')
  if (parsed.some((row) => row == null || typeof row !== 'object' || Array.isArray(row))) throw new Error('각 행은 필드 이름이 있는 객체여야 합니다.')
  return parsed as Record<string, unknown>[]
}

function validateLabels(rows: Record<string, unknown>[]) {
  const labels = new Set<string>()
  for (const row of rows) {
    if (typeof row.barTime !== 'string' || !row.barTime.trim() || row.barTime.length > 40) throw new Error('각 행에 40자 이하의 봉 라벨 barTime이 필요합니다.')
    const label = row.barTime.replace(/[-: ]/g, '').replace('T', '')
    if (labels.has(label)) throw new Error('같은 봉 라벨을 두 번 입력할 수 없습니다.')
    labels.add(label)
  }
}

function numberInRange(value: unknown, min: number, max: number) {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max
}

function parseScenarios(fxRaw: string, dividendRaw: string, isOverseas: boolean) {
  const fx = scenarioRows(fxRaw)
  const dividend = scenarioRows(dividendRaw)
  validateLabels(fx)
  validateLabels(dividend)
  if (!isOverseas && fx.length > 0) throw new Error('국내 종목에는 환율 시나리오를 적용할 수 없습니다. []로 비우세요.')
  if (fx.some((row) => Object.keys(row).some((key) => !['barTime', 'rateKrw'].includes(key)) || !numberInRange(row.rateKrw, 100, 10_000))) throw new Error('환율 행은 barTime과 100~10000 범위의 숫자 rateKrw를 입력하세요.')
  if (dividend.some((row) => Object.keys(row).some((key) => !['barTime', 'amountPerShare', 'withholdingTaxBps'].includes(key)) || !numberInRange(row.amountPerShare, 0, 100_000_000) || !numberInRange(row.withholdingTaxBps, 0, 10_000))) throw new Error('배당 행에는 barTime, 숫자 amountPerShare(0~100000000), withholdingTaxBps(0~10000)가 필요합니다.')
  return { fxScenario: fx as unknown as FxScenarioPoint[], dividendScenario: dividend as unknown as DividendScenarioPoint[] }
}

function validateCost(value: SimulationAssumptions) {
  const note = value.costNote ?? ''
  if ([...note].length > 120 || /[\u0000-\u001f\u007f-\u009f]/u.test(note)) return '비용 출처 메모는 제어문자 없이 120자 이하여야 합니다.'
  if (value.costSource === 'brokerVerifiedManual' && !note.trim()) return '수동 확인한 비용의 출처와 확인 내용을 메모에 입력하세요.'
  if ([value.buyFeeBps, value.sellFeeBps].some((bps) => bps != null && !numberInRange(bps, 0, 1000))) return '방향별 수수료는 0~1000bps 숫자 또는 빈칸이어야 합니다.'
  if (!numberInRange(value.minimumFeeNative ?? 0, 0, 100_000_000)) return '최소 수수료는 0~100000000 범위의 숫자여야 합니다.'
  return null
}

type EditorProps = {
  value: SimulationAssumptions
  onChange: (value: SimulationAssumptions) => void
  isOverseas: boolean
  disabled?: boolean
  onValidationChange: (error: string | null) => void
  onDraftChange: () => void
}

export function SimulationAccountingEditor({ value, onChange, isOverseas, disabled, onValidationChange, onDraftChange }: EditorProps) {
  const [fxDraft, setFxDraft] = useState(() => JSON.stringify(value.fxScenario ?? []))
  const [dividendDraft, setDividendDraft] = useState(() => JSON.stringify(value.dividendScenario ?? []))
  let error = validateCost(value)
  try { parseScenarios(fxDraft, dividendDraft, isOverseas); if (value.dividendPriceBasis === 'dividendAdjusted' && scenarioRows(dividendDraft).length > 0) throw new Error('배당 조정 가격에 현금 배당을 더하면 이중 계산됩니다. 가격 자료 선언 또는 배당 시나리오를 수정하세요.') } catch (caught) { error = (caught as Error).message }
  useEffect(() => { onValidationChange(error) }, [error, onValidationChange])

  const changeScenario = (kind: 'fx' | 'dividend', text: string) => {
    onDraftChange()
    if (kind === 'fx') setFxDraft(text)
    else setDividendDraft(text)
    try {
      const parsed = parseScenarios(kind === 'fx' ? text : fxDraft, kind === 'dividend' ? text : dividendDraft, isOverseas)
      onChange({ ...value, ...parsed })
    } catch {
      // Preserve the draft and prior valid assumptions, while the caller blocks
      // calculation and discards its old preview. Invalid JSON never becomes [].
    }
  }
  const changeFee = (key: 'buyFeeBps' | 'sellFeeBps' | 'minimumFeeNative', text: string) => {
    onDraftChange()
    onChange({ ...value, [key]: text === '' && key !== 'minimumFeeNative' ? null : text === '' ? Number.NaN : Number(text) })
  }
  return (
    <Stack spacing={1.25} sx={{ mt: 1.5, pt: 1.5, borderTop: 1, borderColor: 'divider', minWidth: 0, overflowWrap: 'anywhere' }} data-testid="simulation-accounting-editor">
      <Typography variant="caption" fontWeight={700}>비용 출처와 현금흐름 연구 가정</Typography>
      <Grid container spacing={1.25}>
        <Grid item xs={12} sm={6}>
          <TextField select fullWidth size="small" label="비용 출처" value={value.costSource ?? 'estimated'} disabled={disabled} onChange={(event) => onChange({ ...value, costSource: event.target.value as SimulationAssumptions['costSource'] })}>
            <MenuItem value="estimated">추정 비용</MenuItem>
            <MenuItem value="brokerVerifiedManual">브로커 조건 사용자 수동 확인</MenuItem>
          </TextField>
        </Grid>
        <Grid item xs={12} sm={6}>
          <TextField fullWidth size="small" label="비용 출처 메모" value={value.costNote ?? ''} disabled={disabled} onChange={(event) => onChange({ ...value, costNote: event.target.value })} helperText="수동 확인 시 필수 · 최대 120자" />
        </Grid>
        <Grid item xs={12} sm={4}>
          <TextField fullWidth size="small" type="number" label="매수 수수료(bps)" value={value.buyFeeBps ?? ''} disabled={disabled} onChange={(event) => changeFee('buyFeeBps', event.target.value)} helperText="빈칸 = 공통 수수료 사용" inputProps={{ min: 0, max: 1000, step: 0.1 }} />
        </Grid>
        <Grid item xs={12} sm={4}>
          <TextField fullWidth size="small" type="number" label="매도 수수료(bps)" value={value.sellFeeBps ?? ''} disabled={disabled} onChange={(event) => changeFee('sellFeeBps', event.target.value)} helperText="빈칸 = 공통 수수료 사용" inputProps={{ min: 0, max: 1000, step: 0.1 }} />
        </Grid>
        <Grid item xs={12} sm={4}>
          <TextField fullWidth size="small" type="number" label={`체결당 최소 수수료(${isOverseas ? 'USD' : '원'})`} value={Number.isNaN(value.minimumFeeNative) ? '' : value.minimumFeeNative ?? 0} disabled={disabled} onChange={(event) => changeFee('minimumFeeNative', event.target.value)} inputProps={{ min: 0, max: 100_000_000, step: isOverseas ? 0.01 : 1 }} />
        </Grid>
      </Grid>
      <Alert severity="info" sx={{ py: 0.5 }}>
        수동 확인은 사용자가 확인한 조건을 기록하는 기능이며 앱이 브로커 API로 검증한 값은 아닙니다. 계좌별 할인·실제 최소 수수료·거래세를 자동 반영하지 않습니다.
        {' '}기본 환율은 고정 시나리오이고 유휴 현금은 원화로 유지합니다. 환전 수수료·해외 양도소득세는 별도입니다.
      </Alert>
      <TextField multiline minRows={2} maxRows={7} fullWidth size="small" label="환율 시나리오 JSON" value={fxDraft} disabled={disabled} onChange={(event) => changeScenario('fx', event.target.value)} inputProps={{ maxLength: MAX_DRAFT_LENGTH + 1 }} helperText={'최대 64개 · 예: [{"barTime":"20260702","rateKrw":1400}] · 국내 종목은 []'} />
      <TextField select fullWidth size="small" label="가격 자료의 배당 반영" value={value.dividendPriceBasis ?? 'unverified'} disabled={disabled} onChange={(event) => onChange({ ...value, dividendPriceBasis: event.target.value as SimulationAssumptions['dividendPriceBasis'] })}>
        <MenuItem value="unverified">배당 조정 여부 미확인</MenuItem>
        <MenuItem value="unadjusted">배당 미조정 가격 · 사용자 선언</MenuItem>
        <MenuItem value="dividendAdjusted">배당 조정 가격 · 사용자 선언</MenuItem>
      </TextField>
      <TextField multiline minRows={2} maxRows={7} fullWidth size="small" label="배당 시나리오 JSON" value={dividendDraft} disabled={disabled} onChange={(event) => changeScenario('dividend', event.target.value)} inputProps={{ maxLength: MAX_DRAFT_LENGTH + 1 }} helperText={'최대 64개 · 예: [{"barTime":"20260703","amountPerShare":1,"withholdingTaxBps":1500}]'} />
      <Typography variant="caption" color="text.secondary" sx={{ overflowWrap: 'anywhere' }}>
        평가 구간에 실제 존재하는 봉 라벨만 허용합니다. 환율은 지정 봉의 시작부터 이후 봉으로 이어지며 과거로 소급하지 않습니다.
        배당은 지정 봉 시가 직전 보유 수량에 주당 금액(국내 원·해외 USD)을 배분하고 같은 봉에 원천징수 후 원화 현금으로 입금하는 연구 가정입니다. 실제 배당락·지급일·기업행동 자료와 다릅니다.
      </Typography>
      {(value.dividendPriceBasis ?? 'unverified') === 'unverified' && <Alert severity="warning" sx={{ py: 0.5 }}>가격 자료의 배당 조정 여부가 확인되지 않았습니다. 현금 배당 시나리오와 가격 자료가 중복되지 않는지 확인하세요.</Alert>}
      {error && <Alert severity="error" data-testid="simulation-accounting-error">{error} 계산 전에 입력을 수정하세요.</Alert>}
    </Stack>
  )
}

function percent(value: number) { return `${value.toFixed(2)}%` }

export function accountingAssumptionsLabel(value: SimulationAssumptions) {
  const source = value.costSource === 'brokerVerifiedManual' ? '사용자 수동 확인' : value.costSource === 'estimated' ? '추정' : '출처 정보 없음'
  return `${source} · 매수 ${value.buyFeeBps ?? `공통 ${value.feeBps}`}bps · 매도 ${value.sellFeeBps ?? `공통 ${value.feeBps}`}bps · 최소 ${value.minimumFeeNative ?? '정보 없음'} (종목 통화) · 세금 ${value.taxBps}bps · 슬리피지 ${value.slippageBps}bps · 환율 ${value.exchangeRateKrw} · FX ${value.fxScenario?.length ?? '정보 없음'}점 · 배당 ${value.dividendScenario?.length ?? '정보 없음'}점 · 배당 가격 ${value.dividendPriceBasis === 'unadjusted' ? '미조정 선언' : value.dividendPriceBasis === 'dividendAdjusted' ? '조정 선언' : '미확인'}${value.costNote ? ` · ${value.costNote}` : ''}`
}

export function accountingComparisonKey(value: SimulationAssumptions) {
  return JSON.stringify([value.costSource, value.costNote, value.feeBps, value.buyFeeBps, value.sellFeeBps, value.minimumFeeNative, value.taxBps, value.slippageBps, value.exchangeRateKrw, value.fxScenario, value.dividendScenario, value.dividendPriceBasis])
}

export function AccountingAssumptionSummary({ value }: { value: SimulationAssumptions }) {
  return <Box sx={{ minWidth: 0, overflowWrap: 'anywhere' }}>
    <Typography variant="caption" display="block">{accountingAssumptionsLabel(value)}</Typography>
    {((value.fxScenario?.length ?? 0) > 0 || (value.dividendScenario?.length ?? 0) > 0) && <details><summary>저장된 환율·배당 가정 보기</summary><Typography component="pre" variant="caption" sx={{ whiteSpace: 'pre-wrap', overflowWrap: 'anywhere' }}>{JSON.stringify({ fxScenario: value.fxScenario, dividendScenario: value.dividendScenario }, null, 2)}</Typography></details>}
  </Box>
}

export function AccountingResults({ report, compact = false }: { report: BacktestReport; compact?: boolean }) {
  const accounting = report.accounting
  if (!accounting) return <Alert severity="warning" data-testid="backtest-accounting-unknown">이전 결과에는 손익 분해 정보가 없습니다. 실현·잔여 평가손익, 비용과 배당은 0으로 추정하지 않습니다. 다시 계산하세요.</Alert>
  if (compact) return <Typography variant="caption" display="block">실현 {money(accounting.realizedPnlKrw)} · 잔여 평가 {money(accounting.residualUnrealizedPnlKrw)} · 순배당 {money(accounting.dividendNetKrw)}</Typography>
  const cells: [string, string][] = [
    ['실현손익 · 매도 확정', money(accounting.realizedPnlKrw)],
    ['잔여 평가손익 · 매도 전', money(accounting.residualUnrealizedPnlKrw)],
    ['잔여 수량', accounting.remainingQuantity.toLocaleString('ko-KR')],
    ['잔여 매수 원가', money(accounting.remainingCostBasisKrw)],
    ['잔여 매수 수수료', money(accounting.remainingBuyFeeKrw)],
    ['실현 가격 효과', money(accounting.realizedPricePnlKrw)],
    ['실현 환율 효과', money(accounting.realizedFxPnlKrw)],
    ['잔여 가격 효과', money(accounting.residualPricePnlKrw)],
    ['잔여 환율 효과', money(accounting.residualFxPnlKrw)],
    ['체결 수수료 합계', money(accounting.totalFeeKrw)],
    ['체결 거래세 합계', money(accounting.totalTaxKrw)],
    ['체결 슬리피지 영향', money(accounting.totalSlippageKrw)],
    ['배당 총액', money(accounting.dividendGrossKrw)],
    ['배당 원천징수', money(accounting.dividendTaxKrw)],
    ['배당 순현금', money(accounting.dividendNetKrw)],
  ]
  return (
    <Stack spacing={1.25} data-testid="backtest-accounting-results" sx={{ minWidth: 0 }}>
      <Typography variant="subtitle2" fontWeight={700}>손익과 현금흐름 분해</Typography>
      <Typography variant="caption" sx={{ overflowWrap: 'anywhere' }}>{accountingAssumptionsLabel(report.assumptions)}</Typography>
      <Alert severity={accounting.realizedEquitySignMismatch ? 'warning' : 'info'} data-testid="backtest-accounting-sample">
        완료 거래 표본 {report.summary.completedTrades}회 · 보유 관측 봉 비율 {percent(report.summary.exposurePct)}
        {accounting.realizedEquitySignMismatch && ' · 실현손익과 전체 자산 수익률의 부호가 다릅니다. 잔여 평가손익과 배당을 함께 확인하세요.'}
      </Alert>
      <Typography variant="caption">전체 자산 손익 = 실현손익 + 잔여 평가손익 + 순배당입니다. 가격 효과는 진입 환율로 평가하고, 환율 효과는 종료 가격에 환율 변화를 적용합니다. 실현·잔여 순손익에는 해당 수수료와 세금 배분이 포함됩니다.</Typography>
      <Grid container spacing={1}>
        {cells.map(([label, value]) => <Grid key={label} item xs={12} sm={6} md={4}><Paper variant="outlined" sx={{ p: 1 }}><Typography variant="caption" color="text.secondary" display="block">{label}</Typography><Typography variant="body2">{value}</Typography></Paper></Grid>)}
      </Grid>
      <Box sx={{ p: 1.25, border: 1, borderColor: 'divider', borderRadius: 1 }} data-testid="backtest-capital-observations">
        <Typography variant="body2">봉말 평균 보유 평가액 {money(accounting.averageInvestedNotionalKrw)} · 초기자본 대비 {percent(accounting.averageInvestedCapitalPct)} · 당시 자산 대비 {percent(accounting.averageInvestedEquityPct)}</Typography>
        <Typography variant="body2">최대 투자 비중 · 초기자본 대비 {percent(accounting.peakInvestedCapitalPct)} · 당시 자산 대비 {percent(accounting.peakInvestedEquityPct)}</Typography>
        <Typography variant="caption">봉마다 관측한 값의 단순 평균입니다. 실제 보유 시간으로 가중한 수익률이나 연환산 수익률이 아닙니다.</Typography>
      </Box>
      <Alert severity="info" data-testid="backtest-terminal-liquidation">
        <Typography variant="body2" fontWeight={700}>종료 시 가상 청산 · 실제 거래에 미포함</Typography>
        <Typography variant="body2">잔여분을 종료 시점에 매도한다고 가정한 자산 {money(accounting.terminalLiquidationEquityKrw)} · 수익률 {percent(accounting.terminalLiquidationReturnPct)}</Typography>
        <Typography variant="body2">추가 매도 수수료 {money(accounting.terminalExitFeeKrw)} · 거래세 {money(accounting.terminalExitTaxKrw)} · 슬리피지 {money(accounting.terminalExitSlippageKrw)}</Typography>
        <Typography variant="caption">위 실현손익·실제 체결 수·현금흐름에는 가상 청산을 더하지 않습니다. 슬리피지는 가격에 포함된 영향이며 비용을 다시 차감하지 않습니다.</Typography>
      </Alert>
      {accounting.warnings.map((warning, index) => <Alert severity="warning" key={`${index}:${warning}`} sx={{ py: 0.5 }}>{warning}</Alert>)}
    </Stack>
  )
}


export function ComparisonResults({ report }: { report: BacktestReport }) {
  const comparison = report.comparison
  if (!comparison) return <Alert severity="warning" data-testid="backtest-comparison-unknown">이전 결과에는 단순 보유·동일 체결 경로 비용 비교가 없습니다. 값을 추정하지 않습니다.</Alert>
  const hold = comparison.sameInputQuantityBuyHold
  const reference = comparison.sameExecutionPathCostReference
  return <Stack spacing={1.25} data-testid="backtest-accounting-comparison" sx={{ minWidth: 0 }}>
    <Typography variant="subtitle2" fontWeight={700}>단순 보유와 비용 영향 참고 비교</Typography>
    <Paper variant="outlined" sx={{ p: 1.25 }}>
      <Typography variant="body2" fontWeight={700}>동일 입력 수량의 단순 보유</Typography>
      <Typography variant="body2">요청 {hold.requestedQuantity}주 · 실제 진입 {hold.actualEntryQuantity}주 · 종료 보유 {hold.remainingQuantity}주</Typography>
      <Typography variant="body2">전체 자산 {money(hold.finalEquityKrw)} · 수익률 {percent(hold.cumulativeReturnPct)} · MDD {percent(hold.mddPct)} · 종료 가상 청산 수익률 {percent(hold.terminalLiquidationReturnPct)}</Typography>
      <Typography variant="body2">실현 {money(hold.realizedPnlKrw)} · 잔여 평가 {money(hold.residualUnrealizedPnlKrw)} · 순배당 {money(hold.dividendNetKrw)}</Typography>
      <Typography variant="body2">관측 평균 투자 비중 · 초기자본 대비 {percent(hold.averageInvestedCapitalPct)} · 당시 자산 대비 {percent(hold.averageInvestedEquityPct)} · 보유 관측 봉 {percent(hold.holdingBarPct)}</Typography>
      <Typography variant="body2">최대 투자 비중 · 초기자본 대비 {hold.peakInvestedCapitalPct == null ? '정보 없음' : percent(hold.peakInvestedCapitalPct)} · 당시 자산 대비 {hold.peakInvestedEquityPct == null ? '정보 없음' : percent(hold.peakInvestedEquityPct)}</Typography>
      <Typography variant="body2">체결 {hold.filledOrderCount}개 · 차단 {hold.blockedOrderCount}개 · 미체결 {hold.unfilledOrderCount}개</Typography>
      <Typography variant="caption">첫 평가 봉 종가에서 매수를 지시하고 선택한 체결 모델을 적용한 모의 보유입니다. 연구 손절·익절은 적용하지 않습니다. 입력 수량이 같아도 실제 진입 수량과 투자 비중은 다를 수 있습니다.</Typography>
    </Paper>
    <Typography variant="body2">전략 − 단순 보유 · 평균 초기자본 투자 비중 차이 {percent(comparison.capitalExposureDifferencePct)}p · 보유 관측 봉 차이 {percent(comparison.holdingBarDifferencePct)}p</Typography>
    <Alert severity="info">
      <Typography variant="body2" fontWeight={700}>동일 체결 경로의 비용 제거 참고값</Typography>
      <Typography variant="body2">복원 비용 영향 {money(reference.restoredCostKrw)} · 참고 자산 {money(reference.finalEquityKrw)} · 참고 수익률 {percent(reference.cumulativeReturnPct)}</Typography>
      <Typography variant="caption">실제 계산에서 체결한 수량·시점·노출을 고정하고 발생 당시 환율로 기록한 원화 비용을 산술적으로 복원합니다. 원가나 환율 기여를 다시 분해하지 않습니다. 비용이 없는 전략을 다시 실행한 성과나 실제 거래 결과가 아닙니다.</Typography>
    </Alert>
    <Typography variant="caption">같거나 겹친 기간의 비교는 독립 검증이 아닙니다. 부분기간의 결과를 전체 분기·연속 운용 성과로 합산하지 마세요.</Typography>
    {comparison.limitations.map((limitation, index) => <Typography key={`${index}:${limitation}`} variant="caption">{limitation}</Typography>)}
  </Stack>
}
