import { useRef, useState } from 'react'
import type { Time } from 'lightweight-charts'
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

interface CandlePoint { time: Time; open: number; high: number; low: number; close: number }
interface Projection { candleData: CandlePoint[]; volumeData: Array<{ time: Time; value: number }> }
interface Props {
  points: Projection
  symbol: string
  stockName?: string
  context: string
  currency?: 'KRW' | 'USD'
  validSymbol: boolean
  isLoading: boolean
  isError: boolean
  isFetching: boolean
  dataUpdatedAt: number
  formatTime: (time: Time) => string
}
const number = (value: number) => value.toLocaleString('ko-KR', { maximumFractionDigits: 20 })
const focusStyle = { '&:focus-visible': { outline: '2px solid', outlineColor: 'primary.main', outlineOffset: 2 } }

// Display only the exact projection sent to the canvas; no parsing, price conversion or new query.
export function CandleDataSummary({ points, symbol, stockName, context, currency, validSymbol,
  isLoading, isError, isFetching, dataUpdatedAt, formatTime }: Props) {
  const [selection, setSelection] = useState<{ snapshot: CandlePoint[]; index: number } | null>(null)
  const buttons = useRef<Array<HTMLButtonElement | null>>([])
  const offset = Math.max(0, points.candleData.length - 20)
  const recent = points.candleData.slice(offset)
  const latest = recent[recent.length - 1]
  const selectedIndex = selection?.snapshot === points.candleData ? selection.index : -1
  const selected = recent[selectedIndex]
  const unit = currency ?? '통화 미확인'
  const status = !validSymbol ? '종목 선택 전' : isError ? latest ? '재조회 실패 · 이전 확인 자료' : '차트 조회 실패'
    : isLoading ? '차트 자료를 불러오는 중…' : isFetching ? '갱신 중 · 이전 확인 자료'
      : latest ? '차트 자료 확인 완료' : '표시할 유효한 봉이 없습니다'
  const time = (value: Time) => (
    <time dateTime={typeof value === 'number' ? new Date(value * 1000).toISOString() : typeof value === 'string' ? value : undefined}>
      {formatTime(value)}
    </time>
  )
  const values = (point: CandlePoint, volume: number) => (
    <>시가 {number(point.open)} · 고가 {number(point.high)} · 저가 {number(point.low)} · 종가 {number(point.close)} · 거래량 {number(volume)}</>
  )
  const select = (index: number, focus = false) => {
    setSelection({ snapshot: points.candleData, index })
    if (focus) buttons.current[index]?.focus()
  }

  return (
    <Box component="section" aria-label={`${symbol || '미선택'} 차트 자료`} aria-live="off"
      data-testid="candle-data-summary" sx={{ mt: 2, p: 1.5, border: 1, borderColor: 'divider', borderRadius: 1, minWidth: 0, overflowWrap: 'anywhere' }}>
      <Stack spacing={1}>
        <Typography component="h3" variant="subtitle2">차트 자료 · {stockName ? `${stockName} (${symbol})` : symbol || '미선택'}</Typography>
        <Typography variant="body2">{context} · 가격: {unit} · 거래량: 제공 자료 수량</Typography>
        {!currency && <Typography variant="caption" color="text.secondary">이 차트 응답에는 가격 통화 정보가 없습니다. 선택 시장만으로 통화를 확정하거나 환산하지 않습니다.</Typography>}
        <Typography variant="body2" data-testid="candle-query-state">{status}</Typography>
        {dataUpdatedAt > 0 && <Typography variant="caption" color="text.secondary">
          마지막 조회 성공: <time dateTime={new Date(dataUpdatedAt).toISOString()}>{new Date(dataUpdatedAt).toLocaleString('ko-KR')}</time>
        </Typography>}
        {latest && <>
          <Typography variant="body2">확보된 봉 범위: {time(points.candleData[0].time)} ~ {time(latest.time)} · {points.candleData.length}개</Typography>
          <Typography variant="body2" data-testid="candle-latest">최신 봉: {time(latest.time)} · {values(latest, points.volumeData[points.volumeData.length - 1]?.value ?? 0)}</Typography>
        </>}
        <Typography variant="caption" color="text.secondary">확보된 유효 봉만 표시하며 선택 기간 전체의 자료 확보나 최신 시세를 보장하지 않습니다.</Typography>
        <Box component="details">
          <Box component="summary" sx={{ ...focusStyle, cursor: 'pointer', py: 1 }}>최근 봉 표 (최대 20개)</Box>
          <Typography variant="caption" color="text.secondary">봉 선택 버튼에서 위·아래 화살표와 Home·End로 이동합니다. 자료가 바뀌면 선택은 해제됩니다. 봉 시각은 기존 차트 표시와 같습니다.</Typography>
          <Typography variant="body2" data-testid="candle-selected" sx={{ my: 1 }}>
            {selected ? <>선택 봉: {time(selected.time)} · {values(selected, points.volumeData[offset + selectedIndex]?.value ?? 0)}</> : '선택 봉: 표에서 봉을 선택하세요.'}
          </Typography>
          <TableContainer tabIndex={0} aria-label="최근 봉 표 가로 스크롤" sx={{ ...focusStyle, maxWidth: '100%' }}>
            <Table size="small" aria-label={`${symbol || '미선택'} 최근 봉`} sx={{ '& th, & td': { whiteSpace: 'nowrap' } }}>
              <caption>최근 {recent.length}개 유효 봉 · 가격 {unit} · 거래량 제공 자료 수량</caption>
              <TableHead><TableRow>
                {['봉 시각·선택', '시가', '고가', '저가', '종가', '거래량'].map(label => <TableCell key={label} scope="col">{label}</TableCell>)}
              </TableRow></TableHead>
              <TableBody>
                {recent.map((point, index) => (
                  <TableRow key={JSON.stringify(point.time)} selected={selectedIndex === index}>
                    <TableCell component="th" scope="row">
                      <Button size="small" ref={element => { buttons.current[index] = element }}
                        aria-label={`${formatTime(point.time)} 봉 선택`} aria-pressed={selectedIndex === index}
                        sx={focusStyle} onClick={() => select(index)} onKeyDown={event => {
                          const next = event.key === 'Home' ? 0 : event.key === 'End' ? recent.length - 1
                            : event.key === 'ArrowUp' ? Math.max(0, index - 1) : event.key === 'ArrowDown' ? Math.min(recent.length - 1, index + 1) : null
                          if (next === null) return
                          event.preventDefault()
                          select(next, true)
                        }}>{time(point.time)}</Button>
                    </TableCell>
                    {[point.open, point.high, point.low, point.close, points.volumeData[offset + index]?.value ?? 0].map((value, column) => <TableCell key={column} align="right">{number(value)}</TableCell>)}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </Box>
      </Stack>
    </Box>
  )
}
