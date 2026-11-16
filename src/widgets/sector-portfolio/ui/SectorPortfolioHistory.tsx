import { useState } from 'react'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Typography from '@mui/material/Typography'
import { useTheme } from '@mui/material/styles'
import type { SectorPortfolioResult } from '../../../shared/api/sectorPortfolioTypes'

const amount = (value: number) => Number.isFinite(value) ? value.toLocaleString('en-US', { maximumFractionDigits: 2, minimumFractionDigits: 2 }) : '미확인'
const PAGE_SIZE = 25

/** Bounded pure result view: never fetches account state or issues financial actions. */
export function SectorPortfolioHistory({ portfolio }: { portfolio: SectorPortfolioResult }) {
  const theme = useTheme()
  const [dailyPage, setDailyPage] = useState(0)
  const [tradePage, setTradePage] = useState(0)
  const values = [portfolio.summary.initialCashUsd, ...portfolio.daily.map(day => day.equityUsd)]
  const finite = values.every(Number.isFinite)
  const low = Math.min(...values); const high = Math.max(...values)
  const span = high - low || Math.max(Math.abs(high) * 0.01, 1)
  const points = values.map((value, index) => `${40 + index / Math.max(values.length - 1, 1) * 680},${150 - (value - low) / span * 120}`).join(' ')
  function navigation(page: number, length: number, setPage: (value: number) => void, label: string) {
    return <Stack direction="row" spacing={1} sx={{ alignItems: 'center' }}>
      <Button size="small" disabled={page === 0} onClick={() => setPage(page - 1)} aria-label={`${label} 이전 페이지`}>이전</Button>
      <Typography variant="caption">{page + 1} / {Math.max(1, Math.ceil(length / PAGE_SIZE))} · {length}건</Typography>
      <Button size="small" disabled={(page + 1) * PAGE_SIZE >= length} onClick={() => setPage(page + 1)} aria-label={`${label} 다음 페이지`}>다음</Button>
    </Stack>
  }
  return <Stack spacing={1.5} data-testid="sector-portfolio-history">
    <Typography variant="subtitle2">선택 전략 자산곡선 · USD</Typography>
    {finite && <Box component="svg" role="img" aria-label="초기자금과 일별 종가 평가자산 곡선" viewBox="0 0 760 190" sx={{ width: '100%', maxHeight: 260, color: 'text.secondary' }}>
      <title>초기자금과 선택 전략의 일별 종가 평가자산. 거래일 간격으로 표시합니다.</title>
      <line x1="40" y1="150" x2="720" y2="150" stroke={theme.palette.divider} />
      <polyline points={points} fill="none" stroke={theme.palette.primary.main} strokeWidth="2" />
      <text x="40" y="18" fill="currentColor" fontSize="12">최대 {amount(high)} / 최소 {amount(low)} USD</text>
      <text x="40" y="178" fill="currentColor" fontSize="12">초기자금</text>
      <text x="720" y="178" textAnchor="end" fill="currentColor" fontSize="12">{portfolio.daily[portfolio.daily.length - 1]?.date}</text>
    </Box>}
    <Typography variant="caption" color="text.secondary">거래일 등간격·자동 축척입니다. 곡선은 초기자금과 종가 평가액이며 장중 낙폭·실제 체결을 보여주지 않습니다. 상세 수치는 아래 표에서 확인하세요.</Typography>
    <Box component="details">
      <Box component="summary" sx={{ cursor: 'pointer' }}>일별 평가내역</Box>
      {navigation(dailyPage, portfolio.daily.length, setDailyPage, '일별 평가')}
      <TableContainer tabIndex={0} aria-label="선택 전략 일별 평가내역">
        <Table size="small" sx={{ minWidth: 650 }}>
          <TableHead><TableRow>{['날짜', '자산 USD', '누적손익 USD', '현금 USD', '일수익률', '낙폭', '투자비중'].map(label => <TableCell key={label}>{label}</TableCell>)}</TableRow></TableHead>
          <TableBody>{portfolio.daily.slice(dailyPage * PAGE_SIZE, (dailyPage + 1) * PAGE_SIZE).map(day => <TableRow key={day.date}>
            <TableCell>{day.date}</TableCell><TableCell>{amount(day.equityUsd)}</TableCell><TableCell>{amount(day.equityUsd - portfolio.summary.initialCashUsd)}</TableCell>
            <TableCell>{amount(day.cashUsd)}</TableCell><TableCell>{amount(day.dailyReturnPct)}%</TableCell><TableCell>{amount(day.drawdownPct)}%</TableCell><TableCell>{amount(day.exposurePct)}%</TableCell>
          </TableRow>)}</TableBody>
        </Table>
      </TableContainer>
    </Box>
    <Box component="details">
      <Box component="summary" sx={{ cursor: 'pointer' }}>가상 체결내역</Box>
      {portfolio.trades.length === 0 ? <Typography variant="body2">이 기간에 기록된 가상 체결이 없습니다.</Typography> : <>
        {navigation(tradePage, portfolio.trades.length, setTradePage, '가상 체결')}
        <TableContainer tabIndex={0} aria-label="선택 전략 가상 체결내역">
          <Table size="small" sx={{ minWidth: 700 }}>
            <TableHead><TableRow>{['날짜', '종목', '구분', '요청/체결수량', '가격 USD', '비용 USD', '체결 후 현금 USD', '상태'].map(label => <TableCell key={label}>{label}</TableCell>)}</TableRow></TableHead>
            <TableBody>{portfolio.trades.slice(tradePage * PAGE_SIZE, (tradePage + 1) * PAGE_SIZE).map((trade, index) => <TableRow key={`${tradePage}-${index}`}>
              <TableCell>{trade.date}</TableCell><TableCell>{trade.symbol}</TableCell><TableCell>{trade.side}</TableCell><TableCell>{trade.requestedQuantity}/{trade.quantity}</TableCell>
              <TableCell>{amount(trade.priceUsd)}</TableCell><TableCell>{amount(trade.costUsd)}</TableCell><TableCell>{amount(trade.cashAfterUsd)}</TableCell><TableCell>{trade.status}</TableCell>
            </TableRow>)}</TableBody>
          </Table>
        </TableContainer>
      </>}
    </Box>
  </Stack>
}
