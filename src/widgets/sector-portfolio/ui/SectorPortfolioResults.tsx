import { useState } from 'react'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import MenuItem from '@mui/material/MenuItem'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import type { SectorPortfolioInput, SectorPortfolioReport, SectorStrategy } from '../../../shared/api/sectorPortfolioTypes'

export const sectorStrategyLabel: Record<SectorStrategy, string> = {
  base: '기본형', defensive: '방어형', aggressive: '공격형', spy: 'SPY 보유', cash: '현금',
}
const number = (value: number) => Number.isFinite(value) ? value.toLocaleString('en-US', { maximumFractionDigits: 2, minimumFractionDigits: 2 }) : '미확인'
const pct = (value: number) => `${number(value)}%`

export function SectorPortfolioResults({ report, input }: { report: SectorPortfolioReport; input: SectorPortfolioInput }) {
  const [selected, setSelected] = useState<SectorStrategy>('base')
  const portfolio = report.portfolios.find(item => item.strategy === selected)
  const last = portfolio?.daily[portfolio.daily.length - 1]
  const prices = input.evaluation.find(day => day.date === last?.date)?.prices
  const ranking = report.rankings[report.rankings.length - 1]
  return (
    <Stack spacing={2} data-testid="sector-portfolio-results">
      <Alert severity={report.source.synthetic ? 'warning' : 'info'}>
        {report.source.synthetic ? '합성 자료 결과 · 실제 투자 성과가 아닙니다.' : '입력한 과거 자료의 재생 결과 · 공급원 검증은 입력 선언과 구분합니다.'}
      </Alert>
      <Typography variant="body2">
        평가 {report.startDate} ~ {report.endDate} · {report.evaluationBars}거래일 · 사전자료 {report.historyBars}봉 · 체결 모델 {report.executionModel}
      </Typography>
      <Typography variant="body2" sx={{ overflowWrap: 'anywhere' }}>
        출처 {report.source.name} · 수집 {report.source.retrievedAt} · 달력 {report.source.calendarSource}<br />
        조정 기준 {report.source.adjustmentPolicy} · coverage 선언 {report.source.coverageVerified ? '확인' : '미확인'} · 기업행위 선언 {report.source.corporateActionsVerified ? '확인' : '미확인'}<br />
        엔진 {report.engineVersion} · 입력 SHA-256 {report.inputHash}
      </Typography>
      <TableContainer component={Box} tabIndex={0} aria-label="세 전략과 비교군 성과표" sx={{ maxWidth: '100%' }}>
        <Table size="small" sx={{ minWidth: 1000 }}>
          <TableHead><TableRow>{['전략', '기말자산 USD', '순수익률', 'MDD', '최대 일손실', '일 변동성', '비용 USD', '평균 투자비중', '회전율', '체결 / 현금제한', '현금 USD'].map(label => <TableCell key={label}>{label}</TableCell>)}</TableRow></TableHead>
          <TableBody>{report.portfolios.map(({ strategy, summary: s }) => (
            <TableRow key={strategy}>
              <TableCell component="th" scope="row">{sectorStrategyLabel[strategy]}</TableCell>
              <TableCell>{number(s.endingEquityUsd)}</TableCell><TableCell>{pct(s.netReturnPct)}</TableCell>
              <TableCell>{pct(s.maxDrawdownPct)}</TableCell><TableCell>{pct(s.maxDailyLossPct)}</TableCell>
              <TableCell>{pct(s.dailyVolatilityPct)}</TableCell><TableCell>{number(s.totalCostUsd)}</TableCell>
              <TableCell>{pct(s.averageExposurePct)}</TableCell><TableCell>{pct(s.turnoverPct)}</TableCell>
              <TableCell>{s.filledTradeCount} / {s.cashLimitedCount}</TableCell><TableCell>{number(s.endingCashUsd)}</TableCell>
            </TableRow>
          ))}</TableBody>
        </Table>
      </TableContainer>
      <TextField select label="상세 비교 전략" value={selected} onChange={event => setSelected(event.target.value as SectorStrategy)} size="small" sx={{ maxWidth: 240 }}>
        {report.portfolios.map(item => <MenuItem key={item.strategy} value={item.strategy}>{sectorStrategyLabel[item.strategy]}</MenuItem>)}
      </TextField>
      {portfolio && last && <>
        <Typography variant="subtitle2">마지막 평가일 {last.date}의 계획과 보유</Typography>
        <Typography variant="body2">
          전일 신호 {last.plan.signalDate} · 상태 {last.plan.defensiveState === 'defensive' ? '방어' : '보통'} · 선정 {last.plan.selectedSymbols.join(', ') || '없음'} · 리밸런싱 {last.plan.rebalance ? '실행' : '유지'}<br />
          실현손익 {number(portfolio.summary.realizedPnlUsd)} USD · 잔여평가손익 {number(portfolio.summary.unrealizedPnlUsd)} USD<br />
          같은 체결 경로의 비용 환급 산술수익률 {pct(portfolio.summary.samePathReturnBeforeCostsPct)} (무비용 재실행과 다름)
        </Typography>
        <Typography variant="caption" color="text.secondary">사유: {last.plan.reasons.join(' · ') || '리밸런싱 조건 없음'}. 계획 비중은 전일 기준, 실제 비중은 평가일 종가 기준입니다.</Typography>
        <TableContainer component={Box} tabIndex={0} aria-label="섹터 계획과 실제 보유" sx={{ maxWidth: '100%' }}>
          <Table size="small" sx={{ minWidth: 620 }}>
            <TableHead><TableRow>{['종목', '목표 비중', '전일 실제 비중', '기말 수량', '기말 실제 비중', '평균 원가 USD'].map(label => <TableCell key={label}>{label}</TableCell>)}</TableRow></TableHead>
            <TableBody>{Array.from(new Set([...last.plan.targets.map(item => item.symbol), ...last.holdings.map(item => item.symbol)])).map(symbol => {
              const target = last.plan.targets.find(item => item.symbol === symbol)
              const holding = last.holdings.find(item => item.symbol === symbol)
              const close = prices?.[symbol]?.close
              return <TableRow key={symbol}>
                <TableCell component="th" scope="row">{symbol}</TableCell><TableCell>{pct((target?.weight ?? 0) * 100)}</TableCell>
                <TableCell>{pct((target?.referenceWeight ?? 0) * 100)}</TableCell><TableCell>{holding?.quantity ?? 0}</TableCell>
                <TableCell>{close == null ? '미확인' : pct((holding?.quantity ?? 0) * close / last.equityUsd * 100)}</TableCell>
                <TableCell>{holding ? number(holding.averageCostUsd) : '—'}</TableCell>
              </TableRow>
            })}</TableBody>
          </Table>
        </TableContainer>
      </>}
      {ranking && <>
        <Typography variant="subtitle2">마지막 선정 순위 · 신호 {ranking.signalDate}</Typography>
        <TableContainer component={Box} tabIndex={0} aria-label="섹터 상대강도 순위" sx={{ maxWidth: '100%' }}>
          <Table size="small" sx={{ minWidth: 560 }}>
            <TableHead><TableRow>{['순위', '종목', 'R20', 'RS5', 'RS20', '종합점수', '적격'].map(label => <TableCell key={label}>{label}</TableCell>)}</TableRow></TableHead>
            <TableBody>{ranking.rankings.map((item, index) => <TableRow key={item.symbol}>
              <TableCell>{index + 1}</TableCell><TableCell component="th" scope="row">{item.symbol}</TableCell>
              <TableCell>{pct(item.r20 * 100)}</TableCell><TableCell>{pct(item.rs5 * 100)}</TableCell><TableCell>{pct(item.rs20 * 100)}</TableCell>
              <TableCell>{number(item.score)}</TableCell><TableCell>{item.eligible ? '적격' : '제외'}</TableCell>
            </TableRow>)}</TableBody>
          </Table>
        </TableContainer>
      </>}
      {report.warnings.length > 0 && <Alert severity="warning"><Box component="ul" sx={{ m: 0, pl: 2 }}>{report.warnings.map((warning, index) => <li key={`${index}-${warning}`}>{warning}</li>)}</Box></Alert>}
    </Stack>
  )
}
